//! Event-driven scheduler wakeup handlers.
//!
//! These handlers are intentionally thin. They project scheduler wakeups from
//! durable TLog/control-event observations into lifecycle requests and delegate
//! all mutation to the process boundary that owns task leases and plan writes.
//! There is no timer polling in this module.

use crate::service::supervisor::process::{
    TaskClaimDto, TaskClaimRequest, TaskCompleteDto, TaskCompleteRequest, TaskFailDto,
    TaskFailRequest, WorkerProcess,
};
pub use crate::runtime::event_bus::WakeupKind as SchedulerWakeupKind;

/// A projected scheduler wakeup. The wakeup taxonomy comes from runtime event
/// projection; this adapter only carries process lifecycle context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchedulerWakeup {
    pub kind: SchedulerWakeupKind,
    pub source_seq: u64,
    pub node_id: String,
    pub worker_id: String,
    pub claim_id: u64,
    pub idempotency_key: u64,
    pub observed_at_ms: u64,
    pub lease_expires_at_ms: u64,
    pub lease_ttl_ms: Option<u64>,
    pub retry_after_ms: u64,
}

impl SchedulerWakeup {
    pub fn task_ready(
        source_seq: u64,
        node_id: impl Into<String>,
        worker_id: impl Into<String>,
        idempotency_key: u64,
        observed_at_ms: u64,
        lease_ttl_ms: Option<u64>,
    ) -> Self {
        Self {
            kind: SchedulerWakeupKind::TaskReady,
            source_seq,
            node_id: node_id.into(),
            worker_id: worker_id.into(),
            claim_id: 0,
            idempotency_key,
            observed_at_ms,
            lease_expires_at_ms: 0,
            lease_ttl_ms,
            retry_after_ms: 0,
        }
    }

    pub fn lease_expired(
        source_seq: u64,
        node_id: impl Into<String>,
        worker_id: impl Into<String>,
        claim_id: u64,
        observed_at_ms: u64,
        lease_expires_at_ms: u64,
        retry_after_ms: u64,
    ) -> Self {
        Self {
            kind: SchedulerWakeupKind::LeaseExpired,
            source_seq,
            node_id: node_id.into(),
            worker_id: worker_id.into(),
            claim_id,
            idempotency_key: 0,
            observed_at_ms,
            lease_expires_at_ms,
            lease_ttl_ms: None,
            retry_after_ms,
        }
    }

    pub fn receipt_accepted(
        source_seq: u64,
        node_id: impl Into<String>,
        worker_id: impl Into<String>,
        claim_id: u64,
        observed_at_ms: u64,
        lease_expires_at_ms: u64,
    ) -> Self {
        Self {
            kind: SchedulerWakeupKind::ReceiptAccepted,
            source_seq,
            node_id: node_id.into(),
            worker_id: worker_id.into(),
            claim_id,
            idempotency_key: 0,
            observed_at_ms,
            lease_expires_at_ms,
            lease_ttl_ms: None,
            retry_after_ms: 0,
        }
    }

    pub fn gate_failed(
        source_seq: u64,
        node_id: impl Into<String>,
        worker_id: impl Into<String>,
        claim_id: u64,
        observed_at_ms: u64,
        lease_expires_at_ms: u64,
        retry_after_ms: u64,
    ) -> Self {
        Self {
            kind: SchedulerWakeupKind::GateFailed,
            source_seq,
            node_id: node_id.into(),
            worker_id: worker_id.into(),
            claim_id,
            idempotency_key: 0,
            observed_at_ms,
            lease_expires_at_ms,
            lease_ttl_ms: None,
            retry_after_ms,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SchedulerHandlerEffect {
    Claimed(TaskClaimDto),
    Completed(TaskCompleteDto),
    Failed(TaskFailDto),
    Ignored(SchedulerHandlerIgnore),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchedulerHandlerIgnore {
    WrongWakeupKind,
    MissingClaim,
    LeaseStillActive,
    StaleLease,
    AlreadyApplied,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SchedulerHandlerError {
    Boundary(String),
}

impl From<String> for SchedulerHandlerError {
    fn from(value: String) -> Self {
        if is_idempotent_already_applied(&value) {
            Self::Boundary("already-applied".to_string())
        } else {
            Self::Boundary(value)
        }
    }
}

/// Boundary used by scheduler handlers. Implementations own all lease checks,
/// plan writes, and lifecycle receipts.
pub trait SchedulerLifecycleBoundary {
    fn claim_from_task_ready(&mut self, req: TaskClaimRequest) -> Result<TaskClaimDto, String>;
    fn complete_from_receipt(
        &mut self,
        req: TaskCompleteRequest,
    ) -> Result<TaskCompleteDto, String>;
    fn fail_from_wakeup(&mut self, req: TaskFailRequest) -> Result<TaskFailDto, String>;
}

impl SchedulerLifecycleBoundary for WorkerProcess {
    fn claim_from_task_ready(&mut self, req: TaskClaimRequest) -> Result<TaskClaimDto, String> {
        self.claim_task(req)
    }

    fn complete_from_receipt(
        &mut self,
        req: TaskCompleteRequest,
    ) -> Result<TaskCompleteDto, String> {
        self.complete_task(req)
    }

    fn fail_from_wakeup(&mut self, req: TaskFailRequest) -> Result<TaskFailDto, String> {
        self.fail_task(req)
    }
}

/// Dispatch one projected wakeup to its typed handler.
pub fn handle_scheduler_wakeup(
    boundary: &mut impl SchedulerLifecycleBoundary,
    wakeup: &SchedulerWakeup,
) -> Result<SchedulerHandlerEffect, SchedulerHandlerError> {
    match wakeup.kind {
        SchedulerWakeupKind::TaskReady => handle_task_ready(boundary, wakeup),
        SchedulerWakeupKind::LeaseExpired => handle_lease_expired(boundary, wakeup),
        SchedulerWakeupKind::ReceiptAccepted => handle_receipt_accepted(boundary, wakeup),
        SchedulerWakeupKind::GateFailed => handle_gate_failed(boundary, wakeup),
        SchedulerWakeupKind::EvalVerdict | SchedulerWakeupKind::LearningCandidate => Ok(
            SchedulerHandlerEffect::Ignored(SchedulerHandlerIgnore::WrongWakeupKind),
        ),
    }
}

/// Claim a TLog-projected ready task. Idempotency is delegated to the supervisor
/// boundary via `(node_id, worker_id, idempotency_key)`.
pub fn handle_task_ready(
    boundary: &mut impl SchedulerLifecycleBoundary,
    wakeup: &SchedulerWakeup,
) -> Result<SchedulerHandlerEffect, SchedulerHandlerError> {
    if wakeup.kind != SchedulerWakeupKind::TaskReady {
        return Ok(SchedulerHandlerEffect::Ignored(
            SchedulerHandlerIgnore::WrongWakeupKind,
        ));
    }

    let dto = boundary
        .claim_from_task_ready(TaskClaimRequest {
            node_id: wakeup.node_id.clone(),
            worker_id: wakeup.worker_id.clone(),
            idempotency_key: wakeup.idempotency_key,
            lease_ttl_ms: wakeup.lease_ttl_ms,
        })
        .map_err(SchedulerHandlerError::from)?;
    Ok(SchedulerHandlerEffect::Claimed(dto))
}

/// Convert an event-projected lease expiry into a retry/failure through the
/// supervisor boundary. The observed event time must be at or past the lease
/// expiry to avoid replacing heartbeat semantics with timer polling.
pub fn handle_lease_expired(
    boundary: &mut impl SchedulerLifecycleBoundary,
    wakeup: &SchedulerWakeup,
) -> Result<SchedulerHandlerEffect, SchedulerHandlerError> {
    if wakeup.kind != SchedulerWakeupKind::LeaseExpired {
        return Ok(SchedulerHandlerEffect::Ignored(
            SchedulerHandlerIgnore::WrongWakeupKind,
        ));
    }
    if wakeup.claim_id == 0 {
        return Ok(SchedulerHandlerEffect::Ignored(
            SchedulerHandlerIgnore::MissingClaim,
        ));
    }
    if wakeup.lease_expires_at_ms == 0 || wakeup.observed_at_ms < wakeup.lease_expires_at_ms {
        return Ok(SchedulerHandlerEffect::Ignored(
            SchedulerHandlerIgnore::LeaseStillActive,
        ));
    }

    delegate_failure(boundary, wakeup)
}

/// Complete a task only after an accepted execution receipt wakeup and only
/// while the projected lease is still current.
pub fn handle_receipt_accepted(
    boundary: &mut impl SchedulerLifecycleBoundary,
    wakeup: &SchedulerWakeup,
) -> Result<SchedulerHandlerEffect, SchedulerHandlerError> {
    if wakeup.kind != SchedulerWakeupKind::ReceiptAccepted {
        return Ok(SchedulerHandlerEffect::Ignored(
            SchedulerHandlerIgnore::WrongWakeupKind,
        ));
    }
    if wakeup.claim_id == 0 {
        return Ok(SchedulerHandlerEffect::Ignored(
            SchedulerHandlerIgnore::MissingClaim,
        ));
    }
    if wakeup.lease_expires_at_ms != 0 && wakeup.observed_at_ms > wakeup.lease_expires_at_ms {
        return Ok(SchedulerHandlerEffect::Ignored(
            SchedulerHandlerIgnore::StaleLease,
        ));
    }

    match boundary.complete_from_receipt(TaskCompleteRequest {
        node_id: wakeup.node_id.clone(),
        worker_id: wakeup.worker_id.clone(),
        claim_id: wakeup.claim_id,
    }) {
        Ok(dto) => Ok(SchedulerHandlerEffect::Completed(dto)),
        Err(error) if is_idempotent_already_applied(&error) => Ok(SchedulerHandlerEffect::Ignored(
            SchedulerHandlerIgnore::AlreadyApplied,
        )),
        Err(error) => Err(SchedulerHandlerError::Boundary(error)),
    }
}

/// Fail a leased task when the gate projection reports failure. A still-active
/// lease is required; a stale event is ignored to keep retries deterministic.
pub fn handle_gate_failed(
    boundary: &mut impl SchedulerLifecycleBoundary,
    wakeup: &SchedulerWakeup,
) -> Result<SchedulerHandlerEffect, SchedulerHandlerError> {
    if wakeup.kind != SchedulerWakeupKind::GateFailed {
        return Ok(SchedulerHandlerEffect::Ignored(
            SchedulerHandlerIgnore::WrongWakeupKind,
        ));
    }
    if wakeup.claim_id == 0 {
        return Ok(SchedulerHandlerEffect::Ignored(
            SchedulerHandlerIgnore::MissingClaim,
        ));
    }
    if wakeup.lease_expires_at_ms != 0 && wakeup.observed_at_ms > wakeup.lease_expires_at_ms {
        return Ok(SchedulerHandlerEffect::Ignored(
            SchedulerHandlerIgnore::StaleLease,
        ));
    }

    delegate_failure(boundary, wakeup)
}

fn delegate_failure(
    boundary: &mut impl SchedulerLifecycleBoundary,
    wakeup: &SchedulerWakeup,
) -> Result<SchedulerHandlerEffect, SchedulerHandlerError> {
    match boundary.fail_from_wakeup(TaskFailRequest {
        node_id: wakeup.node_id.clone(),
        worker_id: wakeup.worker_id.clone(),
        claim_id: wakeup.claim_id,
        retry_after_ms: wakeup.retry_after_ms,
    }) {
        Ok(dto) => Ok(SchedulerHandlerEffect::Failed(dto)),
        Err(error) if is_idempotent_already_applied(&error) => Ok(SchedulerHandlerEffect::Ignored(
            SchedulerHandlerIgnore::AlreadyApplied,
        )),
        Err(error) => Err(SchedulerHandlerError::Boundary(error)),
    }
}

fn is_idempotent_already_applied(error: &str) -> bool {
    error.contains("no active lease") || error.contains("is not pending")
}

// ── TaskReadyNotifier ─────────────────────────────────────────────────────────

use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// Shared signal between the supervisor (task source) and task runners (consumers).
///
/// The supervisor calls `notify()` when a node transitions to Pending — on retry,
/// on startup reset, or on plan update. Task runners block on `wait_or_timeout`
/// instead of sleeping on a fixed timer, so they react immediately when work arrives.
///
/// This is a process-level disposable projection. If a signal is missed the
/// 30-second timeout fallback in `wait_or_timeout` reconstructs any pending work.
/// Truth is always the supervisor's plan.json + TLog; the notifier is wakeup-only.
#[derive(Clone)]
pub struct TaskReadyNotifier {
    inner: Arc<(Mutex<bool>, Condvar)>,
}

impl Default for TaskReadyNotifier {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskReadyNotifier {
    pub fn new() -> Self {
        Self {
            inner: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    /// Signal that one or more tasks may be ready to claim.
    /// Idempotent: N calls before a drain collapse to a single check.
    pub fn notify(&self) {
        let (lock, condvar) = &*self.inner;
        *lock.lock().unwrap() = true;
        condvar.notify_all();
    }

    /// Block until notified or until `timeout` elapses (replay-safety fallback).
    /// Returns `true` if a signal arrived, `false` if the timeout elapsed.
    pub fn wait_or_timeout(&self, timeout: Duration) -> bool {
        let (lock, condvar) = &*self.inner;
        let guard = lock.lock().unwrap();
        let (mut guard, _) = condvar.wait_timeout(guard, timeout).unwrap();
        let was_notified = *guard;
        *guard = false;
        was_notified
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct BoundaryStub {
        claim: usize,
        complete: usize,
        fail: usize,
        complete_error: Option<String>,
    }

    impl SchedulerLifecycleBoundary for BoundaryStub {
        fn claim_from_task_ready(&mut self, req: TaskClaimRequest) -> Result<TaskClaimDto, String> {
            self.claim += 1;
            Ok(TaskClaimDto {
                ok: true,
                node_id: req.node_id,
                claim_id: 7,
                expires_at_ms: 2_000,
                receipt_hash: 70,
                tlog_submitted: true,
            })
        }

        fn complete_from_receipt(
            &mut self,
            _req: TaskCompleteRequest,
        ) -> Result<TaskCompleteDto, String> {
            self.complete += 1;
            if let Some(error) = self.complete_error.clone() {
                return Err(error);
            }
            Ok(TaskCompleteDto {
                ok: true,
                receipt_hash: 80,
                tlog_submitted: true,
            })
        }

        fn fail_from_wakeup(&mut self, _req: TaskFailRequest) -> Result<TaskFailDto, String> {
            self.fail += 1;
            Ok(TaskFailDto {
                ok: true,
                receipt_hash: 90,
                tlog_submitted: true,
            })
        }
    }

    #[test]
    fn task_ready_claims_through_boundary() {
        let mut boundary = BoundaryStub::default();
        let wakeup = SchedulerWakeup::task_ready(1, "n1", "w1", 44, 1_000, Some(500));
        let effect = handle_task_ready(&mut boundary, &wakeup).expect("task ready handled");
        assert!(matches!(effect, SchedulerHandlerEffect::Claimed(_)));
        assert_eq!(boundary.claim, 1);
    }

    #[test]
    fn receipt_accepted_ignores_stale_lease() {
        let mut boundary = BoundaryStub::default();
        let wakeup = SchedulerWakeup::receipt_accepted(1, "n1", "w1", 7, 2_001, 2_000);
        let effect = handle_receipt_accepted(&mut boundary, &wakeup).expect("stale ignored");
        assert_eq!(
            effect,
            SchedulerHandlerEffect::Ignored(SchedulerHandlerIgnore::StaleLease)
        );
        assert_eq!(boundary.complete, 0);
    }

    #[test]
    fn lease_expired_fails_after_expiry_only() {
        let mut boundary = BoundaryStub::default();
        let early = SchedulerWakeup::lease_expired(1, "n1", "w1", 7, 1_999, 2_000, 300);
        let effect = handle_lease_expired(&mut boundary, &early).expect("active ignored");
        assert_eq!(
            effect,
            SchedulerHandlerEffect::Ignored(SchedulerHandlerIgnore::LeaseStillActive)
        );
        assert_eq!(boundary.fail, 0);

        let expired = SchedulerWakeup::lease_expired(2, "n1", "w1", 7, 2_000, 2_000, 300);
        let effect = handle_lease_expired(&mut boundary, &expired).expect("expired handled");
        assert!(matches!(effect, SchedulerHandlerEffect::Failed(_)));
        assert_eq!(boundary.fail, 1);
    }

    #[test]
    fn repeated_completion_is_idempotent() {
        let mut boundary = BoundaryStub {
            complete_error: Some("no active lease for node n1".to_string()),
            ..BoundaryStub::default()
        };
        let wakeup = SchedulerWakeup::receipt_accepted(1, "n1", "w1", 7, 1_000, 2_000);
        let effect = handle_receipt_accepted(&mut boundary, &wakeup).expect("repeat ignored");
        assert_eq!(
            effect,
            SchedulerHandlerEffect::Ignored(SchedulerHandlerIgnore::AlreadyApplied)
        );
    }
}
