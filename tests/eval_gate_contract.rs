use ai::{Decision, EvalGate, GateInput, Signal};

#[test]
fn verified_requires_minimum_evidence() {
    let gate = EvalGate::new(2);
    assert_eq!(gate.decide(GateInput::verified(2)), Decision::Allow);
    assert_eq!(gate.decide(GateInput::verified(1)), Decision::Deny);
}

#[test]
fn missing_evidence_uses_repair_budget_before_denying() {
    let gate = EvalGate::new(1);
    assert_eq!(gate.decide(GateInput { signal: Signal::MissingEvidence, evidence_count: 0, repair_budget: 1 }), Decision::Repair);
    assert_eq!(gate.decide(GateInput { signal: Signal::MissingEvidence, evidence_count: 0, repair_budget: 0 }), Decision::Deny);
}