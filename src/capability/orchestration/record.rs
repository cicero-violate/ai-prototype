//! Deterministic routing records owned by the orchestration capability.

use crate::capability::{EvidenceSubmission, PacketEffect};
use crate::kernel::{mix, Evidence, GateId, GateStatus, State, EXECUTION_GATE_ORDER};

const ORCHESTRATION_ROUTE_VERSION: u64 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrchestrationDecision {
    Routed,
    Empty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrchestrationBatchDecision {
    Selected,
    Empty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrchestrationBudget {
    pub max_parallel_runs: u8,
    pub max_submissions: u8,
    pub resource_units: u16,
}

impl OrchestrationBudget {
    pub const fn new(max_parallel_runs: u8, max_submissions: u8, resource_units: u16) -> Self {
        Self {
            max_parallel_runs,
            max_submissions,
            resource_units,
        }
    }

    pub const fn is_valid(self) -> bool {
        self.max_parallel_runs != 0 && self.max_submissions != 0 && self.resource_units != 0
    }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectedCapabilityRoute {
    pub run_ordinal: u8,
    pub objective_id: u64,
    pub route_hash: u64,
    pub route: CapabilityRoute,
    pub resource_units: u16,
}

impl SelectedCapabilityRoute {
    pub fn submission(self) -> EvidenceSubmission {
        self.route.submission(self.route_hash)
    }

    pub fn is_valid(self) -> bool {
        self.run_ordinal != 0
            && self.objective_id != 0
            && self.route_hash != 0
            && self.route.ready
            && self.route.is_valid()
            && self.resource_units == route_resource_units(self.route)
    }
}

impl CapabilityRoute {
    pub fn submission(self, route_hash: u64) -> EvidenceSubmission {
        EvidenceSubmission::with_effect_payload(
            self.gate,
            self.evidence,
            self.ready,
            self.effect,
            route_payload_hash(route_hash, self),
        )
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrchestrationBatchRecord {
    pub batch_version: u64,
    pub budget: OrchestrationBudget,
    pub candidate_count: u8,
    pub selected: Vec<SelectedCapabilityRoute>,
    pub consumed_resource_units: u16,
    pub merge_hash: u64,
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
            && self.routes.iter().copied().all(CapabilityRoute::is_valid)
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
            .map(|route| route.submission(self.route_hash))
            .collect()
    }
}

impl OrchestrationBatchRecord {
    pub fn from_records(records: &[OrchestrationRecord], budget: OrchestrationBudget) -> Self {
        let candidate_count = records.len().min(u8::MAX as usize) as u8;
        let mut candidates = Vec::new();

        if budget.is_valid() {
            for (idx, record) in records.iter().enumerate() {
                if record.decision() != OrchestrationDecision::Routed {
                    continue;
                }

                let run_ordinal = idx.saturating_add(1).min(u8::MAX as usize) as u8;
                for route in record.routes.iter().copied().filter(|route| route.ready) {
                    candidates.push(SelectedCapabilityRoute {
                        run_ordinal,
                        objective_id: record.objective_id,
                        route_hash: record.route_hash,
                        route,
                        resource_units: route_resource_units(route),
                    });
                }
            }
        }

        candidates.sort_by(|a, b| {
            b.route
                .priority
                .cmp(&a.route.priority)
                .then_with(|| a.run_ordinal.cmp(&b.run_ordinal))
                .then_with(|| a.route.ordinal.cmp(&b.route.ordinal))
                .then_with(|| a.objective_id.cmp(&b.objective_id))
                .then_with(|| a.route_hash.cmp(&b.route_hash))
        });

        let mut selected = Vec::new();
        let mut active_runs: Vec<u8> = Vec::new();
        let mut consumed_resource_units = 0u16;

        for candidate in candidates {
            if selected.len() >= budget.max_submissions as usize {
                break;
            }

            let is_active_run = active_runs.contains(&candidate.run_ordinal);
            if !is_active_run && active_runs.len() >= budget.max_parallel_runs as usize {
                continue;
            }

            let Some(next_units) = consumed_resource_units.checked_add(candidate.resource_units)
            else {
                continue;
            };
            if next_units > budget.resource_units {
                continue;
            }

            if !is_active_run {
                active_runs.push(candidate.run_ordinal);
                active_runs.sort_unstable();
            }
            consumed_resource_units = next_units;
            selected.push(candidate);
        }

        let mut record = Self {
            batch_version: ORCHESTRATION_ROUTE_VERSION,
            budget,
            candidate_count,
            selected,
            consumed_resource_units,
            merge_hash: 0,
        };
        record.merge_hash = record.expected_merge_hash();
        record
    }

    pub fn decision(&self) -> OrchestrationBatchDecision {
        if self.is_valid() && !self.selected.is_empty() {
            OrchestrationBatchDecision::Selected
        } else {
            OrchestrationBatchDecision::Empty
        }
    }

    pub fn is_valid(&self) -> bool {
        self.batch_version == ORCHESTRATION_ROUTE_VERSION
            && self.budget.is_valid()
            && self.candidate_count != 0
            && self.selected.len() <= self.budget.max_submissions as usize
            && self
                .selected
                .iter()
                .copied()
                .all(SelectedCapabilityRoute::is_valid)
            && selected_parallel_run_count(&self.selected) <= self.budget.max_parallel_runs as usize
            && self.consumed_resource_units
                == self
                    .selected
                    .iter()
                    .map(|route| route.resource_units)
                    .try_fold(0u16, |acc, units| acc.checked_add(units))
                    .unwrap_or(0)
            && self.consumed_resource_units <= self.budget.resource_units
            && self.merge_hash == self.expected_merge_hash()
    }

    pub fn ordered_submissions(&self) -> Vec<EvidenceSubmission> {
        if self.decision() != OrchestrationBatchDecision::Selected {
            return Vec::new();
        }
        self.selected
            .iter()
            .copied()
            .map(SelectedCapabilityRoute::submission)
            .collect()
    }

    pub fn expected_merge_hash(&self) -> u64 {
        let mut h = 0x94d0_49bb_1331_11ebu64;
        h = mix(h, self.batch_version);
        h = mix(h, self.budget.max_parallel_runs as u64);
        h = mix(h, self.budget.max_submissions as u64);
        h = mix(h, self.budget.resource_units as u64);
        h = mix(h, self.candidate_count as u64);
        h = mix(h, self.consumed_resource_units as u64);
        for selected in &self.selected {
            h = mix(h, selected.run_ordinal as u64);
            h = mix(h, selected.objective_id);
            h = mix(h, selected.route_hash);
            h = mix(h, selected.resource_units as u64);
            h = mix(h, route_payload_hash(selected.route_hash, selected.route));
        }
        h.max(1)
    }
}

fn selected_parallel_run_count(selected: &[SelectedCapabilityRoute]) -> usize {
    let mut runs: Vec<u8> = Vec::new();
    for route in selected {
        if !runs.contains(&route.run_ordinal) {
            runs.push(route.run_ordinal);
        }
    }
    runs.len()
}

fn route_resource_units(route: CapabilityRoute) -> u16 {
    match route.gate {
        GateId::Invariant | GateId::Analysis | GateId::Judgment => 1,
        GateId::Plan => 2,
        GateId::Execution => 5,
        GateId::Verification => 3,
        GateId::Eval => 2,
        GateId::Learning => 4,
    }
}

fn route_payload_hash(route_hash: u64, route: CapabilityRoute) -> u64 {
    let mut h = 0x243f_6a88_85a3_08d3u64;
    h = mix(h, route_hash);
    h = mix(h, route.ordinal as u64);
    h = mix(h, route.priority as u64);
    h = mix(h, route.gate as u64);
    h = mix(h, route.evidence as u64);
    h = mix(h, route.ready as u64);
    h = mix(h, packet_effect_id(route.effect));
    h.max(1)
}

fn route_ready(state: State, gate: GateId) -> bool {
    match gate {
        GateId::Invariant | GateId::Analysis | GateId::Judgment => true,
        GateId::Plan => {
            state.packet.objective_id != 0 && state.packet.objective_required_tasks != 0
        }
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
