//! Prompt construction and per-cycle planning helpers.

use std::fs;
use std::path::Path;

use crate::process::agent::config::AgentLoopConfig;

use super::http::agent_command_url;
use super::learning::load_policy_feedback;

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

pub(super) fn build_project_planning_prompt(
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

pub(super) fn planning_prompt(
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
         Workspace command actions must run relative to this directory unless the task explicitly requires otherwise.\n\n\
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

pub(super) fn spawned_prompt(domain: &str, metric: &str, step: u32, working_dir: &Path) -> String {
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
             3. Use `call_action` with action `workspace:apply_patch` for all file edits.\n\
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

pub(super) fn execute_prompt(turn_num: u32, agent_id: u32, agent_count: u32) -> String {
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
