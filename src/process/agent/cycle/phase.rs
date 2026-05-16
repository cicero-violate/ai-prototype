//! Phase routing helpers for AgentCycle.

// ---- Phase dispatch and evidence submission helpers ----------------------

#[derive(Clone, Copy)]
pub(super) struct PhaseRoute {
    pub(super) phase: &'static str,
    pub(super) uses_teacher: bool,
    pub(super) gate: Option<(&'static str, &'static str)>,
}

const PHASE_ROUTES: &[PhaseRoute] = &[
    PhaseRoute {
        phase: "Invariant",
        uses_teacher: false,
        gate: Some(("Invariant", "InvariantProof")),
    },
    PhaseRoute {
        phase: "Analysis",
        uses_teacher: true,
        gate: Some(("Analysis", "AnalysisReport")),
    },
    PhaseRoute {
        phase: "Judgment",
        uses_teacher: true,
        gate: Some(("Judgment", "JudgmentRecord")),
    },
    PhaseRoute {
        phase: "Plan",
        uses_teacher: true,
        gate: Some(("Plan", "TaskReady")),
    },
    PhaseRoute {
        phase: "Execute",
        uses_teacher: false,
        gate: Some(("Execution", "ArtifactReceipt")),
    },
    PhaseRoute {
        phase: "Verify",
        uses_teacher: false,
        gate: Some(("Verification", "LineageProof")),
    },
    PhaseRoute {
        phase: "Eval",
        uses_teacher: true,
        gate: Some(("Eval", "EvalScore")),
    },
    PhaseRoute {
        phase: "Learning",
        uses_teacher: false,
        gate: Some(("Learning", "PolicyPromotion")),
    },
    PhaseRoute {
        phase: "Recovery",
        uses_teacher: true,
        gate: None,
    },
];

pub(super) fn phase_route(phase: &str) -> Option<PhaseRoute> {
    PHASE_ROUTES
        .iter()
        .find(|route| route.phase == phase)
        .copied()
}

pub(super) fn use_teacher(phase: &str) -> bool {
    phase_route(phase)
        .map(|route| route.uses_teacher)
        .unwrap_or(false)
}

pub(super) fn parse_verdict(text: &str) -> bool {
    text.lines()
        .rev()
        .find_map(|line| {
            let line = line.trim();
            let verdict = line.strip_prefix("VERDICT:")?.trim().to_ascii_lowercase();
            if verdict.starts_with("fail") {
                Some(false)
            } else if verdict.starts_with("pass") {
                Some(true)
            } else {
                None
            }
        })
        .unwrap_or(false)
}

pub(super) fn phase_gate(phase: &str) -> Option<(&'static str, &'static str)> {
    phase_route(phase).and_then(|route| route.gate)
}
