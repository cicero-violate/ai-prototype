//! Domain risk-envelope checks.

use super::contracts::{DomainPlan, DomainRiskEnvelope, LiveEffectLevel};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DomainRiskDecision {
    Pass,
    Block { reason: &'static str },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RiskEnvelopeViolation {
    DomainMismatch,
    MaxUncertaintyOutOfRange { value: u16 },
    MinConfidenceOutOfRange { value: u16 },
    MaxStalenessOutOfRange { value: u16 },
    UnsafeEnvelopeLiveEffect,
    MissingExpectedReceiptSet,
    MissingRollbackOrInvalidation,
    LiveEffectExceedsEnvelope,
    SandboxOnlyBoundary,
}

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

pub fn check_risk_envelope(
    plan: &DomainPlan,
    envelope: &DomainRiskEnvelope,
) -> Result<(), RiskEnvelopeViolation> {
    if plan.domain_id != envelope.domain_id {
        return Err(RiskEnvelopeViolation::DomainMismatch);
    }
    if envelope.max_uncertainty > 1000 {
        return Err(RiskEnvelopeViolation::MaxUncertaintyOutOfRange {
            value: envelope.max_uncertainty,
        });
    }
    if envelope.min_confidence > 1000 {
        return Err(RiskEnvelopeViolation::MinConfidenceOutOfRange {
            value: envelope.min_confidence,
        });
    }
    if envelope.max_staleness > 1000 {
        return Err(RiskEnvelopeViolation::MaxStalenessOutOfRange {
            value: envelope.max_staleness,
        });
    }
    if envelope.max_live_effect_level == LiveEffectLevel::FinancialExecution {
        return Err(RiskEnvelopeViolation::UnsafeEnvelopeLiveEffect);
    }
    if envelope.requires_verification && is_blank(&plan.expected_receipt_set_hash) {
        return Err(RiskEnvelopeViolation::MissingExpectedReceiptSet);
    }
    if (envelope.requires_verification || plan.requested_live_effect_level > LiveEffectLevel::None)
        && is_blank(&plan.rollback_or_invalidation_hash)
    {
        return Err(RiskEnvelopeViolation::MissingRollbackOrInvalidation);
    }
    if plan.requested_live_effect_level > envelope.max_live_effect_level {
        return Err(RiskEnvelopeViolation::LiveEffectExceedsEnvelope);
    }
    if envelope.sandbox_only && plan.requested_live_effect_level > LiveEffectLevel::SandboxWrite {
        return Err(RiskEnvelopeViolation::SandboxOnlyBoundary);
    }

    Ok(())
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
    use crate::domain::contracts::{DomainBridgeTarget, DomainId, PlanKind, DOMAIN_SCHEMA_VERSION};

    #[test]
    fn risk_blocks_live_trading() {
        let plan = DomainPlan {
            schema_version: DOMAIN_SCHEMA_VERSION,
            plan_id: "trading-live-plan".to_string(),
            domain_id: DomainId::TradingSandbox,
            judgment_hash: "hash:judgment".to_string(),
            plan_kind: PlanKind::TradingSimulationPlan,
            bridge_target: DomainBridgeTarget::PlanRecord,
            required_capability_set_hash: "hash:capability-set".to_string(),
            expected_receipt_set_hash: "hash:receipt-set".to_string(),
            risk_envelope_hash: "hash:risk-envelope".to_string(),
            success_metric_hash: "hash:success-metric".to_string(),
            rollback_or_invalidation_hash: "hash:rollback-or-invalidation".to_string(),
            requested_live_effect_level: LiveEffectLevel::FinancialExecution,
        };
        let envelope = DomainRiskEnvelope {
            schema_version: DOMAIN_SCHEMA_VERSION,
            envelope_id: "trading-sandbox-envelope".to_string(),
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
            check_risk_envelope(&plan, &envelope),
            Err(RiskEnvelopeViolation::LiveEffectExceedsEnvelope)
        );
    }

    #[test]
    fn risk_blocks_finance_execution() {
        let plan = DomainPlan {
            schema_version: DOMAIN_SCHEMA_VERSION,
            plan_id: "finance-execution-plan".to_string(),
            domain_id: DomainId::Finance,
            judgment_hash: "hash:judgment".to_string(),
            plan_kind: PlanKind::FinanceAnalysisPlan,
            bridge_target: DomainBridgeTarget::PlanRecord,
            required_capability_set_hash: "hash:capability-set".to_string(),
            expected_receipt_set_hash: "hash:receipt-set".to_string(),
            risk_envelope_hash: "hash:risk-envelope".to_string(),
            success_metric_hash: "hash:success-metric".to_string(),
            rollback_or_invalidation_hash: "hash:rollback-or-invalidation".to_string(),
            requested_live_effect_level: LiveEffectLevel::ExternalWrite,
        };
        let envelope = DomainRiskEnvelope {
            schema_version: DOMAIN_SCHEMA_VERSION,
            envelope_id: "finance-research-envelope".to_string(),
            domain_id: DomainId::Finance,
            max_uncertainty: 650,
            min_confidence: 400,
            max_staleness: 500,
            max_live_effect_level: LiveEffectLevel::ReadOnly,
            requires_human_review: true,
            requires_verification: true,
            sandbox_only: false,
        };

        assert_eq!(
            check_risk_envelope(&plan, &envelope),
            Err(RiskEnvelopeViolation::LiveEffectExceedsEnvelope)
        );
    }

    #[test]
    fn risk_allows_verified_business_plan_with_rollback_and_invalidation() {
        let plan = DomainPlan {
            schema_version: DOMAIN_SCHEMA_VERSION,
            plan_id: "business-workflow-plan".to_string(),
            domain_id: DomainId::Business,
            judgment_hash: "hash:judgment".to_string(),
            plan_kind: PlanKind::BusinessWorkflowPlan,
            bridge_target: DomainBridgeTarget::PlanRecord,
            required_capability_set_hash: "hash:capability-set".to_string(),
            expected_receipt_set_hash: "hash:receipt-set".to_string(),
            risk_envelope_hash: "hash:risk-envelope".to_string(),
            success_metric_hash: "hash:success-metric".to_string(),
            rollback_or_invalidation_hash: "hash:rollback-or-invalidation".to_string(),
            requested_live_effect_level: LiveEffectLevel::SandboxWrite,
        };
        let envelope = DomainRiskEnvelope {
            schema_version: DOMAIN_SCHEMA_VERSION,
            envelope_id: "business-workflow-envelope".to_string(),
            domain_id: DomainId::Business,
            max_uncertainty: 500,
            min_confidence: 400,
            max_staleness: 500,
            max_live_effect_level: LiveEffectLevel::SandboxWrite,
            requires_human_review: false,
            requires_verification: true,
            sandbox_only: false,
        };

        assert_eq!(check_risk_envelope(&plan, &envelope), Ok(()));
    }

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
