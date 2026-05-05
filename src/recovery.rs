#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureClass {
    RuntimeEvidenceGap,
    ArtifactMissing,
    ArtifactScanTimeout,
    ManifestInvalid,
    HashMismatch,
    MergeConflict,
    ValidationTimeout,
    ValidationFailure,
    DirtyWorktree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    StopNoMerge,
    KeepHistoryOnly,
    RefuseUntilClean,
    ValidateThenReport,
}

pub fn recovery_action(class: FailureClass) -> RecoveryAction {
    match class {
        FailureClass::RuntimeEvidenceGap => RecoveryAction::KeepHistoryOnly,
        FailureClass::ArtifactMissing
        | FailureClass::ArtifactScanTimeout
        | FailureClass::ManifestInvalid
        | FailureClass::HashMismatch
        | FailureClass::MergeConflict => RecoveryAction::StopNoMerge,
        FailureClass::DirtyWorktree => RecoveryAction::RefuseUntilClean,
        FailureClass::ValidationTimeout | FailureClass::ValidationFailure => {
            RecoveryAction::ValidateThenReport
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReceipt {
    pub classification: &'static str,
    pub exit_code: Option<i32>,
    pub stdout_snippet: String,
    pub stderr_snippet: String,
}

impl ValidationReceipt {
    pub fn new(classification: &'static str, exit_code: Option<i32>, stdout: &str, stderr: &str) -> Self {
        Self {
            classification,
            exit_code,
            stdout_snippet: bounded_snippet(stdout, 256),
            stderr_snippet: bounded_snippet(stderr, 256),
        }
    }
}

fn bounded_snippet(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_owned();
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_owned()
}
