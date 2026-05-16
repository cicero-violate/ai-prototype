//! Agent step and decision records matching the loop contract.

/// The four receipted action kinds defined in loop_contract.md.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentActionKind {
    /// POST /v1/chat/completions → OpenAiLlmEffectReceipt
    LlmTurn,
    /// LiveMcpCallExecutor.execute_call → McpCallReceipt
    DirectToolCall,
    /// ObservationIngressBatch → evidence hash
    ObservationIngress,
    /// LiveSandboxProcessExecutor.execute_process → SandboxProcessReceipt
    ProcessExecution,
}

impl AgentActionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LlmTurn => "llm_turn",
            Self::DirectToolCall => "direct_tool_call",
            Self::ObservationIngress => "observation_ingress",
            Self::ProcessExecution => "process_execution",
        }
    }
}

#[derive(Clone, Debug)]
pub struct AgentDecision {
    pub selected_action: AgentActionKind,
    pub rationale: String,
    /// 0–100; higher is more confident.
    pub confidence_score: u8,
    /// 0–100; higher means more uncertainty.
    pub uncertainty_score: u8,
}

#[derive(Clone, Debug)]
pub struct AgentStep {
    pub step_index: u64,
    pub objective_id: u64,
    pub decision: AgentDecision,
    pub action_kind: AgentActionKind,
    /// Hash of the outgoing request body or payload.
    pub request_hash: u64,
    /// Hash carried in the capability receipt, if the step completed.
    pub receipt_hash: Option<u64>,
    pub success: bool,
}

#[derive(Clone, Debug)]
pub struct AgentRunSummary {
    pub objective_id: u64,
    pub step_count: u64,
    pub success: bool,
    pub stop_reason: String,
    pub final_phase: String,
    /// Number of events in the worker TLog at the end of the run.
    pub final_tlog_len: usize,
}
