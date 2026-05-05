use super::Decision;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreDelta {
    pub correctness: u8,
    pub efficiency: u8,
    pub robustness: u8,
}

impl ScoreDelta {
    pub const fn new(correctness: u8, efficiency: u8, robustness: u8) -> Self {
        Self { correctness, efficiency, robustness }
    }

    pub const fn bounded_min(&self) -> u8 {
        min3(self.correctness, self.efficiency, self.robustness)
    }

    pub const fn decision(&self, threshold: u8) -> Decision {
        if self.bounded_min() >= threshold {
            Decision::Allow
        } else {
            Decision::Repair
        }
    }
}

const fn min3(a: u8, b: u8, c: u8) -> u8 {
    let ab = if a < b { a } else { b };
    if ab < c { ab } else { c }
}