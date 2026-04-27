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

use crate::capability::{EvidenceProducer, EvidenceSubmission, PacketEffect};
use crate::kernel::{mix, Evidence, GateId, Packet};

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
    pub tool_kind: ToolKind,
    pub objective_id: u64,
    pub task_id: u64,
    pub command_hash: u64,
    pub input_hash: u64,
    pub requested_effect: PacketEffect,
}

impl ToolRequest {
    pub fn from_packet(packet: Packet) -> Self {
        Self {
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
        self.tool_kind == ToolKind::DeterministicArtifact
            && self.objective_id != 0
            && self.task_id != 0
            && self.command_hash != 0
            && self.input_hash != 0
            && self.requested_effect == PacketEffect::MaterializeArtifact
    }

    pub fn contract_hash(self) -> u64 {
        let mut h = 0xbb67ae8584caa73bu64;
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
            && self.objective_id == request.objective_id
            && self.task_id == request.task_id
            && self.command_hash == request.command_hash
            && self.input_hash == request.input_hash
            && self.output_hash != 0
            && self.receipt_hash == expected_receipt_hash(request, self.output_hash, self.exit_code)
    }

    fn success(request: ToolRequest, output_hash: u64) -> Self {
        Self {
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
}

impl Default for DeterministicToolExecutor {
    fn default() -> Self {
        Self {
            max_input_hash: u64::MAX,
        }
    }
}

impl DeterministicToolExecutor {
    pub fn execute(self, request: ToolRequest) -> ToolReceipt {
        if !request.is_admissible() || request.input_hash > self.max_input_hash {
            return ToolReceipt::failure(request, 126);
        }

        ToolReceipt::success(request, tool_output_hash(request))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolExecutionRecord {
    pub request: ToolRequest,
    pub receipt: ToolReceipt,
}

impl ToolExecutionRecord {
    pub fn from_packet(packet: Packet) -> Self {
        Self::from_request(ToolRequest::from_packet(packet))
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
    h = mix(h, record.receipt.objective_id);
    h = mix(h, record.receipt.task_id);
    h = mix(h, record.receipt.command_hash);
    h = mix(h, record.receipt.input_hash);
    h = mix(h, record.receipt.output_hash);
    h = mix(h, record.receipt.exit_code as u64);
    h = mix(h, record.receipt.receipt_hash);
    h.max(1)
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

fn tool_failure_hash(request: ToolRequest, exit_code: u8) -> u64 {
    let mut h = 0x9e3779b97f4a7c15u64;
    h = mix(h, request.contract_hash());
    h = mix(h, exit_code as u64);
    h.max(1)
}