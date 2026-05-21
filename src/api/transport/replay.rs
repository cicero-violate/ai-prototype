#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureClass {
    RuntimeEvidenceGap,
    ArtifactMissing,
    ArtifactScanTimeout,
    ManifestInvalid,
    HashMismatch,
    MergeConflict,
    ValidationTimeout,
    ValidationFailure,
    DirtyWorktree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    StopNoMerge,
    KeepHistoryOnly,
    RefuseUntilClean,
    ValidateThenReport,
}

pub fn recovery_action(class: FailureClass) -> RecoveryAction {
    match class {
        FailureClass::RuntimeEvidenceGap => RecoveryAction::KeepHistoryOnly,
        FailureClass::ArtifactMissing
        | FailureClass::ArtifactScanTimeout
        | FailureClass::ManifestInvalid
        | FailureClass::HashMismatch
        | FailureClass::MergeConflict => RecoveryAction::StopNoMerge,
        FailureClass::DirtyWorktree => RecoveryAction::RefuseUntilClean,
        FailureClass::ValidationTimeout | FailureClass::ValidationFailure => {
            RecoveryAction::ValidateThenReport
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReceipt {
    pub classification: &'static str,
    pub exit_code: Option<i32>,
    pub stdout_snippet: String,
    pub stderr_snippet: String,
}

impl ValidationReceipt {
    pub fn new(
        classification: &'static str,
        exit_code: Option<i32>,
        stdout: &str,
        stderr: &str,
    ) -> Self {
        Self {
            classification,
            exit_code,
            stdout_snippet: bounded_snippet(stdout, 256),
            stderr_snippet: bounded_snippet(stderr, 256),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiptReplayFailure {
    MissingReceipt,
    StaleReceipt,
    DuplicatedReceipt,
    ReorderedReceipt,
    ForgedReceipt,
}

impl ReceiptReplayFailure {
    pub fn classification(self) -> &'static str {
        match self {
            ReceiptReplayFailure::MissingReceipt => "missing_receipt",
            ReceiptReplayFailure::StaleReceipt => "stale_receipt",
            ReceiptReplayFailure::DuplicatedReceipt => "duplicated_receipt",
            ReceiptReplayFailure::ReorderedReceipt => "reordered_receipt",
            ReceiptReplayFailure::ForgedReceipt => "forged_receipt",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReceiptChainEntry {
    pub seq: u64,
    pub run_id: u64,
    pub payload_hash: u64,
    pub prev_hash: u64,
    pub receipt_hash: u64,
}

impl ReceiptChainEntry {
    pub fn new(seq: u64, run_id: u64, payload_hash: u64, prev_hash: u64) -> Self {
        let receipt_hash = receipt_chain_hash(seq, run_id, payload_hash, prev_hash);
        Self {
            seq,
            run_id,
            payload_hash,
            prev_hash,
            receipt_hash,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReceiptReplayReport {
    pub run_id: u64,
    pub receipt_count: usize,
    pub first_seq: u64,
    pub last_seq: u64,
    pub final_hash: u64,
}

pub fn verify_receipt_chain(
    expected_run_id: u64,
    expected_last_seq: u64,
    receipts: &[ReceiptChainEntry],
) -> Result<ReceiptReplayReport, ReceiptReplayFailure> {
    if expected_last_seq == 0 {
        return verify_empty_receipt_chain(expected_run_id, receipts);
    }

    if receipts.is_empty() {
        return Err(ReceiptReplayFailure::MissingReceipt);
    }

    let mut prev_hash = 0;
    let mut prev_seq = 0;

    for (index, receipt) in receipts.iter().enumerate() {
        if receipt.run_id != expected_run_id || receipt.receipt_hash != receipt.expected_hash() {
            return Err(ReceiptReplayFailure::ForgedReceipt);
        }

        if receipt.seq == prev_seq || receipt.prev_hash == receipt.receipt_hash {
            return Err(ReceiptReplayFailure::DuplicatedReceipt);
        }

        if receipt.seq < prev_seq {
            return Err(ReceiptReplayFailure::ReorderedReceipt);
        }

        if receipt.seq > prev_seq + 1
            && receipts[index + 1..]
                .iter()
                .any(|future| future.seq == prev_seq + 1)
        {
            return Err(ReceiptReplayFailure::ReorderedReceipt);
        }

        if receipt.seq != prev_seq + 1 || receipt.prev_hash != prev_hash {
            return Err(ReceiptReplayFailure::MissingReceipt);
        }

        prev_seq = receipt.seq;
        prev_hash = receipt.receipt_hash;
    }

    if prev_seq < expected_last_seq {
        return Err(ReceiptReplayFailure::MissingReceipt);
    }

    if prev_seq > expected_last_seq {
        return Err(ReceiptReplayFailure::StaleReceipt);
    }

    Ok(ReceiptReplayReport {
        run_id: expected_run_id,
        receipt_count: receipts.len(),
        first_seq: receipts.first().map(|receipt| receipt.seq).unwrap_or(0),
        last_seq: prev_seq,
        final_hash: prev_hash,
    })
}

fn verify_empty_receipt_chain(
    expected_run_id: u64,
    receipts: &[ReceiptChainEntry],
) -> Result<ReceiptReplayReport, ReceiptReplayFailure> {
    if !receipts.is_empty() {
        return Err(ReceiptReplayFailure::StaleReceipt);
    }

    Ok(ReceiptReplayReport {
        run_id: expected_run_id,
        receipt_count: 0,
        first_seq: 0,
        last_seq: 0,
        final_hash: 0,
    })
}

impl ReceiptChainEntry {
    fn expected_hash(self) -> u64 {
        receipt_chain_hash(self.seq, self.run_id, self.payload_hash, self.prev_hash)
    }
}

fn receipt_chain_hash(seq: u64, run_id: u64, payload_hash: u64, prev_hash: u64) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;
    hash = mix_receipt_hash(hash, seq);
    hash = mix_receipt_hash(hash, run_id);
    hash = mix_receipt_hash(hash, payload_hash);
    hash = mix_receipt_hash(hash, prev_hash);
    if hash == 0 {
        1
    } else {
        hash
    }
}

fn mix_receipt_hash(hash: u64, value: u64) -> u64 {
    let mut out = hash ^ value;
    out = out.wrapping_mul(0x1000_0000_01b3);
    out.rotate_left(13) ^ (out >> 7)
}

fn bounded_snippet(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_owned();
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chain(count: u64) -> Vec<ReceiptChainEntry> {
        let mut receipts = Vec::new();
        let mut prev_hash = 0;
        for seq in 1..=count {
            let receipt = ReceiptChainEntry::new(seq, 77, seq * 10, prev_hash);
            prev_hash = receipt.receipt_hash;
            receipts.push(receipt);
        }
        receipts
    }

    #[test]
    fn receipt_chain_accepts_valid_replay() {
        let receipts = chain(3);
        let report = verify_receipt_chain(77, 3, &receipts).expect("test setup should succeed");

        assert_eq!(report.run_id, 77);
        assert_eq!(report.receipt_count, 3);
        assert_eq!(report.first_seq, 1);
        assert_eq!(report.last_seq, 3);
        assert_eq!(report.final_hash, receipts[2].receipt_hash);
    }

    #[test]
    fn receipt_chain_rejects_forged_receipt_hash() {
        let mut receipts = chain(3);
        receipts[1].receipt_hash = receipts[1].receipt_hash.wrapping_add(1);

        assert_eq!(
            verify_receipt_chain(77, 3, &receipts),
            Err(ReceiptReplayFailure::ForgedReceipt)
        );
    }

    #[test]
    fn receipt_chain_rejects_forged_run_identity() {
        let receipts = chain(3);

        assert_eq!(
            verify_receipt_chain(88, 3, &receipts),
            Err(ReceiptReplayFailure::ForgedReceipt)
        );
    }

    #[test]
    fn receipt_chain_rejects_duplicated_receipt() {
        let mut receipts = chain(3);
        receipts[2] = receipts[1];

        assert_eq!(
            verify_receipt_chain(77, 3, &receipts),
            Err(ReceiptReplayFailure::DuplicatedReceipt)
        );
    }

    #[test]
    fn receipt_chain_rejects_reordered_receipts() {
        let mut receipts = chain(3);
        receipts.swap(1, 2);

        assert_eq!(
            verify_receipt_chain(77, 3, &receipts),
            Err(ReceiptReplayFailure::ReorderedReceipt)
        );
    }

    #[test]
    fn receipt_chain_rejects_stale_extra_receipts() {
        let receipts = chain(3);

        assert_eq!(
            verify_receipt_chain(77, 2, &receipts),
            Err(ReceiptReplayFailure::StaleReceipt)
        );
    }

    #[test]
    fn receipt_chain_rejects_missing_receipts() {
        let receipts = chain(2);

        assert_eq!(
            verify_receipt_chain(77, 3, &receipts),
            Err(ReceiptReplayFailure::MissingReceipt)
        );
    }

    #[test]
    fn receipt_replay_failure_exposes_compact_classification() {
        assert_eq!(
            ReceiptReplayFailure::ForgedReceipt.classification(),
            "forged_receipt"
        );
        assert_eq!(
            ReceiptReplayFailure::DuplicatedReceipt.classification(),
            "duplicated_receipt"
        );
        assert_eq!(
            ReceiptReplayFailure::ReorderedReceipt.classification(),
            "reordered_receipt"
        );
        assert_eq!(
            ReceiptReplayFailure::StaleReceipt.classification(),
            "stale_receipt"
        );
        assert_eq!(
            ReceiptReplayFailure::MissingReceipt.classification(),
            "missing_receipt"
        );
    }
}
