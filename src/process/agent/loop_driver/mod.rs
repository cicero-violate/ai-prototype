use std::fs;
use std::thread;
use std::time::Duration;

use crate::capability::llm::openai::{OpenAiChatRequest, OpenAiMessage};
use crate::process::agent::config::AgentLoopConfig;
use crate::process::agent::router::{RouterClient, RouterTabCloseOutcome};
use crate::process::agent::sse::ChunkLogger;
use crate::MAX_OBSERVATION_PAYLOAD_BYTES;

pub(crate) mod common;
mod cycle_log;
mod evidence_submit;
pub(crate) mod http;
mod learning;
mod mcp_workspace;
mod prompt_builders;
mod receipt;

use common::stable_agent_hash;
use evidence_submit::{read_score_hash, submit_eval_evidence, submit_observation_ingress};
use http::agent_command_url;
use learning::run_post_cycle_learning;
use mcp_workspace::sync_mcp_workspace;
use prompt_builders::{
    agent_tag, build_turn_prompt_context, prepare_run_cycle, retry_attempt_label,
};
use receipt::{
    finalize_run_cycle_attempt_result, write_agent_turn_receipt, AgentTurnReceiptInput,
    RunCycleAttemptOutcome,
};

type TabCloseResult = Result<RouterTabCloseOutcome, String>;

fn wait_for_spawned_tab_close(
    tag: &str,
    cycle_num: u64,
    close_handle: Option<thread::JoinHandle<TabCloseResult>>,
) {
    let Some(handle) = close_handle else {
        eprintln!("[{tag}] tab close verification skipped: no router target  cycle={cycle_num}");
        return;
    };

    match handle.join() {
        Ok(Ok(RouterTabCloseOutcome::Closed)) => {
            eprintln!("[{tag}] tab close verified  outcome=Closed  cycle={cycle_num}");
        }
        Ok(Ok(outcome)) => {
            eprintln!(
                "[{tag}] tab close verification incomplete  outcome={outcome:?}  cycle={cycle_num}"
            );
        }
        Ok(Err(e)) => {
            eprintln!("[{tag}] tab close verification failed: {e}  cycle={cycle_num}");
        }
        Err(_) => {
            eprintln!(
                "[{tag}] tab close verification failed: close worker panicked  cycle={cycle_num}"
            );
        }
    }
}

#[cfg(test)]
use common::timestamp_ms;
#[cfg(test)]
use http::parse_local_http_url;
#[cfg(test)]
use mcp_workspace::{
    build_mcp_workspace_request, parse_mcp_workspace_endpoint, parse_mcp_workspace_status,
};
#[cfg(test)]
use prompt_builders::{
    agent_identity, execute_prompt, planning_prompt, project_turn_mode, spawned_prompt, LoopMode,
};
#[cfg(test)]
use receipt::agent_turn_kernel_command;

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
    pub fn run_all_agents(&self) -> bool {
        let goal_path = self.config.project_dir.join("GOAL.md");
        if !goal_path.exists() {
            eprintln!(
                "agent: PROJECT_DIR={} has no GOAL.md — loop disabled",
                self.config.project_dir.display()
            );
            return false;
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

        let is_spawned = self.config.domain.is_some();
        match sync_mcp_workspace(&self.config.mcp_connector_url, &self.config.project_dir) {
            Ok(_) => eprintln!(
                "agent: MCP workspace synced ({})",
                self.config.mcp_connector_url
            ),
            Err(e) => {
                if is_spawned {
                    eprintln!(
                        "agent: MCP workspace sync failed for spawned task; continuing without external MCP connector: {e}"
                    );
                } else {
                    eprintln!("agent: MCP workspace sync failed: {e}");
                    return false;
                }
            }
        }

        if self.config.agent_count <= 1 {
            self.run_agent_loop(0)
        } else {
            let mut handles = Vec::new();
            for agent_id in 0..self.config.agent_count {
                let config = self.config.clone();
                let handle = thread::spawn(move || {
                    if agent_id > 0 {
                        thread::sleep(Duration::from_millis(u64::from(agent_id) * 5000));
                    }
                    LoopDriver { config }.run_agent_loop(agent_id)
                });
                handles.push(handle);
            }
            let mut all_completed = true;
            for h in handles {
                all_completed &= h.join().unwrap_or(false);
            }
            all_completed
        }
    }

    fn run_agent_loop(&self, agent_id: u32) -> bool {
        let is_spawned = self.config.domain.is_some();
        let tag = self
            .config
            .plan_node_id
            .as_ref()
            .map(|node_id| format!("mini-{node_id}"))
            .unwrap_or_else(|| agent_tag(agent_id, self.config.agent_count));

        if !is_spawned {
            let goal_path = self.config.project_dir.join("GOAL.md");
            if !goal_path.exists() {
                eprintln!("[{tag}] no GOAL.md — loop disabled");
                return false;
            }
        }

        if let Some(last) = cycle_log::last_completed_cycle(&self.config.sse_chunks_dir, &tag) {
            eprintln!("[{tag}] cycle log: last completed cycle={last}");
        }

        let mut cycle_num: u64 = 0;
        loop {
            cycle_num += 1;
            eprintln!("[{tag}] ── cycle {cycle_num} ──────────────────────────────");

            // Before opening a top-level planner tab, drain any ready DAG work
            // that already exists in state/plan.json.  This prevents a new
            // planner/connector-activation tab from racing the mini-agent tabs
            // when the previous cycle left pending ready nodes behind.
            if !is_spawned && crate::process::scheduler::run_wave(&self.config, &tag, cycle_num) {
                eprintln!(
                    "[{tag}] DAG scheduler ran before planner startup; sleeping {}ms before next cycle",
                    self.config.loop_sleep_ms
                );
                thread::sleep(Duration::from_millis(self.config.loop_sleep_ms));
                continue;
            }

            let mut router = match RouterClient::from_env() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[{tag}] router init failed: {e}");
                    return false;
                }
            };

            let cmd_url = agent_command_url(&self.config);
            cycle_log::append_cycle_start(
                &self.config.sse_chunks_dir,
                &tag,
                cycle_num,
                cmd_url.as_deref(),
            );
            let cycle_completed = match self.run_cycle(cycle_num, agent_id, &tag, &mut router) {
                Ok(()) => true,
                Err(e) => {
                    eprintln!("[{tag}] cycle {cycle_num} failed: {e}");
                    false
                }
            };
            cycle_log::append_cycle_end(
                &self.config.sse_chunks_dir,
                &tag,
                cycle_num,
                cmd_url.as_deref(),
            );

            let close_handle = if let Some(target_id) = router.target_id().map(str::to_string) {
                let tag2 = tag.clone();
                Some(thread::spawn(move || {
                    eprintln!(
                        "[{tag2}] tab close starting  cycle={cycle_num}  target_id={target_id}"
                    );
                    match crate::process::agent::router::close_tab_for_target_id_with_timeout(
                        &target_id, 8_000,
                    ) {
                        Ok(outcome) => {
                            eprintln!("[{tag2}] tab close  outcome={outcome:?}  cycle={cycle_num}");
                            Ok(outcome)
                        }
                        Err(e) => {
                            let message = e.to_string();
                            eprintln!("[{tag2}] tab close failed: {message}  cycle={cycle_num}");
                            Err(message)
                        }
                    }
                }))
            } else if let Some(url) = router.target_url().map(str::to_string) {
                let tag2 = tag.clone();
                Some(thread::spawn(move || {
                    eprintln!("[{tag2}] tab close starting  cycle={cycle_num}  target={url}");
                    match crate::process::agent::router::close_tab_for_url_with_timeout(&url, 8_000)
                    {
                        Ok(outcome) => {
                            eprintln!("[{tag2}] tab close  outcome={outcome:?}  cycle={cycle_num}");
                            Ok(outcome)
                        }
                        Err(e) => {
                            let message = e.to_string();
                            eprintln!("[{tag2}] tab close failed: {message}  cycle={cycle_num}");
                            Err(message)
                        }
                    }
                }))
            } else {
                None
            };

            // Spawned agents run one cycle for their specific task then exit.
            if is_spawned {
                wait_for_spawned_tab_close(&tag, cycle_num, close_handle);
                eprintln!("[{tag}] spawned agent task complete — exiting");
                return cycle_completed;
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
                self.config.plan_node_id.as_deref(),
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

            if completed {
                cycle_log::append_turn_complete(
                    &self.config.sse_chunks_dir,
                    tag,
                    cycle_num,
                    &turn_context.label,
                    &last_reason,
                    command_url.as_deref(),
                );
            } else {
                cycle_log::append_turn_failed(
                    &self.config.sse_chunks_dir,
                    tag,
                    cycle_num,
                    &turn_context.label,
                    command_url.as_deref(),
                );
                return Err(format!(
                    "turn {} did not complete after {} attempt(s): {last_reason}",
                    turn_context.label,
                    self.config.turn_retry_limit + 1,
                ));
            }

            // After the planning turn: dispatch any ready DAG nodes as child agents.
            // If a wave ran, skip the execute turns — the child agents did the work.
            if turn == 0
                && !is_spawned
                && crate::process::scheduler::run_wave(&self.config, tag, cycle_num)
            {
                break;
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
                    &self.config.working_dir,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::agent::RouterStreamingResult;
    use serde_json::json;
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::path::Path;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    fn minimal_loop_config(root: &Path) -> AgentLoopConfig {
        AgentLoopConfig {
            execute_turns: 1,
            turn_retry_limit: 0,
            loop_sleep_ms: 0,
            agent_count: 1,
            executor_count: 1,
            project_dir: root.to_path_buf(),
            working_dir: root.to_path_buf(),
            sse_chunks_dir: root.join("sse-chunks"),
            mcp_connector_url: "http://127.0.0.1:9100".to_string(),
            router_turn_max_ms: 1,
            router_first_capture_ms: 1,
            router_idle_ms: 1,
            worker_port: None,
            supervisor_port: None,
            browser_router_url: None,
            cert_max_steps: 1,
            domain: None,
            metric: None,
            plan_node_id: None,
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
    fn spawned_agent_waits_for_tab_close_before_exit() {
        let (tx, rx) = mpsc::channel();
        let close_handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(25));
            tx.send(()).expect("test receiver should stay open");
            Ok(RouterTabCloseOutcome::Closed)
        });

        wait_for_spawned_tab_close("agent", 1, Some(close_handle));

        rx.try_recv()
            .expect("spawned cleanup must wait until close worker finishes");
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
    fn run_cycle_preparation_preserves_project_and_spawned_turn_schedules() {
        let tmp_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../state/tmp");
        fs::create_dir_all(&tmp_root).expect("canonical temp root should be created");
        let project_root = tmp_root.join(format!("run-cycle-prep-project-{}", timestamp_ms()));
        let spawned_root = tmp_root.join(format!("run-cycle-prep-spawned-{}", timestamp_ms()));
        let _ = fs::remove_dir_all(&project_root);
        let _ = fs::remove_dir_all(&spawned_root);
        fs::create_dir_all(&project_root).expect("project fixture dir should be created");
        fs::create_dir_all(&spawned_root).expect("spawned fixture dir should be created");

        let goal = "deterministic project goal";
        fs::write(project_root.join("GOAL.md"), goal).expect("project goal should be writable");

        let mut project_config = minimal_loop_config(&project_root);
        project_config.execute_turns = 3;
        project_config.supervisor_port = Some(9_101);
        let project_plan =
            prepare_run_cycle(&project_config).expect("project cycle should prepare");
        assert!(!project_plan.is_spawned);
        assert_eq!(
            project_plan.command_url.as_deref(),
            Some("http://127.0.0.1:9101/v1/command")
        );
        assert_eq!(project_plan.total_turns, project_config.execute_turns + 1);
        assert_eq!(project_plan.turn_offset, 0);
        assert_eq!(project_plan.goal, goal);

        let mut spawned_config = minimal_loop_config(&spawned_root);
        spawned_config.execute_turns = 2;
        spawned_config.supervisor_port = Some(9_102);
        spawned_config.domain = Some("spawned domain".to_string());
        spawned_config.metric = Some("spawned metric".to_string());
        spawned_config.plan_node_id = Some("spawned-node".to_string());
        let spawned_plan =
            prepare_run_cycle(&spawned_config).expect("spawned cycle should prepare");
        assert!(spawned_plan.is_spawned);
        assert_eq!(
            spawned_plan.command_url.as_deref(),
            Some("http://127.0.0.1:9102/v1/command")
        );
        assert_eq!(spawned_plan.total_turns, spawned_config.execute_turns);
        assert_eq!(spawned_plan.turn_offset, 1);
        assert_eq!(spawned_plan.goal, "");

        let _ = fs::remove_dir_all(&project_root);
        let _ = fs::remove_dir_all(&spawned_root);
    }

    #[test]
    fn agent_label_helpers_preserve_identity_and_tag_boundaries() {
        let single_identity = "You are the agent for this project.";
        let agent_one_identity = "You are **Agent 1** (one of 3 parallel agents).";
        let agent_two_identity = "You are **Agent 2** (one of 3 parallel agents).";

        assert_eq!(agent_identity(0, 1), single_identity);
        assert_eq!(agent_identity(1, 3), agent_one_identity);
        assert_eq!(agent_identity(2, 3), agent_two_identity);
        assert_eq!(agent_tag(0, 1), "agent");
        assert_eq!(agent_tag(1, 3), "agent-1");
        assert_eq!(agent_tag(2, 3), "agent-2");

        let working_dir = Path::new("/workspace/project");
        let planning = planning_prompt(
            "Preserve agent label boundaries.",
            1,
            3,
            working_dir,
            None,
            None,
            None,
            None,
            None,
        );
        let execute = execute_prompt(2, 2, 3);

        assert!(planning.starts_with(agent_one_identity));
        assert!(planning.contains("planning turn for this agent loop"));
        assert!(execute.starts_with(agent_two_identity));
        assert!(execute.contains("executing implementation step 2"));
        assert!(!planning.contains("agent-1"));
        assert!(!execute.contains("agent-2"));
        assert_ne!(agent_identity(1, 3), agent_tag(1, 3));
        assert_ne!(agent_identity(0, 1), agent_tag(0, 1));
    }

    #[test]
    fn project_prompts_do_not_claim_to_be_worker_certification() {
        let goal = "Ship the next deterministic runtime slice.";
        let working_dir = Path::new("/workspace/project");
        let planning = planning_prompt(goal, 0, 1, working_dir, None, None, None, None, None);
        let execute = execute_prompt(2, 0, 1);

        assert!(planning.contains("planning turn for this agent loop"));
        assert!(planning.contains("## HOW EXECUTOR DISPATCH WORKS"));
        assert!(planning.contains("spawns one executor per ready node in parallel"));
        assert!(planning.contains("TLog-projected plan read model"));
        assert!(planning.contains("project:plan_read"));
        assert!(planning.contains("project:plan_update"));
        assert!(planning.contains("state/agent-evidence/"));
        assert!(planning.contains("append_evidence"));
        assert!(planning.contains("## OPTIMIZATION OBJECTIVE"));
        assert!(planning.contains("argmax_x"));
        assert!(planning.contains("Do not raise scores without evidence"));
        assert!(planning.contains("Commit all changes"));
        assert!(!planning.contains("plan.md"));
        assert!(!planning.contains("status.md"));
        assert!(!planning.contains("score.md"));
        assert!(execute.contains("executing implementation step 2"));
        assert!(execute.contains("DAG scheduler found no ready nodes"));
        assert!(execute.contains("project:plan_read"));
        assert!(execute.contains("state/agent-evidence/"));
        assert!(execute.contains("append_evidence"));
        assert!(!execute.contains("op=set_status"));
        assert!(!execute.contains("reset the node to `pending`"));
        assert!(!execute.contains("plan.md"));
        assert!(!execute.contains("status.md"));
        assert!(!execute.contains("score.md"));
        assert!(!planning.contains("AgentCycle certification"));
        assert!(!execute.contains("AgentCycle certification"));
    }

    #[test]
    fn spawned_prompt_rejects_empty_success_criterion_and_preserves_supervisor_authority() {
        let prompt = spawned_prompt(
            "Align agent cycle route contract",
            "   ",
            Some("align-agent-cycle-route-contract"),
            1,
            Path::new("/workspace/project"),
        );

        assert!(prompt.contains("## SUCCESS CRITERION"));
        assert!(prompt.contains("BLOCKER: no success criterion was supplied"));
        assert!(prompt.contains("Do not invent scope"));
        assert!(prompt.contains("op=append_evidence"));
        assert!(prompt.contains("do not call `op=set_status`"));
        assert!(prompt.contains("The supervisor will decide completion"));
        assert!(!prompt.contains("The scheduler will mark your task done automatically"));
        assert!(!prompt.contains("The scheduler will mark your task failed"));
    }

    #[test]
    fn mcp_workspace_endpoint_parser_preserves_host_port_and_errors() {
        assert_eq!(
            parse_mcp_workspace_endpoint("http://127.0.0.1:9100")
                .expect("test value should be present"),
            ("127.0.0.1".to_string(), 9100)
        );
        assert_eq!(
            parse_mcp_workspace_endpoint("http://localhost/")
                .expect("test value should be present"),
            ("localhost".to_string(), 80)
        );

        assert!(parse_mcp_workspace_endpoint("https://localhost:9100")
            .expect_err("test should fail")
            .contains("must start with http://"));
        assert!(parse_mcp_workspace_endpoint("http://localhost:not-a-port")
            .expect_err("test should fail")
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

        assert!(headers.starts_with("POST /ai/workspace HTTP/1.1\r\n"));
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
        assert!(
            command["command_id"]
                .as_u64()
                .expect("test value should be present")
                != 0
        );
        assert!(
            command["command_hash"]
                .as_u64()
                .expect("test value should be present")
                != 0
        );
        assert_eq!(command["payload"][0]["gate"], "Plan");
        assert_eq!(command["payload"][0]["effect"], "BindReadyTask");
        assert_eq!(command["payload"][1]["gate"], "Execution");
        assert_eq!(command["payload"][1]["passed"], true);
    }

    #[test]
    fn finalize_run_cycle_attempt_result_writes_completed_receipt_and_outcome() {
        let tmp_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../state/tmp");
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
        let tmp_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../state/tmp");
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
            parse_local_http_url("http://127.0.0.1:9100/v1/command")
                .expect("test value should be present"),
            ("127.0.0.1".to_string(), 9100, "/v1/command".to_string())
        );
        assert!(parse_local_http_url("https://127.0.0.1:9100/v1/command").is_err());
        assert!(parse_local_http_url("http://example.com:9100/v1/command").is_err());
    }
}
