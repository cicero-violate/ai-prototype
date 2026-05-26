//! Background invariant mining loop.
//!
//! The loop is intentionally lightweight: it periodically mines candidates from
//! durable workspace state, validates them, and only promotes when explicitly
//! enabled by environment. Promotion is still warning/context-only.

use std::env;
use std::path::PathBuf;
use std::time::Duration;

use crate::service::invariants::{mine_and_write_workspace, promote_workspace, validate_workspace};

const DEFAULT_INTERVAL_SECS: u64 = 300;
const DEFAULT_STARTUP_DELAY_SECS: u64 = 10;
const DEFAULT_MIN_SUPPORT: u64 = 1;

pub fn start(project_dir: PathBuf) {
    if env::var("INVARIANT_MINER_ENABLED").as_deref() == Ok("0") {
        eprintln!("[invariant-miner] disabled by INVARIANT_MINER_ENABLED=0");
        return;
    }
    tokio::spawn(run(project_dir));
}

async fn run(project_dir: PathBuf) {
    let startup_delay = env_u64(
        "INVARIANT_MINER_STARTUP_DELAY_SECS",
        DEFAULT_STARTUP_DELAY_SECS,
    );
    let interval_secs = env_u64("INVARIANT_MINER_INTERVAL_SECS", DEFAULT_INTERVAL_SECS).max(1);

    eprintln!(
        "[invariant-miner] starting  project_dir={}  interval={}s  startup_delay={}s  auto_promote={}",
        project_dir.display(),
        interval_secs,
        startup_delay,
        auto_promote_enabled()
    );

    if startup_delay > 0 {
        tokio::time::sleep(Duration::from_secs(startup_delay)).await;
    }

    loop {
        run_once(&project_dir);
        tokio::time::sleep(Duration::from_secs(interval_secs)).await;
    }
}

fn run_once(project_dir: &PathBuf) {
    let min_support = env_u64("INVARIANT_MIN_SUPPORT", DEFAULT_MIN_SUPPORT).max(1);

    match mine_and_write_workspace(project_dir) {
        Ok(candidates) => eprintln!(
            "[invariant-miner] mined candidates={}  project_dir={}",
            candidates.len(),
            project_dir.display()
        ),
        Err(err) => {
            eprintln!("[invariant-miner] mine failed: {err}");
            return;
        }
    }

    match validate_workspace(project_dir, min_support) {
        Ok(validations) => {
            let passed = validations
                .iter()
                .filter(|validation| validation.passed)
                .count();
            eprintln!(
                "[invariant-miner] validated total={} passed={} min_support={}",
                validations.len(),
                passed,
                min_support
            );
        }
        Err(err) => {
            eprintln!("[invariant-miner] validate failed: {err}");
            return;
        }
    }

    if auto_promote_enabled() {
        match promote_workspace(project_dir, min_support) {
            Ok(promoted) => eprintln!(
                "[invariant-miner] promoted count={} min_support={}",
                promoted.len(),
                min_support
            ),
            Err(err) => eprintln!("[invariant-miner] promote failed: {err}"),
        }
    }
}

fn auto_promote_enabled() -> bool {
    matches!(
        env::var("INVARIANT_AUTO_PROMOTE").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("YES")
    )
}

fn env_u64(name: &str, default: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}
