//! Judgment payload owned outside the kernel.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgmentRecord {
    pub decision_id: u64,
    pub policy_version: u64,
    pub rationale_hash: u64,
}

// TODO: grow this payload once judgment policy becomes versioned.
