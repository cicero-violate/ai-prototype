//! MIR extraction for canonical behavior facts.

use crate::graph::GraphEdge;
use rustc_middle::mir;
use rustc_middle::ty::{TyCtxt, TyKind};
use std::collections::{BTreeMap, BTreeSet};

pub struct MirOutput {
    pub edges: Vec<GraphEdge>,
}

pub fn collect_mir(tcx: TyCtxt<'_>) -> MirOutput {
    let idx = crate::index::build_index(tcx);
    let mut edges = Vec::new();
    let mut profiles = Vec::new();

    for def_id in &idx.def_ids {
        let Some(local_def_id) = def_id.as_local() else {
            continue;
        };
        if !tcx.is_mir_available(local_def_id) {
            continue;
        }

        let owner = tcx.def_path_str(*def_id);
        let body = match tcx.hir_body_const_context(local_def_id) {
            Some(rustc_hir::ConstContext::ConstFn)
            | Some(rustc_hir::ConstContext::Const { .. })
            | Some(rustc_hir::ConstContext::Static(_)) => tcx.mir_for_ctfe(local_def_id),
            None => tcx.optimized_mir(local_def_id),
        };

        let profile = collect_body_facts(tcx, &owner, body, &mut edges);
        profiles.push(profile);
    }

    emit_similarity_edges(&profiles, &mut edges);

    MirOutput {
        edges: dedup_edges(edges),
    }
}

fn collect_body_facts<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: &str,
    body: &mir::Body<'tcx>,
    edges: &mut Vec<GraphEdge>,
) -> BodyProfile {
    let mut profile = BodyProfile::new(owner);
    if body.arg_count > 0 {
        profile.phases.insert("parse");
    }

    for bb in body.basic_blocks.iter() {
        if bb.is_cleanup {
            continue;
        }
        for stmt in &bb.statements {
            collect_statement_facts(owner, stmt, edges, &mut profile);
        }
        let Some(term) = &bb.terminator else {
            continue;
        };
        match &term.kind {
            mir::TerminatorKind::Assert { .. } => {
                emit(edges, "panic", owner, "fact::panic");
            }
            mir::TerminatorKind::Call { func, .. } | mir::TerminatorKind::TailCall { func, .. } => {
                if let Some(callee) = resolve_fn_operand(tcx, func) {
                    profile.callees.insert(callee.clone());
                    if is_validation_callee(&callee) {
                        profile.phases.insert("validate");
                    }
                    emit(edges, "call", owner, callee.clone());
                    for relation in crate::facts::callee_relations(&callee) {
                        emit(edges, relation, owner, format!("fact::{relation}"));
                    }
                }
            }
            mir::TerminatorKind::InlineAsm { .. } => {
                emit(edges, "unsafe", owner, "fact::unsafe");
            }
            _ => {}
        }
    }

    for phase in &profile.phases {
        emit(edges, "phase", owner, format!("phase::{phase}"));
    }
    profile
}

fn collect_statement_facts(
    owner: &str,
    stmt: &mir::Statement<'_>,
    edges: &mut Vec<GraphEdge>,
    profile: &mut BodyProfile,
) {
    match &stmt.kind {
        mir::StatementKind::Assign(assign) => {
            profile.phases.insert("transform");
            emit(edges, "mut", owner, "fact::mut");
            let (_, rvalue) = &**assign;
            collect_rvalue_facts(owner, rvalue, edges);
        }
        mir::StatementKind::SetDiscriminant { .. } | mir::StatementKind::Retag(_, _) => {
            profile.phases.insert("transform");
            emit(edges, "mut", owner, "fact::mut");
        }
        _ => {}
    }
}

fn collect_rvalue_facts(owner: &str, rvalue: &mir::Rvalue<'_>, edges: &mut Vec<GraphEdge>) {
    match rvalue {
        mir::Rvalue::RawPtr(_, _)
        | mir::Rvalue::ThreadLocalRef(_)
        | mir::Rvalue::WrapUnsafeBinder(_, _) => {
            emit(edges, "unsafe", owner, "fact::unsafe");
        }
        mir::Rvalue::Aggregate(kind, _) => {
            if format!("{kind:?}").contains("Box") {
                emit(edges, "alloc", owner, "fact::alloc");
            }
        }
        _ => {}
    }
}

#[derive(Debug, Clone)]
struct BodyProfile {
    owner: String,
    module: String,
    callees: BTreeSet<String>,
    phases: BTreeSet<&'static str>,
}

impl BodyProfile {
    fn new(owner: &str) -> Self {
        Self {
            owner: owner.to_string(),
            module: owner
                .rsplit_once("::")
                .map(|(module, _)| module)
                .unwrap_or(owner)
                .to_string(),
            callees: BTreeSet::new(),
            phases: BTreeSet::new(),
        }
    }
}

fn is_validation_callee(callee: &str) -> bool {
    let name = callee.rsplit("::").next().unwrap_or(callee);
    name.contains("is_valid") || name.contains("is_contract_valid") || name.contains("validate")
}

fn emit_similarity_edges(profiles: &[BodyProfile], edges: &mut Vec<GraphEdge>) {
    let by_module = profiles.iter().fold(
        BTreeMap::<&str, Vec<&BodyProfile>>::new(),
        |mut acc, profile| {
            if !profile.callees.is_empty() {
                acc.entry(&profile.module).or_default().push(profile);
            }
            acc
        },
    );

    for group in by_module.values() {
        for i in 0..group.len() {
            for j in (i + 1)..group.len() {
                let a = group[i];
                let b = group[j];
                let score = jaccard(&a.callees, &b.callees);
                if score >= 0.75 {
                    emit(
                        edges,
                        "similar",
                        &a.owner,
                        format!("similar::{score:.2}::{}", b.owner),
                    );
                    emit(
                        edges,
                        "similar",
                        &b.owner,
                        format!("similar::{score:.2}::{}", a.owner),
                    );
                }
            }
        }
    }
}

fn jaccard(a: &BTreeSet<String>, b: &BTreeSet<String>) -> f64 {
    let union = a.union(b).count();
    if union == 0 {
        return 0.0;
    }
    a.intersection(b).count() as f64 / union as f64
}

fn resolve_fn_operand<'tcx>(tcx: TyCtxt<'tcx>, func: &mir::Operand<'tcx>) -> Option<String> {
    let mir::Operand::Constant(boxed) = func else {
        return None;
    };
    let (mir::Const::Val(_, ty) | mir::Const::Ty(ty, _)) = boxed.const_ else {
        return None;
    };
    if let TyKind::FnDef(def_id, _) = ty.kind() {
        return Some(tcx.def_path_str(*def_id));
    }
    None
}

fn emit(
    edges: &mut Vec<GraphEdge>,
    relation: &str,
    from: impl Into<String>,
    to: impl Into<String>,
) {
    edges.push(GraphEdge {
        relation: relation.to_string(),
        from: from.into(),
        to: to.into(),
        span: None,
    });
}

fn dedup_edges(edges: Vec<GraphEdge>) -> Vec<GraphEdge> {
    let set: BTreeSet<GraphEdge> = edges.into_iter().collect();
    set.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphEdge;

    fn profile(owner: &str, callees: &[&str]) -> BodyProfile {
        let mut profile = BodyProfile::new(owner);
        profile.callees = callees.iter().map(|callee| (*callee).to_string()).collect();
        profile
    }

    #[test]
    fn validation_callee_names_are_phase_hints_only() {
        assert!(is_validation_callee("demo::is_valid"));
        assert!(is_validation_callee("demo::is_contract_valid"));
        assert!(is_validation_callee("demo::validate_receipt"));
        assert!(!is_validation_callee("demo::parse_receipt"));
    }

    #[test]
    fn jaccard_similarity_is_deterministic() {
        let a = ["a", "b", "c"].into_iter().map(String::from).collect();
        let b = ["b", "c", "d"].into_iter().map(String::from).collect();
        assert_eq!(format!("{:.2}", jaccard(&a, &b)), "0.50");
    }

    #[test]
    fn similarity_edges_are_same_module_bounded_and_deduplicated() {
        let profiles = vec![
            profile("demo::m::left", &["a", "b", "c", "d"]),
            profile("demo::m::right", &["a", "b", "c", "d"]),
            profile("demo::other::right", &["a", "b", "c", "d"]),
        ];
        let mut edges = Vec::<GraphEdge>::new();
        emit_similarity_edges(&profiles, &mut edges);
        let edges = dedup_edges(edges);
        assert_eq!(edges.len(), 2);
        assert!(edges.iter().any(|edge| edge.relation == "similar"
            && edge.from == "demo::m::left"
            && edge.to == "similar::1.00::demo::m::right"));
        assert!(edges.iter().any(|edge| edge.relation == "similar"
            && edge.from == "demo::m::right"
            && edge.to == "similar::1.00::demo::m::left"));
        assert!(!edges
            .iter()
            .any(|edge| edge.from.contains("other") || edge.to.contains("other")));
    }
}
