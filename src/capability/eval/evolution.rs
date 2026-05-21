//! Verified evolution candidate ledger owned by the eval capability.
//!
//! The LLM may propose candidates, but this ledger only accepts externally
//! evaluated candidates: sandbox execution, evaluator verdict, replay validity,
//! and proof binding must all be present before a candidate can be selected.

use crate::kernel::mix;

pub const EVOLUTION_LEDGER_SCHEMA_VERSION: u64 = 1;
pub const EVOLUTION_LEDGER_RECORD: u64 = 0x4556_4f4c_5645_3031;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidateVerdict {
    Pass,
    Fail,
}

impl CandidateVerdict {
    pub const fn as_u64(self) -> u64 {
        match self {
            Self::Pass => 1,
            Self::Fail => 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateReceipt {
    pub schema_version: u64,
    pub generation: u64,
    pub candidate_id: u64,
    pub parent_id: u64,
    pub seed_hash: u64,
    pub patch_hash: u64,
    pub sandbox_hash: u64,
    pub evaluator_hash: u64,
    pub score: u64,
    pub threshold: u64,
    pub verdict: CandidateVerdict,
    pub replay_valid: bool,
    pub proof_hash: u64,
    pub lineage_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionRecord {
    pub schema_version: u64,
    pub generation: u64,
    pub selected_candidate_id: u64,
    pub selected_score: u64,
    pub selected_proof_hash: u64,
    pub candidate_count: u64,
    pub pass_count: u64,
    pub selection_hash: u64,
}

impl CandidateReceipt {
    pub fn new(input: CandidateReceiptInput) -> Option<Self> {
        let mut receipt = Self {
            schema_version: EVOLUTION_LEDGER_SCHEMA_VERSION,
            generation: input.generation,
            candidate_id: input.candidate_id,
            parent_id: input.parent_id,
            seed_hash: input.seed_hash,
            patch_hash: input.patch_hash,
            sandbox_hash: input.sandbox_hash,
            evaluator_hash: input.evaluator_hash,
            score: input.score,
            threshold: input.threshold,
            verdict: input.verdict,
            replay_valid: input.replay_valid,
            proof_hash: input.proof_hash,
            lineage_hash: 0,
        };
        receipt.lineage_hash = receipt.expected_lineage_hash()?;
        receipt.is_structurally_valid().then_some(receipt)
    }

    pub fn passed(&self) -> bool {
        self.verdict == CandidateVerdict::Pass
            && self.replay_valid
            && self.score >= self.threshold
            && self.proof_hash != 0
            && self.lineage_hash == self.expected_lineage_hash().unwrap_or(0)
    }

    pub fn is_structurally_valid(&self) -> bool {
        self.schema_version == EVOLUTION_LEDGER_SCHEMA_VERSION
            && self.generation != 0
            && self.candidate_id != 0
            && self.seed_hash != 0
            && self.patch_hash != 0
            && self.sandbox_hash != 0
            && self.evaluator_hash != 0
            && self.threshold != 0
            && self.proof_hash != 0
            && self.lineage_hash != 0
            && self.lineage_hash == self.expected_lineage_hash().unwrap_or(0)
    }

    pub fn expected_lineage_hash(&self) -> Option<u64> {
        if self.schema_version != EVOLUTION_LEDGER_SCHEMA_VERSION
            || self.generation == 0
            || self.candidate_id == 0
            || self.seed_hash == 0
            || self.patch_hash == 0
            || self.sandbox_hash == 0
            || self.evaluator_hash == 0
            || self.threshold == 0
            || self.proof_hash == 0
        {
            return None;
        }

        let mut h = 0x4341_4e44_4c45_4431u64;
        h = mix(h, self.schema_version);
        h = mix(h, self.generation);
        h = mix(h, self.candidate_id);
        h = mix(h, self.parent_id);
        h = mix(h, self.seed_hash);
        h = mix(h, self.patch_hash);
        h = mix(h, self.sandbox_hash);
        h = mix(h, self.evaluator_hash);
        h = mix(h, self.score);
        h = mix(h, self.threshold);
        h = mix(h, self.verdict.as_u64());
        h = mix(h, u64::from(self.replay_valid));
        h = mix(h, self.proof_hash);
        Some(h.max(1))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CandidateReceiptInput {
    pub generation: u64,
    pub candidate_id: u64,
    pub parent_id: u64,
    pub seed_hash: u64,
    pub patch_hash: u64,
    pub sandbox_hash: u64,
    pub evaluator_hash: u64,
    pub score: u64,
    pub threshold: u64,
    pub verdict: CandidateVerdict,
    pub replay_valid: bool,
    pub proof_hash: u64,
}

impl SelectionRecord {
    pub fn select_winner(candidates: &[CandidateReceipt]) -> Option<Self> {
        let first = candidates.first()?;
        let generation = first.generation;
        if candidates.iter().any(|candidate| {
            candidate.generation != generation || !candidate.is_structurally_valid()
        }) {
            return None;
        }

        let winner = best_candidate(candidates, generation)?;

        let pass_count = candidates
            .iter()
            .filter(|candidate| candidate.passed())
            .count() as u64;
        let mut record = Self {
            schema_version: EVOLUTION_LEDGER_SCHEMA_VERSION,
            generation,
            selected_candidate_id: winner.candidate_id,
            selected_score: winner.score,
            selected_proof_hash: winner.proof_hash,
            candidate_count: candidates.len() as u64,
            pass_count,
            selection_hash: 0,
        };
        record.selection_hash = record.expected_selection_hash(candidates)?;
        record.is_valid_for(candidates).then_some(record)
    }

    pub fn is_valid_for(&self, candidates: &[CandidateReceipt]) -> bool {
        if self.schema_version != EVOLUTION_LEDGER_SCHEMA_VERSION
            || self.generation == 0
            || self.selected_candidate_id == 0
            || self.selected_score == 0
            || self.selected_proof_hash == 0
            || self.candidate_count != candidates.len() as u64
            || self.pass_count == 0
            || self.selection_hash == 0
        {
            return false;
        }

        let Some(winner) = best_candidate(candidates, self.generation) else {
            return false;
        };
        let pass_count = candidates
            .iter()
            .filter(|candidate| candidate.passed())
            .count() as u64;

        self.selected_candidate_id == winner.candidate_id
            && self.selected_score == winner.score
            && self.selected_proof_hash == winner.proof_hash
            && self.pass_count == pass_count
            && self.selection_hash == self.expected_selection_hash(candidates).unwrap_or(0)
    }

    pub fn expected_selection_hash(&self, candidates: &[CandidateReceipt]) -> Option<u64> {
        if candidates.is_empty() || self.schema_version != EVOLUTION_LEDGER_SCHEMA_VERSION {
            return None;
        }
        let mut h = 0x5345_4c45_4354_3031u64;
        h = mix(h, self.schema_version);
        h = mix(h, self.generation);
        h = mix(h, self.selected_candidate_id);
        h = mix(h, self.selected_score);
        h = mix(h, self.selected_proof_hash);
        h = mix(h, self.candidate_count);
        h = mix(h, self.pass_count);
        for candidate in candidates {
            h = mix(h, candidate.lineage_hash);
        }
        Some(h.max(1))
    }
}

fn best_candidate(candidates: &[CandidateReceipt], generation: u64) -> Option<&CandidateReceipt> {
    if candidates
        .iter()
        .any(|candidate| candidate.generation != generation || !candidate.is_structurally_valid())
    {
        return None;
    }

    candidates
        .iter()
        .filter(|candidate| candidate.passed())
        .fold(
            None,
            |best: Option<&CandidateReceipt>, candidate| match best {
                None => Some(candidate),
                Some(current) => {
                    if candidate.score > current.score
                        || (candidate.score == current.score
                            && candidate.lineage_hash < current.lineage_hash)
                        || (candidate.score == current.score
                            && candidate.lineage_hash == current.lineage_hash
                            && candidate.candidate_id < current.candidate_id)
                    {
                        Some(candidate)
                    } else {
                        Some(current)
                    }
                }
            },
        )
}

pub fn encode_candidate_receipt_ndjson(receipt: &CandidateReceipt) -> Option<String> {
    receipt.is_structurally_valid().then(|| {
        format!(
            "[{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}]\n",
            receipt.schema_version,
            EVOLUTION_LEDGER_RECORD,
            receipt.generation,
            receipt.candidate_id,
            receipt.parent_id,
            receipt.seed_hash,
            receipt.patch_hash,
            receipt.sandbox_hash,
            receipt.evaluator_hash,
            receipt.score,
            receipt.threshold,
            receipt.verdict.as_u64(),
            u64::from(receipt.replay_valid),
            receipt.proof_hash,
            receipt.lineage_hash,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(
        id: u64,
        score: u64,
        verdict: CandidateVerdict,
        replay_valid: bool,
    ) -> CandidateReceipt {
        CandidateReceipt::new(CandidateReceiptInput {
            generation: 1,
            candidate_id: id,
            parent_id: 99,
            seed_hash: 1000,
            patch_hash: 2000 + id,
            sandbox_hash: 3000 + id,
            evaluator_hash: 4000,
            score,
            threshold: 80,
            verdict,
            replay_valid,
            proof_hash: 5000 + id,
        })
        .expect("test value should be present")
    }

    #[test]
    fn passing_candidate_binds_external_evidence() {
        let receipt = candidate(1, 91, CandidateVerdict::Pass, true);
        assert!(receipt.passed());
        assert!(receipt.is_structurally_valid());
        assert_eq!(
            receipt.lineage_hash,
            receipt
                .expected_lineage_hash()
                .expect("test value should be present")
        );
    }

    #[test]
    fn failed_or_unreplayable_candidates_cannot_pass() {
        assert!(!candidate(1, 91, CandidateVerdict::Fail, true).passed());
        assert!(!candidate(2, 91, CandidateVerdict::Pass, false).passed());
        assert!(!candidate(3, 79, CandidateVerdict::Pass, true).passed());
    }

    #[test]
    fn selection_keeps_highest_verified_score() {
        let candidates = vec![
            candidate(1, 91, CandidateVerdict::Pass, true),
            candidate(2, 95, CandidateVerdict::Pass, true),
            candidate(3, 99, CandidateVerdict::Fail, true),
        ];
        let selected =
            SelectionRecord::select_winner(&candidates).expect("test setup should succeed");
        assert_eq!(selected.selected_candidate_id, 2);
        assert_eq!(selected.selected_score, 95);
        assert_eq!(selected.pass_count, 2);
        assert!(selected.is_valid_for(&candidates));
    }

    #[test]
    fn selection_is_deterministic_on_score_ties() {
        let a = candidate(1, 91, CandidateVerdict::Pass, true);
        let b = candidate(2, 91, CandidateVerdict::Pass, true);
        let candidates = vec![a.clone(), b.clone()];
        let selected =
            SelectionRecord::select_winner(&candidates).expect("test setup should succeed");
        let expected = if a.lineage_hash < b.lineage_hash {
            1
        } else {
            2
        };
        assert_eq!(selected.selected_candidate_id, expected);
    }

    #[test]
    fn tampered_candidate_rejects_selection() {
        let mut candidates = vec![candidate(1, 91, CandidateVerdict::Pass, true)];
        candidates[0].score = 100;
        assert!(SelectionRecord::select_winner(&candidates).is_none());
    }

    #[test]
    fn ndjson_encoder_rejects_invalid_receipt() {
        let mut receipt = candidate(1, 91, CandidateVerdict::Pass, true);
        assert!(encode_candidate_receipt_ndjson(&receipt)
            .expect("test value should be present")
            .starts_with("[1,"));
        receipt.proof_hash ^= 1;
        assert!(encode_candidate_receipt_ndjson(&receipt).is_none());
    }
}
