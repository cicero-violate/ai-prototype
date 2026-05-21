//! Pure trading-domain records and sandbox-only enforcement helpers.
//!
//! This module is descriptor-only. It has no I/O, process, network, runtime,
//! command-ledger, external execution, or TLog mutation authority. Trading
//! records may describe simulation and backtest plans, but live execution
//! remains outside this domain layer.

use super::contracts::{Horizon, LiveEffectLevel};
use super::scoring::BoundedScore;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TradingSimulationPlan {
    pub plan_id: String,
    pub market_id: String,
    pub thesis_hash: String,
    pub rule_set_hash: String,
    pub dataset_hash: String,
    pub time_range_hash: String,
    pub risk_limit_hash: String,
    pub entry_rule_hash: String,
    pub exit_rule_hash: String,
    pub invalidation_hash: String,
    pub expected_metric_hash: String,
    pub horizon: Horizon,
    pub requested_live_effect_level: LiveEffectLevel,
    pub sandbox_only: bool,
    pub external_integration_allowed: bool,
    pub live_execution_allowed: bool,
}

impl TradingSimulationPlan {
    #[expect(
        clippy::too_many_arguments,
        reason = "constructors mirror canonical trading record fields"
    )]
    pub fn new(
        plan_id: impl Into<String>,
        market_id: impl Into<String>,
        thesis_hash: impl Into<String>,
        rule_set_hash: impl Into<String>,
        dataset_hash: impl Into<String>,
        time_range_hash: impl Into<String>,
        risk_limit_hash: impl Into<String>,
        entry_rule_hash: impl Into<String>,
        exit_rule_hash: impl Into<String>,
        invalidation_hash: impl Into<String>,
        expected_metric_hash: impl Into<String>,
        horizon: Horizon,
        requested_live_effect_level: LiveEffectLevel,
        sandbox_only: bool,
        external_integration_allowed: bool,
        live_execution_allowed: bool,
    ) -> Self {
        Self {
            plan_id: plan_id.into(),
            market_id: market_id.into(),
            thesis_hash: thesis_hash.into(),
            rule_set_hash: rule_set_hash.into(),
            dataset_hash: dataset_hash.into(),
            time_range_hash: time_range_hash.into(),
            risk_limit_hash: risk_limit_hash.into(),
            entry_rule_hash: entry_rule_hash.into(),
            exit_rule_hash: exit_rule_hash.into(),
            invalidation_hash: invalidation_hash.into(),
            expected_metric_hash: expected_metric_hash.into(),
            horizon,
            requested_live_effect_level,
            sandbox_only,
            external_integration_allowed,
            live_execution_allowed,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BacktestReceiptRequirements {
    pub dataset_hash: String,
    pub date_range_hash: String,
    pub rule_hash: String,
    pub metric_hash: String,
    pub limitation_hash: String,
    pub post_trade_review_hash: String,
    pub requires_rule_adherence: bool,
    pub requires_data_quality: bool,
    pub requires_overfit_check: bool,
}

impl BacktestReceiptRequirements {
    #[expect(
        clippy::too_many_arguments,
        reason = "constructors mirror canonical trading record fields"
    )]
    pub fn new(
        dataset_hash: impl Into<String>,
        date_range_hash: impl Into<String>,
        rule_hash: impl Into<String>,
        metric_hash: impl Into<String>,
        limitation_hash: impl Into<String>,
        post_trade_review_hash: impl Into<String>,
        requires_rule_adherence: bool,
        requires_data_quality: bool,
        requires_overfit_check: bool,
    ) -> Self {
        Self {
            dataset_hash: dataset_hash.into(),
            date_range_hash: date_range_hash.into(),
            rule_hash: rule_hash.into(),
            metric_hash: metric_hash.into(),
            limitation_hash: limitation_hash.into(),
            post_trade_review_hash: post_trade_review_hash.into(),
            requires_rule_adherence,
            requires_data_quality,
            requires_overfit_check,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TradingRiskLimit {
    pub max_loss: BoundedScore,
    pub max_drawdown: BoundedScore,
    pub max_position: BoundedScore,
    pub max_leverage: BoundedScore,
    pub overfit_risk: BoundedScore,
    pub stop_condition_hash: String,
}

impl TradingRiskLimit {
    pub fn new(
        max_loss: u16,
        max_drawdown: u16,
        max_position: u16,
        max_leverage: u16,
        overfit_risk: u16,
        stop_condition_hash: impl Into<String>,
    ) -> Self {
        Self {
            max_loss: BoundedScore::new(max_loss),
            max_drawdown: BoundedScore::new(max_drawdown),
            max_position: BoundedScore::new(max_position),
            max_leverage: BoundedScore::new(max_leverage),
            overfit_risk: BoundedScore::new(overfit_risk),
            stop_condition_hash: stop_condition_hash.into(),
        }
    }

    pub fn aggregate_risk_score(&self) -> BoundedScore {
        BoundedScore::saturating_weighted_average(&[
            (self.max_loss, 3),
            (self.max_drawdown, 3),
            (self.max_position, 2),
            (self.max_leverage, 3),
            (self.overfit_risk, 2),
        ])
    }
}

fn has_material_hash(value: &str) -> bool {
    !value.trim().is_empty()
}

pub fn enforce_sandbox_only(
    plan: &TradingSimulationPlan,
    receipts: &BacktestReceiptRequirements,
    risk_limit: &TradingRiskLimit,
) -> bool {
    plan.sandbox_only
        && !plan.external_integration_allowed
        && !plan.live_execution_allowed
        && plan.requested_live_effect_level <= LiveEffectLevel::SandboxWrite
        && has_material_hash(&plan.thesis_hash)
        && has_material_hash(&plan.rule_set_hash)
        && has_material_hash(&plan.dataset_hash)
        && has_material_hash(&plan.risk_limit_hash)
        && has_material_hash(&plan.invalidation_hash)
        && has_material_hash(&receipts.dataset_hash)
        && has_material_hash(&receipts.date_range_hash)
        && has_material_hash(&receipts.rule_hash)
        && has_material_hash(&receipts.metric_hash)
        && has_material_hash(&receipts.limitation_hash)
        && has_material_hash(&receipts.post_trade_review_hash)
        && receipts.requires_rule_adherence
        && receipts.requires_data_quality
        && receipts.requires_overfit_check
        && has_material_hash(&risk_limit.stop_condition_hash)
        && risk_limit.aggregate_risk_score().get() <= 750
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sandbox_plan() -> TradingSimulationPlan {
        TradingSimulationPlan::new(
            "plan:trading:simulation:001",
            "market:equities:paper",
            "hash:thesis",
            "hash:rules",
            "hash:dataset",
            "hash:time-range",
            "hash:risk-limit",
            "hash:entry-rule",
            "hash:exit-rule",
            "hash:invalidation",
            "hash:expected-metrics",
            Horizon::Tactical,
            LiveEffectLevel::SandboxWrite,
            true,
            false,
            false,
        )
    }

    fn receipt_requirements() -> BacktestReceiptRequirements {
        BacktestReceiptRequirements::new(
            "hash:dataset",
            "hash:date-range",
            "hash:rules",
            "hash:metrics",
            "hash:limitations",
            "hash:post-trade-review",
            true,
            true,
            true,
        )
    }

    fn risk_limit() -> TradingRiskLimit {
        TradingRiskLimit::new(120, 180, 100, 90, 240, "hash:stop-condition")
    }

    #[test]
    fn trading_simulation_plan_rejects_live_execution() {
        let receipts = receipt_requirements();
        let risk_limit = risk_limit();
        let sandbox = sandbox_plan();

        assert!(enforce_sandbox_only(&sandbox, &receipts, &risk_limit));

        let mut live_execution = sandbox.clone();
        live_execution.requested_live_effect_level = LiveEffectLevel::FinancialExecution;
        live_execution.live_execution_allowed = true;

        assert!(!enforce_sandbox_only(
            &live_execution,
            &receipts,
            &risk_limit
        ));

        let mut brokerage_integration = sandbox;
        brokerage_integration.external_integration_allowed = true;

        assert!(!enforce_sandbox_only(
            &brokerage_integration,
            &receipts,
            &risk_limit
        ));
    }
}
