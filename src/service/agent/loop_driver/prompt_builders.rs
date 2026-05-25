//! Prompt construction and per-cycle planning helpers.

use std::fs;
use std::path::Path;

use crate::service::agent::config::AgentLoopConfig;

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

#[expect(
    clippy::too_many_arguments,
    reason = "prompt assembly requires all turn context fields at one boundary"
)]
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
         {policy_block}\
         {mcp_error_block}\
         ## HOW EXECUTOR DISPATCH WORKS\n\
         After this planning turn finishes, the scheduler reads the TLog-projected plan read model and **spawns one executor per ready node in parallel**. \
         A ready node is projected `pending` whose every dependency node is projected `done`. \
         Nodes with no incoming edges are immediately ready. \
         Each executor receives the node `title` as its task and `description` as its success criterion, runs autonomously, and commits its result. \
         The more work you decompose into independent DAG nodes, the more parallelism you get — up to one executor per node.\n\n\
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
         4. Inspect the specific files, tests, fixtures, and evidence paths that candidate nodes would touch.\n\n\
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

pub(super) fn spawned_prompt(
    domain: &str,
    metric: &str,
    plan_node_id: Option<&str>,
    step: u32,
    working_dir: &Path,
) -> String {
    let dir = working_dir.display();
    let goal_block = match fs::read_to_string(working_dir.join("GOAL.md")) {
        Ok(goal) if !goal.trim().is_empty() => {
            format!("## PROJECT GOAL\n{goal}\n\n")
        }
        _ => String::new(),
    };
    let success_criterion = if metric.trim().is_empty() {
        "BLOCKER: no success criterion was supplied by the scheduler. Do not invent scope. Assess the missing criterion, write blocker evidence, attach it as blocker evidence when a plan node is available, and stop without committing."
    } else {
        metric
    };
    let evidence_protocol = match plan_node_id {
        Some(node_id) => format!(
            "5. Write detailed task evidence to `state/agent-evidence/{node_id}.md` with commands, validation output, artifact paths, counts, and samples.\n\
             6. When the success criterion is met, append that evidence file with `call_action` action `project:plan_update` using `op=append_evidence`, parameters \
             `{{\"op\":\"append_evidence\",\"node_id\":\"{node_id}\",\"evidence\":{{\"path\":\"state/agent-evidence/{node_id}.md\",\"kind\":\"validation\",\"summary\":\"<one-line validation result>\"}}}}`; this accepted evidence is the supervisor completion signal.\n\
             7. Commit only passing work: `git add -A && git commit -m \"<short description>\"`.\n\
             8. Capture the commit hash: `git rev-parse HEAD` → let this be COMMIT_HASH.\n\
             9. Append the commit as additional evidence with `call_action` action `project:plan_update`, parameters \
             `{{\"op\":\"append_evidence\",\"node_id\":\"{node_id}\",\"evidence\":{{\"path\":\"git/COMMIT_HASH\",\"kind\":\"commit\",\"summary\":\"COMMIT_HASH\"}}}}`\n\
             (replace COMMIT_HASH with the actual hash from step 8).\n\
             10. Do not directly edit `state/plan.json`, shared planning files, score files, or TLog files, and do not call `op=set_status`.\n\
             11. When the criterion is met: write the evidence file, append validation evidence, commit passing work, append commit evidence, then stop. The supervisor will decide completion.\n\
             12. If the criterion cannot be met: write the blocker to `state/agent-evidence/{node_id}.md`; do not commit; append evidence with `kind` set to `blocker` and summary describing the blocker, then stop."
        ),
        None => {
            "5. Commit only passing work: `git add -A && git commit -m \"<short description>\"`.\n\
             6. Do not directly edit `state/plan.json`, shared planning files, score files, or TLog files.\n\
             7. When the criterion is met: commit and stop. If it cannot be met: do not commit; document the blocker and stop."
                .to_string()
        }
    };
    if step == 1 {
        format!(
            "You are an executor with a single bounded task. You were spawned by the scheduler.\n\n\
             ## WORKING DIRECTORY\n\
             `{dir}`\n\
             All commands run from this directory.\n\n\
             {goal_block}\
             ## TASK\n\
             {domain}\n\n\
             ## SUCCESS CRITERION\n\
             {success_criterion}\n\n\
             ## PROTOCOL\n\
             1. Assess the current state against the success criterion before doing anything else.\n\
             2. Take the minimal actions needed to meet the criterion.\n\
             3. Use `call_action` with action `workspace:structural_edit` as the canonical editor for file edits. Use `workspace:apply_patch` only when the edit cannot be expressed as structural-editor ops.\n\
             4. Every tool call must include a non-empty `intent` field explaining why.\n\
             {evidence_protocol}"
        )
    } else {
        format!(
            "You are an executor on step {step} of a bounded task. You were spawned by the scheduler.\n\n\
             ## WORKING DIRECTORY\n\
             `{dir}`\n\n\
             {goal_block}\
             ## TASK\n\
             {domain}\n\n\
             ## SUCCESS CRITERION\n\
             {success_criterion}\n\n\
             ## PROTOCOL\n\
             1. Verify the current state against the success criterion first.\n\
             2. If not met: identify the specific gap, close it, then re-verify.\n\
             3. Use `call_action` with action `workspace:structural_edit` as the canonical editor for file edits. Use `workspace:apply_patch` only when the edit cannot be expressed as structural-editor ops.\n\
             4. Every tool call must include a non-empty `intent` field.\n\
             {evidence_protocol}"
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
         - **Failed DAG nodes**: diagnose the failure, fix the root cause, write blocker or recovery evidence, and let the supervisor/recovery policy append lifecycle events for re-dispatch.\n\
         - **Empty DAG**: do direct implementation work that moves the lowest justified score axis. Pick one concrete, bounded task — a specific file, function, or test. Use `call_action workspace:structural_edit` as the canonical editor; use `workspace:apply_patch` only when structural-editor ops cannot express the edit.\n\n\
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
