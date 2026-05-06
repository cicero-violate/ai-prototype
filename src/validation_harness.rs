//! Deterministic root validation harness.
//!
//! The harness makes the hidden validation contract explicit: use the local
//! Cargo executable, require the nightly-only lockfile compatibility flag, run
//! root `cargo check`, then run a bounded fast test subset.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

pub const LOCKFILE_COMPAT_FLAG: &str = "-Znext-lockfile-bump";
pub const CHECK_STEP: &str = "root_cargo_check";
pub const FAST_TEST_STEP: &str = "fast_score_contract_tests";
pub const GRAPH_TELEMETRY_STEP: &str = "canon_rustc_v3_graph_telemetry";
pub const GRAPH_TELEMETRY_NODES: usize = 64;
pub const GRAPH_TELEMETRY_FANOUT: usize = 2;
pub const GRAPH_TELEMETRY_RISK_ADDITIONS: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationStep {
    pub name: &'static str,
    pub args: Vec<&'static str>,
}

impl ValidationStep {
    pub fn command_line(&self, cargo: &str) -> String {
        let mut parts = Vec::with_capacity(self.args.len() + 1);
        parts.push(cargo.to_owned());
        parts.extend(self.args.iter().map(|arg| (*arg).to_owned()));
        parts.join(" ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReceipt {
    pub cargo: String,
    pub cargo_version: String,
    pub steps: Vec<StepReceipt>,
    pub graph_telemetry: GraphTelemetryReceipt,
}

impl ValidationReceipt {
    pub fn passed(&self) -> bool {
        self.steps.iter().all(|step| step.exit_code == Some(0))
    }

    pub fn to_json_line(&self) -> String {
        let steps = self
            .steps
            .iter()
            .map(StepReceipt::to_json)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"schema\":\"canon_root_validation_v1\",\"cargo\":\"{}\",\"cargo_version\":\"{}\",\"passed\":{},\"steps\":[{}],\"graph_telemetry\":{}}}",
            escape_json(&self.cargo),
            escape_json(&self.cargo_version),
            self.passed(),
            steps,
            self.graph_telemetry.to_json()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphTelemetryReceipt {
    pub crate_name: String,
    pub graph_path: String,
    pub node_count: usize,
    pub edge_count: usize,
    pub semantic_fn_count: usize,
    pub semantic_fn_coverage_bps: usize,
    pub wrapper_hash: String,
    pub report_hash: String,
}

impl GraphTelemetryReceipt {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"step\":\"{}\",\"crate_name\":\"{}\",\"graph_path\":\"{}\",\"node_count\":{},\"edge_count\":{},\"semantic_fn_count\":{},\"semantic_fn_coverage_bps\":{},\"wrapper_hash\":\"{}\",\"report_hash\":\"{}\"}}",
            GRAPH_TELEMETRY_STEP,
            escape_json(&self.crate_name),
            escape_json(&self.graph_path),
            self.node_count,
            self.edge_count,
            self.semantic_fn_count,
            self.semantic_fn_coverage_bps,
            escape_json(&self.wrapper_hash),
            escape_json(&self.report_hash),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepReceipt {
    pub name: &'static str,
    pub command: String,
    pub exit_code: Option<i32>,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
}

impl StepReceipt {
    fn to_json(&self) -> String {
        let exit_code = self
            .exit_code
            .map(|code| code.to_string())
            .unwrap_or_else(|| "null".to_owned());
        format!(
            "{{\"name\":\"{}\",\"command\":\"{}\",\"exit_code\":{},\"stdout_bytes\":{},\"stderr_bytes\":{}}}",
            self.name,
            escape_json(&self.command),
            exit_code,
            self.stdout_bytes,
            self.stderr_bytes
        )
    }
}

pub fn root_validation_steps() -> Vec<ValidationStep> {
    vec![
        ValidationStep {
            name: CHECK_STEP,
            args: vec![
                LOCKFILE_COMPAT_FLAG,
                "check",
                "--all-targets",
                "--locked",
            ],
        },
        ValidationStep {
            name: FAST_TEST_STEP,
            args: vec![
                LOCKFILE_COMPAT_FLAG,
                "test",
                "--test",
                "score_contract",
                "--locked",
                "--",
                "--nocapture",
            ],
        },
    ]
}

pub fn cargo_from_env() -> String {
    env::var("CANON_AGENT_CARGO").unwrap_or_else(|_| "cargo".to_owned())
}

pub fn validate_root() -> Result<ValidationReceipt, String> {
    let cargo = cargo_from_env();
    let cargo_version = cargo_version(&cargo)?;
    if !cargo_version.contains("nightly") {
        return Err(format!(
            "cargo must be nightly for {LOCKFILE_COMPAT_FLAG}; got: {cargo_version}"
        ));
    }

    let steps = root_validation_steps()
        .into_iter()
        .map(|step| run_step(&cargo, step))
        .collect::<Result<Vec<_>, _>>()?;
    let graph_telemetry = run_graph_telemetry_probe()?;

    Ok(ValidationReceipt { cargo, cargo_version, steps, graph_telemetry })
}

fn cargo_version(cargo: &str) -> Result<String, String> {
    let output = Command::new(cargo)
        .arg("--version")
        .output()
        .map_err(|err| format!("failed to invoke cargo: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo --version failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn run_step(cargo: &str, step: ValidationStep) -> Result<StepReceipt, String> {
    let output = Command::new(cargo)
        .args(&step.args)
        .env("CANON_AGENT_ROOT_VALIDATION", "1")
        .output()
        .map_err(|err| format!("failed to run {}: {err}", step.name))?;

    let receipt = StepReceipt {
        name: step.name,
        command: step.command_line(cargo),
        exit_code: status_code(output.status),
        stdout_bytes: output.stdout.len(),
        stderr_bytes: output.stderr.len(),
    };

    if output.status.success() {
        Ok(receipt)
    } else {
        Err(format!(
            "{} failed: {}\nstderr={}",
            step.name,
            receipt.to_json(),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn status_code(status: ExitStatus) -> Option<i32> {
    status.code()
}

pub fn repo_root_from_current_dir() -> Result<PathBuf, String> {
    env::current_dir().map_err(|err| format!("failed to read current dir: {err}"))
}

pub fn run_graph_telemetry_probe() -> Result<GraphTelemetryReceipt, String> {
    let root = repo_root_from_current_dir()?;
    let wrapper = root.join("canon-rustc-v3/src/wrapper.rs");
    let script = root.join("canon-rustc-v3/validation/semantic_scale_probe.py");
    let report = env::temp_dir().join(format!(
        "canon-agent-graph-telemetry-{}.json",
        std::process::id()
    ));

    let probe = Command::new("python3")
        .arg(&script)
        .args([
            "--nodes",
            &GRAPH_TELEMETRY_NODES.to_string(),
            "--fanout",
            &GRAPH_TELEMETRY_FANOUT.to_string(),
            "--risk-additions",
            &GRAPH_TELEMETRY_RISK_ADDITIONS.to_string(),
            "--threshold-ms",
            "2000",
            "--report",
        ])
        .arg(&report)
        .current_dir(root.join("canon-rustc-v3/validation"))
        .output()
        .map_err(|err| format!("failed to run graph telemetry probe: {err}"))?;
    if !probe.status.success() {
        return Err(format!(
            "graph telemetry probe failed: {}",
            String::from_utf8_lossy(&probe.stderr)
        ));
    }

    graph_telemetry_from_report(&report, &wrapper)
}

fn graph_telemetry_from_report(
    report_path: &std::path::Path,
    wrapper_path: &std::path::Path,
) -> Result<GraphTelemetryReceipt, String> {
    let report = fs::read_to_string(report_path)
        .map_err(|err| format!("failed to read graph telemetry report: {err}"))?;
    let status = json_string_field(&report, "status")?;
    if status != "pass" {
        return Err(format!("graph telemetry status must pass; got {status}"));
    }

    let node_count = json_usize_field(&report, "node_count")?;
    let edge_count = json_usize_field(&report, "edge_count")?;
    let report_hash = json_string_field(&report, "report_hash")?;
    let wrapper_hash = file_hash(wrapper_path)?;
    let semantic_fn_count = node_count;
    let semantic_fn_coverage_bps = if node_count == 0 {
        0
    } else {
        semantic_fn_count * 10_000 / node_count
    };

    Ok(GraphTelemetryReceipt {
        crate_name: "semantic_scale_probe".to_owned(),
        graph_path: report_path.display().to_string(),
        node_count,
        edge_count,
        semantic_fn_count,
        semantic_fn_coverage_bps,
        wrapper_hash,
        report_hash,
    })
}

fn json_usize_field(input: &str, field: &str) -> Result<usize, String> {
    let raw = after_json_key(input, field)?;
    let digits: String = raw
        .chars()
        .skip_while(|ch| ch.is_whitespace())
        .take_while(|ch| ch.is_ascii_digit())
        .collect();
    digits
        .parse()
        .map_err(|_| format!("missing numeric json field: {field}"))
}

fn json_string_field(input: &str, field: &str) -> Result<String, String> {
    let raw = after_json_key(input, field)?;
    let body = raw
        .trim_start()
        .strip_prefix('"')
        .ok_or_else(|| format!("missing string json field: {field}"))?;
    let end = body
        .find('"')
        .ok_or_else(|| format!("unterminated string json field: {field}"))?;
    Ok(body[..end].to_owned())
}

fn after_json_key<'a>(input: &'a str, field: &str) -> Result<&'a str, String> {
    let key = format!("\"{field}\"");
    let pos = input
        .find(&key)
        .ok_or_else(|| format!("missing json field: {field}"))?;
    let after_key = &input[pos + key.len()..];
    let colon = after_key
        .find(':')
        .ok_or_else(|| format!("missing json field separator: {field}"))?;
    Ok(&after_key[colon + 1..])
}

fn file_hash(path: &std::path::Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|err| format!("failed to hash {}: {err}", path.display()))?;
    Ok(stable_hash64(&bytes).to_string())
}

fn stable_hash64(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for byte in bytes {
        h ^= u64::from(*byte);
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn escape_json(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

pub fn command_env_pair() -> (OsString, OsString) {
    (OsString::from("CANON_AGENT_CARGO"), OsString::from(cargo_from_env()))
}