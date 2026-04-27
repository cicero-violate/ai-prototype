//! Deterministic structured-record adapter for LLM output.

use crate::capability::context::ContextRecord;
use crate::capability::judgment::JudgmentRecord;
use crate::capability::policy::PolicyStore;
use crate::capability::{EvidenceProducer, EvidenceSubmission};
use crate::kernel::{mix, Evidence, GateId};

const LLM_JUDGMENT_SCHEMA_VERSION: u64 = 1;
const LLM_JUDGMENT_SCHEMA_HASH: u64 = 0x4f1bbcdd2f5d7a91;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LlmDecision {
    Structured,
    Refused,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LlmPromptRecord {
    pub schema_version: u64,
    pub context_hash: u64,
    pub policy_version: u64,
    pub policy_hash: u64,
    pub prompt_hash: u64,
}

impl LlmPromptRecord {
    pub fn is_valid(self) -> bool {
        self.schema_version == LLM_JUDGMENT_SCHEMA_VERSION
            && self.context_hash != 0
            && self.policy_version != 0
            && self.policy_hash != 0
            && self.prompt_hash != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LlmResponseRecord {
    pub model_id: u64,
    pub prompt_hash: u64,
    pub response_hash: u64,
    pub token_count: u32,
    pub schema_hash: u64,
}

impl LlmResponseRecord {
    pub fn is_valid(self) -> bool {
        self.model_id != 0
            && self.prompt_hash != 0
            && self.response_hash != 0
            && self.token_count != 0
            && self.schema_hash == LLM_JUDGMENT_SCHEMA_HASH
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LlmRecord {
    pub prompt: LlmPromptRecord,
    pub response: LlmResponseRecord,
    pub judgment: JudgmentRecord,
}

impl LlmRecord {
    pub fn from_context_policy(
        context: &ContextRecord,
        policy: &PolicyStore,
        model_id: u64,
    ) -> Self {
        let policy_version = policy.latest_version().max(1);
        let policy_hash = policy.fingerprint();
        let prompt_hash = if context.is_valid() {
            prompt_hash(context, policy_version, policy_hash)
        } else {
            0
        };
        let response_hash = response_hash(context, model_id, prompt_hash, policy_hash);
        let rationale_hash = rationale_hash(context, response_hash, policy_version, policy_hash);

        Self {
            prompt: LlmPromptRecord {
                schema_version: LLM_JUDGMENT_SCHEMA_VERSION,
                context_hash: context.context_hash,
                policy_version,
                policy_hash,
                prompt_hash,
            },
            response: LlmResponseRecord {
                model_id,
                prompt_hash,
                response_hash,
                token_count: token_count(context),
                schema_hash: LLM_JUDGMENT_SCHEMA_HASH,
            },
            judgment: JudgmentRecord {
                decision_id: response_hash,
                policy_version,
                rationale_hash,
            },
        }
    }

    pub fn decision(&self) -> LlmDecision {
        if self.is_valid() {
            LlmDecision::Structured
        } else {
            LlmDecision::Refused
        }
    }

    pub fn is_valid(&self) -> bool {
        self.prompt.is_valid()
            && self.response.is_valid()
            && self.response.prompt_hash == self.prompt.prompt_hash
            && self.judgment.is_valid()
            && self.judgment.policy_version == self.prompt.policy_version
            && self.judgment.decision_id == self.response.response_hash
    }

    pub fn judgment_record(&self) -> JudgmentRecord {
        self.judgment.clone()
    }

    pub fn submission(&self) -> EvidenceSubmission {
        EvidenceSubmission::with_payload(
            GateId::Judgment,
            Evidence::JudgmentRecord,
            self.decision() == LlmDecision::Structured,
            llm_payload_hash(self),
        )
    }
}

impl EvidenceProducer for LlmRecord {
    type Record = LlmRecord;

    fn record(&self) -> &Self::Record {
        self
    }

    fn submission(&self) -> EvidenceSubmission {
        LlmRecord::submission(self)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LlmStructuredAdapter;

impl LlmStructuredAdapter {
    pub fn record_from_context(
        context: &ContextRecord,
        policy: &PolicyStore,
        model_id: u64,
    ) -> LlmRecord {
        LlmRecord::from_context_policy(context, policy, model_id)
    }
}

fn prompt_hash(context: &ContextRecord, policy_version: u64, policy_hash: u64) -> u64 {
    let mut h = 0x106689d45497fdb5u64;
    h = mix(h, context.objective_id);
    h = mix(h, context.observation_hash);
    h = mix(h, context.memory_aggregate_hash);
    h = mix(h, context.context_hash);
    h = mix(h, context.prior_count as u64);
    h = mix(h, policy_version);
    h = mix(h, policy_hash);
    h = mix(h, LLM_JUDGMENT_SCHEMA_VERSION);
    h.max(1)
}

fn response_hash(
    context: &ContextRecord,
    model_id: u64,
    prompt_hash: u64,
    policy_hash: u64,
) -> u64 {
    let mut h = 0x6c62272e07bb0142u64;
    h = mix(h, model_id);
    h = mix(h, prompt_hash);
    h = mix(h, policy_hash);
    h = mix(h, context.context_hash);
    h = mix(h, context.memory_aggregate_hash);
    h = mix(h, LLM_JUDGMENT_SCHEMA_HASH);
    h.max(1)
}

fn rationale_hash(
    context: &ContextRecord,
    response_hash: u64,
    policy_version: u64,
    policy_hash: u64,
) -> u64 {
    let mut h = 0x80a3dc05d5f2d4f7u64;
    h = mix(h, context.objective_id);
    h = mix(h, context.context_hash);
    h = mix(h, response_hash);
    h = mix(h, policy_version);
    h = mix(h, policy_hash);
    h.max(1)
}

fn llm_payload_hash(record: &LlmRecord) -> u64 {
    let mut h = 0xd6e8_feb8_6659_fd93u64;
    h = mix(h, record.prompt.context_hash);
    h = mix(h, record.prompt.policy_version);
    h = mix(h, record.prompt.policy_hash);
    h = mix(h, record.prompt.prompt_hash);
    h = mix(h, record.response.model_id);
    h = mix(h, record.response.response_hash);
    h = mix(h, record.response.token_count as u64);
    h = mix(h, record.judgment.decision_id);
    h = mix(h, record.judgment.rationale_hash);
    h.max(1)
}

fn token_count(context: &ContextRecord) -> u32 {
    u32::from(context.prior_count)
        .saturating_add(4)
        .saturating_add((context.context_hash.count_ones() % 32).max(1))
}

