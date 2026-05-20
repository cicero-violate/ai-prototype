//! Agent cycle event — a kernel-backed scheduling journal entry.
//!
//! Every significant boundary in the agent loop (cycle start/end, turn
//! complete/failed) is submitted to the kernel as a `SubmitAgentCycleEvent`
//! command.  The kernel records it in the TLog without advancing any gate,
//! making the full scheduling history replayable from the TLog alone.
//!
//! Schema versioning: the contract_hash binds kind + agent_hash + cycle +
//! label_hash + content_hash so any field change is detectable on replay.

use crate::kernel::mix;

const AGENT_CYCLE_EVENT_SCHEMA_VERSION: u64 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AgentCycleEventKind {
    CycleStart = 1,
    TurnComplete = 2,
    TurnFailed = 3,
    CycleEnd = 4,
}

impl AgentCycleEventKind {
    pub fn as_u64(self) -> u64 {
        self as u64
    }
}

/// A single scheduling journal entry, structured for kernel TLog submission.
///
/// All string fields are pre-hashed so the struct is `Copy` and avoids heap
/// allocation on the hot path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AgentCycleEvent {
    /// Event category.
    pub kind: AgentCycleEventKind,
    /// FNV-1a hash of the agent tag string (e.g. `"mini-fix-scc-layer"`).
    pub agent_hash: u64,
    /// Monotonically increasing cycle number within this agent run.
    pub cycle: u64,
    /// FNV-1a hash of the turn label (`"plan"`, `"execute-1"`, …).
    /// Zero for cycle-level events (start / end).
    pub label_hash: u64,
    /// Hash of the turn's response content.
    /// Zero for cycle_start, turn_failed, and cycle_end events.
    pub content_hash: u64,
    /// Self-hash binding all fields — verified in `is_contract_valid`.
    pub contract_hash: u64,
}

impl AgentCycleEvent {
    pub fn new(
        kind: AgentCycleEventKind,
        agent_hash: u64,
        cycle: u64,
        label_hash: u64,
        content_hash: u64,
    ) -> Self {
        let contract_hash =
            expected_contract_hash(kind, agent_hash, cycle, label_hash, content_hash);
        Self {
            kind,
            agent_hash,
            cycle,
            label_hash,
            content_hash,
            contract_hash,
        }
    }

    pub fn is_contract_valid(self) -> bool {
        self.contract_hash != 0
            && self.agent_hash != 0
            && self.cycle != 0
            && self.contract_hash
                == expected_contract_hash(
                    self.kind,
                    self.agent_hash,
                    self.cycle,
                    self.label_hash,
                    self.content_hash,
                )
    }

    pub fn contract_hash(self) -> u64 {
        self.contract_hash
    }
}

fn expected_contract_hash(
    kind: AgentCycleEventKind,
    agent_hash: u64,
    cycle: u64,
    label_hash: u64,
    content_hash: u64,
) -> u64 {
    let mut h = 0x9e37_79b9_7f4a_7c15u64;
    h = mix(h, AGENT_CYCLE_EVENT_SCHEMA_VERSION);
    h = mix(h, kind.as_u64());
    h = mix(h, agent_hash);
    h = mix(h, cycle);
    h = mix(h, label_hash);
    h = mix(h, content_hash);
    h.max(1)
}
