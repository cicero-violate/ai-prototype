//! Task lifecycle helpers: heartbeat, complete, fail, run_with_heartbeat.
//!
//! `TaskRunner` (process/dispatch/task_runner.rs) is the live claim loop.
//! This module owns the per-claim lifecycle primitives it calls.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::service::dispatch::task_client::{TaskClaim, TaskClient};

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
