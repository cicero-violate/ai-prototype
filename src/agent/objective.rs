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

#[derive(Clone, Copy)]
enum ObjectiveHashSlot {
    RiskEnvelope,
    StopCondition,
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

    pub fn with_risk_envelope(self, risk: impl Into<String>) -> Self {
        self.set_hash_slot(ObjectiveHashSlot::RiskEnvelope, risk)
    }

    pub fn with_stop_condition(self, condition: impl Into<String>) -> Self {
        self.set_hash_slot(ObjectiveHashSlot::StopCondition, condition)
    }

    fn set_hash_slot(mut self, slot: ObjectiveHashSlot, value: impl Into<String>) -> Self {
        let hash = hash_str(&value.into());
        match slot {
            ObjectiveHashSlot::RiskEnvelope => self.risk_envelope_hash = hash,
            ObjectiveHashSlot::StopCondition => self.stop_condition_hash = hash,
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn objective_hash_setters_preserve_independent_fields() {
        let baseline = AgentObjective::new("domain", "metric");
        let objective = baseline
            .clone()
            .with_risk_envelope("bounded-risk")
            .with_stop_condition("phase-done");

        assert_ne!(objective.risk_envelope_hash, 0);
        assert_ne!(objective.stop_condition_hash, 0);
        assert_ne!(objective.risk_envelope_hash, objective.stop_condition_hash);

        assert_eq!(objective.objective_id, baseline.objective_id);
        assert_eq!(objective.objective_hash, baseline.objective_hash);
        assert_eq!(objective.success_metric_hash, baseline.success_metric_hash);
        assert_eq!(objective.domain_hint, baseline.domain_hint);
        assert_eq!(objective.success_metric, baseline.success_metric);
    }
}
