#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Repair,
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Verified,
    MissingEvidence,
    Regressed,
    UnsafeBoundary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GateInput {
    pub signal: Signal,
    pub evidence_count: u8,
    pub repair_budget: u8,
}

impl GateInput {
    pub const fn verified(evidence_count: u8) -> Self {
        Self {
            signal: Signal::Verified,
            evidence_count,
            repair_budget: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvalGate {
    min_evidence: u8,
}

impl EvalGate {
    pub const fn new(min_evidence: u8) -> Self {
        Self { min_evidence }
    }

    pub const fn decide(&self, input: GateInput) -> Decision {
        match input.signal {
            Signal::UnsafeBoundary | Signal::Regressed => Decision::Deny,
            Signal::MissingEvidence if input.repair_budget > 0 => Decision::Repair,
            Signal::MissingEvidence => Decision::Deny,
            Signal::Verified if input.evidence_count >= self.min_evidence => Decision::Allow,
            Signal::Verified if input.repair_budget > 0 => Decision::Repair,
            Signal::Verified => Decision::Deny,
        }
    }

    pub const fn min_evidence(&self) -> u8 {
        self.min_evidence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsafe_or_regressed_always_denies() {
        let gate = EvalGate::new(2);
        assert_eq!(gate.decide(GateInput { signal: Signal::UnsafeBoundary, evidence_count: 99, repair_budget: 99 }), Decision::Deny);
        assert_eq!(gate.decide(GateInput { signal: Signal::Regressed, evidence_count: 99, repair_budget: 99 }), Decision::Deny);
    }
}