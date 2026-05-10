//! Descriptor-only bridge mapping from domain concepts to capability families.

use super::contracts::{DomainBridgeTarget, DomainId, DomainVerdict, PlanKind};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
