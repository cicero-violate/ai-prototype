/// Canonical failure classes used by the runtime recovery gate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureClass {
    DirtyWorktree,
    BundleMissing,
    ManifestInvalid,
    HashMismatch,
    ValidationFailure,
    ValidationTimeout,
    RuntimeEvidenceGap,
    NetworkTimeout,
}

/// Pure recovery actions. No IO or retry logic is hidden inside policy code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryAction {
    Refuse,
    RepairManifest,
    Revalidate,
    RetryNetwork,
    QuarantineEvidence,
}

/// Deterministic decision produced by [`RecoveryPolicy`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryDecision {
    pub failure: FailureClass,
    pub action: RecoveryAction,
    pub fatal: bool,
    pub trust_score: u8,
}

impl RecoveryDecision {
    pub const fn new(
        failure: FailureClass,
        action: RecoveryAction,
        fatal: bool,
        trust_score: u8,
    ) -> Self {
        Self { failure, action, fatal, trust_score }
    }
}

/// Small, table-like policy for recovery classification.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecoveryPolicy;

impl RecoveryPolicy {
    pub const fn decide(self, failure: FailureClass) -> RecoveryDecision {
        use FailureClass::*;
        use RecoveryAction::*;

        match failure {
            DirtyWorktree => RecoveryDecision::new(failure, Refuse, true, 0),
            BundleMissing => RecoveryDecision::new(failure, Refuse, true, 0),
            ManifestInvalid => RecoveryDecision::new(failure, RepairManifest, true, 20),
            HashMismatch => RecoveryDecision::new(failure, Refuse, true, 0),
            ValidationFailure => RecoveryDecision::new(failure, Revalidate, true, 40),
            ValidationTimeout => RecoveryDecision::new(failure, Revalidate, true, 35),
            RuntimeEvidenceGap => RecoveryDecision::new(failure, QuarantineEvidence, false, 55),
            NetworkTimeout => RecoveryDecision::new(failure, RetryNetwork, false, 60),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fatal_integrity_failures_never_raise_trust() {
        let policy = RecoveryPolicy;
        for failure in [
            FailureClass::DirtyWorktree,
            FailureClass::BundleMissing,
            FailureClass::HashMismatch,
        ] {
            let decision = policy.decide(failure);
            assert_eq!(decision.action, RecoveryAction::Refuse);
            assert!(decision.fatal);
            assert_eq!(decision.trust_score, 0);
        }
    }

    #[test]
    fn advisory_gaps_are_quarantined_without_blocking_git_truth() {
        let decision = RecoveryPolicy.decide(FailureClass::RuntimeEvidenceGap);
        assert_eq!(decision.action, RecoveryAction::QuarantineEvidence);
        assert!(!decision.fatal);
        assert!(decision.trust_score > 0);
        assert!(decision.trust_score < 100);
    }
}