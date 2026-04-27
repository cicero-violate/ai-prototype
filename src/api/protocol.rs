//! External command/evidence protocol.

use crate::capability::EvidenceSubmission;
use crate::kernel::{ControlEvent, TLog};
pub use crate::runtime::{CommandLedger, CommandReceipt};

pub const API_PROTOCOL_SCHEMA_VERSION: u64 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    SubmitEvidence(EvidenceSubmission),
    SubmitEvidenceBatch(Vec<EvidenceSubmission>),
}

impl Command {
    pub fn is_contract_valid(&self) -> bool {
        match self {
            Self::SubmitEvidence(submission) => submission.is_contract_valid(),
            Self::SubmitEvidenceBatch(submissions) => {
                !submissions.is_empty()
                    && submissions.iter().copied().all(EvidenceSubmission::is_contract_valid)
                    && gates_are_unique(submissions)
            }
        }
    }

    pub fn submission_count(&self) -> usize {
        match self {
            Self::SubmitEvidence(_) => 1,
            Self::SubmitEvidenceBatch(submissions) => submissions.len(),
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
        }
    }
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
    ) -> CommandReceipt {
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

fn envelope_hash(schema_version: u64, command_id: u64, command: &Command) -> u64 {
    let mut h = 0x517cc1b727220a95u64;
    h ^= schema_version;
    h = h.wrapping_mul(0x100000001b3);
    h ^= command_id;
    h = h.wrapping_mul(0x100000001b3);
    h ^= command.contract_hash();
    h.wrapping_mul(0x100000001b3).max(1)
}
