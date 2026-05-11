//! Deterministic bounded integer scoring for domain fixtures and records.

use super::contracts::{DomainId, DomainVerdict};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BoundedScore(u16);

impl BoundedScore {
    pub const MIN: Self = Self(0);
    pub const MAX: Self = Self(1000);

    pub fn new(value: u16) -> Self {
        Self(value.min(1000))
    }

    pub fn get(&self) -> u16 {
        self.0
    }

    pub fn zero() -> Self {
        Self::MIN
    }

    pub fn max() -> Self {
        Self::MAX
    }

    pub fn saturating_weighted_average(weighted_scores: &[(Self, u16)]) -> Self {
        let mut weighted_sum = 0u32;
        let mut total_weight = 0u32;

        for (score, weight) in weighted_scores {
            let weight = u32::from(*weight);
            weighted_sum =
                weighted_sum.saturating_add(u32::from(score.get()).saturating_mul(weight));
            total_weight = total_weight.saturating_add(weight);
        }

        if total_weight == 0 {
            return Self::zero();
        }

        Self::new((weighted_sum / total_weight).min(1000) as u16)
    }
}

pub type Score = BoundedScore;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScoreInputs {
    pub reliability: Score,
    pub directness: Score,
    pub recency: Score,
    pub independence: Score,
    pub evidence_strength: Score,
    pub consistency: Score,
    pub missing_evidence: Score,
    pub contradiction: Score,
    pub volatility: Score,
    pub model_risk: Score,
    pub impact: Score,
    pub likelihood: Score,
    pub irreversibility: Score,
    pub reproducibility: Score,
    pub correctness: Score,
    pub usefulness: Score,
    pub risk_adherence: Score,
    pub regression_risk: Score,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScoreBreakdown {
    pub source_quality: Score,
    pub confidence: Score,
    pub uncertainty: Score,
    pub risk: Score,
    pub promotion: Score,
    pub domain_value: Score,
    pub actionability: Score,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DomainScoreInputs {
    pub opportunity: Score,
    pub confidence: Score,
    pub policy_fit: Score,
    pub verification_readiness: Score,
    pub risk: Score,
    pub uncertainty: Score,
    pub staleness_penalty: Score,
    pub source_quality: Score,
    pub context_quality: Score,
}

pub fn bounded_product(values: &[Score]) -> Score {
    let mut product = 1000u32;
    for value in values {
        product = (product * u32::from(value.get())) / 1000;
    }
    Score::new(product.min(1000) as u16)
}

pub fn domain_value_score(inputs: DomainScoreInputs) -> Score {
    let positive = bounded_product(&[
        inputs.opportunity,
        inputs.confidence,
        inputs.policy_fit,
        inputs.verification_readiness,
    ]);
    let penalty = bounded_product(&[inputs.risk, inputs.uncertainty, inputs.staleness_penalty]);
    Score::new(positive.get().saturating_sub(penalty.get()))
}

pub fn actionability_score(domain_value: Score, inputs: DomainScoreInputs) -> Score {
    bounded_product(&[domain_value, inputs.source_quality, inputs.context_quality])
}

pub fn source_quality_score(
    reliability: Score,
    directness: Score,
    recency: Score,
    independence: Score,
) -> Score {
    bounded_product(&[reliability, directness, recency, independence])
}

pub fn confidence_score(
    evidence_strength: Score,
    source_quality: Score,
    consistency: Score,
) -> Score {
    bounded_product(&[evidence_strength, source_quality, consistency])
}

pub fn uncertainty_score(
    missing_evidence: Score,
    contradiction: Score,
    volatility: Score,
    model_risk: Score,
) -> Score {
    Score::saturating_weighted_average(&[
        (missing_evidence, 3),
        (contradiction, 3),
        (volatility, 2),
        (model_risk, 2),
    ])
}

pub fn risk_score(impact: Score, likelihood: Score, irreversibility: Score) -> Score {
    bounded_product(&[impact, likelihood, irreversibility])
}

pub fn promotion_score(
    reproducibility: Score,
    correctness: Score,
    usefulness: Score,
    risk_adherence: Score,
    regression_risk: Score,
) -> Score {
    let positive = bounded_product(&[reproducibility, correctness, usefulness, risk_adherence]);
    Score::new(positive.get().saturating_sub(regression_risk.get()))
}

pub fn score_breakdown(inputs: ScoreInputs, domain_inputs: DomainScoreInputs) -> ScoreBreakdown {
    let source_quality = source_quality_score(
        inputs.reliability,
        inputs.directness,
        inputs.recency,
        inputs.independence,
    );
    let confidence = confidence_score(inputs.evidence_strength, source_quality, inputs.consistency);
    let uncertainty = uncertainty_score(
        inputs.missing_evidence,
        inputs.contradiction,
        inputs.volatility,
        inputs.model_risk,
    );
    let risk = risk_score(inputs.impact, inputs.likelihood, inputs.irreversibility);
    let promotion = promotion_score(
        inputs.reproducibility,
        inputs.correctness,
        inputs.usefulness,
        inputs.risk_adherence,
        inputs.regression_risk,
    );
    let domain_value = domain_value_score(domain_inputs);
    let actionability = actionability_score(domain_value, domain_inputs);

    ScoreBreakdown {
        source_quality,
        confidence,
        uncertainty,
        risk,
        promotion,
        domain_value,
        actionability,
    }
}

pub fn verdict_for_scores(domain_id: DomainId, inputs: DomainScoreInputs) -> DomainVerdict {
    let domain_value = domain_value_score(inputs);
    let actionability = actionability_score(domain_value, inputs);

    if inputs.policy_fit.get() == 0
        || inputs.risk.get() >= 850
        || inputs.uncertainty.get() >= 850
        || inputs.confidence.get() < 200
    {
        return DomainVerdict::Block;
    }

    match domain_id {
        DomainId::Business
            if domain_value.get() >= 400
                && actionability.get() >= 200
                && inputs.confidence.get() >= 600
                && inputs.risk.get() <= 500 =>
        {
            DomainVerdict::ActBusiness
        }
        DomainId::Finance
            if actionability.get() >= 80
                && inputs.confidence.get() >= 600
                && inputs.policy_fit.get() >= 600
                && inputs.risk.get() <= 650 =>
        {
            DomainVerdict::ActFinanceResearch
        }
        DomainId::TradingSandbox
            if actionability.get() >= 80
                && inputs.policy_fit.get() >= 800
                && inputs.risk.get() <= 600 =>
        {
            DomainVerdict::SimulateTrading
        }
        _ if domain_value.get() < 100 || actionability.get() < 50 => DomainVerdict::Ignore,
        _ if domain_value.get() < 200 => DomainVerdict::Watch,
        _ => DomainVerdict::Research,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_fixture_style_integer_scores() {
        let inputs = DomainScoreInputs {
            opportunity: Score::new(850),
            confidence: Score::new(700),
            policy_fit: Score::new(900),
            verification_readiness: Score::new(800),
            risk: Score::new(300),
            uncertainty: Score::new(400),
            staleness_penalty: Score::new(200),
            source_quality: Score::new(750),
            context_quality: Score::new(650),
        };

        let value = domain_value_score(inputs);
        assert_eq!(value.get(), 404);
        assert_eq!(actionability_score(value, inputs).get(), 196);
    }

    #[test]
    fn bounded_score_clamps_and_averages_with_integer_math() {
        assert_eq!(BoundedScore::zero().get(), 0);
        assert_eq!(BoundedScore::max().get(), 1000);
        assert_eq!(BoundedScore::new(1001).get(), 1000);
        assert_eq!(BoundedScore::saturating_weighted_average(&[]).get(), 0);
        assert_eq!(
            BoundedScore::saturating_weighted_average(&[
                (BoundedScore::new(100), 1),
                (BoundedScore::new(700), 3),
            ])
            .get(),
            550
        );
    }

    #[test]
    fn scoring_helpers_use_integer_only_saturating_math() {
        let source = source_quality_score(
            Score::new(900),
            Score::new(800),
            Score::new(700),
            Score::new(600),
        );
        assert_eq!(source.get(), 302);

        assert_eq!(
            confidence_score(Score::new(850), source, Score::new(900)).get(),
            230
        );
        assert_eq!(
            uncertainty_score(
                Score::new(300),
                Score::new(100),
                Score::new(500),
                Score::new(700),
            )
            .get(),
            360
        );
        assert_eq!(
            risk_score(Score::new(800), Score::new(500), Score::new(250)).get(),
            100
        );
        assert_eq!(
            promotion_score(
                Score::new(900),
                Score::new(800),
                Score::new(700),
                Score::new(600),
                Score::new(100),
            )
            .get(),
            202
        );
    }

    #[test]
    fn score_breakdown_keeps_computed_scores_together() {
        let helper_inputs = ScoreInputs {
            reliability: Score::new(900),
            directness: Score::new(800),
            recency: Score::new(700),
            independence: Score::new(600),
            evidence_strength: Score::new(850),
            consistency: Score::new(900),
            missing_evidence: Score::new(300),
            contradiction: Score::new(100),
            volatility: Score::new(500),
            model_risk: Score::new(700),
            impact: Score::new(800),
            likelihood: Score::new(500),
            irreversibility: Score::new(250),
            reproducibility: Score::new(900),
            correctness: Score::new(800),
            usefulness: Score::new(700),
            risk_adherence: Score::new(600),
            regression_risk: Score::new(100),
        };
        let domain_inputs = DomainScoreInputs {
            opportunity: Score::new(880),
            confidence: Score::new(760),
            policy_fit: Score::new(850),
            verification_readiness: Score::new(820),
            risk: Score::new(260),
            uncertainty: Score::new(250),
            staleness_penalty: Score::new(80),
            source_quality: Score::new(740),
            context_quality: Score::new(790),
        };

        let breakdown = score_breakdown(helper_inputs, domain_inputs);
        assert_eq!(breakdown.source_quality.get(), 302);
        assert_eq!(breakdown.domain_value.get(), 459);
        assert_eq!(breakdown.actionability.get(), 267);
    }

    #[test]
    fn verdict_watch_thresholds() {
        let inputs = DomainScoreInputs {
            opportunity: Score::new(520),
            confidence: Score::new(620),
            policy_fit: Score::new(700),
            verification_readiness: Score::new(760),
            risk: Score::new(420),
            uncertainty: Score::new(500),
            staleness_penalty: Score::new(180),
            source_quality: Score::new(820),
            context_quality: Score::new(640),
        };

        let domain_value = domain_value_score(inputs);
        let actionability = actionability_score(domain_value, inputs);

        assert_eq!(domain_value.get(), 134);
        assert_eq!(actionability.get(), 69);
        assert_eq!(
            verdict_for_scores(DomainId::GlobalIntelligence, inputs),
            DomainVerdict::Watch
        );
    }

    #[test]
    fn verdict_research_thresholds() {
        let inputs = DomainScoreInputs {
            opportunity: Score::new(800),
            confidence: Score::new(700),
            policy_fit: Score::new(700),
            verification_readiness: Score::new(700),
            risk: Score::new(300),
            uncertainty: Score::new(300),
            staleness_penalty: Score::new(100),
            source_quality: Score::new(700),
            context_quality: Score::new(700),
        };

        let domain_value = domain_value_score(inputs);
        let actionability = actionability_score(domain_value, inputs);

        assert_eq!(domain_value.get(), 265);
        assert_eq!(actionability.get(), 129);
        assert_eq!(
            verdict_for_scores(DomainId::Unknown, inputs),
            DomainVerdict::Research
        );
    }

    #[test]
    fn verdict_act_business_thresholds() {
        let inputs = DomainScoreInputs {
            opportunity: Score::new(880),
            confidence: Score::new(760),
            policy_fit: Score::new(850),
            verification_readiness: Score::new(820),
            risk: Score::new(260),
            uncertainty: Score::new(250),
            staleness_penalty: Score::new(80),
            source_quality: Score::new(740),
            context_quality: Score::new(790),
        };

        let domain_value = domain_value_score(inputs);
        let actionability = actionability_score(domain_value, inputs);

        assert_eq!(domain_value.get(), 459);
        assert_eq!(actionability.get(), 267);
        assert_eq!(
            verdict_for_scores(DomainId::Business, inputs),
            DomainVerdict::ActBusiness
        );
    }

    #[test]
    fn verdict_act_finance_research_thresholds() {
        let inputs = DomainScoreInputs {
            opportunity: Score::new(700),
            confidence: Score::new(680),
            policy_fit: Score::new(620),
            verification_readiness: Score::new(740),
            risk: Score::new(520),
            uncertainty: Score::new(430),
            staleness_penalty: Score::new(190),
            source_quality: Score::new(720),
            context_quality: Score::new(680),
        };

        let domain_value = domain_value_score(inputs);
        let actionability = actionability_score(domain_value, inputs);

        assert_eq!(domain_value.get(), 176);
        assert_eq!(actionability.get(), 85);
        assert_eq!(
            verdict_for_scores(DomainId::Finance, inputs),
            DomainVerdict::ActFinanceResearch
        );
    }

    #[test]
    fn verdict_simulate_trading_thresholds() {
        let inputs = DomainScoreInputs {
            opportunity: Score::new(610),
            confidence: Score::new(560),
            policy_fit: Score::new(900),
            verification_readiness: Score::new(760),
            risk: Score::new(430),
            uncertainty: Score::new(470),
            staleness_penalty: Score::new(140),
            source_quality: Score::new(690),
            context_quality: Score::new(620),
        };

        let domain_value = domain_value_score(inputs);
        let actionability = actionability_score(domain_value, inputs);

        assert_eq!(domain_value.get(), 204);
        assert_eq!(actionability.get(), 86);
        assert_eq!(
            verdict_for_scores(DomainId::TradingSandbox, inputs),
            DomainVerdict::SimulateTrading
        );
    }

    #[test]
    fn verdict_block_thresholds() {
        let inputs = DomainScoreInputs {
            opportunity: Score::new(800),
            confidence: Score::new(700),
            policy_fit: Score::new(0),
            verification_readiness: Score::new(500),
            risk: Score::new(900),
            uncertainty: Score::new(800),
            staleness_penalty: Score::new(300),
            source_quality: Score::new(650),
            context_quality: Score::new(620),
        };

        let domain_value = domain_value_score(inputs);
        let actionability = actionability_score(domain_value, inputs);

        assert_eq!(domain_value.get(), 0);
        assert_eq!(actionability.get(), 0);
        assert_eq!(
            verdict_for_scores(DomainId::TradingSandbox, inputs),
            DomainVerdict::Block
        );
    }

    #[test]
    fn verdict_ignore_thresholds() {
        let ignore_inputs = DomainScoreInputs {
            opportunity: Score::new(300),
            confidence: Score::new(450),
            policy_fit: Score::new(500),
            verification_readiness: Score::new(500),
            risk: Score::new(450),
            uncertainty: Score::new(450),
            staleness_penalty: Score::new(200),
            source_quality: Score::new(700),
            context_quality: Score::new(700),
        };

        let ignore_value = domain_value_score(ignore_inputs);
        let ignore_actionability = actionability_score(ignore_value, ignore_inputs);

        assert_eq!(ignore_value.get(), 0);
        assert_eq!(ignore_actionability.get(), 0);
        assert_eq!(
            verdict_for_scores(DomainId::Unknown, ignore_inputs),
            DomainVerdict::Ignore
        );
    }
}
