//! Task lifecycle receipts owned by orchestration.
//!
//! The scheduler/dispatch boundary may change transport, but durable task
//! ownership is represented as typed execution receipts.

use crate::capability::{EvidenceSubmission, PacketEffect};
use crate::kernel::{mix, Evidence, GateId};

const TASK_LIFECYCLE_SCHEMA_VERSION: u64 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TaskLifecycleKind {
    Claimed = 1,
    Heartbeated = 2,
    Completed = 3,
    Failed = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskLifecycleReceipt {
    pub schema_version: u64,
    pub kind: TaskLifecycleKind,
    pub node_id_hash: u64,
    pub worker_id_hash: u64,
    pub claim_id: u64,
    pub idempotency_key: u64,
    pub lease_expires_at_ms: u64,
    pub event_at_ms: u64,
    pub evidence_hash: u64,
    pub retry_after_ms: u64,
    pub receipt_hash: u64,
}

impl TaskLifecycleReceipt {
    pub fn claim(
        node_id: &str,
        worker_id: &str,
        claim_id: u64,
        idempotency_key: u64,
        lease_expires_at_ms: u64,
        event_at_ms: u64,
    ) -> Self {
        Self::new(
            TaskLifecycleKind::Claimed,
            node_id,
            worker_id,
            claim_id,
            idempotency_key,
            lease_expires_at_ms,
            event_at_ms,
            0,
            0,
        )
    }

    pub fn heartbeat(
        node_id: &str,
        worker_id: &str,
        claim_id: u64,
        lease_expires_at_ms: u64,
        event_at_ms: u64,
    ) -> Self {
        Self::new(
            TaskLifecycleKind::Heartbeated,
            node_id,
            worker_id,
            claim_id,
            0,
            lease_expires_at_ms,
            event_at_ms,
            0,
            0,
        )
    }

    pub fn complete(
        node_id: &str,
        worker_id: &str,
        claim_id: u64,
        lease_expires_at_ms: u64,
        event_at_ms: u64,
        evidence_hash: u64,
    ) -> Self {
        Self::new(
            TaskLifecycleKind::Completed,
            node_id,
            worker_id,
            claim_id,
            0,
            lease_expires_at_ms,
            event_at_ms,
            evidence_hash,
            0,
        )
    }

    pub fn fail(
        node_id: &str,
        worker_id: &str,
        claim_id: u64,
        event_at_ms: u64,
        retry_after_ms: u64,
    ) -> Self {
        Self::new(
            TaskLifecycleKind::Failed,
            node_id,
            worker_id,
            claim_id,
            0,
            0,
            event_at_ms,
            0,
            retry_after_ms,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new(
        kind: TaskLifecycleKind,
        node_id: &str,
        worker_id: &str,
        claim_id: u64,
        idempotency_key: u64,
        lease_expires_at_ms: u64,
        event_at_ms: u64,
        evidence_hash: u64,
        retry_after_ms: u64,
    ) -> Self {
        let node_id_hash = hash_text(node_id);
        let worker_id_hash = hash_text(worker_id);
        let receipt_hash = expected_receipt_hash(
            kind,
            node_id_hash,
            worker_id_hash,
            claim_id,
            idempotency_key,
            lease_expires_at_ms,
            event_at_ms,
            evidence_hash,
            retry_after_ms,
        );
        Self {
            schema_version: TASK_LIFECYCLE_SCHEMA_VERSION,
            kind,
            node_id_hash,
            worker_id_hash,
            claim_id,
            idempotency_key,
            lease_expires_at_ms,
            event_at_ms,
            evidence_hash,
            retry_after_ms,
            receipt_hash,
        }
    }

    pub fn is_contract_valid(self) -> bool {
        self.schema_version == TASK_LIFECYCLE_SCHEMA_VERSION
            && self.node_id_hash != 0
            && self.worker_id_hash != 0
            && self.claim_id != 0
            && self.event_at_ms != 0
            && self.receipt_hash != 0
            && self.receipt_hash
                == expected_receipt_hash(
                    self.kind,
                    self.node_id_hash,
                    self.worker_id_hash,
                    self.claim_id,
                    self.idempotency_key,
                    self.lease_expires_at_ms,
                    self.event_at_ms,
                    self.evidence_hash,
                    self.retry_after_ms,
                )
            && match self.kind {
                TaskLifecycleKind::Claimed => {
                    self.lease_expires_at_ms != 0
                        && self.evidence_hash == 0
                        && self.retry_after_ms == 0
                }
                TaskLifecycleKind::Heartbeated => {
                    self.idempotency_key == 0
                        && self.lease_expires_at_ms != 0
                        && self.evidence_hash == 0
                        && self.retry_after_ms == 0
                }
                TaskLifecycleKind::Completed => {
                    self.idempotency_key == 0
                        && self.lease_expires_at_ms != 0
                        && self.evidence_hash != 0
                        && self.retry_after_ms == 0
                }
                TaskLifecycleKind::Failed => {
                    self.idempotency_key == 0
                        && self.lease_expires_at_ms == 0
                        && self.evidence_hash == 0
                }
            }
    }

    pub fn submission(self) -> EvidenceSubmission {
        EvidenceSubmission {
            gate: GateId::Execution,
            evidence: Evidence::ExecutionReceipt,
            passed: self.is_contract_valid(),
            payload_hash: self.receipt_hash,
            effect: PacketEffect::None,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn expected_receipt_hash(
    kind: TaskLifecycleKind,
    node_id_hash: u64,
    worker_id_hash: u64,
    claim_id: u64,
    idempotency_key: u64,
    lease_expires_at_ms: u64,
    event_at_ms: u64,
    evidence_hash: u64,
    retry_after_ms: u64,
) -> u64 {
    let mut h = 0x24cf_e71d_81b5_7789u64;
    h = mix(h, TASK_LIFECYCLE_SCHEMA_VERSION);
    h = mix(h, kind as u64);
    h = mix(h, node_id_hash);
    h = mix(h, worker_id_hash);
    h = mix(h, claim_id);
    h = mix(h, idempotency_key);
    h = mix(h, lease_expires_at_ms);
    h = mix(h, event_at_ms);
    h = mix(h, evidence_hash);
    h = mix(h, retry_after_ms);
    h.max(1)
}

fn hash_text(value: &str) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in value.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{CapabilityId, CapabilityRegistry};

    #[test]
    fn claim_receipt_is_valid_execution_receipt() {
        let receipt = TaskLifecycleReceipt::claim("node-1", "worker-a", 7, 99, 50_000, 10_000);

        assert!(receipt.is_contract_valid());
        let submission = receipt.submission();
        assert!(submission.is_contract_valid());
        assert!(CapabilityRegistry::canonical().allows(CapabilityId::Orchestration, submission));
    }

    #[test]
    fn complete_receipt_requires_bound_evidence_hash() {
        let valid = TaskLifecycleReceipt::complete("node-1", "worker-a", 7, 50_000, 10_000, 42);
        let invalid = TaskLifecycleReceipt::complete("node-1", "worker-a", 7, 50_000, 10_000, 0);

        assert!(valid.is_contract_valid());
        assert!(!invalid.is_contract_valid());
    }

    #[test]
    fn receipt_hash_detects_tampering() {
        let mut receipt = TaskLifecycleReceipt::heartbeat("node-1", "worker-a", 7, 50_000, 10_000);
        receipt.claim_id = 8;

        assert!(!receipt.is_contract_valid());
    }
}
