//! Event-driven task runner.
//!
//! Uses a wakeup-driven model: the runner blocks on a `TaskReadyNotifier` and
//! reacts immediately when the runtime event projection signals TaskReady. A
//! timeout fallback replays the durable plan/TLog read model so missed
//! disposable wakeups are recovered.
//!
//! Execution path:
//!   TaskReady wakeup → replay ready candidates → idempotent claim
//!                    → run_with_heartbeat(LoopDriver)
//!                    → successful run + evidence_path.exists() gate
//!                    → complete or fail with retry

use std::time::Duration;

use crate::domain::plan::{ready_nodes_from_available_plan_state, PlanNode};
use crate::process::agent::config::AgentLoopConfig;
use crate::process::agent::loop_driver::LoopDriver;
use crate::process::agent::worker::{complete_claim, fail_claim, run_with_heartbeat, ActiveClaim};
use crate::process::dispatch::task_client::TaskClient;
use crate::process::scheduler::handler::TaskReadyNotifier;
use crate::process::scheduler::plan_store::{load_plan, load_plan_read_model};

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
    /// sleeping on a fixed interval. The fallback replay scan ensures
    /// correctness even when a disposable signal is missed (e.g., process restart).
    pub fn run(&self) {
        eprintln!(
            "[task_runner] start event-driven  worker_id={}  supervisor={}  lease_ttl={}ms",
            self.worker_id, self.supervisor_url, self.lease_ttl_ms
        );
        // Initial replay scan: catch tasks already pending when the runner starts.
        self.try_claim_once();
        loop {
            let signaled = self
                .notifier
                .wait_or_timeout(Duration::from_secs(REPLAY_FALLBACK_SECS));
            if !signaled {
                eprintln!(
                    "[task_runner] replay-safety scan from plan/TLog  worker_id={}",
                    self.worker_id
                );
            }
            self.try_claim_once();
        }
    }

    /// Attempt to claim and execute exactly one replay-derived ready task.
    /// Returns immediately if no task is available — the outer loop handles
    /// re-scheduling via the notifier or the timeout fallback.
    fn try_claim_once(&self) {
        let client = TaskClient::new(&self.supervisor_url);
        let ready = self.ready_candidates_from_replay();
        if ready.is_empty() {
            return;
        }

        let mut last_claim_error = None;
        for assignment in ready {
            let idempotency_key = idempotency_key_for(&self.worker_id, &assignment.id);
            let task_claim = match client.claim(
                &assignment.id,
                &self.worker_id,
                idempotency_key,
                self.lease_ttl_ms,
            ) {
                Ok(claim) => claim,
                Err(e) => {
                    eprintln!(
                        "[task_runner] claim skipped  node={}  err={e}",
                        assignment.id
                    );
                    last_claim_error = Some(e);
                    continue;
                }
            };

            self.run_claimed_task(assignment, ActiveClaim::from(task_claim));
            return;
        }

        if let Some(e) = last_claim_error {
            eprintln!(
                "[task_runner] no replay-derived ready task claimed  worker={}  last_err={e}",
                self.worker_id
            );
        }
    }

    fn ready_candidates_from_replay(&self) -> Vec<PlanNode> {
        match load_plan_read_model(&self.base_config.project_dir) {
            Ok((plan, projection)) => {
                ready_nodes_from_available_plan_state(&plan, projection.as_ref())
                    .into_iter()
                    .cloned()
                    .collect()
            }
            Err(e) => {
                eprintln!(
                    "[task_runner] plan/TLog replay read-model failed; falling back to plan.json  worker={}  err={e}",
                    self.worker_id
                );
                let plan = load_plan(&self.base_config.project_dir);
                ready_nodes_from_available_plan_state(&plan, None)
                    .into_iter()
                    .cloned()
                    .collect()
            }
        }
    }

    fn run_claimed_task(&self, assignment: PlanNode, claim: ActiveClaim) {
        let node_id = claim.node_id.clone();
        eprintln!(
            "[task_runner] claimed  node={node_id}  worker={}",
            self.worker_id
        );

        let supervisor_url = self.supervisor_url.clone();
        let worker_id = self.worker_id.clone();
        let lease_ttl_ms = self.lease_ttl_ms;
        let mut config = self.base_config.clone();
        config.domain = Some(assignment.title);
        config.metric = Some(assignment.description);
        config.plan_node_id = Some(node_id.clone());

        let succeeded = run_with_heartbeat(
            &supervisor_url,
            &claim,
            &worker_id,
            lease_ttl_ms,
            move || LoopDriver::new(config).run_all_agents(),
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


fn idempotency_key_for(worker_id: &str, node_id: &str) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in worker_id.bytes().chain([0xff]).chain(node_id.bytes()) {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h.max(1)
}
