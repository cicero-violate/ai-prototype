//! EventBus subscription loop — wires TLog replay into recovery handlers.
//!
//! Polls the kernel TLog on a fixed interval, projects it through
//! `RuntimeEventBus`, and reacts to new `GateFailed` and `LeaseExpired`
//! wakeups by resetting all stale running plan nodes to Pending and waking
//! task runners via the `TaskReadyNotifier`.
//!
//! Ownership:
//!   - This module owns: TLog polling, wakeup filtering, recovery dispatch.
//!   - This module does NOT own: the recovery policy decision, task scheduling.
//!
//! Invariants:
//!   - Idempotent: replaying the same TLog events has no additional effect
//!     because `reset_stale_running_nodes` only acts on nodes that are still
//!     Running with an expired lease.
//!   - Replay-safe: if the loop misses a signal (process restart, lag), the
//!     next poll reconstructs all wakeups from TLog and re-evaluates them.
//!   - No claim IDs needed: the coarse recovery action (reset all stale
//!     running nodes) is correct when claim IDs are unavailable. Fine-grained
//!     handlers in `recovery/mod.rs` are for callers that hold a live claim.

use std::path::PathBuf;
use std::time::Duration;

use crate::codec::ndjson::load_tlog_ndjson;
use crate::runtime::event_bus::{replay_event_bus, WakeupKind};
use crate::service::supervisor::SupervisorState;

/// Poll interval between TLog scans.
const POLL_INTERVAL_SECS: u64 = 5;

/// Spawn the recovery event loop as a background tokio task.
///
/// `tlog_path` — path to the canonical kernel TLog NDJSON file.
/// `state`     — shared supervisor state; used for `reset_stale_running_nodes`
///               and `task_ready_notifier.notify()`.
pub fn start(tlog_path: PathBuf, state: SupervisorState) {
    tokio::spawn(run(tlog_path, state));
}

async fn run(tlog_path: PathBuf, state: SupervisorState) {
    eprintln!("[recovery-bus] starting  tlog={}", tlog_path.display());
    let mut interval = tokio::time::interval(Duration::from_secs(POLL_INTERVAL_SECS));
    // `last_seen_seq` is the highest event_seq processed in any previous scan.
    // Only wakeups with event_seq > last_seen_seq are treated as new.
    let mut last_seen_seq = load_tlog_ndjson(&tlog_path)
        .ok()
        .map(|tlog| replay_event_bus(&tlog))
        .map(|bus| max_wakeup_seq(bus.wakeups().iter().map(|w| w.event_seq)))
        .unwrap_or(0);

    loop {
        interval.tick().await;

        let tlog = match load_tlog_ndjson(&tlog_path) {
            Ok(t) => t,
            Err(_) => continue, // TLog not yet written; normal at cold start
        };

        let bus = replay_event_bus(&tlog);
        let wakeups = bus.wakeups();

        // Advance the cursor to the highest event_seq observed in this scan.
        // This prevents re-processing any wakeup we've already acted on.
        let scan_max_seq = wakeups.iter().map(|w| w.event_seq).max().unwrap_or(0);

        // Reconcile terminal evidence on every scan: nodes with blocker or
        // accepted-receipt evidence are promoted to Failed/Done regardless of
        // whether a recovery wakeup was detected.  This closes the gap when a
        // worker writes blocker evidence and stops without sending a completion
        // callback.
        {
            let guard = state.inner.lock().await;
            guard.reconcile_terminal_evidence();
        }

        let recovery_count = count_recovery_wakeups_since(
            wakeups.iter().map(|w| (w.event_seq, w.kind)),
            last_seen_seq,
        );

        // Advance cursor unconditionally so non-recovery wakeups are not
        // re-examined as candidates in future scans.
        last_seen_seq = last_seen_seq.max(scan_max_seq);

        if recovery_count == 0 {
            continue;
        }

        eprintln!(
            "[recovery-bus] {recovery_count} recovery event(s) detected (seq<={last_seen_seq}) — resetting stale nodes"
        );

        {
            let guard = state.inner.lock().await;
            guard.reset_stale_running_nodes();
        }
        state.task_ready_notifier.notify();
    }
}

fn count_recovery_wakeups_since<I>(wakeups: I, last_seen_seq: u64) -> usize
where
    I: IntoIterator<Item = (u64, WakeupKind)>,
{
    wakeups
        .into_iter()
        .filter(|(event_seq, _)| *event_seq > last_seen_seq)
        .filter(|(_, kind)| matches!(kind, WakeupKind::GateFailed | WakeupKind::LeaseExpired))
        .count()
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
    fn recovery_wakeup_count_ignores_cursor_history_and_non_recovery_events() {
        let wakeups = [
            (7, WakeupKind::GateFailed),
            (8, WakeupKind::TaskReady),
            (9, WakeupKind::LeaseExpired),
            (10, WakeupKind::ReceiptAccepted),
        ];

        assert_eq!(count_recovery_wakeups_since(wakeups, 7), 1);
        assert_eq!(count_recovery_wakeups_since(wakeups, 9), 0);
        assert_eq!(count_recovery_wakeups_since(wakeups, 0), 2);
    }

    #[test]
    fn max_wakeup_seq_defaults_to_zero_for_empty_history() {
        assert_eq!(max_wakeup_seq([]), 0);
        assert_eq!(max_wakeup_seq([3, 8, 5]), 8);
    }
}
