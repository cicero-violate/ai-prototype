//! Deterministic transport boundary for external API ingress.
//!
//! This module defines the request-frame contract that an HTTP/gRPC adapter can
//! validate before command envelopes are allowed to mutate runtime state.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::api::protocol::{CommandEnvelope, CommandLedger, ControlEventResponse};
use crate::api::routes::handle_envelope_once;
use crate::error::CanonError;
use crate::kernel::{RuntimeConfig, State, TLog};

pub const API_TRANSPORT_SCHEMA_VERSION: u64 = 1;
pub const API_TRANSPORT_ROUTE_COMMAND: u64 = 1;
pub const API_TRANSPORT_RECEIPT_SCHEMA_VERSION: u64 = 1;
pub const API_TRANSPORT_RECEIPT_RECORD: u64 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiTransportFrame {
    pub schema_version: u64,
    pub route_id: u64,
    pub request_id: u64,
    pub payload_hash: u64,
    pub envelope: CommandEnvelope,
}

impl ApiTransportFrame {
    pub fn new(request_id: u64, envelope: CommandEnvelope) -> Self {
        let payload_hash = transport_frame_hash(
            API_TRANSPORT_SCHEMA_VERSION,
            API_TRANSPORT_ROUTE_COMMAND,
            request_id,
            &envelope,
        );
        Self {
            schema_version: API_TRANSPORT_SCHEMA_VERSION,
            route_id: API_TRANSPORT_ROUTE_COMMAND,
            request_id,
            payload_hash,
            envelope,
        }
    }

    pub fn is_contract_valid(&self) -> bool {
        self.schema_version == API_TRANSPORT_SCHEMA_VERSION
            && self.route_id == API_TRANSPORT_ROUTE_COMMAND
            && self.request_id != 0
            && self.envelope.is_contract_valid()
            && self.payload_hash
                == transport_frame_hash(
                    self.schema_version,
                    self.route_id,
                    self.request_id,
                    &self.envelope,
                )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApiTransportDisposition {
    Accepted,
    Replayed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiTransportResponse {
    pub request_id: u64,
    pub command_id: u64,
    pub command_hash: u64,
    pub event_hash: u64,
    pub disposition: ApiTransportDisposition,
    pub control: ControlEventResponse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApiTransportReceipt {
    pub request_id: u64,
    pub payload_hash: u64,
    pub command_id: u64,
    pub command_hash: u64,
    pub event_hash: u64,
    pub receipt_hash: u64,
}

impl ApiTransportReceipt {
    pub fn new(
        request_id: u64,
        payload_hash: u64,
        command_id: u64,
        command_hash: u64,
        event_hash: u64,
    ) -> Self {
        let receipt_hash = api_transport_receipt_hash(
            request_id,
            payload_hash,
            command_id,
            command_hash,
            event_hash,
        );
        Self {
            request_id,
            payload_hash,
            command_id,
            command_hash,
            event_hash,
            receipt_hash,
        }
    }

    pub fn is_contract_valid(&self) -> bool {
        self.request_id != 0
            && self.payload_hash != 0
            && self.command_id != 0
            && self.command_hash != 0
            && self.event_hash != 0
            && self.receipt_hash
                == api_transport_receipt_hash(
                    self.request_id,
                    self.payload_hash,
                    self.command_id,
                    self.command_hash,
                    self.event_hash,
                )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ApiTransportLedger {
    receipts: Vec<ApiTransportReceipt>,
}

impl ApiTransportLedger {
    pub fn from_receipts(receipts: Vec<ApiTransportReceipt>) -> Result<Self, CanonError> {
        let mut ledger = Self::default();
        for receipt in receipts {
            ledger.push_receipt(receipt)?;
        }
        Ok(ledger)
    }

    pub fn len(&self) -> usize {
        self.receipts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.receipts.is_empty()
    }

    pub fn receipts(&self) -> &[ApiTransportReceipt] {
        &self.receipts
    }

    pub fn receipt_for(&self, frame: &ApiTransportFrame) -> Option<ApiTransportReceipt> {
        self.receipts.iter().copied().find(|receipt| {
            receipt.request_id == frame.request_id && receipt.payload_hash == frame.payload_hash
        })
    }

    pub fn has_conflicting_request(&self, frame: &ApiTransportFrame) -> bool {
        self.receipts.iter().any(|receipt| {
            receipt.request_id == frame.request_id && receipt.payload_hash != frame.payload_hash
        })
    }

    fn push_receipt(&mut self, receipt: ApiTransportReceipt) -> Result<(), CanonError> {
        if !receipt.is_contract_valid()
            || self
                .receipts
                .iter()
                .any(|existing| existing.request_id == receipt.request_id)
        {
            return Err(CanonError::InvalidApiCommand);
        }
        self.receipts.push(receipt);
        Ok(())
    }

    fn push_response(
        &mut self,
        frame: &ApiTransportFrame,
        response: &ApiTransportResponse,
    ) -> Result<(), CanonError> {
        if self.receipt_for(frame).is_none() {
            self.push_receipt(ApiTransportReceipt::new(
                frame.request_id,
                frame.payload_hash,
                response.command_id,
                response.command_hash,
                response.event_hash,
            ))?;
        }
        Ok(())
    }
}

pub fn handle_transport_frame_once(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    command_ledger: &mut CommandLedger,
    transport_ledger: &mut ApiTransportLedger,
    frame: ApiTransportFrame,
) -> Result<ApiTransportResponse, CanonError> {
    if !frame.is_contract_valid() || transport_ledger.has_conflicting_request(&frame) {
        return Err(CanonError::InvalidApiCommand);
    }

    if let Some(receipt) = transport_ledger.receipt_for(&frame) {
        let event = tlog
            .iter()
            .copied()
            .find(|event| event.self_hash == receipt.event_hash)
            .ok_or(CanonError::InvalidReplay)?;
        return Ok(ApiTransportResponse {
            request_id: receipt.request_id,
            command_id: receipt.command_id,
            command_hash: receipt.command_hash,
            event_hash: receipt.event_hash,
            disposition: ApiTransportDisposition::Replayed,
            control: ControlEventResponse { event },
        });
    }

    let request_id = frame.request_id;
    let command_id = frame.envelope.command_id;
    let command_hash = frame.envelope.command_hash;
    let replayed = command_ledger.replayed_event(&frame.envelope, tlog).is_some();
    let control = handle_envelope_once(state, tlog, cfg, command_ledger, frame.envelope.clone())?;

    let response = ApiTransportResponse {
        request_id,
        command_id,
        command_hash,
        event_hash: control.event.self_hash,
        disposition: if replayed {
            ApiTransportDisposition::Replayed
        } else {
            ApiTransportDisposition::Accepted
        },
        control,
    };
    transport_ledger.push_response(&frame, &response)?;
    Ok(response)
}

pub fn encode_api_transport_receipt_ndjson(receipt: ApiTransportReceipt) -> String {
    format!(
        "[{},{},{},{},{},{},{},{}]",
        API_TRANSPORT_RECEIPT_SCHEMA_VERSION,
        API_TRANSPORT_RECEIPT_RECORD,
        receipt.request_id,
        receipt.payload_hash,
        receipt.command_id,
        receipt.command_hash,
        receipt.event_hash,
        receipt.receipt_hash
    )
}

pub fn decode_api_transport_receipt_ndjson(
    line: &str,
) -> Result<ApiTransportReceipt, CanonError> {
    let trimmed = line.trim();
    let body = trimmed
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or(CanonError::InvalidTlogRecord)?;
    let mut fields = Vec::new();
    if !body.trim().is_empty() {
        for raw in body.split(',') {
            fields.push(
                raw.trim()
                    .parse::<u64>()
                    .map_err(|_| CanonError::InvalidTlogRecord)?,
            );
        }
    }
    if fields.len() != 8
        || fields[0] != API_TRANSPORT_RECEIPT_SCHEMA_VERSION
        || fields[1] != API_TRANSPORT_RECEIPT_RECORD
    {
        return Err(CanonError::InvalidTlogRecord);
    }
    let receipt = ApiTransportReceipt {
        request_id: fields[2],
        payload_hash: fields[3],
        command_id: fields[4],
        command_hash: fields[5],
        event_hash: fields[6],
        receipt_hash: fields[7],
    };
    if !receipt.is_contract_valid() {
        return Err(CanonError::InvalidTlogRecord);
    }
    Ok(receipt)
}

pub fn append_api_transport_receipt_ndjson(
    path: impl AsRef<Path>,
    receipt: ApiTransportReceipt,
) -> Result<(), CanonError> {
    if !receipt.is_contract_valid() {
        return Err(CanonError::InvalidTlogRecord);
    }
    let path = path.as_ref();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|_| CanonError::TlogIo)?;
    writeln!(file, "{}", encode_api_transport_receipt_ndjson(receipt))
        .map_err(|_| CanonError::TlogIo)?;
    file.sync_all().map_err(|_| CanonError::TlogIo)
}

pub fn load_api_transport_receipts_ndjson(
    path: impl AsRef<Path>,
) -> Result<Vec<ApiTransportReceipt>, CanonError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = File::open(path).map_err(|_| CanonError::TlogIo)?;
    let reader = BufReader::new(file);
    let mut receipts = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|_| CanonError::TlogIo)?;
        if line.trim().is_empty() {
            continue;
        }
        receipts.push(decode_api_transport_receipt_ndjson(&line)?);
    }
    Ok(receipts)
}

pub fn load_api_transport_ledger_ndjson(
    path: impl AsRef<Path>,
) -> Result<ApiTransportLedger, CanonError> {
    ApiTransportLedger::from_receipts(load_api_transport_receipts_ndjson(path)?)
}

pub fn verify_api_transport_receipts(
    tlog: &TLog,
    receipts: &[ApiTransportReceipt],
) -> Result<(), CanonError> {
    let mut seen_request_ids = Vec::new();
    for receipt in receipts {
        if !receipt.is_contract_valid() || seen_request_ids.contains(&receipt.request_id) {
            return Err(CanonError::InvalidApiCommand);
        }
        let event = tlog
            .iter()
            .find(|event| event.self_hash == receipt.event_hash)
            .ok_or(CanonError::InvalidReplay)?;
        if event.api_command_id != receipt.command_id
            || event.api_command_hash != receipt.command_hash
        {
            return Err(CanonError::InvalidReplay);
        }
        seen_request_ids.push(receipt.request_id);
    }
    Ok(())
}

fn transport_frame_hash(
    schema_version: u64,
    route_id: u64,
    request_id: u64,
    envelope: &CommandEnvelope,
) -> u64 {
    let mut h = 0x7472_616e_7370_6f72u64;
    h ^= schema_version;
    h = h.wrapping_mul(0x100000001b3);
    h ^= route_id;
    h = h.wrapping_mul(0x100000001b3);
    h ^= request_id;
    h = h.wrapping_mul(0x100000001b3);
    h ^= envelope.command_id;
    h = h.wrapping_mul(0x100000001b3);
    h ^= envelope.command_hash;
    h.wrapping_mul(0x100000001b3).max(1)
}

fn api_transport_receipt_hash(
    request_id: u64,
    payload_hash: u64,
    command_id: u64,
    command_hash: u64,
    event_hash: u64,
) -> u64 {
    let mut h = 0x6170_695f_7472_6374u64;
    h ^= API_TRANSPORT_RECEIPT_SCHEMA_VERSION;
    h = h.wrapping_mul(0x100000001b3);
    h ^= API_TRANSPORT_RECEIPT_RECORD;
    h = h.wrapping_mul(0x100000001b3);
    h ^= request_id;
    h = h.wrapping_mul(0x100000001b3);
    h ^= payload_hash;
    h = h.wrapping_mul(0x100000001b3);
    h ^= command_id;
    h = h.wrapping_mul(0x100000001b3);
    h ^= command_hash;
    h = h.wrapping_mul(0x100000001b3);
    h ^= event_hash;
    h.wrapping_mul(0x100000001b3).max(1)
}