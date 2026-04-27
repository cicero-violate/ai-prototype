//! Durable tool execution payload owned by the tooling capability.

use crate::capability::{EvidenceProducer, EvidenceSubmission, PacketEffect};
use crate::kernel::{Evidence, GateId, Packet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolDecision {
    Succeeded,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolExecutionRecord {
    pub task_id: u64,
    pub command_hash: u64,
    pub exit_code: u8,
    pub output_hash: u64,
}

impl ToolExecutionRecord {
    pub fn from_packet(packet: Packet) -> Self {
        Self {
            task_id: packet.active_task_id,
            command_hash: tool_command_hash(packet),
            exit_code: 0,
            output_hash: tool_output_hash(packet),
        }
    }

    pub fn decision(&self) -> ToolDecision {
        if self.is_valid() {
            ToolDecision::Succeeded
        } else {
            ToolDecision::Failed
        }
    }

    pub fn is_valid(&self) -> bool {
        self.task_id != 0 && self.command_hash != 0 && self.exit_code == 0 && self.output_hash != 0
    }

    pub fn submission(&self) -> EvidenceSubmission {
        let passed = self.decision() == ToolDecision::Succeeded;
        EvidenceSubmission::with_effect(
            GateId::Execution,
            Evidence::ArtifactReceipt,
            passed,
            if passed {
                PacketEffect::MaterializeArtifact
            } else {
                PacketEffect::None
            },
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

fn tool_command_hash(packet: Packet) -> u64 {
    let mut h = 0xbb67ae8584caa73bu64;
    h ^= packet.objective_id;
    h = h.wrapping_mul(0x100000001b3);
    h ^= packet.active_task_id;
    h = h.wrapping_mul(0x100000001b3);
    h.max(1)
}

fn tool_output_hash(packet: Packet) -> u64 {
    let mut h = 0x3c6ef372fe94f82bu64;
    h ^= packet.objective_id;
    h = h.wrapping_mul(0x100000001b3);
    h ^= packet.active_task_id;
    h = h.wrapping_mul(0x100000001b3);
    h ^= packet.revision;
    h = h.wrapping_mul(0x100000001b3);
    h.max(1)
}