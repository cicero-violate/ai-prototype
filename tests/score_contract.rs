use ai::{
    recovery::{recovery_action, FailureClass, RecoveryAction, ValidationReceipt},
    score::{geometric_mean, ScoreDelta, ScoreVector},
    timing::PhaseTimings,
};

#[test]
fn timing_phase_sum_adds_all_observed_work() {
    let t = PhaseTimings {
        upload_ms: 1,
        turn_total_ms: 2,
        candidate_scan_ms: 3,
        delta_download_ms: 4,
        delta_apply_ms: 5,
        validation_ms: 6,
        loop_total_ms: 0,
    };
    assert_eq!(t.phase_sum(), 21);
    assert!(!t.satisfies_loop_invariant());
    assert!(t.canonicalized().satisfies_loop_invariant());
}

#[test]
fn score_values_preserve_all_goodness_axes() {
    let s = ScoreVector {
        intelligence: 1.0,
        efficiency: 0.9,
        correctness: 0.8,
        alignment: 0.7,
        robustness: 0.6,
        performance: 0.5,
        scalability: 0.4,
        determinism: 0.3,
        transparency: 0.2,
        collaboration: 0.1,
        empowerment: 0.9,
        benefit: 0.8,
        learning: 0.7,
        structure: 0.65,
        simplicity: 0.6,
        future_proofing: 0.5,
        maintainability: 0.55,
    };
    assert_eq!(s.values().len(), 17);
    assert!(
        s.geometric_mean()
            .expect("geometric mean should be defined")
            < 1.0
    );
}

#[test]
fn geometric_mean_rejects_invalid_score_surfaces() {
    assert_eq!(geometric_mean(&[]), None);
    assert_eq!(geometric_mean(&[1.0, f64::NAN]), None);
    assert_eq!(geometric_mean(&[-1.0]), None);
    assert_eq!(geometric_mean(&[1.0, 0.0]), Some(0.0));
    assert!(ScoreDelta {
        before: 0.80,
        after: 0.81
    }
    .is_real_gain());
}

#[test]
fn recovery_classes_have_deterministic_actions() {
    assert_eq!(
        recovery_action(FailureClass::DirtyWorktree),
        RecoveryAction::RefuseUntilClean
    );
    assert_eq!(
        recovery_action(FailureClass::ManifestInvalid),
        RecoveryAction::StopNoMerge
    );
    assert_eq!(
        recovery_action(FailureClass::RuntimeEvidenceGap),
        RecoveryAction::KeepHistoryOnly
    );
    assert_eq!(
        recovery_action(FailureClass::ValidationFailure),
        RecoveryAction::ValidateThenReport
    );
}

#[test]
fn validation_receipt_bounds_repair_evidence() {
    let receipt = ValidationReceipt::new("validation_failure", Some(1), &"x".repeat(300), "ok");
    assert_eq!(receipt.classification, "validation_failure");
    assert_eq!(receipt.stdout_snippet.len(), 256);
    assert_eq!(receipt.stderr_snippet, "ok");
}
