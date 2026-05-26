//! Temporal invariant mining from totally-ordered event logs.
//!
//! Implements co-occurrence counting Algorithm 3 (v2, w/o ∦) from
//! "Mining Temporal Invariants from Partially Ordered Logs"
//! (Beschastnikh et al., SLAML 2011, §4.2.3).
//!
//! **Algorithm selection rationale (decision equation from the paper):**
//!
//!   Q = required "never concurrent" (∦) invariants.
//!
//!   - Q = 0 → Algorithm 3  Cost ≈ Θ(m·t), fastest.
//!   - Q = 1, sparse DAG   → Algorithm 2  Cost ≈ Θ(m·n), avoids transitive closure.
//!   - small / dense       → Algorithm 1  transitive-closure baseline.
//!
//! The canon-agent TLog is **totally ordered** within each trace (events are
//! sorted by `seq`), so there is no concurrency: ∦ is trivially true everywhere
//! and ‖ is trivially false. Therefore Q = 0 and Algorithm 3 is optimal.
//!
//! Traces are segmented by `api_command_id` (one agent turn = one trace).
//! Within each trace events are deduplicated to their first occurrence per type
//! and sorted by `seq`. Pairwise co-occurrence and ordering counts are
//! accumulated to derive the strongest relation that holds universally:
//!
//!   A → B : A always precedes B in every co-occurring trace (Follows[A][B] = co_occ).
//!
//! `NeverPrecedes(A, B)` (the ↛ invariant) is subsumed: in a TO log,
//! `Follows[A][B] = 0` iff `Follows[B][A] = co_occ`, i.e. `AlwaysPrecedes(B, A)`.
//! Emitting both would duplicate every ordering relation, so only `AlwaysPrecedes`
//! is returned.
//!
//! The mined set is bounded (top-N by co-occurrence count) and written into
//! planning and recovery prompts as a `## TEMPORAL ORDER INVARIANTS` block.

use std::collections::HashMap;
use std::path::Path;

use crate::codec::ndjson::load_tlog_ndjson;
use crate::{Cause, EventKind};

/// An event type is the `(EventKind, Cause)` pair observed in the TLog.
/// Using both fields distinguishes e.g. `Blocked::GateFailed` from `Failed::GateFailed`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EventType {
    pub kind: EventKind,
    pub cause: Cause,
}

impl EventType {
    fn label(&self) -> String {
        format!("{:?}::{:?}", self.kind, self.cause)
    }

    fn sort_key(self) -> (u8, u8) {
        (self.kind as u8, self.cause as u8)
    }
}

/// A single mined temporal invariant: A always precedes B (A → B).
///
/// In Algorithm 3, all returned invariants have the `AlwaysPrecedes` relation.
/// `NeverPrecedes(A, B)` is equivalent to `AlwaysPrecedes(B, A)` in a TO log and
/// is therefore not emitted to avoid duplicates.
#[derive(Clone, Debug)]
pub struct TemporalInvariant {
    pub a: EventType,
    pub b: EventType,
    /// Number of traces where both types co-occurred (support count).
    pub co_occ: u64,
}

impl TemporalInvariant {
    pub fn describe(&self) -> String {
        format!(
            "{} → {} (support={})",
            self.a.label(),
            self.b.label(),
            self.co_occ
        )
    }
}

/// Canonical key for an unordered pair of event types.
fn pair_key(a: EventType, b: EventType) -> (EventType, EventType) {
    if a.sort_key() <= b.sort_key() {
        (a, b)
    } else {
        (b, a)
    }
}

/// Mine temporal invariants from the canon-agent TLog at `tlog_path`.
///
/// Uses Algorithm 3 (co-occurrence counting v2, w/o ∦).
/// Traces are formed by grouping events that share the same non-zero
/// `api_command_id`. Traces with fewer than 2 distinct event types are skipped.
/// Only pairs with `co_occ >= min_support` produce invariants.
pub fn mine_temporal_invariants(
    tlog_path: &Path,
    min_support: u64,
) -> Result<Vec<TemporalInvariant>, String> {
    let tlog = load_tlog_ndjson(tlog_path).map_err(|e| format!("load tlog: {e:?}"))?;
    if tlog.is_empty() {
        return Ok(Vec::new());
    }

    // Group events into traces by api_command_id (non-zero only).
    let mut by_cmd: HashMap<u64, Vec<(u64, EventType)>> = HashMap::new();
    for event in &tlog {
        if event.api_command_id != 0 {
            by_cmd.entry(event.api_command_id).or_default().push((
                event.seq,
                EventType {
                    kind: event.kind,
                    cause: event.cause,
                },
            ));
        }
    }

    // Sort each trace by seq; deduplicate to first occurrence per type;
    // keep only traces with ≥ 2 distinct event types.
    let traces: Vec<Vec<EventType>> = by_cmd
        .into_values()
        .map(|mut v| {
            v.sort_by_key(|(seq, _)| *seq);
            v
        })
        .filter_map(|v| {
            let mut seen: Vec<EventType> = Vec::new();
            for (_, et) in v {
                if !seen.contains(&et) {
                    seen.push(et);
                }
            }
            if seen.len() >= 2 {
                Some(seen)
            } else {
                None
            }
        })
        .collect();

    if traces.is_empty() {
        return Ok(Vec::new());
    }

    // Accumulate Algorithm 3 statistics across traces.
    //
    // co_occ[(lo,hi)]: undirected count of traces containing both types.
    //   Corresponds to CoOcc[a_i][b_j] in the paper (used as denominator).
    //
    // prec[(a,b)]: directed count of traces where the first occurrence of A
    //   precedes the first occurrence of B.
    //   Corresponds to Follows[a_i][b_j] in Algorithm 3: in a TO trace with
    //   deduplication, "A is followed by B" iff A appears before B.
    let mut co_occ: HashMap<(EventType, EventType), u64> = HashMap::new();
    let mut prec: HashMap<(EventType, EventType), u64> = HashMap::new();

    for trace in &traces {
        let n = trace.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let a = trace[i];
                let b = trace[j];
                *co_occ.entry(pair_key(a, b)).or_insert(0) += 1;
                *prec.entry((a, b)).or_insert(0) += 1;
            }
        }
    }

    // Derive invariants: emit AlwaysPrecedes(a, b) when a precedes b in every
    // co-occurring trace (Follows[a][b] == co_occ, Algorithm 3 rule, line 42-43).
    //
    // NeverPrecedes(a, b) (↛, line 44-45) is omitted: in a TO log it is equivalent
    // to AlwaysPrecedes(b, a) and would duplicate every ordering invariant.
    let mut invariants: Vec<TemporalInvariant> = Vec::new();

    for ((lo, hi), total) in &co_occ {
        if *total < min_support {
            continue;
        }
        let lo_hi = *prec.get(&(*lo, *hi)).unwrap_or(&0);
        let hi_lo = *prec.get(&(*hi, *lo)).unwrap_or(&0);

        if lo_hi == *total {
            invariants.push(TemporalInvariant {
                a: *lo,
                b: *hi,
                co_occ: *total,
            });
        }
        if hi_lo == *total {
            invariants.push(TemporalInvariant {
                a: *hi,
                b: *lo,
                co_occ: *total,
            });
        }
    }

    // Sort deterministically: by co_occ desc, then by event type sort keys.
    invariants.sort_by(|x, y| {
        y.co_occ
            .cmp(&x.co_occ)
            .then(x.a.sort_key().cmp(&y.a.sort_key()))
            .then(x.b.sort_key().cmp(&y.b.sort_key()))
    });

    Ok(invariants)
}

/// Build a prompt block for the top temporal invariants.
///
/// Returns `None` if the TLog is absent, empty, or produces no invariants above
/// `min_support`. `limit` caps the number of lines injected into prompts.
pub fn temporal_invariant_prompt_block(
    tlog_path: &Path,
    limit: usize,
    min_support: u64,
) -> Option<String> {
    let invariants = mine_temporal_invariants(tlog_path, min_support).ok()?;
    if invariants.is_empty() {
        return None;
    }
    let total = invariants.len();
    let lines: Vec<String> = invariants
        .into_iter()
        .take(limit)
        .map(|inv| format!("- {}", inv.describe()))
        .collect();
    Some(format!(
        "Derived from TLog (Algorithm 3, co-occurrence counting v2, w/o ∦; {} total, showing top {}):\n{}",
        total,
        lines.len(),
        lines.join("\n")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::append_tlog_ndjson;
    use crate::{
        CapabilityRegistryProjection, Cause, ControlEvent, Decision, EventKind, Evidence, Phase,
        RuntimeConfig, SemanticDelta, State,
    };
    use tempfile::NamedTempFile;

    fn make_event(seq: u64, kind: EventKind, cause: Cause, cmd_id: u64) -> ControlEvent {
        ControlEvent {
            seq,
            from: Phase::Execute,
            to: Phase::Verify,
            kind,
            cause,
            delta: SemanticDelta::NoChange,
            evidence: Evidence::Missing,
            decision: Decision::Continue,
            failure: None,
            recovery_action: None,
            affected_gate: None,
            runtime_config: RuntimeConfig::default(),
            state_before: State::default(),
            state_after: State::default(),
            capability_registry_projection: CapabilityRegistryProjection::default(),
            api_command_id: cmd_id,
            api_command_hash: 0,
            prev_hash: 0,
            self_hash: 0,
            origin_bb: None,
            origin_fn: None,
        }
    }

    #[test]
    fn always_precedes_detected_across_traces() {
        let tmp = NamedTempFile::new().unwrap();
        // Three traces each with Advanced::Start then Completed::EvidenceSubmitted.
        for cmd in 1u64..=3 {
            append_tlog_ndjson(
                tmp.path(),
                &make_event(cmd * 10, EventKind::Advanced, Cause::Start, cmd),
            )
            .unwrap();
            append_tlog_ndjson(
                tmp.path(),
                &make_event(
                    cmd * 10 + 1,
                    EventKind::Completed,
                    Cause::EvidenceSubmitted,
                    cmd,
                ),
            )
            .unwrap();
        }
        let invs = mine_temporal_invariants(tmp.path(), 3).unwrap();
        assert!(
            invs.iter().any(|i| {
                i.a == (EventType {
                    kind: EventKind::Advanced,
                    cause: Cause::Start,
                }) && i.b
                    == (EventType {
                        kind: EventKind::Completed,
                        cause: Cause::EvidenceSubmitted,
                    })
            }),
            "expected Advanced::Start → Completed::EvidenceSubmitted"
        );
    }

    #[test]
    fn reversed_order_emits_always_precedes_in_correct_direction() {
        let tmp = NamedTempFile::new().unwrap();
        // Three traces each with Blocked::GateFailed first then Advanced::GatePassed second.
        // Algorithm 3 should emit AlwaysPrecedes(Blocked::GateFailed, Advanced::GatePassed)
        // and NOT emit a redundant NeverPrecedes in the other direction.
        for cmd in 1u64..=3 {
            append_tlog_ndjson(
                tmp.path(),
                &make_event(cmd * 10, EventKind::Blocked, Cause::GateFailed, cmd),
            )
            .unwrap();
            append_tlog_ndjson(
                tmp.path(),
                &make_event(cmd * 10 + 1, EventKind::Advanced, Cause::GatePassed, cmd),
            )
            .unwrap();
        }
        let invs = mine_temporal_invariants(tmp.path(), 3).unwrap();

        // In a TO log, NeverPrecedes(Advanced::GatePassed, Blocked::GateFailed) is
        // subsumed by AlwaysPrecedes(Blocked::GateFailed, Advanced::GatePassed).
        // Algorithm 3 emits only the latter.
        assert!(
            invs.iter().any(|i| {
                i.a == (EventType {
                    kind: EventKind::Blocked,
                    cause: Cause::GateFailed,
                }) && i.b
                    == (EventType {
                        kind: EventKind::Advanced,
                        cause: Cause::GatePassed,
                    })
            }),
            "expected Blocked::GateFailed → Advanced::GatePassed"
        );
        // Confirm no duplicate in the opposite direction.
        assert!(
            !invs.iter().any(|i| {
                i.a == (EventType {
                    kind: EventKind::Advanced,
                    cause: Cause::GatePassed,
                }) && i.b
                    == (EventType {
                        kind: EventKind::Blocked,
                        cause: Cause::GateFailed,
                    })
            }),
            "unexpected invariant Advanced::GatePassed → Blocked::GateFailed (would contradict)"
        );
    }

    #[test]
    fn below_min_support_produces_no_invariants() {
        let tmp = NamedTempFile::new().unwrap();
        // Only 2 traces but min_support=3.
        for cmd in 1u64..=2 {
            append_tlog_ndjson(
                tmp.path(),
                &make_event(cmd * 10, EventKind::Advanced, Cause::Start, cmd),
            )
            .unwrap();
            append_tlog_ndjson(
                tmp.path(),
                &make_event(
                    cmd * 10 + 1,
                    EventKind::Completed,
                    Cause::EvidenceSubmitted,
                    cmd,
                ),
            )
            .unwrap();
        }
        let invs = mine_temporal_invariants(tmp.path(), 3).unwrap();
        assert!(
            invs.is_empty(),
            "should produce no invariants below min_support"
        );
    }

    #[test]
    fn empty_tlog_returns_empty() {
        let tmp = NamedTempFile::new().unwrap();
        let invs = mine_temporal_invariants(tmp.path(), 1).unwrap();
        assert!(invs.is_empty());
    }
}
