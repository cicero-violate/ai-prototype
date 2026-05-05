//! MIR extraction for canonical behavior facts.

use crate::graph::GraphEdge;
use rustc_middle::mir;
use rustc_middle::ty::{TyCtxt, TyKind};
use std::collections::BTreeSet;

pub struct MirOutput {
    pub edges: Vec<GraphEdge>,
}

pub fn collect_mir(tcx: TyCtxt<'_>) -> MirOutput {
    let idx = crate::index::build_index(tcx);
    let mut edges = Vec::new();

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

        collect_body_facts(tcx, &owner, body, &mut edges);
    }

    MirOutput {
        edges: dedup_edges(edges),
    }
}

fn collect_body_facts<'tcx>(
    tcx: TyCtxt<'tcx>,
    owner: &str,
    body: &mir::Body<'tcx>,
    edges: &mut Vec<GraphEdge>,
) {
    for bb in body.basic_blocks.iter() {
        if bb.is_cleanup {
            continue;
        }
        for stmt in &bb.statements {
            collect_statement_facts(owner, stmt, edges);
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
}

fn collect_statement_facts(owner: &str, stmt: &mir::Statement<'_>, edges: &mut Vec<GraphEdge>) {
    match &stmt.kind {
        mir::StatementKind::Assign(assign) => {
            emit(edges, "mut", owner, "fact::mut");
            let (_, rvalue) = &**assign;
            collect_rvalue_facts(owner, rvalue, edges);
        }
        mir::StatementKind::SetDiscriminant { .. } | mir::StatementKind::Retag(_, _) => {
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
    });
}

fn dedup_edges(edges: Vec<GraphEdge>) -> Vec<GraphEdge> {
    let set: BTreeSet<GraphEdge> = edges.into_iter().collect();
    set.into_iter().collect()
}
