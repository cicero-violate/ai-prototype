//! Task runner daemon.
//!
//! Bridges task ownership to bounded execution:
//!   wait_for_task_ready_and_claim → run_with_heartbeat(LoopDriver) → evidence check → complete/fail
//!
//! This is the missing `W` activation path:
//!   T = plan_node -> claim -> W(executes loop/capability) -> evidence -> complete

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::process::agent::config::AgentLoopConfig;
use crate::process::agent::loop_driver::LoopDriver;
use crate::process::agent::worker::{
    complete_claim, fail_claim, run_with_heartbeat, wait_for_task_ready_and_claim,
};

const DEFAULT_LEASE_TTL_MS: u64 = 120_000;
const RETRY_AFTER_MS: u64 = 60_000;

pub struct TaskRunner {
    supervisor_url: String,
    worker_id: String,
    lease_ttl_ms: u64,
    base_config: AgentLoopConfig,
}

impl TaskRunner {
    pub fn new(supervisor_url: String, worker_id: String, base_config: AgentLoopConfig) -> Self {
        let lease_ttl_ms = std::env::var("TASK_RUNNER_LEASE_TTL_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_LEASE_TTL_MS);
        Self {
            supervisor_url,
            worker_id,
            lease_ttl_ms,
            base_config,
        }
    }

    /// Blocking event-wakeup execute-complete loop. Call from a dedicated thread.
    pub fn run(&self) {
        eprintln!(
            "[task_runner] start  worker_id={}  supervisor={}  lease_ttl={}ms",
            self.worker_id, self.supervisor_url, self.lease_ttl_ms
        );
        loop {
            let idempotency_key = now_ms();
            match wait_for_task_ready_and_claim(
                &self.supervisor_url,
                &self.base_config.project_dir,
                &self.worker_id,
                idempotency_key,
                self.lease_ttl_ms,
            ) {
                Err(e) => {
                    eprintln!("[task_runner] task_ready claim error: {e}");
                    continue;
                }
                Ok(None) => continue,
                Ok(Some((title, description, claim))) => {
                    let node_id = claim.node_id.clone();
                    eprintln!("[task_runner] claimed  node={node_id}");

                    let supervisor_url = self.supervisor_url.clone();
                    let worker_id = self.worker_id.clone();
                    let lease_ttl_ms = self.lease_ttl_ms;
                    let evidence_path = evidence_path_for(&self.base_config.project_dir, &node_id);

                    let mut config = self.base_config.clone();
                    config.domain = Some(title);
                    config.metric = Some(description);
                    config.plan_node_id = Some(node_id.clone());

                    let succeeded = run_with_heartbeat(
                        &supervisor_url,
                        &claim,
                        &worker_id,
                        lease_ttl_ms,
                        move || {
                            LoopDriver::new(config).run_all_agents();
                            evidence_path.exists()
                        },
                    );

                    if succeeded {
                        eprintln!("[task_runner] evidence present — completing  node={node_id}");
                        complete_claim(&supervisor_url, &claim, &worker_id);
                    } else {
                        eprintln!(
                            "[task_runner] evidence missing — failing with retry  node={node_id}"
                        );
                        fail_claim(&supervisor_url, &claim, &worker_id, RETRY_AFTER_MS);
                    }
                }
            }
        }
    }
}

fn evidence_path_for(project_dir: &PathBuf, node_id: &str) -> PathBuf {
    project_dir
        .join("state")
        .join("agent-evidence")
        .join(format!("{node_id}.md"))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
