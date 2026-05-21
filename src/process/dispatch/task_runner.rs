//! Event-driven task runner.
//!
//! Replaces polling GET /v1/task/next on a fixed 2-second timer with a
//! wakeup-driven model: the runner blocks on a `TaskReadyNotifier` and reacts
//! immediately when the supervisor signals a task is available. A 30-second
//! timeout fallback ensures missed signals are caught by replaying plan state.
//!
//! Execution path:
//!   TaskReady wakeup → try_claim_once → run_with_heartbeat(LoopDriver)
//!                    → successful run + evidence_path.exists() gate
//!                    → complete or fail with retry

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::process::agent::config::AgentLoopConfig;
use crate::process::agent::loop_driver::LoopDriver;
use crate::process::agent::worker::{complete_claim, fail_claim, run_with_heartbeat, ActiveClaim};
use crate::process::dispatch::task_client::TaskClient;
use crate::process::scheduler::handler::TaskReadyNotifier;

const DEFAULT_LEASE_TTL_MS: u64 = 120_000;
const RETRY_AFTER_MS: u64 = 60_000;
/// How long to wait for a signal before performing a replay-safety scan.
const REPLAY_FALLBACK_SECS: u64 = 30;

pub struct TaskRunner {
    supervisor_url: String,
    worker_id: String,
    lease_ttl_ms: u64,
    base_config: AgentLoopConfig,
    notifier: TaskReadyNotifier,
}

impl TaskRunner {
    pub fn new(
        supervisor_url: String,
        worker_id: String,
        base_config: AgentLoopConfig,
        notifier: TaskReadyNotifier,
    ) -> Self {
        let lease_ttl_ms = std::env::var("TASK_RUNNER_LEASE_TTL_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_LEASE_TTL_MS);
        Self {
            supervisor_url,
            worker_id,
            lease_ttl_ms,
            base_config,
            notifier,
        }
    }

    /// Event-driven execution loop. Blocks on `TaskReadyNotifier` instead of
    /// sleeping on a fixed interval. The 30-second fallback scan ensures
    /// correctness even when a signal is missed (e.g., process restart).
    pub fn run(&self) {
        eprintln!(
            "[task_runner] start event-driven  worker_id={}  supervisor={}  lease_ttl={}ms",
            self.worker_id, self.supervisor_url, self.lease_ttl_ms
        );
        // Initial scan: catch any tasks already pending when the runner starts.
        self.try_claim_once();
        loop {
            let signaled = self
                .notifier
                .wait_or_timeout(Duration::from_secs(REPLAY_FALLBACK_SECS));
            if !signaled {
                eprintln!(
                    "[task_runner] replay-safety scan  worker_id={}",
                    self.worker_id
                );
            }
            self.try_claim_once();
        }
    }

    /// Attempt to claim and execute exactly one ready task.
    /// Returns immediately if no task is available — the outer loop handles
    /// re-scheduling via the notifier or the timeout fallback.
    fn try_claim_once(&self) {
        let client = TaskClient::new(&self.supervisor_url);

        let assignment = match client.next() {
            Err(e) => {
                eprintln!(
                    "[task_runner] task/next error: {e}  worker={}",
                    self.worker_id
                );
                return;
            }
            Ok(None) => return,
            Ok(Some(assignment)) => assignment,
        };

        let idempotency_key = now_ms();
        let task_claim = match client.claim(
            &assignment.node_id,
            &self.worker_id,
            idempotency_key,
            self.lease_ttl_ms,
        ) {
            Ok(claim) => claim,
            Err(e) => {
                eprintln!(
                    "[task_runner] claim failed  node={}  err={e}",
                    assignment.node_id
                );
                return;
            }
        };

        let claim = ActiveClaim::from(task_claim);
        let node_id = claim.node_id.clone();
        eprintln!(
            "[task_runner] claimed  node={node_id}  worker={}",
            self.worker_id
        );

        let supervisor_url = self.supervisor_url.clone();
        let worker_id = self.worker_id.clone();
        let lease_ttl_ms = self.lease_ttl_ms;
        let evidence_path = evidence_path_for(&self.base_config.project_dir, &node_id);

        let mut config = self.base_config.clone();
        config.domain = Some(assignment.title);
        config.metric = Some(assignment.description);
        config.plan_node_id = Some(node_id.clone());

        let succeeded = run_with_heartbeat(
            &supervisor_url,
            &claim,
            &worker_id,
            lease_ttl_ms,
            move || {
                let run_completed = LoopDriver::new(config).run_all_agents();
                run_completed && evidence_path.exists()
            },
        );

        if succeeded {
            eprintln!("[task_runner] evidence present — completing  node={node_id}");
            complete_claim(&supervisor_url, &claim, &worker_id);
        } else {
            eprintln!("[task_runner] evidence missing — failing with retry  node={node_id}");
            fail_claim(&supervisor_url, &claim, &worker_id, RETRY_AFTER_MS);
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
