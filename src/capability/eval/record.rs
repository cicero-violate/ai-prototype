//! Durable eval payload owned by the eval capability.

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::capability::{EvidenceProducer, EvidenceSubmission, PacketEffect};
use crate::error::CanonError;
use crate::kernel::{mix, Evidence, GateId};

pub const EVAL_SCORECARD_SCHEMA_VERSION: u64 = 1;
pub const EVAL_SCORECARD_RECORD: u64 = 0x0e7a_5001;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvalDimension {
    pub id: &'static str,
    pub score: u64,
    pub threshold: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalRecord {
    pub score: u64,
    pub dimensions: Vec<EvalDimension>,
    pub threshold_used: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvalDecision {
    Pass,
    Fail,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvalScorecardReceipt {
    pub schema_version: u64,
    pub record_type: u64,
    pub score: u64,
    pub threshold_used: u64,
    pub dimension_count: u64,
    pub min_dimension_score: u64,
    pub max_dimension_score: u64,
    pub dimension_threshold_floor: u64,
    pub dimension_order_hash: u64,
    pub dimension_score_hash: u64,
    pub payload_hash: u64,
    pub verdict: EvalDecision,
    pub receipt_hash: u64,
}

impl EvalRecord {
    pub fn decision(&self) -> EvalDecision {
        if self.score >= self.threshold_used
            && !self.dimensions.is_empty()
            && self
                .dimensions
                .iter()
                .all(|dimension| dimension.score >= dimension.threshold)
        {
            EvalDecision::Pass
        } else {
            EvalDecision::Fail
        }
    }

    pub fn scorecard_receipt(&self) -> EvalScorecardReceipt {
        EvalScorecardReceipt::from_record(self)
    }

    pub fn submission(&self) -> EvidenceSubmission {
        let passed = self.decision() == EvalDecision::Pass;
        EvidenceSubmission::with_effect_payload(
            GateId::Eval,
            Evidence::EvalScore,
            passed,
            if passed {
                PacketEffect::CompleteObjective
            } else {
                PacketEffect::None
            },
            eval_payload_hash(self),
        )
    }
}

impl EvalScorecardReceipt {
    pub fn from_record(record: &EvalRecord) -> Self {
        let dimension_count = record.dimensions.len() as u64;
        let min_dimension_score = record
            .dimensions
            .iter()
            .map(|dimension| dimension.score)
            .min()
            .unwrap_or(0);
        let max_dimension_score = record
            .dimensions
            .iter()
            .map(|dimension| dimension.score)
            .max()
            .unwrap_or(0);
        let dimension_threshold_floor = record
            .dimensions
            .iter()
            .map(|dimension| dimension.threshold)
            .min()
            .unwrap_or(0);
        let dimension_order_hash = dimension_order_hash(&record.dimensions);
        let dimension_score_hash = dimension_score_hash(&record.dimensions);
        let payload_hash = eval_payload_hash(record);
        let verdict = record.decision();
        let mut receipt = Self {
            schema_version: EVAL_SCORECARD_SCHEMA_VERSION,
            record_type: EVAL_SCORECARD_RECORD,
            score: record.score,
            threshold_used: record.threshold_used,
            dimension_count,
            min_dimension_score,
            max_dimension_score,
            dimension_threshold_floor,
            dimension_order_hash,
            dimension_score_hash,
            payload_hash,
            verdict,
            receipt_hash: 0,
        };
        receipt.receipt_hash = receipt.expected_receipt_hash();
        receipt
    }

    pub fn is_valid_for(self, record: &EvalRecord) -> bool {
        self == EvalScorecardReceipt::from_record(record) && self.is_self_consistent()
    }

    pub fn is_self_consistent(self) -> bool {
        self.schema_version == EVAL_SCORECARD_SCHEMA_VERSION
            && self.record_type == EVAL_SCORECARD_RECORD
            && self.dimension_count != 0
            && self.payload_hash != 0
            && self.dimension_order_hash != 0
            && self.dimension_score_hash != 0
            && self.receipt_hash == self.expected_receipt_hash()
    }

    pub fn submission(self) -> EvidenceSubmission {
        EvidenceSubmission::with_effect_payload(
            GateId::Eval,
            Evidence::EvalScore,
            self.is_self_consistent() && self.verdict == EvalDecision::Pass,
            if self.verdict == EvalDecision::Pass {
                PacketEffect::CompleteObjective
            } else {
                PacketEffect::None
            },
            self.receipt_hash,
        )
    }

    pub fn expected_receipt_hash(self) -> u64 {
        let mut h = 0x0e7a_5c0f_eeca_1001u64;
        h = mix(h, self.schema_version);
        h = mix(h, self.record_type);
        h = mix(h, self.score);
        h = mix(h, self.threshold_used);
        h = mix(h, self.dimension_count);
        h = mix(h, self.min_dimension_score);
        h = mix(h, self.max_dimension_score);
        h = mix(h, self.dimension_threshold_floor);
        h = mix(h, self.dimension_order_hash);
        h = mix(h, self.dimension_score_hash);
        h = mix(h, self.payload_hash);
        h = mix(h, self.verdict as u64);
        h.max(1)
    }
}

pub fn encode_eval_scorecard_receipt_ndjson(receipt: EvalScorecardReceipt) -> String {
    let verdict = match receipt.verdict {
        EvalDecision::Pass => 1,
        EvalDecision::Fail => 2,
    };
    format!(
        "[{},{},{},{},{},{},{},{},{},{},{},{},{}]",
        receipt.schema_version,
        receipt.record_type,
        receipt.score,
        receipt.threshold_used,
        receipt.dimension_count,
        receipt.min_dimension_score,
        receipt.max_dimension_score,
        receipt.dimension_threshold_floor,
        receipt.dimension_order_hash,
        receipt.dimension_score_hash,
        receipt.payload_hash,
        verdict,
        receipt.receipt_hash
    )
}

pub fn decode_eval_scorecard_receipt_ndjson(
    line: &str,
) -> Result<EvalScorecardReceipt, CanonError> {
    let trimmed = line.trim();
    let body = trimmed
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .ok_or(CanonError::InvalidTlogRecord)?;
    let fields = body
        .split(',')
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| CanonError::InvalidTlogRecord)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if fields.len() != 13 {
        return Err(CanonError::InvalidTlogRecord);
    }
    if fields[0] != EVAL_SCORECARD_SCHEMA_VERSION || fields[1] != EVAL_SCORECARD_RECORD {
        return Err(CanonError::InvalidTlogRecord);
    }
    let verdict = match fields[11] {
        1 => EvalDecision::Pass,
        2 => EvalDecision::Fail,
        _ => return Err(CanonError::InvalidTlogRecord),
    };
    let receipt = EvalScorecardReceipt {
        schema_version: fields[0],
        record_type: fields[1],
        score: fields[2],
        threshold_used: fields[3],
        dimension_count: fields[4],
        min_dimension_score: fields[5],
        max_dimension_score: fields[6],
        dimension_threshold_floor: fields[7],
        dimension_order_hash: fields[8],
        dimension_score_hash: fields[9],
        payload_hash: fields[10],
        verdict,
        receipt_hash: fields[12],
    };
    if !receipt.is_self_consistent() {
        return Err(CanonError::InvalidTlogRecord);
    }
    Ok(receipt)
}

pub fn append_eval_scorecard_receipt_ndjson(
    path: impl AsRef<Path>,
    receipt: EvalScorecardReceipt,
) -> Result<(), CanonError> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|_| CanonError::TlogIo)?;
        }
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|_| CanonError::TlogIo)?;
    writeln!(file, "{}", encode_eval_scorecard_receipt_ndjson(receipt))
        .map_err(|_| CanonError::TlogIo)?;
    file.sync_all().map_err(|_| CanonError::TlogIo)
}

pub fn load_eval_scorecard_receipts_ndjson(
    path: impl AsRef<Path>,
) -> Result<Vec<EvalScorecardReceipt>, CanonError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = fs::File::open(path).map_err(|_| CanonError::TlogIo)?;
    let reader = BufReader::new(file);
    let mut receipts = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|_| CanonError::TlogIo)?;
        if line.trim().is_empty() || !line.trim_start().starts_with('[') {
            continue;
        }
        if let Ok(receipt) = decode_eval_scorecard_receipt_ndjson(&line) {
            receipts.push(receipt);
        }
    }
    Ok(receipts)
}

impl EvidenceProducer for EvalRecord {
    type Record = EvalRecord;

    fn record(&self) -> &Self::Record {
        self
    }

    fn submission(&self) -> EvidenceSubmission {
        EvalRecord::submission(self)
    }
}

fn eval_payload_hash(record: &EvalRecord) -> u64 {
    let mut h = 0x510e_527f_ade6_82d1u64;
    h = mix(h, record.score);
    h = mix(h, record.threshold_used);
    for dimension in &record.dimensions {
        h = mix(h, dimension_id_hash(dimension.id));
        h = mix(h, dimension.score);
        h = mix(h, dimension.threshold);
    }
    h.max(1)
}

fn dimension_order_hash(dimensions: &[EvalDimension]) -> u64 {
    dimension_hash(
        0x0e7a_0d0e_51a7_0001u64,
        dimensions,
        DimensionHashMode::OrderOnly,
    )
}

fn dimension_score_hash(dimensions: &[EvalDimension]) -> u64 {
    dimension_hash(
        0x0e7a_5c02_e15a_0001u64,
        dimensions,
        DimensionHashMode::ScoreAndThreshold,
    )
}

#[derive(Clone, Copy)]
enum DimensionHashMode {
    OrderOnly,
    ScoreAndThreshold,
}

fn dimension_hash(seed: u64, dimensions: &[EvalDimension], mode: DimensionHashMode) -> u64 {
    let mut h = seed;
    h = mix(h, dimensions.len() as u64);
    for dimension in dimensions {
        h = mix(h, dimension_id_hash(dimension.id));
        if matches!(mode, DimensionHashMode::ScoreAndThreshold) {
            h = mix(h, dimension.score);
            h = mix(h, dimension.threshold);
        }
    }
    h.max(1)
}

fn dimension_id_hash(id: &str) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for byte in id.as_bytes() {
        h ^= u64::from(*byte);
        h = h.wrapping_mul(0x100000001b3);
    }
    h.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn passing_record() -> EvalRecord {
        EvalRecord {
            score: 91,
            threshold_used: 80,
            dimensions: vec![
                EvalDimension {
                    id: "correctness",
                    score: 90,
                    threshold: 80,
                },
                EvalDimension {
                    id: "transparency",
                    score: 92,
                    threshold: 80,
                },
            ],
        }
    }

    #[test]
    fn scorecard_receipt_binds_eval_payload_and_verdict() {
        let record = passing_record();
        let receipt = record.scorecard_receipt();

        assert!(receipt.is_self_consistent());
        assert!(receipt.is_valid_for(&record));
        assert_eq!(receipt.verdict, EvalDecision::Pass);
        assert_eq!(receipt.dimension_count, 2);
        assert_eq!(receipt.min_dimension_score, 90);
        assert_eq!(receipt.max_dimension_score, 92);
        assert_eq!(receipt.dimension_threshold_floor, 80);
        assert_eq!(receipt.payload_hash, eval_payload_hash(&record));
        assert_eq!(receipt.submission().gate, GateId::Eval);
        assert!(receipt.submission().passed);
    }

    #[test]
    fn scorecard_receipt_rejects_tampered_dimension_order() {
        let record = passing_record();
        let receipt = record.scorecard_receipt();
        let reordered = EvalRecord {
            dimensions: vec![record.dimensions[1], record.dimensions[0]],
            ..record
        };

        assert_ne!(
            receipt.dimension_order_hash,
            reordered.scorecard_receipt().dimension_order_hash
        );
        assert!(!receipt.is_valid_for(&reordered));
    }

    #[test]
    fn eval_dimension_hash_domains_remain_distinct() {
        let record = passing_record();
        let reordered_dimensions = vec![record.dimensions[1], record.dimensions[0]];
        let score_changed_dimensions = vec![
            EvalDimension {
                score: record.dimensions[0].score + 1,
                ..record.dimensions[0]
            },
            record.dimensions[1],
        ];
        let threshold_changed_dimensions = vec![
            record.dimensions[0],
            EvalDimension {
                threshold: record.dimensions[1].threshold + 1,
                ..record.dimensions[1]
            },
        ];

        assert_ne!(
            dimension_order_hash(&record.dimensions),
            dimension_order_hash(&reordered_dimensions)
        );
        assert_eq!(
            dimension_order_hash(&record.dimensions),
            dimension_order_hash(&score_changed_dimensions)
        );
        assert_eq!(
            dimension_order_hash(&record.dimensions),
            dimension_order_hash(&threshold_changed_dimensions)
        );
        assert_ne!(
            dimension_score_hash(&record.dimensions),
            dimension_score_hash(&score_changed_dimensions)
        );
        assert_ne!(
            dimension_score_hash(&record.dimensions),
            dimension_score_hash(&threshold_changed_dimensions)
        );

        let score_changed_record = EvalRecord {
            score: record.score + 1,
            ..record.clone()
        };
        let threshold_changed_record = EvalRecord {
            threshold_used: record.threshold_used + 1,
            ..record.clone()
        };

        assert_ne!(
            eval_payload_hash(&record),
            eval_payload_hash(&score_changed_record)
        );
        assert_ne!(
            eval_payload_hash(&record),
            eval_payload_hash(&threshold_changed_record)
        );
    }

    #[test]
    fn scorecard_receipt_rejects_tampered_hash() {
        let record = passing_record();
        let mut receipt = record.scorecard_receipt();
        receipt.score = receipt.score.saturating_add(1);

        assert!(!receipt.is_self_consistent());
        assert!(!receipt.is_valid_for(&record));
    }

    #[test]
    fn scorecard_receipt_fails_empty_dimension_record() {
        let record = EvalRecord {
            score: 100,
            threshold_used: 80,
            dimensions: Vec::new(),
        };
        let receipt = record.scorecard_receipt();

        assert_eq!(receipt.verdict, EvalDecision::Fail);
        assert_eq!(receipt.dimension_count, 0);
        assert!(!receipt.is_self_consistent());
        assert!(!receipt.submission().passed);
    }
}
