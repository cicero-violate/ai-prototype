//! Canonical project TLog introspection.

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::codec::ndjson::load_tlog_ndjson;
use crate::kernel::CanonError;

pub const CANONICAL_TLOG_DIR_RELATIVE_PATH: &str = "state/tlog";
pub const CANONICAL_TLOG_FILE_NAME: &str = "canon-agent.tlog.ndjson";
pub const CANONICAL_TLOG_RELATIVE_PATH: &str = "state/tlog/canon-agent.tlog.ndjson";
pub const LEGACY_WORKER_TLOG_FILE_NAME: &str = "worker-tlog.ndjson";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalIntrospectionReport {
    pub canonical_path: PathBuf,
    pub latest_phase: Option<String>,
    pub event_count: usize,
    pub latest_evaluator_result: Option<String>,
    pub latest_validation_result: Option<String>,
    pub latest_score_report_hash: Option<u64>,
    pub worker_state: WorkerStateReport,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerStateReport {
    pub status: String,
    pub legacy_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct CanonicalEvidenceScan {
    latest_evaluator_result: Option<String>,
    latest_validation_result: Option<String>,
    latest_score_report_hash: Option<u64>,
}

pub fn canonical_tlog_path_from_dir(dir: &Path) -> PathBuf {
    dir.join(CANONICAL_TLOG_FILE_NAME)
}

pub fn default_canonical_tlog_path() -> PathBuf {
    PathBuf::from(CANONICAL_TLOG_RELATIVE_PATH)
}

pub fn append_validation_result_ndjson(
    path: impl AsRef<Path>,
    record_type: &str,
    passed: bool,
    receipt_hash: u64,
) -> Result<(), CanonError> {
    let line = serde_json::json!({
        "canonical_kind": "validation_result",
        "record_type": record_type,
        "passed": passed,
        "receipt_hash": receipt_hash
    })
    .to_string();
    append_canonical_line(path, &line)
}

pub fn append_score_report_update_ndjson(
    path: impl AsRef<Path>,
    report_hash: u64,
) -> Result<(), CanonError> {
    let line = serde_json::json!({
        "canonical_kind": "score_report_update",
        "report_hash": report_hash
    })
    .to_string();
    append_canonical_line(path, &line)
}

pub fn append_canonical_line(path: impl AsRef<Path>, line: &str) -> Result<(), CanonError> {
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
    writeln!(file, "{line}").map_err(|_| CanonError::TlogIo)?;
    file.sync_all().map_err(|_| CanonError::TlogIo)
}

pub fn introspect_canonical_tlog(
    path: impl AsRef<Path>,
) -> Result<CanonicalIntrospectionReport, CanonError> {
    let path = path.as_ref();
    let tlog = load_tlog_ndjson(path)?;
    let evidence = scan_canonical_tlog_evidence(path)?;

    Ok(CanonicalIntrospectionReport {
        canonical_path: path.to_path_buf(),
        latest_phase: tlog.last().map(|event| format!("{:?}", event.to)),
        event_count: tlog.len(),
        latest_evaluator_result: evidence.latest_evaluator_result,
        latest_validation_result: evidence.latest_validation_result,
        latest_score_report_hash: evidence.latest_score_report_hash,
        worker_state: detect_worker_state(path),
    })
}

fn scan_canonical_tlog_evidence(path: &Path) -> Result<CanonicalEvidenceScan, CanonError> {
    let mut evidence = CanonicalEvidenceScan::default();

    if !path.exists() {
        return Ok(evidence);
    }

    let file = fs::File::open(path).map_err(|_| CanonError::TlogIo)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.map_err(|_| CanonError::TlogIo)?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(verdict) = eval_scorecard_verdict_from_ndjson(trimmed) {
            evidence.latest_evaluator_result = Some(verdict.to_string());
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
            continue;
        };
        match value.get("canonical_kind").and_then(Value::as_str) {
            Some("validation_result") => {
                let record_type = value
                    .get("record_type")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                let verdict = if value
                    .get("passed")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    "pass"
                } else {
                    "fail"
                };
                evidence.latest_validation_result = Some(format!("{record_type}:{verdict}"));
                if record_type.contains("evaluator") {
                    evidence.latest_evaluator_result = Some(verdict.to_string());
                }
            }
            Some("score_report_update") => {
                evidence.latest_score_report_hash =
                    value.get("report_hash").and_then(Value::as_u64);
            }
            _ => {}
        }
    }

    Ok(evidence)
}

fn eval_scorecard_verdict_from_ndjson(line: &str) -> Option<&'static str> {
    let trimmed = line.trim();
    let body = trimmed.strip_prefix('[')?.strip_suffix(']')?;
    let mut fields = Vec::new();
    for part in body.split(',') {
        fields.push(part.trim().parse::<u64>().ok()?);
    }
    if fields.len() != 13 {
        return None;
    }
    match fields[11] {
        1 => Some("pass"),
        2 => Some("fail"),
        _ => None,
    }
}

fn detect_worker_state(canonical_path: &Path) -> WorkerStateReport {
    let legacy_paths = legacy_worker_tlog_paths(canonical_path);
    let status = if !canonical_path.exists() {
        "missing_canonical_tlog"
    } else if legacy_paths.is_empty() {
        "canonical"
    } else if legacy_paths
        .iter()
        .any(|legacy| legacy_older_than_canonical(legacy, canonical_path))
    {
        "stale_legacy_worker_tlog"
    } else {
        "legacy_worker_tlog_present"
    };
    WorkerStateReport {
        status: status.to_string(),
        legacy_paths,
    }
}

fn legacy_worker_tlog_paths(canonical_path: &Path) -> Vec<PathBuf> {
    let Some(parent) = canonical_path.parent() else {
        return Vec::new();
    };
    let Ok(entries) = fs::read_dir(parent) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == LEGACY_WORKER_TLOG_FILE_NAME)
        })
        .collect()
}

fn legacy_older_than_canonical(legacy: &Path, canonical: &Path) -> bool {
    let Ok(legacy_meta) = fs::metadata(legacy) else {
        return false;
    };
    let Ok(canonical_meta) = fs::metadata(canonical) else {
        return false;
    };
    match (legacy_meta.modified(), canonical_meta.modified()) {
        (Ok(legacy_time), Ok(canonical_time)) => legacy_time < canonical_time,
        _ => false,
    }
}
