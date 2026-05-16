#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RuntimeConfig {
    pub max_steps: u64,
    pub max_recovery_attempts: u8,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_steps: 96,
            max_recovery_attempts: 8,
        }
    }
}

impl RuntimeConfig {
    pub fn is_structurally_valid(self) -> bool {
        self.max_steps != 0 && self.max_recovery_attempts != 0
    }
}
