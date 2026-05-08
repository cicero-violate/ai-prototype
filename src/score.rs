#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScoreVector {
    pub intelligence: f64,
    pub efficiency: f64,
    pub correctness: f64,
    pub alignment: f64,
    pub robustness: f64,
    pub performance: f64,
    pub scalability: f64,
    pub determinism: f64,
    pub transparency: f64,
    pub collaboration: f64,
    pub empowerment: f64,
    pub benefit: f64,
    pub learning: f64,
    pub structure: f64,
    pub simplicity: f64,
    pub future_proofing: f64,
}

impl ScoreVector {
    pub fn values(self) -> [f64; 16] {
        [
            self.intelligence,
            self.efficiency,
            self.correctness,
            self.alignment,
            self.robustness,
            self.performance,
            self.scalability,
            self.determinism,
            self.transparency,
            self.collaboration,
            self.empowerment,
            self.benefit,
            self.learning,
            self.structure,
            self.simplicity,
            self.future_proofing,
        ]
    }

    pub fn geometric_mean(self) -> Option<f64> {
        geometric_mean(&self.values())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScoreDelta {
    pub before: f64,
    pub after: f64,
}

impl ScoreDelta {
    pub fn is_real_gain(self) -> bool {
        self.after.is_finite() && self.before.is_finite() && self.after > self.before
    }
}

pub fn geometric_mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() || values.iter().any(|v| !v.is_finite() || *v < 0.0) {
        return None;
    }

    if values.iter().any(|v| *v == 0.0) {
        return Some(0.0);
    }

    let log_sum: f64 = values.iter().map(|v| v.ln()).sum();
    Some((log_sum / values.len() as f64).exp())
}
