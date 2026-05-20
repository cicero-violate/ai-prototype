//! Prompt construction and per-cycle planning helpers.

use std::fs;
use std::path::Path;

use crate::process::agent::config::AgentLoopConfig;

use super::http::agent_command_url;
use super::learning::{load_mcp_feedback, load_policy_feedback};

pub(super) struct RunCyclePlan {
    pub(super) is_spawned: bool,
    pub(super) command_url: Option<String>,
    pub(super) total_turns: u32,
    pub(super) turn_offset: u32,
    pub(super) goal: String,
}

pub(super) fn prepare_run_cycle(config: &AgentLoopConfig) -> Result<RunCyclePlan, String> {
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

pub(super) struct TurnPromptContext {
    pub(super) effective_turn: u32,
    pub(super) turn_mode: LoopMode,
    pub(super) label: String,
    pub(super) prompt: String,
}

pub(super) fn build_turn_prompt_context(
    turn: u32,
    turn_offset: u32,
    is_spawned: bool,
    goal: &str,
    agent_id: u32,
    agent_count: u32,
    working_dir: &Path,
    domain: Option<&str>,
    metric: Option<&str>,
    plan_node_id: Option<&str>,
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
            plan_node_id,
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

pub(super) fn build_project_planning_prompt(
    goal: &str,
    agent_id: u32,
    agent_count: u32,
    working_dir: &Path,
) -> String {
    let auto_refactor_report = auto_refactor_summary(working_dir);
    let policy_feedback = load_policy_feedback(working_dir);
    let mcp_feedback = load_mcp_feedback(working_dir);
    planning_prompt(
        goal,
        agent_id,
        agent_count,
        working_dir,
        None,
        None,
        None,
        auto_refactor_report.as_deref(),
        policy_feedback.as_deref(),
        mcp_feedback.as_deref(),
    )
}

pub(super) fn planning_prompt(
    goal: &str,
    agent_id: u32,
    agent_count: u32,
    working_dir: &Path,
    domain: Option<&str>,
    metric: Option<&str>,
    _score_report: Option<&str>,
    auto_refactor_report: Option<&str>,
    policy_feedback: Option<&str>,
    mcp_feedback: Option<&str>,
) -> String {
    let agent_line = agent_identity(agent_id, agent_count);
    let focus_block = match (domain, metric) {
        (Some(d), Some(m)) => {
            format!("## YOUR SPECIFIC TASK\n{d}\n\n## SUCCESS CRITERION\n{m}\n\n")
        }
        (Some(d), None) => format!("## YOUR SPECIFIC TASK\n{d}\n\n"),
        _ => String::new(),
    };
    let score_block = String::new();
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
    let mcp_error_block = match mcp_feedback {
        Some(feedback) if !feedback.trim().is_empty() => format!(
            "## MCP TOOL FEEDBACK (last cycle errors — avoid repeating these mistakes)\n\
             {feedback}\n"
        ),
        _ => String::new(),
    };
    format!(
        "{agent_line}\n\
         You are doing the planning turn for this agent loop.\n\n\
         ## WORKING DIRECTORY\n`{dir}`\n\
         Workspace command actions must run relative to this directory unless the task explicitly requires otherwise.\n\n\
         {focus_block}\
         ## GOAL\n{goal}\n\n\
         {score_block}\
         {auto_refactor_block}\
         {policy_block}\
         {mcp_error_block}\
         ## HOW MINI-AGENT DISPATCH WORKS\n\
         After this planning turn finishes, the scheduler reads `state/plan.json` and **spawns one mini-agent per ready node in parallel**. \
         A ready node is a `pending` node whose every dependency node is already `done`. \
         Nodes with no incoming edges are immediately ready. \
         Each mini-agent receives the node `title` as its task and `description` as its success criterion, runs autonomously, and commits its result. \
         The more work you decompose into independent DAG nodes, the more parallelism you get — up to one agent per node.\n\n\
         ## OPTIMIZATION OBJECTIVE\n\
         Decompose the next cycle's work into parallel tasks and publish them as a DAG to maximize score gain per wall-clock cycle. \
         Treat each node's `score_axes` as the current objective surface. \
         Let each possible work item be x, each score axis be p_i(x) in [0,10], and each optional axis weight be w_i >= 0. \
         Prefer tasks by expected score gain under:\n\
         x* = argmax_x (product_i p_i(x)^w_i)^(1 / sum_i w_i).\n\
         Prioritize work that raises the lowest justified score axes first. \
         Do not raise scores without evidence — plan tasks that produce evidence, then let the evidence justify the score.\n\n\
         **Reconnaissance — do all of these before touching the DAG:**\n\
         1. Inspect the `project` landmark to see available plan and score actions:\n\
            `inspect_landmark` → `{{\"landmark_id\": \"project\", \"intent\": \"discover plan and score actions\"}}`\n\
         2. Read the current work DAG:\n\
            `call_action` → `{{\"action\": \"project:plan_read\", \"parameters\": {{}}, \"intent\": \"read current plan DAG\"}}`\n\
            Inspect `ready_node_ids`, pending, running, and failed nodes.\n\
         3. Read the evidence references already attached to current DAG nodes.\n\
         4. Inspect `state/rustc/auto-refactor/*.graph-editor-plan.json` when present — use those to identify graph-backed refactor candidates.\n\
         5. Inspect the specific files, tests, fixtures, and evidence paths that candidate nodes would touch.\n\n\
         **Build or update the DAG — use `call_action` with action `project:plan_update`:**\n\
         - `op=replace` to publish a fresh DAG for this cycle, or `op=upsert_node` / `op=add_edge` to extend an existing one.\n\
         - Each node must have:\n\
           - `id`: short kebab-case slug, e.g. `\"fix-scc-layer\"`\n\
           - `title`: the task in one line, e.g. `\"Fix SCC layer gate validation\"`\n\
           - `description`: the **exact success criterion** — what command output, test result, or file state proves this node is done. Mini-agents receive this verbatim as their success criterion.\n\
           - `files`: list of files the node touches (scopes the mini-agent's work)\n\
           - `score_axes`: axes this node is expected to improve, e.g. `[\"Simplicity\", \"Structure\"]`\n\
           - `evidence`: usually empty at planning time; mini-agents append references when they complete work.\n\
         - Add edges with `op=add_edge` — `{{\"from\": \"A\", \"to\": \"B\"}}` means B cannot start until A is `done`.\n\
         - Only add nodes for work that is concrete, bounded, and independently verifiable.\n\
         - Each parallel node must write detailed evidence to a disjoint file under `state/agent-evidence/` and then call `project:plan_update` with `op=append_evidence`.\n\
         - Do not re-add nodes already marked `done`. Only reset a `failed` node if you have a new approach.\n\n\
         **After updating the DAG:**\n\
         Commit all changes at the end of this turn.",
        dir = working_dir.display(),
    )
}

pub(super) fn auto_refactor_summary(project_dir: &Path) -> Option<String> {
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

pub(super) fn summarize_auto_refactor_plan(path: &Path, text: &str) -> Option<String> {
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

pub(super) fn json_string_field(text: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let start = text.find(&needle)? + needle.len();
    let after_colon = text[start..].find(':')? + start + 1;
    parse_json_string_at(text, after_colon)
}

pub(super) fn json_usize_field(text: &str, key: &str) -> Option<usize> {
    let needle = format!("\"{key}\"");
    let start = text.find(&needle)? + needle.len();
    let after_colon = text[start..].find(':')? + start + 1;
    let rest = text[after_colon..].trim_start();
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

pub(super) fn json_string_values_after_key(text: &str, key: &str, limit: usize) -> Vec<String> {
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

pub(super) fn parse_json_string_at(text: &str, offset: usize) -> Option<String> {
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

pub(super) fn json_array_object_count(text: &str, key: &str) -> Option<usize> {
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

pub(super) fn spawned_prompt(
    domain: &str,
    metric: &str,
    plan_node_id: Option<&str>,
    step: u32,
    working_dir: &Path,
) -> String {
    let dir = working_dir.display();
    let evidence_protocol = match plan_node_id {
        Some(node_id) => format!(
            "5. Write detailed evidence to `state/agent-evidence/{node_id}.md`.\n\
             6. Append an evidence reference with `call_action` action `project:plan_update`, parameters \
             `{{\"op\":\"append_evidence\",\"node_id\":\"{node_id}\",\"evidence\":{{\"path\":\"state/agent-evidence/{node_id}.md\",\"kind\":\"validation\",\"summary\":\"<one-line result>\"}}}}`.\n\
             7. Do not modify shared planning or score files.\n\
             8. When the criterion is met and evidence is attached: commit all changes, then stop. The scheduler will mark your task done automatically.\n\
             9. If the criterion cannot be met: write the blocker to `state/agent-evidence/{node_id}.md`, append evidence with `kind` set to `blocker`, then stop without committing. The scheduler will mark your task failed."
        ),
        None => {
            "5. Write detailed evidence to a task-specific file under `state/agent-evidence/`.\n\
             6. Do not modify shared planning or score files.\n\
             7. When the criterion is met: commit all changes, then stop.\n\
             8. If the criterion cannot be met: write the blocker to the evidence file, then stop without committing."
                .to_string()
        }
    };
    if step == 1 {
        format!(
            "You are a mini-agent with a single bounded task. You were spawned by the scheduler.\n\n\
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
             3. Use `call_action` with action `workspace:apply_patch` for all file edits.\n\
             4. Every tool call must include a non-empty `intent` field explaining why.\n\
             {evidence_protocol}"
        )
    } else {
        format!(
            "You are a mini-agent on step {step} of a bounded task. You were spawned by the scheduler.\n\n\
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
             5. Commit only when the criterion is met and evidence is attached. Do not commit partial or failing work."
        )
    }
}

pub(super) fn execute_prompt(turn_num: u32, agent_id: u32, agent_count: u32) -> String {
    let agent_line = agent_identity(agent_id, agent_count);
    format!(
        "{agent_line}\n\
         You are executing implementation step {turn_num} of this agent loop.\n\n\
         The DAG scheduler found no ready nodes this cycle — the DAG is either empty, \
         all nodes are blocked by unfinished dependencies, or all nodes are already done.\n\n\
         Read the current DAG via `call_action project:plan_read` to understand what is pending, blocked, failed, and already evidenced.\n\n\
         Choose one of the following based on what you find:\n\
         - **Blocked DAG nodes**: identify and resolve the blocking dependency directly (implement the prerequisite, fix the failing test, produce the missing evidence).\n\
         - **Failed DAG nodes**: diagnose the failure, fix the root cause, reset the node to `pending` via `call_action project:plan_update` with `op=set_status`, then let the next planning turn re-dispatch it.\n\
         - **Empty DAG**: do direct implementation work that moves the lowest justified score axis. Pick one concrete, bounded task — a specific file, function, or test. Use `call_action workspace:apply_patch` for all file edits.\n\n\
         After implementation:\n\
         - Run the relevant validation or test command and fix any failures.\n\
         - Write detailed evidence to a task-specific file under `state/agent-evidence/`.\n\
         - If the work maps to a DAG node, append an evidence reference with `project:plan_update` op `append_evidence`.\n\
         - Commit only when checks pass. If checks cannot pass, document the blocker in the evidence file and do not commit.\n\n\
         If there are two or more independent sub-tasks (different files, no shared state), \
         spawn a mini-agent for one via `canon_spawn_agent` with:\n\
         - `domain`: the specific file or function to implement\n\
         - `metric`: the exact test or check that must pass\n\
         Implement the first task yourself — do not spawn without also making progress yourself."
    )
}

pub(super) fn agent_label(
    agent_id: u32,
    agent_count: u32,
    single_agent: &'static str,
    multi_agent: impl FnOnce(u32, u32) -> String,
) -> String {
    if agent_count > 1 {
        multi_agent(agent_id, agent_count)
    } else {
        single_agent.to_string()
    }
}

pub(super) fn agent_identity(agent_id: u32, agent_count: u32) -> String {
    agent_label(
        agent_id,
        agent_count,
        "You are the agent for this project.",
        |agent_id, agent_count| {
            format!("You are **Agent {agent_id}** (one of {agent_count} parallel agents).")
        },
    )
}

pub(super) fn agent_tag(agent_id: u32, agent_count: u32) -> String {
    agent_label(agent_id, agent_count, "agent", |agent_id, _| {
        format!("agent-{agent_id}")
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LoopMode {
    ProjectPlanning,
    ProjectExecution,
}

impl LoopMode {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::ProjectPlanning => "project_planning",
            Self::ProjectExecution => "project_execution",
        }
    }
}

pub(super) fn project_turn_mode(turn: u32) -> LoopMode {
    if turn == 0 {
        LoopMode::ProjectPlanning
    } else {
        LoopMode::ProjectExecution
    }
}

pub(super) fn retry_attempt_label(label: &str, attempt: u32) -> String {
    if attempt == 0 {
        label.to_string()
    } else {
        format!("{label}-retry-{attempt}")
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub(super) fn read_goal_file(project_dir: &Path) -> Result<String, String> {
    let path = project_dir.join("GOAL.md");
    fs::read_to_string(&path).map_err(|e| format!("read GOAL.md: {e}"))
}
