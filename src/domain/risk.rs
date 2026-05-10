//! Domain risk-envelope checks.

use super::contracts::{DomainRiskEnvelope, LiveEffectLevel};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DomainRiskDecision {
    Pass,
    Block { reason: &'static str },
}

pub fn evaluate_risk_envelope(
    envelope: &DomainRiskEnvelope,
    confidence: u16,
    uncertainty: u16,
    staleness: u16,
    requested_effect: LiveEffectLevel,
) -> DomainRiskDecision {
    if uncertainty > envelope.max_uncertainty {
        return DomainRiskDecision::Block {
            reason: "uncertainty_exceeds_envelope",
        };
    }
    if confidence < envelope.min_confidence {
        return DomainRiskDecision::Block {
            reason: "confidence_below_envelope",
        };
    }
    if staleness > envelope.max_staleness {
        return DomainRiskDecision::Block {
            reason: "staleness_exceeds_envelope",
        };
    }
    if requested_effect > envelope.max_live_effect_level {
        return DomainRiskDecision::Block {
            reason: "live_effect_exceeds_envelope",
        };
    }
    if envelope.sandbox_only && requested_effect > LiveEffectLevel::SandboxWrite {
        return DomainRiskDecision::Block {
            reason: "sandbox_only_boundary",
        };
    }

    DomainRiskDecision::Pass
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contracts::{DomainId, DOMAIN_SCHEMA_VERSION};

    #[test]
    fn blocks_live_financial_execution_for_sandbox_envelope() {
        let envelope = DomainRiskEnvelope {
            schema_version: DOMAIN_SCHEMA_VERSION,
            envelope_id: "sandbox".to_string(),
            domain_id: DomainId::TradingSandbox,
            max_uncertainty: 500,
            min_confidence: 400,
            max_staleness: 300,
            max_live_effect_level: LiveEffectLevel::SandboxWrite,
            requires_human_review: true,
            requires_verification: true,
            sandbox_only: true,
        };

        assert_eq!(
            evaluate_risk_envelope(
                &envelope,
                700,
                200,
                100,
                LiveEffectLevel::FinancialExecution
            ),
            DomainRiskDecision::Block {
                reason: "live_effect_exceeds_envelope"
            }
        );
    }
}
