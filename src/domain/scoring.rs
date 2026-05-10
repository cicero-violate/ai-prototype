//! Deterministic bounded integer scoring for domain fixtures and records.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score(u16);

impl Score {
    pub const MIN: Score = Score(0);
    pub const MAX: Score = Score(1000);

    pub fn new(value: u16) -> Self {
        Self(value.min(1000))
    }

    pub fn get(self) -> u16 {
        self.0
    }
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
}
