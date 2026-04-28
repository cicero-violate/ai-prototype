use crate::capability::{CapabilityId, CapabilityRegistry, PacketEffect};
use crate::kernel::{mix, Packet};

use super::hash::{tool_command_hash, tool_input_hash};
use super::types::ToolKind;

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
pub struct SandboxProcessRequest {
    pub capability: CapabilityId,
    pub registry_policy_hash: u64,
    pub command_hash: u64,
    pub argv_hash: u64,
    pub cwd_hash: u64,
    pub env_hash: u64,
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
}

impl SandboxProcessRequest {
    pub fn contract_hash(&self) -> u64 {
        let mut h = 0x510e_527f_ade6_82d1u64;
        h = mix(h, self.capability as u64);
        h = mix(h, self.registry_policy_hash);
        h = mix(h, self.command_hash);
        h = mix(h, self.argv_hash);
        h = mix(h, self.cwd_hash);
        h = mix(h, self.env_hash);
        h = mix(h, self.timeout_ms);
        h = mix(h, self.max_output_bytes);
        h.max(1)
    }

    pub fn is_admissible(&self) -> bool {
        self.capability == CapabilityId::Tooling
            && self.registry_policy_hash != 0
            && self.command_hash != 0
            && self.argv_hash != 0
            && self.cwd_hash != 0
            && self.env_hash != 0
            && self.timeout_ms != 0
            && self.max_output_bytes != 0
    }
}