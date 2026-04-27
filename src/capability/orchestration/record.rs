//! Deterministic routing records owned by the orchestration capability.

use crate::capability::{EvidenceSubmission, PacketEffect};
use crate::kernel::{Evidence, GateId, GateStatus, State, EXECUTION_GATE_ORDER};

const ORCHESTRATION_ROUTE_VERSION: u64 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrchestrationDecision {
    Routed,
    Empty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapabilityRoute {
    pub ordinal: u8,
    pub priority: u8,
    pub gate: GateId,
    pub evidence: Evidence,
    pub ready: bool,
    pub effect: PacketEffect,
}

impl CapabilityRoute {
    pub fn submission(self) -> EvidenceSubmission {
        EvidenceSubmission::with_effect(self.gate, self.evidence, self.ready, self.effect)
    }

    pub fn is_valid(self) -> bool {
        self.ordinal != 0
            && self.priority != 0
            && expected_evidence(self.gate) == self.evidence
            && expected_effect(self.gate) == self.effect
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrchestrationRecord {
    pub objective_id: u64,
    pub route_version: u64,
    pub priority: u8,
    pub route_hash: u64,
    pub routes: Vec<CapabilityRoute>,
}

impl OrchestrationRecord {
    pub fn from_state(state: State, priority: u8) -> Self {
        let priority = priority.max(1);
        let mut scratch = state;
        let mut routes = Vec::new();

        for gate in EXECUTION_GATE_ORDER {
            if scratch.gates.get(gate).status == GateStatus::Pass {
                continue;
            }

            let evidence = expected_evidence(gate);
            let effect = expected_effect(gate);
            let ready = route_ready(scratch, gate);
            let route = CapabilityRoute {
                ordinal: routes.len().saturating_add(1).min(u8::MAX as usize) as u8,
                priority,
                gate,
                evidence,
                ready,
                effect,
            };

            if ready {
                effect.apply_to(&mut scratch);
                scratch.apply_evidence(gate, evidence, true);
            }

            routes.push(route);
        }

        let route_hash = route_hash(
            state.packet.objective_id,
            ORCHESTRATION_ROUTE_VERSION,
            priority,
            &routes,
        );

        Self {
            objective_id: state.packet.objective_id,
            route_version: ORCHESTRATION_ROUTE_VERSION,
            priority,
            route_hash,
            routes,
        }
    }

    pub fn decision(&self) -> OrchestrationDecision {
        if self.is_valid() && self.routes.iter().any(|route| route.ready) {
            OrchestrationDecision::Routed
        } else {
            OrchestrationDecision::Empty
        }
    }

    pub fn is_valid(&self) -> bool {
        self.objective_id != 0
            && self.route_version == ORCHESTRATION_ROUTE_VERSION
            && self.priority != 0
            && !self.routes.is_empty()
            && self
                .routes
                .iter()
                .copied()
                .all(CapabilityRoute::is_valid)
            && self
                .routes
                .windows(2)
                .all(|pair| pair[0].ordinal < pair[1].ordinal)
            && self.route_hash
                == route_hash(
                    self.objective_id,
                    self.route_version,
                    self.priority,
                    &self.routes,
                )
    }

    pub fn ordered_submissions(&self) -> Vec<EvidenceSubmission> {
        if self.decision() != OrchestrationDecision::Routed {
            return Vec::new();
        }

        let mut routes = self.routes.clone();
        routes.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.ordinal.cmp(&b.ordinal))
                .then_with(|| (a.gate as u8).cmp(&(b.gate as u8)))
        });
        routes
            .into_iter()
            .filter(|route| route.ready)
            .map(CapabilityRoute::submission)
            .collect()
    }
}

fn route_ready(state: State, gate: GateId) -> bool {
    match gate {
        GateId::Invariant | GateId::Analysis | GateId::Judgment => true,
        GateId::Plan => state.packet.objective_id != 0 && state.packet.objective_required_tasks != 0,
        GateId::Execution => state.packet.has_ready_task(),
        GateId::Verification => state.packet.artifact_receipt_valid(),
        GateId::Eval => state.packet.lineage_valid(),
        GateId::Learning => false,
    }
}

fn expected_evidence(gate: GateId) -> Evidence {
    match gate {
        GateId::Invariant => Evidence::InvariantProof,
        GateId::Analysis => Evidence::AnalysisReport,
        GateId::Judgment => Evidence::JudgmentRecord,
        GateId::Plan => Evidence::TaskReady,
        GateId::Execution => Evidence::ArtifactReceipt,
        GateId::Verification => Evidence::LineageProof,
        GateId::Eval => Evidence::EvalScore,
        GateId::Learning => Evidence::PolicyPromotion,
    }
}

fn expected_effect(gate: GateId) -> PacketEffect {
    match gate {
        GateId::Plan => PacketEffect::BindReadyTask,
        GateId::Execution => PacketEffect::MaterializeArtifact,
        GateId::Verification => PacketEffect::RepairLineage,
        GateId::Eval => PacketEffect::CompleteObjective,
        GateId::Invariant | GateId::Analysis | GateId::Judgment | GateId::Learning => {
            PacketEffect::None
        }
    }
}

fn route_hash(
    objective_id: u64,
    route_version: u64,
    priority: u8,
    routes: &[CapabilityRoute],
) -> u64 {
    let mut h = 0xbf58476d1ce4e5b9u64;
    h = mix(h, objective_id);
    h = mix(h, route_version);
    h = mix(h, priority as u64);
    for route in routes {
        h = mix(h, route.ordinal as u64);
        h = mix(h, route.priority as u64);
        h = mix(h, route.gate as u64);
        h = mix(h, route.evidence as u64);
        h = mix(h, route.ready as u64);
        h = mix(h, packet_effect_id(route.effect));
    }
    h.max(1)
}

fn packet_effect_id(effect: PacketEffect) -> u64 {
    match effect {
        PacketEffect::None => 0,
        PacketEffect::BindReadyTask => 1,
        PacketEffect::MaterializeArtifact => 2,
        PacketEffect::RepairLineage => 3,
        PacketEffect::CompleteObjective => 4,
    }
}

fn mix(mut h: u64, x: u64) -> u64 {
    h ^= x;
    h = h.wrapping_mul(0x100000001b3);
    h
}