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
use crate::kernel::{ControlEvent, RuntimeConfig, State, TLog};
use crate::recovery::{
    verify_receipt_chain, ReceiptChainEntry, ReceiptReplayFailure, ReceiptReplayReport,
};
use crate::runtime::verify_tlog;

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

    pub fn matches_receipt(&self, receipt: ApiTransportReceipt) -> bool {
        self.request_id == receipt.request_id
            && self.payload_hash == receipt.payload_hash
            && self.envelope.command_id == receipt.command_id
            && self.envelope.command_hash == receipt.command_hash
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
            && self.receipt_hash == self.expected_receipt_hash()
    }

    fn expected_receipt_hash(&self) -> u64 {
        api_transport_receipt_hash(
            self.request_id,
            self.payload_hash,
            self.command_id,
            self.command_hash,
            self.event_hash,
        )
    }

    pub fn matches_event(&self, event: &ControlEvent) -> bool {
        event.api_command_id == self.command_id && event.api_command_hash == self.command_hash
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
        self.receipts
            .iter()
            .copied()
            .find(|receipt| frame.matches_receipt(*receipt))
    }

    fn receipt_for_request_id(&self, request_id: u64) -> Option<ApiTransportReceipt> {
        self.receipts
            .iter()
            .copied()
            .find(|receipt| receipt.request_id == request_id)
    }

    pub fn contains_request_id(&self, request_id: u64) -> bool {
        self.receipt_for_request_id(request_id).is_some()
    }

    pub fn has_conflicting_request(&self, frame: &ApiTransportFrame) -> bool {
        self.receipt_for_request_id(frame.request_id)
            .is_some_and(|receipt| receipt.payload_hash != frame.payload_hash)
    }

    fn push_receipt(&mut self, receipt: ApiTransportReceipt) -> Result<(), CanonError> {
        if !receipt.is_contract_valid() || self.contains_request_id(receipt.request_id) {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiTransportSession {
    state: State,
    tlog: TLog,
    cfg: RuntimeConfig,
    command_ledger: CommandLedger,
    transport_ledger: ApiTransportLedger,
}

impl ApiTransportSession {
    pub fn new(state: State, cfg: RuntimeConfig) -> Self {
        Self {
            state,
            tlog: TLog::default(),
            cfg,
            command_ledger: CommandLedger::default(),
            transport_ledger: ApiTransportLedger::default(),
        }
    }

    pub fn from_parts(
        state: State,
        tlog: TLog,
        cfg: RuntimeConfig,
        command_ledger: CommandLedger,
        transport_ledger: ApiTransportLedger,
    ) -> Result<Self, CanonError> {
        Self::verify_parts(&tlog, &command_ledger, &transport_ledger)?;
        Ok(Self {
            state,
            tlog,
            cfg,
            command_ledger,
            transport_ledger,
        })
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn tlog(&self) -> &TLog {
        &self.tlog
    }

    pub fn command_ledger(&self) -> &CommandLedger {
        &self.command_ledger
    }

    pub fn transport_ledger(&self) -> &ApiTransportLedger {
        &self.transport_ledger
    }

    pub fn handle_frame(
        &mut self,
        frame: ApiTransportFrame,
    ) -> Result<ApiTransportResponse, CanonError> {
        handle_transport_frame_once(
            &mut self.state,
            &mut self.tlog,
            self.cfg,
            &mut self.command_ledger,
            &mut self.transport_ledger,
            frame,
        )
    }

    pub fn verify(&self) -> Result<(), CanonError> {
        Self::verify_parts(&self.tlog, &self.command_ledger, &self.transport_ledger)
    }

    fn verify_parts(
        tlog: &TLog,
        command_ledger: &CommandLedger,
        transport_ledger: &ApiTransportLedger,
    ) -> Result<(), CanonError> {
        verify_tlog(tlog)?;
        verify_command_ledger_matches_tlog(tlog, command_ledger)?;
        verify_api_transport_receipts(tlog, transport_ledger.receipts())
    }

    pub fn into_parts(
        self,
    ) -> (
        State,
        TLog,
        RuntimeConfig,
        CommandLedger,
        ApiTransportLedger,
    ) {
        (
            self.state,
            self.tlog,
            self.cfg,
            self.command_ledger,
            self.transport_ledger,
        )
    }
}

fn verify_command_ledger_matches_tlog(
    tlog: &TLog,
    command_ledger: &CommandLedger,
) -> Result<(), CanonError> {
    let reconstructed = CommandLedger::reconstruct_from_tlog(tlog)?;
    if &reconstructed == command_ledger {
        Ok(())
    } else {
        Err(CanonError::InvalidReplay)
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
        let event = *event_for_transport_receipt(tlog, receipt)?;
        return Ok(ApiTransportResponse {
            request_id: receipt.request_id,
            command_id: receipt.command_id,
            command_hash: receipt.command_hash,
            event_hash: receipt.event_hash,
            disposition: ApiTransportDisposition::Replayed,
            control: ControlEventResponse { event },
        });
    }

    if transport_ledger.contains_request_id(frame.request_id) {
        return Err(CanonError::InvalidReplay);
    }

    let request_id = frame.request_id;
    let command_id = frame.envelope.command_id;
    let command_hash = frame.envelope.command_hash;
    let replayed = command_ledger
        .replayed_event(&frame.envelope, tlog)
        .is_some();
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

fn event_for_transport_receipt(
    tlog: &TLog,
    receipt: ApiTransportReceipt,
) -> Result<&ControlEvent, CanonError> {
    let event = tlog
        .iter()
        .find(|event| event.self_hash == receipt.event_hash)
        .ok_or(CanonError::InvalidReplay)?;
    receipt
        .matches_event(event)
        .then_some(event)
        .ok_or(CanonError::InvalidReplay)
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

pub fn decode_api_transport_receipt_ndjson(line: &str) -> Result<ApiTransportReceipt, CanonError> {
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
    verify_api_transport_receipt_chain(tlog, receipts)
        .map_err(api_receipt_replay_error_to_canon)?;
    Ok(())
}

pub fn verify_api_transport_receipt_chain(
    tlog: &TLog,
    receipts: &[ApiTransportReceipt],
) -> Result<ReceiptReplayReport, ReceiptReplayFailure> {
    verify_api_transport_receipt_chain_with_expected_count(tlog, receipts, receipts.len())
}

pub fn verify_api_transport_receipt_chain_with_expected_count(
    tlog: &TLog,
    receipts: &[ApiTransportReceipt],
    expected_receipt_count: usize,
) -> Result<ReceiptReplayReport, ReceiptReplayFailure> {
    let expected_last_seq = expected_receipt_count as u64;
    let run_id = api_transport_receipt_run_id(tlog);
    let mut prev_chain_hash = 0;
    let mut chain_entries = Vec::new();
    let mut last_event_index = None;

    for receipt in receipts {
        if !receipt.is_contract_valid() {
            return Err(ReceiptReplayFailure::ForgedReceipt);
        }

        let event_index = tlog
            .iter()
            .position(|event| event.self_hash == receipt.event_hash && receipt.matches_event(event))
            .ok_or(ReceiptReplayFailure::StaleReceipt)?;

        if last_event_index == Some(event_index) {
            return Err(ReceiptReplayFailure::DuplicatedReceipt);
        }

        if last_event_index.is_some_and(|last| event_index < last) {
            return Err(ReceiptReplayFailure::ReorderedReceipt);
        }

        let entry = ReceiptChainEntry::new(
            chain_entries.len() as u64 + 1,
            run_id,
            receipt.receipt_hash,
            prev_chain_hash,
        );
        prev_chain_hash = entry.receipt_hash;
        chain_entries.push(entry);
        last_event_index = Some(event_index);
    }

    verify_receipt_chain(run_id, expected_last_seq, &chain_entries)
}

pub fn api_transport_receipt_replay_classification(
    tlog: &TLog,
    receipts: &[ApiTransportReceipt],
) -> Option<&'static str> {
    verify_api_transport_receipt_chain(tlog, receipts)
        .err()
        .map(ReceiptReplayFailure::classification)
}

pub fn api_transport_receipt_replay_classification_with_expected_count(
    tlog: &TLog,
    receipts: &[ApiTransportReceipt],
    expected_receipt_count: usize,
) -> Option<&'static str> {
    verify_api_transport_receipt_chain_with_expected_count(tlog, receipts, expected_receipt_count)
        .err()
        .map(ReceiptReplayFailure::classification)
}

fn api_receipt_replay_error_to_canon(error: ReceiptReplayFailure) -> CanonError {
    match error {
        ReceiptReplayFailure::ForgedReceipt | ReceiptReplayFailure::DuplicatedReceipt => {
            CanonError::InvalidApiCommand
        }
        ReceiptReplayFailure::MissingReceipt
        | ReceiptReplayFailure::ReorderedReceipt
        | ReceiptReplayFailure::StaleReceipt => CanonError::InvalidReplay,
    }
}

fn api_transport_receipt_run_id(tlog: &TLog) -> u64 {
    tlog.last().map(|event| event.self_hash).unwrap_or(0).max(1)
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
