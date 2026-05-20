//! External command/evidence protocol.

use crate::capability::observation::ObservationIngressBatch;
use crate::capability::orchestration::{AgentCycleEvent, ChildCompleteRecord, WaveRecord};
use crate::capability::tooling::{
    McpCallReceipt, McpCallRequest, SandboxProcessReceipt, SandboxProcessRequest,
};
use crate::capability::{CapabilityRegistry, EvidenceSubmission};
use crate::kernel::{ControlEvent, Evidence, GateId, TLog};
pub use crate::runtime::{
    CommandLedger, CommandReceipt, MailboxMessageReceipt, MailboxMessageRequest,
};

pub const API_PROTOCOL_SCHEMA_VERSION: u64 = 6;
pub const API_COMMAND_BATCH_LIMIT: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    SubmitEvidence(EvidenceSubmission),
    SubmitEvidenceBatch(Vec<EvidenceSubmission>),
    SubmitObservationIngress(ObservationIngressBatch),
    AuthorizeMcpCall(McpCallRequest),
    AuthorizeProcessCall(SandboxProcessRequest),
    AuthorizeMailboxMessage(MailboxMessageRequest),
    SubmitMcpCallReceipt(McpCallReceipt),
    SubmitProcessReceipt(SandboxProcessReceipt),
    SubmitMailboxMessageReceipt(MailboxMessageReceipt),
    SubmitProcessReceiptBatch(Vec<SandboxProcessReceipt>),
    SubmitAgentCycleEvent(AgentCycleEvent),
    SubmitWaveDispatch(WaveRecord),
    SubmitChildComplete(ChildCompleteRecord),
}

impl Command {
    pub fn is_contract_valid(&self) -> bool {
        match self {
            Self::SubmitEvidence(submission) => submission.is_contract_valid(),
            Self::SubmitEvidenceBatch(submissions) => {
                !submissions.is_empty()
                    && submissions.len() <= API_COMMAND_BATCH_LIMIT
                    && submissions
                        .iter()
                        .copied()
                        .all(EvidenceSubmission::is_contract_valid)
                    && gates_are_unique(submissions)
            }
            Self::SubmitObservationIngress(batch) => {
                batch.records.len() <= API_COMMAND_BATCH_LIMIT
                    && batch.is_contract_valid()
                    && batch.submission().is_contract_valid()
            }
            Self::AuthorizeMcpCall(request) => request.is_admissible(),
            Self::AuthorizeProcessCall(request) => {
                request.is_admissible()
                    && request.registry_policy_hash == CapabilityRegistry::canonical().policy_hash()
            }
            Self::AuthorizeMailboxMessage(request) => {
                request.is_admissible()
                    && request.registry_policy_hash == CapabilityRegistry::canonical().policy_hash()
            }
            Self::SubmitMcpCallReceipt(receipt) => {
                receipt.is_contract_valid()
                    && receipt.registry_policy_hash == CapabilityRegistry::canonical().policy_hash()
            }
            Self::SubmitProcessReceipt(receipt) => {
                receipt.is_contract_valid()
                    && receipt.registry_policy_hash == CapabilityRegistry::canonical().policy_hash()
            }
            Self::SubmitMailboxMessageReceipt(receipt) => {
                receipt.is_contract_valid()
                    && receipt.registry_policy_hash == CapabilityRegistry::canonical().policy_hash()
            }
            Self::SubmitProcessReceiptBatch(receipts) => {
                !receipts.is_empty()
                    && receipts.len() <= API_COMMAND_BATCH_LIMIT
                    && receipts.iter().all(|receipt| {
                        receipt.is_contract_valid()
                            && receipt.registry_policy_hash
                                == CapabilityRegistry::canonical().policy_hash()
                    })
                    && process_receipt_hashes_are_unique(receipts)
            }
            Self::SubmitAgentCycleEvent(event) => event.is_contract_valid(),
            Self::SubmitWaveDispatch(record) => record.is_contract_valid(),
            Self::SubmitChildComplete(record) => record.is_contract_valid(),
        }
    }

    pub fn submission_count(&self) -> usize {
        match self {
            Self::SubmitEvidence(_) => 1,
            Self::SubmitEvidenceBatch(submissions) => submissions.len(),
            Self::SubmitObservationIngress(batch) => batch.records.len(),
            Self::AuthorizeMcpCall(_) => 1,
            Self::AuthorizeProcessCall(_) => 1,
            Self::AuthorizeMailboxMessage(_) => 1,
            Self::SubmitMcpCallReceipt(_) => 1,
            Self::SubmitProcessReceipt(_) => 1,
            Self::SubmitMailboxMessageReceipt(_) => 1,
            Self::SubmitProcessReceiptBatch(receipts) => receipts.len(),
            Self::SubmitAgentCycleEvent(_) => 1,
            Self::SubmitWaveDispatch(_) => 1,
            Self::SubmitChildComplete(_) => 1,
        }
    }

    pub fn contract_hash(&self) -> u64 {
        match self {
            Self::SubmitEvidence(submission) => {
                let mut h = 0x9e3779b97f4a7c15u64;
                h ^= self.submission_count() as u64;
                h = h.wrapping_mul(0x100000001b3);
                h ^= submission.contract_hash();
                h.wrapping_mul(0x100000001b3).max(1)
            }
            Self::SubmitEvidenceBatch(submissions) => {
                let mut h = 0x94d049bb133111ebu64;
                h ^= self.submission_count() as u64;
                h = h.wrapping_mul(0x100000001b3);
                for submission in submissions {
                    h ^= submission.contract_hash();
                    h = h.wrapping_mul(0x100000001b3);
                }
                h.max(1)
            }
            Self::SubmitObservationIngress(batch) => {
                let mut h = 0x6eed_0e51_0b5e_0001u64;
                h ^= self.submission_count() as u64;
                h = h.wrapping_mul(0x100000001b3);
                h ^= batch.contract_hash();
                h = h.wrapping_mul(0x100000001b3);
                h ^= batch.submission().contract_hash();
                h.wrapping_mul(0x100000001b3).max(1)
            }
            Self::AuthorizeMcpCall(request) => {
                let mut h = 0x4d43_5041_5554_4801u64;
                h ^= self.submission_count() as u64;
                h = h.wrapping_mul(0x100000001b3);
                h ^= request.contract_hash();
                h = h.wrapping_mul(0x100000001b3);
                h ^= mcp_authorization_submission(*request).contract_hash();
                h.wrapping_mul(0x100000001b3).max(1)
            }
            Self::AuthorizeProcessCall(request) => {
                let mut h = 0x5052_4f43_4155_5401u64;
                h ^= self.submission_count() as u64;
                h = h.wrapping_mul(0x100000001b3);
                h ^= request.contract_hash();
                h = h.wrapping_mul(0x100000001b3);
                h ^= process_authorization_submission(*request).contract_hash();
                h.wrapping_mul(0x100000001b3).max(1)
            }
            Self::AuthorizeMailboxMessage(request) => {
                let mut h = 0x4d42_5841_5554_4801u64;
                h ^= self.submission_count() as u64;
                h = h.wrapping_mul(0x100000001b3);
                h ^= request.contract_hash();
                h = h.wrapping_mul(0x100000001b3);
                h ^= mailbox_authorization_submission(*request).contract_hash();
                h.wrapping_mul(0x100000001b3).max(1)
            }
            Self::SubmitMcpCallReceipt(receipt) => {
                let mut h = 0x4d43_5052_4350_5401u64;
                h ^= self.submission_count() as u64;
                h = h.wrapping_mul(0x100000001b3);
                h ^= receipt.contract_hash();
                h = h.wrapping_mul(0x100000001b3);
                h ^= receipt.submission().contract_hash();
                h.wrapping_mul(0x100000001b3).max(1)
            }
            Self::SubmitProcessReceipt(receipt) => {
                let mut h = 0x0d6e_8feb_8665_9fd9u64;
                h ^= self.submission_count() as u64;
                h = h.wrapping_mul(0x100000001b3);
                h ^= receipt.contract_hash();
                h = h.wrapping_mul(0x100000001b3);
                h ^= receipt.effect.contract_hash();
                h.wrapping_mul(0x100000001b3).max(1)
            }
            Self::SubmitMailboxMessageReceipt(receipt) => {
                let mut h = 0x4d42_5852_4350_5401u64;
                h ^= self.submission_count() as u64;
                h = h.wrapping_mul(0x100000001b3);
                h ^= receipt.contract_hash();
                h = h.wrapping_mul(0x100000001b3);
                h ^= receipt.submission().contract_hash();
                h.wrapping_mul(0x100000001b3).max(1)
            }
            Self::SubmitProcessReceiptBatch(receipts) => {
                let mut h = 0x9a9b_5ee1_5e7c_0005u64;
                h ^= self.submission_count() as u64;
                h = h.wrapping_mul(0x100000001b3);
                for receipt in receipts {
                    h ^= receipt.contract_hash();
                    h = h.wrapping_mul(0x100000001b3);
                    h ^= receipt.effect.contract_hash();
                    h = h.wrapping_mul(0x100000001b3);
                }
                h.max(1)
            }
            Self::SubmitAgentCycleEvent(event) => {
                let mut h = 0xc4de_5a7e_3b0f_1906u64;
                h ^= self.submission_count() as u64;
                h = h.wrapping_mul(0x100000001b3);
                h ^= event.contract_hash();
                h.wrapping_mul(0x100000001b3).max(1)
            }
            Self::SubmitWaveDispatch(record) => {
                let mut h = 0xd4a7_3b2e_0091_0001u64;
                h ^= self.submission_count() as u64;
                h = h.wrapping_mul(0x100000001b3);
                h ^= record.contract_hash();
                h.wrapping_mul(0x100000001b3).max(1)
            }
            Self::SubmitChildComplete(record) => {
                let mut h = 0xc3f1_9a4d_0092_0001u64;
                h ^= self.submission_count() as u64;
                h = h.wrapping_mul(0x100000001b3);
                h ^= record.contract_hash();
                h.wrapping_mul(0x100000001b3).max(1)
            }
        }
    }
}

pub fn mcp_authorization_submission(request: McpCallRequest) -> EvidenceSubmission {
    EvidenceSubmission::with_payload(
        GateId::Plan,
        Evidence::TaskReady,
        false,
        request.contract_hash(),
    )
}

pub fn process_authorization_submission(request: SandboxProcessRequest) -> EvidenceSubmission {
    EvidenceSubmission::with_payload(
        GateId::Plan,
        Evidence::TaskReady,
        false,
        request.contract_hash(),
    )
}

pub fn mailbox_authorization_submission(request: MailboxMessageRequest) -> EvidenceSubmission {
    request.submission()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandEnvelope {
    pub schema_version: u64,
    pub command_id: u64,
    pub command_hash: u64,
    pub command: Command,
}

impl CommandEnvelope {
    pub fn new(command_id: u64, command: Command) -> Self {
        let command_hash = envelope_hash(API_PROTOCOL_SCHEMA_VERSION, command_id, &command);
        Self {
            schema_version: API_PROTOCOL_SCHEMA_VERSION,
            command_id,
            command_hash,
            command,
        }
    }

    pub fn is_contract_valid(&self) -> bool {
        self.schema_version == API_PROTOCOL_SCHEMA_VERSION
            && self.command_id != 0
            && self.command.is_contract_valid()
            && self.command_hash
                == envelope_hash(self.schema_version, self.command_id, &self.command)
    }

    pub fn into_command(self) -> Command {
        self.command
    }
}

impl CommandLedger {
    pub fn receipt_for(&self, envelope: &CommandEnvelope) -> Option<CommandReceipt> {
        self.receipt_for_ids(envelope.command_id, envelope.command_hash)
    }

    pub fn has_conflicting_command(&self, envelope: &CommandEnvelope) -> bool {
        self.has_conflicting_command_ids(envelope.command_id, envelope.command_hash)
    }

    pub fn replayed_event(&self, envelope: &CommandEnvelope, tlog: &TLog) -> Option<ControlEvent> {
        self.replayed_event_by_ids(envelope.command_id, envelope.command_hash, tlog)
    }

    pub fn push_response(
        &mut self,
        envelope: &CommandEnvelope,
        event: &ControlEvent,
    ) -> Result<CommandReceipt, crate::kernel::CanonError> {
        self.push_receipt(envelope.command_id, envelope.command_hash, event)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlEventResponse {
    pub event: ControlEvent,
}

fn gates_are_unique(submissions: &[EvidenceSubmission]) -> bool {
    let mut seen = 0u16;
    for submission in submissions {
        let bit = 1u16 << (submission.gate as u8);
        if seen & bit != 0 {
            return false;
        }
        seen |= bit;
    }
    true
}

fn process_receipt_hashes_are_unique(receipts: &[SandboxProcessReceipt]) -> bool {
    for (idx, receipt) in receipts.iter().enumerate() {
        if receipts[..idx]
            .iter()
            .any(|prior| prior.receipt_hash == receipt.receipt_hash)
        {
            return false;
        }
    }
    true
}

fn envelope_hash(schema_version: u64, command_id: u64, command: &Command) -> u64 {
    let mut h = 0x517cc1b727220a95u64;
    h ^= schema_version;
    h = h.wrapping_mul(0x100000001b3);
    h ^= command_id;
    h = h.wrapping_mul(0x100000001b3);
    h ^= command.contract_hash();
    h.wrapping_mul(0x100000001b3).max(1)
}
