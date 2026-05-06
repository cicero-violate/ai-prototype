//! Deterministic DefId → NodeId index (ported from canon-rustc index.rs).

use rustc_hash::FxHashMap;
use rustc_middle::ty::TyCtxt;
use rustc_span::def_id::DefId;
use std::collections::HashSet;

/// Stable mapping from DefId to a dense rank.
#[derive(Debug, Default, Clone)]
pub struct Index {
    pub def_ids: Vec<DefId>,
    pub def_to_rank: FxHashMap<DefId, u32>,
}

/// Build a stable local DefId index with parent closure.
pub fn build_index(tcx: TyCtxt<'_>) -> Index {
    let crate_items = tcx.hir_crate_items(());

    let mut pairs: Vec<(String, DefId)> = crate_items
        .definitions()
        .map(|id| id.to_def_id())
        .filter(|&d| !tcx.is_automatically_derived(d))
        .filter(|&d| !tcx.is_synthetic_mir(d))
        .map(|d| (tcx.def_path_str(d), d))
        .collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));

    let mut set: HashSet<DefId> = pairs.iter().map(|(_, d)| *d).collect();
    let mut changed = true;
    while changed {
        changed = false;
        let current: Vec<DefId> = set.iter().copied().collect();
        for def_id in current {
            if let Some(parent) = tcx.opt_parent(def_id) {
                if !set.contains(&parent) {
                    set.insert(parent);
                    changed = true;
                }
            }
        }
    }

    let mut def_ids: Vec<DefId> = set.into_iter().collect();
    def_ids.sort_by(|a, b| tcx.def_path_str(*a).cmp(&tcx.def_path_str(*b)));

    let mut def_to_rank = FxHashMap::default();
    for (rank, def_id) in def_ids.iter().enumerate() {
        def_to_rank.insert(*def_id, rank as u32);
    }

    Index {
        def_ids,
        def_to_rank,
    }
}

pub fn def_id_key(def_id: DefId) -> String {
    format!("{}:{}", def_id.krate.as_u32(), def_id.index.as_u32())
}
