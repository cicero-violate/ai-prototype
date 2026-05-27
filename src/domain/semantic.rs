//! Semantic state and compiler/failure classification.
//!
//! Adapted from canon-semantic-state/src/lib.rs. Provides structured
//! compiler hints, failure scope/class, and workspace facts for planner context.
//! Updates are stored as receipts or derived projections; this module only
//! defines the data types and serialisation helpers.

use serde::{Deserialize, Serialize};

/// Structured classification of a compiler or tooling failure hint.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompilerHintKind {
    MissingModule,
    DeadCodeForbidConflict,
    MissingEntrypoint,
    UnresolvedImport,
    MissingSymbol,
    DuplicateDefinition,
    TraitBoundFailure,
    GenericCompilerFailure,
}

impl CompilerHintKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MissingModule => "missing_module",
            Self::DeadCodeForbidConflict => "dead_code_forbid_conflict",
            Self::MissingEntrypoint => "missing_entrypoint",
            Self::UnresolvedImport => "unresolved_import",
            Self::MissingSymbol => "missing_symbol",
            Self::DuplicateDefinition => "duplicate_definition",
            Self::TraitBoundFailure => "trait_bound_failure",
            Self::GenericCompilerFailure => "generic_compiler_failure",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "missing_module" => Some(Self::MissingModule),
            "dead_code_forbid_conflict" => Some(Self::DeadCodeForbidConflict),
            "missing_entrypoint" => Some(Self::MissingEntrypoint),
            "unresolved_import" => Some(Self::UnresolvedImport),
            "missing_symbol" => Some(Self::MissingSymbol),
            "duplicate_definition" => Some(Self::DuplicateDefinition),
            "trait_bound_failure" => Some(Self::TraitBoundFailure),
            "generic_compiler_failure" => Some(Self::GenericCompilerFailure),
            _ => None,
        }
    }
}

/// How widely a failure is scoped within the workspace.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureScopeKind {
    None,
    Localized,
    Workspace,
    Tooling,
}

impl FailureScopeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Localized => "localized",
            Self::Workspace => "workspace",
            Self::Tooling => "tooling",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "none" => Some(Self::None),
            "localized" => Some(Self::Localized),
            "workspace" => Some(Self::Workspace),
            "tooling" => Some(Self::Tooling),
            _ => None,
        }
    }
}

/// Stable failure class used in plan generation and recovery prompts.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureClassKind {
    MissingModule,
    DeadCodeForbidConflict,
    MissingEntrypoint,
    UnresolvedImport,
    MissingSymbol,
    DuplicateDefinition,
    TraitBoundFailure,
    GenericCompilerFailure,
    NoActionableFailure,
}

impl FailureClassKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MissingModule => "missing_module",
            Self::DeadCodeForbidConflict => "dead_code_forbid_conflict",
            Self::MissingEntrypoint => "missing_entrypoint",
            Self::UnresolvedImport => "unresolved_import",
            Self::MissingSymbol => "missing_symbol",
            Self::DuplicateDefinition => "duplicate_definition",
            Self::TraitBoundFailure => "trait_bound_failure",
            Self::GenericCompilerFailure => "generic_compiler_failure",
            Self::NoActionableFailure => "no_actionable_failure",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "missing_module" => Some(Self::MissingModule),
            "dead_code_forbid_conflict" => Some(Self::DeadCodeForbidConflict),
            "missing_entrypoint" => Some(Self::MissingEntrypoint),
            "unresolved_import" => Some(Self::UnresolvedImport),
            "missing_symbol" => Some(Self::MissingSymbol),
            "duplicate_definition" => Some(Self::DuplicateDefinition),
            "trait_bound_failure" => Some(Self::TraitBoundFailure),
            "generic_compiler_failure" => Some(Self::GenericCompilerFailure),
            "no_actionable_failure" => Some(Self::NoActionableFailure),
            _ => None,
        }
    }
}

/// One structured compiler hint, suitable for inclusion in a planner context prompt.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CompilerHintRecord {
    pub kind: String,
    pub summary: String,
    pub suggested_repair: String,
    pub target_files: Vec<String>,
    pub failure_scope: String,
}

impl CompilerHintRecord {
    pub fn new(
        kind: CompilerHintKind,
        summary: impl Into<String>,
        suggested_repair: impl Into<String>,
        target_files: Vec<String>,
    ) -> Self {
        Self {
            kind: kind.as_str().to_string(),
            summary: summary.into(),
            suggested_repair: suggested_repair.into(),
            target_files,
            failure_scope: FailureScopeKind::None.as_str().to_string(),
        }
    }

    pub fn with_failure_scope(mut self, scope: FailureScopeKind) -> Self {
        self.failure_scope = scope.as_str().to_string();
        self
    }

    pub fn kind_enum(&self) -> Option<CompilerHintKind> {
        CompilerHintKind::from_str(&self.kind)
    }

    pub fn failure_scope_enum(&self) -> Option<FailureScopeKind> {
        FailureScopeKind::from_str(&self.failure_scope)
    }

    /// Compact single-line rendering for inclusion in a planner prompt.
    pub fn render_line(&self) -> String {
        let targets = if self.target_files.is_empty() {
            "none".to_string()
        } else {
            self.target_files.join("|")
        };
        format!(
            "kind={} scope={} targets={} summary={} repair={}",
            self.kind, self.failure_scope, targets, self.summary, self.suggested_repair
        )
    }
}

/// Workspace-level semantic state summary.
/// Serialized as a derived projection — never the primary truth store.
/// Callers feed `to_workspace_facts()` into plan generation prompts.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SemanticStateSummary {
    pub version: u32,
    pub complete: bool,
    pub target_root: Option<String>,
    pub path_exists: bool,
    pub repo_initialized: bool,
    pub cargo_project: bool,
    pub crate_name: Option<String>,
    pub entrypoint_kind: Option<String>,
    pub rust_file_count: Option<usize>,
    pub source_files: Vec<String>,
    pub module_gaps: Vec<String>,
    pub planning_preconditions: Vec<String>,
    pub repair_intents: Vec<String>,
    pub compiler_hints: Vec<CompilerHintRecord>,
    pub validation_blocked_by_preconditions: bool,
    pub compiler_repair_required: bool,
    pub failure_class: Option<String>,
    pub failure_scope: Option<String>,
}

impl SemanticStateSummary {
    pub const VERSION: u32 = 1;

    /// Emit key=value workspace facts suitable for injection into a prompt.
    pub fn to_workspace_facts(&self) -> Vec<String> {
        let mut facts = Vec::new();
        facts.push(format!("semantic.version={}", self.version));
        facts.push(format!("semantic.complete={}", self.complete));
        if let Some(r) = &self.target_root {
            facts.push(format!("semantic.target_root={r}"));
        }
        facts.push(format!("semantic.path_exists={}", self.path_exists));
        facts.push(format!(
            "semantic.repo_initialized={}",
            self.repo_initialized
        ));
        facts.push(format!("semantic.cargo_project={}", self.cargo_project));
        if let Some(n) = &self.crate_name {
            facts.push(format!("semantic.crate_name={n}"));
        }
        if let Some(k) = &self.entrypoint_kind {
            facts.push(format!("semantic.entrypoint_kind={k}"));
        }
        if let Some(c) = self.rust_file_count {
            facts.push(format!("semantic.rust_file_count={c}"));
        }
        for f in &self.source_files {
            facts.push(format!("semantic.source_file={f}"));
        }
        for g in &self.module_gaps {
            facts.push(format!("semantic.module_gap={g}"));
        }
        for p in &self.planning_preconditions {
            facts.push(format!("semantic.planning_precondition={p}"));
        }
        for i in &self.repair_intents {
            facts.push(format!("semantic.repair_intent={i}"));
        }
        for h in &self.compiler_hints {
            facts.push(format!("semantic.compiler_hint={}", h.render_line()));
        }
        facts.push(format!(
            "semantic.validation_blocked={}",
            self.validation_blocked_by_preconditions
        ));
        facts.push(format!(
            "semantic.compiler_repair_required={}",
            self.compiler_repair_required
        ));
        if let Some(v) = &self.failure_class {
            facts.push(format!("semantic.failure_class={v}"));
        }
        if let Some(v) = &self.failure_scope {
            facts.push(format!("semantic.failure_scope={v}"));
        }
        facts
    }

    /// Record a generic compiler capture failure.
    pub fn apply_rustc_capture_failure(&mut self, message: &str) {
        self.compiler_repair_required = true;
        self.failure_class = Some(
            FailureClassKind::GenericCompilerFailure
                .as_str()
                .to_string(),
        );
        let summary = format!("rustc capture failed: {message}");
        if !self.compiler_hints.iter().any(|h| {
            h.kind == FailureClassKind::GenericCompilerFailure.as_str() && h.summary == summary
        }) {
            self.compiler_hints.push(
                CompilerHintRecord::new(
                    CompilerHintKind::GenericCompilerFailure,
                    &summary,
                    "re-run cargo check and inspect errors",
                    Vec::new(),
                )
                .with_failure_scope(FailureScopeKind::Tooling),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiler_hint_kind_round_trips_through_str() {
        let kinds = [
            CompilerHintKind::MissingModule,
            CompilerHintKind::DeadCodeForbidConflict,
            CompilerHintKind::UnresolvedImport,
            CompilerHintKind::GenericCompilerFailure,
        ];
        for kind in &kinds {
            let s = kind.as_str();
            assert_eq!(CompilerHintKind::from_str(s).as_ref(), Some(kind));
        }
    }

    #[test]
    fn failure_scope_round_trips_through_str() {
        let scopes = [
            FailureScopeKind::None,
            FailureScopeKind::Localized,
            FailureScopeKind::Workspace,
            FailureScopeKind::Tooling,
        ];
        for scope in &scopes {
            let s = scope.as_str();
            assert_eq!(FailureScopeKind::from_str(s).as_ref(), Some(scope));
        }
    }

    #[test]
    fn compiler_hint_record_render_line_includes_all_fields() {
        let rec = CompilerHintRecord::new(
            CompilerHintKind::MissingModule,
            "mod foo missing",
            "add mod foo",
            vec!["src/foo.rs".to_string()],
        )
        .with_failure_scope(FailureScopeKind::Localized);
        let line = rec.render_line();
        assert!(line.contains("kind=missing_module"), "{line}");
        assert!(line.contains("scope=localized"), "{line}");
        assert!(line.contains("src/foo.rs"), "{line}");
    }

    #[test]
    fn semantic_state_to_workspace_facts_includes_key_fields() {
        let mut s = SemanticStateSummary {
            version: SemanticStateSummary::VERSION,
            complete: true,
            cargo_project: true,
            crate_name: Some("ai".to_string()),
            ..Default::default()
        };
        s.compiler_hints.push(CompilerHintRecord::new(
            CompilerHintKind::UnresolvedImport,
            "missing crate",
            "add dep",
            Vec::new(),
        ));
        let facts = s.to_workspace_facts();
        assert!(facts.iter().any(|f| f == "semantic.complete=true"));
        assert!(facts.iter().any(|f| f == "semantic.crate_name=ai"));
        assert!(facts
            .iter()
            .any(|f| f.starts_with("semantic.compiler_hint=")));
    }

    #[test]
    fn apply_rustc_capture_failure_sets_repair_required() {
        let mut s = SemanticStateSummary::default();
        s.apply_rustc_capture_failure("linker failed");
        assert!(s.compiler_repair_required);
        assert_eq!(s.failure_class.as_deref(), Some("generic_compiler_failure"));
        assert_eq!(s.compiler_hints.len(), 1);
        // Idempotent: second call with same message does not double-add.
        s.apply_rustc_capture_failure("linker failed");
        assert_eq!(s.compiler_hints.len(), 1);
    }

    #[test]
    fn semantic_state_round_trips_through_json() {
        let mut s = SemanticStateSummary {
            version: 1,
            complete: false,
            cargo_project: true,
            ..Default::default()
        };
        s.apply_rustc_capture_failure("test failure");
        let json = serde_json::to_string(&s).unwrap();
        let loaded: SemanticStateSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(s, loaded);
    }
}
