use ai::domain::contracts::{
    DomainBridgeTarget, DomainHorizon, DomainId, DomainJudgment, DomainLiveEffectLevel,
    DomainPlan, DomainPlanKind, DomainRiskEnvelope, DomainSignal, DomainSignalClass,
    DomainSourceKind, DomainVerdict, DOMAIN_SCHEMA_VERSION,
};
use ai::domain::identity::domain_hash_json;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
struct DomainFixture {
    schema_version: String,
    fixture_id: String,
    domain_id: String,
    source_kind: String,
    source_id: String,
    provenance_hash: String,
    payload_hash: String,
    horizon: String,
    signal_class: String,
    score_inputs: FixtureScoreInputs,
    expected_domain_value_score: u16,
    expected_actionability_score: u16,
    expected_verdict: String,
    expected_risk_envelope: FixtureRiskEnvelope,
    expected_bridge_target: String,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct FixtureScoreInputs {
    opportunity: u16,
    confidence: u16,
    policy_fit: u16,
    verification_readiness: u16,
    risk: u16,
    uncertainty: u16,
    staleness_penalty: u16,
    source_quality: u16,
    context_quality: u16,
}

#[derive(Debug, Deserialize)]
struct FixtureRiskEnvelope {
    requires_verification: bool,
    sandbox_only: bool,
    max_live_effect_level: String,
}

fn fixture_path(name: &str) -> String {
    format!("tests/fixtures/domain/{name}")
}

fn load_fixture(name: &str) -> DomainFixture {
    let raw = std::fs::read_to_string(fixture_path(name)).expect("fixture file reads");
    serde_json::from_str(&raw).expect("fixture JSON deserializes")
}

fn domain_id(value: &str) -> DomainId {
    match value {
        "GlobalIntelligence" => DomainId::GlobalIntelligence,
        "Business" => DomainId::Business,
        "Finance" => DomainId::Finance,
        "TradingSandbox" => DomainId::TradingSandbox,
        _ => panic!("unsupported fixture domain id: {value}"),
    }
}

fn source_kind(value: &str) -> DomainSourceKind {
    match value {
        "PrimarySource" => DomainSourceKind::WorldSignal,
        "CustomerDiscovery" => DomainSourceKind::CustomerFeedback,
        "PrimaryAndSecondaryResearch" | "HistoricalDataset" => DomainSourceKind::MarketData,
        _ => panic!("unsupported fixture source kind: {value}"),
    }
}

fn horizon(value: &str) -> DomainHorizon {
    match value {
        "Immediate" => DomainHorizon::Immediate,
        "Tactical" => DomainHorizon::Tactical,
        "Strategic" => DomainHorizon::Strategic,
        "Secular" => DomainHorizon::Secular,
        _ => panic!("unsupported fixture horizon: {value}"),
    }
}

fn signal_class(value: &str) -> DomainSignalClass {
    match value {
        "Macro" => DomainSignalClass::MacroTrend,
        "WorkflowBottleneck" => DomainSignalClass::BusinessWorkflow,
        "AllocationHypothesis" => DomainSignalClass::FinanceResearch,
        "PaperTradeHypothesis" => DomainSignalClass::TradingSimulation,
        _ => panic!("unsupported fixture signal class: {value}"),
    }
}

fn verdict(value: &str) -> DomainVerdict {
    match value {
        "Ignore" => DomainVerdict::Ignore,
        "Watch" => DomainVerdict::Watch,
        "Research" => DomainVerdict::Research,
        "ActBusiness" => DomainVerdict::ActBusiness,
        "ActFinanceResearch" => DomainVerdict::ActFinanceResearch,
        "SimulateTrading" => DomainVerdict::SimulateTrading,
        "Block" => DomainVerdict::Block,
        _ => panic!("unsupported fixture verdict: {value}"),
    }
}

fn bridge_target(value: &str) -> DomainBridgeTarget {
    match value {
        "ObservationRecord" => DomainBridgeTarget::ObservationRecord,
        "ContextRecord" => DomainBridgeTarget::ContextRecord,
        "JudgmentRecord" => DomainBridgeTarget::JudgmentRecord,
        "PlanRecord" => DomainBridgeTarget::PlanRecord,
        "VerificationRecord" => DomainBridgeTarget::VerificationRecord,
        "EvalRecord" => DomainBridgeTarget::EvalRecord,
        "PolicyPromotion" => DomainBridgeTarget::PolicyPromotion,
        "Blocked" => DomainBridgeTarget::Blocked,
        _ => panic!("unsupported fixture bridge target: {value}"),
    }
}

fn live_effect_level(value: &str) -> DomainLiveEffectLevel {
    match value {
        "None" => DomainLiveEffectLevel::None,
        "ResearchOnly" => DomainLiveEffectLevel::ReadOnly,
        "SandboxSimulationOnly" => DomainLiveEffectLevel::SandboxWrite,
        "BusinessWorkflowProposal" => DomainLiveEffectLevel::ExternalWrite,
        _ => panic!("unsupported fixture live effect level: {value}"),
    }
}

fn plan_kind(domain_id: DomainId, verdict: DomainVerdict) -> DomainPlanKind {
    match (domain_id, verdict) {
        (DomainId::Business, DomainVerdict::ActBusiness) => DomainPlanKind::BusinessWorkflowPlan,
        (DomainId::Finance, DomainVerdict::ActFinanceResearch) => DomainPlanKind::FinanceAnalysisPlan,
        (DomainId::TradingSandbox, DomainVerdict::SimulateTrading | DomainVerdict::Block) => {
            DomainPlanKind::TradingSimulationPlan
        }
        _ => DomainPlanKind::ResearchPlan,
    }
}

fn assert_bounded_scores(inputs: FixtureScoreInputs) {
    let values = [
        inputs.opportunity,
        inputs.confidence,
        inputs.policy_fit,
        inputs.verification_readiness,
        inputs.risk,
        inputs.uncertainty,
        inputs.staleness_penalty,
        inputs.source_quality,
        inputs.context_quality,
    ];

    for value in values {
        assert!(value <= 1000, "fixture score is bounded");
    }
}

fn assert_fixture_maps_to_contract_records(fixture: DomainFixture) {
    assert_eq!(fixture.schema_version, "canon_domain_fixture_v1");
    assert_bounded_scores(fixture.score_inputs);

    let domain_id = domain_id(&fixture.domain_id);
    let horizon = horizon(&fixture.horizon);
    let signal_class = signal_class(&fixture.signal_class);
    let verdict = verdict(&fixture.expected_verdict);
    let bridge_target = bridge_target(&fixture.expected_bridge_target);
    let max_live_effect_level = live_effect_level(&fixture.expected_risk_envelope.max_live_effect_level);

    let signal = DomainSignal::new(
        format!("{}:signal", fixture.fixture_id),
        domain_id,
        fixture.source_id.clone(),
        "2026-05-11T00:00:00Z",
        horizon,
        signal_class,
        fixture.payload_hash.clone(),
        fixture.provenance_hash.clone(),
        fixture.score_inputs.source_quality,
        1000u16.saturating_sub(fixture.score_inputs.staleness_penalty),
        fixture.score_inputs.uncertainty,
    )
    .expect("fixture maps to DomainSignal");

    assert_eq!(signal.schema_version, DOMAIN_SCHEMA_VERSION);
    assert_eq!(signal.domain_id, domain_id);
    assert_eq!(signal.horizon, horizon);
    assert_eq!(signal.signal_class, signal_class);

    let judgment = DomainJudgment::new(
        format!("{}:judgment", fixture.fixture_id),
        domain_id,
        fixture.score_inputs.opportunity,
        fixture.score_inputs.risk,
        fixture.score_inputs.confidence,
        fixture.score_inputs.uncertainty,
        fixture.expected_actionability_score,
        fixture.score_inputs.policy_fit,
        fixture.expected_domain_value_score,
        verdict,
        format!("{}:rationale", fixture.payload_hash),
    )
    .expect("fixture maps to DomainJudgment");

    assert_eq!(judgment.schema_version, DOMAIN_SCHEMA_VERSION);
    assert_eq!(judgment.verdict, verdict);

    let envelope = DomainRiskEnvelope::new(
        format!("{}:risk", fixture.fixture_id),
        domain_id,
        fixture.score_inputs.uncertainty,
        fixture.score_inputs.confidence,
        fixture.score_inputs.staleness_penalty,
        max_live_effect_level,
        true,
        fixture.expected_risk_envelope.requires_verification,
        fixture.expected_risk_envelope.sandbox_only,
    )
    .expect("fixture maps to DomainRiskEnvelope");

    assert_eq!(envelope.schema_version, DOMAIN_SCHEMA_VERSION);
    assert_eq!(envelope.max_live_effect_level, max_live_effect_level);

    let plan = DomainPlan::new(
        format!("{}:plan", fixture.fixture_id),
        domain_id,
        format!("{}:judgment-hash", fixture.payload_hash),
        plan_kind(domain_id, verdict),
        bridge_target,
        format!("{}:capabilities", fixture.provenance_hash),
        format!("{}:receipts", fixture.provenance_hash),
        format!("{}:risk", fixture.provenance_hash),
        format!("{}:success", fixture.provenance_hash),
        format!("{}:rollback", fixture.provenance_hash),
        max_live_effect_level,
    )
    .expect("fixture maps to DomainPlan");

    assert_eq!(plan.schema_version, DOMAIN_SCHEMA_VERSION);
    assert_eq!(plan.bridge_target, bridge_target);

    let fixture_source_kind = source_kind(&fixture.source_kind);
    assert_ne!(fixture_source_kind, DomainSourceKind::Unknown);
}

#[test]
fn domain_records_deserialize_from_json() {
    for fixture_name in [
        "global_signal_macro.json",
        "business_workflow_opportunity.json",
        "finance_hypothesis_research.json",
        "trading_simulation_sandbox.json",
        "trading_live_blocked.json",
    ] {
        assert_fixture_maps_to_contract_records(load_fixture(fixture_name));
    }
}

#[test]
fn domain_identity_is_deterministic() {
    for fixture_name in [
        "global_signal_macro.json",
        "business_workflow_opportunity.json",
        "finance_hypothesis_research.json",
        "trading_simulation_sandbox.json",
        "trading_live_blocked.json",
    ] {
        let fixture = load_fixture(fixture_name);
        let material_record = json!({
            "schema_version": fixture.schema_version,
            "fixture_id": fixture.fixture_id,
            "domain_id": fixture.domain_id,
            "source_kind": fixture.source_kind,
            "source_id": fixture.source_id,
            "provenance_hash": fixture.provenance_hash,
            "payload_hash": fixture.payload_hash,
            "horizon": fixture.horizon,
            "signal_class": fixture.signal_class,
            "score_inputs": {
                "opportunity": fixture.score_inputs.opportunity,
                "confidence": fixture.score_inputs.confidence,
                "policy_fit": fixture.score_inputs.policy_fit,
                "verification_readiness": fixture.score_inputs.verification_readiness,
                "risk": fixture.score_inputs.risk,
                "uncertainty": fixture.score_inputs.uncertainty,
                "staleness_penalty": fixture.score_inputs.staleness_penalty,
                "source_quality": fixture.score_inputs.source_quality,
                "context_quality": fixture.score_inputs.context_quality,
            },
            "expected_verdict": fixture.expected_verdict,
            "expected_bridge_target": fixture.expected_bridge_target,
        });

        let repeated_hash = domain_hash_json(&material_record);
        assert_eq!(domain_hash_json(&material_record), repeated_hash);

        let mut changed_record = material_record.clone();
        changed_record["payload_hash"] = json!(format!("{}:changed", changed_record["payload_hash"].as_str().unwrap()));
        assert_ne!(domain_hash_json(&changed_record), repeated_hash);
    }
}
