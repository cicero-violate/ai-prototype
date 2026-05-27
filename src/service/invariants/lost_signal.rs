//! Automatic lost-signal invariant mining via the `judgement` crate.
//!
//! Runs `LostSignalPass` against every `state/rustc/*/mir.jsonl` artifact dir,
//! then converts the resulting `Judgement` findings into `InvariantCandidate`
//! records that flow through the standard mine → validate → promote pipeline.
//!
//! Called in two contexts:
//! - `mine_and_write_workspace`: when the LLM triggers `canon_invariants_mine`
//! - `run_lost_signal_cycle`: once per non-spawned agent cycle, automatically

use std::fs;
use std::path::{Path, PathBuf};

use judgement::lost_signal::LostSignalPass;
use judgement::{ArtifactSet, JudgementKind, JudgementPass};

use super::{
    confidence_milli, load_registry, save_registry, write_candidate_artifact, InvariantCandidate,
    InvariantScope, PredicateExpr,
};

/// Collect `InvariantCandidate` records from all rustc artifact dirs under
/// `{project_dir}/state/rustc/`. Returns an empty vec if no `mir.jsonl`
/// files are found or the pass produces no findings.
pub fn candidates_from_mir_artifacts(project_dir: &Path) -> Vec<InvariantCandidate> {
    let rustc_dir = project_dir.join("state").join("rustc");
    if !rustc_dir.exists() {
        return Vec::new();
    }

    let artifact_dirs = match collect_mir_dirs(&rustc_dir) {
        Ok(dirs) => dirs,
        Err(e) => {
            eprintln!("[lost_signal] failed to list rustc dirs: {e}");
            return Vec::new();
        }
    };

    let pass = LostSignalPass;
    let mut candidates = Vec::new();

    for dir in &artifact_dirs {
        let artifacts = ArtifactSet::from_root(dir);
        match pass.run(&artifacts) {
            Ok(judgements) => {
                for j in judgements {
                    if j.kind == JudgementKind::Invariant {
                        candidates.push(judgement_to_candidate(j));
                    }
                }
            }
            Err(e) => {
                eprintln!("[lost_signal] pass failed for {}: {e}", dir.display());
            }
        }
    }

    candidates
}

/// Run lost-signal analysis for one agent cycle and write findings into the
/// invariant registry. Returns the count of high-confidence violations found
/// so the caller can submit recovery evidence. Non-fatal — errors are logged.
pub fn run_lost_signal_cycle(project_dir: &Path, cycle_num: u64, tag: &str) -> usize {
    let candidates = candidates_from_mir_artifacts(project_dir);
    if candidates.is_empty() {
        return 0;
    }

    let mut registry = match load_registry(project_dir) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[{tag}] lost_signal: registry load failed: {e}  cycle={cycle_num}");
            return 0;
        }
    };

    let mut violation_count: usize = 0;
    for candidate in &candidates {
        if let Err(e) = write_candidate_artifact(project_dir, candidate) {
            eprintln!("[{tag}] lost_signal: artifact write failed: {e}  cycle={cycle_num}");
        }
        if candidate.violations > 0 {
            violation_count += 1;
        }
        registry.upsert(candidate.clone());
    }

    match save_registry(project_dir, &registry) {
        Ok(()) => eprintln!(
            "[{tag}] lost_signal: {} candidates ({violation_count} violations)  cycle={cycle_num}",
            candidates.len()
        ),
        Err(e) => eprintln!("[{tag}] lost_signal: registry save failed: {e}  cycle={cycle_num}"),
    }

    violation_count
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn collect_mir_dirs(rustc_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let entries =
        fs::read_dir(rustc_dir).map_err(|e| format!("read_dir {}: {e}", rustc_dir.display()))?;
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.join("mir.jsonl").exists())
        .collect();
    dirs.sort();
    Ok(dirs)
}

fn judgement_to_candidate(j: judgement::Judgement) -> InvariantCandidate {
    let expression = j
        .evidence
        .iter()
        .map(|e| e.summary.as_str())
        .collect::<Vec<_>>()
        .join("; ");

    let first_summary = j.evidence.first().map(|e| e.summary.as_str()).unwrap_or("");

    let mut candidate = InvariantCandidate::new(
        InvariantScope::Custom("LostSignal".into()),
        PredicateExpr::Custom {
            name: "lost_signal".into(),
            expression,
        },
        format!("{}: {first_summary}", j.target),
    );

    // Map judgement confidence to support/violations:
    // high confidence (>0.70) → confirmed violation; lower → candidate only.
    if j.confidence > 0.70 {
        candidate.violations = 1;
        candidate.support = 0;
    }
    candidate.confidence_milli = confidence_milli(candidate.support, candidate.violations);
    candidate
}
