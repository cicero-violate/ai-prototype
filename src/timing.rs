#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhaseTimings {
    pub upload_ms: u64,
    pub turn_total_ms: u64,
    pub candidate_scan_ms: u64,
    pub delta_download_ms: u64,
    pub delta_apply_ms: u64,
    pub validation_ms: u64,
    pub loop_total_ms: u64,
}

impl PhaseTimings {
    pub fn phase_sum(self) -> u64 {
        self.upload_ms
            .saturating_add(self.turn_total_ms)
            .saturating_add(self.candidate_scan_ms)
            .saturating_add(self.delta_download_ms)
            .saturating_add(self.delta_apply_ms)
            .saturating_add(self.validation_ms)
    }

    pub fn canonicalized(mut self) -> Self {
        self.loop_total_ms = self.loop_total_ms.max(self.phase_sum());
        self
    }

    pub fn satisfies_loop_invariant(self) -> bool {
        self.loop_total_ms >= self.phase_sum()
    }
}
