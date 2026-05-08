//! HIR extraction for canonical source-level facts.
//!
//! Gap 1 — Call-site spans: `call` edges now carry the source span of the
//!   call expression, enabling safe mechanical rename of any fn in the crate.
//!
//! Gap 2 — Struct / enum / type-alias nodes: `allowed_node_kind` extended;
//!   struct field definitions and enum variant names are captured.
//!
//! Gap 3 — Attribute spans: item spans are expanded backwards to cover all
//!   outer attributes via `tcx.hir().attrs(hir_id)`.
//!
//! Gap 4 — Use-statement tracking: `use` edges are emitted with source span
//!   so rename / move tools can update import paths.
//!
//! Gap 5 — Structured signatures: `sig` field populated for all `fn` nodes
//!   with parameter names, parameter types, and return type.

use crate::graph::{FieldDef, FnParam, FnSig, GraphEdge, GraphNode, SourceSpan};
use crate::index::{build_index, def_id_key};
use rustc_hir::attrs::AttributeKind;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::Attribute;
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
    let mut def_id_to_path: HashMap<rustc_hir::def_id::DefId, String> = HashMap::new();

    // Seed nodes with the narrow def_span.  The visitor upgrades each node it
    // visits to the attribute-extended full item span.
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
                source_text: None,
                sig: None,
                fields: vec![],
            },
        );
    }

    // Run the visitor inside a block so the mutable borrow of `nodes` ends
    // before we move `nodes` into HirOutput.
    let edges = {
        let mut visitor = HirVisitor {
            tcx,
            source_map,
            workspace_root,
            def_id_to_path: &def_id_to_path,
            nodes: &mut nodes,
            edges: Vec::new(),
            current_fn: None,
        };
        tcx.hir_visit_all_item_likes_in_crate(&mut visitor);
        dedup_edges(visitor.edges)
    };

    HirOutput { nodes, edges }
}

fn allowed_node_kind(kind: DefKind) -> Option<&'static str> {
    match kind {
        DefKind::Fn | DefKind::AssocFn => Some("fn"),
        DefKind::Trait => Some("trait"),
        DefKind::Impl { .. } => Some("impl"),
        DefKind::Struct => Some("struct"),
        DefKind::Enum => Some("enum"),
        DefKind::TyAlias => Some("ty_alias"),
        _ => None,
    }
}

// ─── visitor ─────────────────────────────────────────────────────────────────

struct HirVisitor<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    source_map: &'a SourceMap,
    workspace_root: &'a Path,
    def_id_to_path: &'a HashMap<rustc_hir::def_id::DefId, String>,
    nodes: &'a mut BTreeMap<String, GraphNode>,
    edges: Vec<GraphEdge>,
    current_fn: Option<String>,
}

impl<'a, 'tcx> HirVisitor<'a, 'tcx> {
    /// Return `item.span` extended backwards to cover all outer attributes
    /// (Gap 3).  Attributes precede the item keyword (`pub`, `fn`, etc.) in
    /// the source file and are absent from `item.span` itself.
    fn span_with_attrs(&self, hir_id: rustc_hir::HirId, item_span: Span) -> Span {
        let attrs = self.tcx.hir_attrs(hir_id);
        let attr_lo = attrs
            .iter()
            .filter_map(|attr| {
                let span = Self::attribute_source_span(attr)?;
                if span.from_expansion() {
                    None
                } else {
                    Some(span.lo())
                }
            })
            .min();
        match attr_lo {
            Some(lo) if lo < item_span.lo() => item_span.with_lo(lo),
            _ => item_span,
        }
    }

    fn attribute_source_span(attr: &Attribute) -> Option<Span> {
        match attr {
            Attribute::Unparsed(item) => Some(item.span),
            Attribute::Parsed(AttributeKind::DocComment { span, .. }) => Some(*span),
            Attribute::Parsed(AttributeKind::Deprecated { span, .. }) => Some(*span),
            Attribute::Parsed(AttributeKind::CfgTrace(cfgs)) => cfgs.first().map(|(_, span)| *span),
            Attribute::Parsed(_) => None,
        }
    }

    /// Upgrade a node's `def` span and populate `source_text` from the full
    /// HIR item span (attribute-extended).
    fn upgrade_span(&mut self, def_id: rustc_hir::def_id::DefId, full_span: Span) {
        let path = if let Some(p) = self.def_id_to_path.get(&def_id) {
            p.clone()
        } else {
            self.tcx.def_path_str(def_id)
        };
        if let Some(node) = self.nodes.get_mut(&path) {
            node.def = to_source_span(self.source_map, full_span, self.workspace_root);
            node.source_text = span_snippet(self.source_map, full_span);
        }
    }

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
            span: None,
        });
    }

    /// Emit a `call` edge with the source span of the call expression (Gap 1).
    fn emit_call(
        &mut self,
        caller: impl Into<String>,
        callee: impl Into<String>,
        call_span: Option<SourceSpan>,
    ) {
        self.edges.push(GraphEdge {
            relation: "call".to_string(),
            from: caller.into(),
            to: callee.into(),
            span: call_span,
        });
    }

    fn emit_call_facts(&mut self, callee: &str, call_span: Option<SourceSpan>) {
        let Some(caller) = self.current_fn.clone() else {
            return;
        };
        self.emit_call(caller.clone(), callee, call_span);
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

    fn emit_provider_edges_for_source(&mut self, def_id: rustc_hir::def_id::DefId, span: Span) {
        let Some(source) = span_snippet(self.source_map, span) else {
            return;
        };
        let path = self.resolve(def_id);
        if contains_any_provider_sentinel(
            &source,
            &[
                "OPENAI_COMPAT_PROVIDER",
                "OPENAI_JUDGMENT_PROOF_LINE",
                "openai_provider_hash",
            ],
        ) {
            self.emit("provider", path.clone(), "provider::openai");
        }
        if contains_any_provider_sentinel(
            &source,
            &[
                "OLLAMA_JUDGMENT_PROOF_LINE",
                "ollama_provider_hash",
                "OLLAMA_PROVIDER",
            ],
        ) {
            self.emit("provider", path, "provider::ollama");
        }
    }

    /// Extract a structured function signature from a HIR `FnDecl` and
    /// the corresponding body's parameter list (Gap 5).
    fn extract_fn_sig(
        &self,
        decl: &rustc_hir::FnDecl<'_>,
        body_id: rustc_hir::BodyId,
    ) -> Option<FnSig> {
        let body = self.tcx.hir_body(body_id);
        let params: Vec<FnParam> = decl
            .inputs
            .iter()
            .zip(body.params.iter())
            .map(|(ty, param)| {
                let name = match &param.pat.kind {
                    rustc_hir::PatKind::Binding(_, _, ident, _) => ident.as_str().to_string(),
                    _ => "_".to_string(),
                };
                let ty_str = span_snippet(self.source_map, ty.span).unwrap_or_default();
                FnParam { name, ty: ty_str }
            })
            .collect();
        let return_ty = match &decl.output {
            rustc_hir::FnRetTy::DefaultReturn(_) => "()".to_string(),
            rustc_hir::FnRetTy::Return(ty) => {
                span_snippet(self.source_map, ty.span).unwrap_or_default()
            }
        };
        Some(FnSig { params, return_ty })
    }

    /// Extract a structured function signature for abstract trait methods that
    /// have no body — parameter names come from the ident list (Gap 5).
    fn extract_fn_sig_required(
        &self,
        decl: &rustc_hir::FnDecl<'_>,
        param_names: &[Option<rustc_span::symbol::Ident>],
    ) -> Option<FnSig> {
        let params: Vec<FnParam> = decl
            .inputs
            .iter()
            .enumerate()
            .map(|(i, ty)| {
                let name = param_names
                    .get(i)
                    .and_then(|opt| opt.as_ref())
                    .map(|id| id.as_str().to_string())
                    .unwrap_or_else(|| format!("_{i}"));
                let ty_str = span_snippet(self.source_map, ty.span).unwrap_or_default();
                FnParam { name, ty: ty_str }
            })
            .collect();
        let return_ty = match &decl.output {
            rustc_hir::FnRetTy::DefaultReturn(_) => "()".to_string(),
            rustc_hir::FnRetTy::Return(ty) => {
                span_snippet(self.source_map, ty.span).unwrap_or_default()
            }
        };
        Some(FnSig { params, return_ty })
    }

    /// Set the `sig` field on a node identified by `def_id`.
    fn set_sig(&mut self, def_id: rustc_hir::def_id::DefId, sig: FnSig) {
        let path = self.resolve(def_id);
        if let Some(node) = self.nodes.get_mut(&path) {
            node.sig = Some(sig);
        }
    }

    /// Set the `fields` list on a node identified by `def_id`.
    fn set_fields(&mut self, def_id: rustc_hir::def_id::DefId, fields: Vec<FieldDef>) {
        let path = self.resolve(def_id);
        if let Some(node) = self.nodes.get_mut(&path) {
            node.fields = fields;
        }
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
        let full_span = self.span_with_attrs(item.hir_id(), item.span);

        match &item.kind {
            rustc_hir::ItemKind::Impl(impl_item) => {
                self.upgrade_span(def_id, full_span);
                if let Some(trait_ref) = &impl_item.of_trait {
                    if let Res::Def(_, trait_id) = trait_ref.trait_ref.path.res {
                        self.emit("impl", self.resolve(def_id), self.resolve(trait_id));
                    }
                }
            }
            rustc_hir::ItemKind::Fn { .. } => {
                self.upgrade_span(def_id, full_span);
                self.emit_provider_edges_for_source(def_id, full_span);
                // Gap 5: extract structured signature
                if let rustc_hir::ItemKind::Fn { sig, body, .. } = &item.kind {
                    if let Some(fn_sig) = self.extract_fn_sig(sig.decl, *body) {
                        self.set_sig(def_id, fn_sig);
                    }
                }
                let saved = self.enter_fn(def_id);
                intravisit::walk_item(self, item);
                self.leave_fn(saved);
                return;
            }
            rustc_hir::ItemKind::Trait { .. } => {
                self.upgrade_span(def_id, full_span);
            }
            // Gap 2: struct nodes with field definitions
            rustc_hir::ItemKind::Struct(_, _, variant_data) => {
                self.upgrade_span(def_id, full_span);
                let fields = variant_data
                    .fields()
                    .iter()
                    .map(|f| FieldDef {
                        name: f.ident.as_str().to_string(),
                        ty: span_snippet(self.source_map, f.ty.span).unwrap_or_default(),
                    })
                    .collect();
                self.set_fields(def_id, fields);
            }
            // Gap 2: enum nodes with variant names
            rustc_hir::ItemKind::Enum(_, _, enum_def) => {
                self.upgrade_span(def_id, full_span);
                let fields = enum_def
                    .variants
                    .iter()
                    .map(|v| FieldDef {
                        name: v.ident.as_str().to_string(),
                        ty: String::new(),
                    })
                    .collect();
                self.set_fields(def_id, fields);
            }
            // Gap 2: type alias — span only, no sub-fields
            rustc_hir::ItemKind::TyAlias(..) => {
                self.upgrade_span(def_id, full_span);
            }
            // Gap 4: use-statement edges with source span
            rustc_hir::ItemKind::Use(use_path, _) => {
                let parent_id = self.tcx.parent(item.owner_id.to_def_id());
                let module_path = self.tcx.def_path_str(parent_id);
                let use_span = to_source_span(self.source_map, item.span, self.workspace_root);
                for res in use_path.res.iter() {
                    if let Some(Res::Def(_, target_id)) = *res {
                        let target = self.tcx.def_path_str(target_id);
                        self.edges.push(GraphEdge {
                            relation: "use".to_string(),
                            from: module_path.clone(),
                            to: target,
                            span: use_span.clone(),
                        });
                    }
                }
                return; // no need to walk into use items
            }
            _ => {}
        }

        intravisit::walk_item(self, item);
    }

    fn visit_impl_item(&mut self, item: &'tcx rustc_hir::ImplItem<'tcx>) {
        if item.span.from_expansion() {
            return;
        }
        let def_id = item.owner_id.to_def_id();
        let full_span = self.span_with_attrs(item.hir_id(), item.span);
        self.upgrade_span(def_id, full_span);
        self.emit_provider_edges_for_source(def_id, full_span);

        let is_fn = matches!(&item.kind, rustc_hir::ImplItemKind::Fn(..));
        // Gap 5: extract structured signature for impl methods
        if let rustc_hir::ImplItemKind::Fn(fn_sig, body_id) = &item.kind {
            if let Some(sig) = self.extract_fn_sig(fn_sig.decl, *body_id) {
                self.set_sig(def_id, sig);
            }
        }
        let saved = if is_fn {
            Some(self.enter_fn(def_id))
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
        let def_id = item.owner_id.to_def_id();
        let full_span = self.span_with_attrs(item.hir_id(), item.span);
        self.upgrade_span(def_id, full_span);
        self.emit_provider_edges_for_source(def_id, full_span);

        let is_fn = matches!(&item.kind, rustc_hir::TraitItemKind::Fn(..));
        // Gap 5: extract structured signature for trait methods
        if let rustc_hir::TraitItemKind::Fn(fn_sig, trait_fn) = &item.kind {
            let sig_opt = match trait_fn {
                rustc_hir::TraitFn::Provided(body_id) => self.extract_fn_sig(fn_sig.decl, *body_id),
                rustc_hir::TraitFn::Required(param_names) => {
                    self.extract_fn_sig_required(fn_sig.decl, *param_names)
                }
            };
            if let Some(sig) = sig_opt {
                self.set_sig(def_id, sig);
            }
        }
        let saved = if is_fn {
            Some(self.enter_fn(def_id))
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
                    // Gap 1: attach call-site span to `call` edges
                    let call_span = to_source_span(self.source_map, expr.span, self.workspace_root);
                    self.emit_call_facts(&callee, call_span);
                }
            }
            rustc_hir::ExprKind::MethodCall(seg, _, _, _) => {
                let hir_id = expr.hir_id;
                if let Some(callee_id) = self.tcx.typeck(owner_def_id).type_dependent_def_id(hir_id)
                {
                    let callee = self.resolve(callee_id);
                    // Gap 1: attach call-site span to `call` edges
                    let call_span = to_source_span(self.source_map, expr.span, self.workspace_root);
                    self.emit_call_facts(&callee, call_span);
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

// ─── span helpers ────────────────────────────────────────────────────────────

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

fn span_snippet(source_map: &SourceMap, span: Span) -> Option<String> {
    if span.from_expansion() {
        return None;
    }
    source_map.span_to_snippet(span).ok()
}

fn contains_any_provider_sentinel(source: &str, sentinels: &[&str]) -> bool {
    sentinels.iter().any(|sentinel| source.contains(sentinel))
}
