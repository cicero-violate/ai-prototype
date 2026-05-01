//! Live Ollama/OpenAI-compatible LLM adapter.
//!
//! This module is intentionally dependency-free. It calls Ollama's local
//! OpenAI-compatible `/v1/chat/completions` endpoint through a minimal HTTP/1.1
//! client, then converts the returned assistant text into the same
//! `LlmRecord -> Evidence::JudgmentRecord` path used by deterministic tests.

use std::env;
use std::fs::{self, File, OpenOptions};
use std::fmt;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

use crate::capability::context::ContextRecord;
use crate::capability::llm::record::{LlmRecord, LlmStructuredAdapter};
use crate::capability::policy::PolicyStore;
use crate::capability::verification::{
    verify_verification_proof_record_bindings, GenericVerificationProofSubject,
    ProofSubjectKind, VerificationProofBinding, VerificationProofRecord,
    PROOF_FLAG_PHASE_VERIFIED, PROOF_FLAG_PROVENANCE_VERIFIED, PROOF_FLAG_RECEIPT_VERIFIED,
    PROOF_FLAG_TAMPER_REJECTED,
};
use crate::capability::EvidenceSubmission;
use crate::codec::ndjson::{load_tlog_ndjson, TLOG_RECORD_EVENT};
use crate::kernel::{
    mix, Cause, ControlEvent, Decision, EventKind, Evidence, GateId, GateStatus, Phase, TLog,
};

pub const OLLAMA_PROVIDER: &str = "ollama";
pub const OLLAMA_LLM_EFFECT_RECEIPT_SCHEMA_VERSION: u64 = 5;
pub const OLLAMA_LLM_EFFECT_RECEIPT_RECORD: u64 = 51;
pub const OLLAMA_JUDGMENT_PROOF_SCHEMA_VERSION: u64 = 3;
pub const OLLAMA_JUDGMENT_PROOF_RECORD: u64 = 52;
pub const OLLAMA_JUDGMENT_PROOF_LINE: &str =
    "receipt_verified+tamper_rejected+endpoint_verified+phase_plan";

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:11434/v1";
const DEFAULT_MODEL: &str = "qwen2.5-coder:7b";
const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_MAX_RETRIES: u32 = 0;
const DEFAULT_ATTEMPT_BUDGET: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
    pub timeout_ms: u64,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            model: DEFAULT_MODEL.to_string(),
            timeout_ms: DEFAULT_TIMEOUT_MS,
        }
    }
}

impl OllamaConfig {
    pub fn from_env() -> Result<Self, OllamaError> {
        let timeout_ms = env::var("CANON_OLLAMA_TIMEOUT_MS")
            .ok()
            .and_then(|raw| raw.parse::<u64>().ok())
            .unwrap_or(DEFAULT_TIMEOUT_MS);
        let cfg = Self {
            base_url: env::var("CANON_OLLAMA_BASE_URL")
                .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string()),
            model: env::var("CANON_OLLAMA_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string()),
            timeout_ms,
        };
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<(), OllamaError> {
        if self.model.trim().is_empty() {
            return Err(OllamaError::InvalidConfig("empty model"));
        }
        if self.timeout_ms == 0 {
            return Err(OllamaError::InvalidConfig("zero timeout"));
        }
        parse_local_endpoint(&self.base_url)?;
        Ok(())
    }

    pub fn chat_completions_path(&self) -> Result<String, OllamaError> {
        let endpoint = parse_local_endpoint(&self.base_url)?;
        let prefix = endpoint.path_prefix.trim_end_matches('/');
        if prefix.is_empty() {
            Ok("/v1/chat/completions".to_string())
        } else {
            Ok(format!("{prefix}/chat/completions"))
        }
    }

    pub fn model_id(&self) -> u64 {
        hash_text(&self.model)
    }

    pub fn base_url_id(&self) -> u64 {
        hash_text(&self.base_url)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OllamaRetryBudgetPolicy {
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub attempt_budget: u32,
}

impl OllamaRetryBudgetPolicy {
    pub fn new(timeout_ms: u64, max_retries: u32, attempt_budget: u32) -> Option<Self> {
        if timeout_ms == 0 || attempt_budget == 0 {
            return None;
        }
        let retry_ceiling = max_retries.saturating_add(1);
        if attempt_budget > retry_ceiling {
            return None;
        }
        Some(Self {
            timeout_ms,
            max_retries,
            attempt_budget,
        })
    }

    pub fn from_config(config: &OllamaConfig) -> Self {
        Self {
            timeout_ms: config.timeout_ms,
            max_retries: DEFAULT_MAX_RETRIES,
            attempt_budget: DEFAULT_ATTEMPT_BUDGET,
        }
    }

    pub fn policy_hash(self) -> u64 {
        let mut h = 0x5245_5452_5942_5544u64;
        h = mix(h, self.timeout_ms);
        h = mix(h, self.max_retries as u64);
        h = mix(h, self.attempt_budget as u64);
        h.max(1)
    }

    pub fn request_identity_hash(
        self,
        provider_hash: u64,
        base_url_hash: u64,
        model_id: u64,
        request_hash: u64,
    ) -> u64 {
        let mut h = 0x4944_454d_504f_5445u64;
        h = mix(h, provider_hash);
        h = mix(h, base_url_hash);
        h = mix(h, model_id);
        h = mix(h, request_hash);
        h.max(1)
    }

    pub fn decision_for_attempt(
        self,
        provider_hash: u64,
        base_url_hash: u64,
        model_id: u64,
        request_hash: u64,
        retry_count: u32,
        duplicate_request: bool,
    ) -> Option<OllamaRetryBudgetDecision> {
        let request_identity_hash =
            self.request_identity_hash(provider_hash, base_url_hash, model_id, request_hash);
        let retry_allowed = retry_count <= self.max_retries;
        let budget_allowed = retry_count < self.attempt_budget;
        let allowed = request_identity_hash != 0 && retry_allowed && budget_allowed && !duplicate_request;
        let budget_exhausted = !budget_allowed
            || retry_count.saturating_add(1) >= self.attempt_budget
            || retry_count >= self.max_retries;
        let decision = OllamaRetryBudgetDecision {
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
    ) -> Option<OllamaRetryBudgetDecision> {
        self.decision_for_attempt(provider_hash, base_url_hash, model_id, request_hash, 0, false)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OllamaRetryBudgetDecision {
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

impl OllamaRetryBudgetDecision {
    pub fn is_valid_decision(self) -> bool {
        self.timeout_ms != 0
            && self.attempt_budget != 0
            && self.attempt_budget <= self.max_retries.saturating_add(1)
            && self.request_identity_hash != 0
            && self.retry_budget_hash != 0
            && (!self.allowed
                || (!self.duplicate_request
                    && self.retry_count <= self.max_retries
                    && self.retry_count < self.attempt_budget))
    }

    pub fn is_receiptable_success(self) -> bool {
        self.is_valid_decision()
            && self.allowed
            && !self.duplicate_request
            && self.retry_count <= self.max_retries
            && self.retry_count < self.attempt_budget
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OllamaRetryBudgetLedger {
    policy: OllamaRetryBudgetPolicy,
    seen_request_identity_hashes: Vec<u64>,
    attempts_used: u32,
}

impl OllamaRetryBudgetLedger {
    pub fn new(policy: OllamaRetryBudgetPolicy) -> Self {
        Self {
            policy,
            seen_request_identity_hashes: Vec::new(),
            attempts_used: 0,
        }
    }

    pub fn policy(&self) -> OllamaRetryBudgetPolicy {
        self.policy
    }

    pub fn record_request(
        &mut self,
        provider_hash: u64,
        base_url_hash: u64,
        model_id: u64,
        request_hash: u64,
    ) -> Result<OllamaRetryBudgetDecision, OllamaError> {
        let request_identity_hash = self.policy.request_identity_hash(
            provider_hash,
            base_url_hash,
            model_id,
            request_hash,
        );
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
            .ok_or(OllamaError::InvalidConfig("invalid retry budget decision"))?;

        if duplicate_request {
            return Err(OllamaError::DuplicateRequest);
        }
        if !decision.allowed {
            return Err(OllamaError::BudgetExhausted);
        }

        self.seen_request_identity_hashes
            .push(request_identity_hash);
        self.attempts_used = self.attempts_used.saturating_add(1);
        Ok(decision)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}

impl OllamaMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OllamaChatResponse {
    pub content: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub response_hash: u64,
    pub raw_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OllamaLlmCall {
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

impl OllamaLlmCall {
    fn new(
        record: LlmRecord,
        model_id: u64,
        base_url_hash: u64,
        request_hash: u64,
        response: OllamaChatResponse,
        retry_budget: OllamaRetryBudgetDecision,
    ) -> Self {
        let submission = record.submission();
        Self {
            prompt_hash: record.prompt.prompt_hash,
            response_hash: record.response.response_hash,
            token_count: record.response.token_count,
            payload_hash: submission.payload_hash,
            record,
            provider_hash: ollama_provider_hash(),
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
            && self.provider_hash == ollama_provider_hash()
            && self.base_url_hash != 0
            && self.model_id != 0
            && self.model_id == self.record.response.model_id
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
            && self.response_hash == self.record.response.response_hash
            && self.raw_response_hash != 0
            && self.prompt_hash != 0
            && self.prompt_hash == self.record.prompt.prompt_hash
            && self.token_count != 0
            && self.token_count == self.record.response.token_count
            && self.payload_hash != 0
            && self.payload_hash == self.record.submission().payload_hash
    }

    pub fn submission(&self) -> EvidenceSubmission {
        self.record.submission()
    }

    pub fn base_url_provenance_verified(&self, config: &OllamaConfig) -> bool {
        config.validate().is_ok()
            && self.is_valid()
            && self.provider_hash == ollama_provider_hash()
            && self.base_url_hash == config.base_url_id()
            && self.model_id == config.model_id()
            && self.timeout_ms == config.timeout_ms
    }

    pub fn receipt_for_configured_event(
        &self,
        config: &OllamaConfig,
        command_hash: u64,
        event: &ControlEvent,
    ) -> Option<OllamaLlmEffectReceipt> {
        self.base_url_provenance_verified(config)
            .then(|| self.receipt_for_event_unchecked(command_hash, event))
            .flatten()
    }

    pub fn receipt_for_configured_event_with_retry_budget(
        &self,
        config: &OllamaConfig,
        retry_budget: OllamaRetryBudgetDecision,
        command_hash: u64,
        event: &ControlEvent,
    ) -> Option<OllamaLlmEffectReceipt> {
        if !retry_budget.is_receiptable_success()
            || retry_budget.timeout_ms != config.timeout_ms
            || retry_budget.retry_count != self.retry_count
            || retry_budget.max_retries != self.max_retries
            || retry_budget.attempt_budget != self.attempt_budget
            || retry_budget.request_identity_hash != self.request_identity_hash
            || retry_budget.retry_budget_hash != self.retry_budget_hash
            || retry_budget.budget_exhausted != self.budget_exhausted
            || retry_budget.duplicate_request != self.duplicate_request
        {
            return None;
        }
        self.receipt_for_configured_event(config, command_hash, event)
    }

    fn receipt_for_event_unchecked(
        &self,
        command_hash: u64,
        event: &ControlEvent,
    ) -> Option<OllamaLlmEffectReceipt> {
        if !self.is_valid() || command_hash == 0 {
            return None;
        }

        let mut receipt = OllamaLlmEffectReceipt {
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
        config: &OllamaConfig,
        command_hash: u64,
        tlog: &TLog,
    ) -> Option<OllamaLlmEffectReceipt> {
        tlog.iter()
            .find_map(|event| self.receipt_for_configured_event(config, command_hash, event))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OllamaLlmEffectReceipt {
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

impl OllamaLlmEffectReceipt {
    pub fn is_valid(self) -> bool {
        self.provider_hash == ollama_provider_hash()
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

    pub fn next_proof_event_seq(self) -> Option<u64> {
        self.event_seq.checked_add(1)
    }

    pub fn expected_receipt_core_hash(self) -> u64 {
        let mut h = 0x8e1f_9628_8f75_4b2du64;
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

    pub fn base_url_provenance_verified(self, config: &OllamaConfig) -> bool {
        config.validate().is_ok()
            && self.is_valid()
            && self.provider_hash == ollama_provider_hash()
            && self.base_url_hash == config.base_url_id()
            && self.model_id == config.model_id()
            && self.timeout_ms == config.timeout_ms
            && !self.duplicate_request
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
pub struct OllamaJudgmentProofEvent {
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

impl OllamaJudgmentProofEvent {
    pub fn new(
        receipt: OllamaLlmEffectReceipt,
        receipt_verified: bool,
        tamper_rejected: bool,
        endpoint_verified: bool,
        phase_plan: bool,
    ) -> Option<Self> {
        Self::finalize_receipt(
            receipt,
            receipt_verified,
            tamper_rejected,
            endpoint_verified,
            phase_plan,
        )
        .map(|(_, event)| event)
    }

    pub fn finalize_receipt(
        receipt: OllamaLlmEffectReceipt,
        receipt_verified: bool,
        tamper_rejected: bool,
        endpoint_verified: bool,
        phase_plan: bool,
    ) -> Option<(OllamaLlmEffectReceipt, Self)> {
        let proof_event_seq = receipt.next_proof_event_seq()?;
        Self::finalize_receipt_at_seq(
            receipt,
            proof_event_seq,
            receipt_verified,
            tamper_rejected,
            endpoint_verified,
            phase_plan,
        )
    }

    pub fn finalize_receipt_after_tlog(
        receipt: OllamaLlmEffectReceipt,
        tlog: &TLog,
        receipt_verified: bool,
        tamper_rejected: bool,
        endpoint_verified: bool,
        phase_plan: bool,
    ) -> Option<(OllamaLlmEffectReceipt, Self)> {
        if !receipt.replay_verified(tlog) {
            return None;
        }
        let proof_event_seq = tlog
            .last()
            .map(|event| event.seq)
            .unwrap_or(receipt.event_seq)
            .checked_add(1)?;
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
        receipt: OllamaLlmEffectReceipt,
        proof_event_seq: u64,
        receipt_verified: bool,
        tamper_rejected: bool,
        endpoint_verified: bool,
        phase_plan: bool,
    ) -> Option<(OllamaLlmEffectReceipt, Self)> {
        if !receipt.is_valid()
            || !receipt_verified
            || !tamper_rejected
            || !endpoint_verified
            || !phase_plan
            || proof_event_seq <= receipt.event_seq
        {
            return None;
        }

        let receipt_core_hash = receipt.expected_receipt_core_hash();
        let mut event = Self {
            proof_line_hash: hash_text(OLLAMA_JUDGMENT_PROOF_LINE),
            receipt_core_hash,
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
        let finalized_receipt = receipt.bind_proof_event(event.proof_event_seq, event.proof_hash)?;
        event.receipt_hash = finalized_receipt.receipt_hash;
        event
            .is_valid()
            .then_some((finalized_receipt, event))
    }

    pub fn is_valid(self) -> bool {
        self.proof_line_hash == hash_text(OLLAMA_JUDGMENT_PROOF_LINE)
            && self.receipt_core_hash != 0
            && self.receipt_hash != 0
            && self.receipt_event_seq != 0
            && self.proof_event_seq != 0
            && self.proof_event_seq > self.receipt_event_seq
            && self.receipt_event_hash != 0
            && self.base_url_hash != 0
            && self.model_id != 0
            && self.timeout_ms != 0
            && self.retry_count <= self.max_retries
            && self.attempt_budget != 0
            && self.attempt_budget <= self.max_retries.saturating_add(1)
            && self.retry_count < self.attempt_budget
            && self.request_identity_hash != 0
            && self.retry_budget_hash != 0
            && !self.duplicate_request
            && self.receipt_verified
            && self.tamper_rejected
            && self.endpoint_verified
            && self.phase_plan
            && self.proof_hash != 0
            && self.proof_hash == self.expected_proof_hash()
    }

    pub fn expected_proof_hash(self) -> u64 {
        let mut h = 0x4f4c_4c41_4d41_5652u64;
        h = mix(h, self.proof_line_hash);
        h = mix(h, self.receipt_core_hash);
        h = mix(h, self.receipt_event_seq);
        h = mix(h, self.proof_event_seq);
        h = mix(h, self.receipt_event_hash);
        h = mix(h, self.base_url_hash);
        h = mix(h, self.model_id);
        h = mix(h, self.timeout_ms);
        h = mix(h, self.retry_count as u64);
        h = mix(h, self.max_retries as u64);
        h = mix(h, self.attempt_budget as u64);
        h = mix(h, self.request_identity_hash);
        h = mix(h, self.retry_budget_hash);
        h = mix(h, self.budget_exhausted as u64);
        h = mix(h, self.duplicate_request as u64);
        h = mix(h, self.receipt_verified as u64);
        h = mix(h, self.tamper_rejected as u64);
        h = mix(h, self.endpoint_verified as u64);
        h = mix(h, self.phase_plan as u64);
        h.max(1)
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
        let mut h = 0x4f4c_4c41_4d41_4354u64;
        h = mix(h, self.base_url_hash);
        h = mix(h, self.model_id);
        h = mix(h, self.timeout_ms);
        h = mix(h, self.retry_count as u64);
        h = mix(h, self.max_retries as u64);
        h = mix(h, self.attempt_budget as u64);
        h = mix(h, self.request_identity_hash);
        h = mix(h, self.retry_budget_hash);
        h = mix(h, self.budget_exhausted as u64);
        h = mix(h, self.duplicate_request as u64);
        h.max(1)
    }

    pub fn verification_proof_subject(self) -> Option<GenericVerificationProofSubject> {
        if !self.is_valid() {
            return None;
        }

        GenericVerificationProofSubject::new(
            ProofSubjectKind::LlmEffect,
            self.proof_line_hash,
            self.receipt_core_hash,
            self.receipt_hash,
            self.receipt_event_seq,
            self.proof_event_seq,
            self.receipt_event_hash,
            self.verifier_context_hash(),
            self.proof_flags(),
            self.proof_hash,
        )
    }

    pub fn to_verification_proof_record(self) -> Option<VerificationProofRecord> {
        VerificationProofRecord::from_subject(self.verification_proof_subject()?)
    }

    pub fn matches_receipt(self, receipt: OllamaLlmEffectReceipt, tlog: &TLog) -> bool {
        self.is_valid()
            && receipt.replay_verified(tlog)
            && receipt.has_proof_binding()
            && self.proof_hash == receipt.proof_hash
            && self.receipt_core_hash == receipt.expected_receipt_core_hash()
            && self.receipt_hash == receipt.receipt_hash
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
}

#[derive(Debug)]
pub enum OllamaError {
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

impl fmt::Display for OllamaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(reason) => write!(f, "invalid ollama config: {reason}"),
            Self::InvalidUrl => write!(f, "invalid local ollama url"),
            Self::HttpStatus(status) => write!(f, "ollama returned HTTP {status}"),
            Self::Io(err) => write!(f, "ollama io failed: {err}"),
            Self::InvalidResponse => write!(f, "ollama response was not parseable"),
            Self::InvalidReceipt => write!(f, "ollama effect receipt is invalid"),
            Self::InvalidReceiptRecord => write!(f, "ollama effect receipt record is invalid"),
            Self::InvalidReplay => write!(f, "ollama effect receipt failed replay verification"),
            Self::BudgetExhausted => write!(f, "ollama retry budget exhausted"),
            Self::DuplicateRequest => write!(f, "ollama duplicate request identity rejected"),
        }
    }
}

impl std::error::Error for OllamaError {}

impl From<std::io::Error> for OllamaError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OllamaClient {
    config: OllamaConfig,
}

impl OllamaClient {
    pub fn new(config: OllamaConfig) -> Result<Self, OllamaError> {
        config.validate()?;
        Ok(Self { config })
    }

    pub fn from_env() -> Result<Self, OllamaError> {
        Self::new(OllamaConfig::from_env()?)
    }

    pub fn config(&self) -> &OllamaConfig {
        &self.config
    }

    pub fn request_json(&self, messages: &[OllamaMessage]) -> Result<String, OllamaError> {
        if messages.is_empty() {
            return Err(OllamaError::InvalidConfig("empty message list"));
        }

        let mut json = String::new();
        json.push_str("{\"model\":\"");
        json.push_str(&json_escape(&self.config.model));
        json.push_str("\",\"messages\":[");
        for (idx, message) in messages.iter().enumerate() {
            if idx != 0 {
                json.push(',');
            }
            if message.role.trim().is_empty() || message.content.trim().is_empty() {
                return Err(OllamaError::InvalidConfig("empty message"));
            }
            json.push_str("{\"role\":\"");
            json.push_str(&json_escape(&message.role));
            json.push_str("\",\"content\":\"");
            json.push_str(&json_escape(&message.content));
            json.push_str("\"}");
        }
        json.push_str("],\"stream\":false}");
        Ok(json)
    }

    pub fn request_hash(&self, messages: &[OllamaMessage]) -> Result<u64, OllamaError> {
        Ok(hash_text(&self.request_json(messages)?))
    }

    pub fn chat(&self, messages: &[OllamaMessage]) -> Result<OllamaChatResponse, OllamaError> {
        let body = self.request_json(messages)?;
        self.chat_body(&body)
    }

    pub fn chat_with_retry_budget(
        &self,
        messages: &[OllamaMessage],
        retry_policy: OllamaRetryBudgetPolicy,
    ) -> Result<(OllamaChatResponse, OllamaRetryBudgetDecision), OllamaError> {
        if retry_policy.timeout_ms != self.config.timeout_ms {
            return Err(OllamaError::InvalidConfig("retry timeout must match config"));
        }
        let request_hash = self.request_hash(messages)?;
        let mut last_error = None;
        for retry_count in 0..retry_policy.attempt_budget {
            let decision = retry_policy
                .decision_for_attempt(
                    ollama_provider_hash(),
                    self.config.base_url_id(),
                    self.config.model_id(),
                    request_hash,
                    retry_count,
                    false,
                )
                .ok_or(OllamaError::InvalidConfig("invalid retry budget"))?;
            if !decision.allowed {
                return Err(OllamaError::BudgetExhausted);
            }

            match self.chat(messages) {
                Ok(response) => return Ok((response, decision)),
                Err(err) if !decision.budget_exhausted => {
                    last_error = Some(err);
                }
                Err(err) => return Err(err),
            }
        }
        Err(last_error.unwrap_or(OllamaError::BudgetExhausted))
    }

    fn chat_body(&self, body: &str) -> Result<OllamaChatResponse, OllamaError> {
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
            return Err(OllamaError::HttpStatus(status));
        }
        parse_chat_response_body(body)
    }

    pub fn call_from_context(
        &self,
        context: &ContextRecord,
        policy: &PolicyStore,
    ) -> Result<OllamaLlmCall, OllamaError> {
        self.call_from_context_with_retry_policy(
            context,
            policy,
            OllamaRetryBudgetPolicy::from_config(&self.config),
        )
    }

    pub fn call_from_context_with_retry_policy(
        &self,
        context: &ContextRecord,
        policy: &PolicyStore,
        retry_policy: OllamaRetryBudgetPolicy,
    ) -> Result<OllamaLlmCall, OllamaError> {
        let messages = messages_from_context(context, policy);
        let request_hash = self.request_hash(&messages)?;
        let (response, retry_budget) = self.chat_with_retry_budget(&messages, retry_policy)?;
        self.call_from_response_with_budget(context, policy, request_hash, response, retry_budget)
    }

    pub fn record_from_context(
        &self,
        context: &ContextRecord,
        policy: &PolicyStore,
    ) -> Result<LlmRecord, OllamaError> {
        Ok(self.call_from_context(context, policy)?.record)
    }

    pub fn call_from_response_body(
        &self,
        context: &ContextRecord,
        policy: &PolicyStore,
        response_body: &str,
    ) -> Result<OllamaLlmCall, OllamaError> {
        let messages = messages_from_context(context, policy);
        let request_hash = self.request_hash(&messages)?;
        let response = parse_chat_response_body(response_body)?;
        let retry_budget = OllamaRetryBudgetPolicy::from_config(&self.config)
            .first_attempt(
                ollama_provider_hash(),
                self.config.base_url_id(),
                self.config.model_id(),
                request_hash,
            )
            .ok_or(OllamaError::InvalidConfig("invalid retry budget"))?;
        self.call_from_response_with_budget(context, policy, request_hash, response, retry_budget)
    }

    pub fn record_from_response_body(
        &self,
        context: &ContextRecord,
        policy: &PolicyStore,
        response_body: &str,
    ) -> Result<LlmRecord, OllamaError> {
        Ok(self.call_from_response_body(context, policy, response_body)?.record)
    }

    fn call_from_response_with_budget(
        &self,
        context: &ContextRecord,
        policy: &PolicyStore,
        request_hash: u64,
        response: OllamaChatResponse,
        retry_budget: OllamaRetryBudgetDecision,
    ) -> Result<OllamaLlmCall, OllamaError> {
        if !retry_budget.is_receiptable_success()
            || retry_budget.timeout_ms != self.config.timeout_ms
            || retry_budget.request_identity_hash
                != (OllamaRetryBudgetPolicy {
                    timeout_ms: retry_budget.timeout_ms,
                    max_retries: retry_budget.max_retries,
                    attempt_budget: retry_budget.attempt_budget,
                })
                .request_identity_hash(
                    ollama_provider_hash(),
                    self.config.base_url_id(),
                    self.config.model_id(),
                    request_hash,
                )
        {
            return Err(OllamaError::BudgetExhausted);
        }
        let record = LlmStructuredAdapter::record_from_external_response(
            context,
            policy,
            self.config.model_id(),
            response.response_hash,
            response.total_tokens.max(1),
        );
        let call = OllamaLlmCall::new(
            record,
            self.config.model_id(),
            self.config.base_url_id(),
            request_hash,
            response,
            retry_budget,
        );
        call.is_valid().then_some(call).ok_or(OllamaError::InvalidResponse)
    }
}

pub fn messages_from_context(context: &ContextRecord, policy: &PolicyStore) -> Vec<OllamaMessage> {
    vec![
        OllamaMessage::system(
            "You are the canon agent judgment capability. Return one concise judgment.",
        ),
        OllamaMessage::user(format!(
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct LocalEndpoint {
    host: String,
    port: u16,
    path_prefix: String,
}

fn parse_local_endpoint(base_url: &str) -> Result<LocalEndpoint, OllamaError> {
    let without_scheme = base_url
        .trim()
        .strip_prefix("http://")
        .ok_or(OllamaError::InvalidUrl)?;
    let (host_port, path) = without_scheme
        .split_once('/')
        .map_or((without_scheme, ""), |(host_port, path)| (host_port, path));
    let (host, port) = host_port
        .rsplit_once(':')
        .ok_or(OllamaError::InvalidUrl)?;
    if host != "127.0.0.1" && host != "localhost" {
        return Err(OllamaError::InvalidConfig("ollama base url must be local"));
    }
    let port = port.parse::<u16>().map_err(|_| OllamaError::InvalidUrl)?;
    if port == 0 {
        return Err(OllamaError::InvalidUrl);
    }
    let path_prefix = if path.trim().is_empty() {
        String::new()
    } else {
        format!("/{}", path.trim_matches('/'))
    };
    Ok(LocalEndpoint {
        host: host.to_string(),
        port,
        path_prefix,
    })
}

fn split_http_response(response: &str) -> Result<(u16, &str), OllamaError> {
    let (head, body) = response
        .split_once("\r\n\r\n")
        .ok_or(OllamaError::InvalidResponse)?;
    let status_line = head.lines().next().ok_or(OllamaError::InvalidResponse)?;
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|raw| raw.parse::<u16>().ok())
        .ok_or(OllamaError::InvalidResponse)?;
    Ok((status, body))
}

fn parse_chat_response_body(body: &str) -> Result<OllamaChatResponse, OllamaError> {
    let message_start = body.find("\"message\"").unwrap_or(0);
    let content = extract_json_string(body, "\"content\"", message_start)
        .ok_or(OllamaError::InvalidResponse)?;
    if content.trim().is_empty() {
        return Err(OllamaError::InvalidResponse);
    }
    let prompt_tokens = extract_json_u32(body, "\"prompt_tokens\"").unwrap_or(1);
    let completion_tokens = extract_json_u32(body, "\"completion_tokens\"").unwrap_or(1);
    let total_tokens = extract_json_u32(body, "\"total_tokens\"")
        .unwrap_or_else(|| prompt_tokens.saturating_add(completion_tokens).max(1));
    Ok(OllamaChatResponse {
        response_hash: hash_text(&content),
        raw_hash: hash_text(body),
        content,
        prompt_tokens,
        completion_tokens,
        total_tokens,
    })
}

pub fn append_ollama_llm_effect_receipt_ndjson(
    path: impl AsRef<Path>,
    receipt: &OllamaLlmEffectReceipt,
) -> Result<(), OllamaError> {
    if !receipt.is_valid() {
        return Err(OllamaError::InvalidReceipt);
    }
    append_ollama_ndjson_record(path, encode_ollama_llm_effect_receipt_ndjson(*receipt))
}

pub fn load_ollama_llm_effect_receipts_ndjson(
    path: impl AsRef<Path>,
) -> Result<Vec<OllamaLlmEffectReceipt>, OllamaError> {
    load_ollama_ndjson_records(
        path,
        OLLAMA_LLM_EFFECT_RECEIPT_RECORD,
        decode_ollama_llm_effect_receipt_fields,
    )
}

pub fn verify_ollama_llm_effect_receipts(
    tlog: &TLog,
    receipts: &[OllamaLlmEffectReceipt],
) -> Result<usize, OllamaError> {
    for receipt in receipts {
        if !receipt.replay_verified(tlog) {
            return Err(OllamaError::InvalidReplay);
        }
    }
    Ok(receipts.len())
}

pub fn verify_ollama_judgment_tlog_ndjson(path: impl AsRef<Path>) -> Result<usize, OllamaError> {
    let path = path.as_ref();
    let tlog = load_tlog_ndjson(path).map_err(|_| OllamaError::InvalidReplay)?;
    let receipts = load_ollama_llm_effect_receipts_ndjson_unchecked(path)?;
    if receipts.is_empty() {
        return Err(OllamaError::InvalidReplay);
    }
    verify_ollama_llm_effect_receipts(&tlog, &receipts)
}

pub fn append_ollama_judgment_proof_event_ndjson(
    path: impl AsRef<Path>,
    event: &OllamaJudgmentProofEvent,
) -> Result<(), OllamaError> {
    if !event.is_valid() {
        return Err(OllamaError::InvalidReceipt);
    }
    append_ollama_ndjson_record(path, encode_ollama_judgment_proof_event_ndjson(*event))
}

pub fn load_ollama_judgment_proof_events_ndjson(
    path: impl AsRef<Path>,
) -> Result<Vec<OllamaJudgmentProofEvent>, OllamaError> {
    load_ollama_ndjson_records(
        path,
        OLLAMA_JUDGMENT_PROOF_RECORD,
        decode_ollama_judgment_proof_event_fields,
    )
}

pub fn verify_ollama_judgment_proof_event_order_ndjson(
    path: impl AsRef<Path>,
) -> Result<usize, OllamaError> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(OllamaError::InvalidReplay);
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
            return Err(OllamaError::InvalidReplay);
        }

        match fields[1] {
            TLOG_RECORD_EVENT => {
                if fields.len() < 3 {
                    return Err(OllamaError::InvalidReplay);
                }
                last_control_event_seq = fields[2];
            }
            OLLAMA_LLM_EFFECT_RECEIPT_RECORD => {
                let receipt = decode_ollama_llm_effect_receipt_fields(&fields)?;
                seen_receipt_hashes.push(receipt.receipt_hash);
            }
            OLLAMA_JUDGMENT_PROOF_RECORD => {
                let event = decode_ollama_judgment_proof_event_fields(&fields)?;
                let actual_proof_event_seq = last_control_event_seq
                    .checked_add(1)
                    .filter(|seq| *seq != 0)
                    .ok_or(OllamaError::InvalidReplay)?;
                if event.proof_event_seq != actual_proof_event_seq
                    || !seen_receipt_hashes.contains(&event.receipt_hash)
                {
                    return Err(OllamaError::InvalidReplay);
                }
                verified += 1;
            }
            _ => {}
        }
    }

    if verified == 0 {
        return Err(OllamaError::InvalidReplay);
    }
    Ok(verified)
}

pub fn verify_ollama_judgment_proof_events(
    tlog: &TLog,
    receipts: &[OllamaLlmEffectReceipt],
    events: &[OllamaJudgmentProofEvent],
) -> Result<usize, OllamaError> {
    if events.is_empty() {
        return Err(OllamaError::InvalidReplay);
    }

    let mut proof_records = Vec::new();
    let mut proof_bindings = Vec::new();
    for event in events {
        let receipt = receipts
            .iter()
            .copied()
            .find(|receipt| event.matches_receipt(*receipt, tlog))
            .ok_or(OllamaError::InvalidReplay)?;
        let proof_record = event
            .to_verification_proof_record()
            .ok_or(OllamaError::InvalidReplay)?;
        let proof_binding = receipt
            .verification_proof_binding()
            .ok_or(OllamaError::InvalidReplay)?;
        proof_records.push(proof_record);
        proof_bindings.push(proof_binding);
    }

    verify_verification_proof_record_bindings(&proof_records, &proof_bindings)
        .map_err(|_| OllamaError::InvalidReplay)
}

pub fn verify_ollama_judgment_proof_events_ndjson(
    path: impl AsRef<Path>,
) -> Result<usize, OllamaError> {
    let path = path.as_ref();
    let tlog = load_tlog_ndjson(path).map_err(|_| OllamaError::InvalidReplay)?;
    let receipts = load_ollama_llm_effect_receipts_ndjson(path)?;
    let events = load_ollama_judgment_proof_events_ndjson(path)?;
    let verified = verify_ollama_judgment_proof_events(&tlog, &receipts, &events)?;
    if verify_ollama_judgment_proof_event_order_ndjson(path)? != verified {
        return Err(OllamaError::InvalidReplay);
    }
    Ok(verified)
}

pub fn encode_ollama_llm_effect_receipt_ndjson(receipt: OllamaLlmEffectReceipt) -> String {
    let fields = [
        OLLAMA_LLM_EFFECT_RECEIPT_SCHEMA_VERSION,
        OLLAMA_LLM_EFFECT_RECEIPT_RECORD,
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
    let body = fields
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{body}]")
}

pub fn encode_ollama_judgment_proof_event_ndjson(event: OllamaJudgmentProofEvent) -> String {
    let fields = [
        OLLAMA_JUDGMENT_PROOF_SCHEMA_VERSION,
        OLLAMA_JUDGMENT_PROOF_RECORD,
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
    let body = fields
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{body}]")
}

pub fn decode_ollama_llm_effect_receipt_ndjson(
    line: &str,
) -> Result<OllamaLlmEffectReceipt, OllamaError> {
    decode_ollama_llm_effect_receipt_fields(&parse_u64_fields(line)?)
}

fn decode_ollama_llm_effect_receipt_fields(
    fields: &[u64],
) -> Result<OllamaLlmEffectReceipt, OllamaError> {
    let receipt = decode_ollama_llm_effect_receipt_fields_unchecked(fields)?;
    receipt
        .is_valid()
        .then_some(receipt)
        .ok_or(OllamaError::InvalidReceipt)
}

fn decode_ollama_llm_effect_receipt_fields_unchecked(
    fields: &[u64],
) -> Result<OllamaLlmEffectReceipt, OllamaError> {
    if fields.len() != 25
        || fields[0] != OLLAMA_LLM_EFFECT_RECEIPT_SCHEMA_VERSION
        || fields[1] != OLLAMA_LLM_EFFECT_RECEIPT_RECORD
    {
        return Err(OllamaError::InvalidReceiptRecord);
    }

    let retry_count = u32::try_from(fields[7]).map_err(|_| OllamaError::InvalidReceiptRecord)?;
    let max_retries = u32::try_from(fields[8]).map_err(|_| OllamaError::InvalidReceiptRecord)?;
    let attempt_budget = u32::try_from(fields[9]).map_err(|_| OllamaError::InvalidReceiptRecord)?;
    let token_count = u32::try_from(fields[17]).map_err(|_| OllamaError::InvalidReceiptRecord)?;
    let receipt = OllamaLlmEffectReceipt {
        provider_hash: fields[2],
        base_url_hash: fields[3],
        model_id: fields[4],
        request_hash: fields[5],
        timeout_ms: fields[6],
        retry_count,
        max_retries,
        attempt_budget,
        request_identity_hash: fields[10],
        retry_budget_hash: fields[11],
        budget_exhausted: fields[12] == 1,
        duplicate_request: fields[13] == 1,
        response_hash: fields[14],
        raw_response_hash: fields[15],
        prompt_hash: fields[16],
        token_count,
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

pub fn decode_ollama_judgment_proof_event_ndjson(
    line: &str,
) -> Result<OllamaJudgmentProofEvent, OllamaError> {
    decode_ollama_judgment_proof_event_fields(&parse_u64_fields(line)?)
}

fn decode_ollama_judgment_proof_event_fields(
    fields: &[u64],
) -> Result<OllamaJudgmentProofEvent, OllamaError> {
    if fields.len() != 23
        || fields[0] != OLLAMA_JUDGMENT_PROOF_SCHEMA_VERSION
        || fields[1] != OLLAMA_JUDGMENT_PROOF_RECORD
    {
        return Err(OllamaError::InvalidReceiptRecord);
    }

    let event = OllamaJudgmentProofEvent {
        proof_line_hash: fields[2],
        receipt_core_hash: fields[3],
        receipt_hash: fields[4],
        receipt_event_seq: fields[5],
        proof_event_seq: fields[6],
        receipt_event_hash: fields[7],
        base_url_hash: fields[8],
        model_id: fields[9],
        timeout_ms: fields[10],
        retry_count: u32::try_from(fields[11]).map_err(|_| OllamaError::InvalidReceiptRecord)?,
        max_retries: u32::try_from(fields[12]).map_err(|_| OllamaError::InvalidReceiptRecord)?,
        attempt_budget: u32::try_from(fields[13]).map_err(|_| OllamaError::InvalidReceiptRecord)?,
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
        .ok_or(OllamaError::InvalidReceipt)
}

fn parse_u64_fields(line: &str) -> Result<Vec<u64>, OllamaError> {
    let body = line
        .trim()
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .ok_or(OllamaError::InvalidReceiptRecord)?;
    if body.trim().is_empty() {
        return Ok(Vec::new());
    }
    body.split(',')
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| OllamaError::InvalidReceiptRecord)
        })
        .collect()
}

fn load_ollama_llm_effect_receipts_ndjson_unchecked(
    path: impl AsRef<Path>,
) -> Result<Vec<OllamaLlmEffectReceipt>, OllamaError> {
    load_ollama_ndjson_records(
        path,
        OLLAMA_LLM_EFFECT_RECEIPT_RECORD,
        decode_ollama_llm_effect_receipt_fields_unchecked,
    )
}

fn append_ollama_ndjson_record(
    path: impl AsRef<Path>,
    encoded_record: String,
) -> Result<(), OllamaError> {
    let path = path.as_ref();
    ensure_ollama_record_parent(path)?;

    {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(file, "{}", encoded_record)?;
        file.sync_all()?;
    }

    sync_parent_dir(path)
}

fn load_ollama_ndjson_records<T, F>(
    path: impl AsRef<Path>,
    record_tag: u64,
    decode: F,
) -> Result<Vec<T>, OllamaError>
where
    F: Fn(&[u64]) -> Result<T, OllamaError>,
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

fn ensure_ollama_record_parent(path: &Path) -> Result<(), OllamaError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn sync_parent_dir(path: &Path) -> Result<(), OllamaError> {
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

fn extract_json_u32(body: &str, key: &str) -> Option<u32> {
    let idx = body.find(key)?;
    let after_key = &body[idx + key.len()..];
    let after_colon = after_key.split_once(':')?.1.trim_start();
    let digits: String = after_colon
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect();
    digits.parse::<u32>().ok()
}

fn extract_json_string(body: &str, key: &str, start_at: usize) -> Option<String> {
    let start = body.get(start_at..)?.find(key)? + start_at + key.len();
    let mut chars = body.get(start..)?.chars();
    for ch in chars.by_ref() {
        if ch == ':' {
            break;
        }
    }
    for ch in chars.by_ref() {
        if ch == '"' {
            break;
        }
        if !ch.is_whitespace() {
            return None;
        }
    }

    let mut out = String::new();
    let mut escaped = false;
    let mut unicode_remaining = 0u8;
    for ch in chars {
        if unicode_remaining != 0 {
            unicode_remaining -= 1;
            if unicode_remaining == 0 {
                out.push('?');
            }
            continue;
        }
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
                'u' => unicode_remaining = 4,
                _ => return None,
            }
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Some(out);
        } else {
            out.push(ch);
        }
    }
    None
}

fn json_escape(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        push_json_escaped_char(&mut out, ch);
    }
    out
}

fn push_json_escaped_char(out: &mut String, ch: char) {
    if let Some(escaped) = json_escape_sequence(ch) {
        out.push_str(escaped);
    } else if ch.is_control() {
        out.push(' ');
    } else {
        out.push(ch);
    }
}

fn json_escape_sequence(ch: char) -> Option<&'static str> {
    match ch {
        '"' => Some("\\\""),
        '\\' => Some("\\\\"),
        '\n' => Some("\\n"),
        '\r' => Some("\\r"),
        '\t' => Some("\\t"),
        _ => None,
    }
}

pub(crate) fn hash_text(value: &str) -> u64 {
    let mut h = 0x91f2_49bb_1f6d_7c35u64;
    for byte in value.as_bytes() {
        h = mix(h, u64::from(*byte));
    }
    h.max(1)
}

fn ollama_provider_hash() -> u64 {
    hash_text(OLLAMA_PROVIDER)
}

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
    let Some(policy) = OllamaRetryBudgetPolicy::new(timeout_ms, max_retries, attempt_budget) else {
        return false;
    };
    retry_count <= max_retries
        && retry_count < attempt_budget
        && retry_budget_hash == policy.policy_hash()
        && request_identity_hash
            == policy.request_identity_hash(provider_hash, base_url_hash, model_id, request_hash)
}