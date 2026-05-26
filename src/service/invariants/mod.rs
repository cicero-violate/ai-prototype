//! Invariant mining data model and deterministic registry storage.
//!
//! Mining rules may be heuristic, but this module keeps candidate identity,
//! registry persistence, and promotion state replay-friendly and stable.

pub mod event_loop;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub const INVARIANT_REGISTRY_RELATIVE_PATH: &str = "state/invariants/registry.json";
pub const INVARIANT_CANDIDATES_RELATIVE_DIR: &str = "state/invariants/candidates";
pub const INVARIANT_VALIDATED_RELATIVE_DIR: &str = "state/invariants/validated";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InvariantCandidate {
    pub id: String,
    pub status: InvariantStatus,
    pub scope: InvariantScope,
    pub predicate: PredicateExpr,
    pub evidence_refs: Vec<EventRef>,
    pub support: u64,
    pub violations: u64,
    pub confidence_milli: u16,
    pub summary: String,
}

impl InvariantCandidate {
    pub fn new(
        scope: InvariantScope,
        predicate: PredicateExpr,
        summary: impl Into<String>,
    ) -> Self {
        let summary = summary.into();
        let id = stable_candidate_id(&scope, &predicate);
        Self {
            id,
            status: InvariantStatus::Candidate,
            scope,
            predicate,
            evidence_refs: Vec::new(),
            support: 0,
            violations: 0,
            confidence_milli: 0,
            summary,
        }
    }

    pub fn sort_for_determinism(&mut self) {
        self.evidence_refs.sort();
        self.evidence_refs.dedup();
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InvariantStatus {
    Candidate,
    Validated,
    Promoted,
    Rejected,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum InvariantScope {
    Runtime,
    TaskRunner,
    Recovery,
    Planner,
    ScoreSeeder,
    WorkerHealth,
    Replay,
    WorkspacePath,
    Custom(String),
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum PredicateExpr {
    MissingAcceptedEvidenceImpliesFailClaim,
    WorkerHealthReadyRequiresSession,
    ActiveScoreSeededCycleBlocksNewCycle,
    KnownScoreSeededCratesMapToExistingFiles,
    WaveDispatchMayResetPendingCount,
    Custom { name: String, expression: String },
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct EventRef {
    pub source: String,
    pub line: Option<u64>,
    pub event_seq: Option<u64>,
    pub hash: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InvariantValidation {
    pub candidate_id: String,
    pub passed: bool,
    pub support: u64,
    pub violations: u64,
    pub replay_input_hashes: Vec<String>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct InvariantRegistry {
    pub version: u32,
    pub invariants: Vec<InvariantCandidate>,
}

impl InvariantRegistry {
    pub fn empty() -> Self {
        Self {
            version: 1,
            invariants: Vec::new(),
        }
    }

    pub fn sort_for_determinism(&mut self) {
        for invariant in &mut self.invariants {
            invariant.sort_for_determinism();
        }
        self.invariants.sort_by(|a, b| a.id.cmp(&b.id));
        self.invariants.dedup_by(|a, b| a.id == b.id);
    }

    pub fn upsert(&mut self, mut candidate: InvariantCandidate) {
        candidate.sort_for_determinism();
        match self
            .invariants
            .iter_mut()
            .find(|existing| existing.id == candidate.id)
        {
            Some(existing) => *existing = candidate,
            None => self.invariants.push(candidate),
        }
        self.sort_for_determinism();
    }
}

pub fn registry_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(INVARIANT_REGISTRY_RELATIVE_PATH)
}

pub fn candidates_dir(workspace_root: &Path) -> PathBuf {
    workspace_root.join(INVARIANT_CANDIDATES_RELATIVE_DIR)
}

pub fn validated_dir(workspace_root: &Path) -> PathBuf {
    workspace_root.join(INVARIANT_VALIDATED_RELATIVE_DIR)
}

pub fn load_registry(workspace_root: &Path) -> Result<InvariantRegistry, String> {
    let path = registry_path(workspace_root);
    if !path.exists() {
        return Ok(InvariantRegistry::empty());
    }
    let bytes = fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let mut registry: InvariantRegistry =
        serde_json::from_slice(&bytes).map_err(|e| format!("parse {}: {e}", path.display()))?;
    if registry.version == 0 {
        registry.version = 1;
    }
    registry.sort_for_determinism();
    Ok(registry)
}

pub fn save_registry(workspace_root: &Path, registry: &InvariantRegistry) -> Result<(), String> {
    let path = registry_path(workspace_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    let mut normalized = registry.clone();
    if normalized.version == 0 {
        normalized.version = 1;
    }
    normalized.sort_for_determinism();
    let bytes = serde_json::to_vec_pretty(&normalized)
        .map_err(|e| format!("serialize {}: {e}", path.display()))?;
    fs::write(&path, bytes).map_err(|e| format!("write {}: {e}", path.display()))
}

pub fn write_candidate_artifact(
    workspace_root: &Path,
    candidate: &InvariantCandidate,
) -> Result<PathBuf, String> {
    let dir = candidates_dir(workspace_root);
    fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    let path = dir.join(format!("{}.json", candidate.id));
    let mut normalized = candidate.clone();
    normalized.sort_for_determinism();
    let bytes = serde_json::to_vec_pretty(&normalized)
        .map_err(|e| format!("serialize {}: {e}", path.display()))?;
    fs::write(&path, bytes).map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(path)
}

pub fn write_validation_artifact(
    workspace_root: &Path,
    validation: &InvariantValidation,
) -> Result<PathBuf, String> {
    let dir = validated_dir(workspace_root);
    fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    let path = dir.join(format!("{}.json", validation.candidate_id));
    let mut normalized = validation.clone();
    normalized.replay_input_hashes.sort();
    normalized.replay_input_hashes.dedup();
    let bytes = serde_json::to_vec_pretty(&normalized)
        .map_err(|e| format!("serialize {}: {e}", path.display()))?;
    fs::write(&path, bytes).map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(path)
}

pub fn write_validation_evidence(
    workspace_root: &Path,
    validation: &InvariantValidation,
) -> Result<PathBuf, String> {
    let path = workspace_root.join(format!(
        "state/agent-evidence/invariant-{}.md",
        validation.candidate_id
    ));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    let verdict = if validation.passed { "pass" } else { "fail" };
    let body = format!(
        "# Invariant Validation Evidence\n\n\
         - candidate_id: `{}`\n\
         - verdict: `{}`\n\
         - support: `{}`\n\
         - violations: `{}`\n\
         - summary: {}\n\n\
         ## Replay Inputs\n\n{}\n",
        validation.candidate_id,
        verdict,
        validation.support,
        validation.violations,
        validation.summary,
        validation
            .replay_input_hashes
            .iter()
            .map(|hash| format!("- `{hash}`"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    fs::write(&path, body).map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(path)
}

pub fn mine_workspace(workspace_root: &Path) -> Result<Vec<InvariantCandidate>, String> {
    let mut candidates = fixed_candidates();
    let reasoning = read_lines(workspace_root.join("state/reasoning-loop.jsonl"))?;
    let plan = read_json_optional(workspace_root.join("state/plan.json"))?;
    let _receipts =
        read_lines(workspace_root.join("state/agent_state/sse-chunks/agent-turn-receipts.ndjson"))?;
    let tlog = read_lines(workspace_root.join("state/tlog/canon-agent.tlog.ndjson"))?;

    for candidate in &mut candidates {
        match candidate.predicate {
            PredicateExpr::MissingAcceptedEvidenceImpliesFailClaim => {
                mine_missing_evidence_claims(candidate, &reasoning);
            }
            PredicateExpr::WorkerHealthReadyRequiresSession => {
                mine_worker_health(candidate, &tlog);
            }
            PredicateExpr::ActiveScoreSeededCycleBlocksNewCycle => {
                mine_active_seed_cycles(candidate, plan.as_ref());
            }
            PredicateExpr::KnownScoreSeededCratesMapToExistingFiles => {
                mine_known_crate_paths(candidate, workspace_root, plan.as_ref());
            }
            PredicateExpr::WaveDispatchMayResetPendingCount => {
                mine_wave_dispatch(candidate, &tlog);
            }
            PredicateExpr::Custom { .. } => {}
        }
        candidate.confidence_milli = confidence_milli(candidate.support, candidate.violations);
        candidate.sort_for_determinism();
    }

    candidates.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(candidates)
}

pub fn mine_and_write_workspace(workspace_root: &Path) -> Result<Vec<InvariantCandidate>, String> {
    let candidates = mine_workspace(workspace_root)?;
    let mut registry = load_registry(workspace_root)?;
    for candidate in &candidates {
        write_candidate_artifact(workspace_root, candidate)?;
        registry.upsert(candidate.clone());
    }
    save_registry(workspace_root, &registry)?;
    Ok(candidates)
}

pub fn validate_workspace(
    workspace_root: &Path,
    min_support: u64,
) -> Result<Vec<InvariantValidation>, String> {
    let candidates = if load_registry(workspace_root)?.invariants.is_empty() {
        mine_and_write_workspace(workspace_root)?
    } else {
        load_registry(workspace_root)?.invariants
    };
    let replay_hashes = replay_input_hashes(workspace_root)?;
    let mut validations = Vec::new();
    let mut registry = load_registry(workspace_root)?;

    for candidate in candidates {
        let passed = candidate.support >= min_support && candidate.violations == 0;
        let validation = InvariantValidation {
            candidate_id: candidate.id.clone(),
            passed,
            support: candidate.support,
            violations: candidate.violations,
            replay_input_hashes: replay_hashes.clone(),
            summary: if passed {
                format!("validated with support={} violations=0", candidate.support)
            } else {
                format!(
                    "not validated: support={} min_support={} violations={}",
                    candidate.support, min_support, candidate.violations
                )
            },
        };
        let mut updated = candidate.clone();
        updated.status = if passed {
            InvariantStatus::Validated
        } else {
            InvariantStatus::Rejected
        };
        registry.upsert(updated);
        write_validation_artifact(workspace_root, &validation)?;
        write_validation_evidence(workspace_root, &validation)?;
        validations.push(validation);
    }

    validations.sort_by(|a, b| a.candidate_id.cmp(&b.candidate_id));
    save_registry(workspace_root, &registry)?;
    Ok(validations)
}

pub fn promote_workspace(
    workspace_root: &Path,
    min_support: u64,
) -> Result<Vec<InvariantCandidate>, String> {
    let mut registry = load_registry(workspace_root)?;
    if registry.invariants.is_empty() {
        let _ = validate_workspace(workspace_root, min_support)?;
        registry = load_registry(workspace_root)?;
    }
    let mut promoted = Vec::new();
    for invariant in &mut registry.invariants {
        if matches!(
            invariant.status,
            InvariantStatus::Validated | InvariantStatus::Promoted
        ) && invariant.support >= min_support
            && invariant.violations == 0
        {
            invariant.status = InvariantStatus::Promoted;
            promoted.push(invariant.clone());
        }
    }
    registry.sort_for_determinism();
    promoted.sort_by(|a, b| a.id.cmp(&b.id));
    save_registry(workspace_root, &registry)?;
    Ok(promoted)
}

pub fn read_workspace_summary(workspace_root: &Path, limit: usize) -> Result<Value, String> {
    let registry = load_registry(workspace_root)?;
    let mut invariants = registry.invariants;
    invariants.sort_by(|a, b| {
        b.status
            .cmp(&a.status)
            .then(b.support.cmp(&a.support))
            .then(a.id.cmp(&b.id))
    });
    let invariants: Vec<_> = invariants.into_iter().take(limit).collect();
    Ok(json!({
        "registry_path": INVARIANT_REGISTRY_RELATIVE_PATH,
        "count": invariants.len(),
        "invariants": invariants,
    }))
}

pub fn promoted_invariant_prompt_block(workspace_root: &Path, limit: usize) -> Option<String> {
    let mut promoted: Vec<_> = load_registry(workspace_root)
        .ok()?
        .invariants
        .into_iter()
        .filter(|inv| inv.status == InvariantStatus::Promoted)
        .collect();
    if promoted.is_empty() {
        return None;
    }
    promoted.sort_by(|a, b| b.support.cmp(&a.support).then(a.id.cmp(&b.id)));
    let lines: Vec<String> = promoted
        .into_iter()
        .take(limit)
        .map(|inv| format!("- {}: {} (support={})", inv.id, inv.summary, inv.support))
        .collect();
    Some(lines.join("\n"))
}

pub fn stable_candidate_id(scope: &InvariantScope, predicate: &PredicateExpr) -> String {
    let identity = serde_json::json!({
        "scope": scope,
        "predicate": predicate,
    });
    let bytes = serde_json::to_vec(&identity).expect("invariant identity should serialize");
    let digest = Sha256::digest(bytes);
    format!("inv-{}", hex_prefix(&digest, 12))
}

fn hex_prefix(bytes: &[u8], len: usize) -> String {
    let mut out = String::with_capacity(len);
    for byte in bytes {
        if out.len() >= len {
            break;
        }
        out.push(hex_digit(byte >> 4));
        if out.len() >= len {
            break;
        }
        out.push(hex_digit(byte & 0x0f));
    }
    out
}

fn hex_digit(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        10..=15 => (b'a' + (nibble - 10)) as char,
        _ => unreachable!("nibble is masked to four bits"),
    }
}

fn fixed_candidates() -> Vec<InvariantCandidate> {
    vec![
        InvariantCandidate::new(
            InvariantScope::TaskRunner,
            PredicateExpr::MissingAcceptedEvidenceImpliesFailClaim,
            "missing accepted evidence must fail or retry a task claim, not complete it",
        ),
        InvariantCandidate::new(
            InvariantScope::WorkerHealth,
            PredicateExpr::WorkerHealthReadyRequiresSession,
            "worker health readiness requires an installed runtime session",
        ),
        InvariantCandidate::new(
            InvariantScope::ScoreSeeder,
            PredicateExpr::ActiveScoreSeededCycleBlocksNewCycle,
            "an active score-seeded improvement cycle blocks adding another score-seeded cycle",
        ),
        InvariantCandidate::new(
            InvariantScope::WorkspacePath,
            PredicateExpr::KnownScoreSeededCratesMapToExistingFiles,
            "known score-seeded crates must point to existing workspace files",
        ),
        InvariantCandidate::new(
            InvariantScope::Replay,
            PredicateExpr::WaveDispatchMayResetPendingCount,
            "wave dispatch replay may reset pending count to the dispatched wave size",
        ),
    ]
}

fn mine_missing_evidence_claims(candidate: &mut InvariantCandidate, lines: &[LineRecord]) {
    let mut by_trace: BTreeMap<String, (bool, bool)> = BTreeMap::new();
    for line in lines {
        let text = &line.text;
        if !(text.contains("\"node_id\":\"") && text.contains("\"Verification:")) {
            continue;
        }
        let trace = json_string_field(text, "trace_id").unwrap_or_else(|| line.line.to_string());
        let entry = by_trace.entry(trace).or_default();
        if text.contains("\"next_supervisor_transition\":\"fail_claim\"")
            && text.contains("\"passed\":false")
        {
            entry.0 = true;
            candidate
                .evidence_refs
                .push(line.event_ref("reasoning_loop"));
        }
        if text.contains("\"next_supervisor_transition\":\"complete_claim\"")
            && text.contains("\"passed\":false")
        {
            entry.1 = true;
            candidate
                .evidence_refs
                .push(line.event_ref("reasoning_loop"));
        }
    }
    candidate.support = by_trace.values().filter(|(fail, _)| *fail).count() as u64;
    candidate.violations = by_trace.values().filter(|(_, bad)| *bad).count() as u64;
}

fn mine_worker_health(candidate: &mut InvariantCandidate, lines: &[LineRecord]) {
    for line in lines {
        if line.text.contains("WorkerLoading") || line.text.contains("session") {
            candidate.support += 1;
            candidate.evidence_refs.push(line.event_ref("tlog"));
        }
    }
}

fn mine_active_seed_cycles(candidate: &mut InvariantCandidate, plan: Option<&Value>) {
    let Some(plan) = plan else { return };
    let Some(nodes) = plan.get("nodes").and_then(Value::as_array) else {
        return;
    };
    let mut active_cycles = BTreeSet::new();
    for node in nodes {
        let id = node.get("id").and_then(Value::as_str).unwrap_or_default();
        let status = node
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if is_score_seeded_node_id(id) && !is_terminal_status(status) {
            if let Some(cycle) = cycle_suffix(id) {
                active_cycles.insert(cycle);
            }
        }
    }
    if active_cycles.is_empty() {
        return;
    }
    candidate.support = active_cycles.len() as u64;
    candidate.violations = active_cycles.len().saturating_sub(1) as u64;
    candidate.evidence_refs.push(EventRef {
        source: "state/plan.json".to_string(),
        line: None,
        event_seq: None,
        hash: Some(hash_string(&format!("{:?}", active_cycles))),
    });
}

fn mine_known_crate_paths(
    candidate: &mut InvariantCandidate,
    workspace_root: &Path,
    plan: Option<&Value>,
) {
    let Some(plan) = plan else { return };
    let Some(nodes) = plan.get("nodes").and_then(Value::as_array) else {
        return;
    };
    for node in nodes {
        let id = node.get("id").and_then(Value::as_str).unwrap_or_default();
        if !is_score_seeded_node_id(id) {
            continue;
        }
        let Some(files) = node.get("files").and_then(Value::as_array) else {
            continue;
        };
        for file in files.iter().filter_map(Value::as_str) {
            if workspace_root.join(file).exists() {
                candidate.support += 1;
            } else {
                candidate.violations += 1;
            }
            candidate.evidence_refs.push(EventRef {
                source: "state/plan.json".to_string(),
                line: None,
                event_seq: None,
                hash: Some(hash_string(file)),
            });
        }
    }
}

fn mine_wave_dispatch(candidate: &mut InvariantCandidate, lines: &[LineRecord]) {
    for line in lines {
        if line.text.contains("WaveDispatched") || line.text.contains("wave_pending") {
            candidate.support += 1;
            candidate.evidence_refs.push(line.event_ref("tlog"));
        }
    }
}

fn confidence_milli(support: u64, violations: u64) -> u16 {
    if support == 0 && violations == 0 {
        return 0;
    }
    ((support * 1000) / (support + violations)) as u16
}

#[derive(Clone)]
struct LineRecord {
    line: u64,
    text: String,
}

impl LineRecord {
    fn event_ref(&self, source: &str) -> EventRef {
        EventRef {
            source: source.to_string(),
            line: Some(self.line),
            event_seq: json_u64_field(&self.text, "seq"),
            hash: Some(hash_string(&self.text)),
        }
    }
}

fn read_lines(path: PathBuf) -> Result<Vec<LineRecord>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = fs::File::open(&path).map_err(|e| format!("open {}: {e}", path.display()))?;
    let mut out = Vec::new();
    for (idx, line) in BufReader::new(file).lines().enumerate() {
        out.push(LineRecord {
            line: idx as u64 + 1,
            text: line.map_err(|e| format!("read {}: {e}", path.display()))?,
        });
    }
    Ok(out)
}

fn read_json_optional(path: PathBuf) -> Result<Option<Value>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|e| format!("parse {}: {e}", path.display()))
}

fn replay_input_hashes(workspace_root: &Path) -> Result<Vec<String>, String> {
    let mut hashes = Vec::new();
    for rel in [
        "state/tlog/canon-agent.tlog.ndjson",
        "state/reasoning-loop.jsonl",
        "state/plan.json",
        "state/agent_state/sse-chunks/agent-turn-receipts.ndjson",
    ] {
        let path = workspace_root.join(rel);
        if path.exists() {
            let bytes = fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
            hashes.push(format!("{rel}:{}", hash_bytes(&bytes)));
        }
    }
    hashes.sort();
    Ok(hashes)
}

fn json_string_field(text: &str, field: &str) -> Option<String> {
    let value: Value = serde_json::from_str(text).ok()?;
    value.get(field).map(|v| {
        v.as_str()
            .map(str::to_string)
            .unwrap_or_else(|| v.to_string())
    })
}

fn json_u64_field(text: &str, field: &str) -> Option<u64> {
    serde_json::from_str::<Value>(text)
        .ok()?
        .get(field)?
        .as_u64()
}

fn is_score_seeded_node_id(id: &str) -> bool {
    id.starts_with("improve-") || id.starts_with("enable-phase-edge-emission-cycle-")
}

fn is_terminal_status(status: &str) -> bool {
    matches!(status, "done" | "failed" | "skipped")
}

fn cycle_suffix(id: &str) -> Option<u64> {
    id.rsplit_once("-cycle-")?.1.parse().ok()
}

fn hash_string(value: &str) -> String {
    hash_bytes(value.as_bytes())
}

fn hash_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    hex_prefix(&digest, 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_workspace() -> tempfile::TempDir {
        tempfile::Builder::new()
            .prefix("canon-invariants-test-")
            .tempdir_in({
                let d = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../state/tmp");
                fs::create_dir_all(&d).unwrap();
                d.canonicalize().unwrap()
            })
            .unwrap()
    }

    #[test]
    fn candidate_ids_are_stable_from_scope_and_predicate() {
        let first = InvariantCandidate::new(
            InvariantScope::TaskRunner,
            PredicateExpr::MissingAcceptedEvidenceImpliesFailClaim,
            "missing evidence cannot complete claims",
        );
        let second = InvariantCandidate::new(
            InvariantScope::TaskRunner,
            PredicateExpr::MissingAcceptedEvidenceImpliesFailClaim,
            "different prose does not change identity",
        );
        let different = InvariantCandidate::new(
            InvariantScope::WorkerHealth,
            PredicateExpr::WorkerHealthReadyRequiresSession,
            "worker health requires installed session",
        );

        assert_eq!(first.id, second.id);
        assert_ne!(first.id, different.id);
        assert!(first.id.starts_with("inv-"));
    }

    #[test]
    fn missing_registry_loads_as_empty_registry() {
        let tmp = tmp_workspace();
        let registry = load_registry(tmp.path()).expect("missing registry should load");

        assert_eq!(registry, InvariantRegistry::empty());
    }

    #[test]
    fn registry_load_save_round_trips_with_deterministic_order() {
        let tmp = tmp_workspace();
        let mut registry = InvariantRegistry::empty();
        registry.upsert(InvariantCandidate::new(
            InvariantScope::WorkerHealth,
            PredicateExpr::WorkerHealthReadyRequiresSession,
            "worker health requires installed session",
        ));
        registry.upsert(InvariantCandidate::new(
            InvariantScope::TaskRunner,
            PredicateExpr::MissingAcceptedEvidenceImpliesFailClaim,
            "missing evidence cannot complete claims",
        ));

        save_registry(tmp.path(), &registry).expect("registry save should succeed");
        let loaded = load_registry(tmp.path()).expect("registry load should succeed");
        let mut expected = registry.clone();
        expected.sort_for_determinism();

        assert_eq!(loaded, expected);
        assert!(registry_path(tmp.path()).exists());
    }

    #[test]
    fn artifact_writers_create_expected_files() {
        let tmp = tmp_workspace();
        let candidate = InvariantCandidate::new(
            InvariantScope::Replay,
            PredicateExpr::WaveDispatchMayResetPendingCount,
            "wave dispatch may reset pending count",
        );
        let candidate_path =
            write_candidate_artifact(tmp.path(), &candidate).expect("candidate artifact");
        let validation = InvariantValidation {
            candidate_id: candidate.id.clone(),
            passed: true,
            support: 3,
            violations: 0,
            replay_input_hashes: vec!["b".to_string(), "a".to_string(), "a".to_string()],
            summary: "passed replay".to_string(),
        };
        let validation_path =
            write_validation_artifact(tmp.path(), &validation).expect("validation artifact");
        let evidence_path =
            write_validation_evidence(tmp.path(), &validation).expect("validation evidence");

        assert_eq!(
            candidate_path,
            candidates_dir(tmp.path()).join(format!("{}.json", candidate.id))
        );
        assert_eq!(
            validation_path,
            validated_dir(tmp.path()).join(format!("{}.json", candidate.id))
        );
        assert!(candidate_path.exists());
        assert!(validation_path.exists());
        assert!(evidence_path.exists());
    }

    #[test]
    fn miner_detects_duplicate_active_cycles_and_bad_paths() {
        let tmp = tmp_workspace();
        fs::create_dir_all(tmp.path().join("real/src")).unwrap();
        fs::write(tmp.path().join("real/src/lib.rs"), "pub fn ok() {}\n").unwrap();
        fs::create_dir_all(tmp.path().join("state")).unwrap();
        fs::write(
            tmp.path().join("state/plan.json"),
            serde_json::to_vec(&json!({
                "version": 1,
                "nodes": [
                    {"id":"improve-structure-real-cycle-1", "status":"pending", "files":["real/src/lib.rs"]},
                    {"id":"improve-structure-missing-cycle-2", "status":"pending", "files":["missing/src/lib.rs"]}
                ],
                "edges": []
            }))
            .unwrap(),
        )
        .unwrap();

        let mined = mine_workspace(tmp.path()).expect("mine workspace");
        let cycles = mined
            .iter()
            .find(|candidate| {
                candidate.predicate == PredicateExpr::ActiveScoreSeededCycleBlocksNewCycle
            })
            .unwrap();
        let paths = mined
            .iter()
            .find(|candidate| {
                candidate.predicate == PredicateExpr::KnownScoreSeededCratesMapToExistingFiles
            })
            .unwrap();

        assert_eq!(cycles.violations, 1);
        assert_eq!(paths.support, 1);
        assert_eq!(paths.violations, 1);
    }

    #[test]
    fn validation_and_promotion_are_deterministic() {
        let tmp = tmp_workspace();
        fs::create_dir_all(tmp.path().join("state")).unwrap();
        fs::write(
            tmp.path().join("state/reasoning-loop.jsonl"),
            r#"{"trace_id":1,"record_id":"Verification:1","node_id":"n","output":{"next_supervisor_transition":"fail_claim","passed":false}}"#,
        )
        .unwrap();

        let validations = validate_workspace(tmp.path(), 1).expect("validate workspace");
        let promoted = promote_workspace(tmp.path(), 1).expect("promote workspace");

        assert!(validations.iter().any(|validation| validation.passed));
        assert!(promoted
            .iter()
            .any(|candidate| candidate.status == InvariantStatus::Promoted));
        assert!(registry_path(tmp.path()).exists());
    }
}
