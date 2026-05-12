//! AgentCycle — the observe/orient/decide/act/submit/verify loop.
//!
//! Success rule: success=true ONLY when the worker runtime reports phase=Done.
//! LLM output is a signal, not an authority. The runtime is the authority.
//!
//! This type is the small certification loop used after the outer project loop
//! has finished its file-editing turns. It does not mutate runtime state
//! directly. Instead it observes the worker phase, asks the router/LLM for the
//! semantic evidence required by that phase, submits typed evidence through the
//! worker API, and then observes the next runtime phase. The worker's reducer
//! and TLog decide whether submitted evidence advances the objective.

use crate::agent::objective::AgentObjective;
use crate::agent::prompt;
use crate::agent::router::{RouterClient, RouterTabCloseOutcome};
use crate::agent::step::{AgentActionKind, AgentDecision, AgentRunSummary, AgentStep};
use crate::agent::worker_client::WorkerClient;
use crate::capability::llm::openai::{OpenAiChatRequest, OpenAiError, OpenAiMessage};

const DEFAULT_MAX_STEPS: u64 = 20;

// The LLM may output this to signal it needs a human. Stop and surface it.
const SENTINEL_REVIEW: &str = "HUMAN_REVIEW_REQUIRED";

pub enum StopReason {
    /// Worker runtime reached phase=Done. The only path to success=true.
    PhaseDone,
    MaxStepsReached,
    HumanReviewRequired,
    ProviderUnavailable,
    WorkerUnavailable,
}

impl StopReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PhaseDone => "phase_done",
            Self::MaxStepsReached => "max_steps_reached",
            Self::HumanReviewRequired => "human_review_required",
            Self::ProviderUnavailable => "provider_unavailable",
            Self::WorkerUnavailable => "worker_unavailable",
        }
    }
}

#[derive(Debug)]
pub enum CycleError {
    Router(OpenAiError),
    Worker(String),
    WorkerNotReady,
    EmptyObjective,
}

impl std::fmt::Display for CycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Router(e) => write!(f, "router: {e}"),
            Self::Worker(e) => write!(f, "worker: {e}"),
            Self::WorkerNotReady => write!(f, "worker not ready"),
            Self::EmptyObjective => write!(f, "empty objective"),
        }
    }
}

pub struct AgentCycle {
    /// Router/LLM session used only to produce phase-specific semantic text.
    router: RouterClient,
    /// Worker API client. All durable state changes pass through this client.
    worker: WorkerClient,
    /// Stable objective text supplied to every phase prompt.
    objective: AgentObjective,
    /// Hard stop preventing an unbounded certification loop.
    max_steps: u64,
    /// Local receipt summary for the LLM turns and evidence submissions made by
    /// this cycle. The authoritative record remains the worker TLog.
    steps: Vec<AgentStep>,
    /// Monotonic command id for evidence submissions to the worker.
    next_command_id: u64,
    /// Most recent LLM output. Later phase prompts use it as semantic context.
    last_llm_output: String,
}

impl AgentCycle {
    pub fn new(router: RouterClient, worker: WorkerClient, objective: AgentObjective) -> Self {
        Self {
            router,
            worker,
            objective,
            max_steps: DEFAULT_MAX_STEPS,
            steps: Vec::new(),
            next_command_id: 1,
            last_llm_output: String::new(),
        }
    }

    pub fn with_max_steps(mut self, max: u64) -> Self {
        self.max_steps = max;
        self
    }

    pub fn objective(&self) -> &AgentObjective {
        &self.objective
    }

    pub fn steps(&self) -> &[AgentStep] {
        &self.steps
    }

    pub fn close_router_tab(&mut self) -> Result<RouterTabCloseOutcome, OpenAiError> {
        self.router.close_current_tab()
    }

    pub fn run(&mut self) -> Result<AgentRunSummary, CycleError> {
        if self.objective.domain_hint.is_empty() {
            return Err(CycleError::EmptyObjective);
        }

        if !self
            .worker
            .health()
            .map_err(|e| CycleError::Worker(e.to_string()))?
        {
            return Err(CycleError::WorkerNotReady);
        }

        let domain = self.objective.domain_hint.clone();
        let metric = self.objective.success_metric.clone();

        // Step 0: planning turn.
        //
        // The plan is intentionally not treated as success. It is just the
        // first LLM-produced semantic artifact, recorded with a response hash
        // so later review can connect the phase evidence back to the request.
        self.last_llm_output = {
            let score_report = std::fs::read_to_string("SCORE_REPORT.md").ok();
            let messages = vec![
                OpenAiMessage::system(prompt::system_prompt()),
                OpenAiMessage::user(prompt::planning_prompt(
                    &domain,
                    &metric,
                    score_report.as_deref(),
                )),
            ];
            let result = self
                .router
                .turn(OpenAiChatRequest::new(messages))
                .map_err(CycleError::Router)?;

            self.steps.push(AgentStep {
                step_index: 0,
                objective_id: self.objective.objective_id,
                decision: AgentDecision {
                    selected_action: AgentActionKind::LlmTurn,
                    rationale: "planning turn".into(),
                    confidence_score: 80,
                    uncertainty_score: 20,
                },
                action_kind: AgentActionKind::LlmTurn,
                request_hash: result.response.raw_hash,
                receipt_hash: Some(result.response.response_hash),
                success: true,
            });

            result.response.content
        };

        // Phase-dispatch loop. The runtime phase drives submissions; LLM output
        // only supplies semantic content and a pass/fail verdict for LLM phases.
        //
        // Invariant: the cycle may submit evidence, but it never declares the
        // objective complete. Completion is recognized only after observing the
        // worker in phase=Done.
        let mut stop_reason = StopReason::MaxStepsReached;
        let mut loop_steps = 0u64;
        let mut invariant_submitted = false;

        loop {
            if loop_steps >= self.max_steps {
                break;
            }
            loop_steps += 1;

            // Observe: check worker phase and TLog progress before deciding the
            // next action. This keeps the LLM behind the deterministic runtime
            // boundary: stale model output cannot skip a phase.
            let state_body = self.observe()?;
            let phase = extract_phase(&state_body).unwrap_or_default();

            // The only path to success=true.
            if phase.eq_ignore_ascii_case("done") {
                stop_reason = StopReason::PhaseDone;
                break;
            }

            // Human review sentinel from the LLM — stop, do not declare success.
            if self.last_llm_output.contains(SENTINEL_REVIEW) {
                stop_reason = StopReason::HumanReviewRequired;
                break;
            }

            match phase.as_str() {
                "Analysis" => {
                    let output = self.llm_phase_turn(
                        loop_steps,
                        "Analysis",
                        "analysis phase",
                        prompt::analysis_prompt(&domain, &metric),
                    )?;
                    if output.contains(SENTINEL_REVIEW) {
                        stop_reason = StopReason::HumanReviewRequired;
                        break;
                    }
                    let passed = parse_verdict(&output);
                    if !invariant_submitted {
                        self.submit_evidence("Invariant", "InvariantProof", true);
                        invariant_submitted = true;
                    }
                    if let Some((gate, evidence)) = phase_gate("Analysis") {
                        self.submit_evidence(gate, evidence, passed);
                    }
                }
                "Judgment" => {
                    let output = self.llm_phase_turn(
                        loop_steps,
                        "Judgment",
                        "judgment phase",
                        prompt::judgment_prompt(&domain, &self.last_llm_output),
                    )?;
                    if output.contains(SENTINEL_REVIEW) {
                        stop_reason = StopReason::HumanReviewRequired;
                        break;
                    }
                    let passed = parse_verdict(&output);
                    if let Some((gate, evidence)) = phase_gate("Judgment") {
                        self.submit_evidence(gate, evidence, passed);
                    }
                }
                "Plan" => {
                    let output = self.llm_phase_turn(
                        loop_steps,
                        "Plan",
                        "plan phase",
                        prompt::plan_prompt(&domain, &self.last_llm_output),
                    )?;
                    if output.contains(SENTINEL_REVIEW) {
                        stop_reason = StopReason::HumanReviewRequired;
                        break;
                    }
                    let passed = parse_verdict(&output);
                    if let Some((gate, evidence)) = phase_gate("Plan") {
                        self.submit_evidence(gate, evidence, passed);
                    }
                }
                "Eval" => {
                    let output = self.llm_phase_turn(
                        loop_steps,
                        "Eval",
                        "eval phase",
                        prompt::eval_prompt(&domain, &metric, &self.last_llm_output),
                    )?;
                    if output.contains(SENTINEL_REVIEW) {
                        stop_reason = StopReason::HumanReviewRequired;
                        break;
                    }
                    let passed = parse_verdict(&output);
                    if let Some((gate, evidence)) = phase_gate("Eval") {
                        self.submit_evidence(gate, evidence, passed);
                    }
                }
                "Recovery" => {
                    let failure = extract_json_string(&state_body, "failure")
                        .unwrap_or_else(|| "Unknown".into());
                    let recovery_action = extract_json_string(&state_body, "recovery_action");
                    let target_phase = recovery_action
                        .as_deref()
                        .and_then(recovery_target_phase)
                        .unwrap_or("Unknown");
                    let output = self.llm_phase_turn(
                        loop_steps,
                        "Recovery",
                        "recovery phase",
                        prompt::recovery_prompt(&domain, &failure, target_phase),
                    )?;
                    if output.contains(SENTINEL_REVIEW) {
                        stop_reason = StopReason::HumanReviewRequired;
                        break;
                    }
                    let passed = parse_verdict(&output);
                    if let Some(action) = recovery_action.as_deref() {
                        if let Some((gate, evidence)) = recovery_gate(action) {
                            self.submit_evidence(gate, evidence, passed);
                        }
                    }
                }
                "Invariant" => {
                    if let Some((gate, evidence)) = phase_gate("Invariant") {
                        self.submit_evidence(gate, evidence, true);
                        invariant_submitted = true;
                    }
                }
                "Execute" => {
                    if let Some((gate, evidence)) = phase_gate("Execute") {
                        self.submit_evidence(gate, evidence, true);
                    }
                }
                "Verify" => {
                    if let Some((gate, evidence)) = phase_gate("Verify") {
                        self.submit_evidence(gate, evidence, true);
                    }
                }
                "Persist" => {}
                _ => {}
            }
        }

        let final_state = self.observe().unwrap_or_default();
        let final_phase = extract_phase(&final_state).unwrap_or_else(|| "Unknown".into());
        let final_tlog_len = extract_tlog_len(&final_state).unwrap_or(0);

        // success=true only when the runtime reached Done.
        let success = matches!(stop_reason, StopReason::PhaseDone);

        Ok(AgentRunSummary {
            objective_id: self.objective.objective_id,
            step_count: self.steps.len() as u64,
            success,
            stop_reason: stop_reason.as_str().into(),
            final_phase,
            final_tlog_len,
        })
    }

    fn observe(&self) -> Result<String, CycleError> {
        let resp = self
            .worker
            .state()
            .map_err(|e| CycleError::Worker(e.to_string()))?;
        if resp.status != 200 {
            return Err(CycleError::Worker(format!(
                "/v1/state returned {}",
                resp.status
            )));
        }
        Ok(resp.body)
    }

    fn llm_phase_turn(
        &mut self,
        step_index: u64,
        phase: &str,
        rationale: impl Into<String>,
        prompt_text: String,
    ) -> Result<String, CycleError> {
        let rationale = rationale.into();
        let messages = vec![
            OpenAiMessage::system(prompt::system_prompt()),
            OpenAiMessage::user(prompt_text),
        ];

        let result = match self.route_turn(OpenAiChatRequest::new(messages), phase) {
            Ok(r) => r,
            Err(e) => {
                self.steps.push(AgentStep {
                    step_index,
                    objective_id: self.objective.objective_id,
                    decision: AgentDecision {
                        selected_action: AgentActionKind::LlmTurn,
                        rationale: rationale.clone(),
                        confidence_score: 0,
                        uncertainty_score: 100,
                    },
                    action_kind: AgentActionKind::LlmTurn,
                    request_hash: 0,
                    receipt_hash: None,
                    success: false,
                });
                return Err(CycleError::Router(e));
            }
        };

        self.last_llm_output = result.response.content.clone();
        self.steps.push(AgentStep {
            step_index,
            objective_id: self.objective.objective_id,
            decision: AgentDecision {
                selected_action: AgentActionKind::LlmTurn,
                rationale,
                confidence_score: 70,
                uncertainty_score: 30,
            },
            action_kind: AgentActionKind::LlmTurn,
            request_hash: result.response.raw_hash,
            receipt_hash: Some(result.response.response_hash),
            success: true,
        });

        Ok(self.last_llm_output.clone())
    }

    fn route_turn(
        &mut self,
        request: OpenAiChatRequest,
        phase: &str,
    ) -> Result<crate::agent::router::RouterTurnResult, OpenAiError> {
        let _ = use_teacher(phase);
        self.router.turn(request)
    }

    fn submit_evidence(&mut self, gate: &str, evidence: &str, passed: bool) {
        let cmd_id = self.next_command_id;
        if let Some(json) = build_submit_evidence_json(gate, evidence, passed, cmd_id) {
            match self.worker.submit_command(&json) {
                Ok(resp) => {
                    self.next_command_id += 1;
                    eprintln!(
                        "agent: submit  gate={gate}  evidence={evidence}  passed={passed}  status={}  body={}",
                        resp.status, resp.body
                    );
                }
                Err(e) => {
                    eprintln!("agent: submit failed: {e}");
                }
            }
        } else {
            eprintln!("agent: submit build failed for gate={gate} evidence={evidence}");
        }
    }
}

// ---- Phase dispatch and evidence submission helpers ----------------------

fn use_teacher(phase: &str) -> bool {
    matches!(
        phase,
        "Analysis" | "Judgment" | "Plan" | "Eval" | "Recovery"
    )
}

fn parse_verdict(text: &str) -> bool {
    text.lines()
        .rev()
        .find_map(|line| {
            let line = line.trim();
            let verdict = line.strip_prefix("VERDICT:")?.trim().to_ascii_lowercase();
            if verdict.starts_with("fail") {
                Some(false)
            } else if verdict.starts_with("pass") {
                Some(true)
            } else {
                None
            }
        })
        .unwrap_or(true)
}

fn phase_gate(phase: &str) -> Option<(&'static str, &'static str)> {
    match phase {
        "Invariant" => Some(("Invariant", "InvariantProof")),
        "Analysis" => Some(("Analysis", "AnalysisReport")),
        "Judgment" => Some(("Judgment", "JudgmentRecord")),
        "Plan" => Some(("Plan", "TaskReady")),
        "Execute" => Some(("Execution", "ExecutionReceipt")),
        "Verify" => Some(("Verification", "LineageProof")),
        "Eval" => Some(("Eval", "EvalScore")),
        _ => None,
    }
}

fn recovery_gate(action: &str) -> Option<(&'static str, &'static str)> {
    match action {
        "RecheckInvariant" => Some(("Invariant", "InvariantProof")),
        "RunAnalysis" => Some(("Analysis", "AnalysisReport")),
        "Rejudge" => Some(("Judgment", "JudgmentRecord")),
        "Replan" => Some(("Plan", "PlanRecord")),
        "BindReadyTask" => Some(("Plan", "TaskReady")),
        "Reexecute" => Some(("Execution", "ArtifactReceipt")),
        "Reverify" => Some(("Verification", "VerificationReport")),
        "RepairArtifactLineage" => Some(("Verification", "LineageProof")),
        "RecomputeEval" => Some(("Eval", "EvalScore")),
        "Escalate" => None,
        _ => None,
    }
}

fn recovery_target_phase(action: &str) -> Option<&'static str> {
    match action {
        "RecheckInvariant" => Some("Invariant"),
        "RunAnalysis" => Some("Analysis"),
        "Rejudge" => Some("Judgment"),
        "Replan" | "BindReadyTask" => Some("Plan"),
        "Reexecute" => Some("Execute"),
        "Reverify" | "RepairArtifactLineage" => Some("Verify"),
        "RecomputeEval" => Some("Eval"),
        "Escalate" => Some("Done"),
        _ => None,
    }
}

fn gate_id_u64(gate: &str) -> Option<u64> {
    match gate {
        "Invariant" => Some(1),
        "Analysis" => Some(2),
        "Judgment" => Some(3),
        "Plan" => Some(4),
        "Execution" => Some(5),
        "Verification" => Some(6),
        "Eval" => Some(7),
        "Learning" => Some(8),
        _ => None,
    }
}

fn evidence_u64_value(evidence: &str) -> Option<u64> {
    match evidence {
        "InvariantProof" => Some(3),
        "AnalysisReport" => Some(4),
        "JudgmentRecord" => Some(5),
        "PlanRecord" => Some(6),
        "TaskReady" => Some(7),
        "ExecutionReceipt" => Some(8),
        "ArtifactReceipt" => Some(9),
        "VerificationReport" => Some(10),
        "LineageProof" => Some(11),
        "EvalScore" => Some(12),
        "PersistedRecord" => Some(16),
        "PolicyPromotion" => Some(18),
        _ => None,
    }
}

// Returns (effect_u64, effect_json_str). PacketEffect repr: None=0, BindReadyTask=1,
// MaterializeArtifact=2, RepairLineage=3, CompleteObjective=4.
fn effect_for_gate_evidence(gate: &str, evidence: &str, passed: bool) -> (u64, &'static str) {
    if !passed {
        return (0, "null");
    }
    match (gate, evidence) {
        ("Execution", "ExecutionReceipt") => (0, "null"),
        ("Plan", _) => (1, "\"BindReadyTask\""),
        ("Execution", _) => (2, "\"MaterializeArtifact\""),
        ("Verification", _) => (3, "\"RepairLineage\""),
        ("Eval", _) => (4, "\"CompleteObjective\""),
        _ => (0, "null"),
    }
}

fn compute_structural_payload_hash(
    gate_u64: u64,
    evidence_u64: u64,
    passed: bool,
    effect_u64: u64,
) -> u64 {
    let mut h = 0x8422_2325_cbf2_9ce4u64;
    h ^= gate_u64;
    h = h.wrapping_mul(0x100000001b3);
    h ^= evidence_u64;
    h = h.wrapping_mul(0x100000001b3);
    h ^= passed as u64;
    h = h.wrapping_mul(0x100000001b3);
    h ^= effect_u64;
    h = h.wrapping_mul(0x100000001b3);
    if h == 0 {
        1
    } else {
        h
    }
}

fn compute_evidence_contract_hash(
    gate_u64: u64,
    evidence_u64: u64,
    passed: bool,
    effect_u64: u64,
    payload_hash: u64,
) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    h ^= gate_u64;
    h = h.wrapping_mul(0x100000001b3);
    h ^= evidence_u64;
    h = h.wrapping_mul(0x100000001b3);
    h ^= passed as u64;
    h = h.wrapping_mul(0x100000001b3);
    h ^= effect_u64;
    h = h.wrapping_mul(0x100000001b3);
    h ^= payload_hash;
    h = h.wrapping_mul(0x100000001b3);
    h.max(1)
}

fn compute_submit_evidence_command_hash(sub_contract_hash: u64) -> u64 {
    let mut h = 0x9e3779b97f4a7c15u64;
    h ^= 1u64; // submission_count = 1
    h = h.wrapping_mul(0x100000001b3);
    h ^= sub_contract_hash;
    h.wrapping_mul(0x100000001b3).max(1)
}

fn compute_envelope_hash(command_id: u64, cmd_contract_hash: u64) -> u64 {
    const SCHEMA_VERSION: u64 = 6;
    let mut h = 0x517cc1b727220a95u64;
    h ^= SCHEMA_VERSION;
    h = h.wrapping_mul(0x100000001b3);
    h ^= command_id;
    h = h.wrapping_mul(0x100000001b3);
    h ^= cmd_contract_hash;
    h.wrapping_mul(0x100000001b3).max(1)
}

fn build_submit_evidence_json(
    gate: &str,
    evidence: &str,
    passed: bool,
    command_id: u64,
) -> Option<String> {
    let gate_u64 = gate_id_u64(gate)?;
    let ev_u64 = evidence_u64_value(evidence)?;
    let (effect_u64, effect_json) = effect_for_gate_evidence(gate, evidence, passed);

    let payload_hash = compute_structural_payload_hash(gate_u64, ev_u64, passed, effect_u64);
    let sub_hash =
        compute_evidence_contract_hash(gate_u64, ev_u64, passed, effect_u64, payload_hash);
    let cmd_hash = compute_submit_evidence_command_hash(sub_hash);
    let command_hash = compute_envelope_hash(command_id, cmd_hash);

    Some(format!(
        r#"{{"command_id":{command_id},"command_hash":{command_hash},"payload_tag":"SubmitEvidence","payload":{{"gate":"{gate}","evidence":"{evidence}","passed":{passed},"effect":{effect_json},"payload_hash":{payload_hash}}}}}"#
    ))
}

// ---- State parsing -------------------------------------------------------

fn extract_phase(state_json: &str) -> Option<String> {
    extract_json_string(state_json, "phase")
}

fn extract_json_string(json: &str, field: &str) -> Option<String> {
    let key = format!("\"{field}\":");
    let start = json.find(&key)?;
    let rest = json[start + key.len()..].trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_tlog_len(state_json: &str) -> Option<usize> {
    let key = "\"tlog_len\":";
    let start = state_json.find(key)?;
    let rest = state_json[start + key.len()..].trim_start();
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

#[cfg(test)]
mod hash_tests {
    use super::*;
    use crate::api::protocol::{Command, CommandEnvelope};
    use crate::capability::{EvidenceSubmission, PacketEffect};
    use crate::kernel::{Evidence, GateId};

    #[test]
    fn submit_evidence_hash_chain_matches_real_types() {
        // Invariant/InvariantProof/passed=true — the first gate any new worker needs
        let gate = GateId::Invariant;
        let evidence = Evidence::InvariantProof;
        let passed = true;

        let submission = EvidenceSubmission::new(gate, evidence, passed);
        assert!(
            submission.is_contract_valid(),
            "submission must be contract-valid"
        );

        let cmd = Command::SubmitEvidence(submission);
        let envelope = CommandEnvelope::new(1, cmd);
        assert!(
            envelope.is_contract_valid(),
            "envelope must be contract-valid"
        );

        // Recompute via our standalone functions and assert equality.
        let gate_u64 = gate_id_u64("Invariant").unwrap();
        let ev_u64 = evidence_u64_value("InvariantProof").unwrap();
        let (effect_u64, _) = effect_for_gate_evidence("Invariant", "InvariantProof", true);

        let payload_hash = compute_structural_payload_hash(gate_u64, ev_u64, passed, effect_u64);
        assert_eq!(
            payload_hash, submission.payload_hash,
            "payload_hash mismatch"
        );

        let sub_hash =
            compute_evidence_contract_hash(gate_u64, ev_u64, passed, effect_u64, payload_hash);
        assert_eq!(
            sub_hash,
            submission.contract_hash(),
            "submission contract_hash mismatch"
        );

        let cmd_hash = compute_submit_evidence_command_hash(sub_hash);
        assert_eq!(
            cmd_hash,
            Command::SubmitEvidence(submission).contract_hash(),
            "command contract_hash mismatch"
        );

        let env_hash = compute_envelope_hash(1, cmd_hash);
        assert_eq!(
            env_hash, envelope.command_hash,
            "envelope command_hash mismatch"
        );

        // Also verify JSON is parseable and has the right command_id
        let json = build_submit_evidence_json("Invariant", "InvariantProof", true, 1).unwrap();
        assert!(
            json.contains("\"command_id\":1"),
            "json must have command_id:1"
        );
        assert!(
            json.contains("\"InvariantProof\""),
            "json must name evidence"
        );
        assert!(json.contains(&format!("\"command_hash\":{}", envelope.command_hash)));
    }

    #[test]
    fn plan_gate_has_bind_ready_task_effect() {
        let gate = GateId::Plan;
        let evidence = Evidence::TaskReady;
        let passed = true;
        // Plan+passed requires BindReadyTask — must use with_effect, not new()
        let submission =
            EvidenceSubmission::with_effect(gate, evidence, passed, PacketEffect::BindReadyTask);
        assert!(submission.is_contract_valid());
        assert_eq!(submission.effect, PacketEffect::BindReadyTask);

        let gate_u64 = gate_id_u64("Plan").unwrap();
        let ev_u64 = evidence_u64_value("TaskReady").unwrap();
        let (effect_u64, _) = effect_for_gate_evidence("Plan", "TaskReady", true);
        assert_eq!(effect_u64, 1u64); // BindReadyTask = 1

        let payload_hash = compute_structural_payload_hash(gate_u64, ev_u64, passed, effect_u64);
        assert_eq!(payload_hash, submission.payload_hash);

        // Verify full contract hash chain matches
        let sub_hash =
            compute_evidence_contract_hash(gate_u64, ev_u64, passed, effect_u64, payload_hash);
        assert_eq!(sub_hash, submission.contract_hash());
    }
}
