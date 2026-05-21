//! Event envelope validation for ai TLog compact array records.
//!
//! Ported from canon-check; adapted from JSON-object API to u64-slice API
//! to match the ai compact NDJSON format `[schema_version, record_type, ...]`.
//!
//! Call `run_checks` before durable TLog append or when loading external events
//! to reject malformed records before they cross the truth boundary.

// ---------------------------------------------------------------------------
// Result types — unchanged from canon-check
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct CheckWarning {
    pub check: &'static str,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct CheckError {
    pub check: &'static str,
    pub message: String,
}

/// Warn = degraded/recoverable. Err = invariant violation (reserved for future escalation).
#[derive(Debug, Clone)]
pub enum CheckResult {
    Ok,
    Warn(Vec<CheckWarning>),
    Err(Vec<CheckError>),
}

impl CheckResult {
    pub fn is_ok(&self) -> bool {
        matches!(self, CheckResult::Ok)
    }
}

// ---------------------------------------------------------------------------
// Check trait — adapted to u64 slices instead of serde_json::Value
// to avoid precision loss from large u64 hash fields in compact NDJSON.
// ---------------------------------------------------------------------------

pub trait Check: Send + Sync {
    fn name(&self) -> &'static str;
    fn run(&self, fields: &[u64]) -> CheckResult;
}

// ---------------------------------------------------------------------------
// SchemaVersionCheck — field[0] must equal the expected schema version.
// Equivalent to SourceCheck: identifies the format "origin".
// ---------------------------------------------------------------------------

pub struct SchemaVersionCheck {
    pub expected: u64,
}

impl Check for SchemaVersionCheck {
    fn name(&self) -> &'static str {
        "schema_version"
    }

    fn run(&self, fields: &[u64]) -> CheckResult {
        match fields.first() {
            Some(&v) if v == self.expected => CheckResult::Ok,
            Some(&v) => CheckResult::Warn(vec![CheckWarning {
                check: self.name(),
                message: format!("schema_version={v} expected={}", self.expected),
            }]),
            None => CheckResult::Warn(vec![CheckWarning {
                check: self.name(),
                message: "schema_version field missing".into(),
            }]),
        }
    }
}

// ---------------------------------------------------------------------------
// RecordTypeCheck — field[1] must be in the set of known discriminants.
// Equivalent to KindCheck: identifies the event "kind".
// ---------------------------------------------------------------------------

pub struct RecordTypeCheck {
    pub known: Vec<u64>,
}

impl Check for RecordTypeCheck {
    fn name(&self) -> &'static str {
        "record_type"
    }

    fn run(&self, fields: &[u64]) -> CheckResult {
        match fields.get(1) {
            Some(v) if self.known.contains(v) => CheckResult::Ok,
            Some(v) => CheckResult::Warn(vec![CheckWarning {
                check: self.name(),
                message: format!("record_type={v} not in known discriminant set"),
            }]),
            None => CheckResult::Warn(vec![CheckWarning {
                check: self.name(),
                message: "record_type field missing".into(),
            }]),
        }
    }
}

// ---------------------------------------------------------------------------
// MinFieldsCheck — array must have at least `min` elements.
// Equivalent to EnvelopeCheck: ensures mandatory field structure is present.
// ---------------------------------------------------------------------------

pub struct MinFieldsCheck {
    pub min: usize,
}

impl Check for MinFieldsCheck {
    fn name(&self) -> &'static str {
        "min_fields"
    }

    fn run(&self, fields: &[u64]) -> CheckResult {
        if fields.len() >= self.min {
            CheckResult::Ok
        } else {
            CheckResult::Warn(vec![CheckWarning {
                check: self.name(),
                message: format!("has {} fields, need at least {}", fields.len(), self.min),
            }])
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Run all checks against a parsed u64 field slice, returning only non-Ok results.
pub fn run_checks(checks: &[Box<dyn Check>], fields: &[u64]) -> Vec<CheckResult> {
    checks
        .iter()
        .map(|c| c.run(fields))
        .filter(|r| !r.is_ok())
        .collect()
}

/// Default structural check suite for ai TLog compact array records.
pub fn tlog_event_checks(schema_version: u64, record_type: u64) -> Vec<Box<dyn Check>> {
    vec![
        Box::new(SchemaVersionCheck {
            expected: schema_version,
        }),
        Box::new(RecordTypeCheck {
            known: vec![record_type],
        }),
        Box::new(MinFieldsCheck { min: 3 }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::ndjson::{TLOG_RECORD_EVENT, TLOG_SCHEMA_VERSION};

    fn valid_fields() -> Vec<u64> {
        // Minimal well-formed TLog record header + a few payload fields.
        vec![TLOG_SCHEMA_VERSION, TLOG_RECORD_EVENT, 1, 2, 3]
    }

    #[test]
    fn valid_event_passes_all_checks() {
        let checks = tlog_event_checks(TLOG_SCHEMA_VERSION, TLOG_RECORD_EVENT);
        let results = run_checks(&checks, &valid_fields());
        assert!(
            results.is_empty(),
            "valid event should produce no failures: {results:?}"
        );
    }

    #[test]
    fn wrong_schema_version_produces_warning() {
        let checks = tlog_event_checks(TLOG_SCHEMA_VERSION, TLOG_RECORD_EVENT);
        let mut fields = valid_fields();
        fields[0] = TLOG_SCHEMA_VERSION + 1;
        let results = run_checks(&checks, &fields);
        assert!(!results.is_empty());
        assert!(matches!(results[0], CheckResult::Warn(_)));
    }

    #[test]
    fn unknown_record_type_produces_warning() {
        let checks = tlog_event_checks(TLOG_SCHEMA_VERSION, TLOG_RECORD_EVENT);
        let mut fields = valid_fields();
        fields[1] = 99;
        let results = run_checks(&checks, &fields);
        assert!(!results.is_empty());
        assert!(matches!(results[0], CheckResult::Warn(_)));
    }

    #[test]
    fn empty_fields_reject_all_structural_checks() {
        let checks = tlog_event_checks(TLOG_SCHEMA_VERSION, TLOG_RECORD_EVENT);
        let results = run_checks(&checks, &[]);
        assert_eq!(
            results.len(),
            3,
            "all three checks should fire on empty input"
        );
    }

    #[test]
    fn two_fields_only_fails_min_fields_check() {
        let checks = tlog_event_checks(TLOG_SCHEMA_VERSION, TLOG_RECORD_EVENT);
        let results = run_checks(&checks, &[TLOG_SCHEMA_VERSION, TLOG_RECORD_EVENT]);
        // schema_version and record_type pass; min_fields(3) fails
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], CheckResult::Warn(_)));
    }
}
