//! Legal runtime transition table.

use crate::kernel::{Cause, EventKind, Phase};

#[derive(Clone, Copy)]
pub(crate) struct Transition {
    pub(crate) from: Phase,
    pub(crate) to: Phase,
    pub(crate) kind: EventKind,
    pub(crate) cause: Cause,
}

pub(crate) const TRANSITIONS: [Transition; 38] = [
    Transition { from: Phase::Delta, to: Phase::Invariant, kind: EventKind::Advanced, cause: Cause::Start },
    Transition { from: Phase::Invariant, to: Phase::Analysis, kind: EventKind::Advanced, cause: Cause::GatePassed },
    Transition { from: Phase::Invariant, to: Phase::Recovery, kind: EventKind::Blocked, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Invariant, to: Phase::Recovery, kind: EventKind::Blocked, cause: Cause::GateFailed },
    Transition { from: Phase::Analysis, to: Phase::Judgment, kind: EventKind::Advanced, cause: Cause::GatePassed },
    Transition { from: Phase::Analysis, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Analysis, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::GateFailed },
    Transition { from: Phase::Judgment, to: Phase::Plan, kind: EventKind::Advanced, cause: Cause::JudgmentMade },
    Transition { from: Phase::Judgment, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Judgment, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::GateFailed },
    Transition { from: Phase::Plan, to: Phase::Execute, kind: EventKind::Advanced, cause: Cause::PlanReady },
    Transition { from: Phase::Plan, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Plan, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::GateFailed },
    Transition { from: Phase::Plan, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::ReadyQueueEmpty },
    Transition { from: Phase::Execute, to: Phase::Verify, kind: EventKind::Advanced, cause: Cause::ExecutionFinished },
    Transition { from: Phase::Execute, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Execute, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::GateFailed },
    Transition { from: Phase::Execute, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::TaskReceiptMissing },
    Transition { from: Phase::Verify, to: Phase::Eval, kind: EventKind::Advanced, cause: Cause::VerificationPassed },
    Transition { from: Phase::Verify, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Verify, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::GateFailed },
    Transition { from: Phase::Verify, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::ArtifactLineageBroken },
    Transition { from: Phase::Eval, to: Phase::Persist, kind: EventKind::Advanced, cause: Cause::EvalPassed },
    Transition { from: Phase::Eval, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Eval, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::GateFailed },
    Transition { from: Phase::Eval, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvalFailed },
    Transition { from: Phase::Recovery, to: Phase::Persist, kind: EventKind::Recovered, cause: Cause::RepairSelected },
    Transition { from: Phase::Recovery, to: Phase::Done, kind: EventKind::Failed, cause: Cause::RecoveryLimit },
    Transition { from: Phase::Persist, to: Phase::Invariant, kind: EventKind::Persisted, cause: Cause::RepairApplied },
    Transition { from: Phase::Persist, to: Phase::Analysis, kind: EventKind::Persisted, cause: Cause::RepairApplied },
    Transition { from: Phase::Persist, to: Phase::Judgment, kind: EventKind::Persisted, cause: Cause::RepairApplied },
    Transition { from: Phase::Persist, to: Phase::Plan, kind: EventKind::Persisted, cause: Cause::RepairApplied },
    Transition { from: Phase::Persist, to: Phase::Execute, kind: EventKind::Persisted, cause: Cause::RepairApplied },
    Transition { from: Phase::Persist, to: Phase::Verify, kind: EventKind::Persisted, cause: Cause::RepairApplied },
    Transition { from: Phase::Persist, to: Phase::Eval, kind: EventKind::Persisted, cause: Cause::RepairApplied },
    Transition { from: Phase::Persist, to: Phase::Learn, kind: EventKind::Persisted, cause: Cause::Persisted },
    Transition { from: Phase::Learn, to: Phase::Done, kind: EventKind::Learned, cause: Cause::PolicyPromoted },
    Transition { from: Phase::Done, to: Phase::Done, kind: EventKind::Completed, cause: Cause::EvalPassed },
];
