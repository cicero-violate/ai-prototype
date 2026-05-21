//! Pure finance-domain records and research-only allowance helpers.
//!
//! This module is descriptor-only. It has no I/O, process, network, runtime,
//! command-ledger, or TLog mutation authority. Finance records may describe
//! research and allocation hypotheses, but execution remains outside this
//! domain layer.

use super::contracts::Horizon;
use super::scoring::BoundedScore;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetUniverse {
    Equities,
    Etfs,
    Indexes,
    Rates,
    Fx,
    Commodities,
    Crypto,
    PrivateAssets,
    MultiAsset,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FinanceHypothesis {
    pub hypothesis_id: String,
    pub asset_universe: AssetUniverse,
    pub asset_or_theme_id: String,
    pub thesis_hash: String,
    pub source_set_hash: String,
    pub catalyst_hash: String,
    pub horizon: Horizon,
    pub expected_value_score: BoundedScore,
    pub downside_risk_score: BoundedScore,
    pub liquidity_score: BoundedScore,
    pub confidence_score: BoundedScore,
    pub uncertainty_score: BoundedScore,
    pub invalidation_hash: String,
    pub execution_allowed: bool,
}

impl FinanceHypothesis {
    #[expect(
        clippy::too_many_arguments,
        reason = "constructors mirror canonical finance record fields"
    )]
    pub fn new(
        hypothesis_id: impl Into<String>,
        asset_universe: AssetUniverse,
        asset_or_theme_id: impl Into<String>,
        thesis_hash: impl Into<String>,
        source_set_hash: impl Into<String>,
        catalyst_hash: impl Into<String>,
        horizon: Horizon,
        expected_value_score: u16,
        downside_risk_score: u16,
        liquidity_score: u16,
        confidence_score: u16,
        uncertainty_score: u16,
        invalidation_hash: impl Into<String>,
        execution_allowed: bool,
    ) -> Self {
        Self {
            hypothesis_id: hypothesis_id.into(),
            asset_universe,
            asset_or_theme_id: asset_or_theme_id.into(),
            thesis_hash: thesis_hash.into(),
            source_set_hash: source_set_hash.into(),
            catalyst_hash: catalyst_hash.into(),
            horizon,
            expected_value_score: BoundedScore::new(expected_value_score),
            downside_risk_score: BoundedScore::new(downside_risk_score),
            liquidity_score: BoundedScore::new(liquidity_score),
            confidence_score: BoundedScore::new(confidence_score),
            uncertainty_score: BoundedScore::new(uncertainty_score),
            invalidation_hash: invalidation_hash.into(),
            execution_allowed,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FinanceRiskDimensions {
    pub drawdown: BoundedScore,
    pub volatility: BoundedScore,
    pub liquidity: BoundedScore,
    pub concentration: BoundedScore,
    pub correlation: BoundedScore,
    pub leverage: BoundedScore,
    pub counterparty_or_platform_risk: BoundedScore,
    pub regulatory_risk: BoundedScore,
    pub model_risk: BoundedScore,
}

impl FinanceRiskDimensions {
    #[expect(
        clippy::too_many_arguments,
        reason = "constructors mirror canonical finance record fields"
    )]
    pub fn new(
        drawdown: u16,
        volatility: u16,
        liquidity: u16,
        concentration: u16,
        correlation: u16,
        leverage: u16,
        counterparty_or_platform_risk: u16,
        regulatory_risk: u16,
        model_risk: u16,
    ) -> Self {
        Self {
            drawdown: BoundedScore::new(drawdown),
            volatility: BoundedScore::new(volatility),
            liquidity: BoundedScore::new(liquidity),
            concentration: BoundedScore::new(concentration),
            correlation: BoundedScore::new(correlation),
            leverage: BoundedScore::new(leverage),
            counterparty_or_platform_risk: BoundedScore::new(counterparty_or_platform_risk),
            regulatory_risk: BoundedScore::new(regulatory_risk),
            model_risk: BoundedScore::new(model_risk),
        }
    }

    pub fn aggregate_risk_score(&self) -> BoundedScore {
        BoundedScore::saturating_weighted_average(&[
            (self.drawdown, 3),
            (self.volatility, 2),
            (self.liquidity, 2),
            (self.concentration, 2),
            (self.correlation, 1),
            (self.leverage, 3),
            (self.counterparty_or_platform_risk, 2),
            (self.regulatory_risk, 2),
            (self.model_risk, 2),
        ])
    }
}

fn has_material_hash(value: &str) -> bool {
    !value.trim().is_empty()
}

pub fn finance_research_allowed(
    hypothesis: &FinanceHypothesis,
    risk: &FinanceRiskDimensions,
) -> bool {
    !hypothesis.execution_allowed
        && has_material_hash(&hypothesis.thesis_hash)
        && has_material_hash(&hypothesis.source_set_hash)
        && has_material_hash(&hypothesis.invalidation_hash)
        && hypothesis.confidence_score.get() >= 400
        && hypothesis.uncertainty_score.get() <= 700
        && risk.aggregate_risk_score().get() <= 750
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finance_hypothesis_execution_allowed_is_false() {
        let hypothesis = FinanceHypothesis::new(
            "finance:hypothesis:execution-blocked",
            AssetUniverse::Etfs,
            "broad-market-risk-premium",
            "thesis:hash:001",
            "source:set:hash:001",
            "catalyst:hash:001",
            Horizon::Strategic,
            720,
            380,
            810,
            760,
            240,
            "invalidation:hash:001",
            true,
        );
        let risk = FinanceRiskDimensions::new(220, 240, 180, 260, 210, 100, 190, 200, 230);

        assert!(hypothesis.execution_allowed);
        assert_eq!(risk.aggregate_risk_score().get(), 198);
        assert!(!finance_research_allowed(&hypothesis, &risk));
    }

    #[test]
    fn finance_research_plan_passes_research_only_risk_check() {
        use crate::domain::contracts::{
            DomainBridgeTarget, DomainId, DomainPlan, DomainRiskEnvelope, LiveEffectLevel,
            PlanKind, DOMAIN_SCHEMA_VERSION,
        };
        use crate::domain::risk::check_risk_envelope;

        let hypothesis = FinanceHypothesis::new(
            "finance:hypothesis:research-only",
            AssetUniverse::Etfs,
            "broad-market-risk-premium",
            "thesis:hash:research-only",
            "source:set:hash:research-only",
            "catalyst:hash:research-only",
            Horizon::Strategic,
            720,
            380,
            810,
            760,
            240,
            "invalidation:hash:research-only",
            false,
        );
        let risk = FinanceRiskDimensions::new(220, 240, 180, 260, 210, 100, 190, 200, 230);
        let plan = DomainPlan {
            schema_version: DOMAIN_SCHEMA_VERSION,
            plan_id: "finance-research-plan".to_string(),
            domain_id: DomainId::Finance,
            judgment_hash: "hash:finance-judgment".to_string(),
            plan_kind: PlanKind::FinanceResearch,
            bridge_target: DomainBridgeTarget::PlanRecord,
            required_capability_set_hash: "hash:finance-research-capabilities".to_string(),
            expected_receipt_set_hash: "hash:finance-research-receipts".to_string(),
            risk_envelope_hash: "hash:finance-research-envelope".to_string(),
            success_metric_hash: "hash:finance-research-success".to_string(),
            rollback_or_invalidation_hash: hypothesis.invalidation_hash.clone(),
            requested_live_effect_level: LiveEffectLevel::ReadOnly,
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

        assert!(finance_research_allowed(&hypothesis, &risk));
        assert_eq!(risk.aggregate_risk_score().get(), 198);
        assert_eq!(check_risk_envelope(&plan, &envelope), Ok(()));
    }
}
