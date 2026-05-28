//! `canon_score` MCP tool — runs the score binary and returns axis values.
//!
//! Parameters (all optional):
//!   artifact_root    — workspace-relative path to rustc artifact dir (default: state/rustc)
//!   report           — workspace-relative path to write SCORE_REPORT.md (default: SCORE_REPORT.md)
//!   plan_seed        — workspace-relative path to plan.json for automatic node seeding
//!                      (default: state/plan.json; pass "" to disable)
//!   seed_threshold   — axes scoring below this value get new plan nodes (default: 7.0)

use std::process::Command;

use serde_json::{json, Value};

use crate::runtime::WorkspaceView;
use crate::service::scheduler::plan_store::PLAN_FILE;

pub const CANON_SCORE_TOOL: &str = "canon_score";

pub fn run(args: &Value, workspace: &WorkspaceView) -> Value {
    let artifact_root = args
        .get("artifact_root")
        .and_then(Value::as_str)
        .unwrap_or("state/rustc");
    let report = args
        .get("report")
        .and_then(Value::as_str)
        .unwrap_or("SCORE_REPORT.md");
    // Default to state/plan.json; pass "" to disable seeding.
    let plan_seed = args
        .get("plan_seed")
        .and_then(Value::as_str)
        .unwrap_or(PLAN_FILE);
    let seed_threshold = args
        .get("seed_threshold")
        .and_then(Value::as_f64)
        .unwrap_or(7.0);

    let bin = match score_bin() {
        Ok(b) => b,
        Err(e) => return error(e),
    };

    let artifact_root_abs = workspace.root.join(artifact_root);
    let report_abs = workspace.root.join(report);

    let mut cmd = Command::new(&bin);
    cmd.arg("--artifact-root")
        .arg(&artifact_root_abs)
        .arg("--report")
        .arg(&report_abs)
        .arg("--date")
        .arg(current_date());

    if !plan_seed.is_empty() {
        let plan_seed_abs = workspace.root.join(plan_seed);
        cmd.arg("--plan-seed")
            .arg(&plan_seed_abs)
            .arg("--seed-threshold")
            .arg(seed_threshold.to_string());
    }

    let output = match cmd.current_dir(&workspace.root).output() {
        Ok(o) => o,
        Err(e) => return error(format!("Failed to run score binary at {bin}: {e}")),
    };

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    if !output.status.success() {
        return error(format!(
            "score binary exited with {}\nstdout: {stdout}\nstderr: {stderr}",
            output.status
        ));
    }

    let scores = parse_score_output(&stdout);
    let text = if stderr.trim().is_empty() {
        stdout.clone()
    } else {
        format!("{stdout}\n{stderr}")
    };

    ok(json!({
        "scores": scores,
        "report_path": report_abs.display().to_string(),
        "output": text.trim(),
    }))
}

fn score_bin() -> Result<String, String> {
    if let Ok(path) = std::env::var("SCORE_BIN") {
        return Ok(path);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join("score");
            if candidate.exists() {
                return Ok(candidate.to_string_lossy().into_owned());
            }
        }
    }
    Ok("score".to_string())
}

fn parse_score_output(stdout: &str) -> Value {
    let mut map = serde_json::Map::new();
    for line in stdout.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("score: G = ") {
            if let Some(val) = rest
                .split('/')
                .next()
                .and_then(|s| s.trim().parse::<f64>().ok())
            {
                map.insert("G".to_string(), json!(val));
            }
        } else if let Some(colon) = line.find(':') {
            let key = line[..colon].trim().to_string();
            let val_str = line[colon + 1..].trim();
            if let Ok(val) = val_str.parse::<f64>() {
                map.insert(key, json!(val));
            }
        }
    }
    Value::Object(map)
}

fn current_date() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

fn ok(payload: Value) -> Value {
    let text = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string());
    json!({ "content": [{ "type": "text", "text": text }], "isError": false })
}

fn error(msg: impl Into<String>) -> Value {
    json!({ "content": [{ "type": "text", "text": format!("Error: {}", msg.into()) }], "isError": true })
}
