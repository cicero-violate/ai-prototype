use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::agent::config::AgentLoopConfig;
use crate::agent::router::RouterClient;
use crate::agent::sse::ChunkLogger;
use crate::capability::llm::openai::{OpenAiChatRequest, OpenAiMessage};

/// Outer cycle loop driver. Mirrors agentLoop / runCycle from chatgpt-agent-loop/agent-loop.mjs.
///
/// Reads GOAL.md, syncs MCP workspace, then runs an infinite cycle of
/// 1 planning turn + N execute turns with per-turn retry logic.
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
            Ok(_) => eprintln!("agent: MCP workspace synced ({})", self.config.mcp_connector_url),
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
        let goal_path = self.config.project_dir.join("GOAL.md");

        if !goal_path.exists() {
            eprintln!("[{tag}] no GOAL.md — loop disabled");
            return;
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
        let goal = read_goal_file(&self.config.project_dir)?;
        let total_turns = self.config.execute_turns + 1; // 1 plan + N execute

        for turn in 0..total_turns {
            let is_planning = turn == 0;
            let label = if is_planning {
                "plan".to_string()
            } else {
                format!("execute-{turn}")
            };

            let prompt = if is_planning {
                planning_prompt(&goal, agent_id, self.config.agent_count, &self.config.working_dir)
            } else {
                execute_prompt(turn, agent_id, self.config.agent_count)
            };

            eprintln!(
                "[{tag}] ── turn {}/{total_turns}: {label} (cycle {cycle_num})",
                turn + 1
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

                let attempt_label = if attempt == 0 {
                    label.clone()
                } else {
                    format!("{label}-retry-{attempt}")
                };

                let mut logger = ChunkLogger::new(
                    &self.config.sse_chunks_dir,
                    tag,
                    cycle_num,
                    &attempt_label,
                )
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

        Ok(())
    }
}

// ── Prompt builders ───────────────────────────────────────────────────────────

fn planning_prompt(
    goal: &str,
    agent_id: u32,
    agent_count: u32,
    working_dir: &Path,
) -> String {
    let agent_line = agent_identity(agent_id, agent_count);
    format!(
        "{agent_line}\n\
         You are doing the planning turn for this agent loop.\n\n\
         ## WORKING DIRECTORY\n`{dir}`\n\
         All shell commands must run relative to this directory unless the task explicitly requires otherwise.\n\n\
         ## GOAL\n{goal}\n\n\
         Create or update `plan.md` with the current implementation plan. \
         Create or update `score.md` with current progress and scoring. \
         Keep this turn focused on planning and scoring, and commit the planning/scoring changes at the end of the turn.",
        dir = working_dir.display(),
    )
}

fn execute_prompt(turn_num: u32, agent_id: u32, agent_count: u32) -> String {
    let agent_line = agent_identity(agent_id, agent_count);
    format!(
        "{agent_line}\n\
         You are executing implementation step {turn_num} of this agent loop.\n\n\
         Read `plan.md`, execute the next concrete part of that plan, \
         run the relevant tests/checks, update `plan.md` if the plan changes, \
         update `score.md` with progress and scoring, and commit all turn changes at the end of the turn."
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
