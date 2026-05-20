use std::collections::{BTreeMap, BTreeSet};

use super::{mix, Evidence, FailureClass, GateId, GateSet, Packet, Phase, RecoveryAction};

pub const PLAN_NODE_STATUS_PENDING: u64 = 1;
pub const PLAN_NODE_STATUS_RUNNING: u64 = 2;
pub const PLAN_NODE_STATUS_DONE: u64 = 3;
pub const PLAN_NODE_STATUS_FAILED: u64 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlanEdgeProjection {
    pub from_node_hash: u64,
    pub to_node_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlanEvidenceProjection {
    pub path_hash: u64,
    pub kind_hash: u64,
    pub summary_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlanTaskLeaseProjection {
    pub node_id_hash: u64,
    pub worker_hash: u64,
    pub claim_id: u64,
    pub lease_epoch_ms: u64,
    pub lease_expires_at_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlanTaskFailureProjection {
    pub node_id_hash: u64,
    pub worker_hash: u64,
    pub claim_id: u64,
    pub evidence_hash: u64,
    pub failed_at_ms: u64,
    pub retry_after_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanNodeProjection {
    pub node_id_hash: u64,
    pub title_hash: u64,
    pub description_hash: u64,
    pub status: u64,
    pub assignee_hash: u64,
    pub score_axes_hash: u64,
    pub files_hash: u64,
    pub evidence: Vec<PlanEvidenceProjection>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanStatePatch {
    NodeUpsert { node_id_hash: u64, title_hash: u64, description_hash: u64, status: u64, assignee_hash: u64, score_axes_hash: u64, files_hash: u64 },
    EdgeAdd(PlanEdgeProjection),
    EdgeRemove(PlanEdgeProjection),
    NodeRemove { node_id_hash: u64 },
    StatusChange { node_id_hash: u64, status: u64 },
    AssigneeChange { node_id_hash: u64, assignee_hash: u64 },
    EvidenceAppend { node_id_hash: u64, evidence: PlanEvidenceProjection },
    FullImport { node_count: u64, edge_count: u64, nodes_hash: u64, edges_hash: u64 },
    TaskClaim { node_id_hash: u64, worker_hash: u64, claim_id: u64, lease_epoch_ms: u64, lease_expires_at_ms: u64 },
    TaskLeaseRenewed { node_id_hash: u64, worker_hash: u64, claim_id: u64, lease_epoch_ms: u64, lease_expires_at_ms: u64 },
    TaskLeaseExpired { node_id_hash: u64, claim_id: u64, expired_at_ms: u64 },
    TaskCompleted { node_id_hash: u64, worker_hash: u64, claim_id: u64, completed_at_ms: u64 },
    TaskFailed { node_id_hash: u64, worker_hash: u64, claim_id: u64, evidence_hash: u64, failed_at_ms: u64, retry_after_ms: u64 },
    TaskRetryEligible { node_id_hash: u64, failure_claim_id: u64, eligible_at_ms: u64 },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlanState {
    pub revision: u64,
    pub nodes: BTreeMap<u64, PlanNodeProjection>,
    pub edges: BTreeSet<PlanEdgeProjection>,
    pub imported_nodes_hash: u64,
    pub imported_edges_hash: u64,
    pub task_leases: BTreeMap<u64, PlanTaskLeaseProjection>,
    pub task_failures: BTreeMap<u64, PlanTaskFailureProjection>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct State {
    pub phase: Phase,
    pub gates: GateSet,
    pub packet: Packet,
    pub failure: Option<FailureClass>,
    pub recovery_action: Option<RecoveryAction>,
    pub recovery_attempts: u8,
    /// Number of child DAG tasks currently in flight (wave fan-out counter).
    /// Set by WaveDispatch events; decremented by ChildComplete events.
    /// Stored in TLog so replay can reconstruct the wave state after a crash.
    pub wave_pending: u16,
    /// Hash-bound replay projection for accepted/imported plan events.
    pub plan_state_hash: u64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            phase: Phase::Delta,
            gates: GateSet::default(),
            packet: Packet::empty(),
            failure: None,
            recovery_action: None,
            recovery_attempts: 0,
            wave_pending: 0,
            plan_state_hash: 0,
        }
    }
}

impl State {
    pub fn ready() -> Self {
        Self {
            phase: Phase::Delta,
            gates: GateSet::ready(),
            packet: Packet::ready(),
            failure: None,
            recovery_action: None,
            recovery_attempts: 0,
            wave_pending: 0,
            plan_state_hash: 0,
        }
    }

    pub fn is_success(self) -> bool {
        self.failure.is_none()
            && self.phase == Phase::Done
            && self.gates.all_passed()
            && self.packet.objective_complete()
            && self.packet.lineage_valid()
    }

    pub fn apply_evidence(&mut self, gate: GateId, evidence: Evidence, passed: bool) {
        if passed {
            self.gates.set_pass(gate, evidence);
        } else {
            self.gates.set_fail(gate, evidence);
        }
    }

    pub fn is_structurally_valid(self) -> bool {
        self.gates.is_structurally_valid() && self.packet.is_structurally_valid()
    }
}

impl PlanState {
    pub fn apply_patch(&mut self, mutation: PlanStatePatch) -> Result<u64, PlanStateRejection> {
        self.check_mutation(mutation)?;

        match mutation {
            PlanStatePatch::NodeUpsert { node_id_hash, title_hash, description_hash, status, assignee_hash, score_axes_hash, files_hash } => {
                let evidence = self
                    .nodes
                    .get(&node_id_hash)
                    .map(|node| node.evidence.clone())
                    .unwrap_or_default();
                self.nodes.insert(
                    node_id_hash,
                    PlanNodeProjection {
                        node_id_hash,
                        title_hash,
                        description_hash,
                        status,
                        assignee_hash,
                        score_axes_hash,
                        files_hash,
                        evidence,
                    },
                );
            }
            PlanStatePatch::EdgeAdd(edge) => {
                self.edges.insert(edge);
            }
            PlanStatePatch::EdgeRemove(edge) => {
                self.edges.remove(&edge);
            }
            PlanStatePatch::NodeRemove { node_id_hash } => {
                self.nodes.remove(&node_id_hash);
                self.edges.retain(|edge| edge.from_node_hash != node_id_hash && edge.to_node_hash != node_id_hash);
            }
            PlanStatePatch::StatusChange { node_id_hash, status } => {
                let Some(node) = self.nodes.get_mut(&node_id_hash) else {
                    return Err(PlanStateRejection::MissingNode);
                };
                node.status = status;
            }
            PlanStatePatch::AssigneeChange { node_id_hash, assignee_hash } => {
                let Some(node) = self.nodes.get_mut(&node_id_hash) else {
                    return Err(PlanStateRejection::MissingNode);
                };
                node.assignee_hash = assignee_hash;
            }
            PlanStatePatch::EvidenceAppend { node_id_hash, evidence } => {
                let Some(node) = self.nodes.get_mut(&node_id_hash) else {
                    return Err(PlanStateRejection::MissingNode);
                };
                if !node.evidence.contains(&evidence) {
                    node.evidence.push(evidence);
                    node.evidence.sort();
                }
            }
            PlanStatePatch::FullImport { nodes_hash, edges_hash, .. } => {
                self.imported_nodes_hash = nodes_hash;
                self.imported_edges_hash = edges_hash;
            }
            PlanStatePatch::TaskClaim { node_id_hash, worker_hash, claim_id, lease_epoch_ms, lease_expires_at_ms }
            | PlanStatePatch::TaskLeaseRenewed { node_id_hash, worker_hash, claim_id, lease_epoch_ms, lease_expires_at_ms } => {
                let Some(node) = self.nodes.get_mut(&node_id_hash) else {
                    return Err(PlanStateRejection::MissingNode);
                };
                node.status = PLAN_NODE_STATUS_RUNNING;
                node.assignee_hash = worker_hash;
                self.task_leases.insert(
                    node_id_hash,
                    PlanTaskLeaseProjection { node_id_hash, worker_hash, claim_id, lease_epoch_ms, lease_expires_at_ms },
                );
            }
            PlanStatePatch::TaskLeaseExpired { node_id_hash, .. } => {
                if let Some(node) = self.nodes.get_mut(&node_id_hash) {
                    self.task_leases.remove(&node_id_hash);
                    node.status = PLAN_NODE_STATUS_PENDING;
                    node.assignee_hash = 0;
                }
            }
            PlanStatePatch::TaskCompleted { node_id_hash, .. } => {
                if let Some(node) = self.nodes.get_mut(&node_id_hash) {
                    self.task_leases.remove(&node_id_hash);
                    self.task_failures.remove(&node_id_hash);
                    node.status = PLAN_NODE_STATUS_DONE;
                }
            }
            PlanStatePatch::TaskFailed { node_id_hash, worker_hash, claim_id, evidence_hash, failed_at_ms, retry_after_ms } => {
                if let Some(node) = self.nodes.get_mut(&node_id_hash) {
                    self.task_leases.remove(&node_id_hash);
                    node.status = PLAN_NODE_STATUS_FAILED;
                    node.assignee_hash = worker_hash;
                    self.task_failures.insert(
                        node_id_hash,
                        PlanTaskFailureProjection { node_id_hash, worker_hash, claim_id, evidence_hash, failed_at_ms, retry_after_ms },
                    );
                }
            }
            PlanStatePatch::TaskRetryEligible { node_id_hash, failure_claim_id: _, eligible_at_ms: _ } => {
                if let Some(node) = self.nodes.get_mut(&node_id_hash) {
                    self.task_failures.remove(&node_id_hash);
                    node.status = PLAN_NODE_STATUS_PENDING;
                }
            }
        }

        self.revision = self.revision.saturating_add(1).max(1);
        Ok(self.projection_hash())
    }

    pub fn replay<I>(mutations: I) -> Result<Self, PlanStateRejection>
    where
        I: IntoIterator<Item = PlanStatePatch>,
    {
        let mut state = Self::default();
        for mutation in mutations {
            state.apply_patch(mutation)?;
        }
        Ok(state)
    }

    pub fn nodes_hash(&self) -> u64 {
        let mut h = mix(0x504c_414e_4e4f_4445u64, self.nodes.len() as u64);
        for node in self.nodes.values() {
            h = mix(h, node.node_id_hash);
            h = mix(h, node.title_hash);
            h = mix(h, node.description_hash);
            h = mix(h, node.status);
            h = mix(h, node.assignee_hash);
            h = mix(h, node.score_axes_hash);
            h = mix(h, node.files_hash);
            h = mix(h, node.evidence.len() as u64);
            for evidence in &node.evidence {
                h = mix(h, evidence.path_hash);
                h = mix(h, evidence.kind_hash);
                h = mix(h, evidence.summary_hash);
            }
        }
        h.max(1)
    }

    pub fn edges_hash(&self) -> u64 {
        let mut h = mix(0x504c_414e_4544_4745u64, self.edges.len() as u64);
        for edge in &self.edges {
            h = mix(h, edge.from_node_hash);
            h = mix(h, edge.to_node_hash);
        }
        h.max(1)
    }

    pub fn projection_hash(&self) -> u64 {
        let mut h = mix(0x504c_414e_5354_4154u64, self.revision);
        h = mix(h, self.imported_nodes_hash);
        h = mix(h, self.imported_edges_hash);
        h = mix(h, self.nodes_hash());
        h = mix(h, self.edges_hash());
        h = mix(h, self.task_leases_hash());
        h = mix(h, self.task_failures_hash());
        h.max(1)
    }

    pub fn task_leases_hash(&self) -> u64 {
        let mut h = mix(0x504c_414e_4c45_4153u64, self.task_leases.len() as u64);
        for lease in self.task_leases.values() {
            h = mix(h, lease.node_id_hash);
            h = mix(h, lease.worker_hash);
            h = mix(h, lease.claim_id);
            h = mix(h, lease.lease_epoch_ms);
            h = mix(h, lease.lease_expires_at_ms);
        }
        h.max(1)
    }

    pub fn task_failures_hash(&self) -> u64 {
        let mut h = mix(0x504c_414e_4641_494cu64, self.task_failures.len() as u64);
        for failure in self.task_failures.values() {
            h = mix(h, failure.node_id_hash);
            h = mix(h, failure.worker_hash);
            h = mix(h, failure.claim_id);
            h = mix(h, failure.evidence_hash);
            h = mix(h, failure.failed_at_ms);
            h = mix(h, failure.retry_after_ms);
        }
        h.max(1)
    }

    pub fn check_mutation(&self, mutation: PlanStatePatch) -> Result<(), PlanStateRejection> {
        match mutation {
            PlanStatePatch::NodeUpsert { node_id_hash, title_hash, status, .. } => {
                if node_id_hash != 0 && title_hash != 0 && is_valid_plan_node_status(status) {
                    Ok(())
                } else {
                    Err(PlanStateRejection::InvalidNode)
                }
            }
            PlanStatePatch::EdgeAdd(edge) => self.check_edge_add(edge),
            PlanStatePatch::EdgeRemove(edge) => {
                if self.edges.contains(&edge) { Ok(()) } else { Err(PlanStateRejection::MissingEdge) }
            }
            PlanStatePatch::NodeRemove { node_id_hash } => {
                if self.nodes.contains_key(&node_id_hash) { Ok(()) } else { Err(PlanStateRejection::MissingNode) }
            }
            PlanStatePatch::StatusChange { node_id_hash, status } => {
                if !is_valid_plan_node_status(status) {
                    Err(PlanStateRejection::InvalidStatus)
                } else if self.nodes.contains_key(&node_id_hash) {
                    Ok(())
                } else {
                    Err(PlanStateRejection::MissingNode)
                }
            }
            PlanStatePatch::AssigneeChange { node_id_hash, .. } => {
                if self.nodes.contains_key(&node_id_hash) { Ok(()) } else { Err(PlanStateRejection::MissingNode) }
            }
            PlanStatePatch::EvidenceAppend { node_id_hash, evidence } => {
                if evidence.path_hash == 0 || evidence.kind_hash == 0 || evidence.summary_hash == 0 {
                    Err(PlanStateRejection::InvalidEvidence)
                } else if self.nodes.contains_key(&node_id_hash) {
                    Ok(())
                } else {
                    Err(PlanStateRejection::MissingNode)
                }
            }
            PlanStatePatch::FullImport { node_count, edge_count, nodes_hash, edges_hash } => {
                if self.imported_nodes_hash != 0 || self.imported_edges_hash != 0 {
                    Err(PlanStateRejection::DuplicateImport)
                } else if nodes_hash == 0
                    || edges_hash == 0
                    || node_count as usize != self.nodes.len()
                    || edge_count as usize != self.edges.len()
                    || self.nodes_hash() != nodes_hash
                    || self.edges_hash() != edges_hash
                {
                    Err(PlanStateRejection::InvalidImport)
                } else {
                    Ok(())
                }
            }
            _ => self.check_task_mutation(mutation),
        }
    }

    fn check_edge_add(&self, edge: PlanEdgeProjection) -> Result<(), PlanStateRejection> {
        if edge.from_node_hash == 0 || edge.to_node_hash == 0 || edge.from_node_hash == edge.to_node_hash {
            return Err(PlanStateRejection::InvalidEdge);
        }
        if !self.nodes.contains_key(&edge.from_node_hash) || !self.nodes.contains_key(&edge.to_node_hash) {
            return Err(PlanStateRejection::MissingNode);
        }
        Ok(())
    }

    fn check_task_mutation(&self, mutation: PlanStatePatch) -> Result<(), PlanStateRejection> {
        match mutation {
            PlanStatePatch::TaskClaim { node_id_hash, worker_hash, claim_id, lease_epoch_ms, lease_expires_at_ms } => {
                if node_id_hash == 0 || worker_hash == 0 || claim_id == 0 || lease_expires_at_ms <= lease_epoch_ms {
                    return Err(PlanStateRejection::InvalidTaskLease);
                }
                let Some(node) = self.nodes.get(&node_id_hash) else { return Err(PlanStateRejection::MissingNode); };
                if node.status != PLAN_NODE_STATUS_PENDING { return Err(PlanStateRejection::TaskNotReady); }
                if self.task_leases.get(&node_id_hash).is_some_and(|lease| lease.lease_expires_at_ms > lease_epoch_ms) {
                    return Err(PlanStateRejection::TaskAlreadyClaimed);
                }
                Ok(())
            }
            PlanStatePatch::TaskLeaseRenewed { node_id_hash, worker_hash, claim_id, lease_epoch_ms, lease_expires_at_ms } => {
                if lease_expires_at_ms <= lease_epoch_ms { return Err(PlanStateRejection::InvalidTaskLease); }
                let Some(lease) = self.task_leases.get(&node_id_hash) else { return Err(PlanStateRejection::MissingTaskLease); };
                if lease.worker_hash != worker_hash || lease.claim_id != claim_id { return Err(PlanStateRejection::TaskLeaseMismatch); }
                if lease.lease_expires_at_ms < lease_epoch_ms { return Err(PlanStateRejection::TaskLeaseExpired); }
                Ok(())
            }
            _ => self.check_task_outcome(mutation),
        }
    }

    fn check_task_terminal(&self, node_id_hash: u64, worker_hash: u64, claim_id: u64, event_at_ms: u64) -> Result<(), PlanStateRejection> {
        let Some(lease) = self.task_leases.get(&node_id_hash) else { return Err(PlanStateRejection::MissingTaskLease); };
        if lease.worker_hash != worker_hash || lease.claim_id != claim_id { return Err(PlanStateRejection::TaskLeaseMismatch); }
        if lease.lease_expires_at_ms < event_at_ms { return Err(PlanStateRejection::TaskLeaseExpired); }
        Ok(())
    }

    fn check_task_outcome(&self, mutation: PlanStatePatch) -> Result<(), PlanStateRejection> {
        match mutation {
            PlanStatePatch::TaskLeaseExpired { node_id_hash, claim_id, expired_at_ms } => {
                let Some(lease) = self.task_leases.get(&node_id_hash) else { return Err(PlanStateRejection::MissingTaskLease); };
                if lease.claim_id != claim_id { return Err(PlanStateRejection::TaskLeaseMismatch); }
                if lease.lease_expires_at_ms > expired_at_ms { return Err(PlanStateRejection::TaskNotReady); }
                Ok(())
            }
            PlanStatePatch::TaskCompleted { node_id_hash, worker_hash, claim_id, completed_at_ms } => {
                self.check_task_terminal(node_id_hash, worker_hash, claim_id, completed_at_ms)
            }
            PlanStatePatch::TaskFailed { node_id_hash, worker_hash, claim_id, evidence_hash, failed_at_ms, retry_after_ms } => {
                if evidence_hash == 0 || retry_after_ms < failed_at_ms { return Err(PlanStateRejection::InvalidTaskEvidence); }
                self.check_task_terminal(node_id_hash, worker_hash, claim_id, failed_at_ms)
            }
            PlanStatePatch::TaskRetryEligible { node_id_hash, failure_claim_id, eligible_at_ms } => {
                let Some(failure) = self.task_failures.get(&node_id_hash) else { return Err(PlanStateRejection::MissingTaskFailure); };
                if failure.claim_id != failure_claim_id { return Err(PlanStateRejection::TaskLeaseMismatch); }
                if failure.retry_after_ms > eligible_at_ms { return Err(PlanStateRejection::TaskRetryNotEligible); }
                Ok(())
            }
            _ => Err(PlanStateRejection::InvalidTaskLease),
        }
    }

    pub fn apply_checked_mutation(&mut self, mutation: PlanStatePatch) -> Result<u64, PlanStateRejection> {
        self.check_mutation(mutation)?;
        match mutation {
            PlanStatePatch::NodeUpsert { node_id_hash, title_hash, description_hash, status, assignee_hash, score_axes_hash, files_hash } => {
                let evidence = self.nodes.get(&node_id_hash).map(|node| node.evidence.clone()).unwrap_or_default();
                self.nodes.insert(node_id_hash, PlanNodeProjection { node_id_hash, title_hash, description_hash, status, assignee_hash, score_axes_hash, files_hash, evidence });
            }
            PlanStatePatch::EdgeAdd(edge) => { self.edges.insert(edge); }
            PlanStatePatch::EdgeRemove(edge) => { self.edges.remove(&edge); }
            PlanStatePatch::NodeRemove { node_id_hash } => {
                self.nodes.remove(&node_id_hash);
                self.edges.retain(|edge| edge.from_node_hash != node_id_hash && edge.to_node_hash != node_id_hash);
            }
            _ => self.apply_checked_nonstructural_mutation(mutation),
        }
        self.revision = self.revision.saturating_add(1);
        Ok(self.revision)
    }

    fn apply_checked_nonstructural_mutation(&mut self, mutation: PlanStatePatch) {
        match mutation {
            PlanStatePatch::StatusChange { node_id_hash, status } => {
                if let Some(node) = self.nodes.get_mut(&node_id_hash) { node.status = status; }
            }
            PlanStatePatch::AssigneeChange { node_id_hash, assignee_hash } => {
                if let Some(node) = self.nodes.get_mut(&node_id_hash) { node.assignee_hash = assignee_hash; }
            }
            PlanStatePatch::EvidenceAppend { node_id_hash, evidence } => {
                if let Some(node) = self.nodes.get_mut(&node_id_hash) { node.evidence.push(evidence); }
            }
            PlanStatePatch::FullImport { nodes_hash, edges_hash, .. } => {
                self.nodes.clear();
                self.edges.clear();
                self.imported_nodes_hash = nodes_hash;
                self.imported_edges_hash = edges_hash;
            }
            _ => {}
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanStateRejection {
    InvalidNode,
    InvalidEdge,
    MissingNode,
    MissingEdge,
    InvalidStatus,
    InvalidEvidence,
    InvalidImport,
    DuplicateImport,
    InvalidTaskLease,
    TaskAlreadyClaimed,
    TaskNotReady,
    MissingTaskLease,
    TaskLeaseMismatch,
    TaskLeaseExpired,
    InvalidTaskEvidence,
    MissingTaskFailure,
    TaskRetryNotEligible,
}

impl PlanStateRejection {
    pub fn reason_hash(self) -> u64 {
        mix(0x504c_414e_5245_4a01u64, self as u64 + 1).max(1)
    }
}

fn is_valid_plan_node_status(status: u64) -> bool {
    matches!(status, 1 | 2 | 3 | 4 | 5)
}
