//! Typed outcome boundary for rustc/compiler analysis results.
//!
//! Callers match these variants instead of parsing strings, JSON fields, or
//! formatted compiler errors.

use serde::{Deserialize, Serialize};

use crate::domain::semantic::{
    CompilerHintRecord, FailureClassKind, FailureScopeKind, SemanticStateSummary,
};

/// Required rustc/compiler analysis fact that can be absent.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustcAnalysisFactKind {
    Complete,
    CargoProject,
    RustFileCount,
    SourceFiles,
}

/// Structured facts produced by successful rustc/compiler analysis.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RustcAnalysisFacts {
    pub crate_name: Option<String>,
    pub entrypoint_kind: Option<String>,
    pub rust_file_count: usize,
    pub source_files: Vec<String>,
    pub module_gaps: Vec<String>,
}

impl RustcAnalysisFacts {
    pub fn from_summary(summary: &SemanticStateSummary) -> Self {
        Self {
            crate_name: summary.crate_name.clone(),
            entrypoint_kind: summary.entrypoint_kind.clone(),
            rust_file_count: summary.rust_file_count.unwrap_or_default(),
            source_files: summary.source_files.clone(),
            module_gaps: summary.module_gaps.clone(),
        }
    }
}

/// Typed compiler failure returned by the rustc/compiler analysis boundary.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RustcCompilerFailure {
    pub failure_class: FailureClassKind,
    pub failure_scope: FailureScopeKind,
    pub hints: Vec<CompilerHintRecord>,
}

/// Typed missing-facts boundary for incomplete rustc/compiler analysis.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RustcMissingFacts {
    pub required: Vec<RustcAnalysisFactKind>,
    pub reason: String,
}

/// Typed invocation failure for rustc/compiler command execution failures.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RustcInvocationFailure {
    pub message: String,
}

/// Exhaustive outcome boundary for rustc/compiler analysis results.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "outcome", content = "data")]
pub enum RustcAnalysisOutcome {
    SuccessfulFacts(RustcAnalysisFacts),
    CompilerFailure(RustcCompilerFailure),
    MissingFacts(RustcMissingFacts),
    InvocationFailure(RustcInvocationFailure),
}

impl RustcAnalysisOutcome {
    pub fn from_summary(summary: &SemanticStateSummary) -> Self {
        if summary.compiler_repair_required
            || summary.failure_class.is_some()
            || !summary.compiler_hints.is_empty()
        {
            return Self::CompilerFailure(RustcCompilerFailure {
                failure_class: summary
                    .failure_class
                    .as_deref()
                    .and_then(FailureClassKind::from_str)
                    .unwrap_or(FailureClassKind::GenericCompilerFailure),
                failure_scope: summary
                    .failure_scope
                    .as_deref()
                    .and_then(FailureScopeKind::from_str)
                    .or_else(|| {
                        summary
                            .compiler_hints
                            .iter()
                            .find_map(CompilerHintRecord::failure_scope_enum)
                    })
                    .unwrap_or(FailureScopeKind::Tooling),
                hints: summary.compiler_hints.clone(),
            });
        }

        let mut required = Vec::new();
        if !summary.complete {
            required.push(RustcAnalysisFactKind::Complete);
        }
        if !summary.cargo_project {
            required.push(RustcAnalysisFactKind::CargoProject);
        }
        if summary.rust_file_count.is_none() {
            required.push(RustcAnalysisFactKind::RustFileCount);
        }
        if summary.source_files.is_empty() {
            required.push(RustcAnalysisFactKind::SourceFiles);
        }
        if !required.is_empty() {
            return Self::MissingFacts(RustcMissingFacts {
                required,
                reason: "rustc analysis facts are incomplete".to_string(),
            });
        }

        Self::SuccessfulFacts(RustcAnalysisFacts::from_summary(summary))
    }

    pub fn invocation_failure(message: impl Into<String>) -> Self {
        Self::InvocationFailure(RustcInvocationFailure {
            message: message.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::semantic::{CompilerHintKind, FailureScopeKind};

    #[test]
    fn typed_outcome_returns_successful_facts() {
        let summary = SemanticStateSummary {
            complete: true,
            cargo_project: true,
            crate_name: Some("ai".to_string()),
            rust_file_count: Some(2),
            source_files: vec!["src/lib.rs".to_string(), "src/main.rs".to_string()],
            ..Default::default()
        };

        match RustcAnalysisOutcome::from_summary(&summary) {
            RustcAnalysisOutcome::SuccessfulFacts(facts) => {
                assert_eq!(facts.crate_name.as_deref(), Some("ai"));
                assert_eq!(facts.rust_file_count, 2);
                assert_eq!(facts.source_files.len(), 2);
            }
            other => panic!("expected successful facts, got {other:?}"),
        }
    }

    #[test]
    fn typed_outcome_returns_compiler_failure() {
        let mut summary = SemanticStateSummary::default();
        summary.failure_class = Some(FailureClassKind::MissingModule.as_str().to_string());
        summary.compiler_hints.push(
            CompilerHintRecord::new(
                CompilerHintKind::MissingModule,
                "module declared but no file exists",
                "add src/foo.rs",
                vec!["src/foo.rs".to_string()],
            )
            .with_failure_scope(FailureScopeKind::Localized),
        );

        match RustcAnalysisOutcome::from_summary(&summary) {
            RustcAnalysisOutcome::CompilerFailure(failure) => {
                assert_eq!(failure.failure_class, FailureClassKind::MissingModule);
                assert_eq!(failure.failure_scope, FailureScopeKind::Localized);
                assert_eq!(failure.hints.len(), 1);
            }
            other => panic!("expected compiler failure, got {other:?}"),
        }
    }

    #[test]
    fn typed_outcome_returns_missing_facts() {
        let summary = SemanticStateSummary {
            complete: true,
            cargo_project: true,
            ..Default::default()
        };

        match RustcAnalysisOutcome::from_summary(&summary) {
            RustcAnalysisOutcome::MissingFacts(missing) => {
                assert_eq!(
                    missing.required,
                    vec![
                        RustcAnalysisFactKind::RustFileCount,
                        RustcAnalysisFactKind::SourceFiles
                    ]
                );
            }
            other => panic!("expected missing facts, got {other:?}"),
        }
    }

    #[test]
    fn typed_outcome_returns_invocation_failure() {
        match RustcAnalysisOutcome::invocation_failure("rustc executable not found") {
            RustcAnalysisOutcome::InvocationFailure(failure) => {
                assert_eq!(failure.message, "rustc executable not found");
            }
            other => panic!("expected invocation failure, got {other:?}"),
        }
    }
}
