//! Durable planning payload owned by the planning capability.
//!
//! The planner stays deterministic: it derives an ordered task graph from the
//! objective packet, records dependencies and lineage, and exposes the same
//! kernel-visible `TaskReady` evidence used by the runtime.

use crate::capability::{EvidenceProducer, EvidenceSubmission, PacketEffect};
use crate::kernel::{
    mix, Evidence, GateId, Packet, PlanEdgeProjection, PlanEvidenceProjection,
    PlanStatePatch,
};
use serde::Deserialize;

const PLAN_SCHEMA_VERSION: u64 = 2;
const MAX_PLAN_TASKS: u8 = 8;
pub const PLAN_RECEIPT_SCHEMA_VERSION: u64 = 1;
pub const PLAN_RECEIPT_RECORD: u64 = 0x91a0_0001;
pub const PLAN_PATCH_SCHEMA_VERSION: u64 = 1;
pub const PLAN_PATCH_RECORD: u64 = 0x91a0_1001;
pub const PLAN_PATCH_ACCEPTED_RECORD: u64 = 0x91a0_1002;
pub const PLAN_PATCH_REJECTED_RECORD: u64 = 0x91a0_1003;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanDecision {
    Ready,
    Blocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanTask {
    pub task_id: u64,
    pub depends_on: u64,
    pub ordinal: u8,
    pub ready: bool,
}

impl PlanTask {
    pub fn is_valid(self, objective_id: u64) -> bool {
        self.task_id != 0
            && self.ordinal != 0
            && self.task_id / 100 == objective_id
            && (self.depends_on == 0 || self.depends_on / 100 == objective_id)
    }

    fn hash(self) -> u64 {
        let mut h = 0x5441_534b_504c_414eu64;
        h = mix(h, self.task_id);
        h = mix(h, self.depends_on);
        h = mix(h, u64::from(self.ordinal));
        h = mix(h, u64::from(self.ready));
        h.max(1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanRecord {
    pub objective_id: u64,
    pub task_id: u64,
    pub ready_tasks: u8,
    pub dependency_hash: u64,
    pub objective_hash: u64,
    pub plan_version: u64,
    pub plan_revision: u64,
    pub task_count: u8,
    pub completed_tasks: u8,
    pub ready_set_hash: u64,
    pub lineage_hash: u64,
    pub tasks: Vec<PlanTask>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanReceipt {
    pub schema_version: u64,
    pub record_type: u64,
    pub objective_id: u64,
    pub task_id: u64,
    pub task_count: u64,
    pub completed_tasks: u64,
    pub ready_tasks: u64,
    pub plan_version: u64,
    pub plan_revision: u64,
    pub objective_hash: u64,
    pub dependency_hash: u64,
    pub ready_set_hash: u64,
    pub lineage_hash: u64,
    pub payload_hash: u64,
    pub verdict: PlanDecision,
    pub receipt_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PlanNodeStatus {
    Pending = 1,
    Running = 2,
    Done = 3,
    Failed = 4,
    Skipped = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PlanPatchKind {
    NodeUpsert = 1,
    EdgeAdd = 2,
    EdgeRemove = 3,
    NodeRemove = 4,
    StatusChange = 5,
    AssigneeChange = 6,
    EvidenceAppend = 7,
    FullImport = 8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanNodeUpsertPatch {
    pub node_id_hash: u64,
    pub title_hash: u64,
    pub description_hash: u64,
    pub status: PlanNodeStatus,
    pub assignee_hash: u64,
    pub score_axes_hash: u64,
    pub files_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanEdgePatch {
    pub from_node_hash: u64,
    pub to_node_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanNodeRemovePatch {
    pub node_id_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanStatusChangePatch {
    pub node_id_hash: u64,
    pub status: PlanNodeStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanAssigneeChangePatch {
    pub node_id_hash: u64,
    pub assignee_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanEvidenceAppendPatch {
    pub node_id_hash: u64,
    pub path_hash: u64,
    pub kind_hash: u64,
    pub summary_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanFullImportPatch {
    pub node_count: u64,
    pub edge_count: u64,
    pub nodes_hash: u64,
    pub edges_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanPatchPayload {
    NodeUpsert(PlanNodeUpsertPatch),
    EdgeAdd(PlanEdgePatch),
    EdgeRemove(PlanEdgePatch),
    NodeRemove(PlanNodeRemovePatch),
    StatusChange(PlanStatusChangePatch),
    AssigneeChange(PlanAssigneeChangePatch),
    EvidenceAppend(PlanEvidenceAppendPatch),
    FullImport(PlanFullImportPatch),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanPatchRecord {
    pub schema_version: u64,
    pub record_type: u64,
    pub source_hash: u64,
    pub cycle_id: u64,
    pub patch_seq: u64,
    pub contract_hash: u64,
    pub payload: PlanPatchPayload,
    pub payload_hash: u64,
    pub patch_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AcceptedPlanPatchRecord {
    pub schema_version: u64,
    pub record_type: u64,
    pub source_hash: u64,
    pub cycle_id: u64,
    pub patch_seq: u64,
    pub contract_hash: u64,
    pub patch_hash: u64,
    pub applied_revision: u64,
    pub acceptance_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RejectedPlanPatchRecord {
    pub schema_version: u64,
    pub record_type: u64,
    pub source_hash: u64,
    pub cycle_id: u64,
    pub patch_seq: u64,
    pub contract_hash: u64,
    pub patch_hash: u64,
    pub reason_hash: u64,
    pub rejection_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanJsonImportEvent {
    pub source_hash: u64,
    pub cycle_id: u64,
    pub first_patch_seq: u64,
    pub patches: Vec<PlanPatchRecord>,
    pub node_count: u64,
    pub edge_count: u64,
    pub nodes_hash: u64,
    pub edges_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanJsonImportError {
    InvalidJson,
    MissingNodeId,
    MissingNodeTitle,
    MissingEdgeEndpoint,
    InvalidStatus,
    EmptyImport,
    ReplayRejected,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
struct PlanJsonDocument {
    #[serde(default)]
    nodes: Vec<PlanJsonNode>,
    #[serde(default)]
    edges: Vec<PlanJsonEdge>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
struct PlanJsonNode {
    id: String,
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default = "default_pending_status")]
    status: String,
    #[serde(default)]
    assignee: String,
    #[serde(default)]
    score_axes: Vec<String>,
    #[serde(default)]
    files: Vec<String>,
    #[serde(default)]
    evidence: Vec<PlanJsonEvidence>,
}

#[allow(dead_code)]
fn default_pending_status() -> String {
    "pending".to_string()
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
struct PlanJsonEvidence {
    path: String,
    kind: String,
    summary: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
struct PlanJsonEdge {
    from: String,
    to: String,
}

impl PlanPatchPayload {
    pub fn kind(self) -> PlanPatchKind {
        match self {
            Self::NodeUpsert(_) => PlanPatchKind::NodeUpsert,
            Self::EdgeAdd(_) => PlanPatchKind::EdgeAdd,
            Self::EdgeRemove(_) => PlanPatchKind::EdgeRemove,
            Self::NodeRemove(_) => PlanPatchKind::NodeRemove,
            Self::StatusChange(_) => PlanPatchKind::StatusChange,
            Self::AssigneeChange(_) => PlanPatchKind::AssigneeChange,
            Self::EvidenceAppend(_) => PlanPatchKind::EvidenceAppend,
            Self::FullImport(_) => PlanPatchKind::FullImport,
        }
    }

    pub fn canonical_hash(self) -> u64 {
        match self {
            Self::NodeUpsert(payload) => fold_ordered_plan_hash(
                0x504c_414e_5550_0001u64,
                &[
                    PlanPatchKind::NodeUpsert as u64,
                    payload.node_id_hash,
                    payload.title_hash,
                    payload.description_hash,
                    payload.status as u64,
                    payload.assignee_hash,
                    payload.score_axes_hash,
                    payload.files_hash,
                ],
            ),
            Self::EdgeAdd(payload) => fold_ordered_plan_hash(
                0x504c_414e_4541_0001u64,
                &[
                    PlanPatchKind::EdgeAdd as u64,
                    payload.from_node_hash,
                    payload.to_node_hash,
                ],
            ),
            Self::EdgeRemove(payload) => fold_ordered_plan_hash(
                0x504c_414e_4552_0001u64,
                &[
                    PlanPatchKind::EdgeRemove as u64,
                    payload.from_node_hash,
                    payload.to_node_hash,
                ],
            ),
            Self::NodeRemove(payload) => fold_ordered_plan_hash(
                0x504c_414e_4e52_0001u64,
                &[PlanPatchKind::NodeRemove as u64, payload.node_id_hash],
            ),
            Self::StatusChange(payload) => fold_ordered_plan_hash(
                0x504c_414e_5354_0001u64,
                &[
                    PlanPatchKind::StatusChange as u64,
                    payload.node_id_hash,
                    payload.status as u64,
                ],
            ),
            Self::AssigneeChange(payload) => fold_ordered_plan_hash(
                0x504c_414e_4153_0001u64,
                &[
                    PlanPatchKind::AssigneeChange as u64,
                    payload.node_id_hash,
                    payload.assignee_hash,
                ],
            ),
            Self::EvidenceAppend(payload) => fold_ordered_plan_hash(
                0x504c_414e_4556_0001u64,
                &[
                    PlanPatchKind::EvidenceAppend as u64,
                    payload.node_id_hash,
                    payload.path_hash,
                    payload.kind_hash,
                    payload.summary_hash,
                ],
            ),
            Self::FullImport(payload) => fold_ordered_plan_hash(
                0x504c_414e_494d_0001u64,
                &[
                    PlanPatchKind::FullImport as u64,
                    payload.node_count,
                    payload.edge_count,
                    payload.nodes_hash,
                    payload.edges_hash,
                ],
            ),
        }
    }

    pub fn is_valid(self) -> bool {
        match self {
            Self::NodeUpsert(payload) => payload.node_id_hash != 0 && payload.title_hash != 0,
            Self::EdgeAdd(payload) | Self::EdgeRemove(payload) => {
                payload.from_node_hash != 0
                    && payload.to_node_hash != 0
                    && payload.from_node_hash != payload.to_node_hash
            }
            Self::NodeRemove(payload) => payload.node_id_hash != 0,
            Self::StatusChange(payload) => payload.node_id_hash != 0,
            Self::AssigneeChange(payload) => payload.node_id_hash != 0,
            Self::EvidenceAppend(payload) => {
                payload.node_id_hash != 0
                    && payload.path_hash != 0
                    && payload.kind_hash != 0
                    && payload.summary_hash != 0
            }
            Self::FullImport(payload) => {
                payload.nodes_hash != 0
                    && payload.edges_hash != 0
                    && (payload.node_count != 0 || payload.edge_count == 0)
            }
        }
    }
}

impl PlanPatchPayload {
    pub fn plan_state_patch(self) -> PlanStatePatch {
        match self {
            Self::NodeUpsert(p) => PlanStatePatch::NodeUpsert { node_id_hash: p.node_id_hash, title_hash: p.title_hash, description_hash: p.description_hash, status: p.status as u64, assignee_hash: p.assignee_hash, score_axes_hash: p.score_axes_hash, files_hash: p.files_hash },
            Self::EdgeAdd(p) => PlanStatePatch::EdgeAdd(PlanEdgeProjection { from_node_hash: p.from_node_hash, to_node_hash: p.to_node_hash }),
            Self::EdgeRemove(p) => PlanStatePatch::EdgeRemove(PlanEdgeProjection { from_node_hash: p.from_node_hash, to_node_hash: p.to_node_hash }),
            Self::NodeRemove(p) => PlanStatePatch::NodeRemove { node_id_hash: p.node_id_hash },
            Self::StatusChange(p) => PlanStatePatch::StatusChange { node_id_hash: p.node_id_hash, status: p.status as u64 },
            Self::AssigneeChange(p) => PlanStatePatch::AssigneeChange { node_id_hash: p.node_id_hash, assignee_hash: p.assignee_hash },
            Self::EvidenceAppend(p) => PlanStatePatch::EvidenceAppend { node_id_hash: p.node_id_hash, evidence: PlanEvidenceProjection { path_hash: p.path_hash, kind_hash: p.kind_hash, summary_hash: p.summary_hash } },
            Self::FullImport(p) => PlanStatePatch::FullImport { node_count: p.node_count, edge_count: p.edge_count, nodes_hash: p.nodes_hash, edges_hash: p.edges_hash },
        }
    }
}

impl PlanPatchRecord {
    pub fn new(source_hash: u64, cycle_id: u64, patch_seq: u64, payload: PlanPatchPayload) -> Self {
        let contract_hash = plan_patch_contract_hash();
        let payload_hash = payload.canonical_hash();
        let mut record = Self {
            schema_version: PLAN_PATCH_SCHEMA_VERSION,
            record_type: PLAN_PATCH_RECORD,
            source_hash,
            cycle_id,
            patch_seq,
            contract_hash,
            payload,
            payload_hash,
            patch_hash: 0,
        };
        record.patch_hash = record.expected_patch_hash();
        record
    }

    pub fn accepted(self, applied_revision: u64) -> AcceptedPlanPatchRecord {
        AcceptedPlanPatchRecord::from_patch(self, applied_revision)
    }

    pub fn rejected(self, reason_hash: u64) -> RejectedPlanPatchRecord {
        RejectedPlanPatchRecord::from_patch(self, reason_hash)
    }

    pub fn expected_patch_hash(self) -> u64 {
        fold_ordered_plan_hash(
            0x91a0_1001_5eed_0001u64,
            &[
                self.schema_version,
                self.record_type,
                self.source_hash,
                self.cycle_id,
                self.patch_seq,
                self.contract_hash,
                self.payload.kind() as u64,
                self.payload_hash,
            ],
        )
    }

    pub fn is_self_consistent(self) -> bool {
        self.schema_version == PLAN_PATCH_SCHEMA_VERSION
            && self.record_type == PLAN_PATCH_RECORD
            && self.source_hash != 0
            && self.cycle_id != 0
            && self.patch_seq != 0
            && self.contract_hash == plan_patch_contract_hash()
            && self.payload.is_valid()
            && self.payload_hash == self.payload.canonical_hash()
            && self.patch_hash == self.expected_patch_hash()
    }
}

impl AcceptedPlanPatchRecord {
    pub fn from_patch(patch: PlanPatchRecord, applied_revision: u64) -> Self {
        let mut record = Self {
            schema_version: PLAN_PATCH_SCHEMA_VERSION,
            record_type: PLAN_PATCH_ACCEPTED_RECORD,
            source_hash: patch.source_hash,
            cycle_id: patch.cycle_id,
            patch_seq: patch.patch_seq,
            contract_hash: patch.contract_hash,
            patch_hash: patch.patch_hash,
            applied_revision,
            acceptance_hash: 0,
        };
        record.acceptance_hash = record.expected_acceptance_hash();
        record
    }

    pub fn expected_acceptance_hash(self) -> u64 {
        fold_ordered_plan_hash(
            0x91a0_1002_5eed_0001u64,
            &[
                self.schema_version,
                self.record_type,
                self.source_hash,
                self.cycle_id,
                self.patch_seq,
                self.contract_hash,
                self.patch_hash,
                self.applied_revision,
            ],
        )
    }

    pub fn is_self_consistent(self) -> bool {
        self.schema_version == PLAN_PATCH_SCHEMA_VERSION
            && self.record_type == PLAN_PATCH_ACCEPTED_RECORD
            && self.source_hash != 0
            && self.cycle_id != 0
            && self.patch_seq != 0
            && self.contract_hash == plan_patch_contract_hash()
            && self.patch_hash != 0
            && self.applied_revision != 0
            && self.acceptance_hash == self.expected_acceptance_hash()
    }
}

impl RejectedPlanPatchRecord {
    pub fn from_patch(patch: PlanPatchRecord, reason_hash: u64) -> Self {
        let mut record = Self {
            schema_version: PLAN_PATCH_SCHEMA_VERSION,
            record_type: PLAN_PATCH_REJECTED_RECORD,
            source_hash: patch.source_hash,
            cycle_id: patch.cycle_id,
            patch_seq: patch.patch_seq,
            contract_hash: patch.contract_hash,
            patch_hash: patch.patch_hash,
            reason_hash,
            rejection_hash: 0,
        };
        record.rejection_hash = record.expected_rejection_hash();
        record
    }

    pub fn expected_rejection_hash(self) -> u64 {
        fold_ordered_plan_hash(
            0x91a0_1003_5eed_0001u64,
            &[
                self.schema_version,
                self.record_type,
                self.source_hash,
                self.cycle_id,
                self.patch_seq,
                self.contract_hash,
                self.patch_hash,
                self.reason_hash,
            ],
        )
    }

    pub fn is_self_consistent(self) -> bool {
        self.schema_version == PLAN_PATCH_SCHEMA_VERSION
            && self.record_type == PLAN_PATCH_REJECTED_RECORD
            && self.source_hash != 0
            && self.cycle_id != 0
            && self.patch_seq != 0
            && self.contract_hash == plan_patch_contract_hash()
            && self.patch_hash != 0
            && self.reason_hash != 0
            && self.rejection_hash == self.expected_rejection_hash()
    }
}

impl PlanRecord {
    pub fn from_packet(packet: Packet) -> Self {
        let objective_id = packet.objective_id;
        let required = packet.objective_required_tasks.clamp(1, MAX_PLAN_TASKS);
        let completed = packet.objective_done_tasks.min(required);
        let tasks = plan_tasks(objective_id, required, completed);
        let ready_tasks = tasks
            .iter()
            .filter(|task| task.ready)
            .count()
            .min(u8::MAX as usize) as u8;
        let task_id = tasks
            .iter()
            .find(|task| task.ready)
            .map(|task| task.task_id)
            .unwrap_or(0);
        let objective_hash = objective_hash(packet);
        let dependency_hash = dependency_hash(&tasks);
        let ready_set_hash = ready_set_hash(&tasks);
        let plan_revision = packet.revision.max(1);
        let lineage_hash = lineage_hash(
            objective_hash,
            dependency_hash,
            ready_set_hash,
            plan_revision,
            packet.objective_done_tasks,
        );

        Self {
            objective_id,
            task_id,
            ready_tasks,
            dependency_hash,
            objective_hash,
            plan_version: PLAN_SCHEMA_VERSION,
            plan_revision,
            task_count: tasks.len().min(u8::MAX as usize) as u8,
            completed_tasks: completed,
            ready_set_hash,
            lineage_hash,
            tasks,
        }
    }

    pub fn decision(&self) -> PlanDecision {
        if self.is_valid() && self.ready_tasks != 0 {
            PlanDecision::Ready
        } else {
            PlanDecision::Blocked
        }
    }

    pub fn is_valid(&self) -> bool {
        self.objective_id != 0
            && self.plan_version == PLAN_SCHEMA_VERSION
            && self.plan_revision != 0
            && self.task_count != 0
            && self.task_count as usize == self.tasks.len()
            && self.completed_tasks <= self.task_count
            && self.task_id != 0
            && self.ready_tasks != 0
            && self.dependency_hash == dependency_hash(&self.tasks)
            && self.ready_set_hash == ready_set_hash(&self.tasks)
            && self.objective_hash != 0
            && self.lineage_hash
                == lineage_hash(
                    self.objective_hash,
                    self.dependency_hash,
                    self.ready_set_hash,
                    self.plan_revision,
                    self.completed_tasks,
                )
            && self
                .tasks
                .iter()
                .copied()
                .all(|task| task.is_valid(self.objective_id))
            && self
                .tasks
                .windows(2)
                .all(|pair| pair[0].ordinal < pair[1].ordinal)
            && self.tasks.iter().filter(|task| task.ready).count() == usize::from(self.ready_tasks)
            && self
                .tasks
                .iter()
                .any(|task| task.ready && task.task_id == self.task_id)
    }

    pub fn submission(&self) -> EvidenceSubmission {
        let passed = self.decision() == PlanDecision::Ready;
        EvidenceSubmission::with_effect_payload(
            GateId::Plan,
            Evidence::TaskReady,
            passed,
            if passed {
                PacketEffect::BindReadyTask
            } else {
                PacketEffect::None
            },
            plan_payload_hash(self),
        )
    }

    pub fn receipt(&self) -> PlanReceipt {
        PlanReceipt::from_record(self)
    }
}

impl PlanReceipt {
    pub fn from_record(record: &PlanRecord) -> Self {
        let mut receipt = Self {
            schema_version: PLAN_RECEIPT_SCHEMA_VERSION,
            record_type: PLAN_RECEIPT_RECORD,
            objective_id: record.objective_id,
            task_id: record.task_id,
            task_count: u64::from(record.task_count),
            completed_tasks: u64::from(record.completed_tasks),
            ready_tasks: u64::from(record.ready_tasks),
            plan_version: record.plan_version,
            plan_revision: record.plan_revision,
            objective_hash: record.objective_hash,
            dependency_hash: record.dependency_hash,
            ready_set_hash: record.ready_set_hash,
            lineage_hash: record.lineage_hash,
            payload_hash: plan_payload_hash(record),
            verdict: record.decision(),
            receipt_hash: 0,
        };
        receipt.receipt_hash = receipt.expected_receipt_hash();
        receipt
    }

    pub fn is_valid_for(self, record: &PlanRecord) -> bool {
        self == PlanReceipt::from_record(record) && self.is_self_consistent()
    }

    pub fn is_self_consistent(self) -> bool {
        self.schema_version == PLAN_RECEIPT_SCHEMA_VERSION
            && self.record_type == PLAN_RECEIPT_RECORD
            && self.objective_id != 0
            && self.task_count != 0
            && self.completed_tasks <= self.task_count
            && self.plan_version == PLAN_SCHEMA_VERSION
            && self.plan_revision != 0
            && self.objective_hash != 0
            && self.dependency_hash != 0
            && self.ready_set_hash != 0
            && self.lineage_hash != 0
            && self.payload_hash != 0
            && self.receipt_hash == self.expected_receipt_hash()
            && match self.verdict {
                PlanDecision::Ready => self.task_id != 0 && self.ready_tasks != 0,
                PlanDecision::Blocked => self.ready_tasks == 0,
            }
    }

    pub fn submission(self) -> EvidenceSubmission {
        let passed = self.is_self_consistent() && self.verdict == PlanDecision::Ready;
        EvidenceSubmission::with_effect_payload(
            GateId::Plan,
            Evidence::TaskReady,
            passed,
            if passed {
                PacketEffect::BindReadyTask
            } else {
                PacketEffect::None
            },
            self.receipt_hash,
        )
    }

    pub fn expected_receipt_hash(self) -> u64 {
        let mut h = 0x91a0_0001_5eed_0001u64;
        h = mix(h, self.schema_version);
        h = mix(h, self.record_type);
        h = mix(h, self.objective_id);
        h = mix(h, self.task_id);
        h = mix(h, self.task_count);
        h = mix(h, self.completed_tasks);
        h = mix(h, self.ready_tasks);
        h = mix(h, self.plan_version);
        h = mix(h, self.plan_revision);
        h = mix(h, self.objective_hash);
        h = mix(h, self.dependency_hash);
        h = mix(h, self.ready_set_hash);
        h = mix(h, self.lineage_hash);
        h = mix(h, self.payload_hash);
        h = mix(h, self.verdict as u64);
        h.max(1)
    }
}

impl EvidenceProducer for PlanRecord {
    type Record = PlanRecord;

    fn record(&self) -> &Self::Record {
        self
    }

    fn submission(&self) -> EvidenceSubmission {
        PlanRecord::submission(self)
    }
}

fn plan_tasks(objective_id: u64, required: u8, completed: u8) -> Vec<PlanTask> {
    (0..required)
        .map(|idx| {
            let ordinal = idx.saturating_add(1);
            let task_id = objective_id
                .saturating_mul(100)
                .saturating_add(u64::from(ordinal));
            let depends_on = if ordinal == 1 {
                0
            } else {
                objective_id
                    .saturating_mul(100)
                    .saturating_add(u64::from(ordinal.saturating_sub(1)))
            };
            PlanTask {
                task_id,
                depends_on,
                ordinal,
                ready: idx == completed,
            }
        })
        .collect()
}

fn objective_hash(packet: Packet) -> u64 {
    fold_ordered_plan_hash(
        0x4f42_4a45_4354_0001u64,
        &[
            packet.objective_id,
            u64::from(packet.objective_required_tasks),
            packet.revision.max(1),
        ],
    )
}

fn dependency_hash(tasks: &[PlanTask]) -> u64 {
    let mut h = 0x4445_5053_504c_414eu64;
    for task in tasks {
        h = mix(h, task.task_id);
        h = mix(h, task.depends_on);
        h = mix(h, u64::from(task.ordinal));
    }
    h.max(1)
}

fn ready_set_hash(tasks: &[PlanTask]) -> u64 {
    let mut h = 0x5245_4144_595f_5345u64;
    for task in tasks.iter().filter(|task| task.ready) {
        h = mix(h, task.hash());
    }
    h.max(1)
}

fn lineage_hash(
    objective_hash: u64,
    dependency_hash: u64,
    ready_set_hash: u64,
    plan_revision: u64,
    completed_tasks: u8,
) -> u64 {
    fold_ordered_plan_hash(
        0x4c49_4e45_4147_4501u64,
        &[
            PLAN_SCHEMA_VERSION,
            objective_hash,
            dependency_hash,
            ready_set_hash,
            plan_revision,
            u64::from(completed_tasks),
        ],
    )
}

fn plan_payload_hash(record: &PlanRecord) -> u64 {
    fold_ordered_plan_hash(
        0x9e37_79b9_7f4a_7c15u64,
        &[
            record.objective_id,
            record.task_id,
            u64::from(record.ready_tasks),
            record.dependency_hash,
            record.objective_hash,
            record.ready_set_hash,
            record.lineage_hash,
        ],
    )
}

pub fn plan_patch_contract_hash() -> u64 {
    fold_ordered_plan_hash(
        0x504c_414e_5041_5443u64,
        &[
            PLAN_PATCH_SCHEMA_VERSION,
            PLAN_PATCH_RECORD,
            PLAN_PATCH_ACCEPTED_RECORD,
            PLAN_PATCH_REJECTED_RECORD,
            PlanPatchKind::NodeUpsert as u64,
            PlanPatchKind::EdgeAdd as u64,
            PlanPatchKind::EdgeRemove as u64,
            PlanPatchKind::NodeRemove as u64,
            PlanPatchKind::StatusChange as u64,
            PlanPatchKind::AssigneeChange as u64,
            PlanPatchKind::EvidenceAppend as u64,
            PlanPatchKind::FullImport as u64,
        ],
    )
}

fn fold_ordered_plan_hash(seed: u64, fields: &[u64]) -> u64 {
    fields.iter().fold(seed, |h, field| mix(h, *field)).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::Packet;

    fn packet(required: u8, completed: u8) -> Packet {
        let mut packet = Packet::empty();
        packet.objective_id = 42;
        packet.objective_required_tasks = required;
        packet.objective_done_tasks = completed;
        packet.revision = 9;
        packet
    }

    #[test]
    fn plan_receipt_binds_lineage_and_payload() {
        let record = PlanRecord::from_packet(packet(4, 1));
        let receipt = record.receipt();

        assert!(receipt.is_self_consistent());
        assert!(receipt.is_valid_for(&record));
        assert_eq!(receipt.verdict, PlanDecision::Ready);
        assert_eq!(receipt.task_id, 4202);
        assert_eq!(receipt.task_count, 4);
        assert_eq!(receipt.completed_tasks, 1);
        assert_eq!(receipt.ready_tasks, 1);
        assert_eq!(receipt.payload_hash, plan_payload_hash(&record));
        assert_eq!(receipt.submission().gate, GateId::Plan);
        assert!(receipt.submission().passed);
    }

    #[test]
    fn planning_hash_fold_helper_preserves_objective_lineage_and_payload_boundaries() {
        let record = PlanRecord::from_packet(packet(4, 1));
        let receipt = PlanReceipt::from_record(&record);
        let objective_hash = objective_hash(packet(4, 1));
        let lineage_hash_value = record.lineage_hash;
        let payload_hash = plan_payload_hash(&record);
        let receipt_hash = receipt.expected_receipt_hash();

        assert_eq!(record.objective_hash, objective_hash);
        assert_eq!(record.submission().payload_hash, payload_hash);
        assert_eq!(receipt.payload_hash, payload_hash);
        assert_eq!(receipt.receipt_hash, receipt_hash);

        assert_ne!(objective_hash, 0);
        assert_ne!(lineage_hash_value, 0);
        assert_ne!(payload_hash, 0);
        assert_ne!(receipt_hash, 0);
        assert_ne!(objective_hash, lineage_hash_value);
        assert_ne!(objective_hash, payload_hash);
        assert_ne!(objective_hash, receipt_hash);
        assert_ne!(lineage_hash_value, payload_hash);
        assert_ne!(lineage_hash_value, receipt_hash);
        assert_ne!(payload_hash, receipt_hash);

        let required_changed = PlanRecord::from_packet(packet(5, 1));
        assert_ne!(objective_hash, required_changed.objective_hash);
        assert_ne!(payload_hash, plan_payload_hash(&required_changed));

        let mut revision_packet = packet(4, 1);
        revision_packet.revision += 1;
        let revision_changed = PlanRecord::from_packet(revision_packet);
        assert_ne!(objective_hash, revision_changed.objective_hash);
        assert_ne!(payload_hash, plan_payload_hash(&revision_changed));

        let completed_changed = PlanRecord::from_packet(packet(4, 2));
        assert_eq!(objective_hash, completed_changed.objective_hash);
        assert_ne!(lineage_hash_value, completed_changed.lineage_hash);
        assert_ne!(payload_hash, plan_payload_hash(&completed_changed));

        let mut ready_task_id_changed = record.clone();
        ready_task_id_changed.task_id = record.tasks[2].task_id;
        assert_eq!(objective_hash, ready_task_id_changed.objective_hash);
        assert_ne!(payload_hash, plan_payload_hash(&ready_task_id_changed));

        let mut readiness_changed = record.clone();
        readiness_changed.tasks[0].ready = true;
        readiness_changed.ready_tasks = 2;
        readiness_changed.ready_set_hash = ready_set_hash(&readiness_changed.tasks);
        readiness_changed.lineage_hash = lineage_hash(
            readiness_changed.objective_hash,
            readiness_changed.dependency_hash,
            readiness_changed.ready_set_hash,
            readiness_changed.plan_revision,
            readiness_changed.completed_tasks,
        );
        assert_eq!(objective_hash, readiness_changed.objective_hash);
        assert_ne!(payload_hash, plan_payload_hash(&readiness_changed));
    }

    #[test]
    fn plan_receipt_rejects_tampered_lineage() {
        let record = PlanRecord::from_packet(packet(3, 0));
        let mut receipt = record.receipt();
        receipt.lineage_hash ^= 1;

        assert!(!receipt.is_self_consistent());
        assert!(!receipt.is_valid_for(&record));
        assert!(!receipt.submission().passed);
    }

    #[test]
    fn plan_receipt_blocks_completed_plan() {
        let record = PlanRecord::from_packet(packet(2, 2));
        let receipt = record.receipt();

        assert_eq!(receipt.verdict, PlanDecision::Blocked);
        assert!(receipt.is_self_consistent());
        assert!(receipt.is_valid_for(&record));
        assert!(!receipt.submission().passed);
    }

    #[test]
    fn plan_patch_records_cover_mutation_variants_with_stable_hashes() {
        let source_hash = fold_ordered_plan_hash(0x5352_4301, &[11]);
        let node_hash = fold_ordered_plan_hash(0x4e4f_4445, &[1]);
        let other_node_hash = fold_ordered_plan_hash(0x4e4f_4445, &[2]);
        let text_hash = fold_ordered_plan_hash(0x5445_5854, &[3]);

        let payloads = [
            PlanPatchPayload::NodeUpsert(PlanNodeUpsertPatch {
                node_id_hash: node_hash,
                title_hash: text_hash,
                description_hash: 0,
                status: PlanNodeStatus::Pending,
                assignee_hash: 0,
                score_axes_hash: 0,
                files_hash: 0,
            }),
            PlanPatchPayload::EdgeAdd(PlanEdgePatch {
                from_node_hash: node_hash,
                to_node_hash: other_node_hash,
            }),
            PlanPatchPayload::EdgeRemove(PlanEdgePatch {
                from_node_hash: node_hash,
                to_node_hash: other_node_hash,
            }),
            PlanPatchPayload::NodeRemove(PlanNodeRemovePatch {
                node_id_hash: node_hash,
            }),
            PlanPatchPayload::StatusChange(PlanStatusChangePatch {
                node_id_hash: node_hash,
                status: PlanNodeStatus::Done,
            }),
            PlanPatchPayload::AssigneeChange(PlanAssigneeChangePatch {
                node_id_hash: node_hash,
                assignee_hash: text_hash,
            }),
            PlanPatchPayload::EvidenceAppend(PlanEvidenceAppendPatch {
                node_id_hash: node_hash,
                path_hash: text_hash,
                kind_hash: text_hash,
                summary_hash: text_hash,
            }),
            PlanPatchPayload::FullImport(PlanFullImportPatch {
                node_count: 2,
                edge_count: 1,
                nodes_hash: node_hash,
                edges_hash: other_node_hash,
            }),
        ];

        for (idx, payload) in payloads.iter().copied().enumerate() {
            let patch = PlanPatchRecord::new(source_hash, 7, idx as u64 + 1, payload);
            assert!(patch.is_self_consistent());
            assert_eq!(patch.contract_hash, plan_patch_contract_hash());
            assert_eq!(patch.payload_hash, payload.canonical_hash());
            assert_eq!(patch.patch_hash, patch.expected_patch_hash());

            let accepted = patch.accepted(9);
            assert!(accepted.is_self_consistent());
            assert_eq!(accepted.patch_hash, patch.patch_hash);

            let rejected = patch.rejected(text_hash);
            assert!(rejected.is_self_consistent());
            assert_eq!(rejected.patch_hash, patch.patch_hash);
        }
    }

    #[test]
    fn plan_patch_records_reject_tampered_bindings() {
        let payload = PlanPatchPayload::StatusChange(PlanStatusChangePatch {
            node_id_hash: 99,
            status: PlanNodeStatus::Running,
        });
        let mut patch = PlanPatchRecord::new(11, 12, 13, payload);
        assert!(patch.is_self_consistent());

        patch.payload_hash ^= 1;
        assert!(!patch.is_self_consistent());

        let mut accepted = PlanPatchRecord::new(11, 12, 13, payload).accepted(1);
        accepted.contract_hash ^= 1;
        assert!(!accepted.is_self_consistent());

        let mut rejected = PlanPatchRecord::new(11, 12, 13, payload).rejected(14);
        rejected.reason_hash = 0;
        assert!(!rejected.is_self_consistent());
    }
}
