//! External command/evidence protocol.

use crate::capability::EvidenceSubmission;
use crate::kernel::{ControlEvent, TLog};
use crate::runtime::CanonError;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandReceipt {
    pub command_id: u64,
    pub command_hash: u64,
    pub event_hash: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommandLedger {
    receipts: Vec<CommandReceipt>,
}

impl CommandLedger {
    pub fn len(&self) -> usize {
        self.receipts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.receipts.is_empty()
    }

    pub fn receipts(&self) -> &[CommandReceipt] {
        &self.receipts
    }

    pub fn reconstruct_from_tlog(tlog: &TLog) -> Result<Self, CanonError> {
        let mut ledger = Self::default();

        for event in tlog {
            if event.api_command_id == 0 && event.api_command_hash == 0 {
                continue;
            }

            if event.api_command_id == 0 || event.api_command_hash == 0 {
                return Err(CanonError::InvalidApiCommand);
            }

            let receipt = CommandReceipt {
                command_id: event.api_command_id,
                command_hash: event.api_command_hash,
                event_hash: event.self_hash,
            };

            if ledger.receipts.iter().any(|existing| {
                existing.command_id == receipt.command_id
                    && existing.command_hash != receipt.command_hash
            }) {
                return Err(CanonError::InvalidApiCommand);
            }

            if ledger.receipts.iter().any(|existing| {
                existing.command_id == receipt.command_id
                    && existing.command_hash == receipt.command_hash
                    && existing.event_hash != receipt.event_hash
            }) {
                return Err(CanonError::InvalidReplay);
            }

            if !ledger.receipts.iter().any(|existing| *existing == receipt) {
                ledger.receipts.push(receipt);
            }
        }

        Ok(ledger)
    }

    pub fn receipt_for(&self, envelope: &CommandEnvelope) -> Option<CommandReceipt> {
        self.receipts
            .iter()
            .copied()
            .find(|receipt| {
                receipt.command_id == envelope.command_id
                    && receipt.command_hash == envelope.command_hash
            })
    }

    pub fn has_conflicting_command(&self, envelope: &CommandEnvelope) -> bool {
        self.receipts.iter().any(|receipt| {
            receipt.command_id == envelope.command_id
                && receipt.command_hash != envelope.command_hash
        })
    }

    pub fn replayed_event(&self, envelope: &CommandEnvelope, tlog: &TLog) -> Option<ControlEvent> {
        let receipt = self.receipt_for(envelope)?;
        tlog.iter()
            .copied()
            .find(|event| event.self_hash == receipt.event_hash)
    }

    pub fn push_response(
        &mut self,
        envelope: &CommandEnvelope,
        event: &ControlEvent,
    ) -> CommandReceipt {
        let receipt = CommandReceipt {
            command_id: envelope.command_id,
            command_hash: envelope.command_hash,
            event_hash: event.self_hash,
        };
        self.receipts.push(receipt);
        receipt
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
