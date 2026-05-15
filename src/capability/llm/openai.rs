//! OpenAI-compatible LLM adapter for the local browser router.
//!
//! This targets the router-server `/v1/chat/completions` surface. Tool calls are
//! represented as OpenAI-compatible chat history for prompt simulation; the
//! router still does not execute tools natively.

use std::env;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

use crate::capability::context::ContextRecord;
use crate::capability::llm::record::{
    retry_budget_decision_receiptable, retry_budget_decision_valid, retry_budget_exhausted,
    retry_budget_policy_valid, LlmRecord, LlmStructuredAdapter,
};
use crate::capability::llm::transport::{
    chat_completions_path as shared_chat_completions_path, parse_local_http_endpoint,
    provider_text_hash, request_identity_hash as shared_request_identity_hash,
    retry_policy_hash as shared_retry_policy_hash, LocalEndpointError, LocalLlmEndpoint,
};
use crate::capability::policy::PolicyStore;
use crate::capability::verification::{
    verify_verification_proof_record_bindings, CanonicalEffect, CanonicalEffectProof,
    CanonicalEffectReceipt, ProofSubjectKind, VerificationProofBinding, VerificationProofRecord,
    PROOF_FLAG_PHASE_VERIFIED, PROOF_FLAG_PROVENANCE_VERIFIED, PROOF_FLAG_RECEIPT_VERIFIED,
    PROOF_FLAG_TAMPER_REJECTED,
};
use crate::codec::ndjson::{load_tlog_ndjson, TLOG_RECORD_EVENT};
use crate::kernel::{
    mix, Cause, ControlEvent, Decision, EventKind, Evidence, GateId, GateStatus, Phase, TLog,
};

pub const OPENAI_COMPAT_PROVIDER: &str = "openai-compatible";
pub const OPENAI_LLM_EFFECT_RECEIPT_SCHEMA_VERSION: u64 = 1;
pub const OPENAI_LLM_EFFECT_RECEIPT_RECORD: u64 = 61;
pub const OPENAI_JUDGMENT_PROOF_SCHEMA_VERSION: u64 = 1;
pub const OPENAI_JUDGMENT_PROOF_RECORD: u64 = 62;
pub const OPENAI_JUDGMENT_PROOF_LINE: &str =
    "receipt_verified+tamper_rejected+endpoint_verified+phase_plan";

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8082/v1";
const DEFAULT_MODEL: &str = "chatgpt-project";
const DEFAULT_TIMEOUT_MS: u64 = 120_000;
const DEFAULT_MAX_RETRIES: u32 = 0;
const DEFAULT_ATTEMPT_BUDGET: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenAiConfig {
    pub base_url: String,
    pub model: String,
    pub timeout_ms: u64,
}

impl Default for OpenAiConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            model: DEFAULT_MODEL.to_string(),
            timeout_ms: DEFAULT_TIMEOUT_MS,
        }
    }
}

impl OpenAiConfig {
    pub fn from_env() -> Result<Self, OpenAiError> {
        let timeout_ms = env::var("CANON_OPENAI_TIMEOUT_MS")
            .ok()
            .and_then(|raw| raw.parse::<u64>().ok())
            .unwrap_or(DEFAULT_TIMEOUT_MS);
        let cfg = Self {
            base_url: env::var("CANON_OPENAI_BASE_URL")
                .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string()),
            model: env::var("CANON_OPENAI_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string()),
            timeout_ms,
        };
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<(), OpenAiError> {
        if self.model.trim().is_empty() {
            return Err(OpenAiError::InvalidConfig("empty model"));
        }
        if self.timeout_ms == 0 {
            return Err(OpenAiError::InvalidConfig("zero timeout"));
        }
        parse_local_endpoint(&self.base_url)?;
        Ok(())
    }

    pub fn chat_completions_path(&self) -> Result<String, OpenAiError> {
        let endpoint = parse_local_endpoint(&self.base_url)?;
        Ok(shared_chat_completions_path(&endpoint.path_prefix))
    }

    pub fn model_id(&self) -> u64 {
        openai_config_text_id(&self.model)
    }

    pub fn base_url_id(&self) -> u64 {
        openai_config_text_id(&self.base_url)
    }
}

fn openai_config_text_id(text: &str) -> u64 {
    provider_text_hash(text)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenAiRetryBudgetPolicy {
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub attempt_budget: u32,
}

impl OpenAiRetryBudgetPolicy {
    pub fn new(timeout_ms: u64, max_retries: u32, attempt_budget: u32) -> Option<Self> {
        if !retry_budget_policy_valid(timeout_ms, max_retries, attempt_budget) {
            return None;
        }
        Some(Self {
            timeout_ms,
            max_retries,
            attempt_budget,
        })
    }

    pub fn from_config(config: &OpenAiConfig) -> Self {
        Self {
            timeout_ms: config.timeout_ms,
            max_retries: DEFAULT_MAX_RETRIES,
            attempt_budget: DEFAULT_ATTEMPT_BUDGET,
        }
    }

    pub fn policy_hash(self) -> u64 {
        shared_retry_policy_hash(
            0x4f50_454e_4149_4255u64,
            self.timeout_ms,
            self.max_retries,
            self.attempt_budget,
        )
    }

    pub fn request_identity_hash(
        self,
        provider_hash: u64,
        base_url_hash: u64,
        model_id: u64,
        request_hash: u64,
    ) -> u64 {
        shared_request_identity_hash(
            0x4f50_454e_4149_4944u64,
            provider_hash,
            base_url_hash,
            model_id,
            request_hash,
        )
    }

    pub fn decision_for_attempt(
        self,
        provider_hash: u64,
        base_url_hash: u64,
        model_id: u64,
        request_hash: u64,
        retry_count: u32,
        duplicate_request: bool,
    ) -> Option<OpenAiRetryBudgetDecision> {
        let request_identity_hash =
            self.request_identity_hash(provider_hash, base_url_hash, model_id, request_hash);
        let retry_allowed = retry_count <= self.max_retries;
        let budget_allowed = retry_count < self.attempt_budget;
        let allowed =
            request_identity_hash != 0 && retry_allowed && budget_allowed && !duplicate_request;
        let budget_exhausted = !budget_allowed
            || retry_budget_exhausted(retry_count, self.max_retries, self.attempt_budget);
        let decision = OpenAiRetryBudgetDecision {
            timeout_ms: self.timeout_ms,
            retry_count,
            max_retries: self.max_retries,
            attempt_budget: self.attempt_budget,
            request_identity_hash,
            retry_budget_hash: self.policy_hash(),
            budget_exhausted,
            duplicate_request,
            allowed,
        };
        decision.is_valid_decision().then_some(decision)
    }

    pub fn first_attempt(
        self,
        provider_hash: u64,
        base_url_hash: u64,
        model_id: u64,
        request_hash: u64,
    ) -> Option<OpenAiRetryBudgetDecision> {
        self.decision_for_attempt(
            provider_hash,
            base_url_hash,
            model_id,
            request_hash,
            0,
            false,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenAiRetryBudgetDecision {
    pub timeout_ms: u64,
    pub retry_count: u32,
    pub max_retries: u32,
    pub attempt_budget: u32,
    pub request_identity_hash: u64,
    pub retry_budget_hash: u64,
    pub budget_exhausted: bool,
    pub duplicate_request: bool,
    pub allowed: bool,
}

impl OpenAiRetryBudgetDecision {
    pub fn is_valid_decision(self) -> bool {
        retry_budget_decision_valid(
            self.timeout_ms,
            self.retry_count,
            self.max_retries,
            self.attempt_budget,
            self.request_identity_hash,
            self.retry_budget_hash,
            self.duplicate_request,
            self.allowed,
        )
    }

    pub fn is_receiptable_success(self) -> bool {
        retry_budget_decision_receiptable(
            self.timeout_ms,
            self.retry_count,
            self.max_retries,
            self.attempt_budget,
            self.request_identity_hash,
            self.retry_budget_hash,
            self.duplicate_request,
            self.allowed,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenAiRetryBudgetLedger {
    policy: OpenAiRetryBudgetPolicy,
    seen_request_identity_hashes: Vec<u64>,
    attempts_used: u32,
}

impl OpenAiRetryBudgetLedger {
    pub fn new(policy: OpenAiRetryBudgetPolicy) -> Self {
        Self {
            policy,
            seen_request_identity_hashes: Vec::new(),
            attempts_used: 0,
        }
    }

    pub fn record_request(
        &mut self,
        provider_hash: u64,
        base_url_hash: u64,
        model_id: u64,
        request_hash: u64,
    ) -> Result<OpenAiRetryBudgetDecision, OpenAiError> {
        let request_identity_hash =
            self.policy
                .request_identity_hash(provider_hash, base_url_hash, model_id, request_hash);
        let duplicate_request = self
            .seen_request_identity_hashes
            .contains(&request_identity_hash);
        let decision = self
            .policy
            .decision_for_attempt(
                provider_hash,
                base_url_hash,
                model_id,
                request_hash,
                self.attempts_used,
                duplicate_request,
            )
            .ok_or(OpenAiError::InvalidConfig("invalid retry budget decision"))?;
        if duplicate_request {
            return Err(OpenAiError::DuplicateRequest);
        }
        if !decision.allowed {
            return Err(OpenAiError::BudgetExhausted);
        }
        self.seen_request_identity_hashes
            .push(request_identity_hash);
        self.attempts_used = self.attempts_used.saturating_add(1);
        Ok(decision)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenAiMessage {
    pub role: String,
    pub content: Option<String>,
    pub tool_call_id: Option<String>,
    pub tool_calls: Vec<OpenAiToolCall>,
}

impl OpenAiMessage {
    pub fn system(content: impl Into<String>) -> Self {
        openai_message("system", Some(content.into()), None, Vec::new())
    }

    pub fn user(content: impl Into<String>) -> Self {
        openai_message("user", Some(content.into()), None, Vec::new())
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        openai_message("assistant", Some(content.into()), None, Vec::new())
    }

    pub fn assistant_tool_call(tool_call: OpenAiToolCall) -> Self {
        openai_message("assistant", None, None, vec![tool_call])
    }

    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        openai_message(
            "tool",
            Some(content.into()),
            Some(tool_call_id.into()),
            Vec::new(),
        )
    }
}

fn openai_message(
    role: &str,
    content: Option<String>,
    tool_call_id: Option<String>,
    tool_calls: Vec<OpenAiToolCall>,
) -> OpenAiMessage {
    OpenAiMessage {
        role: role.to_string(),
        content,
        tool_call_id,
        tool_calls,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenAiFunctionTool {
    pub name: String,
    pub description: Option<String>,
    pub parameters_json: String,
}

impl OpenAiFunctionTool {
    pub fn new(name: impl Into<String>, parameters_json: impl Into<String>) -> Self {
        openai_function_tool(name.into(), parameters_json.into(), None)
    }

    pub fn with_description(self, description: impl Into<String>) -> Self {
        openai_function_tool(self.name, self.parameters_json, Some(description.into()))
    }
}

fn openai_function_tool(
    name: String,
    parameters_json: String,
    description: Option<String>,
) -> OpenAiFunctionTool {
    OpenAiFunctionTool {
        name,
        description,
        parameters_json,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenAiTool {
    pub function: OpenAiFunctionTool,
}

impl OpenAiTool {
    pub fn function(name: impl Into<String>, parameters_json: impl Into<String>) -> Self {
        Self {
            function: OpenAiFunctionTool::new(name, parameters_json),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenAiFunctionCall {
    pub name: String,
    pub arguments: String,
}

impl OpenAiFunctionCall {
    pub fn new(name: impl Into<String>, arguments: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            arguments: arguments.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenAiToolCall {
    pub id: String,
    pub function: OpenAiFunctionCall,
}

impl OpenAiToolCall {
    pub fn function(
        id: impl Into<String>,
        name: impl Into<String>,
        arguments: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            function: OpenAiFunctionCall::new(name, arguments),
        }
    }
}

/// Router-server browser routing options.
///
/// The router-server accepts an optional `browser` object alongside standard
/// OpenAI fields. `new_chat` opens a fresh provider tab. `target_url` pins the
/// turn to a specific existing tab URL (returned from a previous turn's response).
/// Both fields are ignored by non-router OpenAI-compatible endpoints.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OpenAiBrowserOptions {
    /// Open a new chat tab for this turn. Mutually exclusive with `target_url`.
    pub new_chat: bool,
    /// Reuse the tab at this URL. Obtained from a previous `OpenAiChatResponse::target_url`.
    pub target_url: Option<String>,
}

impl OpenAiBrowserOptions {
    pub fn new_chat() -> Self {
        Self {
            new_chat: true,
            target_url: None,
        }
    }

    pub fn continue_at(target_url: impl Into<String>) -> Self {
        Self {
            new_chat: false,
            target_url: Some(target_url.into()),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OpenAiChatRequest {
    pub messages: Vec<OpenAiMessage>,
    pub tools: Vec<OpenAiTool>,
    /// Router-server browser routing. `None` uses the router's default tab selection.
    pub browser: Option<OpenAiBrowserOptions>,
}

impl OpenAiChatRequest {
    pub fn new(messages: Vec<OpenAiMessage>) -> Self {
        Self {
            messages,
            tools: Vec::new(),
            browser: None,
        }
    }

    pub fn with_tools(mut self, tools: Vec<OpenAiTool>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_browser(mut self, browser: OpenAiBrowserOptions) -> Self {
        self.browser = Some(browser);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenAiChatResponse {
    pub id: String,
    pub content: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub response_hash: u64,
    pub raw_hash: u64,
    /// The ChatGPT tab URL the router used for this turn.
    /// Pass to the next turn via `OpenAiBrowserOptions::continue_at` to maintain conversation continuity.
    pub target_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenAiLlmCall {
    pub record: LlmRecord,
    pub provider_hash: u64,
    pub base_url_hash: u64,
    pub model_id: u64,
    pub request_hash: u64,
    pub timeout_ms: u64,
    pub retry_count: u32,
    pub max_retries: u32,
    pub attempt_budget: u32,
    pub request_identity_hash: u64,
    pub retry_budget_hash: u64,
    pub budget_exhausted: bool,
    pub duplicate_request: bool,
    pub response_hash: u64,
    pub raw_response_hash: u64,
    pub prompt_hash: u64,
    pub token_count: u32,
    pub payload_hash: u64,
}

impl OpenAiLlmCall {
    fn new(
        record: LlmRecord,
        model_id: u64,
        base_url_hash: u64,
        request_hash: u64,
        response: OpenAiChatResponse,
        retry_budget: OpenAiRetryBudgetDecision,
    ) -> Self {
        let submission = record.submission();
        Self {
            prompt_hash: record.prompt.prompt_hash,
            response_hash: record.response.response_hash,
            token_count: record.response.token_count,
            payload_hash: submission.payload_hash,
            record,
            provider_hash: hash_text(OPENAI_COMPAT_PROVIDER),
            base_url_hash,
            model_id,
            request_hash,
            timeout_ms: retry_budget.timeout_ms,
            retry_count: retry_budget.retry_count,
            max_retries: retry_budget.max_retries,
            attempt_budget: retry_budget.attempt_budget,
            request_identity_hash: retry_budget.request_identity_hash,
            retry_budget_hash: retry_budget.retry_budget_hash,
            budget_exhausted: retry_budget.budget_exhausted,
            duplicate_request: retry_budget.duplicate_request,
            raw_response_hash: response.raw_hash,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.record.is_valid()
            && self.provider_hash == hash_text(OPENAI_COMPAT_PROVIDER)
            && self.base_url_hash != 0
            && self.model_id != 0
            && self.request_hash != 0
            && self.timeout_ms != 0
            && self.retry_count <= self.max_retries
            && self.attempt_budget != 0
            && self.attempt_budget <= self.max_retries.saturating_add(1)
            && self.retry_count < self.attempt_budget
            && self.request_identity_hash != 0
            && self.retry_budget_hash != 0
            && !self.duplicate_request
            && self.response_hash != 0
            && self.raw_response_hash != 0
            && self.prompt_hash != 0
            && self.token_count != 0
            && self.payload_hash == self.record.submission().payload_hash
    }

    pub fn submission(&self) -> crate::capability::EvidenceSubmission {
        self.record.submission()
    }

    pub fn base_url_provenance_verified(&self, config: &OpenAiConfig) -> bool {
        config.validate().is_ok()
            && self.is_valid()
            && self.provider_hash == hash_text(OPENAI_COMPAT_PROVIDER)
            && self.base_url_hash == config.base_url_id()
            && self.model_id == config.model_id()
            && self.timeout_ms == config.timeout_ms
    }

    pub fn receipt_for_configured_event(
        &self,
        config: &OpenAiConfig,
        command_hash: u64,
        event: &ControlEvent,
    ) -> Option<OpenAiLlmEffectReceipt> {
        self.base_url_provenance_verified(config)
            .then(|| self.receipt_for_event_unchecked(command_hash, event))
            .flatten()
    }

    fn receipt_for_event_unchecked(
        &self,
        command_hash: u64,
        event: &ControlEvent,
    ) -> Option<OpenAiLlmEffectReceipt> {
        if !self.is_valid() || command_hash == 0 {
            return None;
        }
        let mut receipt = OpenAiLlmEffectReceipt {
            provider_hash: self.provider_hash,
            base_url_hash: self.base_url_hash,
            model_id: self.model_id,
            request_hash: self.request_hash,
            timeout_ms: self.timeout_ms,
            retry_count: self.retry_count,
            max_retries: self.max_retries,
            attempt_budget: self.attempt_budget,
            request_identity_hash: self.request_identity_hash,
            retry_budget_hash: self.retry_budget_hash,
            budget_exhausted: self.budget_exhausted,
            duplicate_request: self.duplicate_request,
            response_hash: self.response_hash,
            raw_response_hash: self.raw_response_hash,
            prompt_hash: self.prompt_hash,
            token_count: self.token_count,
            payload_hash: self.payload_hash,
            command_hash,
            event_seq: event.seq,
            event_hash: event.self_hash,
            proof_event_seq: 0,
            proof_hash: 0,
            receipt_hash: 0,
        };
        receipt.receipt_hash = receipt.expected_receipt_hash();
        receipt.matches_event(event).then_some(receipt)
    }

    pub fn receipt_from_tlog(
        &self,
        config: &OpenAiConfig,
        command_hash: u64,
        tlog: &TLog,
    ) -> Option<OpenAiLlmEffectReceipt> {
        tlog.iter()
            .find_map(|event| self.receipt_for_configured_event(config, command_hash, event))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenAiLlmEffectReceipt {
    pub provider_hash: u64,
    pub base_url_hash: u64,
    pub model_id: u64,
    pub request_hash: u64,
    pub timeout_ms: u64,
    pub retry_count: u32,
    pub max_retries: u32,
    pub attempt_budget: u32,
    pub request_identity_hash: u64,
    pub retry_budget_hash: u64,
    pub budget_exhausted: bool,
    pub duplicate_request: bool,
    pub response_hash: u64,
    pub raw_response_hash: u64,
    pub prompt_hash: u64,
    pub token_count: u32,
    pub payload_hash: u64,
    pub command_hash: u64,
    pub event_seq: u64,
    pub event_hash: u64,
    pub proof_event_seq: u64,
    pub proof_hash: u64,
    pub receipt_hash: u64,
}

impl OpenAiLlmEffectReceipt {
    pub fn is_valid(self) -> bool {
        self.provider_hash == hash_text(OPENAI_COMPAT_PROVIDER)
            && self.base_url_hash != 0
            && self.model_id != 0
            && self.request_hash != 0
            && self.timeout_ms != 0
            && self.retry_count <= self.max_retries
            && self.attempt_budget != 0
            && self.attempt_budget <= self.max_retries.saturating_add(1)
            && self.retry_count < self.attempt_budget
            && self.request_identity_hash != 0
            && self.retry_budget_hash != 0
            && retry_budget_binding_is_valid(
                self.provider_hash,
                self.base_url_hash,
                self.model_id,
                self.request_hash,
                self.timeout_ms,
                self.retry_count,
                self.max_retries,
                self.attempt_budget,
                self.request_identity_hash,
                self.retry_budget_hash,
            )
            && !self.duplicate_request
            && self.response_hash != 0
            && self.raw_response_hash != 0
            && self.prompt_hash != 0
            && self.token_count != 0
            && self.payload_hash != 0
            && self.command_hash != 0
            && self.event_seq != 0
            && self.event_hash != 0
            && ((self.proof_event_seq == 0 && self.proof_hash == 0)
                || (self.proof_event_seq > self.event_seq && self.proof_hash != 0))
            && self.receipt_hash != 0
            && self.receipt_hash == self.expected_receipt_hash()
    }

    pub fn has_proof_binding(self) -> bool {
        self.is_valid() && self.proof_event_seq != 0 && self.proof_hash != 0
    }

    pub fn expected_receipt_core_hash(self) -> u64 {
        let mut h = 0x4f50_454e_4149_5243u64;
        h = mix(h, self.provider_hash);
        h = mix(h, self.base_url_hash);
        h = mix(h, self.model_id);
        h = mix(h, self.request_hash);
        h = mix(h, self.timeout_ms);
        h = mix(h, self.retry_count as u64);
        h = mix(h, self.max_retries as u64);
        h = mix(h, self.attempt_budget as u64);
        h = mix(h, self.request_identity_hash);
        h = mix(h, self.retry_budget_hash);
        h = mix(h, self.budget_exhausted as u64);
        h = mix(h, self.duplicate_request as u64);
        h = mix(h, self.response_hash);
        h = mix(h, self.raw_response_hash);
        h = mix(h, self.prompt_hash);
        h = mix(h, self.token_count as u64);
        h = mix(h, self.payload_hash);
        h = mix(h, self.command_hash);
        h = mix(h, self.event_seq);
        h = mix(h, self.event_hash);
        h.max(1)
    }

    pub fn expected_receipt_hash(self) -> u64 {
        let mut h = self.expected_receipt_core_hash();
        h = mix(h, self.proof_event_seq);
        h = mix(h, self.proof_hash);
        h.max(1)
    }

    pub fn bind_proof_event(self, proof_event_seq: u64, proof_hash: u64) -> Option<Self> {
        if !self.is_valid()
            || proof_event_seq == 0
            || proof_hash == 0
            || proof_event_seq <= self.event_seq
        {
            return None;
        }
        let mut receipt = Self {
            proof_event_seq,
            proof_hash,
            receipt_hash: 0,
            ..self
        };
        receipt.receipt_hash = receipt.expected_receipt_hash();
        receipt.is_valid().then_some(receipt)
    }

    pub fn matches_event(self, event: &ControlEvent) -> bool {
        self.is_valid()
            && event.seq == self.event_seq
            && event.self_hash == self.event_hash
            && event.api_command_hash == self.command_hash
            && event.from == Phase::Judgment
            && event.to == Phase::Judgment
            && event.kind == EventKind::Persisted
            && event.cause == Cause::EvidenceSubmitted
            && event.evidence == Evidence::JudgmentRecord
            && event.decision == Decision::Continue
            && event.failure.is_none()
            && event.recovery_action.is_none()
            && event.affected_gate == Some(GateId::Judgment)
            && event.state_after.gates.judgment.status == GateStatus::Pass
            && event.state_after.gates.judgment.evidence == Evidence::JudgmentRecord
    }

    pub fn replay_verified(self, tlog: &TLog) -> bool {
        self.is_valid() && tlog.iter().any(|event| self.matches_event(event))
    }

    pub fn base_url_provenance_verified(self, config: &OpenAiConfig) -> bool {
        config.validate().is_ok()
            && self.is_valid()
            && self.provider_hash == hash_text(OPENAI_COMPAT_PROVIDER)
            && self.base_url_hash == config.base_url_id()
            && self.model_id == config.model_id()
            && self.timeout_ms == config.timeout_ms
            && !self.duplicate_request
    }

    pub fn canonical_effect(self) -> Option<CanonicalEffect> {
        self.is_valid().then_some(())?;
        CanonicalEffect::llm(
            self.response_hash,
            self.raw_response_hash,
            self.prompt_hash,
            self.token_count as u64,
            self.model_id,
        )
    }

    pub fn canonical_effect_receipt(self) -> Option<CanonicalEffectReceipt> {
        self.is_valid().then_some(())?;
        CanonicalEffectReceipt::new_unbound(
            ProofSubjectKind::LlmEffect,
            self.canonical_effect()?,
            self.canonical_authority_hash()?,
            self.canonical_request_hash()?,
            self.event_seq,
            self.event_hash,
        )
    }

    pub fn canonical_authority_hash(self) -> Option<u64> {
        self.is_valid().then_some(())?;
        Some(Self::fold_ordered_effect_receipt_hash(
            0x4f50_454e_4149_4155u64,
            &[
                self.provider_hash,
                self.base_url_hash,
                self.model_id,
                self.timeout_ms,
                self.max_retries as u64,
                self.attempt_budget as u64,
                self.retry_budget_hash,
            ],
        ))
    }

    pub fn canonical_request_hash(self) -> Option<u64> {
        self.is_valid().then_some(())?;
        Some(Self::fold_ordered_effect_receipt_hash(
            0x4f50_454e_4149_5251u64,
            &[
                self.request_hash,
                self.command_hash,
                self.request_identity_hash,
            ],
        ))
    }

    fn fold_ordered_effect_receipt_hash(seed: u64, fields: &[u64]) -> u64 {
        let mut h = seed;
        for field in fields {
            h = mix(h, *field);
        }
        h.max(1)
    }

    pub fn verification_proof_binding(self) -> Option<VerificationProofBinding> {
        if !self.has_proof_binding() {
            return None;
        }
        VerificationProofBinding::new(
            ProofSubjectKind::LlmEffect,
            self.expected_receipt_core_hash(),
            self.receipt_hash,
            self.event_seq,
            self.event_hash,
            self.proof_hash,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenAiJudgmentProofEvent {
    pub proof_line_hash: u64,
    pub receipt_core_hash: u64,
    pub receipt_hash: u64,
    pub receipt_event_seq: u64,
    pub proof_event_seq: u64,
    pub receipt_event_hash: u64,
    pub base_url_hash: u64,
    pub model_id: u64,
    pub timeout_ms: u64,
    pub retry_count: u32,
    pub max_retries: u32,
    pub attempt_budget: u32,
    pub request_identity_hash: u64,
    pub retry_budget_hash: u64,
    pub budget_exhausted: bool,
    pub duplicate_request: bool,
    pub receipt_verified: bool,
    pub tamper_rejected: bool,
    pub endpoint_verified: bool,
    pub phase_plan: bool,
    pub proof_hash: u64,
}

impl OpenAiJudgmentProofEvent {
    pub fn finalize_receipt_after_tlog(
        receipt: OpenAiLlmEffectReceipt,
        tlog: &TLog,
        receipt_verified: bool,
        tamper_rejected: bool,
        endpoint_verified: bool,
        phase_plan: bool,
    ) -> Option<(OpenAiLlmEffectReceipt, Self)> {
        if !receipt.replay_verified(tlog) {
            return None;
        }
        let proof_event_seq = tlog
            .last()
            .map(|event| event.seq.saturating_add(1))
            .unwrap_or(receipt.event_seq.saturating_add(1));
        Self::finalize_receipt_at_seq(
            receipt,
            proof_event_seq,
            receipt_verified,
            tamper_rejected,
            endpoint_verified,
            phase_plan,
        )
    }

    fn finalize_receipt_at_seq(
        receipt: OpenAiLlmEffectReceipt,
        proof_event_seq: u64,
        receipt_verified: bool,
        tamper_rejected: bool,
        endpoint_verified: bool,
        phase_plan: bool,
    ) -> Option<(OpenAiLlmEffectReceipt, Self)> {
        if !receipt.is_valid()
            || !receipt_verified
            || !tamper_rejected
            || !endpoint_verified
            || !phase_plan
            || proof_event_seq <= receipt.event_seq
        {
            return None;
        }
        let mut event = Self {
            proof_line_hash: hash_text(OPENAI_JUDGMENT_PROOF_LINE),
            receipt_core_hash: receipt.expected_receipt_core_hash(),
            receipt_hash: 0,
            receipt_event_seq: receipt.event_seq,
            proof_event_seq,
            receipt_event_hash: receipt.event_hash,
            base_url_hash: receipt.base_url_hash,
            model_id: receipt.model_id,
            timeout_ms: receipt.timeout_ms,
            retry_count: receipt.retry_count,
            max_retries: receipt.max_retries,
            attempt_budget: receipt.attempt_budget,
            request_identity_hash: receipt.request_identity_hash,
            retry_budget_hash: receipt.retry_budget_hash,
            budget_exhausted: receipt.budget_exhausted,
            duplicate_request: receipt.duplicate_request,
            receipt_verified,
            tamper_rejected,
            endpoint_verified,
            phase_plan,
            proof_hash: 0,
        };
        event.proof_hash = event.expected_proof_hash();
        let finalized_receipt =
            receipt.bind_proof_event(event.proof_event_seq, event.proof_hash)?;
        event.receipt_hash = finalized_receipt.receipt_hash;
        event.is_valid().then_some((finalized_receipt, event))
    }

    pub fn is_valid(self) -> bool {
        self.proof_line_hash == hash_text(OPENAI_JUDGMENT_PROOF_LINE)
            && self.receipt_core_hash != 0
            && self.receipt_hash != 0
            && self.receipt_event_seq != 0
            && self.proof_event_seq > self.receipt_event_seq
            && self.receipt_event_hash != 0
            && self.base_url_hash != 0
            && self.model_id != 0
            && self.timeout_ms != 0
            && self.retry_count <= self.max_retries
            && self.attempt_budget != 0
            && self.request_identity_hash != 0
            && self.retry_budget_hash != 0
            && self.receipt_verified
            && self.tamper_rejected
            && self.endpoint_verified
            && self.phase_plan
            && self.proof_hash == self.expected_proof_hash()
    }

    pub fn expected_proof_hash(self) -> u64 {
        Self::fold_ordered_openai_proof_event_hash(
            0x4f50_454e_4149_5052u64,
            &[
                self.proof_line_hash,
                self.receipt_core_hash,
                self.receipt_event_seq,
                self.proof_event_seq,
                self.receipt_event_hash,
                self.base_url_hash,
                self.model_id,
                self.timeout_ms,
                self.retry_count as u64,
                self.max_retries as u64,
                self.attempt_budget as u64,
                self.request_identity_hash,
                self.retry_budget_hash,
                self.budget_exhausted as u64,
                self.duplicate_request as u64,
                self.receipt_verified as u64,
                self.tamper_rejected as u64,
                self.endpoint_verified as u64,
                self.phase_plan as u64,
            ],
        )
    }

    fn fold_ordered_openai_proof_event_hash(seed: u64, fields: &[u64]) -> u64 {
        fields
            .iter()
            .fold(seed, |hash, field| mix(hash, *field))
            .max(1)
    }

    pub fn proof_flags(self) -> u64 {
        let mut flags = 0;
        if self.receipt_verified {
            flags |= PROOF_FLAG_RECEIPT_VERIFIED;
        }
        if self.tamper_rejected {
            flags |= PROOF_FLAG_TAMPER_REJECTED;
        }
        if self.endpoint_verified {
            flags |= PROOF_FLAG_PROVENANCE_VERIFIED;
        }
        if self.phase_plan {
            flags |= PROOF_FLAG_PHASE_VERIFIED;
        }
        flags
    }

    pub fn verifier_context_hash(self) -> u64 {
        Self::fold_ordered_openai_proof_event_hash(
            0x4f50_454e_4149_4354u64,
            &[
                self.base_url_hash,
                self.model_id,
                self.timeout_ms,
                self.retry_count as u64,
                self.max_retries as u64,
                self.attempt_budget as u64,
                self.request_identity_hash,
                self.retry_budget_hash,
                self.budget_exhausted as u64,
                self.duplicate_request as u64,
            ],
        )
    }

    pub fn matches_receipt(self, receipt: OpenAiLlmEffectReceipt, tlog: &TLog) -> bool {
        self.is_valid()
            && receipt.replay_verified(tlog)
            && receipt.has_proof_binding()
            && self.proof_hash == receipt.proof_hash
            && self.receipt_hash == receipt.receipt_hash
            && self.receipt_core_hash == receipt.expected_receipt_core_hash()
            && self.receipt_event_seq == receipt.event_seq
            && self.proof_event_seq == receipt.proof_event_seq
            && self.receipt_event_hash == receipt.event_hash
            && self.base_url_hash == receipt.base_url_hash
            && self.model_id == receipt.model_id
            && self.timeout_ms == receipt.timeout_ms
            && self.retry_count == receipt.retry_count
            && self.max_retries == receipt.max_retries
            && self.attempt_budget == receipt.attempt_budget
            && self.request_identity_hash == receipt.request_identity_hash
            && self.retry_budget_hash == receipt.retry_budget_hash
            && self.budget_exhausted == receipt.budget_exhausted
            && self.duplicate_request == receipt.duplicate_request
    }

    pub fn to_canonical_effect_proof(
        self,
        receipt: OpenAiLlmEffectReceipt,
    ) -> Option<(CanonicalEffectReceipt, CanonicalEffectProof)> {
        if !self.is_valid()
            || !receipt.has_proof_binding()
            || self.proof_hash != receipt.proof_hash
            || self.receipt_hash != receipt.receipt_hash
            || self.receipt_event_seq != receipt.event_seq
            || self.proof_event_seq != receipt.proof_event_seq
            || self.receipt_event_hash != receipt.event_hash
        {
            return None;
        }
        let canonical_receipt = receipt.canonical_effect_receipt()?;
        CanonicalEffectProof::finalize(
            canonical_receipt,
            self.proof_event_seq,
            self.verifier_context_hash(),
            self.proof_flags(),
            self.proof_hash,
        )
    }

    pub fn to_canonical_verification_proof_record(
        self,
        receipt: OpenAiLlmEffectReceipt,
    ) -> Option<(CanonicalEffectReceipt, VerificationProofRecord)> {
        let (canonical_receipt, proof) = self.to_canonical_effect_proof(receipt)?;
        Some((canonical_receipt, proof.to_verification_proof_record()?))
    }
}

#[derive(Debug)]
pub enum OpenAiError {
    InvalidConfig(&'static str),
    InvalidUrl,
    HttpStatus(u16),
    Io(std::io::Error),
    InvalidResponse,
    InvalidReceipt,
    InvalidReceiptRecord,
    InvalidReplay,
    BudgetExhausted,
    DuplicateRequest,
}

impl fmt::Display for OpenAiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(reason) => write!(f, "invalid openai-compatible config: {reason}"),
            Self::InvalidUrl => write!(f, "invalid local openai-compatible url"),
            Self::HttpStatus(status) => {
                write!(f, "openai-compatible endpoint returned HTTP {status}")
            }
            Self::Io(err) => write!(f, "openai-compatible io failed: {err}"),
            Self::InvalidResponse => write!(f, "openai-compatible response was not parseable"),
            Self::InvalidReceipt => write!(f, "openai-compatible effect receipt is invalid"),
            Self::InvalidReceiptRecord => {
                write!(f, "openai-compatible effect receipt record is invalid")
            }
            Self::InvalidReplay => write!(
                f,
                "openai-compatible effect receipt failed replay verification"
            ),
            Self::BudgetExhausted => write!(f, "openai-compatible retry budget exhausted"),
            Self::DuplicateRequest => {
                write!(f, "openai-compatible duplicate request identity rejected")
            }
        }
    }
}

impl std::error::Error for OpenAiError {}

impl From<std::io::Error> for OpenAiError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenAiClient {
    config: OpenAiConfig,
}

impl OpenAiClient {
    pub fn new(config: OpenAiConfig) -> Result<Self, OpenAiError> {
        config.validate()?;
        Ok(Self { config })
    }

    pub fn from_env() -> Result<Self, OpenAiError> {
        Self::new(OpenAiConfig::from_env()?)
    }

    pub fn config(&self) -> &OpenAiConfig {
        &self.config
    }

    pub fn request_json(&self, messages: &[OpenAiMessage]) -> Result<String, OpenAiError> {
        self.request_json_for(&OpenAiChatRequest::new(messages.to_vec()))
    }

    pub fn request_json_for(&self, request: &OpenAiChatRequest) -> Result<String, OpenAiError> {
        if request.messages.is_empty() {
            return Err(OpenAiError::InvalidConfig("empty message list"));
        }

        let mut json = String::new();
        json.push_str("{\"model\":\"");
        json.push_str(&json_escape(&self.config.model));
        json.push_str("\",\"messages\":[");
        for (idx, message) in request.messages.iter().enumerate() {
            if idx != 0 {
                json.push(',');
            }
            if !message_contract_valid(message) {
                return Err(OpenAiError::InvalidConfig("empty message"));
            }
            push_message_json(&mut json, message);
        }
        json.push(']');
        if !request.tools.is_empty() {
            json.push_str(",\"tools\":[");
            for (idx, tool) in request.tools.iter().enumerate() {
                if idx != 0 {
                    json.push(',');
                }
                if !tool_contract_valid(tool) {
                    return Err(OpenAiError::InvalidConfig("invalid tool"));
                }
                push_tool_json(&mut json, tool);
            }
            json.push(']');
        }
        if let Some(browser) = &request.browser {
            json.push_str(",\"browser\":{");
            if browser.new_chat {
                json.push_str("\"new_chat\":true");
                if browser.target_url.is_some() {
                    json.push(',');
                }
            }
            if let Some(url) = &browser.target_url {
                if !browser.new_chat {
                    json.push_str("\"new_chat\":false,");
                }
                json.push_str("\"target_url\":\"");
                json.push_str(&json_escape(url));
                json.push('"');
            }
            json.push('}');
        }
        json.push_str(",\"stream\":false}");
        Ok(json)
    }

    pub fn request_hash(&self, messages: &[OpenAiMessage]) -> Result<u64, OpenAiError> {
        Ok(hash_text(&self.request_json(messages)?))
    }

    pub fn request_hash_for(&self, request: &OpenAiChatRequest) -> Result<u64, OpenAiError> {
        Ok(hash_text(&self.request_json_for(request)?))
    }

    pub fn chat(&self, messages: &[OpenAiMessage]) -> Result<OpenAiChatResponse, OpenAiError> {
        let body = self.request_json(messages)?;
        self.chat_body(&body)
    }

    pub fn chat_with_request(
        &self,
        request: &OpenAiChatRequest,
    ) -> Result<OpenAiChatResponse, OpenAiError> {
        let body = self.request_json_for(request)?;
        self.chat_body(&body)
    }

    pub fn chat_with_retry_budget(
        &self,
        request: &OpenAiChatRequest,
        retry_policy: OpenAiRetryBudgetPolicy,
    ) -> Result<(OpenAiChatResponse, OpenAiRetryBudgetDecision), OpenAiError> {
        if retry_policy.timeout_ms != self.config.timeout_ms {
            return Err(OpenAiError::InvalidConfig(
                "retry timeout must match config",
            ));
        }
        let request_hash = self.request_hash_for(request)?;
        let mut last_error = None;
        for retry_count in 0..retry_policy.attempt_budget {
            let decision = retry_policy
                .decision_for_attempt(
                    hash_text(OPENAI_COMPAT_PROVIDER),
                    self.config.base_url_id(),
                    self.config.model_id(),
                    request_hash,
                    retry_count,
                    false,
                )
                .ok_or(OpenAiError::InvalidConfig("invalid retry budget"))?;
            if !decision.allowed {
                return Err(OpenAiError::BudgetExhausted);
            }
            match self.chat_with_request(request) {
                Ok(response) => return Ok((response, decision)),
                Err(err) if !decision.budget_exhausted => last_error = Some(err),
                Err(err) => return Err(err),
            }
        }
        Err(last_error.unwrap_or(OpenAiError::BudgetExhausted))
    }

    pub fn call_from_context(
        &self,
        context: &ContextRecord,
        policy: &PolicyStore,
    ) -> Result<OpenAiLlmCall, OpenAiError> {
        let messages = messages_from_context(context, policy);
        let request = OpenAiChatRequest::new(messages);
        let request_hash = self.request_hash_for(&request)?;
        let (response, retry_budget) = self
            .chat_with_retry_budget(&request, OpenAiRetryBudgetPolicy::from_config(&self.config))?;
        self.call_from_response(context, policy, request_hash, response, retry_budget)
    }

    pub fn record_from_context(
        &self,
        context: &ContextRecord,
        policy: &PolicyStore,
    ) -> Result<LlmRecord, OpenAiError> {
        Ok(self.call_from_context(context, policy)?.record)
    }

    fn call_from_response(
        &self,
        context: &ContextRecord,
        policy: &PolicyStore,
        request_hash: u64,
        response: OpenAiChatResponse,
        retry_budget: OpenAiRetryBudgetDecision,
    ) -> Result<OpenAiLlmCall, OpenAiError> {
        if !retry_budget.is_receiptable_success()
            || retry_budget.timeout_ms != self.config.timeout_ms
        {
            return Err(OpenAiError::BudgetExhausted);
        }
        let record = LlmStructuredAdapter::record_from_external_response(
            context,
            policy,
            self.config.model_id(),
            response.response_hash,
            response.total_tokens.max(1),
        );
        let call = OpenAiLlmCall::new(
            record,
            self.config.model_id(),
            self.config.base_url_id(),
            request_hash,
            response,
            retry_budget,
        );
        call.is_valid()
            .then_some(call)
            .ok_or(OpenAiError::InvalidResponse)
    }

    fn chat_body(&self, body: &str) -> Result<OpenAiChatResponse, OpenAiError> {
        let endpoint = parse_local_endpoint(&self.config.base_url)?;
        let path = self.config.chat_completions_path()?;
        let mut stream = TcpStream::connect((endpoint.host.as_str(), endpoint.port))?;
        let timeout = Duration::from_millis(self.config.timeout_ms);
        stream.set_read_timeout(Some(timeout))?;
        stream.set_write_timeout(Some(timeout))?;

        let request = format!(
            "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nAccept: application/json\r\nConnection: close\r\nContent-Length: {len}\r\n\r\n{body}",
            host = endpoint.host,
            port = endpoint.port,
            len = body.len(),
        );
        stream.write_all(request.as_bytes())?;
        stream.flush()?;

        let mut response = String::new();
        stream.read_to_string(&mut response)?;
        let (status, body) = split_http_response(&response)?;
        if status != 200 {
            return Err(OpenAiError::HttpStatus(status));
        }
        parse_chat_response_body(body)
    }
}

pub fn messages_from_context(context: &ContextRecord, policy: &PolicyStore) -> Vec<OpenAiMessage> {
    vec![
        OpenAiMessage::system(
            "You are the canon agent judgment capability. Return one concise judgment.",
        ),
        OpenAiMessage::user(format!(
            "objective_id={}; observation_hash={}; memory_hash={}; context_hash={}; prior_count={}; policy_version={}; policy_hash={}. Produce a safe next judgment.",
            context.objective_id,
            context.observation_hash,
            context.memory_aggregate_hash,
            context.context_hash,
            context.prior_count,
            policy.latest_version().max(1),
            policy.fingerprint(),
        )),
    ]
}

fn message_contract_valid(message: &OpenAiMessage) -> bool {
    if message.role.trim().is_empty() {
        return false;
    }
    if message.role == "tool" {
        return message
            .tool_call_id
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
            && message
                .content
                .as_ref()
                .is_some_and(|value| !value.trim().is_empty());
    }
    if message.role == "assistant" && !message.tool_calls.is_empty() {
        return message.tool_calls.iter().all(tool_call_contract_valid);
    }
    message
        .content
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty())
}

fn tool_contract_valid(tool: &OpenAiTool) -> bool {
    !tool.function.name.trim().is_empty()
        && tool.function.parameters_json.trim().starts_with('{')
        && tool.function.parameters_json.trim().ends_with('}')
}

fn tool_call_contract_valid(tool_call: &OpenAiToolCall) -> bool {
    !tool_call.id.trim().is_empty()
        && !tool_call.function.name.trim().is_empty()
        && !tool_call.function.arguments.trim().is_empty()
}

fn push_message_json(out: &mut String, message: &OpenAiMessage) {
    out.push_str("{\"role\":\"");
    out.push_str(&json_escape(&message.role));
    out.push('"');
    if let Some(content) = &message.content {
        out.push_str(",\"content\":\"");
        out.push_str(&json_escape(content));
        out.push('"');
    }
    if let Some(tool_call_id) = &message.tool_call_id {
        out.push_str(",\"tool_call_id\":\"");
        out.push_str(&json_escape(tool_call_id));
        out.push('"');
    }
    if !message.tool_calls.is_empty() {
        out.push_str(",\"tool_calls\":[");
        for (idx, tool_call) in message.tool_calls.iter().enumerate() {
            if idx != 0 {
                out.push(',');
            }
            push_tool_call_json(out, tool_call);
        }
        out.push(']');
    }
    out.push('}');
}

fn push_tool_json(out: &mut String, tool: &OpenAiTool) {
    out.push_str("{\"type\":\"function\",\"function\":{\"name\":\"");
    out.push_str(&json_escape(&tool.function.name));
    out.push('"');
    if let Some(description) = &tool.function.description {
        out.push_str(",\"description\":\"");
        out.push_str(&json_escape(description));
        out.push('"');
    }
    out.push_str(",\"parameters\":");
    out.push_str(tool.function.parameters_json.trim());
    out.push_str("}}");
}

fn push_tool_call_json(out: &mut String, tool_call: &OpenAiToolCall) {
    out.push_str("{\"id\":\"");
    out.push_str(&json_escape(&tool_call.id));
    out.push_str("\",\"type\":\"function\",\"function\":{\"name\":\"");
    out.push_str(&json_escape(&tool_call.function.name));
    out.push_str("\",\"arguments\":\"");
    out.push_str(&json_escape(&tool_call.function.arguments));
    out.push_str("\"}}");
}

fn parse_local_endpoint(base_url: &str) -> Result<LocalLlmEndpoint, OpenAiError> {
    parse_local_http_endpoint(base_url).map_err(|err| match err {
        LocalEndpointError::InvalidUrl => OpenAiError::InvalidUrl,
        LocalEndpointError::NonLocalHost => OpenAiError::InvalidConfig("base url must be local"),
    })
}

fn split_http_response(response: &str) -> Result<(u16, &str), OpenAiError> {
    let (head, body) = response
        .split_once("\r\n\r\n")
        .ok_or(OpenAiError::InvalidResponse)?;
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|raw| raw.parse::<u16>().ok())
        .ok_or(OpenAiError::InvalidResponse)?;
    Ok((status, body))
}

fn parse_chat_response_body(body: &str) -> Result<OpenAiChatResponse, OpenAiError> {
    let id = json_string_field(body, "\"id\"").unwrap_or_default();
    let content = message_content_field(body).ok_or(OpenAiError::InvalidResponse)?;
    let prompt_tokens = json_u32_field(body, "\"prompt_tokens\"").unwrap_or(0);
    let completion_tokens = json_u32_field(body, "\"completion_tokens\"").unwrap_or(0);
    let total_tokens = json_u32_field(body, "\"total_tokens\"")
        .unwrap_or_else(|| prompt_tokens.saturating_add(completion_tokens))
        .max(1);
    // The router-server embeds `"browser":{"target_url":"..."}` in the response.
    // Extract it so callers can pin subsequent turns to the same ChatGPT tab.
    let target_url = json_string_field(body, "\"target_url\"").filter(|s| !s.is_empty());
    Ok(OpenAiChatResponse {
        id,
        response_hash: hash_text(&content),
        raw_hash: hash_text(body),
        content,
        prompt_tokens,
        completion_tokens,
        total_tokens,
        target_url,
    })
}

pub fn append_openai_llm_effect_receipt_ndjson(
    path: impl AsRef<Path>,
    receipt: &OpenAiLlmEffectReceipt,
) -> Result<(), OpenAiError> {
    if !receipt.is_valid() {
        return Err(OpenAiError::InvalidReceipt);
    }
    append_openai_ndjson_record(path, encode_openai_llm_effect_receipt_ndjson(*receipt))
}

pub fn load_openai_llm_effect_receipts_ndjson(
    path: impl AsRef<Path>,
) -> Result<Vec<OpenAiLlmEffectReceipt>, OpenAiError> {
    load_openai_ndjson_records(
        path,
        OPENAI_LLM_EFFECT_RECEIPT_RECORD,
        decode_openai_llm_effect_receipt_fields,
    )
}

pub fn verify_openai_llm_effect_receipts(
    tlog: &TLog,
    receipts: &[OpenAiLlmEffectReceipt],
) -> Result<usize, OpenAiError> {
    for receipt in receipts {
        if !receipt.replay_verified(tlog) {
            return Err(OpenAiError::InvalidReplay);
        }
    }
    Ok(receipts.len())
}

pub fn verify_openai_judgment_tlog_ndjson(path: impl AsRef<Path>) -> Result<usize, OpenAiError> {
    let path = path.as_ref();
    let tlog = load_tlog_ndjson(path).map_err(|_| OpenAiError::InvalidReplay)?;
    let receipts = load_openai_llm_effect_receipts_ndjson_unchecked(path)?;
    if receipts.is_empty() {
        return Err(OpenAiError::InvalidReplay);
    }
    verify_openai_llm_effect_receipts(&tlog, &receipts)
}

pub fn append_openai_judgment_proof_event_ndjson(
    path: impl AsRef<Path>,
    event: &OpenAiJudgmentProofEvent,
) -> Result<(), OpenAiError> {
    if !event.is_valid() {
        return Err(OpenAiError::InvalidReceipt);
    }
    append_openai_ndjson_record(path, encode_openai_judgment_proof_event_ndjson(*event))
}

pub fn load_openai_judgment_proof_events_ndjson(
    path: impl AsRef<Path>,
) -> Result<Vec<OpenAiJudgmentProofEvent>, OpenAiError> {
    load_openai_ndjson_records(
        path,
        OPENAI_JUDGMENT_PROOF_RECORD,
        decode_openai_judgment_proof_event_fields,
    )
}

pub fn verify_openai_judgment_proof_event_order_ndjson(
    path: impl AsRef<Path>,
) -> Result<usize, OpenAiError> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(OpenAiError::InvalidReplay);
    }
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut last_control_event_seq = 0u64;
    let mut seen_receipt_hashes = Vec::new();
    let mut verified = 0usize;
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let fields = parse_u64_fields(&line)?;
        if fields.len() < 2 {
            return Err(OpenAiError::InvalidReplay);
        }
        match fields[1] {
            TLOG_RECORD_EVENT => {
                if fields.len() < 3 {
                    return Err(OpenAiError::InvalidReplay);
                }
                last_control_event_seq = fields[2];
            }
            OPENAI_LLM_EFFECT_RECEIPT_RECORD => {
                let receipt = decode_openai_llm_effect_receipt_fields(&fields)?;
                seen_receipt_hashes.push(receipt.receipt_hash);
            }
            OPENAI_JUDGMENT_PROOF_RECORD => {
                let event = decode_openai_judgment_proof_event_fields(&fields)?;
                let actual_proof_event_seq = last_control_event_seq
                    .checked_add(1)
                    .filter(|seq| *seq != 0)
                    .ok_or(OpenAiError::InvalidReplay)?;
                if event.proof_event_seq != actual_proof_event_seq
                    || !seen_receipt_hashes.contains(&event.receipt_hash)
                {
                    return Err(OpenAiError::InvalidReplay);
                }
                verified += 1;
            }
            _ => {}
        }
    }
    if verified == 0 {
        return Err(OpenAiError::InvalidReplay);
    }
    Ok(verified)
}

pub fn verify_openai_judgment_proof_events(
    tlog: &TLog,
    receipts: &[OpenAiLlmEffectReceipt],
    events: &[OpenAiJudgmentProofEvent],
) -> Result<usize, OpenAiError> {
    if events.is_empty() {
        return Err(OpenAiError::InvalidReplay);
    }
    let mut proof_records = Vec::new();
    let mut proof_bindings = Vec::new();
    for event in events {
        let receipt = receipts
            .iter()
            .copied()
            .find(|receipt| event.matches_receipt(*receipt, tlog))
            .ok_or(OpenAiError::InvalidReplay)?;
        let (_, proof_record) = event
            .to_canonical_verification_proof_record(receipt)
            .ok_or(OpenAiError::InvalidReplay)?;
        let proof_binding = receipt
            .verification_proof_binding()
            .ok_or(OpenAiError::InvalidReplay)?;
        proof_records.push(proof_record);
        proof_bindings.push(proof_binding);
    }
    verify_verification_proof_record_bindings(&proof_records, &proof_bindings)
        .map_err(|_| OpenAiError::InvalidReplay)
}

pub fn verify_openai_judgment_proof_events_ndjson(
    path: impl AsRef<Path>,
) -> Result<usize, OpenAiError> {
    let path = path.as_ref();
    let tlog = load_tlog_ndjson(path).map_err(|_| OpenAiError::InvalidReplay)?;
    let receipts = load_openai_llm_effect_receipts_ndjson(path)?;
    let events = load_openai_judgment_proof_events_ndjson(path)?;
    let verified = verify_openai_judgment_proof_events(&tlog, &receipts, &events)?;
    if verify_openai_judgment_proof_event_order_ndjson(path)? != verified {
        return Err(OpenAiError::InvalidReplay);
    }
    Ok(verified)
}

pub fn encode_openai_llm_effect_receipt_ndjson(receipt: OpenAiLlmEffectReceipt) -> String {
    let fields = [
        OPENAI_LLM_EFFECT_RECEIPT_SCHEMA_VERSION,
        OPENAI_LLM_EFFECT_RECEIPT_RECORD,
        receipt.provider_hash,
        receipt.base_url_hash,
        receipt.model_id,
        receipt.request_hash,
        receipt.timeout_ms,
        receipt.retry_count as u64,
        receipt.max_retries as u64,
        receipt.attempt_budget as u64,
        receipt.request_identity_hash,
        receipt.retry_budget_hash,
        receipt.budget_exhausted as u64,
        receipt.duplicate_request as u64,
        receipt.response_hash,
        receipt.raw_response_hash,
        receipt.prompt_hash,
        receipt.token_count as u64,
        receipt.payload_hash,
        receipt.command_hash,
        receipt.event_seq,
        receipt.event_hash,
        receipt.proof_event_seq,
        receipt.proof_hash,
        receipt.receipt_hash,
    ];
    encode_openai_u64_fields_ndjson(&fields)
}

pub fn encode_openai_judgment_proof_event_ndjson(event: OpenAiJudgmentProofEvent) -> String {
    let fields = [
        OPENAI_JUDGMENT_PROOF_SCHEMA_VERSION,
        OPENAI_JUDGMENT_PROOF_RECORD,
        event.proof_line_hash,
        event.receipt_core_hash,
        event.receipt_hash,
        event.receipt_event_seq,
        event.proof_event_seq,
        event.receipt_event_hash,
        event.base_url_hash,
        event.model_id,
        event.timeout_ms,
        event.retry_count as u64,
        event.max_retries as u64,
        event.attempt_budget as u64,
        event.request_identity_hash,
        event.retry_budget_hash,
        event.budget_exhausted as u64,
        event.duplicate_request as u64,
        event.receipt_verified as u64,
        event.tamper_rejected as u64,
        event.endpoint_verified as u64,
        event.phase_plan as u64,
        event.proof_hash,
    ];
    encode_openai_u64_fields_ndjson(&fields)
}

fn encode_openai_u64_fields_ndjson(fields: &[u64]) -> String {
    format!(
        "[{}]",
        fields
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub fn decode_openai_llm_effect_receipt_ndjson(
    line: &str,
) -> Result<OpenAiLlmEffectReceipt, OpenAiError> {
    decode_openai_llm_effect_receipt_fields(&parse_u64_fields(line)?)
}

fn decode_openai_llm_effect_receipt_fields(
    fields: &[u64],
) -> Result<OpenAiLlmEffectReceipt, OpenAiError> {
    let receipt = decode_openai_llm_effect_receipt_fields_unchecked(fields)?;
    receipt
        .is_valid()
        .then_some(receipt)
        .ok_or(OpenAiError::InvalidReceipt)
}

fn decode_openai_llm_effect_receipt_fields_unchecked(
    fields: &[u64],
) -> Result<OpenAiLlmEffectReceipt, OpenAiError> {
    if fields.len() != 25
        || fields[0] != OPENAI_LLM_EFFECT_RECEIPT_SCHEMA_VERSION
        || fields[1] != OPENAI_LLM_EFFECT_RECEIPT_RECORD
    {
        return Err(OpenAiError::InvalidReceiptRecord);
    }

    let receipt = OpenAiLlmEffectReceipt {
        provider_hash: fields[2],
        base_url_hash: fields[3],
        model_id: fields[4],
        request_hash: fields[5],
        timeout_ms: fields[6],
        retry_count: u32::try_from(fields[7]).map_err(|_| OpenAiError::InvalidReceiptRecord)?,
        max_retries: u32::try_from(fields[8]).map_err(|_| OpenAiError::InvalidReceiptRecord)?,
        attempt_budget: u32::try_from(fields[9]).map_err(|_| OpenAiError::InvalidReceiptRecord)?,
        request_identity_hash: fields[10],
        retry_budget_hash: fields[11],
        budget_exhausted: fields[12] == 1,
        duplicate_request: fields[13] == 1,
        response_hash: fields[14],
        raw_response_hash: fields[15],
        prompt_hash: fields[16],
        token_count: u32::try_from(fields[17]).map_err(|_| OpenAiError::InvalidReceiptRecord)?,
        payload_hash: fields[18],
        command_hash: fields[19],
        event_seq: fields[20],
        event_hash: fields[21],
        proof_event_seq: fields[22],
        proof_hash: fields[23],
        receipt_hash: fields[24],
    };
    Ok(receipt)
}

pub fn decode_openai_judgment_proof_event_ndjson(
    line: &str,
) -> Result<OpenAiJudgmentProofEvent, OpenAiError> {
    decode_openai_judgment_proof_event_fields(&parse_u64_fields(line)?)
}

fn decode_openai_judgment_proof_event_fields(
    fields: &[u64],
) -> Result<OpenAiJudgmentProofEvent, OpenAiError> {
    if fields.len() != 23
        || fields[0] != OPENAI_JUDGMENT_PROOF_SCHEMA_VERSION
        || fields[1] != OPENAI_JUDGMENT_PROOF_RECORD
    {
        return Err(OpenAiError::InvalidReceiptRecord);
    }

    let event = OpenAiJudgmentProofEvent {
        proof_line_hash: fields[2],
        receipt_core_hash: fields[3],
        receipt_hash: fields[4],
        receipt_event_seq: fields[5],
        proof_event_seq: fields[6],
        receipt_event_hash: fields[7],
        base_url_hash: fields[8],
        model_id: fields[9],
        timeout_ms: fields[10],
        retry_count: u32::try_from(fields[11]).map_err(|_| OpenAiError::InvalidReceiptRecord)?,
        max_retries: u32::try_from(fields[12]).map_err(|_| OpenAiError::InvalidReceiptRecord)?,
        attempt_budget: u32::try_from(fields[13]).map_err(|_| OpenAiError::InvalidReceiptRecord)?,
        request_identity_hash: fields[14],
        retry_budget_hash: fields[15],
        budget_exhausted: fields[16] == 1,
        duplicate_request: fields[17] == 1,
        receipt_verified: fields[18] == 1,
        tamper_rejected: fields[19] == 1,
        endpoint_verified: fields[20] == 1,
        phase_plan: fields[21] == 1,
        proof_hash: fields[22],
    };

    event
        .is_valid()
        .then_some(event)
        .ok_or(OpenAiError::InvalidReceipt)
}

fn parse_u64_fields(line: &str) -> Result<Vec<u64>, OpenAiError> {
    let body = line
        .trim()
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .ok_or(OpenAiError::InvalidReceiptRecord)?;
    if body.trim().is_empty() {
        return Ok(Vec::new());
    }
    body.split(',')
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| OpenAiError::InvalidReceiptRecord)
        })
        .collect()
}

fn load_openai_llm_effect_receipts_ndjson_unchecked(
    path: impl AsRef<Path>,
) -> Result<Vec<OpenAiLlmEffectReceipt>, OpenAiError> {
    load_openai_ndjson_records(
        path,
        OPENAI_LLM_EFFECT_RECEIPT_RECORD,
        decode_openai_llm_effect_receipt_fields_unchecked,
    )
}

fn append_openai_ndjson_record(
    path: impl AsRef<Path>,
    encoded_record: String,
) -> Result<(), OpenAiError> {
    let path = path.as_ref();
    ensure_openai_record_parent(path)?;

    {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(file, "{}", encoded_record)?;
        file.sync_all()?;
    }

    sync_parent_dir(path)
}

fn load_openai_ndjson_records<T, F>(
    path: impl AsRef<Path>,
    record_tag: u64,
    decode: F,
) -> Result<Vec<T>, OpenAiError>
where
    F: Fn(&[u64]) -> Result<T, OpenAiError>,
{
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let fields = parse_u64_fields(&line)?;
        if fields.len() >= 2 && fields[1] == record_tag {
            records.push(decode(&fields)?);
        }
    }
    Ok(records)
}

fn ensure_openai_record_parent(path: &Path) -> Result<(), OpenAiError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn sync_parent_dir(path: &Path) -> Result<(), OpenAiError> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() {
        return Ok(());
    }
    let dir = File::open(parent)?;
    dir.sync_all()?;
    Ok(())
}

fn message_content_field(body: &str) -> Option<String> {
    let message_idx = body.find("\"message\"")?;
    scoped_json_string_field(body, message_idx, "\"content\"")
}

fn json_string_field(body: &str, field: &str) -> Option<String> {
    scoped_json_string_field(body, 0, field)
}

fn scoped_json_string_field(body: &str, start_idx: usize, field: &str) -> Option<String> {
    let idx = body[start_idx..].find(field)? + start_idx;
    json_string_at(body, idx)
}

fn json_string_at(body: &str, field_idx: usize) -> Option<String> {
    let after_field = &body[field_idx..];
    let colon = after_field.find(':')?;
    let after_colon = after_field[colon + 1..].trim_start();
    decode_json_string(after_colon)
}

fn json_u32_field(body: &str, field: &str) -> Option<u32> {
    let idx = body.find(field)?;
    let after_field = &body[idx..];
    let colon = after_field.find(':')?;
    let digits = after_field[colon + 1..]
        .trim_start()
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    digits.parse::<u32>().ok()
}

fn decode_json_string(raw: &str) -> Option<String> {
    let mut chars = raw.chars();
    if chars.next()? != '"' {
        return None;
    }

    let mut out = String::new();
    let mut escaped = false;
    for ch in chars {
        if escaped {
            match ch {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                '/' => out.push('/'),
                'b' => out.push('\u{0008}'),
                'f' => out.push('\u{000c}'),
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                'u' => return None,
                _ => return None,
            }
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' => return Some(out),
            _ => out.push(ch),
        }
    }
    None
}

fn json_escape(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

fn hash_text(value: &str) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    h = mix(h, value.len() as u64);
    for byte in value.as_bytes() {
        h = mix(h, *byte as u64);
    }
    h.max(1)
}

#[expect(
    clippy::too_many_arguments,
    reason = "retry-budget binding compares the full receipt hash preimage"
)]
fn retry_budget_binding_is_valid(
    provider_hash: u64,
    base_url_hash: u64,
    model_id: u64,
    request_hash: u64,
    timeout_ms: u64,
    retry_count: u32,
    max_retries: u32,
    attempt_budget: u32,
    request_identity_hash: u64,
    retry_budget_hash: u64,
) -> bool {
    let Some(policy) = OpenAiRetryBudgetPolicy::new(timeout_ms, max_retries, attempt_budget) else {
        return false;
    };
    retry_count <= max_retries
        && retry_count < attempt_budget
        && retry_budget_hash == policy.policy_hash()
        && request_identity_hash
            == policy.request_identity_hash(provider_hash, base_url_hash, model_id, request_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_config_id_helpers_preserve_distinct_field_boundaries() {
        let config = OpenAiConfig {
            base_url: "http://127.0.0.1:8082/v1".to_string(),
            model: "chatgpt-project".to_string(),
            timeout_ms: 120_000,
        };

        let model_id = config.model_id();
        let base_url_id = config.base_url_id();

        assert_ne!(model_id, 0);
        assert_ne!(base_url_id, 0);
        assert_eq!(model_id, config.model_id());
        assert_eq!(base_url_id, config.base_url_id());
        assert_ne!(model_id, base_url_id);

        let model_changed = OpenAiConfig {
            model: "chatgpt-project-alt".to_string(),
            ..config.clone()
        };
        assert_ne!(model_changed.model_id(), model_id);
        assert_eq!(model_changed.base_url_id(), base_url_id);

        let base_url_changed = OpenAiConfig {
            base_url: "http://127.0.0.1:8083/v1".to_string(),
            ..config.clone()
        };
        assert_eq!(base_url_changed.model_id(), model_id);
        assert_ne!(base_url_changed.base_url_id(), base_url_id);
    }

    #[test]
    fn openai_message_constructors_preserve_role_boundaries() {
        let system = OpenAiMessage::system("system prompt");
        assert_eq!(system.role, "system");
        assert_eq!(system.content.as_deref(), Some("system prompt"));
        assert_eq!(system.tool_call_id, None);
        assert!(system.tool_calls.is_empty());
        assert!(message_contract_valid(&system));

        let user = OpenAiMessage::user("user prompt");
        assert_eq!(user.role, "user");
        assert_eq!(user.content.as_deref(), Some("user prompt"));
        assert_eq!(user.tool_call_id, None);
        assert!(user.tool_calls.is_empty());
        assert!(message_contract_valid(&user));

        let assistant = OpenAiMessage::assistant("assistant answer");
        assert_eq!(assistant.role, "assistant");
        assert_eq!(assistant.content.as_deref(), Some("assistant answer"));
        assert_eq!(assistant.tool_call_id, None);
        assert!(assistant.tool_calls.is_empty());
        assert!(message_contract_valid(&assistant));

        let tool_call = OpenAiToolCall::function("call-1", "lookup", "{\"query\":\"canon\"}");
        let assistant_tool_call = OpenAiMessage::assistant_tool_call(tool_call.clone());
        assert_eq!(assistant_tool_call.role, "assistant");
        assert_eq!(assistant_tool_call.content, None);
        assert_eq!(assistant_tool_call.tool_call_id, None);
        assert_eq!(assistant_tool_call.tool_calls, vec![tool_call]);
        assert!(message_contract_valid(&assistant_tool_call));

        let tool = OpenAiMessage::tool("call-1", "tool result");
        assert_eq!(tool.role, "tool");
        assert_eq!(tool.content.as_deref(), Some("tool result"));
        assert_eq!(tool.tool_call_id.as_deref(), Some("call-1"));
        assert!(tool.tool_calls.is_empty());
        assert!(message_contract_valid(&tool));
    }

    #[test]
    fn openai_chat_response_parser_preserves_message_scoped_content_fields() {
        let body = r#"{
            "id":"chatcmpl-parser-scope",
            "metadata":{"content":"outer content must not win"},
            "choices":[{
                "message":{
                    "role":"assistant",
                    "content":"message scoped content wins"
                }
            }],
            "usage":{
                "prompt_tokens":7,
                "completion_tokens":11,
                "total_tokens":18
            },
            "browser":{"target_url":"http://127.0.0.1:9222/devtools/page/abc"}
        }"#;

        let response = parse_chat_response_body(body).expect("body should parse");

        assert_eq!(response.id, "chatcmpl-parser-scope");
        assert_eq!(response.content, "message scoped content wins");
        assert_ne!(response.content, "outer content must not win");
        assert_eq!(response.prompt_tokens, 7);
        assert_eq!(response.completion_tokens, 11);
        assert_eq!(response.total_tokens, 18);
        assert_eq!(
            response.target_url.as_deref(),
            Some("http://127.0.0.1:9222/devtools/page/abc")
        );
        assert_eq!(response.response_hash, hash_text(&response.content));
        assert_eq!(response.raw_hash, hash_text(body));

        let repeated = parse_chat_response_body(body).expect("body should parse deterministically");
        assert_eq!(repeated, response);
    }

    #[test]
    fn openai_function_tool_constructors_preserve_description_boundary() {
        let parameters_json = r#"{"type":"object","properties":{"query":{"type":"string"}}}"#;
        let bare = OpenAiFunctionTool::new("lookup", parameters_json);

        assert_eq!(bare.name, "lookup");
        assert_eq!(bare.parameters_json, parameters_json);
        assert_eq!(bare.description, None);
        assert_eq!(
            bare,
            OpenAiTool::function("lookup", parameters_json).function
        );

        let described = bare
            .clone()
            .with_description("Lookup local indexed records");

        assert_eq!(described.name, bare.name);
        assert_eq!(described.parameters_json, bare.parameters_json);
        assert_eq!(
            described.description.as_deref(),
            Some("Lookup local indexed records")
        );

        let mut expected = bare.clone();
        expected.description = Some("Lookup local indexed records".to_string());
        assert_eq!(described, expected);
        assert_ne!(described, bare);
    }
}
