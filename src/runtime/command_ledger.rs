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

impl CommandReceipt {
    pub fn matches_ids(self, command_id: u64, command_hash: u64) -> bool {
        self.command_id == command_id && self.command_hash == command_hash
    }

    pub fn conflicts_with_ids(self, command_id: u64, command_hash: u64) -> bool {
        self.command_id == command_id && self.command_hash != command_hash
    }
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
            ledger.observe_event(event)?;
        }

        Ok(ledger)
    }

    pub fn observe_event(&mut self, event: &ControlEvent) -> Result<(), CanonError> {
        if event.api_command_id == 0 && event.api_command_hash == 0 {
            return Ok(());
        }
        if event.api_command_id == 0 || event.api_command_hash == 0 || event.self_hash == 0 {
            return Err(CanonError::InvalidApiCommand);
        }

        self.insert_or_update_checked(CommandReceipt {
            command_id: event.api_command_id,
            command_hash: event.api_command_hash,
            event_hash: event.self_hash,
        })
    }

    pub fn receipt_for_ids(
        &self,
        command_id: u64,
        command_hash: u64,
    ) -> Option<CommandReceipt> {
        self.receipts
            .iter()
            .copied()
            .find(|receipt| receipt.matches_ids(command_id, command_hash))
    }

    pub fn has_conflicting_command_ids(&self, command_id: u64, command_hash: u64) -> bool {
        self.receipts
            .iter()
            .any(|receipt| receipt.conflicts_with_ids(command_id, command_hash))
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
    ) -> Result<CommandReceipt, CanonError> {
        if command_id == 0 || command_hash == 0 || event.self_hash == 0 {
            return Err(CanonError::InvalidApiCommand);
        }
        if event.api_command_id != command_id || event.api_command_hash != command_hash {
            return Err(CanonError::InvalidReplay);
        }
        let receipt = CommandReceipt {
            command_id,
            command_hash,
            event_hash: event.self_hash,
        };
        self.insert_or_update_checked(receipt)?;
        Ok(receipt)
    }

    fn insert_or_update_checked(&mut self, receipt: CommandReceipt) -> Result<(), CanonError> {
        if self.has_conflicting_command_ids(receipt.command_id, receipt.command_hash) {
            return Err(CanonError::InvalidApiCommand);
        }

        self.insert_or_update(receipt);
        Ok(())
    }

    fn insert_or_update(&mut self, receipt: CommandReceipt) {
        if let Some(existing) = self.receipts.iter_mut().find(|existing| {
            existing.matches_ids(receipt.command_id, receipt.command_hash)
        }) {
            *existing = receipt;
        } else {
            self.receipts.push(receipt);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::{RuntimeConfig, State};

    #[test]
    fn observing_command_event_updates_ledger_without_replay_scan() {
        let mut state = State::default();
        let mut tlog = Vec::new();

        crate::runtime::tick_with_api_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            7,
            11,
        )
        .unwrap();

        let event = tlog[0];
        let mut ledger = CommandLedger::default();
        ledger.observe_event(&event).unwrap();

        assert_eq!(ledger.len(), 1);
        assert_eq!(
            ledger.receipt_for_ids(7, 11),
            Some(CommandReceipt {
                command_id: 7,
                command_hash: 11,
                event_hash: event.self_hash,
            })
        );
    }
}