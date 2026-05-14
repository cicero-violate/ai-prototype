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

struct CertificationPrompt<'a> {
    phase: &'static str,
    domain: &'a str,
    context: String,
    instruction: &'static str,
}

fn certification_prompt(spec: CertificationPrompt<'_>) -> String {
    format!(
        "{} phase — domain: {}\n{}\n\n{} \
         If you need human input, output HUMAN_REVIEW_REQUIRED.\n\
         {CERTIFICATION_OUTPUT_RULE}",
        spec.phase, spec.domain, spec.context, spec.instruction
    )
}

fn certification_phase_prompt(
    phase: &'static str,
    domain: &str,
    context: String,
    instruction: &'static str,
) -> String {
    certification_prompt(CertificationPrompt {
        phase,
        domain,
        context,
        instruction,
    })
}

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
    certification_phase_prompt(
        "Analysis",
        domain,
        format!("Goal: {goal}"),
        "Analyze the objective and identify the facts, assumptions, risks, and unknowns that matter for deciding whether the work should proceed.",
    )
}

pub fn judgment_prompt(domain: &str, analysis: &str) -> String {
    certification_phase_prompt(
        "Judgment",
        domain,
        format!("Analysis:\n{analysis}"),
        "Judge whether the analysis supports moving forward. State the decision, confidence, and any blocking concerns.",
    )
}

pub fn plan_prompt(domain: &str, judgment: &str) -> String {
    certification_phase_prompt(
        "Plan",
        domain,
        format!("Judgment:\n{judgment}"),
        "Produce the concrete plan of work implied by the judgment. Include ordered tasks, expected receipts, and completion criteria.",
    )
}

pub fn eval_prompt(domain: &str, metric: &str, execution: &str) -> String {
    certification_phase_prompt(
        "Eval",
        domain,
        format!("Success metric: {metric}\nExecution context:\n{execution}"),
        "Evaluate the completed work against the success metric. Call out any remaining gaps or reasons the result should not be accepted.",
    )
}

pub fn recovery_prompt(domain: &str, failure: &str, target_phase: &str) -> String {
    certification_phase_prompt(
        "Recovery",
        domain,
        format!("Failure: {failure}\nTarget phase: {target_phase}"),
        "Explain the recovery work needed to return the objective to a coherent state. Focus on the semantic repair, not runtime protocol details.",
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

    #[test]
    fn certification_prompt_formatter_preserves_phase_specific_content() {
        let prompts = [
            (
                analysis_prompt("domain", "goal"),
                "Analysis phase",
                ["Goal: goal"].as_slice(),
            ),
            (
                judgment_prompt("domain", "analysis"),
                "Judgment phase",
                ["Analysis:\nanalysis"].as_slice(),
            ),
            (
                plan_prompt("domain", "judgment"),
                "Plan phase",
                ["Judgment:\njudgment"].as_slice(),
            ),
            (
                eval_prompt("domain", "metric", "execution"),
                "Eval phase",
                ["Success metric: metric", "Execution context:\nexecution"].as_slice(),
            ),
            (
                recovery_prompt("domain", "failure", "Execute"),
                "Recovery phase",
                ["Failure: failure", "Target phase: Execute"].as_slice(),
            ),
        ];

        for (prompt, phase, phase_specific_content) in prompts {
            assert!(prompt.contains(phase));
            assert!(prompt.contains("domain: domain"));
            assert!(prompt.contains("HUMAN_REVIEW_REQUIRED"));
            assert!(prompt.contains("Do not call tools"));
            assert!(prompt.contains("VERDICT: pass"));
            assert!(prompt.contains("VERDICT: fail"));

            for content in phase_specific_content {
                assert!(prompt.contains(content));
            }
        }
    }
}
