use super::{Evidence, FailureClass, GateId, GateSet, Packet, Phase, RecoveryAction};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct State {
    pub phase: Phase,
    pub gates: GateSet,
    pub packet: Packet,
    pub failure: Option<FailureClass>,
    pub recovery_action: Option<RecoveryAction>,
    pub recovery_attempts: u8,
    /// Number of child DAG tasks currently in flight (wave fan-out counter).
    /// Set by WaveDispatch events; decremented by ChildComplete events.
    /// Stored in TLog so replay can reconstruct the wave state after a crash.
    pub wave_pending: u16,
}

impl Default for State {
    fn default() -> Self {
        Self {
            phase: Phase::Delta,
            gates: GateSet::default(),
            packet: Packet::empty(),
            failure: None,
            recovery_action: None,
            recovery_attempts: 0,
            wave_pending: 0,
        }
    }
}

impl State {
    pub fn ready() -> Self {
        Self {
            phase: Phase::Delta,
            gates: GateSet::ready(),
            packet: Packet::ready(),
            failure: None,
            recovery_action: None,
            recovery_attempts: 0,
            wave_pending: 0,
        }
    }

    pub fn is_success(self) -> bool {
        self.failure.is_none()
            && self.phase == Phase::Done
            && self.gates.all_passed()
            && self.packet.objective_complete()
            && self.packet.lineage_valid()
    }

    pub fn apply_evidence(&mut self, gate: GateId, evidence: Evidence, passed: bool) {
        if passed {
            self.gates.set_pass(gate, evidence);
        } else {
            self.gates.set_fail(gate, evidence);
        }
    }

    pub fn is_structurally_valid(self) -> bool {
        self.gates.is_structurally_valid() && self.packet.is_structurally_valid()
    }
}
