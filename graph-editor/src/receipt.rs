//! GraphMutationReceipt — the TLog evidence record for one mutation run.
//!
//! A receipt binds the op set hash, the old and new graph hashes, the list of
//! structural changes that were verified, the stale/rejected op list, and a
//! final verdict.  It is designed to slot into the same TLog evidence contract
//! used by the rest of the canon-agent capability layer.

use crate::graph::CrateGraph;
use crate::patch::PatchSet;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Every op landed in the new graph; no stale ops.
    Pass,
    /// Some ops were stale (rejected); all valid ops landed.
    Partial,
    /// The new graph was not provided for verification.
    PatchOnly,
    /// At least one valid op did not land in the new graph.
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMutationReceipt {
    pub schema_version: u32,
    pub record_type: String,
    /// SHA-256 of the serialized op list (JSON).
    pub op_set_hash: String,
    /// `meta.graph_hash` from the graph before mutation.
    pub old_graph_hash: String,
    /// `meta.graph_hash` from the graph after re-capture (empty if not yet verified).
    pub new_graph_hash: String,
    /// Paths whose nodes were removed by ops that landed.
    pub nodes_removed: Vec<String>,
    /// Paths whose intent labels were overridden.
    pub intents_changed: Vec<IntentChange>,
    /// Attribute insertions that were patched.
    pub attributes_added: Vec<AttrAdd>,
    /// Ops that were rejected due to stale lo/hi offsets.
    pub stale_ops: Vec<String>,
    pub verdict: Verdict,
    /// SHA-256 of the canonical JSON of this receipt (excluding this field).
    pub receipt_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentChange {
    pub path: String,
    pub new_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttrAdd {
    pub path: String,
    pub attr: String,
}

impl GraphMutationReceipt {
    /// Build a receipt from the patch set and the old graph.
    /// `new_graph` is `None` if the re-capture has not yet run (verdict = PatchOnly).
    /// `ops_json` is the raw bytes of the ops file, used only for hashing.
    pub fn build(
        ops_json: &[u8],
        old_graph: &CrateGraph,
        patch_set: &PatchSet,
        new_graph: Option<&CrateGraph>,
        nodes_removed: Vec<String>,
        attrs_added: Vec<AttrAdd>,
    ) -> Self {
        let op_set_hash = hex_sha256(ops_json);
        let old_graph_hash = old_graph.meta.graph_hash.clone();
        let new_graph_hash = new_graph
            .map(|g| g.meta.graph_hash.clone())
            .unwrap_or_default();

        let intents_changed: Vec<IntentChange> = patch_set
            .intent_overrides
            .iter()
            .map(|(path, label)| IntentChange { path: path.clone(), new_label: label.clone() })
            .collect();

        let verdict = determine_verdict(patch_set, new_graph, &nodes_removed, &intents_changed);

        let mut r = GraphMutationReceipt {
            schema_version: 1,
            record_type: "graph_mutation_receipt".into(),
            op_set_hash,
            old_graph_hash,
            new_graph_hash,
            nodes_removed,
            intents_changed,
            attributes_added: attrs_added,
            stale_ops: patch_set.stale_ops.clone(),
            verdict,
            receipt_hash: String::new(),
        };
        r.receipt_hash = r.compute_hash();
        r
    }

    fn compute_hash(&self) -> String {
        // Serialize without the receipt_hash field to avoid circular dependency.
        #[derive(Serialize)]
        struct Hashable<'a> {
            schema_version: u32,
            record_type: &'a str,
            op_set_hash: &'a str,
            old_graph_hash: &'a str,
            new_graph_hash: &'a str,
            nodes_removed: &'a [String],
            intents_changed: &'a [IntentChange],
            attributes_added: &'a [AttrAdd],
            stale_ops: &'a [String],
            verdict: &'a Verdict,
        }
        let h = Hashable {
            schema_version: self.schema_version,
            record_type: &self.record_type,
            op_set_hash: &self.op_set_hash,
            old_graph_hash: &self.old_graph_hash,
            new_graph_hash: &self.new_graph_hash,
            nodes_removed: &self.nodes_removed,
            intents_changed: &self.intents_changed,
            attributes_added: &self.attributes_added,
            stale_ops: &self.stale_ops,
            verdict: &self.verdict,
        };
        hex_sha256(&serde_json::to_vec(&h).unwrap_or_default())
    }

    pub fn is_valid(&self) -> bool {
        self.receipt_hash == self.compute_hash()
    }
}

fn determine_verdict(
    patch_set: &PatchSet,
    new_graph: Option<&CrateGraph>,
    nodes_removed: &[String],
    intents_changed: &[IntentChange],
) -> Verdict {
    let Some(ng) = new_graph else {
        return Verdict::PatchOnly;
    };

    // Every node that should be removed must be absent from the new graph.
    let all_removed = nodes_removed
        .iter()
        .all(|p| !ng.nodes.contains_key(p.as_str()));

    // Every intent override must appear in the new graph's intents.
    let all_intents: BTreeMap<&str, &str> =
        ng.intents.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    let all_intents_applied = intents_changed.iter().all(|ic| {
        all_intents.get(ic.path.as_str()).map(|v| *v == ic.new_label).unwrap_or(false)
    });

    if !patch_set.stale_ops.is_empty() {
        return Verdict::Partial;
    }

    if all_removed && all_intents_applied {
        Verdict::Pass
    } else {
        Verdict::Fail
    }
}

fn hex_sha256(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    hex::encode(h.finalize())
}

// ─── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{CrateGraph, GraphMeta};
    use crate::patch::PatchSet;
    use std::collections::BTreeMap;

    fn empty_patch() -> PatchSet {
        PatchSet {
            source_patch: String::new(),
            intent_overrides: BTreeMap::new(),
            stale_ops: vec![],
        }
    }

    fn graph_with_hash(h: &str) -> CrateGraph {
        CrateGraph {
            meta: GraphMeta {
                schema_version: 11,
                graph_hash: h.to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn receipt_is_self_consistent() {
        let old = graph_with_hash("oldhash");
        let patch = empty_patch();
        let r = GraphMutationReceipt::build(b"[]", &old, &patch, None, vec![], vec![]);
        assert!(r.is_valid());
        assert_eq!(r.verdict, Verdict::PatchOnly);
    }

    #[test]
    fn tampered_receipt_fails_validation() {
        let old = graph_with_hash("oldhash");
        let patch = empty_patch();
        let mut r = GraphMutationReceipt::build(b"[]", &old, &patch, None, vec![], vec![]);
        r.old_graph_hash = "tampered".into();
        assert!(!r.is_valid());
    }

    #[test]
    fn pass_verdict_when_node_absent_from_new_graph() {
        let old = graph_with_hash("old");
        let new = graph_with_hash("new"); // no nodes
        let patch = empty_patch();
        let r = GraphMutationReceipt::build(
            b"[]",
            &old,
            &patch,
            Some(&new),
            vec!["foo::bar".into()],
            vec![],
        );
        assert_eq!(r.verdict, Verdict::Pass);
    }

    #[test]
    fn fail_verdict_when_node_still_present_in_new_graph() {
        use crate::graph::GraphNode;
        let old = graph_with_hash("old");
        let mut new = graph_with_hash("new");
        new.nodes.insert(
            "foo::bar".into(),
            GraphNode { def_id: "0:1".into(), path: "foo::bar".into(), kind: "fn".into(), def: None, source_text: None, sig: None, fields: vec![] },
        );
        let patch = empty_patch();
        let r = GraphMutationReceipt::build(
            b"[]",
            &old,
            &patch,
            Some(&new),
            vec!["foo::bar".into()],
            vec![],
        );
        assert_eq!(r.verdict, Verdict::Fail);
    }

    #[test]
    fn partial_verdict_when_stale_ops_present() {
        let old = graph_with_hash("old");
        let new = graph_with_hash("new");
        let patch = PatchSet {
            source_patch: String::new(),
            intent_overrides: BTreeMap::new(),
            stale_ops: vec!["RemoveNode(foo): stale".into()],
        };
        let r = GraphMutationReceipt::build(b"[]", &old, &patch, Some(&new), vec![], vec![]);
        assert_eq!(r.verdict, Verdict::Partial);
    }
}
