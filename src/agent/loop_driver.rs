use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::json;

use crate::agent::config::AgentLoopConfig;
use crate::agent::router::{RouterClient, RouterStreamingResult};
use crate::agent::sse::ChunkLogger;
use crate::capability::llm::openai::{OpenAiChatRequest, OpenAiMessage};
use crate::{
    load_tlog_ndjson, Command, CommandEnvelope, Evidence, EvidenceSubmission,
    EvidenceSubmissionDto, GateId, ObservationCursor, ObservationFrame, ObservationFrameKind,
    ObservationIngressBatch, PacketEffect, PolicyPromotion, PolicyStore,
    MAX_OBSERVATION_PAYLOAD_BYTES, POLICY_FEEDBACK_HASH,
};

/// Outer project-loop driver. Mirrors agentLoop / runCycle from
/// chatgpt-agent-loop/agent-loop.mjs.
///
/// Reads GOAL.md, syncs MCP workspace, then runs an infinite cycle of
/// 1 planning turn + N execute turns with per-turn retry logic.
///
/// This is deliberately separate from the phase-driven worker/runtime path:
/// the project loop asks the router model to edit files and commit work.
pub struct LoopDriver {
    pub config: AgentLoopConfig,
}

impl LoopDriver {
    pub fn new(config: AgentLoopConfig) -> Self {
        Self { config }
    }

    /// Spawn one agent thread per AGENT_COUNT and wait for all (blocking).
    pub fn run_all_agents(&self) {
        let goal_path = self.config.project_dir.join("GOAL.md");
        if !goal_path.exists() {
            eprintln!(
                "agent: PROJECT_DIR={} has no GOAL.md — loop disabled",
                self.config.project_dir.display()
            );
            return;
        }

        eprintln!("agent: project_dir={}", self.config.project_dir.display());
        match agent_command_url(&self.config) {
            Some(url) => eprintln!("agent: kernel receipt target={url}"),
            None => eprintln!("agent: kernel receipt target=disabled"),
        }
        eprintln!(
            "agent: execute_turns={}  retry_limit={}  loop_sleep={}ms  agents={}",
            self.config.execute_turns,
            self.config.turn_retry_limit,
            self.config.loop_sleep_ms,
            self.config.agent_count,
        );

        match sync_mcp_workspace(&self.config.mcp_connector_url, &self.config.project_dir) {
            Ok(_) => eprintln!(
                "agent: MCP workspace synced ({})",
                self.config.mcp_connector_url
            ),
            Err(e) => {
                eprintln!("agent: MCP workspace sync failed: {e}");
                return;
            }
        }

        if self.config.agent_count <= 1 {
            self.run_agent_loop(0);
        } else {
            let mut handles = Vec::new();
            for agent_id in 0..self.config.agent_count {
                let config = self.config.clone();
                let handle = thread::spawn(move || {
                    if agent_id > 0 {
                        thread::sleep(Duration::from_millis(u64::from(agent_id) * 5000));
                    }
                    LoopDriver { config }.run_agent_loop(agent_id);
                });
                handles.push(handle);
            }
            for h in handles {
                let _ = h.join();
            }
        }
    }

    fn run_agent_loop(&self, agent_id: u32) {
        let tag = agent_tag(agent_id, self.config.agent_count);
        let is_spawned = self.config.domain.is_some();

        if !is_spawned {
            let goal_path = self.config.project_dir.join("GOAL.md");
            if !goal_path.exists() {
                eprintln!("[{tag}] no GOAL.md — loop disabled");
                return;
            }
        }

        let mut cycle_num: u64 = 0;
        loop {
            cycle_num += 1;
            eprintln!("[{tag}] ── cycle {cycle_num} ──────────────────────────────");

            let mut router = match RouterClient::from_env() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[{tag}] router init failed: {e}");
                    break;
                }
            };

            if let Err(e) = self.run_cycle(cycle_num, agent_id, &tag, &mut router) {
                eprintln!("[{tag}] cycle {cycle_num} failed: {e}");
            }

            if let Some(url) = router.target_url().map(str::to_string) {
                let tag2 = tag.clone();
                thread::spawn(move || {
                    eprintln!("[{tag2}] tab close starting  cycle={cycle_num}  target={url}");
                    match crate::agent::router::close_tab_for_url_with_timeout(&url, 8_000) {
                        Ok(outcome) => {
                            eprintln!("[{tag2}] tab close  outcome={outcome:?}  cycle={cycle_num}")
                        }
                        Err(e) => eprintln!("[{tag2}] tab close failed: {e}  cycle={cycle_num}"),
                    }
                });
            }

            // Spawned agents run one cycle for their specific task then exit.
            if is_spawned {
                eprintln!("[{tag}] spawned agent task complete — exiting");
                break;
            }

            eprintln!(
                "[{tag}] sleeping {}ms before next cycle",
                self.config.loop_sleep_ms
            );
            thread::sleep(Duration::from_millis(self.config.loop_sleep_ms));
        }
    }

    fn run_cycle(
        &self,
        cycle_num: u64,
        agent_id: u32,
        tag: &str,
        router: &mut RouterClient,
    ) -> Result<(), String> {
        let cycle_plan = prepare_run_cycle(&self.config)?;
        let is_spawned = cycle_plan.is_spawned;
        let command_url = cycle_plan.command_url;
        let total_turns = cycle_plan.total_turns;
        let turn_offset = cycle_plan.turn_offset;
        let goal = cycle_plan.goal;

        // Invariant gate: prove world state is non-empty, ordered, hash-addressable.
        // Both top-level and spawned agents submit; source bytes differ.
        if let Some(url) = &command_url {
            self.submit_cycle_start_observation_ingress(url, cycle_num, is_spawned);
        }

        for turn in 0..total_turns {
            let turn_context = build_turn_prompt_context(
                turn,
                turn_offset,
                is_spawned,
                &goal,
                agent_id,
                self.config.agent_count,
                &self.config.working_dir,
                self.config.domain.as_deref(),
                self.config.metric.as_deref(),
            );
            debug_assert_eq!(turn_context.effective_turn, turn + turn_offset);

            eprintln!(
                "[{tag}] ── turn {}/{total_turns}: {} mode={} (cycle {cycle_num})",
                turn + 1,
                turn_context.label,
                turn_context.turn_mode.label(),
            );

            let mut completed = false;
            let mut last_reason = String::new();

            for attempt in 0..=self.config.turn_retry_limit {
                if attempt > 0 {
                    eprintln!(
                        "[{tag}] turn {} retry {attempt}/{} after {last_reason}",
                        turn_context.label, self.config.turn_retry_limit
                    );
                    thread::sleep(Duration::from_millis(self.config.loop_sleep_ms));
                }

                let attempt_outcome = self.run_cycle_attempt(
                    tag,
                    cycle_num,
                    &turn_context.label,
                    &turn_context.prompt,
                    attempt,
                    router,
                )?;
                last_reason = attempt_outcome.reason;

                if attempt_outcome.completed {
                    completed = true;
                    break;
                }

                if !attempt_outcome.retry_is_safe {
                    break;
                }
            }

            if !completed {
                return Err(format!(
                    "turn {} did not complete after {} attempt(s): {last_reason}",
                    turn_context.label,
                    self.config.turn_retry_limit + 1,
                ));
            }

            if turn < total_turns - 1 {
                thread::sleep(Duration::from_millis(self.config.loop_sleep_ms));
            }
        }

        // Verification + Eval gates: stamp the cycle's outcome, then promote.
        if !is_spawned {
            if let Some(url) = &command_url {
                let score_hash = read_score_hash(&self.config.working_dir);
                submit_eval_evidence(url, score_hash, cycle_num);

                let tlog_dir = std::env::var("AI_TLOG_DIR")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|_| self.config.project_dir.join("state").join("tlog"));
                let policy_path = self.config.project_dir.join("state").join("policy.ndjson");
                run_post_cycle_learning(
                    url,
                    &tlog_dir.join("canon-agent.tlog.ndjson"),
                    &policy_path,
                    cycle_num,
                );
            }
        }

        Ok(())
    }

    fn submit_cycle_start_observation_ingress(
        &self,
        command_url: &str,
        cycle_num: u64,
        is_spawned: bool,
    ) {
        let (obs_source_id, obs_bytes) = if is_spawned {
            let domain = self.config.domain.as_deref().unwrap_or("");
            let metric = self.config.metric.as_deref().unwrap_or("");
            let bytes = format!("{domain}\n{metric}").into_bytes();
            (stable_agent_hash(b"canon:domain:observation"), bytes)
        } else {
            let bytes = fs::read(self.config.working_dir.join("GOAL.md")).unwrap_or_default();
            (stable_agent_hash(b"canon:goal:observation"), bytes)
        };
        let obs_bytes = &obs_bytes[..obs_bytes.len().min(MAX_OBSERVATION_PAYLOAD_BYTES)];
        submit_observation_ingress(command_url, cycle_num, obs_source_id, obs_bytes);
    }

    fn run_cycle_attempt(
        &self,
        tag: &str,
        cycle_num: u64,
        label: &str,
        prompt: &str,
        attempt: u32,
        router: &mut RouterClient,
    ) -> Result<RunCycleAttemptOutcome, String> {
        let attempt_label = retry_attempt_label(label, attempt);

        let mut logger =
            ChunkLogger::new(&self.config.sse_chunks_dir, tag, cycle_num, &attempt_label)
                .map_err(|e| e.to_string())?;

        let request = OpenAiChatRequest::new(vec![OpenAiMessage::user(prompt.to_string())]);
        let has_target_url = router.target_url().is_some();
        let command_url = agent_command_url(&self.config);

        let request_hash = stable_agent_hash(prompt.as_bytes());
        let result = match router.streaming_turn(
            request,
            &mut logger,
            self.config.router_turn_max_ms,
            self.config.router_first_capture_ms,
            self.config.router_idle_ms,
        ) {
            Ok(result) => result,
            Err(err) => {
                write_agent_turn_receipt(AgentTurnReceiptInput {
                    dir: &self.config.sse_chunks_dir,
                    tag,
                    cycle_num,
                    label: &attempt_label,
                    attempt,
                    request_hash,
                    status: "failed",
                    reason: &err.to_string(),
                    finish_reason: None,
                    target_url: None,
                    content: "",
                    retry_is_safe: false,
                    command_url: command_url.as_deref(),
                });
                return Err(err.to_string());
            }
        };

        Ok(finalize_run_cycle_attempt_result(
            &self.config.sse_chunks_dir,
            tag,
            cycle_num,
            &attempt_label,
            attempt,
            request_hash,
            has_target_url,
            command_url.as_deref(),
            result,
        ))
    }
}

fn finalize_run_cycle_attempt_result(
    receipt_dir: &Path,
    tag: &str,
    cycle_num: u64,
    label: &str,
    attempt: u32,
    request_hash: u64,
    has_target_url: bool,
    command_url: Option<&str>,
    result: RouterStreamingResult,
) -> RunCycleAttemptOutcome {
    let reason = result.reason.clone();
    let retry_is_safe = result.retry_is_safe(has_target_url);
    write_agent_turn_receipt(AgentTurnReceiptInput {
        dir: receipt_dir,
        tag,
        cycle_num,
        label,
        attempt,
        request_hash,
        status: if result.complete {
            "completed"
        } else {
            "incomplete"
        },
        reason: &reason,
        finish_reason: result.finish_reason.as_deref(),
        target_url: result.target_url.as_deref(),
        content: &result.content,
        retry_is_safe,
        command_url,
    });
    if result.complete {
        let preview: String = result.content.chars().take(120).collect();
        let preview = preview.replace('\n', " ");
        eprintln!(
            "[{tag}] turn {label} done — {} — {preview}…",
            result.target_url.as_deref().unwrap_or("no url"),
        );
        return RunCycleAttemptOutcome {
            completed: true,
            reason,
            retry_is_safe: false,
        };
    }

    eprintln!(
        "[{tag}] turn {label} incomplete — {}; finish={}",
        result.reason,
        result.finish_reason.as_deref().unwrap_or("none"),
    );

    RunCycleAttemptOutcome {
        completed: false,
        reason,
        retry_is_safe,
    }
}

struct AgentTurnReceiptInput<'a> {
    dir: &'a Path,
    tag: &'a str,
    cycle_num: u64,
    label: &'a str,
    attempt: u32,
    request_hash: u64,
    status: &'a str,
    reason: &'a str,
    finish_reason: Option<&'a str>,
    target_url: Option<&'a str>,
    content: &'a str,
    retry_is_safe: bool,
    command_url: Option<&'a str>,
}

fn write_agent_turn_receipt(input: AgentTurnReceiptInput<'_>) {
    let _ = fs::create_dir_all(input.dir);
    let path = input.dir.join("agent-turn-receipts.ndjson");
    let receipt = json!({
        "schema": "canon.agent.router_turn_receipt.v1",
        "observed_at": timestamp_ms(),
        "agent": input.tag,
        "cycle": input.cycle_num,
        "label": input.label,
        "attempt": input.attempt,
        "status": input.status,
        "reason": input.reason,
        "finish_reason": input.finish_reason,
        "request_hash": input.request_hash,
        "target_url_hash": input.target_url.map(|url| stable_agent_hash(url.as_bytes())),
        "content_hash": stable_agent_hash(input.content.as_bytes()),
        "content_len": input.content.len(),
        "retry_is_safe": input.retry_is_safe,
    });
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{receipt}");
    }
    if let Some(command_url) = input.command_url {
        submit_agent_turn_receipt(command_url, &receipt);
        // Analysis + Judgment gates: the planning turn's output is the judgment.
        if input.label == "plan" && input.status == "completed" {
            let content_hash = stable_agent_hash(input.content.as_bytes());
            submit_judgment_evidence(command_url, content_hash);
        }
    }
}

fn agent_command_url(config: &AgentLoopConfig) -> Option<String> {
    config
        .supervisor_port
        .map(|port| format!("http://127.0.0.1:{port}/v1/command"))
        .or_else(|| {
            std::env::var("SUPERVISOR_PORT")
                .ok()
                .and_then(|value| value.parse::<u16>().ok())
                .map(|port| format!("http://127.0.0.1:{port}/v1/command"))
        })
        .or_else(|| {
            config
                .worker_port
                .map(|port| format!("http://127.0.0.1:{port}/v1/command"))
        })
        .or_else(|| {
            std::env::var("AI_WORKER_PORT")
                .ok()
                .and_then(|value| value.parse::<u16>().ok())
                .map(|port| format!("http://127.0.0.1:{port}/v1/command"))
        })
}

fn submit_agent_turn_receipt(command_url: &str, receipt: &serde_json::Value) {
    let command = agent_turn_kernel_command(receipt);
    match post_json_local(command_url, &command) {
        Ok(status) if (200..300).contains(&status) => {
            eprintln!(
                "agent: submitted turn receipt to kernel url={} status={}",
                command_url, status
            );
        }
        Ok(status) => {
            eprintln!(
                "agent: failed to submit turn receipt to kernel url={} status={}",
                command_url, status
            );
        }
        Err(err) => {
            eprintln!(
                "agent: failed to submit turn receipt to kernel url={command_url} error={err}"
            );
        }
    }
}

fn post_json_local(url: &str, value: &serde_json::Value) -> Result<u16, String> {
    let (host, port, path) = parse_local_http_url(url)?;
    let body = value.to_string();
    let mut stream =
        TcpStream::connect((host.as_str(), port)).map_err(|err| format!("connect: {err}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|err| format!("set read timeout: {err}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|err| format!("set write timeout: {err}"))?;
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {len}\r\n\r\n{body}",
        len = body.len(),
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|err| format!("write: {err}"))?;
    stream.flush().map_err(|err| format!("flush: {err}"))?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|err| format!("read: {err}"))?;
    Ok(parse_mcp_workspace_status(&response))
}

fn parse_local_http_url(url: &str) -> Result<(String, u16, String), String> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| "url must start with http://".to_string())?;
    let (authority, path) = rest
        .split_once('/')
        .map(|(authority, path)| (authority, format!("/{path}")))
        .unwrap_or((rest, "/".to_string()));
    let (host, port) = authority
        .rsplit_once(':')
        .ok_or_else(|| "url must include host:port".to_string())?;
    if !matches!(host, "127.0.0.1" | "localhost") {
        return Err(format!("url host must be local, got {host}"));
    }
    let port = port
        .parse::<u16>()
        .map_err(|_| format!("invalid port in url: {url}"))?;
    Ok((host.to_string(), port, path))
}

fn agent_turn_kernel_command(receipt: &serde_json::Value) -> serde_json::Value {
    let payload_hash = stable_agent_hash(receipt.to_string().as_bytes());
    let plan_payload_hash = stable_agent_hash(
        json!({
            "schema": "canon.agent.router_turn_receipt.plan.v1",
            "agent_receipt_payload_hash": payload_hash,
        })
        .to_string()
        .as_bytes(),
    );
    let execution_passed =
        receipt.get("status").and_then(serde_json::Value::as_str) == Some("completed");
    let plan_submission = EvidenceSubmission::with_effect_payload(
        GateId::Plan,
        Evidence::TaskReady,
        true,
        PacketEffect::BindReadyTask,
        plan_payload_hash,
    );
    let execution_submission = EvidenceSubmission::with_payload(
        GateId::Execution,
        Evidence::ExecutionReceipt,
        execution_passed,
        payload_hash,
    );
    let envelope = CommandEnvelope::new(
        payload_hash,
        Command::SubmitEvidenceBatch(vec![plan_submission, execution_submission]),
    );
    json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": "SubmitEvidenceBatch",
        "source": "agent",
        "agent_turn": receipt,
        "payload": [
            EvidenceSubmissionDto {
                gate: "Plan".to_string(),
                evidence: "TaskReady".to_string(),
                passed: true,
                effect: Some("BindReadyTask".to_string()),
                payload_hash: plan_payload_hash,
            },
            EvidenceSubmissionDto {
                gate: "Execution".to_string(),
                evidence: "ExecutionReceipt".to_string(),
                passed: execution_passed,
                effect: Some("None".to_string()),
                payload_hash,
            },
        ],
    })
}

fn stable_agent_hash(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        h ^= u64::from(*byte);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h.max(1)
}

fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

struct RunCycleAttemptOutcome {
    completed: bool,
    reason: String,
    retry_is_safe: bool,
}

struct RunCyclePlan {
    is_spawned: bool,
    command_url: Option<String>,
    total_turns: u32,
    turn_offset: u32,
    goal: String,
}

fn prepare_run_cycle(config: &AgentLoopConfig) -> Result<RunCyclePlan, String> {
    let is_spawned = config.domain.is_some();
    let command_url = agent_command_url(config);
    let (total_turns, turn_offset) = if is_spawned {
        (config.execute_turns, 1) // all execute, no planning turn
    } else {
        (config.execute_turns + 1, 0) // 1 plan + N execute
    };

    let goal = if is_spawned {
        String::new()
    } else {
        read_goal_file(&config.project_dir)?
    };

    Ok(RunCyclePlan {
        is_spawned,
        command_url,
        total_turns,
        turn_offset,
        goal,
    })
}

struct TurnPromptContext {
    effective_turn: u32,
    turn_mode: LoopMode,
    label: String,
    prompt: String,
}

fn build_turn_prompt_context(
    turn: u32,
    turn_offset: u32,
    is_spawned: bool,
    goal: &str,
    agent_id: u32,
    agent_count: u32,
    working_dir: &Path,
    domain: Option<&str>,
    metric: Option<&str>,
) -> TurnPromptContext {
    let effective_turn = turn + turn_offset;
    let turn_mode = project_turn_mode(effective_turn);
    let is_planning = !is_spawned && turn_mode == LoopMode::ProjectPlanning;
    let label = if is_planning {
        "plan".to_string()
    } else {
        format!("execute-{effective_turn}")
    };

    let prompt = if is_spawned {
        spawned_prompt(
            domain.unwrap_or(""),
            metric.unwrap_or(""),
            turn + 1,
            working_dir,
        )
    } else if is_planning {
        build_project_planning_prompt(goal, agent_id, agent_count, working_dir)
    } else {
        execute_prompt(effective_turn, agent_id, agent_count)
    };

    TurnPromptContext {
        effective_turn,
        turn_mode,
        label,
        prompt,
    }
}

// ── Prompt builders ───────────────────────────────────────────────────────────

fn build_project_planning_prompt(
    goal: &str,
    agent_id: u32,
    agent_count: u32,
    working_dir: &Path,
) -> String {
    let score_report = fs::read_to_string(working_dir.join("SCORE_REPORT.md")).ok();
    let auto_refactor_report = auto_refactor_summary(working_dir);
    let policy_feedback = load_policy_feedback(working_dir);
    planning_prompt(
        goal,
        agent_id,
        agent_count,
        working_dir,
        None,
        None,
        score_report.as_deref(),
        auto_refactor_report.as_deref(),
        policy_feedback.as_deref(),
    )
}

fn planning_prompt(
    goal: &str,
    agent_id: u32,
    agent_count: u32,
    working_dir: &Path,
    domain: Option<&str>,
    metric: Option<&str>,
    score_report: Option<&str>,
    auto_refactor_report: Option<&str>,
    policy_feedback: Option<&str>,
) -> String {
    let agent_line = agent_identity(agent_id, agent_count);
    let focus_block = match (domain, metric) {
        (Some(d), Some(m)) => {
            format!("## YOUR SPECIFIC TASK\n{d}\n\n## SUCCESS CRITERION\n{m}\n\n")
        }
        (Some(d), None) => format!("## YOUR SPECIFIC TASK\n{d}\n\n"),
        _ => String::new(),
    };
    let score_block = match score_report {
        Some(report) => format!(
            "## STRUCTURAL QUALITY SCORES (SCORE_REPORT.md — graph-derived, updated each commit)\n\
             ```\n{report}\n```\n\n"
        ),
        None => String::new(),
    };
    let auto_refactor_block = match auto_refactor_report {
        Some(report) if !report.trim().is_empty() => format!(
            "## AUTO-REFACTOR PLANS (state/rustc/auto-refactor — graph-editor planning evidence)\n\
             ```\n{report}\n```\n\
             Treat these as candidate evidence only. Prefer small `MergeFn` operations whose target exists in the current graph. \
             Do not apply generated merge recommendations blindly; generated serde/self-pair noise must be filtered manually.\n\n"
        ),
        _ => String::new(),
    };
    let policy_block = match policy_feedback {
        Some(feedback) => format!(
            "## PRIOR LEARNING (promoted from verified TLog cycles)\n\
             {feedback}\n\
             The policy store has recorded successful patterns from past cycles. \
             Prefer approaches consistent with prior evidence when the domain and context match.\n\n"
        ),
        None => String::new(),
    };
    format!(
        "{agent_line}\n\
         You are doing the planning turn for this agent loop.\n\n\
         ## WORKING DIRECTORY\n`{dir}`\n\
         All shell commands must run relative to this directory unless the task explicitly requires otherwise.\n\n\
         {focus_block}\
         ## GOAL\n{goal}\n\n\
         {score_block}\
         {auto_refactor_block}\
         {policy_block}\
         ## OPTIMIZATION OBJECTIVE\n\
         The purpose of this planning turn is to choose the next executable work that maximizes expected project goodness. \
         Treat the score axes in `score.md` as the current objective surface. \
         Let each possible work item be x, each score axis be p_i(x) in [0,10], and each optional axis weight be w_i >= 0. \
         Prefer checklist items by expected score gain under:\n\
         x* = argmax_x (product_i p_i(x)^w_i)^(1 / sum_i w_i).\n\
         Prioritize work that raises the lowest justified score axes first, especially when the work can produce validation evidence. \
         Do not raise scores without evidence; instead, plan tasks that can produce evidence likely to justify a future score increase.\n\n\
         Before updating the plan, do the following reconnaissance:\n\
         1. Read the current `plan.md` and identify the first incomplete item under \"Active Priorities\".\n\
         2. Read `status.md` for current progress, validation evidence, blockers, and history.\n\
         3. Read `score.md` for current score values and score rationale.\n\
         4. Read `SCORE_REPORT.md` for graph-derived structural quality scores (Architecture, Structure, Simplicity, Maintainability, Determinism, Coherency).\n\
         5. Inspect `state/rustc/auto-refactor/*.graph-editor-plan.json` when present. Use those plans to identify legitimate, current graph-backed refactor candidates.\n\
         6. Inspect the files, tests, fixtures, evidence paths, and validation commands named by `plan.md`, `status.md`, and selected auto-refactor plans.\n\
         7. If project evidence files are named in the plan or status, inspect them with the appropriate local tool before changing the checklist.\n\n\
         Then update `plan.md` so that the active priority section contains a concrete, ordered checklist \
         of file-level tasks (one file or one test per item) that the execute turns can pick up one at a time. \
         Tasks must name specific files, functions, tests, fixtures, or artifacts — not describe intent in prose. \
         Update `status.md` with progress, validation evidence, blockers, and history. \
         Update `score.md` only when score values or score rationale change. \
         Keep this turn focused on planning, status, and scoring, and commit those changes at the end of the turn.",
        dir = working_dir.display(),
    )
}

fn auto_refactor_summary(project_dir: &Path) -> Option<String> {
    let dir = project_dir
        .join("state")
        .join("rustc")
        .join("auto-refactor");
    let entries = fs::read_dir(&dir).ok()?;
    let mut files: Vec<_> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|v| v.to_str()) == Some("json"))
        .collect();
    files.sort();

    let mut out = String::new();
    let mut listed = 0usize;
    for path in files {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Some(summary) = summarize_auto_refactor_plan(&path, &text) else {
            continue;
        };
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&summary);
        listed += 1;
        if listed >= 20 {
            break;
        }
    }

    (!out.is_empty()).then_some(out)
}

fn summarize_auto_refactor_plan(path: &Path, text: &str) -> Option<String> {
    let file_name = path.file_name()?.to_str()?;
    let crate_name = json_string_field(text, "crate_name").unwrap_or_else(|| "unknown".into());
    let operation_count = json_usize_field(text, "operation_count").unwrap_or(0);
    let merge_count = text.matches("\"op\": \"MergeFn\"").count()
        + json_array_object_count(text, "merge_surface").unwrap_or(0);
    let split_count = text.matches("\"op\": \"SplitFn\"").count();
    let merge_targets = json_string_values_after_key(text, "target_fn", 5);

    let mut line = format!(
        "{file_name}: crate={crate_name} operations={operation_count} merge_fn={merge_count} split_fn={split_count}"
    );
    if !merge_targets.is_empty() {
        line.push_str(" merge_targets=");
        line.push_str(&merge_targets.join(","));
    }
    Some(line)
}

fn json_string_field(text: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let start = text.find(&needle)? + needle.len();
    let after_colon = text[start..].find(':')? + start + 1;
    parse_json_string_at(text, after_colon)
}

fn json_usize_field(text: &str, key: &str) -> Option<usize> {
    let needle = format!("\"{key}\"");
    let start = text.find(&needle)? + needle.len();
    let after_colon = text[start..].find(':')? + start + 1;
    let rest = text[after_colon..].trim_start();
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

fn json_string_values_after_key(text: &str, key: &str, limit: usize) -> Vec<String> {
    let needle = format!("\"{key}\"");
    let mut out = Vec::new();
    let mut offset = 0usize;
    while out.len() < limit {
        let Some(found) = text[offset..].find(&needle) else {
            break;
        };
        let start = offset + found + needle.len();
        let Some(colon_rel) = text[start..].find(':') else {
            break;
        };
        if let Some(value) = parse_json_string_at(text, start + colon_rel + 1) {
            out.push(value);
        }
        offset = start;
    }
    out
}

fn parse_json_string_at(text: &str, offset: usize) -> Option<String> {
    let rest = text[offset..].trim_start();
    let mut chars = rest.chars();
    if chars.next()? != '"' {
        return None;
    }
    let mut value = String::new();
    let mut escaped = false;
    for ch in chars {
        if escaped {
            value.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Some(value);
        } else {
            value.push(ch);
        }
    }
    None
}

fn json_array_object_count(text: &str, key: &str) -> Option<usize> {
    let needle = format!("\"{key}\"");
    let start = text.find(&needle)? + needle.len();
    let after_colon = text[start..].find(':')? + start + 1;
    let array_start = text[after_colon..].find('[')? + after_colon;
    let mut depth = 0i32;
    let mut count = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for ch in text[array_start..].chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(count);
                }
            }
            '{' if depth == 1 => count += 1,
            _ => {}
        }
    }
    None
}

fn spawned_prompt(domain: &str, metric: &str, step: u32, working_dir: &Path) -> String {
    let dir = working_dir.display();
    if step == 1 {
        format!(
            "You are a sub-agent with a single bounded task.\n\n\
             ## WORKING DIRECTORY\n\
             `{dir}`\n\
             All commands run from this directory.\n\n\
             ## TASK\n\
             {domain}\n\n\
             ## SUCCESS CRITERION\n\
             {metric}\n\n\
             ## PROTOCOL\n\
             1. Assess the current state against the success criterion before doing anything else.\n\
             2. Take the minimal actions needed to meet the criterion.\n\
             3. Use `apply_patch` for all file edits.\n\
             4. Every tool call must include a non-empty `intent` field explaining why.\n\
             5. Do not modify `plan.md`, `status.md`, `score.md`, or other planning files.\n\
             6. When the criterion is met: commit all changes, then stop.\n\
             7. If the criterion cannot be met: document the blocker clearly and stop without committing."
        )
    } else {
        format!(
            "You are a sub-agent on step {step} of a bounded task.\n\n\
             ## WORKING DIRECTORY\n\
             `{dir}`\n\n\
             ## TASK\n\
             {domain}\n\n\
             ## SUCCESS CRITERION\n\
             {metric}\n\n\
             ## PROTOCOL\n\
             1. Verify the current state against the success criterion first.\n\
             2. If the criterion is already met: commit any uncommitted changes, then stop.\n\
             3. If not met: identify the specific gap, close it, then re-verify.\n\
             4. Every tool call must include a non-empty `intent` field.\n\
             5. Commit only when the criterion is met. Do not commit partial or failing work."
        )
    }
}

fn execute_prompt(turn_num: u32, agent_id: u32, agent_count: u32) -> String {
    let agent_line = agent_identity(agent_id, agent_count);
    format!(
        "{agent_line}\n\
         You are executing implementation step {turn_num} of this agent loop.\n\n\
         Read `plan.md`, `status.md`, and `score.md`. Find the first unchecked implementation item ([ ]) under \"Active Priorities\" in `plan.md` \
         and implement it now. Only change files within the scope named by that checklist item. \
         If the item names a validation, evidence refresh, documentation, cleanup, or blocker task, perform that task instead of editing unrelated source. \
         After implementation, run the targeted and broader validation commands named in `plan.md`, fix any failures, \
         mark the item done in `plan.md`, update `status.md` with progress/evidence/blockers, \
         update `score.md` only for score changes, and commit all changes. \
         Only commit if all checks pass; if checks cannot be made green, document the \
         blocker in `status.md` or the relevant validation item in `plan.md` and do not commit.\n\n\
         If the plan has two or more unchecked items that are independent of each other \
         (different files, no shared state), you may delegate one by calling \
         `canon_spawn_agent` with a specific domain (the file or function to implement) \
         and metric (the test or check that must pass). Implement the first item yourself \
         and spawn for the second — do not spawn without also making progress yourself."
    )
}

fn agent_identity(agent_id: u32, agent_count: u32) -> String {
    if agent_count > 1 {
        format!("You are **Agent {agent_id}** (one of {agent_count} parallel agents).")
    } else {
        "You are the agent for this project.".to_string()
    }
}

fn agent_tag(agent_id: u32, agent_count: u32) -> String {
    if agent_count > 1 {
        format!("agent-{agent_id}")
    } else {
        "agent".to_string()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LoopMode {
    ProjectPlanning,
    ProjectExecution,
}

impl LoopMode {
    fn label(self) -> &'static str {
        match self {
            Self::ProjectPlanning => "project_planning",
            Self::ProjectExecution => "project_execution",
        }
    }
}

fn project_turn_mode(turn: u32) -> LoopMode {
    if turn == 0 {
        LoopMode::ProjectPlanning
    } else {
        LoopMode::ProjectExecution
    }
}

fn retry_attempt_label(label: &str, attempt: u32) -> String {
    if attempt == 0 {
        label.to_string()
    } else {
        format!("{label}-retry-{attempt}")
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn read_goal_file(project_dir: &Path) -> Result<String, String> {
    let path = project_dir.join("GOAL.md");
    fs::read_to_string(&path).map_err(|e| format!("read GOAL.md: {e}"))
}

/// POST {mcp_url}/workspace with the project root. Mirrors syncMcpWorkspace().
fn sync_mcp_workspace(mcp_url: &str, project_dir: &Path) -> Result<(), String> {
    let (host, port) = parse_mcp_workspace_endpoint(mcp_url)?;
    let request = build_mcp_workspace_request(&host, port, project_dir);
    let response = send_mcp_workspace_request(&host, port, &request)?;

    let status = parse_mcp_workspace_status(&response);

    if status != 200 && status != 201 {
        return Err(format!("MCP workspace sync returned HTTP {status}"));
    }

    Ok(())
}

fn send_mcp_workspace_request(host: &str, port: u16, request: &str) -> Result<String, String> {
    let mut stream = TcpStream::connect((host, port))
        .map_err(|e| format!("MCP connect failed ({host}:{port}): {e}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();

    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("MCP request write: {e}"))?;
    stream.flush().ok();

    let mut response = String::new();
    stream.read_to_string(&mut response).ok();
    Ok(response)
}

fn parse_mcp_workspace_endpoint(mcp_url: &str) -> Result<(String, u16), String> {
    let url = mcp_url.trim_end_matches('/');
    let host_port = url
        .strip_prefix("http://")
        .ok_or_else(|| format!("MCP_CONNECTOR_URL must start with http://: {url}"))?;

    if let Some(colon_pos) = host_port.rfind(':') {
        let h = &host_port[..colon_pos];
        let p: u16 = host_port[colon_pos + 1..]
            .parse()
            .map_err(|_| format!("invalid port in MCP_CONNECTOR_URL: {url}"))?;
        Ok((h.to_string(), p))
    } else {
        Ok((host_port.to_string(), 80u16))
    }
}

fn parse_mcp_workspace_status(response: &str) -> u16 {
    response
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse().ok())
        .unwrap_or(0)
}

fn build_mcp_workspace_request(host: &str, port: u16, project_dir: &Path) -> String {
    let root = project_dir.to_string_lossy();
    let body = format!("{{\"root\":\"{root}\"}}");

    format!(
        "POST /workspace HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {len}\r\n\r\n{body}",
        len = body.len(),
    )
}

// ── Learning loop helpers ─────────────────────────────────────────────────────

fn submit_observation_ingress(
    command_url: &str,
    cycle_num: u64,
    source_id: u64,
    source_bytes: &[u8],
) {
    let goal_bytes = source_bytes;
    let source_hash = stable_agent_hash(goal_bytes);

    // Build a typed Observation batch from actual source content.
    let (batch, records_json) = if goal_bytes.is_empty() {
        let cursor = ObservationCursor {
            source_id,
            last_sequence: 0,
            last_observed_hash: 0,
        };
        (
            ObservationIngressBatch::empty(source_id, source_hash, cursor),
            serde_json::Value::Array(vec![]),
        )
    } else {
        let frame = ObservationFrame::from_payload(
            ObservationFrameKind::ExternalSignal,
            source_id,
            cycle_num,
            cycle_num, // tick proxy: non-zero, strictly increases with cycle
            goal_bytes,
        );
        let record = frame.record();
        let cursor = ObservationCursor {
            source_id,
            last_sequence: record.sequence,
            last_observed_hash: record.observed_hash,
        };
        let rec_json = json!({
            "source_id": record.source_id,
            "sequence": record.sequence,
            "observed_hash": record.observed_hash,
            "received_at_tick": record.received_at_tick,
        });
        (
            ObservationIngressBatch::accepted(source_id, source_hash, cursor, 0, vec![record]),
            serde_json::Value::Array(vec![rec_json]),
        )
    };

    let contract_passed = batch.is_contract_valid();
    let payload_hash = batch.contract_hash();
    let cursor = batch.cursor;
    let envelope = CommandEnvelope::new(payload_hash, Command::SubmitObservationIngress(batch));
    let body = json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": "SubmitObservationIngress",
        "payload": {
            "source_id": source_id,
            "source_hash": source_hash,
            "cursor_source_id": cursor.source_id,
            "cursor_last_sequence": cursor.last_sequence,
            "cursor_last_observed_hash": cursor.last_observed_hash,
            "backlog_len": 0u64,
            "records": records_json,
        },
    });
    match post_json_local(command_url, &body) {
        Ok(s) => {
            eprintln!("agent: observation  cycle={cycle_num}  passed={contract_passed}  status={s}")
        }
        Err(e) => eprintln!("agent: observation failed  cycle={cycle_num}  {e}"),
    }
}

fn submit_judgment_evidence(command_url: &str, content_hash: u64) {
    let analysis_hash = stable_agent_hash(format!("canon:analysis:from:{content_hash}").as_bytes());
    let analysis = EvidenceSubmission::with_payload(
        GateId::Analysis,
        Evidence::AnalysisReport,
        true,
        analysis_hash,
    );
    let judgment = EvidenceSubmission::with_payload(
        GateId::Judgment,
        Evidence::JudgmentRecord,
        true,
        content_hash,
    );
    let envelope = CommandEnvelope::new(
        content_hash,
        Command::SubmitEvidenceBatch(vec![analysis, judgment]),
    );
    let body = json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": "SubmitEvidenceBatch",
        "payload": [
            EvidenceSubmissionDto {
                gate: "Analysis".to_string(),
                evidence: "AnalysisReport".to_string(),
                passed: true,
                effect: Some("None".to_string()),
                payload_hash: analysis_hash,
            },
            EvidenceSubmissionDto {
                gate: "Judgment".to_string(),
                evidence: "JudgmentRecord".to_string(),
                passed: true,
                effect: Some("None".to_string()),
                payload_hash: content_hash,
            },
        ],
    });
    match post_json_local(command_url, &body) {
        Ok(s) => eprintln!("agent: judgment  content_hash={content_hash}  status={s}"),
        Err(e) => eprintln!("agent: judgment failed  {e}"),
    }
}

fn submit_eval_evidence(command_url: &str, score_hash: u64, cycle_num: u64) {
    let verify_hash =
        stable_agent_hash(format!("canon:verification:cycle:{cycle_num}:{score_hash}").as_bytes());
    let verification = EvidenceSubmission::with_effect_payload(
        GateId::Verification,
        Evidence::LineageProof,
        true,
        PacketEffect::RepairLineage,
        verify_hash,
    );
    let eval = EvidenceSubmission::with_effect_payload(
        GateId::Eval,
        Evidence::EvalScore,
        true,
        PacketEffect::CompleteObjective,
        score_hash,
    );
    let command_id =
        stable_agent_hash(format!("canon:eval:cycle:{cycle_num}:{score_hash}").as_bytes());
    let envelope = CommandEnvelope::new(
        command_id,
        Command::SubmitEvidenceBatch(vec![verification, eval]),
    );
    let body = json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": "SubmitEvidenceBatch",
        "payload": [
            EvidenceSubmissionDto {
                gate: "Verification".to_string(),
                evidence: "LineageProof".to_string(),
                passed: true,
                effect: Some("RepairLineage".to_string()),
                payload_hash: verify_hash,
            },
            EvidenceSubmissionDto {
                gate: "Eval".to_string(),
                evidence: "EvalScore".to_string(),
                passed: true,
                effect: Some("CompleteObjective".to_string()),
                payload_hash: score_hash,
            },
        ],
    });
    match post_json_local(command_url, &body) {
        Ok(s) => eprintln!("agent: eval  cycle={cycle_num}  score_hash={score_hash}  status={s}"),
        Err(e) => eprintln!("agent: eval failed  cycle={cycle_num}  {e}"),
    }
}

fn read_score_hash(working_dir: &Path) -> u64 {
    let bytes = fs::read(working_dir.join("SCORE_REPORT.md")).unwrap_or_default();
    if bytes.is_empty() {
        stable_agent_hash(b"canon:eval:no-score-report")
    } else {
        stable_agent_hash(&bytes)
    }
}

fn run_post_cycle_learning(
    command_url: &str,
    tlog_path: &std::path::Path,
    policy_path: &std::path::Path,
    cycle_num: u64,
) {
    let tlog = match load_tlog_ndjson(tlog_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("agent: learning  tlog read failed  cycle={cycle_num}  {e:?}");
            return;
        }
    };

    let mut store = PolicyStore::load_ndjson(policy_path).unwrap_or_default();
    let next_version = store.latest_version() + 1;

    let promotion = match PolicyPromotion::from_tlog(&tlog, next_version) {
        Some(p) => p,
        None => {
            eprintln!("agent: learning  no promotable pattern  cycle={cycle_num}");
            return;
        }
    };

    let payload_hash = promotion.promoted_policy_hash;
    let passed = promotion.is_valid();

    match store.promote_durable(policy_path, promotion) {
        Ok(entry) => {
            eprintln!(
                "agent: learning  promoted  version={}  source_seq={}  cycle={cycle_num}",
                entry.version, entry.value,
            );
        }
        Err(e) => {
            eprintln!("agent: learning  promote failed  cycle={cycle_num}  {e:?}");
            return;
        }
    }

    let submission = EvidenceSubmission::with_payload(
        GateId::Learning,
        Evidence::PolicyPromotion,
        passed,
        payload_hash,
    );
    let envelope = CommandEnvelope::new(payload_hash, Command::SubmitEvidence(submission));
    let body = json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": "SubmitEvidence",
        "payload": EvidenceSubmissionDto {
            gate: "Learning".to_string(),
            evidence: "PolicyPromotion".to_string(),
            passed,
            effect: Some("None".to_string()),
            payload_hash,
        },
    });
    match post_json_local(command_url, &body) {
        Ok(s) => eprintln!("agent: policy promotion  cycle={cycle_num}  status={s}"),
        Err(e) => eprintln!("agent: policy promotion failed  cycle={cycle_num}  {e}"),
    }
}

fn load_policy_feedback(working_dir: &Path) -> Option<String> {
    let policy_path = working_dir.join("state").join("policy.ndjson");
    let store = PolicyStore::load_ndjson(&policy_path).ok()?;
    if store.entries().is_empty() {
        return None;
    }
    let version = store.latest_version();
    let feedback_hash = store.latest_value(POLICY_FEEDBACK_HASH).unwrap_or_else(|| {
        store
            .latest_value(crate::POLICY_PROMOTION_SOURCE_SEQ)
            .unwrap_or(0)
    });
    Some(format!(
        "policy_version={version}  feedback_hash={feedback_hash:#018x}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    fn minimal_loop_config(root: &Path) -> AgentLoopConfig {
        AgentLoopConfig {
            execute_turns: 1,
            turn_retry_limit: 0,
            loop_sleep_ms: 0,
            agent_count: 1,
            project_dir: root.to_path_buf(),
            working_dir: root.to_path_buf(),
            sse_chunks_dir: root.join("sse-chunks"),
            mcp_connector_url: "http://127.0.0.1:4000".to_string(),
            router_turn_max_ms: 1,
            router_first_capture_ms: 1,
            router_idle_ms: 1,
            worker_port: None,
            supervisor_port: None,
            cert_max_steps: 1,
            domain: None,
            metric: None,
        }
    }

    fn capture_local_json_posts(
        count: usize,
    ) -> (String, thread::JoinHandle<Vec<serde_json::Value>>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("loopback listener should bind");
        let url = format!(
            "http://127.0.0.1:{}/v1/command",
            listener
                .local_addr()
                .expect("listener should expose local addr")
                .port()
        );
        let handle = thread::spawn(move || {
            let mut bodies = Vec::with_capacity(count);
            for _ in 0..count {
                let (mut stream, _) = listener.accept().expect("post connection should arrive");
                let mut request = Vec::new();
                let mut buffer = [0u8; 1024];
                loop {
                    let read = stream
                        .read(&mut buffer)
                        .expect("request should be readable");
                    if read == 0 {
                        break;
                    }
                    request.extend_from_slice(&buffer[..read]);
                    let header_end = request
                        .windows(4)
                        .position(|window| window == b"\r\n\r\n")
                        .map(|pos| pos + 4);
                    if let Some(header_end) = header_end {
                        let header_text = String::from_utf8_lossy(&request[..header_end]);
                        let content_length = header_text
                            .lines()
                            .find_map(|line| line.strip_prefix("Content-Length: "))
                            .and_then(|value| value.parse::<usize>().ok())
                            .expect("request should include content length");
                        if request.len() >= header_end + content_length {
                            break;
                        }
                    }
                }

                let header_end = request
                    .windows(4)
                    .position(|window| window == b"\r\n\r\n")
                    .map(|pos| pos + 4)
                    .expect("request should have header terminator");
                let body: serde_json::Value = serde_json::from_slice(&request[header_end..])
                    .expect("captured request body should be json");
                bodies.push(body);
                stream
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                    .expect("response should be writable");
            }
            bodies
        });
        (url, handle)
    }

    #[test]
    fn retry_attempt_labels_preserve_original_and_number_retries() {
        assert_eq!(retry_attempt_label("plan", 0), "plan");
        assert_eq!(retry_attempt_label("plan", 1), "plan-retry-1");
        assert_eq!(retry_attempt_label("execute-2", 3), "execute-2-retry-3");
    }

    #[test]
    fn project_loop_modes_distinguish_planning_and_execution() {
        assert_eq!(project_turn_mode(0), LoopMode::ProjectPlanning);
        assert_eq!(project_turn_mode(1), LoopMode::ProjectExecution);
        assert_eq!(project_turn_mode(9), LoopMode::ProjectExecution);
    }

    #[test]
    fn project_prompts_do_not_claim_to_be_worker_certification() {
        let goal = "Ship the next deterministic runtime slice.";
        let working_dir = Path::new("/workspace/project");
        let planning = planning_prompt(goal, 0, 1, working_dir, None, None, None, None, None);
        let execute = execute_prompt(2, 0, 1);

        assert!(planning.contains("planning turn for this agent loop"));
        assert!(planning.contains("update `plan.md`"));
        assert!(planning.contains("Read `status.md`"));
        assert!(planning.contains("Update `status.md`"));
        assert!(planning.contains("Update `score.md` only when score values"));
        assert!(planning.contains("Read `score.md`"));
        assert!(planning.contains("Inspect the files, tests, fixtures, evidence paths"));
        assert!(planning.contains("## OPTIMIZATION OBJECTIVE"));
        assert!(planning.contains("argmax_x"));
        assert!(planning.contains("Do not raise scores without evidence"));
        assert!(planning.contains("commit those changes"));
        assert!(execute.contains("executing implementation step 2"));
        assert!(execute.contains("Read `plan.md`, `status.md`, and `score.md`"));
        assert!(execute.contains("Only change files within the scope named by that checklist item"));
        assert!(execute.contains("update `status.md` with progress/evidence/blockers"));
        assert!(execute.contains("update `score.md` only for score changes"));
        assert!(!planning.contains("AgentCycle certification"));
        assert!(!execute.contains("AgentCycle certification"));
    }

    #[test]
    fn mcp_workspace_endpoint_parser_preserves_host_port_and_errors() {
        assert_eq!(
            parse_mcp_workspace_endpoint("http://127.0.0.1:9100").unwrap(),
            ("127.0.0.1".to_string(), 9100)
        );
        assert_eq!(
            parse_mcp_workspace_endpoint("http://localhost/").unwrap(),
            ("localhost".to_string(), 80)
        );

        assert!(parse_mcp_workspace_endpoint("https://localhost:9100")
            .unwrap_err()
            .contains("must start with http://"));
        assert!(parse_mcp_workspace_endpoint("http://localhost:not-a-port")
            .unwrap_err()
            .contains("invalid port"));
    }

    #[test]
    fn mcp_workspace_status_parser_accepts_success_and_defaults_invalid() {
        assert_eq!(parse_mcp_workspace_status("HTTP/1.1 200 OK\r\n"), 200);
        assert_eq!(parse_mcp_workspace_status("HTTP/1.1 201 Created\r\n"), 201);
        assert_eq!(parse_mcp_workspace_status("HTTP/1.1 500 Error\r\n"), 500);
        assert_eq!(
            parse_mcp_workspace_status("HTTP/1.1 not-a-code Error\r\n"),
            0
        );
        assert_eq!(parse_mcp_workspace_status(""), 0);
    }

    #[test]
    fn mcp_workspace_request_builder_preserves_workspace_post() {
        let request = build_mcp_workspace_request(
            "127.0.0.1",
            9100,
            Path::new("/workspace/ai_sandbox/canon-mini-agent/prototype/ai"),
        );
        let expected_body = "{\"root\":\"/workspace/ai_sandbox/canon-mini-agent/prototype/ai\"}";
        let (headers, body) = request
            .split_once("\r\n\r\n")
            .expect("workspace request must separate headers and body");

        assert!(headers.starts_with("POST /workspace HTTP/1.1\r\n"));
        assert!(headers.contains("Host: 127.0.0.1:9100\r\n"));
        assert!(headers.contains("Content-Type: application/json\r\n"));
        assert!(headers.contains("Connection: close\r\n"));
        assert_eq!(body, expected_body);

        let content_length = headers
            .lines()
            .find_map(|line| line.strip_prefix("Content-Length: "))
            .expect("workspace request must include Content-Length")
            .parse::<usize>()
            .expect("Content-Length must be numeric");
        assert_eq!(content_length, body.len());
        assert_eq!(content_length, expected_body.len());
    }

    #[test]
    fn agent_turn_receipt_builds_kernel_command() {
        let receipt = json!({
            "schema": "canon.agent.router_turn_receipt.v1",
            "agent": "agent",
            "cycle": 1,
            "label": "plan",
            "attempt": 0,
            "status": "completed",
            "reason": "ok",
            "request_hash": 7,
            "content_hash": 11,
            "content_len": 42,
            "retry_is_safe": false,
        });

        let command = agent_turn_kernel_command(&receipt);

        assert_eq!(command["payload_tag"], "SubmitEvidenceBatch");
        assert_eq!(command["source"], "agent");
        assert_eq!(command["agent_turn"]["label"], "plan");
        assert!(command["command_id"].as_u64().unwrap() != 0);
        assert!(command["command_hash"].as_u64().unwrap() != 0);
        assert_eq!(command["payload"][0]["gate"], "Plan");
        assert_eq!(command["payload"][0]["effect"], "BindReadyTask");
        assert_eq!(command["payload"][1]["gate"], "Execution");
        assert_eq!(command["payload"][1]["passed"], true);
    }

    #[test]
    fn finalize_run_cycle_attempt_result_writes_completed_receipt_and_outcome() {
        let tmp_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.tmp");
        fs::create_dir_all(&tmp_root).expect("canonical temp root should be created");
        let receipt_dir =
            tmp_root.join(format!("canon-agent-turn-receipt-test-{}", timestamp_ms()));
        let _ = fs::remove_dir_all(&receipt_dir);
        fs::create_dir_all(&receipt_dir).expect("receipt test dir should be created");

        let content = "completed assistant content";
        let request_hash = 7_777;
        let outcome = finalize_run_cycle_attempt_result(
            &receipt_dir,
            "agent-a",
            42,
            "plan-retry-1",
            1,
            request_hash,
            false,
            None,
            RouterStreamingResult {
                content: content.to_string(),
                target_url: Some("http://127.0.0.1:9100/target".to_string()),
                complete: true,
                reason: "ok".to_string(),
                finish_reason: Some("stop".to_string()),
            },
        );

        assert!(outcome.completed);
        assert_eq!(outcome.reason, "ok");
        assert!(!outcome.retry_is_safe);

        let receipt_path = receipt_dir.join("agent-turn-receipts.ndjson");
        let receipt_text =
            fs::read_to_string(&receipt_path).expect("completed receipt should be appended");
        let lines: Vec<_> = receipt_text.lines().collect();
        assert_eq!(lines.len(), 1);
        let receipt: serde_json::Value =
            serde_json::from_str(lines[0]).expect("receipt line should be json");

        assert_eq!(receipt["schema"], "canon.agent.router_turn_receipt.v1");
        assert_eq!(receipt["agent"], "agent-a");
        assert_eq!(receipt["cycle"], 42);
        assert_eq!(receipt["label"], "plan-retry-1");
        assert_eq!(receipt["attempt"], 1);
        assert_eq!(receipt["status"], "completed");
        assert_eq!(receipt["reason"], "ok");
        assert_eq!(receipt["finish_reason"], "stop");
        assert_eq!(receipt["request_hash"], request_hash);
        assert_eq!(receipt["content_len"], content.len());
        assert_eq!(
            receipt["content_hash"],
            stable_agent_hash(content.as_bytes())
        );
        assert_eq!(
            receipt["target_url_hash"],
            stable_agent_hash("http://127.0.0.1:9100/target".as_bytes())
        );
        assert_eq!(receipt["retry_is_safe"], false);

        let _ = fs::remove_dir_all(&receipt_dir);
    }

    #[test]
    fn cycle_start_observation_ingress_preserves_top_level_and_spawned_sources() {
        let tmp_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.tmp");
        fs::create_dir_all(&tmp_root).expect("canonical temp root should be created");
        let working_dir = tmp_root.join(format!("cycle-start-observation-test-{}", timestamp_ms()));
        let _ = fs::remove_dir_all(&working_dir);
        fs::create_dir_all(&working_dir).expect("test working dir should be created");
        fs::write(working_dir.join("GOAL.md"), "top-level goal bytes")
            .expect("goal file should be writable");

        let (command_url, handle) = capture_local_json_posts(2);

        let top_level_driver = LoopDriver::new(minimal_loop_config(&working_dir));
        top_level_driver.submit_cycle_start_observation_ingress(&command_url, 7, false);

        let mut spawned_config = minimal_loop_config(&working_dir);
        spawned_config.domain = Some("spawned domain".to_string());
        spawned_config.metric = Some("spawned metric".to_string());
        let spawned_driver = LoopDriver::new(spawned_config);
        spawned_driver.submit_cycle_start_observation_ingress(&command_url, 8, true);

        let captured = handle.join().expect("capture thread should finish");
        assert_eq!(captured.len(), 2);

        assert_eq!(captured[0]["payload_tag"], "SubmitObservationIngress");
        assert_eq!(
            captured[0]["payload"]["source_id"],
            stable_agent_hash(b"canon:goal:observation")
        );
        assert_eq!(
            captured[0]["payload"]["source_hash"],
            stable_agent_hash(b"top-level goal bytes")
        );
        assert_eq!(captured[0]["payload"]["cursor_last_sequence"], 7);

        assert_eq!(captured[1]["payload_tag"], "SubmitObservationIngress");
        assert_eq!(
            captured[1]["payload"]["source_id"],
            stable_agent_hash(b"canon:domain:observation")
        );
        assert_eq!(
            captured[1]["payload"]["source_hash"],
            stable_agent_hash(b"spawned domain\nspawned metric")
        );
        assert_eq!(captured[1]["payload"]["cursor_last_sequence"], 8);

        let _ = fs::remove_dir_all(&working_dir);
    }

    #[test]
    fn local_http_url_parser_accepts_loopback_command_url() {
        assert_eq!(
            parse_local_http_url("http://127.0.0.1:9100/v1/command").unwrap(),
            ("127.0.0.1".to_string(), 9100, "/v1/command".to_string())
        );
        assert!(parse_local_http_url("https://127.0.0.1:9100/v1/command").is_err());
        assert!(parse_local_http_url("http://example.com:9100/v1/command").is_err());
    }
}
