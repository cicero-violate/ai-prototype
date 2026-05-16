use super::mix;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Packet {
    pub objective_id: u64,
    pub objective_required_tasks: u8,
    pub objective_done_tasks: u8,
    pub ready_tasks: u8,
    pub active_task_id: u64,
    pub artifact_id: u64,
    pub parent_artifact_id: u64,
    pub artifact_bytes: u64,
    pub artifact_receipt_hash: u64,
    pub artifact_lineage_hash: u64,
    pub revision: u64,
}

impl Packet {
    pub const fn empty() -> Self {
        Self {
            objective_id: 1,
            objective_required_tasks: 1,
            objective_done_tasks: 0,
            ready_tasks: 0,
            active_task_id: 0,
            artifact_id: 0,
            parent_artifact_id: 0,
            artifact_bytes: 0,
            artifact_receipt_hash: 0,
            artifact_lineage_hash: 0,
            revision: 0,
        }
    }

    pub fn ready() -> Self {
        let mut packet = Self::empty();
        packet.bind_ready_task();
        packet.materialize_artifact();
        packet.repair_lineage();
        packet.complete_objective();
        packet
    }

    pub fn has_ready_task(self) -> bool {
        self.ready_tasks > 0 && self.active_task_id != 0
    }

    pub fn objective_complete(self) -> bool {
        self.objective_required_tasks > 0
            && self.objective_done_tasks >= self.objective_required_tasks
    }

    pub fn artifact_present(self) -> bool {
        self.artifact_id != 0 && self.artifact_bytes != 0 && self.artifact_receipt_hash != 0
    }

    pub fn artifact_receipt_valid(self) -> bool {
        self.artifact_id != 0
            && self.artifact_bytes != 0
            && self.artifact_receipt_hash == self.expected_receipt_hash()
    }

    pub fn lineage_valid(self) -> bool {
        self.artifact_receipt_valid() && self.artifact_lineage_hash == self.expected_lineage_hash()
    }

    pub fn bind_ready_task(&mut self) {
        self.bump_revision();
        self.ready_tasks = self.ready_tasks.max(1);
        if self.active_task_id == 0 {
            self.active_task_id = self.objective_id.saturating_mul(100).saturating_add(1);
        }
    }

    pub fn materialize_artifact(&mut self) {
        self.bump_revision();
        if self.active_task_id == 0 {
            self.bind_ready_task();
        }
        self.parent_artifact_id = self.artifact_id;
        self.artifact_id = self
            .objective_id
            .saturating_mul(10_000)
            .saturating_add(self.active_task_id)
            .saturating_add(self.revision);
        self.artifact_bytes = self.artifact_id.saturating_mul(3).saturating_add(17);
        self.artifact_receipt_hash = self.expected_receipt_hash();
        self.ready_tasks = self.ready_tasks.saturating_sub(1);
        self.repair_lineage();
    }

    pub fn repair_lineage(&mut self) {
        if self.artifact_receipt_valid() {
            self.artifact_lineage_hash = self.expected_lineage_hash();
        }
    }

    pub fn complete_objective(&mut self) {
        if self.lineage_valid() {
            self.objective_done_tasks = self.objective_required_tasks;
        }
    }

    pub fn expected_receipt_hash(self) -> u64 {
        let mut h = self.artifact_hash_base(0x243f6a8885a308d3u64);
        h = mix(h, self.revision);
        h
    }

    pub fn expected_lineage_hash(self) -> u64 {
        let mut h = self.artifact_hash_base(0x9e3779b97f4a7c15u64);
        h = mix(h, self.artifact_receipt_hash);
        h = mix(h, self.revision);
        h
    }

    fn artifact_hash_base(self, seed: u64) -> u64 {
        let mut h = seed;
        h = mix(h, self.objective_id);
        h = mix(h, self.active_task_id);
        h = mix(h, self.parent_artifact_id);
        h = mix(h, self.artifact_id);
        mix(h, self.artifact_bytes)
    }

    fn bump_revision(&mut self) {
        self.revision = self.revision.saturating_add(1);
    }

    pub fn is_structurally_valid(self) -> bool {
        if self.objective_id == 0 || self.objective_required_tasks == 0 {
            return false;
        }

        if self.objective_done_tasks > self.objective_required_tasks {
            return false;
        }

        if self.ready_tasks > 0 && self.active_task_id == 0 {
            return false;
        }

        if self.artifact_id == 0 {
            self.parent_artifact_id == 0
                && self.artifact_bytes == 0
                && self.artifact_receipt_hash == 0
                && self.artifact_lineage_hash == 0
        } else {
            self.artifact_bytes != 0 && self.artifact_receipt_hash != 0
        }
    }
}
