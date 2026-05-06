//! Deterministic root validation harness.
//!
//! The harness makes the hidden validation contract explicit: use the local
//! Cargo executable, require the nightly-only lockfile compatibility flag, run
//! root `cargo check`, then run a bounded fast test subset.

use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

pub const LOCKFILE_COMPAT_FLAG: &str = "-Znext-lockfile-bump";
pub const CHECK_STEP: &str = "root_cargo_check";
pub const FAST_TEST_STEP: &str = "fast_score_contract_tests";

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
            "{{\"schema\":\"canon_root_validation_v1\",\"cargo\":\"{}\",\"cargo_version\":\"{}\",\"passed\":{},\"steps\":[{}]}}",
            escape_json(&self.cargo),
            escape_json(&self.cargo_version),
            self.passed(),
            steps
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

    Ok(ValidationReceipt {
        cargo,
        cargo_version,
        steps,
    })
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