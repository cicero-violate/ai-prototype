//! Structural invariant mining from MIR call-graph JSONL.
//!
//! The MIR JSONL emitted by canon-rustc records `term` records (terminators).
//! `Call` terminators carry a `callee` function name and the basic-block index
//! (`bb`) of the call site in the caller function.
//!
//! **Algorithm selection (paper §4.2.3):**
//! Each caller function's call sequence is a totally-ordered list of callees
//! sorted by `bb`. This is a TO log (Q = 0) so Algorithm 3 is optimal, identical
//! to the temporal miner but operating over (caller, callee) event types instead
//! of (EventKind, Cause) pairs.
//!
//! Derived invariants express structural guarantees such as:
//!   "fn foo always calls bar before calling baz (support=12)"
//!
//! These can be correlated with behavioral TLog invariants when `ControlEvent`
//! carries `origin_fn` — the caller's function index — allowing a concrete code
//! location to be pinned to a behavioral ordering constraint.
//!
//! **Input format:** NDJSON with schema:
//!   `{"kind":"term","fn":"<caller>","bb":<u32>,"term":"Call","callee":"<callee>","args":[...],"ret":{...}}`
//!
//! Only `kind == "term"` and `term == "Call"` records are processed.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// A structural call-graph invariant mined from MIR:
/// `caller` always calls `callee_a` before `callee_b` (by BB order).
#[derive(Clone, Debug)]
pub struct MirCallInvariant {
    /// The function that contains both call sites.
    pub caller: String,
    /// Callee always called first.
    pub callee_a: String,
    /// Callee always called after `callee_a`.
    pub callee_b: String,
    /// Number of traces (distinct caller functions) where both callees co-occur.
    pub co_occ: u64,
}

impl MirCallInvariant {
    pub fn describe(&self) -> String {
        format!(
            "{}() → {} before {} (support={})",
            self.caller, self.callee_a, self.callee_b, self.co_occ
        )
    }
}

/// A call-site event type: the callee function name (deduplicated to first BB).
type CalleeType = String;

/// Canonical key for an unordered callee pair.
fn pair_key<'a>(a: &'a str, b: &'a str) -> (&'a str, &'a str) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

/// Parse MIR JSONL at `mir_path` and return a map from caller function name
/// to its call sequence: `(bb, callee)` pairs sorted by `bb`.
fn load_call_sequences(mir_path: &Path) -> Result<HashMap<String, Vec<(u32, String)>>, String> {
    let file =
        std::fs::File::open(mir_path).map_err(|e| format!("open {}: {e}", mir_path.display()))?;
    let reader = BufReader::new(file);

    let mut by_caller: HashMap<String, Vec<(u32, String)>> = HashMap::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("read {}: {e}", mir_path.display()))?;
        if line.is_empty() {
            continue;
        }
        // Fast-path: skip non-term and non-Call records before JSON parse.
        if !line.contains("\"Call\"") {
            continue;
        }

        let Ok(record) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        if record.get("kind").and_then(|v| v.as_str()) != Some("term") {
            continue;
        }
        if record.get("term").and_then(|v| v.as_str()) != Some("Call") {
            continue;
        }
        let Some(caller) = record.get("fn").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(callee) = record.get("callee").and_then(|v| v.as_str()) else {
            continue;
        };
        let bb = record.get("bb").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

        by_caller
            .entry(caller.to_string())
            .or_default()
            .push((bb, callee.to_string()));
    }

    Ok(by_caller)
}

/// Mine structural call-order invariants from `mir_path`.
///
/// Each caller function is treated as a trace: callees sorted by BB order form
/// the event sequence. Algorithm 3 is applied to these traces to derive
/// "callee_a always precedes callee_b" invariants.
///
/// Only invariants with `co_occ >= min_support` are returned, bounded by `limit`.
pub fn mine_mir_call_invariants(
    mir_path: &Path,
    min_support: u64,
    limit: usize,
) -> Result<Vec<MirCallInvariant>, String> {
    let by_caller = load_call_sequences(mir_path)?;

    // Build one trace per caller: sorted, deduplicated call sequence.
    let traces: Vec<(String, Vec<CalleeType>)> = by_caller
        .into_iter()
        .filter_map(|(caller, mut calls)| {
            calls.sort_by_key(|(bb, _)| *bb);
            let mut seen: Vec<String> = Vec::new();
            for (_, callee) in calls {
                if !seen.contains(&callee) {
                    seen.push(callee);
                }
            }
            if seen.len() >= 2 {
                Some((caller, seen))
            } else {
                None
            }
        })
        .collect();

    if traces.is_empty() {
        return Ok(Vec::new());
    }

    // Algorithm 3 accumulation over call traces.
    // co_occ[(lo,hi)]: undirected count of callers containing both callees.
    // prec[(a,b)]:     directed count of callers where callee_a appears before callee_b.
    let mut co_occ: HashMap<(String, String), u64> = HashMap::new();
    let mut prec: HashMap<(String, String), u64> = HashMap::new();
    // Map (unordered callee pair) → example caller function name.
    let mut example_caller: HashMap<(String, String), String> = HashMap::new();

    for (caller, seq) in &traces {
        let n = seq.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let a = &seq[i];
                let b = &seq[j];
                let (lo, hi) = pair_key(a, b);
                let key = (lo.to_string(), hi.to_string());
                *co_occ.entry(key.clone()).or_insert(0) += 1;
                example_caller.entry(key).or_insert_with(|| caller.clone());
                *prec.entry((a.clone(), b.clone())).or_insert(0) += 1;
            }
        }
    }

    // Derive invariants.
    let mut invariants: Vec<MirCallInvariant> = Vec::new();

    for ((lo, hi), total) in &co_occ {
        if *total < min_support {
            continue;
        }
        let lo_hi = *prec.get(&(lo.clone(), hi.clone())).unwrap_or(&0);
        let hi_lo = *prec.get(&(hi.clone(), lo.clone())).unwrap_or(&0);
        let caller = example_caller
            .get(&(lo.clone(), hi.clone()))
            .cloned()
            .unwrap_or_default();

        if lo_hi == *total {
            invariants.push(MirCallInvariant {
                caller: caller.clone(),
                callee_a: lo.clone(),
                callee_b: hi.clone(),
                co_occ: *total,
            });
        }
        if hi_lo == *total {
            invariants.push(MirCallInvariant {
                caller: caller.clone(),
                callee_a: hi.clone(),
                callee_b: lo.clone(),
                co_occ: *total,
            });
        }
    }

    invariants.sort_by(|x, y| {
        y.co_occ
            .cmp(&x.co_occ)
            .then(x.caller.cmp(&y.caller))
            .then(x.callee_a.cmp(&y.callee_a))
            .then(x.callee_b.cmp(&y.callee_b))
    });

    invariants.truncate(limit);
    Ok(invariants)
}

/// Build a prompt block from MIR structural call-order invariants.
///
/// Returns `None` if the MIR file is absent or produces no invariants above
/// `min_support`. `limit` caps the number of lines in the prompt block.
pub fn mir_call_invariant_prompt_block(
    mir_path: &Path,
    limit: usize,
    min_support: u64,
) -> Option<String> {
    if !mir_path.exists() {
        return None;
    }
    let invariants = mine_mir_call_invariants(mir_path, min_support, limit).ok()?;
    if invariants.is_empty() {
        return None;
    }
    let total_before_limit = invariants.len();
    let lines: Vec<String> = invariants
        .into_iter()
        .map(|inv| format!("- {}", inv.describe()))
        .collect();
    Some(format!(
        "Derived from MIR call graph (Algorithm 3, w/o ∦; {} total, showing top {}):\n{}",
        total_before_limit,
        lines.len(),
        lines.join("\n")
    ))
}

/// Trim a fully-qualified Rust function name to at most `max_segments` path
/// components for readability in prompts.
///
/// E.g. `<std::vec::Vec<T> as core::ops::Drop>::drop` → `Vec::drop`
pub fn short_fn_name(name: &str, max_segments: usize) -> String {
    // Strip angle-bracket wrapper if present.
    let inner = if name.starts_with('<') {
        name.trim_start_matches('<')
            .splitn(2, " as ")
            .last()
            .unwrap_or(name)
            .trim_end_matches('>')
    } else {
        name
    };
    let parts: Vec<&str> = inner.split("::").collect();
    let start = parts.len().saturating_sub(max_segments);
    parts[start..].join("::")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_mir(records: &[serde_json::Value]) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        for r in records {
            writeln!(f, "{}", r).unwrap();
        }
        f
    }

    #[test]
    fn invariant_detected_when_two_callees_always_ordered() {
        let f = write_mir(&[
            serde_json::json!({"kind":"term","fn":"foo","bb":0,"term":"Call","callee":"alpha","args":[],"ret":{"local":"_1","ty":"()"}}),
            serde_json::json!({"kind":"term","fn":"foo","bb":1,"term":"Call","callee":"beta","args":[],"ret":{"local":"_2","ty":"()"}}),
            serde_json::json!({"kind":"term","fn":"bar","bb":0,"term":"Call","callee":"alpha","args":[],"ret":{"local":"_1","ty":"()"}}),
            serde_json::json!({"kind":"term","fn":"bar","bb":1,"term":"Call","callee":"beta","args":[],"ret":{"local":"_2","ty":"()"}}),
        ]);
        let invs = mine_mir_call_invariants(f.path(), 2, 10).unwrap();
        // Both foo and bar call alpha before beta — should produce AlwaysPrecedes(alpha, beta).
        assert!(
            invs.iter()
                .any(|inv| inv.callee_a == "alpha" && inv.callee_b == "beta" && inv.co_occ == 2),
            "expected alpha → beta with co_occ=2, got: {invs:?}"
        );
        // No beta → alpha invariant (reversed).
        assert!(
            !invs
                .iter()
                .any(|inv| inv.callee_a == "beta" && inv.callee_b == "alpha"),
            "unexpected reversed invariant"
        );
    }

    #[test]
    fn no_invariant_below_min_support() {
        let f = write_mir(&[
            serde_json::json!({"kind":"term","fn":"foo","bb":0,"term":"Call","callee":"alpha","args":[],"ret":{"local":"_1","ty":"()"}}),
            serde_json::json!({"kind":"term","fn":"foo","bb":1,"term":"Call","callee":"beta","args":[],"ret":{"local":"_2","ty":"()"}}),
        ]);
        // min_support=2 but only 1 caller — should produce nothing.
        let invs = mine_mir_call_invariants(f.path(), 2, 10).unwrap();
        assert!(
            invs.is_empty(),
            "expected no invariants below min_support=2"
        );
    }

    #[test]
    fn short_fn_name_strips_path_prefix() {
        assert_eq!(
            short_fn_name("std::collections::hash_map::HashMap::insert", 2),
            "HashMap::insert"
        );
        assert_eq!(
            short_fn_name("<std::vec::Vec<T> as core::ops::Drop>::drop", 2),
            "Drop>::drop"
        );
    }

    #[test]
    fn assign_records_are_ignored() {
        let f = write_mir(&[
            serde_json::json!({"kind":"assign","fn":"foo","bb":0,"stmt":0,"lhs":{"local":"_1","ty":"u32"},"rhs":{"op":"Use","local":"_2"}}),
            serde_json::json!({"kind":"term","fn":"foo","bb":0,"term":"Return"}),
        ]);
        let invs = mine_mir_call_invariants(f.path(), 1, 10).unwrap();
        assert!(invs.is_empty());
    }
}
