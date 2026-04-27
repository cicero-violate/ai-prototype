//! Runtime-owned reconstruction of API command receipts from canonical events.
//!
//! The ledger is derived from the TLog and therefore belongs below the API
//! surface. API envelopes adapt into `(command_id, command_hash)` pairs at the
//! boundary; runtime never imports API protocol types.

use crate::error::CanonError;
use crate::kernel::{ControlEvent, TLog};

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

            ledger.insert_or_update(receipt);
        }

        Ok(ledger)
    }

    pub fn receipt_for_ids(
        &self,
        command_id: u64,
        command_hash: u64,
    ) -> Option<CommandReceipt> {
        self.receipts
            .iter()
            .copied()
            .find(|receipt| {
                receipt.command_id == command_id && receipt.command_hash == command_hash
            })
    }

    pub fn has_conflicting_command_ids(&self, command_id: u64, command_hash: u64) -> bool {
        self.receipts.iter().any(|receipt| {
            receipt.command_id == command_id && receipt.command_hash != command_hash
        })
    }

    pub fn replayed_event_by_ids(
        &self,
        command_id: u64,
        command_hash: u64,
        tlog: &TLog,
    ) -> Option<ControlEvent> {
        let receipt = self.receipt_for_ids(command_id, command_hash)?;
        tlog.iter()
            .copied()
            .find(|event| event.self_hash == receipt.event_hash)
    }

    pub fn push_receipt(
        &mut self,
        command_id: u64,
        command_hash: u64,
        event: &ControlEvent,
    ) -> CommandReceipt {
        let receipt = CommandReceipt {
            command_id,
            command_hash,
            event_hash: event.self_hash,
        };
        self.insert_or_update(receipt);
        receipt
    }

    fn insert_or_update(&mut self, receipt: CommandReceipt) {
        if let Some(existing) = self.receipts.iter_mut().find(|existing| {
            existing.command_id == receipt.command_id
                && existing.command_hash == receipt.command_hash
        }) {
            *existing = receipt;
        } else {
            self.receipts.push(receipt);
        }
    }
}