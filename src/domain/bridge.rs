//! Descriptor-only bridge mapping from domain concepts to capability families.

use super::contracts::{
    DomainBridgeTarget, DomainContext, DomainEval, DomainId, DomainJudgment, DomainPlan,
    DomainSignal, DomainVerdict, PlanKind,
};

const OBSERVATION_RECEIPTS: &[&str] = &["observation_record"];
const CONTEXT_RECEIPTS: &[&str] = &["context_record"];
const JUDGMENT_RECEIPTS: &[&str] = &["judgment_record"];
const PLAN_RECEIPTS: &[&str] = &["plan_record", "verification_record"];
const EVAL_RECEIPTS: &[&str] = &["eval_record"];
const BLOCK_RECEIPTS: &[&str] = &["blocked_record"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DomainBridgeDescriptor {
    pub domain_id: DomainId,
    pub record_family: &'static str,
    pub target: DomainBridgeTarget,
    pub capability_family: &'static str,
    pub required_receipt_families: &'static [&'static str],
    pub plan_kind: Option<PlanKind>,
}

impl DomainBridgeDescriptor {
    pub const fn new(
        domain_id: DomainId,
        record_family: &'static str,
        target: DomainBridgeTarget,
        capability_family: &'static str,
        required_receipt_families: &'static [&'static str],
        plan_kind: Option<PlanKind>,
    ) -> Self {
        Self {
            domain_id,
            record_family,
            target,
            capability_family,
            required_receipt_families,
            plan_kind,
        }
    }
}

pub fn bridge_target_for_verdict(verdict: DomainVerdict) -> DomainBridgeTarget {
    match verdict {
        DomainVerdict::Ignore | DomainVerdict::Watch => DomainBridgeTarget::ObservationRecord,
        DomainVerdict::Research => DomainBridgeTarget::JudgmentRecord,
        DomainVerdict::ActBusiness
        | DomainVerdict::ActFinanceResearch
        | DomainVerdict::SimulateTrading => DomainBridgeTarget::PlanRecord,
        DomainVerdict::Block => DomainBridgeTarget::Blocked,
    }
}

pub fn default_plan_kind(domain_id: DomainId, verdict: DomainVerdict) -> Option<PlanKind> {
    match (domain_id, verdict) {
        (DomainId::Business, DomainVerdict::ActBusiness) => Some(PlanKind::BusinessWorkflowPlan),
        (DomainId::Finance, DomainVerdict::ActFinanceResearch) => {
            Some(PlanKind::FinanceAnalysisPlan)
        }
        (DomainId::TradingSandbox, DomainVerdict::SimulateTrading) => {
            Some(PlanKind::TradingSimulationPlan)
        }
        (_, DomainVerdict::Research) => Some(PlanKind::ResearchPlan),
        _ => None,
    }
}

fn descriptor_for_target(
    domain_id: DomainId,
    record_family: &'static str,
    target: DomainBridgeTarget,
    plan_kind: Option<PlanKind>,
) -> DomainBridgeDescriptor {
    match target {
        DomainBridgeTarget::ObservationRecord => DomainBridgeDescriptor::new(
            domain_id,
            record_family,
            target,
            "observation",
            OBSERVATION_RECEIPTS,
            plan_kind,
        ),
        DomainBridgeTarget::ContextRecord => DomainBridgeDescriptor::new(
            domain_id,
            record_family,
            target,
            "context",
            CONTEXT_RECEIPTS,
            plan_kind,
        ),
        DomainBridgeTarget::JudgmentRecord => DomainBridgeDescriptor::new(
            domain_id,
            record_family,
            target,
            "judgment",
            JUDGMENT_RECEIPTS,
            plan_kind,
        ),
        DomainBridgeTarget::PlanRecord => DomainBridgeDescriptor::new(
            domain_id,
            record_family,
            target,
            "planning",
            PLAN_RECEIPTS,
            plan_kind,
        ),
        DomainBridgeTarget::VerificationRecord => DomainBridgeDescriptor::new(
            domain_id,
            record_family,
            target,
            "verification",
            PLAN_RECEIPTS,
            plan_kind,
        ),
        DomainBridgeTarget::EvalRecord => DomainBridgeDescriptor::new(
            domain_id,
            record_family,
            target,
            "evaluation",
            EVAL_RECEIPTS,
            plan_kind,
        ),
        DomainBridgeTarget::PolicyPromotion => DomainBridgeDescriptor::new(
            domain_id,
            record_family,
            target,
            "policy_promotion",
            EVAL_RECEIPTS,
            plan_kind,
        ),
        DomainBridgeTarget::Blocked => DomainBridgeDescriptor::new(
            domain_id,
            record_family,
            target,
            "blocked",
            BLOCK_RECEIPTS,
            plan_kind,
        ),
    }
}

pub fn bridge_target_for_signal(signal: &DomainSignal) -> DomainBridgeDescriptor {
    descriptor_for_target(
        signal.domain_id,
        "DomainSignal",
        DomainBridgeTarget::ObservationRecord,
        None,
    )
}

pub fn bridge_target_for_context(context: &DomainContext) -> DomainBridgeDescriptor {
    descriptor_for_target(
        context.domain_id,
        "DomainContext",
        DomainBridgeTarget::ContextRecord,
        None,
    )
}

pub fn bridge_target_for_judgment(judgment: &DomainJudgment) -> DomainBridgeDescriptor {
    let target = bridge_target_for_verdict(judgment.verdict);
    descriptor_for_target(
        judgment.domain_id,
        "DomainJudgment",
        target,
        default_plan_kind(judgment.domain_id, judgment.verdict),
    )
}

pub fn bridge_target_for_plan(plan: &DomainPlan) -> DomainBridgeDescriptor {
    descriptor_for_target(
        plan.domain_id,
        "DomainPlan",
        plan.bridge_target,
        Some(plan.plan_kind),
    )
}

pub fn bridge_target_for_eval(eval: &DomainEval) -> DomainBridgeDescriptor {
    let target = if eval.promotion_allowed {
        DomainBridgeTarget::PolicyPromotion
    } else {
        DomainBridgeTarget::EvalRecord
    };

    descriptor_for_target(eval.domain_id, "DomainEval", target, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contracts::{
        DomainHorizon, DomainSignalClass, DomainSourceKind, LiveEffectLevel, DOMAIN_SCHEMA_VERSION,
    };

    #[test]
    fn maps_stage_one_fixture_verdicts_to_bridge_targets() {
        assert_eq!(
            bridge_target_for_verdict(DomainVerdict::Watch),
            DomainBridgeTarget::ObservationRecord
        );
        assert_eq!(
            bridge_target_for_verdict(DomainVerdict::ActBusiness),
            DomainBridgeTarget::PlanRecord
        );
        assert_eq!(
            bridge_target_for_verdict(DomainVerdict::ActFinanceResearch),
            DomainBridgeTarget::PlanRecord
        );
        assert_eq!(
            bridge_target_for_verdict(DomainVerdict::SimulateTrading),
            DomainBridgeTarget::PlanRecord
        );
        assert_eq!(
            bridge_target_for_verdict(DomainVerdict::Block),
            DomainBridgeTarget::Blocked
        );
    }

    #[test]
    fn bridge_maps_each_domain_record_family() {
        let signal = DomainSignal::new(
            "signal-1",
            DomainId::GlobalIntelligence,
            "source-1",
            "2026-05-11T00:00:00Z",
            DomainHorizon::Tactical,
            DomainSignalClass::MacroTrend,
            "hash:payload",
            "hash:provenance",
            800,
            700,
            100,
        )
        .expect("valid signal");
        let context = DomainContext::new(
            "context-1",
            DomainId::GlobalIntelligence,
            "hash:signal",
            "hash:context",
            "hash:provenance",
            DomainHorizon::Tactical,
            DomainSourceKind::WorldSignal,
            700,
            100,
        )
        .expect("valid context");
        let judgment = DomainJudgment::new(
            "judgment-1",
            DomainId::Business,
            800,
            200,
            700,
            200,
            600,
            900,
            500,
            DomainVerdict::ActBusiness,
            "hash:rationale",
        )
        .expect("valid judgment");
        let plan = DomainPlan {
            schema_version: DOMAIN_SCHEMA_VERSION,
            plan_id: "plan-1".to_string(),
            domain_id: DomainId::Business,
            judgment_hash: "hash:judgment".to_string(),
            plan_kind: PlanKind::BusinessWorkflowPlan,
            bridge_target: DomainBridgeTarget::PlanRecord,
            required_capability_set_hash: "hash:capabilities".to_string(),
            expected_receipt_set_hash: "hash:receipts".to_string(),
            risk_envelope_hash: "hash:risk".to_string(),
            success_metric_hash: "hash:metric".to_string(),
            rollback_or_invalidation_hash: "hash:rollback".to_string(),
            requested_live_effect_level: LiveEffectLevel::SandboxWrite,
        };
        let eval = DomainEval::new(
            "eval-1",
            DomainId::Business,
            "hash:plan",
            "hash:result",
            900,
            800,
            900,
            700,
            850,
            false,
        )
        .expect("valid eval");

        let signal_descriptor = bridge_target_for_signal(&signal);
        assert_eq!(signal_descriptor.record_family, "DomainSignal");
        assert_eq!(signal_descriptor.domain_id, DomainId::GlobalIntelligence);
        assert_eq!(
            signal_descriptor.target,
            DomainBridgeTarget::ObservationRecord
        );
        assert_eq!(signal_descriptor.capability_family, "observation");
        assert_eq!(
            signal_descriptor.required_receipt_families,
            OBSERVATION_RECEIPTS
        );
        assert_eq!(signal_descriptor.plan_kind, None);

        let context_descriptor = bridge_target_for_context(&context);
        assert_eq!(context_descriptor.record_family, "DomainContext");
        assert_eq!(context_descriptor.domain_id, DomainId::GlobalIntelligence);
        assert_eq!(context_descriptor.target, DomainBridgeTarget::ContextRecord);
        assert_eq!(context_descriptor.capability_family, "context");
        assert_eq!(context_descriptor.required_receipt_families, CONTEXT_RECEIPTS);
        assert_eq!(context_descriptor.plan_kind, None);

        let judgment_descriptor = bridge_target_for_judgment(&judgment);
        assert_eq!(judgment_descriptor.record_family, "DomainJudgment");
        assert_eq!(judgment_descriptor.domain_id, DomainId::Business);
        assert_eq!(judgment_descriptor.target, DomainBridgeTarget::PlanRecord);
        assert_eq!(judgment_descriptor.capability_family, "planning");
        assert_eq!(judgment_descriptor.required_receipt_families, PLAN_RECEIPTS);
        assert_eq!(
            judgment_descriptor.plan_kind,
            Some(PlanKind::BusinessWorkflowPlan)
        );

        let plan_descriptor = bridge_target_for_plan(&plan);
        assert_eq!(plan_descriptor.record_family, "DomainPlan");
        assert_eq!(plan_descriptor.domain_id, DomainId::Business);
        assert_eq!(plan_descriptor.target, DomainBridgeTarget::PlanRecord);
        assert_eq!(plan_descriptor.capability_family, "planning");
        assert_eq!(plan_descriptor.required_receipt_families, PLAN_RECEIPTS);
        assert_eq!(
            plan_descriptor.plan_kind,
            Some(PlanKind::BusinessWorkflowPlan)
        );

        let eval_descriptor = bridge_target_for_eval(&eval);
        assert_eq!(eval_descriptor.record_family, "DomainEval");
        assert_eq!(eval_descriptor.domain_id, DomainId::Business);
        assert_eq!(eval_descriptor.target, DomainBridgeTarget::EvalRecord);
        assert_eq!(eval_descriptor.capability_family, "evaluation");
        assert_eq!(eval_descriptor.required_receipt_families, EVAL_RECEIPTS);
        assert_eq!(eval_descriptor.plan_kind, None);
    }

    #[test]
    fn bridge_never_targets_live_trading_execution() {
        fn assert_not_live_trading_descriptor(descriptor: DomainBridgeDescriptor) {
            assert_ne!(descriptor.capability_family, "live_trading");
            assert_ne!(descriptor.capability_family, "financial_execution");
            assert_ne!(descriptor.capability_family, "brokerage");
            assert!(!descriptor
                .required_receipt_families
                .contains(&"execution_record"));
            assert!(!descriptor
                .required_receipt_families
                .contains(&"brokerage_receipt"));
        }

        let simulation_judgment = DomainJudgment::new(
            "judgment-trading-simulation",
            DomainId::TradingSandbox,
            700,
            350,
            700,
            200,
            600,
            850,
            550,
            DomainVerdict::SimulateTrading,
            "hash:rationale",
        )
        .expect("valid simulation judgment");
        let simulation_descriptor = bridge_target_for_judgment(&simulation_judgment);
        assert_eq!(simulation_descriptor.domain_id, DomainId::TradingSandbox);
        assert_eq!(simulation_descriptor.record_family, "DomainJudgment");
        assert_eq!(simulation_descriptor.target, DomainBridgeTarget::PlanRecord);
        assert_eq!(simulation_descriptor.capability_family, "planning");
        assert_eq!(simulation_descriptor.required_receipt_families, PLAN_RECEIPTS);
        assert_eq!(
            simulation_descriptor.plan_kind,
            Some(PlanKind::TradingSimulationPlan)
        );
        assert_not_live_trading_descriptor(simulation_descriptor);

        let simulation_plan = DomainPlan::new(
            "plan-trading-simulation",
            DomainId::TradingSandbox,
            "hash:judgment",
            PlanKind::TradingSimulationPlan,
            DomainBridgeTarget::PlanRecord,
            "hash:capabilities",
            "hash:receipts",
            "hash:risk",
            "hash:metric",
            "hash:rollback",
            LiveEffectLevel::SandboxWrite,
        )
        .expect("valid sandbox trading plan");
        let plan_descriptor = bridge_target_for_plan(&simulation_plan);
        assert_eq!(plan_descriptor.domain_id, DomainId::TradingSandbox);
        assert_eq!(plan_descriptor.record_family, "DomainPlan");
        assert_eq!(plan_descriptor.target, DomainBridgeTarget::PlanRecord);
        assert_eq!(plan_descriptor.capability_family, "planning");
        assert_eq!(plan_descriptor.required_receipt_families, PLAN_RECEIPTS);
        assert_eq!(
            plan_descriptor.plan_kind,
            Some(PlanKind::TradingSimulationPlan)
        );
        assert_not_live_trading_descriptor(plan_descriptor);

        let blocked_judgment = DomainJudgment::new(
            "judgment-trading-blocked",
            DomainId::TradingSandbox,
            0,
            900,
            500,
            500,
            0,
            0,
            0,
            DomainVerdict::Block,
            "hash:block-rationale",
        )
        .expect("valid blocked judgment");
        let blocked_descriptor = bridge_target_for_judgment(&blocked_judgment);
        assert_eq!(blocked_descriptor.domain_id, DomainId::TradingSandbox);
        assert_eq!(blocked_descriptor.record_family, "DomainJudgment");
        assert_eq!(blocked_descriptor.target, DomainBridgeTarget::Blocked);
        assert_eq!(blocked_descriptor.capability_family, "blocked");
        assert_eq!(blocked_descriptor.required_receipt_families, BLOCK_RECEIPTS);
        assert_eq!(blocked_descriptor.plan_kind, None);
        assert_not_live_trading_descriptor(blocked_descriptor);
    }
}
