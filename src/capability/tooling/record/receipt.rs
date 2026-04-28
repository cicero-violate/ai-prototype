use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::capability::{CapabilityId, CapabilityRegistry, EvidenceProducer, EvidenceSubmission};
use crate::kernel::{
    Cause, ControlEvent, Decision, EventKind, Evidence, GateId, GateStatus, Phase, TLog,
};

use super::artifact::{persisted_execution_effect_is_valid, ToolExecutionRecord};
use super::hash::{capability_from_u64, sync_dir, tool_effect_kind_from_u64, tool_effect_output_hash};
use super::process::SandboxProcessReceipt;
use super::types::{
    Effect, ToolEffectKind, ToolSandboxError, PROCESS_EFFECT_RECEIPT_RECORD,
    PROCESS_EFFECT_RECEIPT_SCHEMA_VERSION, TOOL_EFFECT_RECEIPT_RECORD,
    TOOL_EFFECT_RECEIPT_SCHEMA_VERSION,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolEffectReceipt {
    pub capability: CapabilityId,
    pub registry_policy_hash: u64,
    pub request_hash: u64,
    pub receipt_hash: u64,
    pub effect_hash: u64,
    pub effect: Effect,
    pub event_seq: u64,
    pub event_hash: u64,
    pub artifact_id: u64,
    pub artifact_receipt_hash: u64,
    pub artifact_path_hash: u64,
    pub artifact_content_hash: u64,
    pub artifact_bytes: u64,
    pub sandbox_root_hash: u64,
}

impl ToolEffectReceipt {
    pub fn from_persisted_event(
        record: &ToolExecutionRecord,
        event: &ControlEvent,
    ) -> Option<Self> {
        if !persisted_execution_effect_is_valid(record, event) {
            return None;
        }

        let effect_hash = tool_effect_output_hash(event.state_before.packet, event.state_after.packet);
        let effect = Effect::artifact(
            effect_hash,
            record.receipt.artifact_path_hash,
            record.receipt.artifact_content_hash,
            record.receipt.artifact_bytes,
            record.receipt.sandbox_root_hash,
        );
        Some(Self {
            capability: record.request.capability,
            registry_policy_hash: record.request.registry_policy_hash,
            request_hash: record.request.contract_hash(),
            receipt_hash: record.receipt.receipt_hash,
            effect_hash,
            effect,
            event_seq: event.seq,
            event_hash: event.self_hash,
            artifact_id: event.state_after.packet.artifact_id,
            artifact_receipt_hash: event.state_after.packet.artifact_receipt_hash,
            artifact_path_hash: record.receipt.artifact_path_hash,
            artifact_content_hash: record.receipt.artifact_content_hash,
            artifact_bytes: record.receipt.artifact_bytes,
            sandbox_root_hash: record.receipt.sandbox_root_hash,
        })
    }

    pub fn is_valid(self) -> bool {
        self.capability == CapabilityId::Tooling
            && self.registry_policy_hash != 0
            && self.request_hash != 0
            && self.receipt_hash != 0
            && self.effect_hash != 0
            && self.effect.is_valid()
            && self.effect.kind == ToolEffectKind::Artifact
            && self.effect.digest == self.effect_hash
            && self.effect
                == Effect::artifact(
                    self.effect_hash,
                    self.artifact_path_hash,
                    self.artifact_content_hash,
                    self.artifact_bytes,
                    self.sandbox_root_hash,
                )
            && self.event_seq != 0
            && self.event_hash != 0
            && self.artifact_id != 0
            && self.artifact_receipt_hash != 0
    }

    pub fn is_sandbox_artifact_bound(self) -> bool {
        self.effect.kind == ToolEffectKind::Artifact
            && self.artifact_path_hash != 0
            && self.artifact_content_hash != 0
            && self.artifact_bytes != 0
            && self.sandbox_root_hash != 0
    }

    pub fn replay_verified(self, tlog: &TLog) -> bool {
        self.is_valid() && tlog.iter().any(|event| self.matches_event(event))
    }

    pub fn replay_verified_with_registry(self, tlog: &TLog, registry: CapabilityRegistry) -> bool {
        self.registry_policy_hash == registry.policy_hash()
            && tlog.iter().any(|event| self.matches_event(event))
    }

    pub fn matches_event(self, event: &ControlEvent) -> bool {
        event.seq == self.event_seq
            && event.self_hash == self.event_hash
            && self.capability == CapabilityId::Tooling
            && self.registry_policy_hash != 0
            && event.capability_registry_projection.policy_hash == self.registry_policy_hash
            && event.state_after.packet.artifact_id == self.artifact_id
            && event.state_after.packet.artifact_receipt_hash == self.artifact_receipt_hash
            && self.effect.digest == self.effect_hash
            && tool_effect_output_hash(event.state_before.packet, event.state_after.packet)
                == self.effect_hash
    }
}

impl EvidenceProducer for ToolExecutionRecord {
    type Record = ToolExecutionRecord;

    fn record(&self) -> &Self::Record {
        self
    }

    fn submission(&self) -> EvidenceSubmission {
        ToolExecutionRecord::submission(self)
    }
}

pub fn append_tool_effect_receipt_ndjson(
    path: impl AsRef<Path>,
    receipt: &ToolEffectReceipt,
) -> Result<(), ToolSandboxError> {
    if !receipt.is_valid() {
        return Err(ToolSandboxError::InvalidToolReceipt);
    }

    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|_| ToolSandboxError::SandboxIo)?;
        }
    }

    {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|_| ToolSandboxError::SandboxIo)?;
        writeln!(file, "{}", encode_tool_effect_receipt_ndjson(*receipt))
            .map_err(|_| ToolSandboxError::SandboxIo)?;
        file.sync_all().map_err(|_| ToolSandboxError::SandboxIo)?;
    }

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            sync_dir(parent)?;
        }
    }

    Ok(())
}

pub fn load_tool_effect_receipts_ndjson(
    path: impl AsRef<Path>,
) -> Result<Vec<ToolEffectReceipt>, ToolSandboxError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path).map_err(|_| ToolSandboxError::SandboxIo)?;
    let reader = BufReader::new(file);
    let mut receipts = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|_| ToolSandboxError::SandboxIo)?;
        if line.trim().is_empty() {
            continue;
        }
        receipts.push(decode_tool_effect_receipt_ndjson(&line)?);
    }

    Ok(receipts)
}

pub fn verify_tool_effect_receipts(
    tlog: &TLog,
    receipts: &[ToolEffectReceipt],
) -> Result<usize, ToolSandboxError> {
    for receipt in receipts {
        if !receipt.replay_verified(tlog) {
            return Err(ToolSandboxError::InvalidReplay);
        }
    }

    Ok(receipts.len())
}

pub fn encode_tool_effect_receipt_ndjson(receipt: ToolEffectReceipt) -> String {
    let fields = [
        TOOL_EFFECT_RECEIPT_SCHEMA_VERSION,
        TOOL_EFFECT_RECEIPT_RECORD,
        receipt.capability as u64,
        receipt.registry_policy_hash,
        receipt.request_hash,
        receipt.receipt_hash,
        receipt.effect_hash,
        receipt.effect.kind as u64,
        receipt.effect.digest,
        receipt.effect.metadata,
        receipt.event_seq,
        receipt.event_hash,
        receipt.artifact_id,
        receipt.artifact_receipt_hash,
        receipt.artifact_path_hash,
        receipt.artifact_content_hash,
        receipt.artifact_bytes,
        receipt.sandbox_root_hash,
    ];
    let body = fields
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{body}]")
}

pub fn decode_tool_effect_receipt_ndjson(
    line: &str,
) -> Result<ToolEffectReceipt, ToolSandboxError> {
    let trimmed = line.trim();
    let body = trimmed
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .ok_or(ToolSandboxError::InvalidToolReceiptRecord)?;
    let fields = if body.trim().is_empty() {
        Vec::new()
    } else {
        body.split(',')
            .map(|raw| {
                raw.trim()
                    .parse::<u64>()
                    .map_err(|_| ToolSandboxError::InvalidToolReceiptRecord)
            })
            .collect::<Result<Vec<_>, _>>()?
    };

    if fields.len() != 18
        || fields[0] != TOOL_EFFECT_RECEIPT_SCHEMA_VERSION
        || fields[1] != TOOL_EFFECT_RECEIPT_RECORD
    {
        return Err(ToolSandboxError::InvalidToolReceiptRecord);
    }

    let effect = Effect {
        kind: tool_effect_kind_from_u64(fields[7])?,
        digest: fields[8],
        metadata: fields[9],
    };

    let receipt = ToolEffectReceipt {
        capability: capability_from_u64(fields[2])?,
        registry_policy_hash: fields[3],
        request_hash: fields[4],
        receipt_hash: fields[5],
        effect_hash: fields[6],
        effect,
        event_seq: fields[10],
        event_hash: fields[11],
        artifact_id: fields[12],
        artifact_receipt_hash: fields[13],
        artifact_path_hash: fields[14],
        artifact_content_hash: fields[15],
        artifact_bytes: fields[16],
        sandbox_root_hash: fields[17],
    };

    if !receipt.is_valid() {
        return Err(ToolSandboxError::InvalidToolReceipt);
    }

    Ok(receipt)
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessEffectReceipt {
    pub capability: CapabilityId,
    pub registry_policy_hash: u64,
    pub request_hash: u64,
    pub receipt_hash: u64,
    pub effect_hash: u64,
    pub effect: Effect,
    pub event_seq: u64,
    pub event_hash: u64,
}

impl ProcessEffectReceipt {
    pub fn from_persisted_event(
        receipt: &SandboxProcessReceipt,
        event: &ControlEvent,
    ) -> Option<Self> {
        if !persisted_process_execution_effect_is_valid(receipt, event) {
            return None;
        }

        Some(Self {
            capability: CapabilityId::Tooling,
            registry_policy_hash: receipt.registry_policy_hash,
            request_hash: receipt.request_hash,
            receipt_hash: receipt.receipt_hash,
            effect_hash: receipt.effect.contract_hash(),
            effect: receipt.effect,
            event_seq: event.seq,
            event_hash: event.self_hash,
        })
    }

    pub fn is_valid(self) -> bool {
        self.capability == CapabilityId::Tooling
            && self.registry_policy_hash != 0
            && self.request_hash != 0
            && self.receipt_hash != 0
            && self.effect_hash != 0
            && self.effect.is_valid()
            && self.effect.kind == ToolEffectKind::Process
            && self.effect_hash == self.effect.contract_hash()
            && self.event_seq != 0
            && self.event_hash != 0
    }

    pub fn replay_verified(self, tlog: &TLog) -> bool {
        self.is_valid() && tlog.iter().any(|event| self.matches_event(event))
    }

    pub fn replay_verified_with_registry(self, tlog: &TLog, registry: CapabilityRegistry) -> bool {
        self.registry_policy_hash == registry.policy_hash()
            && tlog.iter().any(|event| self.matches_event(event))
    }

    pub fn matches_event(self, event: &ControlEvent) -> bool {
        event.seq == self.event_seq
            && event.self_hash == self.event_hash
            && self.capability == CapabilityId::Tooling
            && event.capability_registry_projection.policy_hash == self.registry_policy_hash
            && event.from == Phase::Execute
            && event.to == Phase::Execute
            && event.kind == EventKind::Persisted
            && event.cause == Cause::EvidenceSubmitted
            && event.evidence == Evidence::ExecutionReceipt
            && event.affected_gate == Some(GateId::Execution)
            && self.effect.kind == ToolEffectKind::Process
            && self.effect_hash == self.effect.contract_hash()
    }
}

pub(crate) fn persisted_process_execution_effect_is_valid(
    receipt: &SandboxProcessReceipt,
    event: &ControlEvent,
) -> bool {
    if !receipt.is_contract_valid()
        || event.from != Phase::Execute
        || event.to != Phase::Execute
        || event.kind != EventKind::Persisted
        || event.cause != Cause::EvidenceSubmitted
        || event.evidence != Evidence::ExecutionReceipt
        || event.failure.is_some()
        || event.recovery_action.is_some()
        || event.affected_gate != Some(GateId::Execution)
        || event.capability_registry_projection.policy_hash != receipt.registry_policy_hash
    {
        return false;
    }

    let passed = receipt.is_success();
    let expected_decision = if passed {
        Decision::Continue
    } else {
        Decision::Block
    };

    if event.decision != expected_decision {
        return false;
    }

    let mut expected = event.state_before;
    expected.apply_evidence(GateId::Execution, Evidence::ExecutionReceipt, passed);

    event.state_after == expected
        && event.state_after.gates.execution.evidence == Evidence::ExecutionReceipt
        && event.state_after.gates.execution.status
            == if passed { GateStatus::Pass } else { GateStatus::Fail }
}

pub fn append_process_effect_receipt_ndjson(
    path: impl AsRef<Path>,
    receipt: &ProcessEffectReceipt,
) -> Result<(), ToolSandboxError> {
    if !receipt.is_valid() {
        return Err(ToolSandboxError::InvalidToolReceipt);
    }

    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|_| ToolSandboxError::SandboxIo)?;
        }
    }

    {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|_| ToolSandboxError::SandboxIo)?;
        writeln!(file, "{}", encode_process_effect_receipt_ndjson(*receipt))
            .map_err(|_| ToolSandboxError::SandboxIo)?;
        file.sync_all().map_err(|_| ToolSandboxError::SandboxIo)?;
    }

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            sync_dir(parent)?;
        }
    }

    Ok(())
}

pub fn load_process_effect_receipts_ndjson(
    path: impl AsRef<Path>,
) -> Result<Vec<ProcessEffectReceipt>, ToolSandboxError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path).map_err(|_| ToolSandboxError::SandboxIo)?;
    let reader = BufReader::new(file);
    let mut receipts = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|_| ToolSandboxError::SandboxIo)?;
        if line.trim().is_empty() {
            continue;
        }
        receipts.push(decode_process_effect_receipt_ndjson(&line)?);
    }

    Ok(receipts)
}

pub fn verify_process_effect_receipts(
    tlog: &TLog,
    receipts: &[ProcessEffectReceipt],
) -> Result<usize, ToolSandboxError> {
    for receipt in receipts {
        if !receipt.replay_verified(tlog) {
            return Err(ToolSandboxError::InvalidReplay);
        }
    }

    Ok(receipts.len())
}

pub fn encode_process_effect_receipt_ndjson(receipt: ProcessEffectReceipt) -> String {
    let fields = [
        PROCESS_EFFECT_RECEIPT_SCHEMA_VERSION,
        PROCESS_EFFECT_RECEIPT_RECORD,
        receipt.capability as u64,
        receipt.registry_policy_hash,
        receipt.request_hash,
        receipt.receipt_hash,
        receipt.effect_hash,
        receipt.effect.kind as u64,
        receipt.effect.digest,
        receipt.effect.metadata,
        receipt.event_seq,
        receipt.event_hash,
    ];
    let body = fields
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{body}]")
}

pub fn decode_process_effect_receipt_ndjson(
    line: &str,
) -> Result<ProcessEffectReceipt, ToolSandboxError> {
    let trimmed = line.trim();
    let body = trimmed
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .ok_or(ToolSandboxError::InvalidToolReceiptRecord)?;
    let fields = if body.trim().is_empty() {
        Vec::new()
    } else {
        body.split(',')
            .map(|raw| {
                raw.trim()
                    .parse::<u64>()
                    .map_err(|_| ToolSandboxError::InvalidToolReceiptRecord)
            })
            .collect::<Result<Vec<_>, _>>()?
    };

    if fields.len() != 12
        || fields[0] != PROCESS_EFFECT_RECEIPT_SCHEMA_VERSION
        || fields[1] != PROCESS_EFFECT_RECEIPT_RECORD
    {
        return Err(ToolSandboxError::InvalidToolReceiptRecord);
    }

    let effect = Effect {
        kind: tool_effect_kind_from_u64(fields[7])?,
        digest: fields[8],
        metadata: fields[9],
    };

    let receipt = ProcessEffectReceipt {
        capability: capability_from_u64(fields[2])?,
        registry_policy_hash: fields[3],
        request_hash: fields[4],
        receipt_hash: fields[5],
        effect_hash: fields[6],
        effect,
        event_seq: fields[10],
        event_hash: fields[11],
    };

    if !receipt.is_valid() {
        return Err(ToolSandboxError::InvalidToolReceipt);
    }

    Ok(receipt)
}