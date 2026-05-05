use ai::runtime::{FailureClass, RecoveryAction, RecoveryInput, RecoveryPolicy};

fn event(failure: FailureClass, attempts: u32, transient: bool) -> RecoveryInput {
    RecoveryInput { failure, attempts, transient, invariant_broken: false, evidence_hash: 7 }
}

#[test]
fn terminal_classes_never_retry() {
    let policy = RecoveryPolicy::default();
    for failure in [FailureClass::Kernel, FailureClass::Policy] {
        assert_ne!(policy.decide(event(failure, 0, true)).action, RecoveryAction::Retry);
    }
}

#[test]
fn exhausted_attempts_halt_before_more_work() {
    let decision = RecoveryPolicy::bounded(2, 2, 1).decide(event(FailureClass::Transport, 2, true));
    assert_eq!(decision.action, RecoveryAction::Halt);
    assert_eq!(decision.retry_after_ticks, 0);
}

#[test]
fn very_large_backoff_cannot_overflow() {
    let decision = RecoveryPolicy::bounded(u32::MAX, u32::MAX, u64::MAX)
        .decide(event(FailureClass::Sandbox, u32::MAX - 1, true));
    assert_eq!(decision.action, RecoveryAction::Retry);
    assert_eq!(decision.retry_after_ticks, 1_000_000);
}

#[test]
fn non_transient_unknown_work_is_quarantined() {
    assert_eq!(RecoveryPolicy::default().decide(event(FailureClass::Unknown, 0, false)).action, RecoveryAction::Quarantine);
}