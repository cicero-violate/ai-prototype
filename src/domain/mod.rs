//! Domain architecture sketch.
//!
//! This module sketches the pure domain layer that should sit above raw world
//! inputs and below existing capability evidence production.
//!
//! Boundary:
//! - domain: finance/global-intelligence/business/trading interpretation
//! - capability: evidence records, receipts, tools, verification
//! - kernel: frozen state machine
//! - runtime: deterministic transitions and replay
//!
//! Non-goals:
//! - no direct `State` mutation
//! - no direct `TLog` append
//! - no changes to `kernel`
//! - no changes to `runtime::reduce`
//! - no live trading path

pub mod bridge;
pub mod contracts;
pub mod identity;
pub mod risk;
pub mod scoring;

pub use bridge::{bridge_target_for_verdict, default_plan_kind};
pub use contracts::{
    DomainBridgeTarget, DomainEval, DomainHorizon, DomainId, DomainJudgment, DomainLiveEffectLevel,
    DomainPlan, DomainPlanKind, DomainPromotionCandidate, DomainRiskClass, DomainRiskEnvelope,
    DomainSchemaVersion, DomainSignal, DomainSignalClass, DomainSourceKind, DomainVerdict, Horizon,
    LiveEffectLevel, PlanKind, DOMAIN_SCHEMA_VERSION,
};
pub use identity::stable_domain_id;
pub use risk::{evaluate_risk_envelope, DomainRiskDecision};
pub use scoring::{
    actionability_score, bounded_product, domain_value_score, DomainScoreInputs, Score,
};
