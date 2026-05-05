use ai::{FailureClass, RecoveryAction, RecoveryPolicy};

#[test]
fn recovery_policy_keeps_authority_order_deterministic() {
    let policy = RecoveryPolicy;

    let dirty = policy.decide(FailureClass::DirtyWorktree);
    let hash = policy.decide(FailureClass::HashMismatch);
    let evidence = policy.decide(FailureClass::RuntimeEvidenceGap);
    let network = policy.decide(FailureClass::NetworkTimeout);

    assert_eq!(dirty.action, RecoveryAction::Refuse);
    assert_eq!(hash.action, RecoveryAction::Refuse);
    assert!(dirty.fatal && hash.fatal);

    assert_eq!(evidence.action, RecoveryAction::QuarantineEvidence);
    assert_eq!(network.action, RecoveryAction::RetryNetwork);
    assert!(!evidence.fatal && !network.fatal);
    assert!(network.trust_score >= evidence.trust_score);
}

#[test]
fn validation_failures_share_one_repair_path() {
    let policy = RecoveryPolicy;
    let failure = policy.decide(FailureClass::ValidationFailure);
    let timeout = policy.decide(FailureClass::ValidationTimeout);

    assert_eq!(failure.action, RecoveryAction::Revalidate);
    assert_eq!(timeout.action, RecoveryAction::Revalidate);
    assert!(failure.fatal && timeout.fatal);
}