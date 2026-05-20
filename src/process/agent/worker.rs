//! Temporal-style event-wakeup worker.
//!
//! Subscribes to TaskReady wakeups reconstructed from the runtime event log,
//! atomically claims a node, heartbeats, dispatches the LLM capability, then
//! completes or fails.  This is the `W` in the architecture equation:
//! W = TaskReady wakeup + claim + lease + heartbeat + complete/fail + retry.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::codec::ndjson::load_tlog_ndjson;
use crate::domain::plan::{load_plan, ready_nodes_from_plan_state, PlanNode};
use crate::process::dispatch::task_client::{TaskClaim, TaskClient};
use crate::runtime::event_bus::{replay_event_bus, WakeupKind};
use crate::runtime::introspection::canonical_tlog_path_from_dir;
use crate::runtime::workspace::workspace_state_dir;

const TASK_READY_WAKE_SLEEP_MS: u64 = 2_000;

/// Opaque task ownership claim returned by the dispatch task client.
#[derive(Clone, Debug)]
pub struct ActiveClaim {
    pub node_id: String,
    pub claim_id: u64,
    pub expires_at_ms: u64,
    pub receipt_hash: u64,
    pub tlog_submitted: bool,
}

impl From<TaskClaim> for ActiveClaim {
    fn from(claim: TaskClaim) -> Self {
        Self {
            node_id: claim.node_id,
            claim_id: claim.claim_id,
            expires_at_ms: claim.expires_at_ms,
            receipt_hash: claim.receipt_hash,
            tlog_submitted: claim.tlog_submitted,
        }
    }
}

impl From<&ActiveClaim> for TaskClaim {
    fn from(claim: &ActiveClaim) -> Self {
        Self {
            node_id: claim.node_id.clone(),
            claim_id: claim.claim_id,
            expires_at_ms: claim.expires_at_ms,
            receipt_hash: claim.receipt_hash,
            tlog_submitted: claim.tlog_submitted,
        }
    }
}

/// Subscribe to TaskReady wakeups, replay persisted TLog on startup so missed
/// wakeups are recovered, then claim the next ready node idempotently.
pub fn wait_for_task_ready_and_claim(
    supervisor_url: &str,
    project_dir: &Path,
    worker_id: &str,
    idempotency_key: u64,
    lease_ttl_ms: u64,
) -> Result<Option<(String, String, ActiveClaim)>, String> {
    let client = TaskClient::new(supervisor_url);
    let mut wakeups = TaskReadyWakeups::subscribe(project_dir);
    loop {
        let Some(assignment) = wakeups.recv() else {
            continue;
        };

        match client.claim(
            &assignment.node_id,
            worker_id,
            idempotency_key,
            lease_ttl_ms,
        ) {
            Ok(claim) => {
                return Ok(Some((
                    assignment.title,
                    assignment.description,
                    ActiveClaim::from(claim),
                )));
            }
            Err(e) => {
                eprintln!(
                    "[worker] claim failed for TaskReady node={} ({e}) — awaiting next wakeup",
                    assignment.node_id
                );
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TaskReadyAssignment {
    node_id: String,
    title: String,
    description: String,
}

impl From<&PlanNode> for TaskReadyAssignment {
    fn from(node: &PlanNode) -> Self {
        Self {
            node_id: node.id.clone(),
            title: node.title.clone(),
            description: node.description.clone(),
        }
    }
}

/// Runtime event-bus subscription backed by replay of the canonical TLog.
///
/// On creation it replays the existing TLog and enqueues current ready plan
/// nodes for replayed TaskReady wakeups emitted while this worker was down.  On
/// each receive it rebuilds the disposable runtime event-bus projection from
/// TLog, consumes unseen TaskReady wakeups, and refreshes the ready-node
/// projection.  Claim remains the authority, so duplicate wakeups are harmless
/// and concurrent workers race idempotently at `/v1/task/claim`.
struct TaskReadyWakeups {
    project_dir: PathBuf,
    tlog_path: PathBuf,
    last_task_ready_seq: u64,
    pending: VecDeque<TaskReadyAssignment>,
}

impl TaskReadyWakeups {
    fn subscribe(project_dir: &Path) -> Self {
        let tlog_dir = std::env::var("AI_TLOG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| workspace_state_dir(project_dir).join("tlog"));
        let mut wakeups = Self {
            project_dir: project_dir.to_path_buf(),
            tlog_path: canonical_tlog_path_from_dir(&tlog_dir),
            last_task_ready_seq: 0,
            pending: VecDeque::new(),
        };
        wakeups.replay_missed_wakeups();
        wakeups
    }

    fn recv(&mut self) -> Option<TaskReadyAssignment> {
        loop {
            if let Some(next) = self.pending.pop_front() {
                return Some(next);
            }

            self.replay_missed_wakeups();
            if let Some(next) = self.pending.pop_front() {
                return Some(next);
            }

            thread::sleep(Duration::from_millis(TASK_READY_WAKE_SLEEP_MS));
        }
    }

    fn replay_missed_wakeups(&mut self) {
        let latest_task_ready_seq = latest_task_ready_wakeup_seq(&self.tlog_path).unwrap_or_else(|e| {
            eprintln!(
                "[worker] TaskReady event-bus replay unavailable at {}: {e}",
                self.tlog_path.display()
            );
            self.last_task_ready_seq
        });

        if latest_task_ready_seq == self.last_task_ready_seq && !self.pending.is_empty() {
            return;
        }

        if latest_task_ready_seq > self.last_task_ready_seq {
            self.last_task_ready_seq = latest_task_ready_seq;
            self.refresh_ready_projection();
        }
    }

    fn refresh_ready_projection(&mut self) {
        let plan = load_plan(&self.project_dir);
        self.pending = ready_nodes_from_plan_state(&plan)
            .into_iter()
            .map(TaskReadyAssignment::from)
            .collect();
    }
}

fn latest_task_ready_wakeup_seq(tlog_path: &Path) -> Result<u64, String> {
    if !tlog_path.exists() {
        return Ok(0);
    }
    let tlog = load_tlog_ndjson(tlog_path).map_err(|e| e.to_string())?;
    let bus = replay_event_bus(&tlog);
    Ok(bus
        .wakeups()
        .iter()
        .filter(|wakeup| wakeup.kind == WakeupKind::TaskReady)
        .map(|wakeup| wakeup.event_seq)
        .max()
        .unwrap_or(0))
}

/// Report task completion.
pub fn complete_claim(supervisor_url: &str, claim: &ActiveClaim, worker_id: &str) {
    let client = TaskClient::new(supervisor_url);
    match client.complete(&TaskClaim::from(claim), worker_id) {
        Ok(ack) => {
            eprintln!(
                "[worker] task complete  node={}  receipt_hash={}  tlog_submitted={}",
                claim.node_id, ack.receipt_hash, ack.tlog_submitted
            );
        }
        Err(e) => {
            eprintln!("[worker] complete failed: {e}  node={}", claim.node_id);
        }
    }
}

/// Renew the task lease before it expires.
///
/// Workers should call this every `lease_ttl_ms / 3` while capability work is
/// active so the supervisor knows the task is still being executed.
pub fn heartbeat_claim(
    supervisor_url: &str,
    claim: &ActiveClaim,
    worker_id: &str,
    lease_ttl_ms: u64,
) {
    let client = TaskClient::new(supervisor_url);
    match client.heartbeat(&TaskClaim::from(claim), worker_id, lease_ttl_ms) {
        Ok(heartbeat) => {
            eprintln!(
                "[worker] heartbeat ok  node={}  expires_at_ms={}  receipt_hash={}  tlog_submitted={}",
                claim.node_id, heartbeat.expires_at_ms, heartbeat.receipt_hash, heartbeat.tlog_submitted
            );
        }
        Err(e) => {
            eprintln!("[worker] heartbeat failed: {e}  node={}", claim.node_id);
        }
    }
}

/// Run capability work while a background heartbeat keeps the claim alive.
pub fn run_with_heartbeat<T, F>(
    supervisor_url: &str,
    claim: &ActiveClaim,
    worker_id: &str,
    lease_ttl_ms: u64,
    work: F,
) -> T
where
    F: FnOnce() -> T,
{
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let heartbeat_supervisor_url = supervisor_url.to_string();
    let heartbeat_claim_value = claim.clone();
    let heartbeat_worker_id = worker_id.to_string();
    let heartbeat_interval_ms = lease_ttl_ms.saturating_div(3).max(1);

    let heartbeat = thread::spawn(move || {
        while stop_rx
            .recv_timeout(Duration::from_millis(heartbeat_interval_ms))
            .is_err()
        {
            heartbeat_claim(
                &heartbeat_supervisor_url,
                &heartbeat_claim_value,
                &heartbeat_worker_id,
                lease_ttl_ms,
            );
        }
    });

    let result = work();
    let _ = stop_tx.send(());
    let _ = heartbeat.join();
    result
}

/// Report task failure.  retry_after_ms==0 → permanent failure; >0 → retry.
pub fn fail_claim(supervisor_url: &str, claim: &ActiveClaim, worker_id: &str, retry_after_ms: u64) {
    let client = TaskClient::new(supervisor_url);
    match client.fail(&TaskClaim::from(claim), worker_id, retry_after_ms) {
        Ok(ack) => {
            eprintln!(
                "[worker] task failed  node={}  retry_after_ms={retry_after_ms}  receipt_hash={}  tlog_submitted={}",
                claim.node_id, ack.receipt_hash, ack.tlog_submitted
            );
        }
        Err(e) => {
            eprintln!("[worker] fail call failed: {e}  node={}", claim.node_id);
        }
    }
}
