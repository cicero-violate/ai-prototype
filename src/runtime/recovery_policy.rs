#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    Continue,
    Repair,
    Escalate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryPolicy {
    max_attempts: u8,
}

impl RecoveryPolicy {
    pub const fn bounded(max_attempts: u8) -> Self {
        Self { max_attempts }
    }

    pub const fn classify(&self, attempts: u8, has_executable_surface: bool) -> RecoveryAction {
        if attempts == 0 {
            return RecoveryAction::Continue;
        }
        if !has_executable_surface || attempts > self.max_attempts {
            return RecoveryAction::Escalate;
        }
        RecoveryAction::Repair
    }
}