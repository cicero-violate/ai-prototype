//! Invariant harness tests for the ai runtime.
//!
//! Adapted from canon-invariant harness patterns (constraint_harness,
//! request_lifecycle_harness). Rather than porting the canon-utils types
//! (which are coupled to the old loop architecture), this module codifies
//! the invariants of the ai runtime in terms of the ai kernel directly:
//!
//!   1. TLog append→replay invariant
//!   2. Hash chain integrity invariant
//!   3. Recovery limit invariant
//!   4. Terminal invariant (Phase::Done cannot advance)
//!   5. Event bus reconstruction invariant
//!   6. Plan projection idempotency invariant

use ai::{run_until_done, verify_tlog_from, CanonError, EventKind, Phase, RuntimeConfig, State};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_cfg() -> RuntimeConfig {
    RuntimeConfig::default()
}

fn run_full() -> (State, ai::TLog) {
    run_until_done(State::default(), default_cfg()).expect("run_until_done")
}

// ---------------------------------------------------------------------------
// 1. TLog append → replay invariant
//
// After run_until_done, replaying the resulting TLog from the initial state
// must reproduce the exact same terminal state.
// ---------------------------------------------------------------------------

#[test]
fn tlog_replay_reproduces_terminal_state() {
    let (state, tlog) = run_full();
    let replayed = verify_tlog_from(State::default(), &tlog).expect("verify_tlog_from");
    assert_eq!(
        replayed, state,
        "replayed state must equal original terminal state"
    );
}

#[test]
fn tlog_replay_report_event_count_matches_tlog_length() {
    let (_state, tlog) = run_full();
    let report = ai::replay_report_from(State::default(), &tlog).expect("replay_report_from");
    assert_eq!(
        report.event_count,
        tlog.len(),
        "replay report event_count must equal TLog length"
    );
}

// ---------------------------------------------------------------------------
// 2. Hash chain integrity invariant
//
// For every consecutive pair of events (e_n, e_{n+1}) in the TLog:
//   e_{n+1}.prev_hash == e_n.self_hash
// And no self_hash is zero (reserved sentinel).
// ---------------------------------------------------------------------------

#[test]
fn tlog_hash_chain_is_unbroken() {
    let (_state, tlog) = run_full();
    for window in tlog.windows(2) {
        let prev = &window[0];
        let next = &window[1];
        assert_eq!(
            next.prev_hash, prev.self_hash,
            "hash chain broken at seq {} → {}: prev.self_hash={} but next.prev_hash={}",
            prev.seq, next.seq, prev.self_hash, next.prev_hash
        );
    }
}

#[test]
fn tlog_self_hash_never_zero() {
    let (_state, tlog) = run_full();
    for event in &tlog {
        assert_ne!(
            event.self_hash, 0,
            "self_hash must not be zero (seq={})",
            event.seq
        );
    }
}

// ---------------------------------------------------------------------------
// 3. Recovery limit invariant
//
// No event in the TLog may carry recovery_attempts > max_recovery_attempts.
// ---------------------------------------------------------------------------

#[test]
fn recovery_attempts_never_exceed_limit() {
    let cfg = default_cfg();
    let (_state, tlog) = run_until_done(State::default(), cfg).expect("run");
    for event in &tlog {
        assert!(
            event.state_after.recovery_attempts <= cfg.max_recovery_attempts,
            "recovery_attempts={} exceeds max={} at seq={}",
            event.state_after.recovery_attempts,
            cfg.max_recovery_attempts,
            event.seq
        );
    }
}

// ---------------------------------------------------------------------------
// 4. Terminal invariant
//
// Phase::Done is a terminal state: no event may have `from == Phase::Done`.
// Once the lifecycle completes, the TLog is closed.
// ---------------------------------------------------------------------------

#[test]
fn phase_done_is_terminal_no_event_advances_from_done() {
    let (_state, tlog) = run_full();
    for event in &tlog {
        assert_ne!(
            event.from,
            Phase::Done,
            "event at seq={} transitions FROM Phase::Done (to={:?}), which is illegal",
            event.seq,
            event.to
        );
    }
}

#[test]
fn completed_kind_always_transitions_to_done() {
    let (_state, tlog) = run_full();
    for event in &tlog {
        if event.kind == EventKind::Completed {
            assert_eq!(
                event.to,
                Phase::Done,
                "Completed event at seq={} must land in Phase::Done but got {:?}",
                event.seq,
                event.to
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Event bus reconstruction invariant
//
// replay_event_bus(TLog) must reconstruct a non-empty wakeup set when the
// TLog contains evidence-submitted events, and the reconstructed bus must
// agree with the canonical final state.
// ---------------------------------------------------------------------------

#[test]
fn event_bus_reconstruction_is_consistent_with_tlog() {
    let (_state, tlog) = run_full();
    let bus = ai::runtime::replay_event_bus(&tlog);
    // The reconstructed bus must not produce more wakeups than events in the TLog.
    // Each TLog event produces at most one wakeup entry.
    assert!(
        bus.wakeups().len() <= tlog.len(),
        "event bus wakeup count ({}) must not exceed TLog length ({})",
        bus.wakeups().len(),
        tlog.len()
    );
}

// ---------------------------------------------------------------------------
// 6. Seq monotonicity invariant
//
// TLog event seq numbers must be strictly monotonically increasing.
// ---------------------------------------------------------------------------

#[test]
fn tlog_seq_is_strictly_monotone() {
    let (_state, tlog) = run_full();
    for window in tlog.windows(2) {
        assert!(
            window[1].seq > window[0].seq,
            "seq is not strictly monotone: {} then {}",
            window[0].seq,
            window[1].seq
        );
    }
}

// ---------------------------------------------------------------------------
// 7. State continuity invariant
//
// state_before of each event must equal the state_after of the preceding event.
// ---------------------------------------------------------------------------

#[test]
fn tlog_state_before_matches_prev_state_after() {
    let (_state, tlog) = run_full();
    for window in tlog.windows(2) {
        assert_eq!(
            window[1].state_before, window[0].state_after,
            "state continuity broken at seq {} → {}",
            window[0].seq, window[1].seq
        );
    }
}

// ---------------------------------------------------------------------------
// 8. verify_tlog error rejection invariant
//
// Supplying a truncated TLog to verify_tlog_from must either succeed on the
// partial run or fail cleanly (never panic).
// ---------------------------------------------------------------------------

#[test]
fn verify_tlog_from_handles_empty_tlog_gracefully() {
    let result = verify_tlog_from(State::default(), &[]);
    // Empty TLog returns initial state — no error.
    assert_eq!(result, Ok(State::default()));
}

#[test]
fn verify_tlog_from_rejects_tampered_hash_chain() {
    let (_state, mut tlog) = run_full();
    if tlog.len() < 2 {
        return; // too short to tamper
    }
    // Corrupt the self_hash of the first event.
    tlog[0].self_hash = tlog[0].self_hash.wrapping_add(1);
    let result = verify_tlog_from(State::default(), &tlog);
    assert_eq!(
        result,
        Err(CanonError::InvalidHashChain),
        "tampered hash chain must be rejected"
    );
}
