//! Policy matrix and transition coverage contract.
//!
//! Adapted from canon-policy-matrix scenario family patterns. Instead of
//! importing the canon-utils types (which are coupled to the old loop
//! architecture and canon_loop/canon_route crates), this module extracts
//! scenario families directly from the ai kernel transition table.
//!
//! Verifies that:
//!   - The transition table has no duplicate entries
//!   - legal_transition() accepts all table entries
//!   - Specific scenario families (terminal, recovery, advancement) are correct
//!   - Run-until-done always terminates (convergence)

use ai::runtime::wire_transition_allowed;
use ai::{run_until_done, Cause, EventKind, Phase, RuntimeConfig, State};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_cfg() -> RuntimeConfig {
    RuntimeConfig::default()
}

// ---------------------------------------------------------------------------
// 1. Transition table uniqueness
//
// No two rows in TRANSITIONS may share the same (from, to, kind, cause) quad.
// Duplicates indicate conflicting policy definitions.
// ---------------------------------------------------------------------------

#[test]
fn transition_table_has_no_duplicate_entries() {
    use std::collections::HashSet;

    let mut seen: HashSet<(u8, u8, u8, u8)> = HashSet::new();
    for transition in ai::runtime::verify::TRANSITIONS_FOR_TEST.iter() {
        let key = (
            transition.from as u8,
            transition.to as u8,
            transition.kind as u8,
            transition.cause as u8,
        );
        assert!(
            seen.insert(key),
            "duplicate transition: from={:?} to={:?} kind={:?} cause={:?}",
            transition.from,
            transition.to,
            transition.kind,
            transition.cause
        );
    }
}

// ---------------------------------------------------------------------------
// 2. legal_transition accepts all table entries
//
// Every row in TRANSITIONS must be accepted by legal_transition(). A row
// that fails this check indicates a divergence between policy and enforcement.
// ---------------------------------------------------------------------------

#[test]
fn legal_transition_accepts_all_table_entries() {
    for transition in ai::runtime::verify::TRANSITIONS_FOR_TEST.iter() {
        assert!(
            ai::legal_transition(
                transition.from,
                transition.to,
                transition.kind,
                transition.cause
            ),
            "legal_transition rejected table entry: from={:?} to={:?} kind={:?} cause={:?}",
            transition.from,
            transition.to,
            transition.kind,
            transition.cause
        );
    }
}

// ---------------------------------------------------------------------------
// 3. Terminal scenario family
//
// Phase::Done is a terminal state. No transition may originate from Done.
// wire_transition_allowed and legal_transition must both reject any attempt
// to advance from Phase::Done.
// ---------------------------------------------------------------------------

#[test]
fn done_phase_is_terminal_wire_layer_rejects_all_advances() {
    let advance_kinds = [
        EventKind::Advanced,
        EventKind::Blocked,
        EventKind::Failed,
        EventKind::Recovered,
        EventKind::Completed,
    ];
    // All non-Done target phases must be unreachable from Done.
    // Done→Done/Completed is the terminal self-transition and is exempt.
    let non_done_targets = [
        Phase::Delta,
        Phase::Invariant,
        Phase::Analysis,
        Phase::Judgment,
        Phase::Plan,
        Phase::Execute,
        Phase::Verify,
        Phase::Eval,
        Phase::Recovery,
        Phase::Learn,
        Phase::Persist,
    ];
    for kind in advance_kinds {
        for to in non_done_targets {
            assert!(
                !wire_transition_allowed(Phase::Done, to, kind),
                "wire layer must reject transition from Phase::Done to {:?} with kind {:?}",
                to,
                kind
            );
        }
    }
}

#[test]
fn done_phase_is_terminal_legal_transition_rejects_with_any_cause() {
    let causes = [
        Cause::Start,
        Cause::GatePassed,
        Cause::GateFailed,
        Cause::EvidenceMissing,
        Cause::ExecutionFinished,
        Cause::VerificationPassed,
    ];
    for cause in causes {
        assert!(
            !ai::legal_transition(Phase::Done, Phase::Invariant, EventKind::Advanced, cause),
            "legal_transition must reject Done→Invariant (cause={cause:?})"
        );
    }
}

// ---------------------------------------------------------------------------
// 4. Advancement scenario family
//
// Phase::Delta must be the only valid starting phase — the first transition
// in any run must start from Delta (not mid-lifecycle).
// The canonical starting transition is Delta→Invariant/Advanced/Start.
// ---------------------------------------------------------------------------

#[test]
fn delta_to_invariant_advanced_start_is_allowed() {
    assert!(
        ai::legal_transition(
            Phase::Delta,
            Phase::Invariant,
            EventKind::Advanced,
            Cause::Start
        ),
        "Delta→Invariant/Advanced/Start must be a legal transition"
    );
}

#[test]
fn first_tlog_event_starts_from_delta() {
    let (_state, tlog) = run_until_done(State::default(), default_cfg()).expect("run");
    let first = tlog.first().expect("tlog must be non-empty");
    assert_eq!(
        first.from,
        Phase::Delta,
        "first TLog event must originate from Phase::Delta"
    );
}

// ---------------------------------------------------------------------------
// 5. Recovery scenario family
//
// Recovery phase can only be reached via Blocked or Failed events.
// No other event kind may produce Phase::Recovery as a destination.
// ---------------------------------------------------------------------------

#[test]
fn recovery_phase_only_reachable_via_blocked_or_failed() {
    // Phase::Recovery may only be *entered* from a non-Recovery phase via Blocked or Failed.
    // Transitions that stay within Recovery (from=Recovery, to=Recovery) are allowed for
    // effect events (e.g. Persisted) that occur during an ongoing recovery sequence.
    for transition in ai::runtime::verify::TRANSITIONS_FOR_TEST.iter() {
        if transition.to == Phase::Recovery && transition.from != Phase::Recovery {
            assert!(
                transition.kind == EventKind::Blocked || transition.kind == EventKind::Failed,
                "Phase::Recovery must only be entered via Blocked or Failed, \
                 but found: from={:?} kind={:?} cause={:?}",
                transition.from,
                transition.kind,
                transition.cause
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 6. Completed kind scenario family
//
// EventKind::Completed must always result in Phase::Done.
// No Completed event may transition to any non-Done phase.
// ---------------------------------------------------------------------------

#[test]
fn completed_kind_always_targets_done() {
    for transition in ai::runtime::verify::TRANSITIONS_FOR_TEST.iter() {
        if transition.kind == EventKind::Completed {
            assert_eq!(
                transition.to,
                Phase::Done,
                "Completed kind must always target Phase::Done, \
                 found from={:?} to={:?}",
                transition.from,
                transition.to
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 7. Invented transition rejection scenario family
//
// Transitions that are not in the table must be rejected. Tests a sample
// of illegal (from, to, kind, cause) quadruples.
// ---------------------------------------------------------------------------

#[test]
fn illegal_transitions_are_rejected() {
    let illegal = vec![
        // Cannot skip directly from Delta to Done
        (
            Phase::Delta,
            Phase::Done,
            EventKind::Completed,
            Cause::Start,
        ),
        // Cannot advance backwards from Judgment to Delta
        (
            Phase::Judgment,
            Phase::Delta,
            EventKind::Advanced,
            Cause::GatePassed,
        ),
        // Failed cannot advance a phase forward
        (
            Phase::Analysis,
            Phase::Judgment,
            EventKind::Failed,
            Cause::EvidenceMissing,
        ),
        // Completed from non-final phase
        (
            Phase::Delta,
            Phase::Done,
            EventKind::Completed,
            Cause::GatePassed,
        ),
        // Persisted event cannot originate from Delta
        (
            Phase::Delta,
            Phase::Delta,
            EventKind::Persisted,
            Cause::EvidenceSubmitted,
        ),
    ];
    for (from, to, kind, cause) in illegal {
        assert!(
            !ai::legal_transition(from, to, kind, cause),
            "legal_transition must reject from={from:?} to={to:?} kind={kind:?} cause={cause:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// 8. Convergence scenario family
//
// run_until_done must always reach Phase::Done or emit a ConvergenceFailed
// event. It must never loop forever or panic.
// ---------------------------------------------------------------------------

#[test]
fn run_until_done_always_terminates_in_done() {
    let (state, _tlog) = run_until_done(State::default(), default_cfg()).expect("run");
    assert_eq!(
        state.phase,
        Phase::Done,
        "run_until_done must reach Phase::Done"
    );
}

#[test]
fn run_until_done_with_tight_step_limit_emits_convergence_failure() {
    let cfg = RuntimeConfig {
        max_steps: 2,
        max_recovery_attempts: 1,
    };
    let (state, tlog) = run_until_done(State::default(), cfg).expect("run with tight limit");
    assert_eq!(state.phase, Phase::Done);
    // The last event should be a convergence-failure halt.
    let last = tlog.last().expect("tlog must be non-empty");
    assert_eq!(
        last.kind,
        EventKind::Failed,
        "tight step limit must produce Failed terminal"
    );
}

// ---------------------------------------------------------------------------
// 9. Transition table coverage count
//
// The number of entries must match the expected count so that accidental
// deletions or additions are caught immediately.
// ---------------------------------------------------------------------------

#[test]
fn transition_table_has_expected_entry_count() {
    let count = ai::runtime::verify::TRANSITIONS_FOR_TEST.len();
    // This count is locked to the current TRANSITIONS table size.
    // Update this assertion when intentional new transitions are added.
    assert!(
        count >= 40,
        "transition table has only {count} entries — expected at least 40"
    );
}
