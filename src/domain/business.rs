//! Pure business-domain records and deterministic scoring helpers.
//!
//! This module is descriptor-only. It has no I/O, process, network, runtime,
//! command-ledger, or TLog mutation authority. Business records describe
//! opportunities and workflow candidates that must still be executed and
//! verified by capabilities.

use super::scoring::BoundedScore;

pub type WorkflowId = String;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BusinessOpportunity {
    pub opportunity_id: String,
    pub workflow_id: WorkflowId,
    pub market_need: BoundedScore,
    pub willingness_to_pay: BoundedScore,
    pub implementation_fit: BoundedScore,
    pub verification_readiness: BoundedScore,
    pub delivery_risk: BoundedScore,
}

impl BusinessOpportunity {
    pub fn new(
        opportunity_id: impl Into<String>,
        workflow_id: impl Into<String>,
        market_need: u16,
        willingness_to_pay: u16,
        implementation_fit: u16,
        verification_readiness: u16,
        delivery_risk: u16,
    ) -> Self {
        Self {
            opportunity_id: opportunity_id.into(),
            workflow_id: workflow_id.into(),
            market_need: BoundedScore::new(market_need),
            willingness_to_pay: BoundedScore::new(willingness_to_pay),
            implementation_fit: BoundedScore::new(implementation_fit),
            verification_readiness: BoundedScore::new(verification_readiness),
            delivery_risk: BoundedScore::new(delivery_risk),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkflowAutomationCandidate {
    pub candidate_id: String,
    pub workflow_id: WorkflowId,
    pub manual_cost: BoundedScore,
    pub repeatability: BoundedScore,
    pub auditability: BoundedScore,
    pub rollback_readiness: BoundedScore,
    pub integration_risk: BoundedScore,
}

impl WorkflowAutomationCandidate {
    pub fn new(
        candidate_id: impl Into<String>,
        workflow_id: impl Into<String>,
        manual_cost: u16,
        repeatability: u16,
        auditability: u16,
        rollback_readiness: u16,
        integration_risk: u16,
    ) -> Self {
        Self {
            candidate_id: candidate_id.into(),
            workflow_id: workflow_id.into(),
            manual_cost: BoundedScore::new(manual_cost),
            repeatability: BoundedScore::new(repeatability),
            auditability: BoundedScore::new(auditability),
            rollback_readiness: BoundedScore::new(rollback_readiness),
            integration_risk: BoundedScore::new(integration_risk),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomerFeedbackSignal {
    pub feedback_id: String,
    pub workflow_id: WorkflowId,
    pub urgency: BoundedScore,
    pub frequency: BoundedScore,
    pub evidence_quality: BoundedScore,
    pub contradiction: BoundedScore,
}

impl CustomerFeedbackSignal {
    pub fn new(
        feedback_id: impl Into<String>,
        workflow_id: impl Into<String>,
        urgency: u16,
        frequency: u16,
        evidence_quality: u16,
        contradiction: u16,
    ) -> Self {
        Self {
            feedback_id: feedback_id.into(),
            workflow_id: workflow_id.into(),
            urgency: BoundedScore::new(urgency),
            frequency: BoundedScore::new(frequency),
            evidence_quality: BoundedScore::new(evidence_quality),
            contradiction: BoundedScore::new(contradiction),
        }
    }
}

pub fn monetization_score(
    opportunity: &BusinessOpportunity,
    workflow: &WorkflowAutomationCandidate,
    feedback: &CustomerFeedbackSignal,
) -> BoundedScore {
    let demand = BoundedScore::saturating_weighted_average(&[
        (opportunity.market_need, 3),
        (opportunity.willingness_to_pay, 3),
        (feedback.urgency, 2),
        (feedback.frequency, 2),
    ]);
    let execution = BoundedScore::saturating_weighted_average(&[
        (opportunity.implementation_fit, 2),
        (opportunity.verification_readiness, 2),
        (workflow.repeatability, 2),
        (workflow.auditability, 2),
        (workflow.rollback_readiness, 1),
    ]);
    let cost_relief = BoundedScore::saturating_weighted_average(&[
        (workflow.manual_cost, 2),
        (feedback.evidence_quality, 1),
    ]);
    let positive =
        BoundedScore::saturating_weighted_average(&[(demand, 4), (execution, 3), (cost_relief, 2)]);
    let risk_penalty = BoundedScore::saturating_weighted_average(&[
        (opportunity.delivery_risk, 2),
        (workflow.integration_risk, 2),
        (feedback.contradiction, 1),
    ]);

    BoundedScore::new(positive.get().saturating_sub(risk_penalty.get() / 2))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opportunity() -> BusinessOpportunity {
        BusinessOpportunity::new(
            "business-opportunity-1",
            "workflow-1",
            820,
            760,
            700,
            680,
            240,
        )
    }

    fn workflow() -> WorkflowAutomationCandidate {
        WorkflowAutomationCandidate::new(
            "workflow-candidate-1",
            "workflow-1",
            840,
            780,
            720,
            700,
            260,
        )
    }

    fn feedback() -> CustomerFeedbackSignal {
        CustomerFeedbackSignal::new("feedback-1", "workflow-1", 760, 720, 800, 120)
    }

    #[test]
    fn business_module_records_and_score_helper_compile() {
        let score = monetization_score(&opportunity(), &workflow(), &feedback());

        assert_eq!(score.get(), 652);
    }

    #[test]
    fn business_monetization_score_is_deterministic() {
        let opportunity = opportunity();
        let workflow = workflow();
        let feedback = feedback();

        let first = monetization_score(&opportunity, &workflow, &feedback);
        let second = monetization_score(&opportunity, &workflow, &feedback);
        let third = monetization_score(&opportunity, &workflow, &feedback);

        assert_eq!(first, second);
        assert_eq!(second, third);
        assert_eq!(first.get(), 652);
    }

    #[test]
    fn business_monetization_score_is_bounded() {
        let opportunity = BusinessOpportunity::new(
            "business-opportunity-overflow",
            "workflow-overflow",
            1_200,
            1_300,
            1_400,
            1_500,
            0,
        );
        let workflow = WorkflowAutomationCandidate::new(
            "workflow-candidate-overflow",
            "workflow-overflow",
            1_600,
            1_700,
            1_800,
            1_900,
            0,
        );
        let feedback = CustomerFeedbackSignal::new(
            "feedback-overflow",
            "workflow-overflow",
            2_000,
            2_100,
            2_200,
            0,
        );

        let score = monetization_score(&opportunity, &workflow, &feedback);

        assert_eq!(score, BoundedScore::max());
        assert!(score.get() <= BoundedScore::max().get());
    }
}
