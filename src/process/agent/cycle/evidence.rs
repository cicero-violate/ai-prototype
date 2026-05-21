//! Evidence submission JSON and hash helpers for AgentCycle.

#[derive(Clone, Copy)]
struct U64Route {
    name: &'static str,
    value: u64,
}

#[derive(Clone, Copy)]
pub(super) enum U64RouteTable {
    GateId,
    EvidenceValue,
}

const GATE_ID_ROUTES: &[U64Route] = &[
    U64Route {
        name: "Invariant",
        value: 1,
    },
    U64Route {
        name: "Analysis",
        value: 2,
    },
    U64Route {
        name: "Judgment",
        value: 3,
    },
    U64Route {
        name: "Plan",
        value: 4,
    },
    U64Route {
        name: "Execution",
        value: 5,
    },
    U64Route {
        name: "Verification",
        value: 6,
    },
    U64Route {
        name: "Eval",
        value: 7,
    },
    U64Route {
        name: "Learning",
        value: 8,
    },
];

const EVIDENCE_U64_ROUTES: &[U64Route] = &[
    U64Route {
        name: "InvariantProof",
        value: 3,
    },
    U64Route {
        name: "AnalysisReport",
        value: 4,
    },
    U64Route {
        name: "JudgmentRecord",
        value: 5,
    },
    U64Route {
        name: "PlanRecord",
        value: 6,
    },
    U64Route {
        name: "TaskReady",
        value: 7,
    },
    U64Route {
        name: "ExecutionReceipt",
        value: 8,
    },
    U64Route {
        name: "ArtifactReceipt",
        value: 9,
    },
    U64Route {
        name: "VerificationReport",
        value: 10,
    },
    U64Route {
        name: "LineageProof",
        value: 11,
    },
    U64Route {
        name: "EvalScore",
        value: 12,
    },
    U64Route {
        name: "PersistedRecord",
        value: 16,
    },
    U64Route {
        name: "PolicyPromotion",
        value: 18,
    },
];

#[derive(Clone, Copy)]
struct EffectRoute {
    gate: &'static str,
    evidence: Option<&'static str>,
    effect_u64: u64,
    effect_json: &'static str,
}

const EFFECT_ROUTES: &[EffectRoute] = &[
    EffectRoute {
        gate: "Execution",
        evidence: Some("ExecutionReceipt"),
        effect_u64: 0,
        effect_json: "null",
    },
    EffectRoute {
        gate: "Plan",
        evidence: None,
        effect_u64: 1,
        effect_json: "\"BindReadyTask\"",
    },
    EffectRoute {
        gate: "Execution",
        evidence: None,
        effect_u64: 2,
        effect_json: "\"MaterializeArtifact\"",
    },
    EffectRoute {
        gate: "Verification",
        evidence: None,
        effect_u64: 3,
        effect_json: "\"RepairLineage\"",
    },
    EffectRoute {
        gate: "Eval",
        evidence: None,
        effect_u64: 4,
        effect_json: "\"CompleteObjective\"",
    },
];

pub(super) fn lookup_u64_route(table: U64RouteTable, name: &str) -> Option<u64> {
    let routes = match table {
        U64RouteTable::GateId => GATE_ID_ROUTES,
        U64RouteTable::EvidenceValue => EVIDENCE_U64_ROUTES,
    };

    routes
        .iter()
        .find(|route| route.name == name)
        .map(|route| route.value)
}

pub(super) fn gate_id_u64(gate: &str) -> Option<u64> {
    lookup_u64_route(U64RouteTable::GateId, gate)
}

pub(super) fn evidence_u64_value(evidence: &str) -> Option<u64> {
    lookup_u64_route(U64RouteTable::EvidenceValue, evidence)
}

// Returns (effect_u64, effect_json_str). PacketEffect repr: None=0, BindReadyTask=1,
// MaterializeArtifact=2, RepairLineage=3, CompleteObjective=4.
pub(super) fn effect_for_gate_evidence(
    gate: &str,
    evidence: &str,
    passed: bool,
) -> (u64, &'static str) {
    if !passed {
        return (0, "null");
    }
    EFFECT_ROUTES
        .iter()
        .find(|route| {
            route.gate == gate && route.evidence.is_none_or(|expected| expected == evidence)
        })
        .map(|route| (route.effect_u64, route.effect_json))
        .unwrap_or((0, "null"))
}

pub(super) fn compute_structural_payload_hash(
    gate_u64: u64,
    evidence_u64: u64,
    passed: bool,
    effect_u64: u64,
) -> u64 {
    let mut h = 0x8422_2325_cbf2_9ce4u64;
    h ^= gate_u64;
    h = h.wrapping_mul(0x100000001b3);
    h ^= evidence_u64;
    h = h.wrapping_mul(0x100000001b3);
    h ^= passed as u64;
    h = h.wrapping_mul(0x100000001b3);
    h ^= effect_u64;
    h = h.wrapping_mul(0x100000001b3);
    if h == 0 {
        1
    } else {
        h
    }
}

pub(super) fn compute_evidence_contract_hash(
    gate_u64: u64,
    evidence_u64: u64,
    passed: bool,
    effect_u64: u64,
    payload_hash: u64,
) -> u64 {
    compute_submit_evidence_wrapper_hash(
        HashDomain::EvidenceContract,
        &[
            gate_u64,
            evidence_u64,
            passed as u64,
            effect_u64,
            payload_hash,
        ],
    )
}

pub(super) fn compute_submit_evidence_command_hash(sub_contract_hash: u64) -> u64 {
    compute_submit_evidence_wrapper_hash(HashDomain::SubmitEvidenceCommand, &[sub_contract_hash])
}

pub(super) fn compute_envelope_hash(command_id: u64, cmd_contract_hash: u64) -> u64 {
    compute_submit_evidence_wrapper_hash(
        HashDomain::CommandEnvelope,
        &[command_id, cmd_contract_hash],
    )
}

pub(super) fn compute_submit_evidence_wrapper_hash(
    domain: HashDomain,
    ordered_fields: &[u64],
) -> u64 {
    compute_ordered_submit_evidence_hash(domain, ordered_fields)
}

pub(super) fn compute_ordered_submit_evidence_hash(
    domain: HashDomain,
    ordered_fields: &[u64],
) -> u64 {
    compute_hash_domain_vector(domain, ordered_fields)
}

pub(super) fn compute_hash_domain_vector(domain: HashDomain, fields: &[u64]) -> u64 {
    compute_domain_contract_hash(domain, fields)
}

#[derive(Clone, Copy)]
pub(super) enum HashDomain {
    EvidenceContract,
    SubmitEvidenceCommand,
    CommandEnvelope,
}

struct HashDomainDescriptor {
    seed: u64,
    prefix_fields: &'static [u64],
}

impl HashDomain {
    fn descriptor(self) -> HashDomainDescriptor {
        match self {
            Self::EvidenceContract => HashDomainDescriptor {
                seed: 0xcbf29ce484222325u64,
                prefix_fields: &[],
            },
            Self::SubmitEvidenceCommand => HashDomainDescriptor {
                seed: 0x9e3779b97f4a7c15u64,
                prefix_fields: &[1u64],
            },
            Self::CommandEnvelope => HashDomainDescriptor {
                seed: 0x517cc1b727220a95u64,
                prefix_fields: &[6u64],
            },
        }
    }
}

pub(super) fn compute_domain_contract_hash(domain: HashDomain, fields: &[u64]) -> u64 {
    let descriptor = domain.descriptor();
    let h = mix_contract_hash(descriptor.seed, descriptor.prefix_fields);
    mix_contract_hash(h, fields)
}

pub(super) fn mix_contract_hash(seed: u64, fields: &[u64]) -> u64 {
    let mut h = seed;
    for field in fields {
        h ^= *field;
        h = h.wrapping_mul(0x100000001b3);
    }
    h.max(1)
}

pub(super) fn build_submit_evidence_json(
    gate: &str,
    evidence: &str,
    passed: bool,
    command_id: u64,
) -> Option<String> {
    let gate_u64 = gate_id_u64(gate)?;
    let ev_u64 = evidence_u64_value(evidence)?;
    let (effect_u64, effect_json) = effect_for_gate_evidence(gate, evidence, passed);

    let payload_hash = compute_structural_payload_hash(gate_u64, ev_u64, passed, effect_u64);
    let sub_hash =
        compute_evidence_contract_hash(gate_u64, ev_u64, passed, effect_u64, payload_hash);
    let cmd_hash = compute_submit_evidence_command_hash(sub_hash);
    let command_hash = compute_envelope_hash(command_id, cmd_hash);

    Some(format!(
        r#"{{"command_id":{command_id},"command_hash":{command_hash},"payload_tag":"SubmitEvidence","payload":{{"gate":"{gate}","evidence":"{evidence}","passed":{passed},"effect":{effect_json},"payload_hash":{payload_hash}}}}}"#
    ))
}
