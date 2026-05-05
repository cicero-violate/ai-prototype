use crate::stable_hash_fields;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureClass {
    Transport,
    Sandbox,
    Observation,
    Policy,
    Kernel,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryAction {
    Retry,
    Quarantine,
    Halt,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryInput {
    pub failure: FailureClass,
    pub attempts: u32,
    pub transient: bool,
    pub invariant_broken: bool,
    pub evidence_hash: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryDecision {
    pub action: RecoveryAction,
    pub retry_after_ticks: u64,
    pub max_attempts: u32,
    pub reason_hash: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryPolicy {
    pub max_attempts: u32,
    pub quarantine_after: u32,
    pub backoff_base_ticks: u64,
}

impl Default for RecoveryPolicy {
    fn default() -> Self {
        Self::bounded(3, 2, 1)
    }
}

impl RecoveryPolicy {
    pub fn bounded(max_attempts: u32, quarantine_after: u32, backoff_base_ticks: u64) -> Self {
        Self {
            max_attempts: max_attempts.max(1),
            quarantine_after: quarantine_after.max(1),
            backoff_base_ticks: backoff_base_ticks.max(1),
        }
    }

    pub fn decide(self, input: RecoveryInput) -> RecoveryDecision {
        let action = if input.invariant_broken || input.failure == FailureClass::Kernel {
            RecoveryAction::Halt
        } else if input.attempts >= self.max_attempts {
            RecoveryAction::Halt
        } else if input.failure == FailureClass::Policy {
            RecoveryAction::Quarantine
        } else if input.transient && input.attempts < self.quarantine_after {
            RecoveryAction::Retry
        } else {
            RecoveryAction::Quarantine
        };

        RecoveryDecision {
            action,
            retry_after_ticks: if action == RecoveryAction::Retry { self.retry_delay(input.attempts) } else { 0 },
            max_attempts: self.max_attempts,
            reason_hash: self.reason_hash(input, action),
        }
    }

    fn retry_delay(self, attempts: u32) -> u64 {
        let multiplier = 1_u64.checked_shl(attempts.min(20)).unwrap_or(1 << 20);
        self.backoff_base_ticks.saturating_mul(multiplier).min(1_000_000)
    }

    fn reason_hash(self, input: RecoveryInput, action: RecoveryAction) -> u64 {
        stable_hash_fields(&[
            format!("{:?}", input.failure).as_bytes(),
            format!("{:?}", action).as_bytes(),
            &input.attempts.to_le_bytes(),
            &[u8::from(input.transient)],
            &[u8::from(input.invariant_broken)],
            &input.evidence_hash.to_le_bytes(),
            &self.max_attempts.to_le_bytes(),
            &self.quarantine_after.to_le_bytes(),
            &self.backoff_base_ticks.to_le_bytes(),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(failure: FailureClass) -> RecoveryInput {
        RecoveryInput { failure, attempts: 0, transient: true, invariant_broken: false, evidence_hash: 42 }
    }

    #[test]
    fn retry_is_bounded_and_deterministic() {
        let policy = RecoveryPolicy::bounded(5, 4, 3);
        let decision = policy.decide(event(FailureClass::Transport));
        assert_eq!(decision.action, RecoveryAction::Retry);
        assert_eq!(decision.retry_after_ticks, 3);
        assert_eq!(decision, policy.decide(event(FailureClass::Transport)));
    }

    #[test]
    fn invariant_breaks_halt_without_retry() {
        let mut event = event(FailureClass::Transport);
        event.invariant_broken = true;
        let decision = RecoveryPolicy::default().decide(event);
        assert_eq!(decision.action, RecoveryAction::Halt);
        assert_eq!(decision.retry_after_ticks, 0);
    }

    #[test]
    fn policy_failures_are_quarantined() {
        assert_eq!(RecoveryPolicy::default().decide(event(FailureClass::Policy)).action, RecoveryAction::Quarantine);
    }
}