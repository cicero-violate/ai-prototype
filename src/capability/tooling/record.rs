//! Durable tool execution payload owned by the tooling capability.
//!
//! Tooling is the first capability boundary that must stop being a synthetic
//! record facade. This module now models the live execution loop:
//!
//! ```text
//! ToolRequest -> registry decision -> sandbox artifact write -> ToolReceipt
//!     -> EvidenceSubmission -> TLog event -> durable ToolEffectReceipt replay
//! ```
//!
//! The kernel still sees only evidence, gates, packet effects, and hashes.
//! External work is isolated here behind deny-by-default capability routing and
//! an append-only receipt sidecar.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::capability::{
    CapabilityId, CapabilityRegistry, EvidenceProducer, EvidenceSubmission, PacketEffect,
};
use crate::kernel::{
    mix, Cause, ControlEvent, Decision, EventKind, Evidence, GateId, GateStatus, Packet, Phase,
    TLog,
};

pub const TOOL_EFFECT_RECEIPT_SCHEMA_VERSION: u64 = 1;
pub const TOOL_EFFECT_RECEIPT_RECORD: u64 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolDecision {
    Succeeded,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ToolKind {
    DeterministicArtifact = 1,
    Noop = 2,
    Denied = 3,
    SandboxFile = 4,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolSandboxError {
    SandboxIo,
    PathEscapesSandbox,
    ArtifactTooLarge,
    InvalidToolReceipt,
    InvalidToolReceiptRecord,
    InvalidReplay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolRequest {
    pub capability: CapabilityId,
    pub registry_policy_hash: u64,
    pub tool_kind: ToolKind,
    pub objective_id: u64,
    pub task_id: u64,
    pub command_hash: u64,
    pub input_hash: u64,
    pub requested_effect: PacketEffect,
}

impl ToolRequest {
    pub fn from_packet(packet: Packet) -> Self {
        Self::from_packet_with_registry(packet, CapabilityRegistry::canonical())
    }

    pub fn from_packet_with_registry(packet: Packet, registry: CapabilityRegistry) -> Self {
        Self::from_packet_with_registry_and_kind(packet, registry, ToolKind::DeterministicArtifact)
    }

    pub fn from_packet_with_registry_and_kind(
        packet: Packet,
        registry: CapabilityRegistry,
        tool_kind: ToolKind,
    ) -> Self {
        Self {
            capability: CapabilityId::Tooling,
            registry_policy_hash: registry.policy_hash(),
            tool_kind: if packet.has_ready_task() {
                tool_kind
            } else {
                ToolKind::Denied
            },
            objective_id: packet.objective_id,
            task_id: packet.active_task_id,
            command_hash: tool_command_hash(packet),
            input_hash: tool_input_hash(packet),
            requested_effect: PacketEffect::MaterializeArtifact,
        }
    }

    pub fn is_admissible(self) -> bool {
        self.capability == CapabilityId::Tooling
            && self.registry_policy_hash != 0
            && matches!(
                self.tool_kind,
                ToolKind::DeterministicArtifact | ToolKind::SandboxFile
            )
            && self.objective_id != 0
            && self.task_id != 0
            && self.command_hash != 0
            && self.input_hash != 0
            && self.requested_effect == PacketEffect::MaterializeArtifact
    }

    pub fn matches_packet(self, packet: Packet) -> bool {
        self.objective_id == packet.objective_id
            && self.task_id == packet.active_task_id
            && self.command_hash == tool_command_hash(packet)
            && self.input_hash == tool_input_hash(packet)
            && self.requested_effect == PacketEffect::MaterializeArtifact
    }

    pub fn contract_hash(self) -> u64 {
        let mut h = 0xbb67ae8584caa73bu64;
        h = mix(h, self.capability as u64);
        h = mix(h, self.registry_policy_hash);
        h = mix(h, self.tool_kind as u64);
        h = mix(h, self.objective_id);
        h = mix(h, self.task_id);
        h = mix(h, self.command_hash);
        h = mix(h, self.input_hash);
        h = mix(h, self.requested_effect as u64);
        h.max(1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolReceipt {
    pub registry_policy_hash: u64,
    pub objective_id: u64,
    pub task_id: u64,
    pub command_hash: u64,
    pub input_hash: u64,
    pub output_hash: u64,
    pub exit_code: u8,
    pub artifact_path_hash: u64,
    pub artifact_content_hash: u64,
    pub artifact_bytes: u64,
    pub sandbox_root_hash: u64,
    pub receipt_hash: u64,
}

impl ToolReceipt {
    pub fn is_success_for(self, request: ToolRequest) -> bool {
        self.exit_code == 0
            && self.registry_policy_hash == request.registry_policy_hash
            && self.objective_id == request.objective_id
            && self.task_id == request.task_id
            && self.command_hash == request.command_hash
            && self.input_hash == request.input_hash
            && self.output_hash != 0
            && self.receipt_hash
                == expected_receipt_hash(
                    request,
                    self.output_hash,
                    self.exit_code,
                    self.artifact_path_hash,
                    self.artifact_content_hash,
                    self.artifact_bytes,
                    self.sandbox_root_hash,
                )
    }

    pub fn is_sandbox_artifact_bound(self) -> bool {
        self.artifact_path_hash != 0
            && self.artifact_content_hash != 0
            && self.artifact_bytes != 0
            && self.sandbox_root_hash != 0
    }

    fn success(request: ToolRequest, output_hash: u64) -> Self {
        Self::success_with_artifact(request, output_hash, 0, 0, 0, 0)
    }

    fn success_with_artifact(
        request: ToolRequest,
        output_hash: u64,
        artifact_path_hash: u64,
        artifact_content_hash: u64,
        artifact_bytes: u64,
        sandbox_root_hash: u64,
    ) -> Self {
        Self {
            registry_policy_hash: request.registry_policy_hash,
            objective_id: request.objective_id,
            task_id: request.task_id,
            command_hash: request.command_hash,
            input_hash: request.input_hash,
            output_hash,
            exit_code: 0,
            artifact_path_hash,
            artifact_content_hash,
            artifact_bytes,
            sandbox_root_hash,
            receipt_hash: expected_receipt_hash(
                request,
                output_hash,
                0,
                artifact_path_hash,
                artifact_content_hash,
                artifact_bytes,
                sandbox_root_hash,
            ),
        }
    }

    fn failure(request: ToolRequest, exit_code: u8) -> Self {
        let output_hash = tool_failure_hash(request, exit_code);
        Self {
            registry_policy_hash: request.registry_policy_hash,
            objective_id: request.objective_id,
            task_id: request.task_id,
            command_hash: request.command_hash,
            input_hash: request.input_hash,
            output_hash,
            exit_code,
            artifact_path_hash: 0,
            artifact_content_hash: 0,
            artifact_bytes: 0,
            sandbox_root_hash: 0,
            receipt_hash: expected_receipt_hash(request, output_hash, exit_code, 0, 0, 0, 0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeterministicToolExecutor {
    pub max_input_hash: u64,
    pub registry: CapabilityRegistry,
}

impl Default for DeterministicToolExecutor {
    fn default() -> Self {
        Self {
            max_input_hash: u64::MAX,
            registry: CapabilityRegistry::canonical(),
        }
    }
}

impl DeterministicToolExecutor {
    pub fn execute(self, request: ToolRequest) -> ToolReceipt {
        if !self.authorizes(request) || request.input_hash > self.max_input_hash {
            return ToolReceipt::failure(request, 126);
        }

        ToolReceipt::success(request, tool_output_hash(request))
    }

    pub fn execute_packet(self, packet: Packet) -> ToolReceipt {
        let request = ToolRequest::from_packet_with_registry(packet, self.registry);
        if !self.authorizes(request) || request.input_hash > self.max_input_hash {
            return ToolReceipt::failure(request, 126);
        }

        let mut after = packet;
        after.materialize_artifact();
        ToolReceipt::success(request, tool_effect_output_hash(packet, after))
    }

    pub fn authorizes(self, request: ToolRequest) -> bool {
        request.is_admissible()
            && request.registry_policy_hash == self.registry.policy_hash()
            && self.registry.permits_effect(
                request.capability,
                GateId::Execution,
                Evidence::ArtifactReceipt,
                request.requested_effect,
            )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveSandboxToolExecutor {
    pub root: PathBuf,
    pub max_artifact_bytes: u64,
    pub registry: CapabilityRegistry,
}

impl LiveSandboxToolExecutor {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            max_artifact_bytes: 64 * 1024,
            registry: CapabilityRegistry::canonical(),
        }
    }

    pub fn with_registry(mut self, registry: CapabilityRegistry) -> Self {
        self.registry = registry;
        self
    }

    pub fn with_max_artifact_bytes(mut self, max_artifact_bytes: u64) -> Self {
        self.max_artifact_bytes = max_artifact_bytes;
        self
    }

    pub fn execute_packet(&self, packet: Packet) -> Result<ToolExecutionRecord, ToolSandboxError> {
        let request = ToolRequest::from_packet_with_registry_and_kind(
            packet,
            self.registry,
            ToolKind::SandboxFile,
        );

        if !self.authorizes(request) {
            return Ok(ToolExecutionRecord {
                request,
                receipt: ToolReceipt::failure(request, 126),
            });
        }

        let mut after = packet;
        after.materialize_artifact();
        let body = sandbox_artifact_body(request, packet, after);
        let artifact_bytes = body.len() as u64;
        if artifact_bytes == 0 || artifact_bytes > self.max_artifact_bytes {
            return Err(ToolSandboxError::ArtifactTooLarge);
        }

        let root = self.prepare_root()?;
        let artifact_path = self.artifact_path_for(request)?;
        let parent = artifact_path.parent().ok_or(ToolSandboxError::SandboxIo)?;
        fs::create_dir_all(parent).map_err(|_| ToolSandboxError::SandboxIo)?;
        ensure_under(&root, &artifact_path)?;

        {
            let mut file = File::create(&artifact_path).map_err(|_| ToolSandboxError::SandboxIo)?;
            file.write_all(&body)
                .map_err(|_| ToolSandboxError::SandboxIo)?;
            file.sync_all().map_err(|_| ToolSandboxError::SandboxIo)?;
        }
        sync_dir(parent)?;

        let receipt = ToolReceipt::success_with_artifact(
            request,
            tool_effect_output_hash(packet, after),
            path_hash(&artifact_relative_name(request)),
            bytes_hash(&body),
            artifact_bytes,
            path_hash(root.to_string_lossy().as_bytes()),
        );

        Ok(ToolExecutionRecord { request, receipt })
    }

    pub fn artifact_path_for(&self, request: ToolRequest) -> Result<PathBuf, ToolSandboxError> {
        let root = self.prepare_root()?;
        let path = root
            .join("artifacts")
            .join(artifact_relative_name(request));
        ensure_relative_name(&artifact_relative_name(request))?;
        Ok(path)
    }

    pub fn authorizes(&self, request: ToolRequest) -> bool {
        request.is_admissible()
            && request.tool_kind == ToolKind::SandboxFile
            && request.registry_policy_hash == self.registry.policy_hash()
            && self.registry.permits_effect(
                request.capability,
                GateId::Execution,
                Evidence::ArtifactReceipt,
                request.requested_effect,
            )
    }

    fn prepare_root(&self) -> Result<PathBuf, ToolSandboxError> {
        fs::create_dir_all(&self.root).map_err(|_| ToolSandboxError::SandboxIo)?;
        self.root
            .canonicalize()
            .map_err(|_| ToolSandboxError::SandboxIo)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolExecutionRecord {
    pub request: ToolRequest,
    pub receipt: ToolReceipt,
}

impl ToolExecutionRecord {
    pub fn from_packet(packet: Packet) -> Self {
        let request = ToolRequest::from_packet(packet);
        let receipt = DeterministicToolExecutor::default().execute_packet(packet);
        Self { request, receipt }
    }

    pub fn from_packet_with_executor(packet: Packet, executor: DeterministicToolExecutor) -> Self {
        let request = ToolRequest::from_packet_with_registry(packet, executor.registry);
        let receipt = executor.execute_packet(packet);
        Self { request, receipt }
    }

    pub fn from_request(request: ToolRequest) -> Self {
        let receipt = DeterministicToolExecutor::default().execute(request);
        Self { request, receipt }
    }

    pub fn decision(&self) -> ToolDecision {
        if self.is_valid() {
            ToolDecision::Succeeded
        } else {
            ToolDecision::Failed
        }
    }

    pub fn is_valid(&self) -> bool {
        self.request.is_admissible()
            && self.receipt.is_success_for(self.request)
            && self.request.requested_effect == PacketEffect::MaterializeArtifact
    }

    pub fn verifies_persisted_event(&self, event: &ControlEvent) -> bool {
        persisted_execution_effect_is_valid(self, event)
    }

    pub fn effect_receipt_for_event(&self, event: &ControlEvent) -> Option<ToolEffectReceipt> {
        ToolEffectReceipt::from_persisted_event(self, event)
    }

    pub fn submission(&self) -> EvidenceSubmission {
        let passed = self.decision() == ToolDecision::Succeeded;
        EvidenceSubmission::with_effect_payload(
            GateId::Execution,
            Evidence::ArtifactReceipt,
            passed,
            if passed {
                PacketEffect::MaterializeArtifact
            } else {
                PacketEffect::None
            },
            tooling_payload_hash(self),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolEffectReceipt {
    pub capability: CapabilityId,
    pub registry_policy_hash: u64,
    pub request_hash: u64,
    pub receipt_hash: u64,
    pub effect_hash: u64,
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
        Some(Self {
            capability: record.request.capability,
            registry_policy_hash: record.request.registry_policy_hash,
            request_hash: record.request.contract_hash(),
            receipt_hash: record.receipt.receipt_hash,
            effect_hash,
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
            && self.event_seq != 0
            && self.event_hash != 0
            && self.artifact_id != 0
            && self.artifact_receipt_hash != 0
    }

    pub fn is_sandbox_artifact_bound(self) -> bool {
        self.artifact_path_hash != 0
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

    if fields.len() != 15
        || fields[0] != TOOL_EFFECT_RECEIPT_SCHEMA_VERSION
        || fields[1] != TOOL_EFFECT_RECEIPT_RECORD
    {
        return Err(ToolSandboxError::InvalidToolReceiptRecord);
    }

    let receipt = ToolEffectReceipt {
        capability: capability_from_u64(fields[2])?,
        registry_policy_hash: fields[3],
        request_hash: fields[4],
        receipt_hash: fields[5],
        effect_hash: fields[6],
        event_seq: fields[7],
        event_hash: fields[8],
        artifact_id: fields[9],
        artifact_receipt_hash: fields[10],
        artifact_path_hash: fields[11],
        artifact_content_hash: fields[12],
        artifact_bytes: fields[13],
        sandbox_root_hash: fields[14],
    };

    if !receipt.is_valid() {
        return Err(ToolSandboxError::InvalidToolReceipt);
    }

    Ok(receipt)
}

fn tooling_payload_hash(record: &ToolExecutionRecord) -> u64 {
    let mut h = 0xa54f_f53a_5f1d_36f1u64;
    h = mix(h, record.request.contract_hash());
    h = mix(h, record.request.registry_policy_hash);
    h = mix(h, record.receipt.registry_policy_hash);
    h = mix(h, record.receipt.objective_id);
    h = mix(h, record.receipt.task_id);
    h = mix(h, record.receipt.command_hash);
    h = mix(h, record.receipt.input_hash);
    h = mix(h, record.receipt.output_hash);
    h = mix(h, record.receipt.exit_code as u64);
    h = mix(h, record.receipt.artifact_path_hash);
    h = mix(h, record.receipt.artifact_content_hash);
    h = mix(h, record.receipt.artifact_bytes);
    h = mix(h, record.receipt.sandbox_root_hash);
    h = mix(h, record.receipt.receipt_hash);
    h.max(1)
}

fn persisted_execution_effect_is_valid(record: &ToolExecutionRecord, event: &ControlEvent) -> bool {
    if !record.is_valid()
        || event.from != Phase::Execute
        || event.to != Phase::Execute
        || event.kind != EventKind::Persisted
        || event.cause != Cause::EvidenceSubmitted
        || event.evidence != Evidence::ArtifactReceipt
        || event.decision != Decision::Continue
        || event.failure.is_some()
        || event.recovery_action.is_some()
        || event.affected_gate != Some(GateId::Execution)
        || event.capability_registry_projection.policy_hash != record.request.registry_policy_hash
        || !record.request.matches_packet(event.state_before.packet)
    {
        return false;
    }

    let mut expected = event.state_before;
    expected.packet.materialize_artifact();
    expected.apply_evidence(GateId::Execution, Evidence::ArtifactReceipt, true);

    event.state_after == expected
        && event.state_after.gates.execution.status == GateStatus::Pass
        && event.state_after.gates.execution.evidence == Evidence::ArtifactReceipt
        && record.receipt.output_hash
            == tool_effect_output_hash(event.state_before.packet, event.state_after.packet)
}

fn expected_receipt_hash(
    request: ToolRequest,
    output_hash: u64,
    exit_code: u8,
    artifact_path_hash: u64,
    artifact_content_hash: u64,
    artifact_bytes: u64,
    sandbox_root_hash: u64,
) -> u64 {
    let mut h = 0x3c6ef372fe94f82bu64;
    h = mix(h, request.contract_hash());
    h = mix(h, output_hash);
    h = mix(h, exit_code as u64);
    h = mix(h, artifact_path_hash);
    h = mix(h, artifact_content_hash);
    h = mix(h, artifact_bytes);
    h = mix(h, sandbox_root_hash);
    h.max(1)
}

fn tool_command_hash(packet: Packet) -> u64 {
    let mut h = 0xbb67ae8584caa73bu64;
    h = mix(h, packet.objective_id);
    h = mix(h, packet.active_task_id);
    h.max(1)
}

fn tool_input_hash(packet: Packet) -> u64 {
    let mut h = 0x13198a2e03707344u64;
    h = mix(h, packet.objective_id);
    h = mix(h, packet.active_task_id);
    h = mix(h, packet.ready_tasks as u64);
    h = mix(h, packet.revision);
    h.max(1)
}

fn tool_output_hash(request: ToolRequest) -> u64 {
    let mut h = 0x243f6a8885a308d3u64;
    h = mix(h, request.objective_id);
    h = mix(h, request.task_id);
    h = mix(h, request.command_hash);
    h = mix(h, request.input_hash);
    h.max(1)
}

fn tool_effect_output_hash(before: Packet, after: Packet) -> u64 {
    let mut h = 0x4528_21e6_38d0_1377u64;
    h = mix_packet(h, before);
    h = mix_packet(h, after);
    h.max(1)
}

fn mix_packet(mut h: u64, packet: Packet) -> u64 {
    h = mix(h, packet.objective_id);
    h = mix(h, packet.objective_required_tasks as u64);
    h = mix(h, packet.objective_done_tasks as u64);
    h = mix(h, packet.ready_tasks as u64);
    h = mix(h, packet.active_task_id);
    h = mix(h, packet.artifact_id);
    h = mix(h, packet.parent_artifact_id);
    h = mix(h, packet.artifact_bytes);
    h = mix(h, packet.artifact_receipt_hash);
    h = mix(h, packet.artifact_lineage_hash);
    h = mix(h, packet.revision);
    h
}

fn tool_failure_hash(request: ToolRequest, exit_code: u8) -> u64 {
    let mut h = 0x9e3779b97f4a7c15u64;
    h = mix(h, request.contract_hash());
    h = mix(h, exit_code as u64);
    h.max(1)
}

fn artifact_relative_name(request: ToolRequest) -> String {
    format!(
        "artifact-{:016x}-{:016x}.canon",
        request.contract_hash(),
        request.input_hash
    )
}

fn sandbox_artifact_body(request: ToolRequest, before: Packet, after: Packet) -> Vec<u8> {
    format!(
        concat!(
            "canon-tool-artifact-v1\n",
            "capability={}\n",
            "registry_policy_hash={}\n",
            "request_hash={}\n",
            "objective_id={}\n",
            "task_id={}\n",
            "input_hash={}\n",
            "effect=MaterializeArtifact\n",
            "before_revision={}\n",
            "after_revision={}\n",
            "artifact_id={}\n",
            "artifact_receipt_hash={}\n"
        ),
        request.capability as u64,
        request.registry_policy_hash,
        request.contract_hash(),
        request.objective_id,
        request.task_id,
        request.input_hash,
        before.revision,
        after.revision,
        after.artifact_id,
        after.artifact_receipt_hash
    )
    .into_bytes()
}

fn ensure_relative_name(name: &str) -> Result<(), ToolSandboxError> {
    let path = Path::new(name);
    if path.components().count() == 1 && !name.is_empty() && !name.contains(std::path::MAIN_SEPARATOR)
    {
        Ok(())
    } else {
        Err(ToolSandboxError::PathEscapesSandbox)
    }
}

fn ensure_under(root: &Path, path: &Path) -> Result<(), ToolSandboxError> {
    let parent = path.parent().ok_or(ToolSandboxError::SandboxIo)?;
    fs::create_dir_all(parent).map_err(|_| ToolSandboxError::SandboxIo)?;
    let parent = parent
        .canonicalize()
        .map_err(|_| ToolSandboxError::SandboxIo)?;
    if parent.starts_with(root) {
        Ok(())
    } else {
        Err(ToolSandboxError::PathEscapesSandbox)
    }
}

fn sync_dir(path: &Path) -> Result<(), ToolSandboxError> {
    let dir = File::open(path).map_err(|_| ToolSandboxError::SandboxIo)?;
    dir.sync_all().map_err(|_| ToolSandboxError::SandboxIo)
}

fn bytes_hash(bytes: &[u8]) -> u64 {
    let mut h = 0x6a09e667f3bcc909u64;
    h = mix(h, bytes.len() as u64);
    for byte in bytes {
        h = mix(h, *byte as u64);
    }
    h.max(1)
}

fn path_hash(path: impl AsRef<[u8]>) -> u64 {
    bytes_hash(path.as_ref())
}

fn capability_from_u64(value: u64) -> Result<CapabilityId, ToolSandboxError> {
    match value {
        7 => Ok(CapabilityId::Tooling),
        _ => Err(ToolSandboxError::InvalidToolReceiptRecord),
    }
}