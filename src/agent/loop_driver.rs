use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::agent::config::AgentLoopConfig;
use crate::agent::cycle::AgentCycle;
use crate::agent::objective::AgentObjective;
use crate::agent::router::RouterClient;
use crate::agent::sse::ChunkLogger;
use crate::agent::worker_client::WorkerClient;
use crate::capability::llm::openai::{OpenAiChatRequest, OpenAiMessage};

/// Outer project-loop driver. Mirrors agentLoop / runCycle from
/// chatgpt-agent-loop/agent-loop.mjs.
///
/// Reads GOAL.md, syncs MCP workspace, then runs an infinite cycle of
/// 1 planning turn + N execute turns with per-turn retry logic.
///
/// This is deliberately separate from the phase-driven worker/runtime path:
/// the project loop asks the router model to edit files and commit work, while
/// `AgentCycle` certification submits typed evidence through the worker so the
/// deterministic state machine can verify and record the result.
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

            let router = match RouterClient::from_env() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[{tag}] router init failed: {e}");
                    break;
                }
            };

            if let Err(e) = self.run_cycle(cycle_num, agent_id, &tag, router) {
                eprintln!("[{tag}] cycle {cycle_num} failed: {e}");
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
        mut router: RouterClient,
    ) -> Result<(), String> {
        let is_spawned = self.config.domain.is_some();
        let (total_turns, turn_offset) = if is_spawned {
            (self.config.execute_turns, 1) // all execute, no planning turn
        } else {
            (self.config.execute_turns + 1, 0) // 1 plan + N execute
        };

        let goal = if is_spawned {
            String::new()
        } else {
            read_goal_file(&self.config.project_dir)?
        };

        for turn in 0..total_turns {
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
                    self.config.domain.as_deref().unwrap_or(""),
                    self.config.metric.as_deref().unwrap_or(""),
                    turn + 1,
                    &self.config.working_dir,
                )
            } else if is_planning {
                let score_report =
                    std::fs::read_to_string(self.config.working_dir.join("SCORE_REPORT.md")).ok();
                let auto_refactor_report = auto_refactor_summary(&self.config.working_dir);
                planning_prompt(
                    &goal,
                    agent_id,
                    self.config.agent_count,
                    &self.config.working_dir,
                    None,
                    None,
                    score_report.as_deref(),
                    auto_refactor_report.as_deref(),
                )
            } else {
                execute_prompt(effective_turn, agent_id, self.config.agent_count)
            };

            eprintln!(
                "[{tag}] ── turn {}/{total_turns}: {label} mode={} (cycle {cycle_num})",
                turn + 1,
                turn_mode.label(),
            );

            let mut completed = false;
            let mut last_reason = String::new();

            for attempt in 0..=self.config.turn_retry_limit {
                if attempt > 0 {
                    eprintln!(
                        "[{tag}] turn {label} retry {attempt}/{} after {last_reason}",
                        self.config.turn_retry_limit
                    );
                    thread::sleep(Duration::from_millis(self.config.loop_sleep_ms));
                }

                let attempt_label = retry_attempt_label(&label, attempt);

                let mut logger =
                    ChunkLogger::new(&self.config.sse_chunks_dir, tag, cycle_num, &attempt_label)
                        .map_err(|e| e.to_string())?;

                let request = OpenAiChatRequest::new(vec![OpenAiMessage::user(prompt.clone())]);
                let has_target_url = router.target_url().is_some();

                let result = router
                    .streaming_turn(
                        request,
                        &mut logger,
                        self.config.router_turn_max_ms,
                        self.config.router_first_capture_ms,
                        self.config.router_idle_ms,
                    )
                    .map_err(|e| e.to_string())?;

                last_reason = result.reason.clone();

                if result.complete {
                    completed = true;
                    let preview: String = result.content.chars().take(120).collect();
                    let preview = preview.replace('\n', " ");
                    eprintln!(
                        "[{tag}] turn {label} done — {} — {preview}…",
                        result.target_url.as_deref().unwrap_or("no url"),
                    );
                    break;
                }

                eprintln!(
                    "[{tag}] turn {label} incomplete — {}; finish={}",
                    result.reason,
                    result.finish_reason.as_deref().unwrap_or("none"),
                );

                if !result.retry_is_safe(has_target_url) {
                    break;
                }
            }

            if !completed {
                return Err(format!(
                    "turn {label} did not complete after {} attempt(s): {last_reason}",
                    self.config.turn_retry_limit + 1,
                ));
            }

            if turn < total_turns - 1 {
                thread::sleep(Duration::from_millis(self.config.loop_sleep_ms));
            }
        }

        // All loop turns completed; close the project-loop browser tab before
        // certification opens its own router session.
        close_router_tab(tag, "project", &mut router);

        // Run evidence certification against the worker.
        self.run_certification(cycle_num, tag);

        Ok(())
    }

    /// Run an AgentCycle against the worker to stamp evidence gates to the tlog.
    /// Called after each successful loop cycle.
    fn run_certification(&self, cycle_num: u64, tag: &str) {
        let mode = LoopMode::WorkerCertification;
        let Some(mut worker_port) = self.config.worker_port else {
            return;
        };

        // If a supervisor is configured, reload the worker to get a fresh state.
        if let Some(sup_port) = self.config.supervisor_port {
            eprintln!("[{tag}] cert: reloading worker via supervisor (port {sup_port})");
            match supervisor_reload(sup_port, worker_port) {
                Ok(reloaded_worker_port) => {
                    worker_port = reloaded_worker_port;
                    eprintln!("[{tag}] cert: waiting for worker on port {worker_port}");
                    if !wait_for_worker_healthy(worker_port, 30) {
                        eprintln!(
                            "[{tag}] cert: worker did not become healthy after reload — skipping"
                        );
                        return;
                    }
                }
                Err(e) => {
                    eprintln!("[{tag}] cert: supervisor reload failed: {e} — continuing with current worker state");
                }
            }
        }

        let router = match RouterClient::from_env() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[{tag}] cert: router init failed: {e}");
                return;
            }
        };

        let worker = WorkerClient::new(worker_port);
        let objective = build_cert_objective(&self.config.project_dir);

        eprintln!(
            "[{tag}] cert: cycle {cycle_num} mode={} — domain={}…",
            mode.label(),
            objective
                .domain_hint
                .chars()
                .take(80)
                .collect::<String>()
                .replace('\n', " "),
        );

        let mut cycle =
            AgentCycle::new(router, worker, objective).with_max_steps(self.config.cert_max_steps);

        match cycle.run() {
            Ok(summary) => eprintln!(
                "[{tag}] cert: done  steps={}  success={}  stop={}  phase={}  tlog_len={}",
                summary.step_count,
                summary.success,
                summary.stop_reason,
                summary.final_phase,
                summary.final_tlog_len,
            ),
            Err(e) => eprintln!("[{tag}] cert: AgentCycle failed: {e}"),
        }
        match cycle.close_router_tab() {
            Ok(outcome) => eprintln!("[{tag}] cert: browser tab close outcome: {outcome:?}"),
            Err(e) => eprintln!("[{tag}] cert: browser tab close failed: {e}"),
        }
    }
}

fn close_router_tab(tag: &str, label: &str, router: &mut RouterClient) {
    match router.close_current_tab() {
        Ok(outcome) => eprintln!("[{tag}] {label}: browser tab close outcome: {outcome:?}"),
        Err(e) => eprintln!("[{tag}] {label}: browser tab close failed: {e}"),
    }
}

// ── Prompt builders ───────────────────────────────────────────────────────────

fn planning_prompt(
    goal: &str,
    agent_id: u32,
    agent_count: u32,
    working_dir: &Path,
    domain: Option<&str>,
    metric: Option<&str>,
    score_report: Option<&str>,
    auto_refactor_report: Option<&str>,
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
             Treat these as candidate evidence only. Prefer small `SplitFn` operations whose target exists in the current graph. \
             Do not apply `merge_surface` recommendations blindly; generated serde/self-pair noise must be filtered manually.\n\n"
        ),
        _ => String::new(),
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
    let split_count = text.matches("\"op\": \"SplitFn\"").count();
    let merge_count = json_array_object_count(text, "merge_surface").unwrap_or(0);
    let split_targets = json_string_values_after_key(text, "fn_path", 5);

    let mut line = format!(
        "{file_name}: crate={crate_name} operations={operation_count} split_fn={split_count} merge_surface={merge_count}"
    );
    if !split_targets.is_empty() {
        line.push_str(" split_targets=");
        line.push_str(&split_targets.join(","));
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
            "You are a sub-agent with a specific task.\n\n\
             ## WORKING DIRECTORY\n`{dir}`\n\
             All shell commands must run relative to this directory.\n\n\
             ## YOUR TASK\n{domain}\n\n\
             ## SUCCESS CRITERION\n{metric}\n\n\
             Implement this now. Write or edit the necessary source files, run tests, \
             fix any failures, and commit your changes. \
             Do not touch `plan.md`, `status.md`, `score.md`, or any other planning/status files. \
             Focus only on the task above."
        )
    } else {
        format!(
            "You are a sub-agent continuing your task (step {step}).\n\n\
             ## WORKING DIRECTORY\n`{dir}`\n\n\
             ## YOUR TASK\n{domain}\n\n\
             ## SUCCESS CRITERION\n{metric}\n\n\
             Continue implementing. Check what remains, fix any failures, and commit."
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
    WorkerCertification,
}

impl LoopMode {
    fn label(self) -> &'static str {
        match self {
            Self::ProjectPlanning => "project_planning",
            Self::ProjectExecution => "project_execution",
            Self::WorkerCertification => "worker_certification",
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
    let url = mcp_url.trim_end_matches('/');
    let host_port = url
        .strip_prefix("http://")
        .ok_or_else(|| format!("MCP_CONNECTOR_URL must start with http://: {url}"))?;

    let (host, port) = if let Some(colon_pos) = host_port.rfind(':') {
        let h = &host_port[..colon_pos];
        let p: u16 = host_port[colon_pos + 1..]
            .parse()
            .map_err(|_| format!("invalid port in MCP_CONNECTOR_URL: {url}"))?;
        (h.to_string(), p)
    } else {
        (host_port.to_string(), 80u16)
    };

    let root = project_dir.to_string_lossy();
    let body = format!("{{\"root\":\"{root}\"}}");

    let mut stream = TcpStream::connect((host.as_str(), port))
        .map_err(|e| format!("MCP connect failed ({host}:{port}): {e}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();

    let request = format!(
        "POST /workspace HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {len}\r\n\r\n{body}",
        len = body.len(),
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("MCP request write: {e}"))?;
    stream.flush().ok();

    let mut response = String::new();
    stream.read_to_string(&mut response).ok();

    let status: u16 = response
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if status != 200 && status != 201 {
        return Err(format!("MCP workspace sync returned HTTP {status}"));
    }

    Ok(())
}

// ── Certification helpers ─────────────────────────────────────────────────────

/// Build an AgentObjective from the project files produced by the loop cycle.
fn build_cert_objective(project_dir: &Path) -> AgentObjective {
    let goal = read_file_truncated(&project_dir.join("GOAL.md"), 800);
    let plan = read_file_truncated(&project_dir.join("plan.md"), 400);
    let status = read_file_truncated(&project_dir.join("status.md"), 500);
    let score = read_file_truncated(&project_dir.join("score.md"), 300);

    let mut domain = goal;
    if !plan.is_empty() {
        domain.push_str("\n\nPlan:\n");
        domain.push_str(&plan);
    }
    if !status.is_empty() {
        domain.push_str("\n\nStatus evidence:\n");
        domain.push_str(&status);
    }

    let metric = if score.is_empty() {
        "All objectives in GOAL.md completed".into()
    } else {
        score
    };

    AgentObjective::new(domain, metric)
}

fn read_file_truncated(path: &Path, max_chars: usize) -> String {
    fs::read_to_string(path)
        .map(|s| {
            let count = s.chars().count();
            if count > max_chars {
                let truncated: String = s.chars().take(max_chars).collect();
                format!("{truncated}…")
            } else {
                s
            }
        })
        .unwrap_or_default()
}

/// POST /reload to the supervisor and return the active worker port.
fn supervisor_reload(supervisor_port: u16, fallback_worker_port: u16) -> Result<u16, String> {
    let mut stream = TcpStream::connect(("127.0.0.1", supervisor_port))
        .map_err(|e| format!("supervisor connect: {e}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();

    let request = format!(
        "POST /reload HTTP/1.1\r\nHost: 127.0.0.1:{supervisor_port}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("supervisor write: {e}"))?;
    stream.flush().ok();

    let mut response = String::new();
    stream.read_to_string(&mut response).ok();

    let status: u16 = response
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    match status {
        200 => parse_reload_worker_port(&response),
        204 => Ok(fallback_worker_port),
        _ => Err(format!("supervisor /reload returned HTTP {status}")),
    }
}

fn parse_reload_worker_port(response: &str) -> Result<u16, String> {
    let body = response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .or_else(|| response.split_once("\n\n").map(|(_, body)| body))
        .ok_or_else(|| "supervisor /reload response missing body".to_string())?;
    let json: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("supervisor /reload JSON: {e}"))?;
    let port = json
        .get("active")
        .and_then(|active| active.get("worker_port"))
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "supervisor /reload response missing active.worker_port".to_string())?;
    u16::try_from(port).map_err(|_| format!("supervisor /reload worker_port out of range: {port}"))
}

/// Poll the worker health endpoint until it responds OK or the timeout expires.
fn wait_for_worker_healthy(port: u16, timeout_secs: u64) -> bool {
    let client = WorkerClient::new(port);
    let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);
    while std::time::Instant::now() < deadline {
        if client.health().unwrap_or(false) {
            return true;
        }
        thread::sleep(Duration::from_millis(500));
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_attempt_labels_preserve_original_and_number_retries() {
        assert_eq!(retry_attempt_label("plan", 0), "plan");
        assert_eq!(retry_attempt_label("plan", 1), "plan-retry-1");
        assert_eq!(retry_attempt_label("execute-2", 3), "execute-2-retry-3");
    }

    #[test]
    fn project_loop_modes_distinguish_planning_execution_and_worker_certification() {
        assert_eq!(project_turn_mode(0), LoopMode::ProjectPlanning);
        assert_eq!(project_turn_mode(1), LoopMode::ProjectExecution);
        assert_eq!(project_turn_mode(9), LoopMode::ProjectExecution);
        assert_eq!(
            LoopMode::WorkerCertification.label(),
            "worker_certification"
        );
    }

    #[test]
    fn project_prompts_do_not_claim_to_be_worker_certification() {
        let goal = "Ship the next deterministic runtime slice.";
        let working_dir = Path::new("/workspace/project");
        let planning = planning_prompt(goal, 0, 1, working_dir, None, None, None, None);
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
    fn certification_objective_uses_truncated_project_evidence() {
        let root = std::env::temp_dir().join(format!("canon-loop-cert-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("GOAL.md"), "goal\n".repeat(1000)).unwrap();
        fs::write(root.join("plan.md"), "plan\n".repeat(1000)).unwrap();
        fs::write(root.join("status.md"), "status\n".repeat(1000)).unwrap();
        fs::write(root.join("score.md"), "score\n".repeat(1000)).unwrap();

        let objective = build_cert_objective(&root);
        assert!(objective.domain_hint.contains("Plan:"));
        assert!(objective.domain_hint.contains("Status evidence:"));
        assert!(objective.domain_hint.ends_with('…'));
        assert!(objective.success_metric.ends_with('…'));
        assert!(objective.domain_hint.chars().count() < 1_800);
        assert!(objective.success_metric.chars().count() <= 301);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn supervisor_reload_response_parses_fresh_worker_port() {
        let response = "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\r\n{\"ok\":true,\"active\":{\"generation\":2,\"worker_port\":41234}}";

        assert_eq!(parse_reload_worker_port(response).unwrap(), 41234);
    }

    #[test]
    fn supervisor_reload_response_rejects_missing_worker_port() {
        let response = "HTTP/1.1 200 OK\r\n\r\n{\"ok\":true,\"active\":{\"generation\":2}}";

        assert!(parse_reload_worker_port(response).is_err());
    }
}
