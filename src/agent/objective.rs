//! Agent objective record.

fn hash_str(s: &str) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for byte in s.as_bytes() {
        h ^= *byte as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h.max(1)
}

#[derive(Clone, Debug)]
pub struct AgentObjective {
    pub objective_id: u64,
    pub objective_hash: u64,
    pub domain_hint: String,
    pub success_metric: String,
    pub success_metric_hash: u64,
    pub risk_envelope_hash: u64,
    pub stop_condition_hash: u64,
}

impl AgentObjective {
    pub fn new(domain_hint: impl Into<String>, success_metric: impl Into<String>) -> Self {
        let domain_hint = domain_hint.into();
        let success_metric = success_metric.into();
        let objective_hash = hash_str(&domain_hint);
        let success_metric_hash = hash_str(&success_metric);
        let combined = format!("{objective_hash}:{success_metric_hash}");
        let objective_id = hash_str(&combined);
        Self {
            objective_id,
            objective_hash,
            domain_hint,
            success_metric,
            success_metric_hash,
            risk_envelope_hash: 0,
            stop_condition_hash: 0,
        }
    }

    pub fn with_risk_envelope(mut self, risk: impl Into<String>) -> Self {
        self.risk_envelope_hash = hash_str(&risk.into());
        self
    }

    pub fn with_stop_condition(mut self, condition: impl Into<String>) -> Self {
        self.stop_condition_hash = hash_str(&condition.into());
        self
    }
}
