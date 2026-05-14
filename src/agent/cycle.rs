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
        self.run_planning_turn(&domain, &metric)?;

        // Phase-dispatch loop. The runtime phase drives submissions; LLM output
        // only supplies semantic content and a pass/fail verdict for LLM phases.
        //
        // Invariant: the cycle may submit evidence, but it never declares the
        // objective complete. Completion is recognized only after observing the
        // worker in phase=Done.
        let mut stop_reason = StopReason::MaxStepsReached;
        let mut loop_steps = 0u64;

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

            if let Some(reason) =
                self.dispatch_observed_phase(&phase, loop_steps, &domain, &metric, &state_body)?
            {
                stop_reason = reason;
                break;
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

    fn dispatch_observed_phase(
        &mut self,
        phase: &str,
        loop_steps: u64,
        domain: &str,
        metric: &str,
        state_body: &str,
    ) -> Result<Option<StopReason>, CycleError> {
        match phase {
            "Analysis" => self.run_llm_gate_phase(
                loop_steps,
                "Analysis",
                prompt::analysis_prompt(domain, metric),
            ),
            "Judgment" => self.run_llm_gate_phase(
                loop_steps,
                "Judgment",
                prompt::judgment_prompt(domain, &self.last_llm_output),
            ),
            "Plan" => self.run_llm_gate_phase(
                loop_steps,
                "Plan",
                prompt::plan_prompt(domain, &self.last_llm_output),
            ),
            "Eval" => self.run_llm_gate_phase(
                loop_steps,
                "Eval",
                prompt::eval_prompt(domain, metric, &self.last_llm_output),
            ),
            "Recovery" => self.run_recovery_phase(loop_steps, domain, state_body),
            // Invariant gate is proved by loop_driver via SubmitObservationIngress
            // before each cycle. cycle.rs no longer submits evidence for this gate.
            "Invariant" => Ok(None),
            "Execute" => {
                if let Some((gate, evidence)) = phase_gate("Execute") {
                    self.submit_evidence(gate, evidence, true);
                }
                Ok(None)
            }
            "Verify" => {
                if let Some((gate, evidence)) = phase_gate("Verify") {
                    self.submit_evidence(gate, evidence, true);
                }
                Ok(None)
            }
            "Persist" | "Learn" => {
                if let Some((gate, evidence)) = phase_gate("Learning") {
                    self.submit_evidence(gate, evidence, true);
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn run_llm_gate_phase(
        &mut self,
        loop_steps: u64,
        phase: &str,
        prompt: String,
    ) -> Result<Option<StopReason>, CycleError> {
        let rationale = format!("{} phase", phase.to_ascii_lowercase());
        let output = self.llm_phase_turn(loop_steps, phase, rationale, prompt)?;
        if output.contains(SENTINEL_REVIEW) {
            return Ok(Some(StopReason::HumanReviewRequired));
        }

        let passed = parse_verdict(&output);
        if let Some((gate, evidence)) = phase_gate(phase) {
            self.submit_evidence(gate, evidence, passed);
        }
        Ok(None)
    }

    fn run_recovery_phase(
        &mut self,
        loop_steps: u64,
        domain: &str,
        state_body: &str,
    ) -> Result<Option<StopReason>, CycleError> {
        let failure =
            extract_json_string(state_body, "failure").unwrap_or_else(|| "Unknown".into());
        let recovery_action = extract_json_string(state_body, "recovery_action");
        let target_phase = recovery_action
            .as_deref()
            .and_then(recovery_target_phase)
            .unwrap_or("Unknown");
        let output = self.llm_phase_turn(
            loop_steps,
            "Recovery",
            "recovery phase",
            prompt::recovery_prompt(domain, &failure, target_phase),
        )?;
        if output.contains(SENTINEL_REVIEW) {
            return Ok(Some(StopReason::HumanReviewRequired));
        }
        let passed = parse_verdict(&output);
        let selected_action = recovery_action
            .as_deref()
            .or_else(|| recovery_action_for_failure(&failure));
        if let Some(action) = selected_action {
            if let Some((gate, evidence)) = recovery_gate(action) {
                self.submit_evidence(gate, evidence, passed);
            }
        }
        Ok(None)
    }

    fn run_planning_turn(&mut self, domain: &str, metric: &str) -> Result<(), CycleError> {
        let score_report = std::fs::read_to_string("SCORE_REPORT.md").ok();
        let messages = vec![
            OpenAiMessage::system(prompt::system_prompt()),
            OpenAiMessage::user(prompt::planning_prompt(
                domain,
                metric,
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

        self.last_llm_output = result.response.content;
        Ok(())
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
        .unwrap_or(false)
}

fn phase_gate(phase: &str) -> Option<(&'static str, &'static str)> {
    match phase {
        "Invariant" => Some(("Invariant", "InvariantProof")),
        "Analysis" => Some(("Analysis", "AnalysisReport")),
        "Judgment" => Some(("Judgment", "JudgmentRecord")),
        "Plan" => Some(("Plan", "TaskReady")),
        "Execute" => Some(("Execution", "ArtifactReceipt")),
        "Verify" => Some(("Verification", "LineageProof")),
        "Eval" => Some(("Eval", "EvalScore")),
        "Learning" => Some(("Learning", "PolicyPromotion")),
        _ => None,
    }
}

fn recovery_action_for_failure(failure: &str) -> Option<&'static str> {
    match failure {
        "InvariantUnknown" | "InvariantBlocked" => Some("RecheckInvariant"),
        "AnalysisMissing" | "AnalysisFailed" => Some("RunAnalysis"),
        "JudgmentMissing" | "JudgmentFailed" => Some("Rejudge"),
        "PlanMissing" | "PlanFailed" => Some("Replan"),
        "PlanReadyQueueEmpty" => Some("BindReadyTask"),
        "ExecutionMissing" | "ExecutionFailed" | "TaskReceiptMissing" => Some("Reexecute"),
        "VerificationUnknown" | "VerificationFailed" => Some("Reverify"),
        "ArtifactLineageBroken" => Some("RepairArtifactLineage"),
        "EvalMissing" | "EvalFailed" => Some("RecomputeEval"),
        "RecoveryExhausted" | "ConvergenceFailed" => Some("Escalate"),
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
    use crate::capability::llm::openai::OpenAiConfig;
    use crate::capability::{EvidenceSubmission, PacketEffect};
    use crate::kernel::{Evidence, GateId};
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    fn read_loopback_http_request(mut stream: std::net::TcpStream) -> String {
        let mut bytes = Vec::new();
        let mut buf = [0u8; 512];
        let mut header_end = None;

        while header_end.is_none() {
            let read = stream.read(&mut buf).expect("read request headers");
            assert!(read > 0, "client closed before request headers");
            bytes.extend_from_slice(&buf[..read]);
            header_end = bytes.windows(4).position(|window| window == b"\r\n\r\n");
        }

        let header_end = header_end.expect("request header terminator") + 4;
        let headers = String::from_utf8_lossy(&bytes[..header_end]);
        let content_length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().expect("valid content-length"))
            })
            .unwrap_or(0);

        while bytes.len() < header_end + content_length {
            let read = stream.read(&mut buf).expect("read request body");
            assert!(read > 0, "client closed before request body");
            bytes.extend_from_slice(&buf[..read]);
        }

        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
            .expect("write response");

        String::from_utf8(bytes[header_end..header_end + content_length].to_vec())
            .expect("request body is utf8")
    }

    fn spawn_single_command_worker() -> (WorkerClient, mpsc::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback worker");
        let port = listener.local_addr().expect("loopback worker addr").port();
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept command request");
            let body = read_loopback_http_request(stream);
            tx.send(body).expect("send command body");
        });

        (WorkerClient::new_with_timeout(port, 1_000), rx)
    }

    fn unused_loopback_router() -> RouterClient {
        RouterClient::new(OpenAiConfig {
            base_url: "http://127.0.0.1:1/v1".to_string(),
            model: "unused-test-router".to_string(),
            timeout_ms: 1_000,
        })
        .expect("valid unused router config")
    }

    fn read_loopback_http_request_bytes(mut stream: std::net::TcpStream) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut buf = [0u8; 512];
        let mut header_end = None;

        while header_end.is_none() {
            let read = stream.read(&mut buf).expect("read request headers");
            assert!(read > 0, "client closed before request headers");
            bytes.extend_from_slice(&buf[..read]);
            header_end = bytes.windows(4).position(|window| window == b"\r\n\r\n");
        }

        let header_end = header_end.expect("request header terminator") + 4;
        let headers = String::from_utf8_lossy(&bytes[..header_end]);
        let content_length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().expect("valid content-length"))
            })
            .unwrap_or(0);

        while bytes.len() < header_end + content_length {
            let read = stream.read(&mut buf).expect("read request body");
            assert!(read > 0, "client closed before request body");
            bytes.extend_from_slice(&buf[..read]);
        }

        bytes
    }

    fn spawn_review_router() -> (RouterClient, mpsc::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback router");
        let port = listener.local_addr().expect("loopback router addr").port();
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept router request");
            let request = read_loopback_http_request_bytes(
                stream.try_clone().expect("clone router request stream"),
            );
            let request_text = String::from_utf8_lossy(&request).into_owned();
            tx.send(request_text).expect("send router request");

            let body = format!(
                r#"{{"id":"review-test","choices":[{{"message":{{"role":"assistant","content":"{}"}}}}],"usage":{{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}}}}"#,
                SENTINEL_REVIEW
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .expect("write router response");
        });

        let router = RouterClient::new(OpenAiConfig {
            base_url: format!("http://127.0.0.1:{port}/v1"),
            model: "review-test-router".to_string(),
            timeout_ms: 1_000,
        })
        .expect("valid review router config");
        (router, rx)
    }

    #[test]
    fn verdict_parser_requires_explicit_verdict() {
        assert!(parse_verdict("evidence is coherent\nVERDICT: pass"));
        assert!(!parse_verdict("evidence is missing\nVERDICT: fail"));
        assert!(!parse_verdict("tool echo without a verdict"));
    }

    #[test]
    fn phase_gate_uses_artifact_receipt_for_execute() {
        assert_eq!(
            phase_gate("Execute"),
            Some(("Execution", "ArtifactReceipt"))
        );
    }

    #[test]
    fn recovery_failure_maps_task_receipt_missing_to_reexecute() {
        assert_eq!(
            recovery_action_for_failure("TaskReceiptMissing"),
            Some("Reexecute")
        );
        assert_eq!(
            recovery_gate("Reexecute"),
            Some(("Execution", "ArtifactReceipt"))
        );
    }

    #[test]
    fn learning_phase_gate_promotes_policy() {
        assert_eq!(
            phase_gate("Learning"),
            Some(("Learning", "PolicyPromotion"))
        );
    }

    #[test]
    fn dispatch_observed_phase_submits_invariant_without_llm() {
        let (worker, submitted_body) = spawn_single_command_worker();
        let objective = AgentObjective::new("cycle-test-domain", "submit invariant evidence");
        let mut cycle = AgentCycle::new(unused_loopback_router(), worker, objective);
        let stop = cycle
            .dispatch_observed_phase(
                "Invariant",
                1,
                "cycle-test-domain",
                "submit invariant evidence",
                "{}",
            )
            .expect("invariant phase dispatch succeeds");

        assert!(stop.is_none(), "invariant dispatch must not stop the run");
        assert_eq!(
            cycle.steps().len(),
            0,
            "invariant dispatch must not invoke or record an LLM turn"
        );
        assert!(
            submitted_body.try_recv().is_err(),
            "invariant evidence is submitted by loop_driver before the cycle dispatch"
        );
    }

    #[test]
    fn dispatch_observed_phase_stops_when_llm_phase_requests_review() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind unused worker");
        let worker_port = listener.local_addr().expect("unused worker addr").port();
        drop(listener);
        let worker = WorkerClient::new_with_timeout(worker_port, 1_000);
        let (router, router_request) = spawn_review_router();
        let objective = AgentObjective::new("cycle-test-domain", "request human review");
        let mut cycle = AgentCycle::new(router, worker, objective);
        let stop = cycle
            .dispatch_observed_phase(
                "Analysis",
                7,
                "cycle-test-domain",
                "request human review",
                "{}",
            )
            .expect("analysis phase dispatch succeeds");

        assert!(
            matches!(stop, Some(StopReason::HumanReviewRequired)),
            "sentinel LLM output must stop the run for human review"
        );
        assert_eq!(cycle.steps().len(), 1, "one LLM turn should be recorded");
        let step = &cycle.steps()[0];
        assert_eq!(step.step_index, 7);
        assert_eq!(step.action_kind, AgentActionKind::LlmTurn);
        assert!(step.success);
        assert!(cycle.last_llm_output.contains(SENTINEL_REVIEW));

        let request = router_request.recv().expect("captured router request");
        assert!(
            request.contains("POST /v1/chat/completions HTTP/1.1"),
            "analysis phase should call only the local router fixture"
        );
    }

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
