//! Supervisor worker-process lifecycle and agent spawning.
//!
//! This belongs to the outer API layer because it owns OS process lifecycle,
//! HTTP health probing, and bridge calls into the agent loop driver.

use std::collections::HashMap;
use std::env;
use std::net::TcpListener as StdTcpListener;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::process::{Child, Command as TokioCommand};

use crate::api::protocol::{Command as KernelCommand, CommandEnvelope};
use crate::capability::orchestration::TaskLifecycleReceipt;
use crate::domain::plan::{
    plan_text_hash, NodeStatus, PlanEvidenceRef, EVIDENCE_GATE_EXECUTION,
    EVIDENCE_KIND_EXECUTION_RECEIPT, EVIDENCE_TYPE_EXECUTION_RECEIPT,
};
use crate::kernel::{mix, PlanEvidenceProjection};
use crate::process::agent::loop_driver::http::post_json_local;
use crate::process::agent::{
    AgentLoopConfig, LoopDriver, DEFAULT_EXECUTOR_COUNT, MAX_EXECUTOR_COUNT,
};
use crate::process::scheduler::plan_store::{
    append_evidence_patch, append_node_remove_patch, append_status_change_patch, load_plan,
    load_plan_read_model, load_tlog_projected_plan_state,
};
use crate::runtime::workspace::workspace_state_dir;

const DEFAULT_LEASE_TTL_MS: u64 = 300_000;

fn current_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub struct WorkerProcess {
    active: Option<WorkerInstance>,
    retired: Vec<RetiredWorker>,
    next_generation: u64,
    binary_path: PathBuf,
    tlog_dir: PathBuf,
    mcp_worker_url: String,
    drain_after: Duration,
    project_dir: PathBuf,
    mcp_connector_url: String,
    supervisor_port: u16,
    next_spawn: u64,
    /// Active task leases keyed by node_id.
    task_leases: HashMap<String, TaskLease>,
    next_claim_id: u64,
}

impl WorkerProcess {
    pub fn new(
        binary_path: PathBuf,
        tlog_dir: PathBuf,
        mcp_worker_url: String,
        _router_url: String,
        project_dir: PathBuf,
        mcp_connector_url: String,
        supervisor_port: u16,
    ) -> Self {
        Self {
            active: None,
            retired: Vec::new(),
            next_generation: 1,
            binary_path,
            tlog_dir,
            mcp_worker_url,
            drain_after: Duration::from_secs(30),
            project_dir,
            mcp_connector_url,
            supervisor_port,
            next_spawn: 0,
            task_leases: HashMap::new(),
            next_claim_id: 1,
        }
    }

    pub async fn health(&mut self) -> Result<HealthDto, String> {
        self.reap_retired().await;
        let Some(active) = self.active.as_mut() else {
            return Err("no active worker".to_string());
        };
        if let Some(status) = active
            .child
            .try_wait()
            .map_err(|err| format!("worker wait failed: {err}"))?
        {
            let generation = active.generation;
            self.active = None;
            return Err(format!(
                "active worker generation {generation} exited with {status}"
            ));
        }
        Ok(HealthDto {
            ok: true,
            generation: active.generation,
            worker_port: active.port,
        })
    }

    pub async fn reload_inner(&mut self) -> Result<ReloadDto, String> {
        self.reap_retired().await;
        let generation = self.next_generation;
        eprintln!("supervisor: starting worker generation {generation}");
        let instance = spawn_worker(
            &self.binary_path,
            generation,
            &self.tlog_dir,
            &self.mcp_worker_url,
        )
        .await?;
        eprintln!(
            "supervisor: worker generation {generation} ready on port {}",
            instance.port
        );
        self.next_generation = self
            .next_generation
            .checked_add(1)
            .ok_or_else(|| "generation overflow".to_string())?;
        if let Some(old) = self.active.take() {
            eprintln!("supervisor: retiring worker generation {}", old.generation);
            self.retired.push(RetiredWorker {
                instance: old,
                retired_at: Instant::now(),
            });
        }
        let active = ActiveWorkerDto {
            generation: instance.generation,
            worker_port: instance.port,
        };
        self.active = Some(instance);
        Ok(ReloadDto { ok: true, active })
    }

    pub async fn reap_retired(&mut self) {
        let now = Instant::now();
        let mut survivors = Vec::new();
        for mut retired in self.retired.drain(..) {
            if now.duration_since(retired.retired_at) >= self.drain_after {
                let _ = retired.instance.child.kill().await;
                let _ = retired.instance.child.wait().await;
            } else {
                survivors.push(retired);
            }
        }
        self.retired = survivors;
    }

    pub async fn shutdown(&mut self) {
        if let Some(mut active) = self.active.take() {
            let _ = active.child.kill().await;
            let _ = active.child.wait().await;
        }
        for mut retired in self.retired.drain(..) {
            let _ = retired.instance.child.kill().await;
            let _ = retired.instance.child.wait().await;
        }
    }

    pub fn spawn_agent(
        &mut self,
        domain: &str,
        metric: &str,
        max_steps: u64,
    ) -> Result<SpawnDto, String> {
        let worker_port = self
            .active
            .as_ref()
            .map(|w| w.port)
            .ok_or_else(|| "no active worker — call /reload first".to_string())?;

        let n = self.next_spawn;
        self.next_spawn += 1;
        let spawn_id = format!("agent-{n}");

        let project_dir = self.project_dir.clone();
        let mcp_connector_url = self.mcp_connector_url.clone();
        let execute_turns = max_steps.clamp(1, 100) as u32;
        let sse_chunks_dir = workspace_state_dir(&project_dir)
            .join("agent_state")
            .join("sse-chunks");
        let config = AgentLoopConfig {
            execute_turns,
            turn_retry_limit: 2,
            loop_sleep_ms: 5000,
            agent_count: 1,
            executor_count: 1,
            working_dir: project_dir.clone(),
            sse_chunks_dir,
            project_dir,
            mcp_connector_url,
            router_turn_max_ms: 600_000,
            router_first_capture_ms: 60_000,
            router_idle_ms: 2_500,
            worker_port: Some(worker_port),
            supervisor_port: Some(self.supervisor_port),
            browser_router_url: std::env::var("CANON_OPENAI_BASE_URL").ok().map(|url| {
                let url = url.trim_end_matches('/').to_string();
                url.strip_suffix("/v1").unwrap_or(&url).to_string()
            }),
            cert_max_steps: 30,
            domain: Some(domain.to_string()),
            metric: Some(metric.to_string()),
            plan_node_id: None,
        };

        eprintln!(
            "supervisor: spawning agent {spawn_id}  domain={domain:?}  metric={metric:?}  worker_port={worker_port}"
        );
        let sid = spawn_id.clone();
        std::thread::spawn(move || {
            LoopDriver::new(config).run_all_agents();
            eprintln!("supervisor: agent {sid} finished");
        });

        Ok(SpawnDto {
            ok: true,
            spawn_id,
            pid: std::process::id(),
            domain: domain.to_string(),
            metric: metric.to_string(),
            worker_port,
        })
    }

    pub fn start_main_loop(&mut self, req: StartLoopRequest) -> Result<StartLoopDto, String> {
        let worker_port = self
            .active
            .as_ref()
            .map(|w| w.port)
            .ok_or_else(|| "no active worker — call /reload first".to_string())?;

        let project_dir = req
            .working_dir
            .map(PathBuf::from)
            .unwrap_or_else(|| self.project_dir.clone());
        let mcp_connector_url = self.mcp_connector_url.clone();
        let execute_turns: u32 = req.execute_turns.unwrap_or_else(|| {
            env::var("EXECUTE_TURNS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2)
        });
        let agent_count: u32 = req.agent_count.unwrap_or_else(|| {
            env::var("AGENT_COUNT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1)
        });
        let executor_count: u32 = req
            .executor_count
            .or_else(|| {
                env::var("CANON_EXECUTOR_COUNT")
                    .ok()
                    .and_then(|v| v.parse().ok())
            })
            .unwrap_or(DEFAULT_EXECUTOR_COUNT)
            .clamp(1, MAX_EXECUTOR_COUNT);
        let sse_chunks_dir = workspace_state_dir(&project_dir)
            .join("agent_state")
            .join("sse-chunks");
        let config = AgentLoopConfig {
            execute_turns,
            turn_retry_limit: 2,
            loop_sleep_ms: 5000,
            agent_count,
            executor_count,
            working_dir: project_dir.clone(),
            sse_chunks_dir,
            project_dir,
            mcp_connector_url,
            router_turn_max_ms: 600_000,
            router_first_capture_ms: 60_000,
            router_idle_ms: 2_500,
            worker_port: Some(worker_port),
            supervisor_port: Some(self.supervisor_port),
            browser_router_url: std::env::var("CANON_OPENAI_BASE_URL").ok().map(|url| {
                let url = url.trim_end_matches('/').to_string();
                url.strip_suffix("/v1").unwrap_or(&url).to_string()
            }),
            cert_max_steps: 30,
            domain: None,
            metric: None,
            plan_node_id: None,
        };

        eprintln!("supervisor: starting main agent loop  execute_turns={execute_turns}  agents={agent_count}  mini_agents={executor_count}  worker_port={worker_port}");
        std::thread::spawn(move || {
            LoopDriver::new(config).run_all_agents();
            eprintln!("supervisor: main agent loop finished");
        });

        Ok(StartLoopDto {
            ok: true,
            worker_port,
            execute_turns,
            agent_count,
            executor_count,
        })
    }

    /// Claim a ready plan node for a worker.  Atomic under the supervisor Mutex.
    pub fn claim_task(&mut self, req: TaskClaimRequest) -> Result<TaskClaimDto, String> {
        let now_ms = current_ms();
        self.task_leases.retain(|_, l| l.expires_at_ms > now_ms);

        // Idempotency: same worker + same key on the same node → re-use existing claim.
        if let Some(existing) = self.task_leases.get(&req.node_id) {
            if existing.worker_id == req.worker_id
                && existing.idempotency_key == req.idempotency_key
            {
                let receipt = TaskLifecycleReceipt::claim(
                    &existing.node_id,
                    &existing.worker_id,
                    existing.claim_id,
                    existing.idempotency_key,
                    existing.expires_at_ms,
                    now_ms,
                );
                let tlog_submitted = self.submit_task_lifecycle_receipt(receipt, "claim");
                return Ok(TaskClaimDto {
                    ok: true,
                    node_id: req.node_id,
                    claim_id: existing.claim_id,
                    expires_at_ms: existing.expires_at_ms,
                    receipt_hash: receipt.receipt_hash,
                    tlog_submitted,
                });
            }
            return Err(format!("node {} is already claimed", req.node_id));
        }

        let (plan, _) = load_plan_read_model(&self.project_dir)
            .unwrap_or_else(|_| (load_plan(&self.project_dir), None));
        let node = plan
            .nodes
            .iter()
            .find(|n| n.id == req.node_id)
            .ok_or_else(|| format!("node {} not found in plan", req.node_id))?;

        if node.status != NodeStatus::Pending {
            return Err(format!(
                "node {} is not pending (current status is not claimable)",
                req.node_id
            ));
        }

        let claim_id = self.next_claim_id;
        self.next_claim_id = self.next_claim_id.saturating_add(1).max(1);

        let lease_ttl_ms = req.lease_ttl_ms.unwrap_or(DEFAULT_LEASE_TTL_MS);
        let expires_at_ms = now_ms.saturating_add(lease_ttl_ms);

        append_status_change_patch(&self.project_dir, &req.node_id, &NodeStatus::Running)
            .map_err(|e| format!("append task claim plan patch failed: {e}"))?;

        self.task_leases.insert(
            req.node_id.clone(),
            TaskLease {
                node_id: req.node_id.clone(),
                worker_id: req.worker_id.clone(),
                claim_id,
                idempotency_key: req.idempotency_key,
                expires_at_ms,
            },
        );

        let receipt = TaskLifecycleReceipt::claim(
            &req.node_id,
            &req.worker_id,
            claim_id,
            req.idempotency_key,
            expires_at_ms,
            now_ms,
        );
        let tlog_submitted = self.submit_task_lifecycle_receipt(receipt, "claim");

        eprintln!(
            "supervisor: task claimed  node={}  worker={}  claim_id={}  expires_at_ms={}",
            req.node_id, req.worker_id, claim_id, expires_at_ms
        );
        Ok(TaskClaimDto {
            ok: true,
            node_id: req.node_id.clone(),
            claim_id,
            expires_at_ms,
            receipt_hash: receipt.receipt_hash,
            tlog_submitted,
        })
    }

    /// Renew the lease for an active claim.
    pub fn heartbeat_task(
        &mut self,
        req: TaskHeartbeatRequest,
    ) -> Result<TaskHeartbeatDto, String> {
        let now_ms = current_ms();

        let lease = self
            .task_leases
            .get_mut(&req.node_id)
            .ok_or_else(|| format!("no active lease for node {}", req.node_id))?;

        if lease.claim_id != req.claim_id {
            return Err(format!("claim_id mismatch for node {}", req.node_id));
        }
        if lease.worker_id != req.worker_id {
            return Err(format!("worker_id mismatch for node {}", req.node_id));
        }
        if lease.expires_at_ms <= now_ms {
            self.task_leases.remove(&req.node_id);
            return Err(format!("lease for node {} has expired", req.node_id));
        }

        let lease_ttl_ms = req.lease_ttl_ms.unwrap_or(DEFAULT_LEASE_TTL_MS);
        lease.expires_at_ms = now_ms.saturating_add(lease_ttl_ms);
        let expires_at_ms = lease.expires_at_ms;

        let receipt = TaskLifecycleReceipt::heartbeat(
            &req.node_id,
            &req.worker_id,
            req.claim_id,
            expires_at_ms,
            now_ms,
        );
        let tlog_submitted = self.submit_task_lifecycle_receipt(receipt, "heartbeat");

        Ok(TaskHeartbeatDto {
            ok: true,
            expires_at_ms,
            receipt_hash: receipt.receipt_hash,
            tlog_submitted,
        })
    }

    /// Mark a claimed task as successfully completed.
    pub fn complete_task(&mut self, req: TaskCompleteRequest) -> Result<TaskCompleteDto, String> {
        let now_ms = current_ms();

        let lease = self
            .task_leases
            .get(&req.node_id)
            .ok_or_else(|| format!("no active lease for node {}", req.node_id))?;

        if lease.claim_id != req.claim_id {
            return Err(format!("claim_id mismatch for node {}", req.node_id));
        }
        if lease.worker_id != req.worker_id {
            return Err(format!("worker_id mismatch for node {}", req.node_id));
        }
        if lease.expires_at_ms <= now_ms {
            self.task_leases.remove(&req.node_id);
            return Err(format!("lease for node {} has expired", req.node_id));
        }

        let lease_expires_at_ms = lease.expires_at_ms;
        let evidence_hash = projected_node_evidence_hash(&self.project_dir, &req.node_id)
            .ok_or_else(|| format!("node {} not found in plan", req.node_id))
            .and_then(|hash| {
                if hash == 0 {
                    Err(format!(
                        "node {} cannot be completed without projected task evidence",
                        req.node_id
                    ))
                } else {
                    Ok(hash)
                }
            })?;

        let receipt = TaskLifecycleReceipt::complete(
            &req.node_id,
            &req.worker_id,
            req.claim_id,
            lease_expires_at_ms,
            now_ms,
            evidence_hash,
        );
        if !receipt.is_contract_valid() {
            return Err(format!(
                "invalid completion receipt for node {}",
                req.node_id
            ));
        }
        let tlog_submitted = self.submit_task_lifecycle_receipt(receipt, "complete");
        if self.active.is_some() && !tlog_submitted {
            return Err(format!(
                "node {} cannot be completed because completion receipt was not accepted",
                req.node_id
            ));
        }

        let execution_evidence = completion_execution_evidence(&req.node_id, receipt.receipt_hash);
        attach_supervisor_execution_evidence(&self.project_dir, &req.node_id, &execution_evidence)
            .map_err(|e| format!("append task completion evidence patch failed: {e}"))?;
        if !has_projected_evidence_ref(&self.project_dir, &req.node_id, &execution_evidence) {
            return Err(format!(
                "node {} cannot be completed without accepted execution receipt evidence",
                req.node_id
            ));
        }

        append_status_change_patch(&self.project_dir, &req.node_id, &NodeStatus::Done)
            .map_err(|e| format!("append task completion plan patch failed: {e}"))?;
        self.task_leases.remove(&req.node_id);

        eprintln!(
            "supervisor: task completed  node={}  worker={}  claim_id={}",
            req.node_id, req.worker_id, req.claim_id
        );
        Ok(TaskCompleteDto {
            ok: true,
            receipt_hash: receipt.receipt_hash,
            tlog_submitted,
        })
    }

    /// Mark a claimed task as failed, optionally scheduling a retry.
    /// `retry_after_ms == 0` → set status to Failed (no retry).
    /// `retry_after_ms > 0`  → reset status to Pending so it can be re-claimed.
    pub fn fail_task(&mut self, req: TaskFailRequest) -> Result<TaskFailDto, String> {
        // Allow failing even with a mismatched/expired lease so workers can
        // always report failure (idempotent failure path).
        if let Some(lease) = self.task_leases.get(&req.node_id) {
            if lease.claim_id != req.claim_id {
                return Err(format!("claim_id mismatch for node {}", req.node_id));
            }
            if lease.worker_id != req.worker_id {
                return Err(format!("worker_id mismatch for node {}", req.node_id));
            }
        }
        let status = if req.retry_after_ms == 0 {
            NodeStatus::Failed
        } else {
            NodeStatus::Pending
        };
        append_status_change_patch(&self.project_dir, &req.node_id, &status)
            .map_err(|e| format!("append task failure plan patch failed: {e}"))?;
        self.task_leases.remove(&req.node_id);

        let receipt = TaskLifecycleReceipt::fail(
            &req.node_id,
            &req.worker_id,
            req.claim_id,
            current_ms(),
            req.retry_after_ms,
        );
        let tlog_submitted = self.submit_task_lifecycle_receipt(receipt, "fail");

        eprintln!(
            "supervisor: task failed  node={}  worker={}  claim_id={}  retry_after_ms={}",
            req.node_id, req.worker_id, req.claim_id, req.retry_after_ms
        );
        Ok(TaskFailDto {
            ok: true,
            receipt_hash: receipt.receipt_hash,
            tlog_submitted,
        })
    }

    /// On supervisor startup the in-memory lease table is empty, so any node
    /// persisted as Running in plan.json has no valid lease and is stale.
    /// Reset all such nodes to Pending so the task runner can re-claim them.
    pub fn reset_stale_running_nodes(&self) {
        let (plan, _) = load_plan_read_model(&self.project_dir)
            .unwrap_or_else(|_| (load_plan(&self.project_dir), None));
        let mut changed = 0usize;
        for node in &plan.nodes {
            if node.status == NodeStatus::Running {
                eprintln!(
                    "supervisor: stale running node reset to pending  node={}  was_assignee={:?}",
                    node.id, node.assignee
                );
                match append_status_change_patch(&self.project_dir, &node.id, &NodeStatus::Pending)
                {
                    Ok(()) => changed += 1,
                    Err(e) => eprintln!(
                        "supervisor: reset_stale_running_nodes append patch failed for {}: {e}",
                        node.id
                    ),
                }
            }
        }
        if changed > 0 {
            eprintln!("supervisor: reset {changed} stale running node(s) to pending");
        }
    }

    /// Scan the plan read-model for nodes that have terminal evidence in the TLog
    /// but whose status has not been promoted yet.
    ///
    /// Two cases are closed:
    /// - Any non-terminal node with a `blocker` evidence entry → Failed.
    ///   Workers that hit an unresolvable condition write a blocker and stop
    ///   without sending a completion callback, leaving the node stuck.
    /// - Any non-terminal node with an accepted execution-receipt evidence entry
    ///   → Done. Guards against missed completion callbacks (e.g. worker crash
    ///   after appending evidence but before the HTTP POST returned).
    pub fn reconcile_terminal_evidence(&self) {
        let (plan, _) = load_plan_read_model(&self.project_dir)
            .unwrap_or_else(|_| (load_plan(&self.project_dir), None));

        let mut promoted_done = 0usize;
        let mut promoted_failed = 0usize;

        for node in &plan.nodes {
            if matches!(
                node.status,
                NodeStatus::Done | NodeStatus::Failed | NodeStatus::Skipped
            ) {
                continue;
            }

            let has_blocker = node.evidence.iter().any(|e| e.is_blocker());
            let has_accepted_receipt = node
                .evidence
                .iter()
                .any(|e| e.is_accepted_execution_receipt());

            // Accepted receipt takes precedence over a blocker if somehow both exist.
            if has_accepted_receipt {
                eprintln!(
                    "supervisor: reconcile — promoting {} to Done (accepted receipt evidence found)",
                    node.id
                );
                match append_status_change_patch(&self.project_dir, &node.id, &NodeStatus::Done) {
                    Ok(()) => {
                        self.archive_reconciled_terminal_node(&node.id);
                        promoted_done += 1;
                    }
                    Err(e) => eprintln!(
                        "supervisor: reconcile Done patch failed for {}: {e}",
                        node.id
                    ),
                }
            } else if has_blocker {
                eprintln!(
                    "supervisor: reconcile — promoting {} to Failed (blocker evidence found)",
                    node.id
                );
                match append_status_change_patch(&self.project_dir, &node.id, &NodeStatus::Failed) {
                    Ok(()) => {
                        self.archive_reconciled_terminal_node(&node.id);
                        promoted_failed += 1;
                    }
                    Err(e) => eprintln!(
                        "supervisor: reconcile Failed patch failed for {}: {e}",
                        node.id
                    ),
                }
            }
        }

        if promoted_done + promoted_failed > 0 {
            eprintln!(
                "supervisor: reconcile complete — done={promoted_done} failed={promoted_failed}"
            );
        }
    }

    fn archive_reconciled_terminal_node(&self, node_id: &str) {
        if let Err(err) = append_node_remove_patch(&self.project_dir, node_id) {
            eprintln!("supervisor: reconcile node remove failed for {node_id}: {err}");
        }
    }

    /// Node IDs with non-expired leases (used by GET /v1/task/next to filter).
    pub fn active_claimed_node_ids(&self) -> Vec<String> {
        let now_ms = current_ms();
        self.task_leases
            .iter()
            .filter(|(_, l)| l.expires_at_ms > now_ms)
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn active_worker_port(&self) -> Result<u16, String> {
        self.active
            .as_ref()
            .map(|worker| worker.port)
            .ok_or_else(|| "no active worker — call /reload first".to_string())
    }

    /// Returns the active worker port, auto-reloading if the worker has exited.
    pub async fn ensure_worker_alive_or_reload(&mut self) -> Result<u16, String> {
        if let Some(active) = self.active.as_mut() {
            if let Ok(Some(status)) = active.child.try_wait() {
                let generation = active.generation;
                self.active = None;
                eprintln!(
                    "supervisor: active worker generation {generation} exited with {status}, auto-reloading"
                );
            }
        }
        if self.active.is_none() {
            self.reload_inner().await?;
        } else {
            self.reap_retired().await;
        }
        self.active_worker_port()
    }

    pub fn active_generation(&self) -> Option<u64> {
        self.active.as_ref().map(|worker| worker.generation)
    }

    pub fn project_dir(&self) -> &std::path::Path {
        &self.project_dir
    }

    fn submit_task_lifecycle_receipt(&self, receipt: TaskLifecycleReceipt, label: &str) -> bool {
        if !receipt.is_contract_valid() {
            return false;
        }
        let Some(worker_port) = self.active.as_ref().map(|worker| worker.port) else {
            eprintln!(
                "supervisor: task lifecycle receipt not submitted  label={label}  receipt_hash={}  reason=no-active-worker",
                receipt.receipt_hash
            );
            return false;
        };

        let submission = receipt.submission();
        let command = KernelCommand::SubmitEvidence(submission);
        let envelope = CommandEnvelope::new(receipt.receipt_hash, command);
        let url = format!("http://127.0.0.1:{worker_port}/v1/command");
        let payload = serde_json::json!({
            "command_id": envelope.command_id,
            "command_hash": envelope.command_hash,
            "payload_tag": "SubmitEvidence",
            "payload": {
                "gate": "Execution",
                "evidence": "ExecutionReceipt",
                "passed": submission.passed,
                "effect": "None",
                "payload_hash": submission.payload_hash,
            },
            "source": "supervisor_task_lifecycle",
        });

        match post_json_local(&url, &payload) {
            Ok(status) if (200..300).contains(&status) => {
                eprintln!(
                    "supervisor: task lifecycle receipt submitted  label={label}  receipt_hash={}  status={status}",
                    receipt.receipt_hash
                );
                true
            }
            Ok(status) => {
                eprintln!(
                    "supervisor: task lifecycle receipt submit failed  label={label}  receipt_hash={}  status={status}",
                    receipt.receipt_hash
                );
                false
            }
            Err(error) => {
                eprintln!(
                    "supervisor: task lifecycle receipt submit error  label={label}  receipt_hash={}  error={error}",
                    receipt.receipt_hash
                );
                false
            }
        }
    }
}

struct WorkerInstance {
    child: Child,
    port: u16,
    generation: u64,
}

struct RetiredWorker {
    instance: WorkerInstance,
    retired_at: Instant,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct HealthDto {
    pub ok: bool,
    pub generation: u64,
    pub worker_port: u16,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ReloadDto {
    pub ok: bool,
    pub active: ActiveWorkerDto,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct RestartDto {
    pub ok: bool,
    pub pid: u32,
    pub replacement: String,
    pub delay_ms: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ActiveWorkerDto {
    pub generation: u64,
    pub worker_port: u16,
}

#[derive(Debug, Deserialize)]
pub struct SpawnRequest {
    pub domain: String,
    pub metric: String,
    pub max_steps: Option<u64>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct SpawnDto {
    pub ok: bool,
    pub spawn_id: String,
    pub pid: u32,
    pub domain: String,
    pub metric: String,
    pub worker_port: u16,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct StartLoopRequest {
    /// Override working/project directory (default: supervisor's PROJECT_DIR).
    pub working_dir: Option<String>,
    /// Execute turns per cycle (default: EXECUTE_TURNS env or 2).
    pub execute_turns: Option<u32>,
    /// Parallel agents (default: AGENT_COUNT env or 1).
    pub agent_count: Option<u32>,
    /// Parallel bounded DAG mini-agents per scheduling wave (default: 3, max: 5).
    pub executor_count: Option<u32>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct StartLoopDto {
    pub ok: bool,
    pub worker_port: u16,
    pub execute_turns: u32,
    pub agent_count: u32,
    pub executor_count: u32,
}

/// A single dequeued task returned by GET /v1/task/next.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct TaskNextDto {
    pub ok: bool,
    pub node_id: String,
    pub title: String,
    pub description: String,
    pub ready_count: usize,
}

/// Summary counts returned by GET /v1/plan/status.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct PlanStatusDto {
    pub ok: bool,
    pub pending: usize,
    pub running: usize,
    pub done: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total: usize,
    pub ready: usize,
}

// ── Task lifecycle (claim / heartbeat / complete / fail) ─────────────────────

/// In-memory lease record held by the supervisor under its Mutex.
pub struct TaskLease {
    pub node_id: String,
    pub worker_id: String,
    pub claim_id: u64,
    pub idempotency_key: u64,
    pub expires_at_ms: u64,
}

/// POST /v1/task/claim request body.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct TaskClaimRequest {
    pub node_id: String,
    pub worker_id: String,
    /// Client-chosen key: same (node_id, worker_id, idempotency_key) → idempotent re-claim.
    #[serde(default)]
    pub idempotency_key: u64,
    /// Lease duration in milliseconds (default: 300 000 = 5 min).
    pub lease_ttl_ms: Option<u64>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct TaskClaimDto {
    pub ok: bool,
    pub node_id: String,
    pub claim_id: u64,
    pub expires_at_ms: u64,
    pub receipt_hash: u64,
    pub tlog_submitted: bool,
}

/// POST /v1/task/heartbeat request body.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct TaskHeartbeatRequest {
    pub node_id: String,
    pub worker_id: String,
    pub claim_id: u64,
    /// New TTL from now (default: 300 000 ms).
    pub lease_ttl_ms: Option<u64>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct TaskHeartbeatDto {
    pub ok: bool,
    pub expires_at_ms: u64,
    pub receipt_hash: u64,
    pub tlog_submitted: bool,
}

/// POST /v1/task/complete request body.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct TaskCompleteRequest {
    pub node_id: String,
    pub worker_id: String,
    pub claim_id: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct TaskCompleteDto {
    pub ok: bool,
    pub receipt_hash: u64,
    pub tlog_submitted: bool,
}

/// POST /v1/task/fail request body.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct TaskFailRequest {
    pub node_id: String,
    pub worker_id: String,
    pub claim_id: u64,
    /// 0 → mark Failed permanently; >0 → reset to Pending for retry.
    #[serde(default)]
    pub retry_after_ms: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct TaskFailDto {
    pub ok: bool,
    pub receipt_hash: u64,
    pub tlog_submitted: bool,
}

fn plan_evidence_refs_hash(evidence: &[PlanEvidenceRef]) -> u64 {
    let mut h = 0xe11d_eace_4fd0_0101u64;
    h = mix(h, evidence.len() as u64);
    for item in evidence {
        h = mix(h, string_hash(&item.path));
        h = mix(h, string_hash(&item.kind));
        h = mix(h, string_hash(&item.summary));
        h = mix(h, string_hash(&item.gate));
        h = mix(h, string_hash(&item.evidence));
        h = mix(h, item.receipt_hash);
        h = mix(h, u64::from(item.accepted));
    }
    h.max(1)
}

fn projected_node_evidence_hash(project_dir: &Path, node_id: &str) -> Option<u64> {
    if let Ok(Some(plan_state)) = load_tlog_projected_plan_state(project_dir) {
        return plan_state
            .nodes
            .get(&plan_text_hash(node_id))
            .map(|node| plan_projected_evidence_hash(&node.evidence));
    }

    let plan = load_plan(project_dir);
    plan.nodes
        .into_iter()
        .find(|node| node.id == node_id)
        .map(|node| plan_evidence_refs_hash(&node.evidence))
}

fn plan_projected_evidence_hash(evidence: &[PlanEvidenceProjection]) -> u64 {
    if evidence.is_empty() {
        return 0;
    }
    let mut h = 0xe11d_eace_4fd0_0201u64;
    h = mix(h, evidence.len() as u64);
    for item in evidence {
        h = mix(h, item.path_hash);
        h = mix(h, item.kind_hash);
        h = mix(h, item.summary_hash);
    }
    h.max(1)
}

fn has_projected_evidence_ref(
    project_dir: &Path,
    node_id: &str,
    evidence: &PlanEvidenceRef,
) -> bool {
    let Ok(Some(plan_state)) = load_tlog_projected_plan_state(project_dir) else {
        return false;
    };
    let Some(node) = plan_state.nodes.get(&plan_text_hash(node_id)) else {
        return false;
    };
    let expected = PlanEvidenceProjection {
        path_hash: plan_text_hash(&evidence.path),
        kind_hash: plan_text_hash(&evidence.kind),
        summary_hash: plan_text_hash(&evidence.summary),
    };
    node.evidence.contains(&expected)
}

fn completion_execution_evidence(node_id: &str, receipt_hash: u64) -> PlanEvidenceRef {
    PlanEvidenceRef {
        path: format!("state/agent-evidence/{node_id}.md"),
        kind: EVIDENCE_KIND_EXECUTION_RECEIPT.to_string(),
        summary: format!("Supervisor accepted completion ExecutionReceipt {receipt_hash}."),
        gate: EVIDENCE_GATE_EXECUTION.to_string(),
        evidence: EVIDENCE_TYPE_EXECUTION_RECEIPT.to_string(),
        receipt_hash,
        accepted: true,
    }
}

fn attach_supervisor_execution_evidence(
    project_dir: &Path,
    node_id: &str,
    evidence: &PlanEvidenceRef,
) -> Result<(), String> {
    let plan = load_plan(project_dir);
    plan.nodes
        .iter()
        .find(|node| node.id == node_id)
        .ok_or_else(|| format!("node {node_id} not found in plan"))?;
    append_evidence_patch(
        project_dir,
        node_id,
        &evidence.path,
        &evidence.kind,
        &evidence.summary,
    )
}

fn string_hash(value: &str) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in value.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h.max(1)
}

async fn spawn_worker(
    binary_path: &PathBuf,
    generation: u64,
    tlog_dir: &PathBuf,
    mcp_worker_url: &str,
) -> Result<WorkerInstance, String> {
    ensure_worker_binary(binary_path).await?;
    let port = allocate_port()?;
    let health_deadline = worker_health_deadline();
    eprintln!(
        "supervisor: spawning worker generation {generation}  bin={}  port={}  health_deadline={}s",
        binary_path.display(),
        port,
        health_deadline.as_secs()
    );
    let mut child = TokioCommand::new(binary_path)
        .env("AI_WORKER_MODE", "1")
        .env("PORT", port.to_string())
        .env("AI_TLOG_DIR", tlog_dir)
        .env("AI_MCP_WORKER_URL", mcp_worker_url)
        .env("AI_WORKER_GENERATION", generation.to_string())
        .kill_on_drop(true)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|err| format!("spawn worker failed: {err}"))?;

    if let Err(err) = wait_for_health(port, health_deadline, &mut child).await {
        let _ = child.kill().await;
        let _ = child.wait().await;
        return Err(err);
    }

    Ok(WorkerInstance {
        child,
        port,
        generation,
    })
}

async fn ensure_worker_binary(binary_path: &Path) -> Result<(), String> {
    if binary_path.exists() {
        return Ok(());
    }

    eprintln!(
        "supervisor: worker binary missing at {}; building it",
        binary_path.display()
    );
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_manifest = manifest_dir
        .parent()
        .ok_or_else(|| "CARGO_MANIFEST_DIR has no workspace parent".to_string())?
        .join("Cargo.toml");

    let mut command = TokioCommand::new("cargo");
    command
        .arg("build")
        .arg("--manifest-path")
        .arg(&workspace_manifest)
        .arg("--bin")
        .arg("kernel_tlog")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if binary_path
        .components()
        .any(|component| component.as_os_str() == "release")
    {
        command.arg("--release");
    }
    let status = command
        .status()
        .await
        .map_err(|err| format!("build worker binary failed to spawn: {err}"))?;
    if !status.success() {
        return Err(format!("build worker binary exited with {status}"));
    }
    binary_path
        .exists()
        .then_some(())
        .ok_or_else(|| format!("built worker binary not found at {}", binary_path.display()))
}

fn allocate_port() -> Result<u16, String> {
    let listener = StdTcpListener::bind("127.0.0.1:0")
        .map_err(|err| format!("allocate worker port failed: {err}"))?;
    listener
        .local_addr()
        .map(|addr| addr.port())
        .map_err(|err| format!("read allocated worker port failed: {err}"))
}

fn worker_health_deadline() -> Duration {
    env::var("AI_WORKER_HEALTH_DEADLINE_SECS")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .filter(|seconds| *seconds > 0)
        .map(Duration::from_secs)
        .unwrap_or_else(|| Duration::from_secs(60))
}

async fn wait_for_health(
    port: u16,
    health_deadline: Duration,
    child: &mut Child,
) -> Result<(), String> {
    let url = format!("http://127.0.0.1:{port}/health/worker");
    let deadline = Instant::now() + health_deadline;
    while Instant::now() < deadline {
        if let Some(status) = child
            .try_wait()
            .map_err(|err| format!("worker wait failed during health probe: {err}"))?
        {
            return Err(format!(
                "worker exited before health became ready on port {port}: {status}"
            ));
        }
        match reqwest::get(&url).await {
            Ok(response) if response.status().is_success() => return Ok(()),
            _ => tokio::time::sleep(Duration::from_millis(100)).await,
        }
    }
    Err(format!(
        "worker health deadline expired after {}s for port {port}",
        health_deadline.as_secs()
    ))
}
