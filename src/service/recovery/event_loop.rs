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

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::codec::ndjson::load_tlog_ndjson;
use crate::domain::plan::NodeStatus;
use crate::runtime::event_bus::{replay_event_bus, WakeupKind};
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

        let spawned = try_llm_recovery(&state, reason, &mut recovery_cooldowns).await;

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
/// spawned. Errors from individual spawns are logged and skipped.
async fn try_llm_recovery(
    state: &SupervisorState,
    wakeup_reason: &str,
    cooldowns: &mut HashMap<String, Instant>,
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
        let metric = format!(
            "Task '{}' is stuck: {}. \
             Examine the current plan state and any evidence already attached to this node. \
             If the task is permanently blocked (unsatisfiable constraint, missing dependency, \
             repeated failure with no path forward), write a blocker evidence entry so the node \
             can be marked Failed and downstream work can proceed. \
             Otherwise write a short diagnostic evidence entry describing what failed and why, \
             to help the next retry attempt. \
             Do not attempt to redo the full task — only diagnose and annotate.",
            node.title, wakeup_reason
        );

        let result = {
            let mut guard = state.inner.lock().await;
            guard.spawn_agent(&domain, &metric, RECOVERY_AGENT_MAX_STEPS)
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
