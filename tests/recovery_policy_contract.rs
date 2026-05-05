use ai::{RecoveryAction, RecoveryPolicy};

#[test]
fn blocked_repair_surface_escalates() {
    let policy = RecoveryPolicy::bounded(2);
    assert_eq!(policy.classify(1, false), RecoveryAction::Escalate);
}

#[test]
fn bounded_attempts_prevent_infinite_repair() {
    let policy = RecoveryPolicy::bounded(2);
    assert_eq!(policy.classify(0, true), RecoveryAction::Continue);
    assert_eq!(policy.classify(1, true), RecoveryAction::Repair);
    assert_eq!(policy.classify(3, true), RecoveryAction::Escalate);
}