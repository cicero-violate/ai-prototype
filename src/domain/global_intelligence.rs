//! Pure global-intelligence domain helpers.
//!
//! This module is descriptor-only. It performs deterministic classification and
//! scoring over already-captured domain records; it has no I/O, process,
//! network, runtime, command-ledger, or TLog mutation authority.

use super::contracts::{DomainHorizon, DomainSignal, DomainSignalClass};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignalClass {
    Unknown,
    MacroTrend,
    Regulation,
    TechnologyShift,
    RiskAlert,
    CapabilityLearning,
}

impl From<DomainSignalClass> for SignalClass {
    fn from(value: DomainSignalClass) -> Self {
        match value {
            DomainSignalClass::MacroTrend => Self::MacroTrend,
            DomainSignalClass::Regulation => Self::Regulation,
            DomainSignalClass::TechnologyShift => Self::TechnologyShift,
            DomainSignalClass::RiskAlert => Self::RiskAlert,
            DomainSignalClass::CapabilityLearning => Self::CapabilityLearning,
            _ => Self::Unknown,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlobalSignalProfile {
    pub signal_id: String,
    pub class: SignalClass,
    pub horizon: DomainHorizon,
    pub source_quality_score: u16,
    pub freshness_score: u16,
    pub contradiction_score: u16,
}

impl GlobalSignalProfile {
    pub fn from_signal(signal: &DomainSignal) -> Self {
        Self {
            signal_id: signal.signal_id.clone(),
            class: SignalClass::from(signal.signal_class),
            horizon: signal.horizon,
            source_quality_score: signal.source_quality_score.min(1000),
            freshness_score: signal.freshness_score.min(1000),
            contradiction_score: signal.contradiction_score.min(1000),
        }
    }
}

pub fn stale_for_horizon(profile: &GlobalSignalProfile) -> bool {
    let freshness_floor = match profile.horizon {
        DomainHorizon::Immediate => 750,
        DomainHorizon::Tactical => 550,
        DomainHorizon::Strategic => 350,
        DomainHorizon::Secular => 200,
    };

    profile.freshness_score < freshness_floor || profile.contradiction_score >= 850
}

pub fn actionability_hint(profile: &GlobalSignalProfile) -> u16 {
    if stale_for_horizon(profile) {
        return 0;
    }

    let horizon_weight = match profile.horizon {
        DomainHorizon::Immediate => 4u32,
        DomainHorizon::Tactical => 5,
        DomainHorizon::Strategic => 3,
        DomainHorizon::Secular => 2,
    };
    let class_weight = match profile.class {
        SignalClass::MacroTrend => 5u32,
        SignalClass::Regulation => 5,
        SignalClass::TechnologyShift => 4,
        SignalClass::RiskAlert => 3,
        SignalClass::CapabilityLearning => 3,
        SignalClass::Unknown => 1,
    };

    let base = u32::from(profile.source_quality_score)
        .saturating_mul(u32::from(profile.freshness_score))
        / 1000;
    let weighted = base.saturating_mul(horizon_weight).saturating_mul(class_weight) / 25;
    let contradiction_penalty = u32::from(profile.contradiction_score) / 2;

    weighted.saturating_sub(contradiction_penalty).min(1000) as u16
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::contracts::{DomainId, DomainSignal};

    fn profile() -> GlobalSignalProfile {
        GlobalSignalProfile {
            signal_id: "global-signal-1".to_string(),
            class: SignalClass::MacroTrend,
            horizon: DomainHorizon::Tactical,
            source_quality_score: 820,
            freshness_score: 700,
            contradiction_score: 180,
        }
    }

    #[test]
    fn global_signal_profile_from_domain_signal_is_deterministic() {
        let signal = DomainSignal::new(
            "global-signal-1",
            DomainId::GlobalIntelligence,
            "source-1",
            "2026-05-11T00:00:00Z",
            DomainHorizon::Tactical,
            DomainSignalClass::MacroTrend,
            "hash:payload",
            "hash:provenance",
            820,
            700,
            180,
        )
        .expect("signal constructor accepts bounded global signal inputs");

        assert_eq!(GlobalSignalProfile::from_signal(&signal), profile());
    }

    #[test]
    fn global_signal_profile_helpers_are_deterministic() {
        let profile = profile();

        assert!(!stale_for_horizon(&profile));
        assert_eq!(stale_for_horizon(&profile), stale_for_horizon(&profile));
        assert_eq!(actionability_hint(&profile), 484);
        assert_eq!(actionability_hint(&profile), actionability_hint(&profile));
    }

    #[test]
    fn global_signal_profile_staleness_is_deterministic() {
        let profile = profile();

        let first = stale_for_horizon(&profile);
        let second = stale_for_horizon(&profile);
        let third = stale_for_horizon(&profile);

        assert!(!first);
        assert_eq!(first, second);
        assert_eq!(second, third);
    }

    #[test]
    fn global_signal_actionability_hint_is_deterministic() {
        let profile = profile();

        let first = actionability_hint(&profile);
        let second = actionability_hint(&profile);
        let third = actionability_hint(&profile);

        assert_eq!(first, 484);
        assert_eq!(first, second);
        assert_eq!(second, third);
    }
}