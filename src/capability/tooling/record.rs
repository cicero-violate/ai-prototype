//! Durable tool execution payload owned by the tooling capability.
//!
//! Tooling is the first capability boundary that must stop being a synthetic
//! record facade. This module models the minimum deterministic execution loop:
//!
//! ```text
//! ToolRequest -> DeterministicToolExecutor -> ToolReceipt -> EvidenceSubmission
//! ```
//!
//! The executor intentionally does not spawn a process yet. It gives the API a
//! typed request/receipt seam that can later be backed by a filesystem, browser,
//! command, or provider sandbox without changing the kernel evidence contract.
//! The receipt is now also checked against the exact packet effect committed to
//! the TLog, so execution replay proves more than "the execution gate passed."
//! Registry policy hash is part of the request/receipt contract, so replay can
//! reject executions under a drifted capability authority surface.

use crate::capability::{
    CapabilityId, CapabilityRegistry, EvidenceProducer, EvidenceSubmission, PacketEffect,
};
use crate::kernel::{
    mix, Cause, ControlEvent, Decision, EventKind, Evidence, GateId, GateStatus, Packet, Phase,
    TLog,
};

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
        Self {
            capability: CapabilityId::Tooling,
            registry_policy_hash: registry.policy_hash(),
            tool_kind: if packet.has_ready_task() {
                ToolKind::DeterministicArtifact
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
            && self.tool_kind == ToolKind::DeterministicArtifact
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
            && self.receipt_hash == expected_receipt_hash(request, self.output_hash, self.exit_code)
    }

    fn success(request: ToolRequest, output_hash: u64) -> Self {
        Self {
            registry_policy_hash: request.registry_policy_hash,
            objective_id: request.objective_id,
            task_id: request.task_id,
            command_hash: request.command_hash,
            input_hash: request.input_hash,
            output_hash,
            exit_code: 0,
            receipt_hash: expected_receipt_hash(request, output_hash, 0),
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
            receipt_hash: expected_receipt_hash(request, output_hash, exit_code),
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
        })
    }

    pub fn replay_verified(self, tlog: &TLog) -> bool {
        tlog.iter().any(|event| self.matches_event(event))
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

fn expected_receipt_hash(request: ToolRequest, output_hash: u64, exit_code: u8) -> u64 {
    let mut h = 0x3c6ef372fe94f82bu64;
    h = mix(h, request.contract_hash());
    h = mix(h, output_hash);
    h = mix(h, exit_code as u64);
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