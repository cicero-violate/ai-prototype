//! Prompt builders for agent LLM turns.

/// Shared system prompt for all agent turns.
pub fn system_prompt() -> &'static str {
    "You are a Canon agent driving a deterministic runtime. \
     The runtime decides success — not you. \
     Output HUMAN_REVIEW_REQUIRED if you need a human. \
     Do not claim the objective is complete; the runtime phase does that. \
     During certification turns, do not call tools, browse, echo, or fetch time. \
     Return plain text only."
}

const CERTIFICATION_OUTPUT_RULE: &str =
    "Do not call tools, browse, echo, or fetch time. Return plain text only. \
     Output exactly one VERDICT: pass or VERDICT: fail line as the final line.";

/// First turn: ask for a step-by-step plan.
///
/// `score_report` is the content of SCORE_REPORT.md if available. When
/// present the structural quality scores are injected into the prompt so the
/// agent can factor them into its plan without needing a separate file-read
/// tool call.
pub fn planning_prompt(
    domain_hint: &str,
    success_metric: &str,
    score_report: Option<&str>,
) -> String {
    let score_section = match score_report {
        Some(report) => format!(
            "\n\nStructural quality scores (SCORE_REPORT.md — graph-derived, updated each commit):\n\
             ```\n{report}\n```"
        ),
        None => String::new(),
    };
    format!(
        "Domain: {domain_hint}\n\
         Success metric: {success_metric}{score_section}\n\n\
         Produce a numbered plan. For each step state the action kind \
         (llm_turn | direct_tool_call | observation_ingress | process_execution) \
         and what outcome advances the runtime phase."
    )
}

pub fn analysis_prompt(domain: &str, goal: &str) -> String {
    format!(
        "Analysis phase — domain: {domain}\n\
         Goal: {goal}\n\n\
         Analyze the objective and identify the facts, assumptions, risks, and unknowns \
         that matter for deciding whether the work should proceed. \
         If you need human input, output HUMAN_REVIEW_REQUIRED.\n\
         {CERTIFICATION_OUTPUT_RULE}"
    )
}

pub fn judgment_prompt(domain: &str, analysis: &str) -> String {
    format!(
        "Judgment phase — domain: {domain}\n\
         Analysis:\n{analysis}\n\n\
         Judge whether the analysis supports moving forward. State the decision, \
         confidence, and any blocking concerns. \
         If you need human input, output HUMAN_REVIEW_REQUIRED.\n\
         {CERTIFICATION_OUTPUT_RULE}"
    )
}

pub fn plan_prompt(domain: &str, judgment: &str) -> String {
    format!(
        "Plan phase — domain: {domain}\n\
         Judgment:\n{judgment}\n\n\
         Produce the concrete plan of work implied by the judgment. Include ordered \
         tasks, expected receipts, and completion criteria. \
         If you need human input, output HUMAN_REVIEW_REQUIRED.\n\
         {CERTIFICATION_OUTPUT_RULE}"
    )
}

pub fn eval_prompt(domain: &str, metric: &str, execution: &str) -> String {
    format!(
        "Eval phase — domain: {domain}\n\
         Success metric: {metric}\n\
         Execution context:\n{execution}\n\n\
         Evaluate the completed work against the success metric. Call out any \
         remaining gaps or reasons the result should not be accepted. \
         If you need human input, output HUMAN_REVIEW_REQUIRED.\n\
         {CERTIFICATION_OUTPUT_RULE}"
    )
}

pub fn recovery_prompt(domain: &str, failure: &str, target_phase: &str) -> String {
    format!(
        "Recovery phase — domain: {domain}\n\
         Failure: {failure}\n\
         Target phase: {target_phase}\n\n\
         Explain the recovery work needed to return the objective to a coherent \
         state. Focus on the semantic repair, not runtime protocol details. \
         If you need human input, output HUMAN_REVIEW_REQUIRED.\n\
         {CERTIFICATION_OUTPUT_RULE}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn certification_prompts_forbid_tool_calls() {
        assert!(system_prompt().contains("do not call tools"));
        assert!(analysis_prompt("domain", "goal").contains("Return plain text only"));
        assert!(recovery_prompt("domain", "failure", "Execute").contains("Do not call tools"));
    }

    #[test]
    fn certification_prompts_require_final_verdict() {
        let prompts = [
            analysis_prompt("domain", "goal"),
            judgment_prompt("domain", "analysis"),
            plan_prompt("domain", "judgment"),
            eval_prompt("domain", "metric", "execution"),
            recovery_prompt("domain", "failure", "Execute"),
        ];

        for prompt in prompts {
            assert!(prompt.contains("VERDICT: pass"));
            assert!(prompt.contains("final line"));
        }
    }
}
