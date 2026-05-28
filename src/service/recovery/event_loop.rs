//! EventBus subscription loop — wires TLog replay into recovery handlers.
//!
//! Polls the kernel TLog on a fixed interval, projects it through
//! `RuntimeEventBus`, and reacts to new `GateFailed` and `LeaseExpired`
//! wakeups with a two-phase recovery strategy:
//!
//!   Phase 1 — LLM diagnosis (non-deterministic, bounded):
//!     A recovery agent is spawned for each stuck Running node. It has
//!     `RECOVERY_AGENT_MAX_STEPS` turns to read state and either write a
//!     blocker evidence entry (node → Failed) or diagnostic evidence (informs
//!     the next retry). The agent's writes go through the kernel gate just like
//!     any other tool call — authority stays deterministic.
//!
//!   Phase 2 — Generic reset (deterministic, always fires):
//!     After `RECOVERY_AGENT_WAIT_SECS`, `reset_stale_running_nodes` resets
//!     any still-Running node back to Pending regardless of what the LLM did.
//!     This is the safe fallback that fires even if the LLM call times out,
//!     errors, or was skipped due to per-node cooldown.
//!
//! Ownership:
//!   - This module owns: TLog polling, wakeup filtering, recovery dispatch.
//!   - This module does NOT own: recovery policy decisions, task scheduling.
//!
//! Invariants:
//!   - Idempotent: `reset_stale_running_nodes` only acts on nodes still Running
//!     with an expired lease; replaying the same events has no additional effect.
//!   - Replay-safe: process restart → next poll reconstructs all wakeups from
//!     TLog and re-evaluates them.
//!   - Depth-bounded: each node gets at most one recovery agent per
//!     `RECOVERY_AGENT_COOLDOWN_SECS`. Recovery agents are limited to
//!     `RECOVERY_AGENT_MAX_STEPS` turns.
//!   - Replan-bounded: at most one replan agent fires per `REPLAN_COOLDOWN_SECS`
//!     when the plan is exhausted with failures. The agent adds replacement nodes
//!     or declares the goal unachievable.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::codec::ndjson::load_tlog_ndjson;
use crate::domain::plan::NodeStatus;
use crate::runtime::event_bus::{replay_event_bus, WakeupKind};
use crate::runtime::CANONICAL_TLOG_RELATIVE_PATH;
use crate::service::invariants::mir_cfg::mir_call_invariant_prompt_block;
use crate::service::invariants::promoted_invariant_prompt_block;
use crate::service::invariants::temporal::temporal_invariant_prompt_block;
use crate::service::scheduler::plan_store::{load_plan, load_plan_read_model};
use crate::service::supervisor::SupervisorState;

/// Poll interval between TLog scans.
const POLL_INTERVAL_SECS: u64 = 5;

/// Number of turns the LLM recovery agent is allowed.
const RECOVERY_AGENT_MAX_STEPS: u64 = 2;

/// How long to wait after spawning recovery agents before the generic reset fires.
const RECOVERY_AGENT_WAIT_SECS: u64 = 60;

/// Minimum gap between recovery agent spawns for the same node.
const RECOVERY_AGENT_COOLDOWN_SECS: u64 = 300;

/// Minimum gap between replan agent spawns (plan-exhaustion replanning).
const REPLAN_COOLDOWN_SECS: u64 = 600;

/// Number of turns the LLM replan agent is allowed.
const REPLAN_AGENT_MAX_STEPS: u64 = 3;

/// Spawn the recovery event loop as a background tokio task.
pub fn start(tlog_path: PathBuf, state: SupervisorState) {
    tokio::spawn(run(tlog_path, state));
}

async fn run(tlog_path: PathBuf, state: SupervisorState) {
    eprintln!("[recovery-bus] starting  tlog={}", tlog_path.display());
    let mut interval = tokio::time::interval(Duration::from_secs(POLL_INTERVAL_SECS));
    let mut last_seen_seq = load_tlog_ndjson(&tlog_path)
        .ok()
        .map(|tlog| replay_event_bus(&tlog))
        .map(|bus| max_wakeup_seq(bus.wakeups().iter().map(|w| w.event_seq)))
        .unwrap_or(0);
    // Per-node cooldown so we don't spam recovery agents for the same stuck node.
    let mut recovery_cooldowns: HashMap<String, Instant> = HashMap::new();
    // Global cooldown for plan-exhaustion replan agents.
    let mut last_replan: Option<Instant> = None;

    // Load crash-interrupted nodes from the restart manifest written at startup.
    // These nodes were mid-flight when the system last exited. Recovery prompts for
    // these nodes use a crash-specific framing rather than the generic "stuck" framing.
    let project_dir = state.inner.lock().await.project_dir().to_path_buf();
    let crash_interrupted = read_restart_manifest(&project_dir);
    if !crash_interrupted.is_empty() {
        eprintln!(
            "[recovery-bus] crash manifest loaded: {} node(s) were mid-flight at last restart",
            crash_interrupted.len()
        );
    }

    loop {
        interval.tick().await;

        let tlog = match load_tlog_ndjson(&tlog_path) {
            Ok(t) => t,
            Err(_) => continue, // TLog not yet written; normal at cold start
        };

        let bus = replay_event_bus(&tlog);
        let wakeups = bus.wakeups();

        let scan_max_seq = wakeups.iter().map(|w| w.event_seq).max().unwrap_or(0);

        // Reconcile terminal evidence on every scan regardless of recovery wakeups.
        {
            let guard = state.inner.lock().await;
            guard.reconcile_terminal_evidence();
        }

        // After reconciliation, check whether the plan is exhausted with failures
        // and spawn a replan agent if so. Runs every scan, independent of wakeups.
        maybe_trigger_replan(&state, &mut last_replan).await;

        let new_recovery: Vec<WakeupKind> = wakeups
            .iter()
            .filter(|w| w.event_seq > last_seen_seq)
            .filter(|w| matches!(w.kind, WakeupKind::GateFailed | WakeupKind::LeaseExpired))
            .map(|w| w.kind)
            .collect();

        last_seen_seq = last_seen_seq.max(scan_max_seq);

        if new_recovery.is_empty() {
            continue;
        }

        let reason = wakeup_reason_label(&new_recovery);
        eprintln!(
            "[recovery-bus] {} recovery event(s) detected (seq<={last_seen_seq})  reason={reason}",
            new_recovery.len()
        );

        let spawned =
            try_llm_recovery(&state, reason, &mut recovery_cooldowns, &crash_interrupted).await;

        if spawned > 0 {
            // Recovery agents are running. Schedule the generic reset after a wait
            // so the agents have time to write evidence before nodes are reset.
            let state_clone = state.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(RECOVERY_AGENT_WAIT_SECS)).await;
                eprintln!("[recovery-bus] recovery wait elapsed — applying generic reset");
                {
                    let guard = state_clone.inner.lock().await;
                    guard.reset_stale_running_nodes();
                }
                state_clone.task_ready_notifier.notify();
            });
        } else {
            // All nodes on cooldown or no Running nodes — reset immediately.
            eprintln!("[recovery-bus] no recovery agents spawned — resetting stale nodes");
            {
                let guard = state.inner.lock().await;
                guard.reset_stale_running_nodes();
            }
            state.task_ready_notifier.notify();
        }
    }
}

/// Attempt to spawn a recovery agent for each stuck Running plan node.
///
/// Nodes on cooldown are skipped. Returns the number of agents successfully
/// Spawn a replan agent if the plan has no Pending/Running nodes but has
/// Failed/Skipped nodes, indicating the plan was exhausted by failures.
///
/// Guarded by `REPLAN_COOLDOWN_SECS` to prevent repeated spawns while the
/// replan agent is still running or the planner hasn't yet committed new nodes.
async fn maybe_trigger_replan(state: &SupervisorState, last_replan: &mut Option<Instant>) {
    let now = Instant::now();
    if let Some(last) = last_replan {
        if now.duration_since(*last) < Duration::from_secs(REPLAN_COOLDOWN_SECS) {
            return;
        }
    }

    let project_dir = state.inner.lock().await.project_dir().to_path_buf();

    let (plan, _) = match load_plan_read_model(&project_dir) {
        Ok(result) => result,
        Err(_) => return,
    };

    let has_active = plan
        .nodes
        .iter()
        .any(|n| matches!(n.status, NodeStatus::Pending | NodeStatus::Running));
    if has_active {
        return;
    }

    let failed_nodes: Vec<_> = plan
        .nodes
        .iter()
        .filter(|n| n.status == NodeStatus::Failed)
        .collect();
    let skipped_count = plan
        .nodes
        .iter()
        .filter(|n| n.status == NodeStatus::Skipped)
        .count();

    if failed_nodes.is_empty() && skipped_count == 0 {
        return;
    }

    let mut summary_lines: Vec<String> = failed_nodes
        .iter()
        .map(|node| {
            let blocker = node
                .evidence
                .iter()
                .find(|e| e.is_blocker())
                .map(|e| e.summary.as_str())
                .unwrap_or("no blocker evidence");
            format!("- {} (Failed): {}", node.id, blocker)
        })
        .collect();
    if skipped_count > 0 {
        summary_lines.push(format!(
            "- ({skipped_count} node(s) Skipped due to upstream failures)"
        ));
    }

    let failed_block = summary_lines.join("\n");
    let tlog_path = project_dir.join(CANONICAL_TLOG_RELATIVE_PATH);
    let mir_path = project_dir.join("state/rustc/ai/mir.jsonl");
    let temporal_block = temporal_invariant_prompt_block(&tlog_path, 5, 3)
        .map(|block| format!("\nTemporal order invariants (Algorithm 3, always-precedes relations from TLog):\n{block}\n"))
        .unwrap_or_default();
    let mir_block = mir_call_invariant_prompt_block(&mir_path, 4, 2)
        .map(|block| format!("\nMIR structural call-order invariants:\n{block}\n"))
        .unwrap_or_default();
    let compile_time_instruction = if temporal_block.is_empty() && mir_block.is_empty() {
        ""
    } else {
        "\nEncoding Invariants into the Compile-Time Type System: when an invariant explains a failure or durable recovery path, plan replacement work that encodes it into Rust types, typestates, enums, constructors, validators, or contract tests instead of leaving it as prompt-only guidance.\n"
    };
    let domain = "Replan: plan exhausted with failures";
    let metric = format!(
        "The plan has no remaining Pending or Running nodes: {} node(s) Failed, \
         {skipped_count} Skipped.\n\n\
         Failed nodes and their blockers:\n{failed_block}\n\
         {temporal_block}\
         {mir_block}\n\
         {compile_time_instruction}\
         Use project:plan_read to inspect current state, then add replacement nodes \
         with project:plan_update op=upsert_node. Add edges with op=add_edge to \
         preserve dependencies. The scheduler dispatches executors for any Pending \
         node with no unfinished upstream dependencies — make replacement nodes \
         independent where possible.\n\
         If the goal is provably unachievable given the blockers, write a summary \
         to state/agent-evidence/replan-blocked.md and stop without adding nodes. \
         Do not modify the status of existing nodes.",
        failed_nodes.len(),
    );

    let mut guard = state.inner.lock().await;
    match guard.spawn_agent(domain, &metric, REPLAN_AGENT_MAX_STEPS, None) {
        Ok(dto) => {
            eprintln!(
                "[recovery-bus] replan agent spawned  spawn_id={}  failed={}  skipped={skipped_count}",
                dto.spawn_id,
                failed_nodes.len(),
            );
            *last_replan = Some(now);
        }
        Err(e) => eprintln!("[recovery-bus] replan agent spawn failed  err={e}"),
    }
}

/// Attempt to spawn a recovery agent for each stuck Running plan node.
///
/// Nodes on cooldown are skipped. Returns the number of agents successfully
/// spawned. Errors from individual spawns are logged and skipped.
async fn try_llm_recovery(
    state: &SupervisorState,
    wakeup_reason: &str,
    cooldowns: &mut HashMap<String, Instant>,
    crash_interrupted: &HashSet<String>,
) -> usize {
    let project_dir = state.inner.lock().await.project_dir().to_path_buf();

    let running_nodes: Vec<_> = match load_plan_read_model(&project_dir) {
        Ok((plan, _)) => plan
            .nodes
            .into_iter()
            .filter(|n| n.status == NodeStatus::Running)
            .collect(),
        Err(_) => load_plan(&project_dir)
            .nodes
            .into_iter()
            .filter(|n| n.status == NodeStatus::Running)
            .collect(),
    };

    if running_nodes.is_empty() {
        return 0;
    }

    let now = Instant::now();
    let mut spawned = 0;

    for node in &running_nodes {
        if let Some(last) = cooldowns.get(&node.id) {
            if now.duration_since(*last) < Duration::from_secs(RECOVERY_AGENT_COOLDOWN_SECS) {
                eprintln!(
                    "[recovery-bus] recovery agent skipped (cooldown)  node={}",
                    node.id
                );
                continue;
            }
        }

        let domain = format!("Recovery: {}", node.title);
        let crash_note = if crash_interrupted.contains(&node.id) {
            "Note: this node was mid-flight when the system last restarted (crash recovery). \
             It may have left partial work — check for uncommitted changes or incomplete evidence \
             before diagnosing a logic failure.\n"
        } else {
            ""
        };
        let tlog_path = project_dir.join(CANONICAL_TLOG_RELATIVE_PATH);
        let mir_path = project_dir.join("state/rustc/ai/mir.jsonl");
        let invariant_block = promoted_invariant_prompt_block(&project_dir, 5)
            .map(|block| format!(" Relevant promoted invariants:\n{block}\n"))
            .unwrap_or_default();
        let temporal_block = temporal_invariant_prompt_block(&tlog_path, 5, 3)
            .map(|block| format!(" Temporal order invariants (Algorithm 3):\n{block}\n"))
            .unwrap_or_default();
        let mir_block = mir_call_invariant_prompt_block(&mir_path, 4, 2)
            .map(|block| format!(" MIR structural call-order invariants:\n{block}\n"))
            .unwrap_or_default();
        let compile_time_instruction = if invariant_block.is_empty()
            && temporal_block.is_empty()
            && mir_block.is_empty()
        {
            ""
        } else {
            " Encoding Invariants into the Compile-Time Type System: if an invariant explains this stuck node, include in your diagnosis how it should be encoded into Rust types, typestates, enums, constructors, validators, or contract tests; do not edit source files during recovery.\n"
        };
        let prior_unaccepted = node.evidence.iter().filter(|e| !e.accepted).count();
        let metric = if prior_unaccepted >= 2 {
            let files_note = if node.files.is_empty() {
                String::new()
            } else {
                format!(
                    " The node lists these source files: {}. Verify they exist before writing your entry.",
                    node.files.join(", ")
                )
            };
            format!(
                "Task '{}' is stuck: {}. \
                 {}{}{}{}\
                 {}\
                 {} prior evidence entries exist and none have been accepted. \
                 Per the evidence-gate invariant, repeated unaccepted evidence means the node \
                 must be marked Failed rather than retried indefinitely. \
                 Write a blocker evidence entry — do not write another diagnostic. \
                 Do not edit source files, run build commands, or use workspace:structural_edit. \
                 Use project:plan_read to inspect current state, then append a blocker with \
                 op=append_evidence and kind=blocker.{}",
                node.title,
                wakeup_reason,
                invariant_block,
                temporal_block,
                mir_block,
                crash_note,
                compile_time_instruction,
                prior_unaccepted,
                files_note
            )
        } else {
            let prior_hint = if prior_unaccepted > 0 {
                let files_note = if node.files.is_empty() {
                    String::new()
                } else {
                    format!(
                        " Verify that the path(s) listed under files ({}) actually exist on disk before deciding whether this is retryable or permanently blocked.",
                        node.files.join(", ")
                    )
                };
                format!(
                    "Note: {} prior evidence entr{} exist with none accepted.{} ",
                    prior_unaccepted,
                    if prior_unaccepted == 1 { "y" } else { "ies" },
                    files_note
                )
            } else {
                String::new()
            };
            format!(
                "Task '{}' is stuck: {}. \
                 Examine the current plan state and any evidence already attached to this node. \
                 {}{}{}{}\
                 {}\
                 {}\
                 If the task is permanently blocked (unsatisfiable constraint, missing dependency, \
                 repeated failure with no path forward), write a blocker evidence entry so the node \
                 can be marked Failed and downstream work can proceed. \
                 Otherwise write a short diagnostic evidence entry describing what failed and why, \
                 to help the next retry attempt. \
                 Do not attempt to redo the full task — only diagnose and annotate.",
                node.title,
                wakeup_reason,
                invariant_block,
                temporal_block,
                mir_block,
                crash_note,
                compile_time_instruction,
                prior_hint
            )
        };

        let result = {
            let mut guard = state.inner.lock().await;
            guard.spawn_agent(
                &domain,
                &metric,
                RECOVERY_AGENT_MAX_STEPS,
                Some(node.id.clone()),
            )
        };

        match result {
            Ok(dto) => {
                eprintln!(
                    "[recovery-bus] recovery agent spawned  node={}  spawn_id={}  max_steps={RECOVERY_AGENT_MAX_STEPS}  reason={wakeup_reason}",
                    node.id, dto.spawn_id
                );
                cooldowns.insert(node.id.clone(), now);
                spawned += 1;
            }
            Err(e) => {
                eprintln!(
                    "[recovery-bus] recovery agent spawn failed  node={}  err={e}",
                    node.id
                );
            }
        }
    }

    spawned
}

fn wakeup_reason_label(wakeups: &[WakeupKind]) -> &'static str {
    if wakeups.iter().any(|k| matches!(k, WakeupKind::GateFailed)) {
        "gate verification failed"
    } else {
        "task lease expired"
    }
}

fn max_wakeup_seq<I>(seqs: I) -> u64
where
    I: IntoIterator<Item = u64>,
{
    seqs.into_iter().max().unwrap_or(0)
}

/// Read the restart manifest written by the supervisor on startup.
///
/// Returns a set of node IDs that were mid-flight (Running) when the process
/// last exited. Returns an empty set if the manifest is absent or unparseable.
fn read_restart_manifest(project_dir: &std::path::Path) -> HashSet<String> {
    let path = project_dir.join("state/tlog/last-restart.json");
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return HashSet::new(),
    };
    // Parse the JSON array from "interrupted_node_ids":[...].
    // Deliberately hand-rolled to avoid pulling in serde here.
    let mut ids = HashSet::new();
    if let Some(start) = text.find("\"interrupted_node_ids\":[") {
        let rest = &text[start + "\"interrupted_node_ids\":[".len()..];
        if let Some(end) = rest.find(']') {
            for raw in rest[..end].split(',') {
                let trimmed = raw.trim().trim_matches('"');
                if !trimmed.is_empty() {
                    ids.insert(trimmed.to_string());
                }
            }
        }
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wakeup_reason_label_prefers_gate_failed_over_lease_expired() {
        assert_eq!(
            wakeup_reason_label(&[WakeupKind::GateFailed, WakeupKind::LeaseExpired]),
            "gate verification failed"
        );
        assert_eq!(
            wakeup_reason_label(&[WakeupKind::LeaseExpired]),
            "task lease expired"
        );
    }

    #[test]
    fn wakeup_reason_label_ignores_non_recovery_kinds() {
        assert_eq!(
            wakeup_reason_label(&[WakeupKind::TaskReady, WakeupKind::GateFailed]),
            "gate verification failed"
        );
    }

    #[test]
    fn max_wakeup_seq_defaults_to_zero_for_empty_history() {
        assert_eq!(max_wakeup_seq([]), 0);
        assert_eq!(max_wakeup_seq([3, 8, 5]), 8);
    }
}
