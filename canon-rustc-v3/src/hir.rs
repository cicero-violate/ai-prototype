//! HIR extraction for canonical source-level facts.

use crate::graph::{GraphEdge, GraphNode, SourceSpan};
use crate::index::{build_index, def_id_key};
use rustc_hir::def::{DefKind, Res};
use rustc_hir::intravisit::{self, Visitor};
use rustc_middle::ty::TyCtxt;
use rustc_span::source_map::SourceMap;
use rustc_span::{FileName, Span};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::sync::Arc;

pub struct HirOutput {
    pub nodes: BTreeMap<String, GraphNode>,
    pub edges: Vec<GraphEdge>,
}

pub fn collect_hir(tcx: TyCtxt<'_>, workspace_root: &Path) -> HirOutput {
    let idx = build_index(tcx);
    let source_map = tcx.sess.source_map();
    let mut nodes = BTreeMap::new();
    let mut def_id_to_path = HashMap::new();

    for def_id in &idx.def_ids {
        let Some(kind) = allowed_node_kind(tcx.def_kind(*def_id)) else {
            continue;
        };
        let path = tcx.def_path_str(*def_id);
        def_id_to_path.insert(*def_id, path.clone());
        nodes.insert(
            path.clone(),
            GraphNode {
                def_id: def_id_key(*def_id),
                path,
                kind: kind.to_string(),
                def: to_source_span(source_map, tcx.def_span(*def_id), workspace_root),
            },
        );
    }

    let mut visitor = HirVisitor {
        tcx,
        def_id_to_path: &def_id_to_path,
        edges: Vec::new(),
        current_fn: None,
    };
    tcx.hir_visit_all_item_likes_in_crate(&mut visitor);

    HirOutput {
        nodes,
        edges: dedup_edges(visitor.edges),
    }
}

fn allowed_node_kind(kind: DefKind) -> Option<&'static str> {
    match kind {
        DefKind::Fn | DefKind::AssocFn => Some("fn"),
        DefKind::Trait => Some("trait"),
        DefKind::Impl { .. } => Some("impl"),
        _ => None,
    }
}

struct HirVisitor<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    def_id_to_path: &'a HashMap<rustc_hir::def_id::DefId, String>,
    edges: Vec<GraphEdge>,
    current_fn: Option<String>,
}

impl<'a, 'tcx> HirVisitor<'a, 'tcx> {
    fn resolve(&self, def_id: rustc_hir::def_id::DefId) -> String {
        self.def_id_to_path
            .get(&def_id)
            .cloned()
            .unwrap_or_else(|| self.tcx.def_path_str(def_id))
    }

    fn emit(&mut self, relation: &str, from: impl Into<String>, to: impl Into<String>) {
        self.edges.push(GraphEdge {
            relation: relation.to_string(),
            from: from.into(),
            to: to.into(),
        });
    }

    fn emit_call_facts(&mut self, callee: &str) {
        let Some(caller) = self.current_fn.clone() else {
            return;
        };
        self.emit("call", caller.clone(), callee.to_string());
        for relation in crate::facts::callee_relations(callee) {
            self.emit(relation, caller.clone(), format!("fact::{relation}"));
        }
    }

    fn enter_fn(&mut self, def_id: rustc_hir::def_id::DefId) -> Option<String> {
        let saved = self.current_fn.take();
        self.current_fn = Some(self.resolve(def_id));
        saved
    }

    fn leave_fn(&mut self, saved: Option<String>) {
        self.current_fn = saved;
    }
}

impl<'a, 'tcx> Visitor<'tcx> for HirVisitor<'a, 'tcx> {
    type NestedFilter = rustc_middle::hir::nested_filter::All;

    fn maybe_tcx(&mut self) -> Self::MaybeTyCtxt {
        self.tcx
    }

    fn visit_item(&mut self, item: &'tcx rustc_hir::Item<'tcx>) {
        if item.span.from_expansion() {
            intravisit::walk_item(self, item);
            return;
        }

        let def_id = item.owner_id.to_def_id();
        match &item.kind {
            rustc_hir::ItemKind::Impl(impl_item) => {
                if let Some(trait_ref) = &impl_item.of_trait {
                    if let Res::Def(_, trait_id) = trait_ref.trait_ref.path.res {
                        self.emit("impl", self.resolve(def_id), self.resolve(trait_id));
                    }
                }
            }
            rustc_hir::ItemKind::Fn { .. } => {
                let saved = self.enter_fn(def_id);
                intravisit::walk_item(self, item);
                self.leave_fn(saved);
                return;
            }
            _ => {}
        }

        intravisit::walk_item(self, item);
    }

    fn visit_impl_item(&mut self, item: &'tcx rustc_hir::ImplItem<'tcx>) {
        if item.span.from_expansion() {
            return;
        }
        let pushed = matches!(&item.kind, rustc_hir::ImplItemKind::Fn(..));
        let saved = if pushed {
            Some(self.enter_fn(item.owner_id.to_def_id()))
        } else {
            None
        };
        intravisit::walk_impl_item(self, item);
        if let Some(saved) = saved {
            self.leave_fn(saved);
        }
    }

    fn visit_trait_item(&mut self, item: &'tcx rustc_hir::TraitItem<'tcx>) {
        if item.span.from_expansion() {
            return;
        }
        let pushed = matches!(&item.kind, rustc_hir::TraitItemKind::Fn(..));
        let saved = if pushed {
            Some(self.enter_fn(item.owner_id.to_def_id()))
        } else {
            None
        };
        intravisit::walk_trait_item(self, item);
        if let Some(saved) = saved {
            self.leave_fn(saved);
        }
    }

    fn visit_expr(&mut self, expr: &'tcx rustc_hir::Expr<'tcx>) {
        let owner_def_id = expr.hir_id.owner.def_id;
        match &expr.kind {
            rustc_hir::ExprKind::Assign(..) | rustc_hir::ExprKind::AssignOp(..) => {
                if let Some(caller) = self.current_fn.clone() {
                    self.emit("mut", caller, "fact::mut");
                }
            }
            rustc_hir::ExprKind::Block(block, _) => {
                if matches!(block.rules, rustc_hir::BlockCheckMode::UnsafeBlock(_)) {
                    if let Some(caller) = self.current_fn.clone() {
                        self.emit("unsafe", caller, "fact::unsafe");
                    }
                }
            }
            rustc_hir::ExprKind::Call(func_expr, _) => {
                let callee_ty = self.tcx.typeck(owner_def_id).node_type(func_expr.hir_id);
                if let rustc_middle::ty::TyKind::FnDef(callee_id, _) = callee_ty.kind() {
                    let callee = self.tcx.def_path_str(*callee_id);
                    self.emit_call_facts(&callee);
                }
            }
            rustc_hir::ExprKind::MethodCall(seg, _, _, _) => {
                let hir_id = expr.hir_id;
                if let Some(callee_id) = self.tcx.typeck(owner_def_id).type_dependent_def_id(hir_id)
                {
                    let callee = self.resolve(callee_id);
                    self.emit_call_facts(&callee);
                    if seg.ident.as_str().contains("unwrap")
                        || seg.ident.as_str().contains("expect")
                    {
                        if let Some(caller) = self.current_fn.clone() {
                            self.emit("panic", caller, "fact::panic");
                        }
                    }
                }
            }
            _ => {}
        }
        intravisit::walk_expr(self, expr);
    }
}

fn dedup_edges(edges: Vec<GraphEdge>) -> Vec<GraphEdge> {
    let set: BTreeSet<GraphEdge> = edges.into_iter().collect();
    set.into_iter().collect()
}

fn to_source_span(source_map: &SourceMap, span: Span, workspace_root: &Path) -> Option<SourceSpan> {
    if span.from_expansion() {
        return None;
    }
    let lo = source_map.lookup_byte_offset(span.lo());
    let hi = source_map.lookup_byte_offset(span.hi());
    if !Arc::ptr_eq(&lo.sf, &hi.sf) {
        return None;
    }
    let FileName::Real(real_path) = &lo.sf.name else {
        return None;
    };
    let path = real_path.local_path()?.to_path_buf();
    let path = std::fs::canonicalize(&path).unwrap_or(path);
    let workspace_root =
        std::fs::canonicalize(workspace_root).unwrap_or_else(|_| workspace_root.to_path_buf());
    let path = path.strip_prefix(workspace_root).ok()?.to_path_buf();
    let loc = source_map.lookup_char_pos(span.lo());
    Some(SourceSpan {
        file: path.to_string_lossy().replace('\\', "/"),
        line: loc.line as u32,
        col: loc.col.0 as u32,
        lo: lo.pos.0,
        hi: hi.pos.0,
    })
}
