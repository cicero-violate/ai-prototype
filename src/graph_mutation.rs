//! Graph-as-source-of-truth mutation contract.
//!
//! The canon-rustc-v3 wrapper emits `graph.json` nodes with source byte spans.
//! This module defines the root project contract used by external mutation
//! agents: typed graph operations are checked against the current graph, turned
//! into deterministic unified-diff hunks, and can later be verified by graph
//! re-capture plus graph-diff receipts.

use crate::kernel::mix;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const GRAPH_MUTATION_SCHEMA_VERSION: u64 = 1;
pub const GRAPH_JSON_SCHEMA_VERSION: u64 = 11;
pub const GRAPH_MUTATION_RECEIPT_RECORD: u64 = 0xa7a0_0001;
pub const GRAPH_MUTATION_VERIFY_RECORD: u64 = 0xa7a0_0002;
pub const GRAPH_MUTATION_LEDGER_RECORD: u64 = 0xa7a0_0003;
pub const GRAPH_MUTATION_OPSET_RECORD: u64 = 0xa7a0_0004;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphSourceSpan {
    pub file: String,
    pub line: u64,
    pub col: u64,
    pub lo: usize,
    pub hi: usize,
}

impl GraphSourceSpan {
    pub fn new(file: impl Into<String>, line: u64, col: u64, lo: usize, hi: usize) -> Self {
        Self {
            file: file.into(),
            line,
            col,
            lo,
            hi,
        }
    }

    pub fn is_valid_for(&self, content: &str) -> bool {
        !self.file.is_empty() && self.lo <= self.hi && self.hi <= content.len()
    }

    fn hash(&self) -> u64 {
        let mut h = 0x4752_5350_414e_0001u64;
        h = mix_text(h, &self.file);
        h = mix(h, self.line);
        h = mix(h, self.col);
        h = mix(h, self.lo as u64);
        h = mix(h, self.hi as u64);
        h.max(1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodeContract {
    pub path: String,
    pub kind: String,
    pub def_id: String,
    pub span: GraphSourceSpan,
}

impl GraphNodeContract {
    pub fn new(
        path: impl Into<String>,
        kind: impl Into<String>,
        def_id: impl Into<String>,
        span: GraphSourceSpan,
    ) -> Self {
        Self {
            path: path.into(),
            kind: kind.into(),
            def_id: def_id.into(),
            span,
        }
    }

    fn hash(&self) -> u64 {
        let mut h = 0x4752_4e4f_4445_0001u64;
        h = mix_text(h, &self.path);
        h = mix_text(h, &self.kind);
        h = mix_text(h, &self.def_id);
        h = mix(h, self.span.hash());
        h.max(1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GraphEdgeContract {
    pub relation: String,
    pub from: String,
    pub to: String,
}

impl GraphEdgeContract {
    pub fn new(
        relation: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> Self {
        Self {
            relation: relation.into(),
            from: from.into(),
            to: to.into(),
        }
    }

    fn hash(&self) -> u64 {
        let mut h = 0x4752_4544_4745_0001u64;
        h = mix_text(h, &self.relation);
        h = mix_text(h, &self.from);
        h = mix_text(h, &self.to);
        h.max(1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphSnapshotContract {
    pub schema_version: u64,
    pub graph_hash: u64,
    pub nodes: BTreeMap<String, GraphNodeContract>,
    pub edges: BTreeSet<GraphEdgeContract>,
    pub intents: BTreeMap<String, String>,
}

impl GraphSnapshotContract {
    pub fn new(schema_version: u64, graph_hash: u64) -> Self {
        Self {
            schema_version,
            graph_hash,
            nodes: BTreeMap::new(),
            edges: BTreeSet::new(),
            intents: BTreeMap::new(),
        }
    }

    pub fn insert_node(&mut self, node: GraphNodeContract) {
        self.nodes.insert(node.path.clone(), node);
    }

    pub fn insert_edge(&mut self, edge: GraphEdgeContract) {
        self.edges.insert(edge);
    }

    pub fn set_intent(&mut self, path: impl Into<String>, intent: impl Into<String>) {
        self.intents.insert(path.into(), intent.into());
    }

    pub fn contract_hash(&self) -> u64 {
        let mut h = 0x4752_4150_485f_0001u64;
        h = mix(h, self.schema_version);
        h = mix(h, self.graph_hash);
        for node in self.nodes.values() {
            h = mix(h, node.hash());
        }
        for edge in &self.edges {
            h = mix(h, edge.hash());
        }
        for (path, intent) in &self.intents {
            h = mix_text(h, path);
            h = mix_text(h, intent);
        }
        h.max(1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphSourceFile {
    pub path: String,
    pub content: String,
}

impl GraphSourceFile {
    pub fn new(path: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content: content.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphMutationOp {
    RemoveNode {
        path: String,
        file: String,
        lo: usize,
        hi: usize,
    },
    RetypeIntent {
        path: String,
        file: String,
        lo: usize,
        hi: usize,
        new_label: String,
    },
    AddAttribute {
        path: String,
        file: String,
        lo: usize,
        hi: usize,
        attr: String,
    },
    RemoveEdge {
        relation: String,
        from: String,
        to: String,
        file: String,
        lo: usize,
        hi: usize,
    },
}

impl GraphMutationOp {
    pub fn file(&self) -> &str {
        match self {
            Self::RemoveNode { file, .. }
            | Self::RetypeIntent { file, .. }
            | Self::AddAttribute { file, .. }
            | Self::RemoveEdge { file, .. } => file,
        }
    }

    pub fn lo(&self) -> usize {
        match self {
            Self::RemoveNode { lo, .. }
            | Self::RetypeIntent { lo, .. }
            | Self::AddAttribute { lo, .. }
            | Self::RemoveEdge { lo, .. } => *lo,
        }
    }

    pub fn hi(&self) -> usize {
        match self {
            Self::RemoveNode { hi, .. }
            | Self::RetypeIntent { hi, .. }
            | Self::AddAttribute { hi, .. }
            | Self::RemoveEdge { hi, .. } => *hi,
        }
    }

    pub fn hash(&self) -> u64 {
        let mut h = 0x4752_4f50_0001_0001u64;
        h = mix(h, self.kind_tag());
        match self {
            Self::RemoveNode { path, file, lo, hi } => {
                h = mix_text(h, path);
                h = mix_text(h, file);
                h = mix(h, *lo as u64);
                h = mix(h, *hi as u64);
            }
            Self::RetypeIntent {
                path,
                file,
                lo,
                hi,
                new_label,
            } => {
                h = mix_text(h, path);
                h = mix_text(h, file);
                h = mix(h, *lo as u64);
                h = mix(h, *hi as u64);
                h = mix_text(h, new_label);
            }
            Self::AddAttribute {
                path,
                file,
                lo,
                hi,
                attr,
            } => {
                h = mix_text(h, path);
                h = mix_text(h, file);
                h = mix(h, *lo as u64);
                h = mix(h, *hi as u64);
                h = mix_text(h, attr);
            }
            Self::RemoveEdge {
                relation,
                from,
                to,
                file,
                lo,
                hi,
            } => {
                h = mix_text(h, relation);
                h = mix_text(h, from);
                h = mix_text(h, to);
                h = mix_text(h, file);
                h = mix(h, *lo as u64);
                h = mix(h, *hi as u64);
            }
        }
        h.max(1)
    }

    fn kind_tag(&self) -> u64 {
        match self {
            Self::RemoveNode { .. } => 1,
            Self::RetypeIntent { .. } => 2,
            Self::AddAttribute { .. } => 3,
            Self::RemoveEdge { .. } => 4,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphMutationOpRow {
    pub schema_version: u64,
    pub record_type: u64,
    pub op_index: u64,
    pub op: GraphMutationOp,
    pub op_hash: u64,
    pub row_hash: u64,
}

impl GraphMutationOpRow {
    pub fn new(op_index: u64, op: GraphMutationOp) -> Self {
        let mut row = Self {
            schema_version: GRAPH_MUTATION_SCHEMA_VERSION,
            record_type: GRAPH_MUTATION_OPSET_RECORD,
            op_index,
            op_hash: op.hash(),
            op,
            row_hash: 0,
        };
        row.row_hash = row.expected_row_hash();
        row
    }

    pub fn is_self_consistent(&self) -> bool {
        self.schema_version == GRAPH_MUTATION_SCHEMA_VERSION
            && self.record_type == GRAPH_MUTATION_OPSET_RECORD
            && self.op_hash == self.op.hash()
            && self.row_hash == self.expected_row_hash()
    }

    pub fn expected_row_hash(&self) -> u64 {
        let mut h = 0xa7a0_0004_524f_5701u64;
        h = mix(h, self.schema_version);
        h = mix(h, self.record_type);
        h = mix(h, self.op_index);
        h = mix(h, self.op_hash);
        h.max(1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphMutationOpSetReceipt {
    pub schema_version: u64,
    pub record_type: u64,
    pub op_count: u64,
    pub valid_row_count: u64,
    pub invalid_row_count: u64,
    pub ordered_op_set_hash: u64,
    pub sorted_op_set_hash: u64,
    pub ledger_hash: u64,
    pub verdict: GraphMutationVerdict,
    pub receipt_hash: u64,
}

impl GraphMutationOpSetReceipt {
    pub fn is_self_consistent(&self) -> bool {
        self.schema_version == GRAPH_MUTATION_SCHEMA_VERSION
            && self.record_type == GRAPH_MUTATION_OPSET_RECORD
            && self.ordered_op_set_hash != 0
            && self.sorted_op_set_hash != 0
            && self.ledger_hash != 0
            && self.receipt_hash == self.expected_receipt_hash()
            && match self.verdict {
                GraphMutationVerdict::Pass => self.invalid_row_count == 0,
                GraphMutationVerdict::Fail => self.invalid_row_count != 0,
            }
    }

    pub fn expected_receipt_hash(&self) -> u64 {
        let mut h = 0xa7a0_0004_5eed_0001u64;
        h = mix(h, self.schema_version);
        h = mix(h, self.record_type);
        h = mix(h, self.op_count);
        h = mix(h, self.valid_row_count);
        h = mix(h, self.invalid_row_count);
        h = mix(h, self.ordered_op_set_hash);
        h = mix(h, self.sorted_op_set_hash);
        h = mix(h, self.ledger_hash);
        h = mix(h, self.verdict as u64);
        h.max(1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphMutationVerdict {
    Pass,
    Fail,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphPatchReceipt {
    pub schema_version: u64,
    pub record_type: u64,
    pub graph_schema_version: u64,
    pub op_count: u64,
    pub op_set_hash: u64,
    pub old_graph_hash: u64,
    pub source_set_hash: u64,
    pub patch_hash: u64,
    pub hunk_count: u64,
    pub stale_op_count: u64,
    pub overlap_count: u64,
    pub verdict: GraphMutationVerdict,
    pub receipt_hash: u64,
}

impl GraphPatchReceipt {
    pub fn is_self_consistent(&self) -> bool {
        self.schema_version == GRAPH_MUTATION_SCHEMA_VERSION
            && self.record_type == GRAPH_MUTATION_RECEIPT_RECORD
            && self.graph_schema_version == GRAPH_JSON_SCHEMA_VERSION
            && self.op_count != 0
            && self.op_set_hash != 0
            && self.old_graph_hash != 0
            && self.source_set_hash != 0
            && self.patch_hash != 0
            && self.hunk_count == self.op_count
            && self.receipt_hash == self.expected_receipt_hash()
            && match self.verdict {
                GraphMutationVerdict::Pass => self.stale_op_count == 0 && self.overlap_count == 0,
                GraphMutationVerdict::Fail => self.stale_op_count != 0 || self.overlap_count != 0,
            }
    }

    pub fn expected_receipt_hash(&self) -> u64 {
        let mut h = 0xa7a0_0001_5eed_0001u64;
        h = mix(h, self.schema_version);
        h = mix(h, self.record_type);
        h = mix(h, self.graph_schema_version);
        h = mix(h, self.op_count);
        h = mix(h, self.op_set_hash);
        h = mix(h, self.old_graph_hash);
        h = mix(h, self.source_set_hash);
        h = mix(h, self.patch_hash);
        h = mix(h, self.hunk_count);
        h = mix(h, self.stale_op_count);
        h = mix(h, self.overlap_count);
        h = mix(h, self.verdict as u64);
        h.max(1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphPatchPlan {
    pub diff: String,
    pub receipt: GraphPatchReceipt,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphMutationReceipt {
    pub schema_version: u64,
    pub record_type: u64,
    pub graph_schema_version: u64,
    pub op_count: u64,
    pub op_set_hash: u64,
    pub old_graph_hash: u64,
    pub new_graph_hash: u64,
    pub nodes_removed: u64,
    pub edges_removed: u64,
    pub intents_changed: u64,
    pub attributes_added: u64,
    pub missing_landing_count: u64,
    pub verdict: GraphMutationVerdict,
    pub receipt_hash: u64,
}

impl GraphMutationReceipt {
    pub fn is_self_consistent(&self) -> bool {
        self.schema_version == GRAPH_MUTATION_SCHEMA_VERSION
            && self.record_type == GRAPH_MUTATION_VERIFY_RECORD
            && self.graph_schema_version == GRAPH_JSON_SCHEMA_VERSION
            && self.op_count != 0
            && self.op_set_hash != 0
            && self.old_graph_hash != 0
            && self.new_graph_hash != 0
            && self.receipt_hash == self.expected_receipt_hash()
            && match self.verdict {
                GraphMutationVerdict::Pass => self.missing_landing_count == 0,
                GraphMutationVerdict::Fail => self.missing_landing_count != 0,
            }
    }

    pub fn expected_receipt_hash(&self) -> u64 {
        let mut h = 0xa7a0_0002_5eed_0001u64;
        h = mix(h, self.schema_version);
        h = mix(h, self.record_type);
        h = mix(h, self.graph_schema_version);
        h = mix(h, self.op_count);
        h = mix(h, self.op_set_hash);
        h = mix(h, self.old_graph_hash);
        h = mix(h, self.new_graph_hash);
        h = mix(h, self.nodes_removed);
        h = mix(h, self.edges_removed);
        h = mix(h, self.intents_changed);
        h = mix(h, self.attributes_added);
        h = mix(h, self.missing_landing_count);
        h = mix(h, self.verdict as u64);
        h.max(1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphReceiptLedgerReceipt {
    pub schema_version: u64,
    pub record_type: u64,
    pub patch_receipt_count: u64,
    pub mutation_receipt_count: u64,
    pub passing_patch_count: u64,
    pub passing_mutation_count: u64,
    pub failing_patch_count: u64,
    pub failing_mutation_count: u64,
    pub invalid_line_count: u64,
    pub patch_ledger_hash: u64,
    pub mutation_ledger_hash: u64,
    pub aggregate_receipt_hash: u64,
    pub verdict: GraphMutationVerdict,
    pub receipt_hash: u64,
}

impl GraphReceiptLedgerReceipt {
    pub fn is_self_consistent(&self) -> bool {
        self.schema_version == GRAPH_MUTATION_SCHEMA_VERSION
            && self.record_type == GRAPH_MUTATION_LEDGER_RECORD
            && self.patch_ledger_hash != 0
            && self.mutation_ledger_hash != 0
            && self.aggregate_receipt_hash != 0
            && self.receipt_hash == self.expected_receipt_hash()
            && match self.verdict {
                GraphMutationVerdict::Pass => self.invalid_line_count == 0,
                GraphMutationVerdict::Fail => self.invalid_line_count != 0,
            }
    }

    pub fn expected_receipt_hash(&self) -> u64 {
        let mut h = 0xa7a0_0003_5eed_0001u64;
        h = mix(h, self.schema_version);
        h = mix(h, self.record_type);
        h = mix(h, self.patch_receipt_count);
        h = mix(h, self.mutation_receipt_count);
        h = mix(h, self.passing_patch_count);
        h = mix(h, self.passing_mutation_count);
        h = mix(h, self.failing_patch_count);
        h = mix(h, self.failing_mutation_count);
        h = mix(h, self.invalid_line_count);
        h = mix(h, self.patch_ledger_hash);
        h = mix(h, self.mutation_ledger_hash);
        h = mix(h, self.aggregate_receipt_hash);
        h = mix(h, self.verdict as u64);
        h.max(1)
    }
}

pub fn verify_graph_mutation_landing(
    old_graph: &GraphSnapshotContract,
    new_graph: &GraphSnapshotContract,
    ops: &[GraphMutationOp],
) -> Result<GraphMutationReceipt, GraphPatchError> {
    if ops.is_empty() {
        return Err(GraphPatchError::EmptyOps);
    }
    if old_graph.schema_version != GRAPH_JSON_SCHEMA_VERSION
        || new_graph.schema_version != GRAPH_JSON_SCHEMA_VERSION
    {
        return Err(GraphPatchError::UnsupportedGraphSchema);
    }

    let mut nodes_removed = 0u64;
    let mut edges_removed = 0u64;
    let mut intents_changed = 0u64;
    let mut attributes_added = 0u64;
    let mut missing_landing_count = 0u64;

    for op in ops {
        match op {
            GraphMutationOp::RemoveNode { path, .. } => {
                let landed =
                    old_graph.nodes.contains_key(path) && !new_graph.nodes.contains_key(path);
                if landed {
                    nodes_removed += 1;
                } else {
                    missing_landing_count += 1;
                }
            }
            GraphMutationOp::RetypeIntent {
                path, new_label, ..
            } => {
                let landed = old_graph.intents.get(path) != Some(new_label)
                    && new_graph.intents.get(path) == Some(new_label);
                if landed {
                    intents_changed += 1;
                } else {
                    missing_landing_count += 1;
                }
            }
            GraphMutationOp::AddAttribute { path, .. } => {
                let landed = old_graph.nodes.get(path) != new_graph.nodes.get(path)
                    || old_graph.contract_hash() != new_graph.contract_hash();
                if landed {
                    attributes_added += 1;
                } else {
                    missing_landing_count += 1;
                }
            }
            GraphMutationOp::RemoveEdge {
                relation, from, to, ..
            } => {
                let edge = GraphEdgeContract::new(relation.clone(), from.clone(), to.clone());
                let landed = old_graph.edges.contains(&edge) && !new_graph.edges.contains(&edge);
                if landed {
                    edges_removed += 1;
                } else {
                    missing_landing_count += 1;
                }
            }
        }
    }

    let sorted_ops = sorted_ops(ops);
    let mut receipt = GraphMutationReceipt {
        schema_version: GRAPH_MUTATION_SCHEMA_VERSION,
        record_type: GRAPH_MUTATION_VERIFY_RECORD,
        graph_schema_version: old_graph.schema_version,
        op_count: sorted_ops.len() as u64,
        op_set_hash: op_set_hash(&sorted_ops),
        old_graph_hash: old_graph.contract_hash(),
        new_graph_hash: new_graph.contract_hash(),
        nodes_removed,
        edges_removed,
        intents_changed,
        attributes_added,
        missing_landing_count,
        verdict: if missing_landing_count == 0 {
            GraphMutationVerdict::Pass
        } else {
            GraphMutationVerdict::Fail
        },
        receipt_hash: 0,
    };
    receipt.receipt_hash = receipt.expected_receipt_hash();
    Ok(receipt)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphPatchError {
    EmptyOps,
    UnsupportedGraphSchema,
    MissingSource(String),
    StaleOperation(String),
    OverlappingOperation(String),
    InvalidSourceSpan(String),
}

impl fmt::Display for GraphPatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyOps => write!(f, "graph patch operation set is empty"),
            Self::UnsupportedGraphSchema => write!(f, "graph schema version is unsupported"),
            Self::MissingSource(path) => write!(f, "source file is missing: {path}"),
            Self::StaleOperation(path) => write!(f, "graph operation is stale: {path}"),
            Self::OverlappingOperation(path) => {
                write!(f, "graph operations overlap in file: {path}")
            }
            Self::InvalidSourceSpan(path) => {
                write!(f, "graph operation has invalid source span: {path}")
            }
        }
    }
}

impl std::error::Error for GraphPatchError {}

pub fn encode_graph_patch_receipt_ndjson(receipt: &GraphPatchReceipt) -> String {
    format!(
        "[{},{},{},{},{},{},{},{},{},{},{},{},{}]\n",
        receipt.schema_version,
        receipt.record_type,
        receipt.graph_schema_version,
        receipt.op_count,
        receipt.op_set_hash,
        receipt.old_graph_hash,
        receipt.source_set_hash,
        receipt.patch_hash,
        receipt.hunk_count,
        receipt.stale_op_count,
        receipt.overlap_count,
        receipt.verdict as u64,
        receipt.receipt_hash
    )
}

pub fn decode_graph_patch_receipt_ndjson(line: &str) -> Option<GraphPatchReceipt> {
    let values = parse_u64_array(line)?;
    if values.len() != 13 {
        return None;
    }
    let receipt = GraphPatchReceipt {
        schema_version: values[0],
        record_type: values[1],
        graph_schema_version: values[2],
        op_count: values[3],
        op_set_hash: values[4],
        old_graph_hash: values[5],
        source_set_hash: values[6],
        patch_hash: values[7],
        hunk_count: values[8],
        stale_op_count: values[9],
        overlap_count: values[10],
        verdict: decode_verdict(values[11])?,
        receipt_hash: values[12],
    };
    receipt.is_self_consistent().then_some(receipt)
}

pub fn append_graph_patch_receipt_ndjson(
    path: impl AsRef<std::path::Path>,
    receipt: &GraphPatchReceipt,
) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(encode_graph_patch_receipt_ndjson(receipt).as_bytes())
}

pub fn load_graph_patch_receipts_ndjson(
    path: impl AsRef<std::path::Path>,
) -> std::io::Result<Vec<GraphPatchReceipt>> {
    let input = std::fs::read_to_string(path)?;
    Ok(input
        .lines()
        .filter_map(decode_graph_patch_receipt_ndjson)
        .collect())
}

pub fn encode_graph_mutation_receipt_ndjson(receipt: &GraphMutationReceipt) -> String {
    format!(
        "[{},{},{},{},{},{},{},{},{},{},{},{},{},{}]\n",
        receipt.schema_version,
        receipt.record_type,
        receipt.graph_schema_version,
        receipt.op_count,
        receipt.op_set_hash,
        receipt.old_graph_hash,
        receipt.new_graph_hash,
        receipt.nodes_removed,
        receipt.edges_removed,
        receipt.intents_changed,
        receipt.attributes_added,
        receipt.missing_landing_count,
        receipt.verdict as u64,
        receipt.receipt_hash
    )
}

pub fn encode_graph_mutation_opset_receipt_ndjson(receipt: &GraphMutationOpSetReceipt) -> String {
    format!(
        "[{},{},{},{},{},{},{},{},{},{}]\n",
        receipt.schema_version,
        receipt.record_type,
        receipt.op_count,
        receipt.valid_row_count,
        receipt.invalid_row_count,
        receipt.ordered_op_set_hash,
        receipt.sorted_op_set_hash,
        receipt.ledger_hash,
        receipt.verdict as u64,
        receipt.receipt_hash
    )
}

pub fn encode_graph_receipt_ledger_receipt_ndjson(receipt: &GraphReceiptLedgerReceipt) -> String {
    format!(
        "[{},{},{},{},{},{},{},{},{},{},{},{},{},{}]\n",
        receipt.schema_version,
        receipt.record_type,
        receipt.patch_receipt_count,
        receipt.mutation_receipt_count,
        receipt.passing_patch_count,
        receipt.passing_mutation_count,
        receipt.failing_patch_count,
        receipt.failing_mutation_count,
        receipt.invalid_line_count,
        receipt.patch_ledger_hash,
        receipt.mutation_ledger_hash,
        receipt.aggregate_receipt_hash,
        receipt.verdict as u64,
        receipt.receipt_hash
    )
}

pub fn encode_graph_snapshot_contract_ndjson(graph: &GraphSnapshotContract) -> String {
    let mut out = format!("G|{}|{}\n", graph.schema_version, graph.graph_hash);
    for node in graph.nodes.values() {
        out.push_str(&format!(
            "N|{}|{}|{}|{}|{}|{}|{}|{}\n",
            escape_field(&node.path),
            escape_field(&node.kind),
            escape_field(&node.def_id),
            escape_field(&node.span.file),
            node.span.line,
            node.span.col,
            node.span.lo,
            node.span.hi
        ));
    }
    for edge in &graph.edges {
        out.push_str(&format!(
            "E|{}|{}|{}\n",
            escape_field(&edge.relation),
            escape_field(&edge.from),
            escape_field(&edge.to)
        ));
    }
    for (path, label) in &graph.intents {
        out.push_str(&format!(
            "I|{}|{}\n",
            escape_field(path),
            escape_field(label)
        ));
    }
    out
}

pub fn decode_graph_snapshot_contract_ndjson(input: &str) -> Option<GraphSnapshotContract> {
    let mut graph = None;
    for line in input.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = split_escaped_fields(line.trim())?;
        match fields.first().map(String::as_str)? {
            "G" if fields.len() == 3 && graph.is_none() => {
                graph = Some(GraphSnapshotContract::new(
                    fields[1].parse::<u64>().ok()?,
                    fields[2].parse::<u64>().ok()?,
                ));
            }
            "N" if fields.len() == 9 => {
                let graph = graph.as_mut()?;
                graph.insert_node(GraphNodeContract::new(
                    unescape_field(&fields[1])?,
                    unescape_field(&fields[2])?,
                    unescape_field(&fields[3])?,
                    GraphSourceSpan::new(
                        unescape_field(&fields[4])?,
                        fields[5].parse::<u64>().ok()?,
                        fields[6].parse::<u64>().ok()?,
                        fields[7].parse::<usize>().ok()?,
                        fields[8].parse::<usize>().ok()?,
                    ),
                ));
            }
            "E" if fields.len() == 4 => {
                let graph = graph.as_mut()?;
                graph.insert_edge(GraphEdgeContract::new(
                    unescape_field(&fields[1])?,
                    unescape_field(&fields[2])?,
                    unescape_field(&fields[3])?,
                ));
            }
            "I" if fields.len() == 3 => {
                let graph = graph.as_mut()?;
                graph.set_intent(unescape_field(&fields[1])?, unescape_field(&fields[2])?);
            }
            _ => return None,
        }
    }
    graph
}

pub fn load_graph_snapshot_contract_ndjson(
    path: impl AsRef<std::path::Path>,
) -> std::io::Result<Option<GraphSnapshotContract>> {
    let input = std::fs::read_to_string(path)?;
    Ok(decode_graph_snapshot_contract_ndjson(&input))
}

pub fn decode_graph_mutation_receipt_ndjson(line: &str) -> Option<GraphMutationReceipt> {
    let values = parse_u64_array(line)?;
    if values.len() != 14 {
        return None;
    }
    let receipt = GraphMutationReceipt {
        schema_version: values[0],
        record_type: values[1],
        graph_schema_version: values[2],
        op_count: values[3],
        op_set_hash: values[4],
        old_graph_hash: values[5],
        new_graph_hash: values[6],
        nodes_removed: values[7],
        edges_removed: values[8],
        intents_changed: values[9],
        attributes_added: values[10],
        missing_landing_count: values[11],
        verdict: decode_verdict(values[12])?,
        receipt_hash: values[13],
    };
    receipt.is_self_consistent().then_some(receipt)
}

pub fn append_graph_mutation_receipt_ndjson(
    path: impl AsRef<std::path::Path>,
    receipt: &GraphMutationReceipt,
) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(encode_graph_mutation_receipt_ndjson(receipt).as_bytes())
}

pub fn load_graph_mutation_receipts_ndjson(
    path: impl AsRef<std::path::Path>,
) -> std::io::Result<Vec<GraphMutationReceipt>> {
    let input = std::fs::read_to_string(path)?;
    Ok(input
        .lines()
        .filter_map(decode_graph_mutation_receipt_ndjson)
        .collect())
}

pub fn encode_graph_mutation_op_row_ndjson(row: &GraphMutationOpRow) -> String {
    let mut fields = vec![
        row.schema_version.to_string(),
        row.record_type.to_string(),
        row.op_index.to_string(),
        row.op_hash.to_string(),
        row.row_hash.to_string(),
        row.op.kind_tag().to_string(),
    ];
    match &row.op {
        GraphMutationOp::RemoveNode { path, file, lo, hi } => {
            fields.push(escape_field(path));
            fields.push(escape_field(file));
            fields.push(lo.to_string());
            fields.push(hi.to_string());
        }
        GraphMutationOp::RetypeIntent {
            path,
            file,
            lo,
            hi,
            new_label,
        } => {
            fields.push(escape_field(path));
            fields.push(escape_field(file));
            fields.push(lo.to_string());
            fields.push(hi.to_string());
            fields.push(escape_field(new_label));
        }
        GraphMutationOp::AddAttribute {
            path,
            file,
            lo,
            hi,
            attr,
        } => {
            fields.push(escape_field(path));
            fields.push(escape_field(file));
            fields.push(lo.to_string());
            fields.push(hi.to_string());
            fields.push(escape_field(attr));
        }
        GraphMutationOp::RemoveEdge {
            relation,
            from,
            to,
            file,
            lo,
            hi,
        } => {
            fields.push(escape_field(relation));
            fields.push(escape_field(from));
            fields.push(escape_field(to));
            fields.push(escape_field(file));
            fields.push(lo.to_string());
            fields.push(hi.to_string());
        }
    }
    format!("{}\n", fields.join("|"))
}

pub fn decode_graph_mutation_op_row_ndjson(line: &str) -> Option<GraphMutationOpRow> {
    let fields = split_escaped_fields(line.trim())?;
    if fields.len() < 10 {
        return None;
    }
    let schema_version = fields[0].parse::<u64>().ok()?;
    let record_type = fields[1].parse::<u64>().ok()?;
    let op_index = fields[2].parse::<u64>().ok()?;
    let op_hash = fields[3].parse::<u64>().ok()?;
    let row_hash = fields[4].parse::<u64>().ok()?;
    let kind = fields[5].parse::<u64>().ok()?;
    let op = match kind {
        1 if fields.len() == 10 => GraphMutationOp::RemoveNode {
            path: unescape_field(&fields[6])?,
            file: unescape_field(&fields[7])?,
            lo: fields[8].parse::<usize>().ok()?,
            hi: fields[9].parse::<usize>().ok()?,
        },
        2 if fields.len() == 11 => GraphMutationOp::RetypeIntent {
            path: unescape_field(&fields[6])?,
            file: unescape_field(&fields[7])?,
            lo: fields[8].parse::<usize>().ok()?,
            hi: fields[9].parse::<usize>().ok()?,
            new_label: unescape_field(&fields[10])?,
        },
        3 if fields.len() == 11 => GraphMutationOp::AddAttribute {
            path: unescape_field(&fields[6])?,
            file: unescape_field(&fields[7])?,
            lo: fields[8].parse::<usize>().ok()?,
            hi: fields[9].parse::<usize>().ok()?,
            attr: unescape_field(&fields[10])?,
        },
        4 if fields.len() == 12 => GraphMutationOp::RemoveEdge {
            relation: unescape_field(&fields[6])?,
            from: unescape_field(&fields[7])?,
            to: unescape_field(&fields[8])?,
            file: unescape_field(&fields[9])?,
            lo: fields[10].parse::<usize>().ok()?,
            hi: fields[11].parse::<usize>().ok()?,
        },
        _ => return None,
    };
    let row = GraphMutationOpRow {
        schema_version,
        record_type,
        op_index,
        op,
        op_hash,
        row_hash,
    };
    row.is_self_consistent().then_some(row)
}

pub fn encode_graph_mutation_ops_ndjson(ops: &[GraphMutationOp]) -> String {
    ops.iter()
        .enumerate()
        .map(|(index, op)| {
            encode_graph_mutation_op_row_ndjson(&GraphMutationOpRow::new(index as u64, op.clone()))
        })
        .collect()
}

pub fn decode_graph_mutation_ops_ndjson(input: &str) -> Option<Vec<GraphMutationOp>> {
    let mut rows = Vec::new();
    for line in input.lines() {
        if line.trim().is_empty() {
            continue;
        }
        rows.push(decode_graph_mutation_op_row_ndjson(line)?);
    }
    rows.sort_by_key(|row| row.op_index);
    if rows
        .iter()
        .enumerate()
        .any(|(expected, row)| row.op_index != expected as u64)
    {
        return None;
    }
    Some(rows.into_iter().map(|row| row.op).collect())
}

pub fn verify_graph_mutation_ops_ndjson(input: &str) -> GraphMutationOpSetReceipt {
    let mut valid_rows = Vec::new();
    let mut invalid_row_count = 0u64;
    let mut ledger_hash = 0xa7a0_0004_4f50_4c47u64;

    for line in input.lines() {
        if line.trim().is_empty() {
            continue;
        }
        ledger_hash = mix_text(ledger_hash, line);
        match decode_graph_mutation_op_row_ndjson(line) {
            Some(row) => valid_rows.push(row),
            None => invalid_row_count += 1,
        }
    }

    valid_rows.sort_by_key(|row| row.op_index);
    let mut ordered_ops = Vec::new();
    for (expected_index, row) in valid_rows.iter().enumerate() {
        let expected_index = expected_index as u64;
        if row.op_index != expected_index {
            invalid_row_count += 1;
        }
        ordered_ops.push(row.op.clone());
    }

    let sorted = sorted_ops(&ordered_ops);
    let ordered_op_set_hash = op_set_hash(&ordered_ops);
    let sorted_op_set_hash = op_set_hash(&sorted);
    let mut receipt = GraphMutationOpSetReceipt {
        schema_version: GRAPH_MUTATION_SCHEMA_VERSION,
        record_type: GRAPH_MUTATION_OPSET_RECORD,
        op_count: ordered_ops.len() as u64,
        valid_row_count: valid_rows.len() as u64,
        invalid_row_count,
        ordered_op_set_hash,
        sorted_op_set_hash,
        ledger_hash: ledger_hash.max(1),
        verdict: if invalid_row_count == 0 {
            GraphMutationVerdict::Pass
        } else {
            GraphMutationVerdict::Fail
        },
        receipt_hash: 0,
    };
    receipt.receipt_hash = receipt.expected_receipt_hash();
    receipt
}

pub fn verify_graph_receipt_ledgers_ndjson(
    patch_ledger: &str,
    mutation_ledger: &str,
) -> GraphReceiptLedgerReceipt {
    let mut patch_receipt_count = 0u64;
    let mut mutation_receipt_count = 0u64;
    let mut passing_patch_count = 0u64;
    let mut passing_mutation_count = 0u64;
    let mut failing_patch_count = 0u64;
    let mut failing_mutation_count = 0u64;
    let mut invalid_line_count = 0u64;
    let mut patch_ledger_hash = 0xa7a0_0003_5041_5443u64;
    let mut mutation_ledger_hash = 0xa7a0_0003_4d55_5441u64;
    let mut aggregate_receipt_hash = 0xa7a0_0003_4147_4752u64;

    for line in patch_ledger.lines() {
        if line.trim().is_empty() {
            continue;
        }
        patch_ledger_hash = mix_text(patch_ledger_hash, line);
        match decode_graph_patch_receipt_ndjson(line) {
            Some(receipt) => {
                patch_receipt_count += 1;
                aggregate_receipt_hash = mix(aggregate_receipt_hash, receipt.receipt_hash);
                match receipt.verdict {
                    GraphMutationVerdict::Pass => passing_patch_count += 1,
                    GraphMutationVerdict::Fail => failing_patch_count += 1,
                }
            }
            None => invalid_line_count += 1,
        }
    }

    for line in mutation_ledger.lines() {
        if line.trim().is_empty() {
            continue;
        }
        mutation_ledger_hash = mix_text(mutation_ledger_hash, line);
        match decode_graph_mutation_receipt_ndjson(line) {
            Some(receipt) => {
                mutation_receipt_count += 1;
                aggregate_receipt_hash = mix(aggregate_receipt_hash, receipt.receipt_hash);
                match receipt.verdict {
                    GraphMutationVerdict::Pass => passing_mutation_count += 1,
                    GraphMutationVerdict::Fail => failing_mutation_count += 1,
                }
            }
            None => invalid_line_count += 1,
        }
    }

    let mut receipt = GraphReceiptLedgerReceipt {
        schema_version: GRAPH_MUTATION_SCHEMA_VERSION,
        record_type: GRAPH_MUTATION_LEDGER_RECORD,
        patch_receipt_count,
        mutation_receipt_count,
        passing_patch_count,
        passing_mutation_count,
        failing_patch_count,
        failing_mutation_count,
        invalid_line_count,
        patch_ledger_hash: patch_ledger_hash.max(1),
        mutation_ledger_hash: mutation_ledger_hash.max(1),
        aggregate_receipt_hash: aggregate_receipt_hash.max(1),
        verdict: if invalid_line_count == 0 {
            GraphMutationVerdict::Pass
        } else {
            GraphMutationVerdict::Fail
        },
        receipt_hash: 0,
    };
    receipt.receipt_hash = receipt.expected_receipt_hash();
    receipt
}

pub fn verify_graph_receipt_ledger_files_ndjson(
    patch_path: impl AsRef<std::path::Path>,
    mutation_path: impl AsRef<std::path::Path>,
) -> std::io::Result<GraphReceiptLedgerReceipt> {
    let patch_ledger = std::fs::read_to_string(patch_path)?;
    let mutation_ledger = std::fs::read_to_string(mutation_path)?;
    Ok(verify_graph_receipt_ledgers_ndjson(
        &patch_ledger,
        &mutation_ledger,
    ))
}

fn decode_verdict(value: u64) -> Option<GraphMutationVerdict> {
    match value {
        0 => Some(GraphMutationVerdict::Pass),
        1 => Some(GraphMutationVerdict::Fail),
        _ => None,
    }
}

fn parse_u64_array(line: &str) -> Option<Vec<u64>> {
    let body = line.trim().strip_prefix('[')?.strip_suffix(']')?;
    if body.trim().is_empty() {
        return Some(Vec::new());
    }
    body.split(',')
        .map(|part| part.trim().parse::<u64>().ok())
        .collect()
}

fn escape_field(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '|' => out.push_str("\\p"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out
}

fn unescape_field(value: &str) -> Option<String> {
    let mut out = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next()? {
            '\\' => out.push('\\'),
            'p' => out.push('|'),
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            _ => return None,
        }
    }
    Some(out)
}

fn split_escaped_fields(line: &str) -> Option<Vec<String>> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut escaped = false;
    for ch in line.chars() {
        if escaped {
            current.push('\\');
            current.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '|' => {
                fields.push(current);
                current = String::new();
            }
            _ => current.push(ch),
        }
    }
    if escaped {
        return None;
    }
    fields.push(current);
    Some(fields)
}

pub fn generate_graph_patch(
    graph: &GraphSnapshotContract,
    sources: &[GraphSourceFile],
    ops: &[GraphMutationOp],
) -> Result<GraphPatchPlan, GraphPatchError> {
    if ops.is_empty() {
        return Err(GraphPatchError::EmptyOps);
    }
    if graph.schema_version != GRAPH_JSON_SCHEMA_VERSION {
        return Err(GraphPatchError::UnsupportedGraphSchema);
    }

    let source_map: BTreeMap<&str, &str> = sources
        .iter()
        .map(|source| (source.path.as_str(), source.content.as_str()))
        .collect();

    let sorted_ops = sorted_ops(ops);

    validate_no_overlaps(&sorted_ops)?;

    let mut diff = String::new();
    let mut current_file = "";
    for op in &sorted_ops {
        let content = source_map
            .get(op.file())
            .copied()
            .ok_or_else(|| GraphPatchError::MissingSource(op.file().to_string()))?;
        validate_op_against_graph(graph, content, op)?;

        if current_file != op.file() {
            current_file = op.file();
            diff.push_str(&format!("--- a/{current_file}\n+++ b/{current_file}\n"));
        }
        diff.push_str(&hunk_for_op(content, op)?);
    }

    let receipt = build_graph_patch_receipt(graph, sources, &sorted_ops, &diff);

    Ok(GraphPatchPlan { diff, receipt })
}

fn build_graph_patch_receipt(
    graph: &GraphSnapshotContract,
    sources: &[GraphSourceFile],
    sorted_ops: &[GraphMutationOp],
    diff: &str,
) -> GraphPatchReceipt {
    let mut receipt = GraphPatchReceipt {
        schema_version: GRAPH_MUTATION_SCHEMA_VERSION,
        record_type: GRAPH_MUTATION_RECEIPT_RECORD,
        graph_schema_version: graph.schema_version,
        op_count: sorted_ops.len() as u64,
        op_set_hash: op_set_hash(sorted_ops),
        old_graph_hash: graph.contract_hash(),
        source_set_hash: source_set_hash(sources),
        patch_hash: hash_text(diff),
        hunk_count: sorted_ops.len() as u64,
        stale_op_count: 0,
        overlap_count: 0,
        verdict: GraphMutationVerdict::Pass,
        receipt_hash: 0,
    };
    receipt.receipt_hash = receipt.expected_receipt_hash();
    receipt
}

fn validate_op_against_graph(
    graph: &GraphSnapshotContract,
    content: &str,
    op: &GraphMutationOp,
) -> Result<(), GraphPatchError> {
    if op.lo() > op.hi() || op.hi() > content.len() {
        return Err(GraphPatchError::InvalidSourceSpan(op.file().to_string()));
    }

    match op {
        GraphMutationOp::RemoveNode { path, file, lo, hi }
        | GraphMutationOp::RetypeIntent {
            path, file, lo, hi, ..
        }
        | GraphMutationOp::AddAttribute {
            path, file, lo, hi, ..
        } => {
            let node = graph
                .nodes
                .get(path)
                .ok_or_else(|| GraphPatchError::StaleOperation(path.clone()))?;
            if node.span.file != *file || node.span.lo != *lo || node.span.hi != *hi {
                return Err(GraphPatchError::StaleOperation(path.clone()));
            }
            if !node.span.is_valid_for(content) {
                return Err(GraphPatchError::InvalidSourceSpan(path.clone()));
            }
        }
        GraphMutationOp::RemoveEdge {
            relation,
            from,
            to,
            file,
            lo,
            hi,
        } => {
            let edge = GraphEdgeContract::new(relation.clone(), from.clone(), to.clone());
            if !graph.edges.contains(&edge) {
                return Err(GraphPatchError::StaleOperation(format!(
                    "{relation}:{from}->{to}"
                )));
            }
            let node = graph
                .nodes
                .get(from)
                .ok_or_else(|| GraphPatchError::StaleOperation(from.clone()))?;
            if node.span.file != *file || *lo < node.span.lo || *hi > node.span.hi {
                return Err(GraphPatchError::StaleOperation(from.clone()));
            }
        }
    }

    Ok(())
}

fn validate_no_overlaps(ops: &[GraphMutationOp]) -> Result<(), GraphPatchError> {
    for pair in ops.windows(2) {
        let a = &pair[0];
        let b = &pair[1];
        if a.file() == b.file() && a.hi() > b.lo() {
            return Err(GraphPatchError::OverlappingOperation(a.file().to_string()));
        }
    }
    Ok(())
}

fn hunk_for_op(content: &str, op: &GraphMutationOp) -> Result<String, GraphPatchError> {
    let old = &content[op.lo()..op.hi()];
    let new = match op {
        GraphMutationOp::RemoveNode { .. } | GraphMutationOp::RemoveEdge { .. } => String::new(),
        GraphMutationOp::RetypeIntent { new_label, .. } => {
            format!("#[canon_intent = \"{}\"]\n{old}", escape_attr(new_label))
        }
        GraphMutationOp::AddAttribute { attr, .. } => {
            format!("{}\n{old}", attr.trim())
        }
    };

    let old_line = line_number_at(content, op.lo());
    let old_count = line_count_for_hunk(old).max(1);
    let new_count = line_count_for_hunk(&new).max(if new.is_empty() { 0 } else { 1 });
    let mut hunk = format!("@@ -{old_line},{old_count} +{old_line},{new_count} @@\n");
    for line in old.lines() {
        hunk.push('-');
        hunk.push_str(line);
        hunk.push('\n');
    }
    if old.ends_with('\n') && old.lines().next().is_none() {
        hunk.push_str("-\n");
    }
    for line in new.lines() {
        hunk.push('+');
        hunk.push_str(line);
        hunk.push('\n');
    }
    Ok(hunk)
}

fn line_number_at(content: &str, byte_offset: usize) -> usize {
    content[..byte_offset.min(content.len())]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}

fn line_count_for_hunk(text: &str) -> usize {
    if text.is_empty() {
        0
    } else {
        text.lines().count().max(1)
    }
}

fn escape_attr(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn op_set_hash(ops: &[GraphMutationOp]) -> u64 {
    let mut h = 0x4752_4f50_5345_5401u64;
    for op in ops {
        h = mix(h, op.hash());
    }
    h.max(1)
}

fn sorted_ops(ops: &[GraphMutationOp]) -> Vec<GraphMutationOp> {
    let mut sorted = ops.to_vec();
    sorted.sort_by(|a, b| {
        a.file()
            .cmp(b.file())
            .then(a.lo().cmp(&b.lo()))
            .then(a.hi().cmp(&b.hi()))
            .then(a.hash().cmp(&b.hash()))
    });
    sorted
}

fn source_set_hash(sources: &[GraphSourceFile]) -> u64 {
    let mut sorted = sources.to_vec();
    sorted.sort_by(|a, b| a.path.cmp(&b.path));
    let mut h = 0x4752_5352_4353_4554u64;
    for source in sorted {
        h = mix_text(h, &source.path);
        h = mix_text(h, &source.content);
    }
    h.max(1)
}

fn hash_text(text: &str) -> u64 {
    mix_text(0xcbf2_9ce4_8422_2325u64, text).max(1)
}

fn mix_text(mut h: u64, text: &str) -> u64 {
    for byte in text.as_bytes() {
        h = mix(h, u64::from(*byte));
    }
    h.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (GraphSnapshotContract, Vec<GraphSourceFile>) {
        let content = "fn keep() {}\nfn remove() {\n    call_target();\n}\nfn tail() {}\n";
        let mut graph = GraphSnapshotContract::new(GRAPH_JSON_SCHEMA_VERSION, 0xfeed);
        graph.insert_node(GraphNodeContract::new(
            "demo::remove",
            "fn",
            "def-remove",
            GraphSourceSpan::new("src/lib.rs", 2, 1, 13, 45),
        ));
        graph.insert_node(GraphNodeContract::new(
            "demo::tail",
            "fn",
            "def-tail",
            GraphSourceSpan::new("src/lib.rs", 5, 1, 45, 57),
        ));
        graph.insert_edge(GraphEdgeContract::new(
            "call",
            "demo::remove",
            "demo::call_target",
        ));
        graph.set_intent("demo::remove", "mutation");
        (graph, vec![GraphSourceFile::new("src/lib.rs", content)])
    }

    #[test]
    fn graph_patch_removes_node_with_stable_receipt() {
        let (graph, sources) = fixture();
        let plan = generate_graph_patch(
            &graph,
            &sources,
            &[GraphMutationOp::RemoveNode {
                path: "demo::remove".into(),
                file: "src/lib.rs".into(),
                lo: 13,
                hi: 45,
            }],
        )
        .unwrap();

        assert!(plan.receipt.is_self_consistent());
        assert_eq!(plan.receipt.verdict, GraphMutationVerdict::Pass);
        assert_eq!(plan.receipt.hunk_count, 1);
        assert!(plan
            .diff
            .starts_with("--- a/src/lib.rs\n+++ b/src/lib.rs\n"));
        assert!(plan.diff.contains("-fn remove() {"));
        assert!(plan.diff.contains("-    call_target();"));
    }

    #[test]
    fn graph_patch_adds_intent_attribute_without_removing_node() {
        let (graph, sources) = fixture();
        let plan = generate_graph_patch(
            &graph,
            &sources,
            &[GraphMutationOp::RetypeIntent {
                path: "demo::remove".into(),
                file: "src/lib.rs".into(),
                lo: 13,
                hi: 45,
                new_label: "pure".into(),
            }],
        )
        .unwrap();

        assert!(plan.receipt.is_self_consistent());
        assert!(plan.diff.contains("+#[canon_intent = \"pure\"]"));
        assert!(plan.diff.contains("+fn remove() {"));
    }

    #[test]
    fn graph_patch_rejects_stale_node_span() {
        let (graph, sources) = fixture();
        let err = generate_graph_patch(
            &graph,
            &sources,
            &[GraphMutationOp::RemoveNode {
                path: "demo::remove".into(),
                file: "src/lib.rs".into(),
                lo: 14,
                hi: 45,
            }],
        )
        .unwrap_err();

        assert_eq!(err, GraphPatchError::StaleOperation("demo::remove".into()));
    }

    #[test]
    fn graph_patch_rejects_overlapping_operations() {
        let (graph, sources) = fixture();
        let err = generate_graph_patch(
            &graph,
            &sources,
            &[
                GraphMutationOp::RemoveNode {
                    path: "demo::remove".into(),
                    file: "src/lib.rs".into(),
                    lo: 13,
                    hi: 45,
                },
                GraphMutationOp::RemoveEdge {
                    relation: "call".into(),
                    from: "demo::remove".into(),
                    to: "demo::call_target".into(),
                    file: "src/lib.rs".into(),
                    lo: 27,
                    hi: 45,
                },
            ],
        )
        .unwrap_err();

        assert_eq!(
            err,
            GraphPatchError::OverlappingOperation("src/lib.rs".into())
        );
    }
    #[test]
    fn graph_mutation_receipt_accepts_landed_node_and_edge_removal() {
        let (old_graph, _sources) = fixture();
        let mut new_graph = old_graph.clone();
        new_graph.nodes.remove("demo::remove");
        new_graph.edges.remove(&GraphEdgeContract::new(
            "call",
            "demo::remove",
            "demo::call_target",
        ));
        new_graph.graph_hash = 0xbeef;

        let receipt = verify_graph_mutation_landing(
            &old_graph,
            &new_graph,
            &[
                GraphMutationOp::RemoveNode {
                    path: "demo::remove".into(),
                    file: "src/lib.rs".into(),
                    lo: 13,
                    hi: 45,
                },
                GraphMutationOp::RemoveEdge {
                    relation: "call".into(),
                    from: "demo::remove".into(),
                    to: "demo::call_target".into(),
                    file: "src/lib.rs".into(),
                    lo: 27,
                    hi: 45,
                },
            ],
        )
        .unwrap();

        assert!(receipt.is_self_consistent());
        assert_eq!(receipt.verdict, GraphMutationVerdict::Pass);
        assert_eq!(receipt.nodes_removed, 1);
        assert_eq!(receipt.edges_removed, 1);
        assert_eq!(receipt.missing_landing_count, 0);
    }

    #[test]
    fn graph_mutation_receipt_accepts_landed_intent_change() {
        let (old_graph, _sources) = fixture();
        let mut new_graph = old_graph.clone();
        new_graph.set_intent("demo::remove", "pure");
        new_graph.graph_hash = 0xbeef;

        let receipt = verify_graph_mutation_landing(
            &old_graph,
            &new_graph,
            &[GraphMutationOp::RetypeIntent {
                path: "demo::remove".into(),
                file: "src/lib.rs".into(),
                lo: 13,
                hi: 45,
                new_label: "pure".into(),
            }],
        )
        .unwrap();

        assert!(receipt.is_self_consistent());
        assert_eq!(receipt.verdict, GraphMutationVerdict::Pass);
        assert_eq!(receipt.intents_changed, 1);
    }

    #[test]
    fn graph_mutation_receipt_fails_when_operation_does_not_land() {
        let (old_graph, _sources) = fixture();
        let receipt = verify_graph_mutation_landing(
            &old_graph,
            &old_graph,
            &[GraphMutationOp::RemoveNode {
                path: "demo::remove".into(),
                file: "src/lib.rs".into(),
                lo: 13,
                hi: 45,
            }],
        )
        .unwrap();

        assert!(receipt.is_self_consistent());
        assert_eq!(receipt.verdict, GraphMutationVerdict::Fail);
        assert_eq!(receipt.missing_landing_count, 1);
        assert_eq!(receipt.nodes_removed, 0);
    }

    #[test]
    fn graph_patch_receipt_round_trips_through_ndjson() {
        let (graph, sources) = fixture();
        let plan = generate_graph_patch(
            &graph,
            &sources,
            &[GraphMutationOp::RemoveNode {
                path: "demo::remove".into(),
                file: "src/lib.rs".into(),
                lo: 13,
                hi: 45,
            }],
        )
        .unwrap();

        let encoded = encode_graph_patch_receipt_ndjson(&plan.receipt);
        let decoded = decode_graph_patch_receipt_ndjson(&encoded).unwrap();

        assert_eq!(decoded, plan.receipt);
        assert!(decode_graph_patch_receipt_ndjson("[1,2]").is_none());
    }

    #[test]
    fn graph_mutation_receipt_round_trips_through_ndjson_and_rejects_tamper() {
        let (old_graph, _sources) = fixture();
        let mut new_graph = old_graph.clone();
        new_graph.nodes.remove("demo::remove");
        new_graph.graph_hash = 0xbeef;

        let receipt = verify_graph_mutation_landing(
            &old_graph,
            &new_graph,
            &[GraphMutationOp::RemoveNode {
                path: "demo::remove".into(),
                file: "src/lib.rs".into(),
                lo: 13,
                hi: 45,
            }],
        )
        .unwrap();

        let encoded = encode_graph_mutation_receipt_ndjson(&receipt);
        assert_eq!(
            decode_graph_mutation_receipt_ndjson(&encoded).unwrap(),
            receipt
        );

        let tampered = encoded.replacen(&receipt.nodes_removed.to_string(), "9", 1);
        assert!(decode_graph_mutation_receipt_ndjson(&tampered).is_none());
    }

    #[test]
    fn graph_receipt_ledger_verifier_counts_valid_patch_and_mutation_rows() {
        let (old_graph, sources) = fixture();
        let patch = generate_graph_patch(
            &old_graph,
            &sources,
            &[GraphMutationOp::RemoveNode {
                path: "demo::remove".into(),
                file: "src/lib.rs".into(),
                lo: 13,
                hi: 45,
            }],
        )
        .unwrap();
        let mut new_graph = old_graph.clone();
        new_graph.nodes.remove("demo::remove");
        new_graph.graph_hash = 0xbeef;
        let mutation = verify_graph_mutation_landing(
            &old_graph,
            &new_graph,
            &[GraphMutationOp::RemoveNode {
                path: "demo::remove".into(),
                file: "src/lib.rs".into(),
                lo: 13,
                hi: 45,
            }],
        )
        .unwrap();

        let receipt = verify_graph_receipt_ledgers_ndjson(
            &encode_graph_patch_receipt_ndjson(&patch.receipt),
            &encode_graph_mutation_receipt_ndjson(&mutation),
        );

        assert!(receipt.is_self_consistent());
        assert_eq!(receipt.verdict, GraphMutationVerdict::Pass);
        assert_eq!(receipt.patch_receipt_count, 1);
        assert_eq!(receipt.mutation_receipt_count, 1);
        assert_eq!(receipt.passing_patch_count, 1);
        assert_eq!(receipt.passing_mutation_count, 1);
        assert_eq!(receipt.invalid_line_count, 0);
    }

    #[test]
    fn graph_receipt_ledger_verifier_rejects_tampered_rows() {
        let (graph, sources) = fixture();
        let patch = generate_graph_patch(
            &graph,
            &sources,
            &[GraphMutationOp::RemoveNode {
                path: "demo::remove".into(),
                file: "src/lib.rs".into(),
                lo: 13,
                hi: 45,
            }],
        )
        .unwrap();
        let mut encoded = encode_graph_patch_receipt_ndjson(&patch.receipt);
        encoded = encoded.replacen(&patch.receipt.hunk_count.to_string(), "99", 1);

        let receipt = verify_graph_receipt_ledgers_ndjson(&encoded, "");

        assert!(receipt.is_self_consistent());
        assert_eq!(receipt.verdict, GraphMutationVerdict::Fail);
        assert_eq!(receipt.patch_receipt_count, 0);
        assert_eq!(receipt.invalid_line_count, 1);
    }

    #[test]
    fn graph_mutation_ops_round_trip_with_escaped_fields() {
        let ops = vec![
            GraphMutationOp::AddAttribute {
                path: "demo::remove".into(),
                file: "src/lib.rs".into(),
                lo: 13,
                hi: 45,
                attr: "#[must_use = \"a|b\"]".into(),
            },
            GraphMutationOp::RemoveEdge {
                relation: "call".into(),
                from: "demo::remove".into(),
                to: "demo::call_target".into(),
                file: "src/lib.rs".into(),
                lo: 27,
                hi: 45,
            },
        ];

        let encoded = encode_graph_mutation_ops_ndjson(&ops);
        let decoded = decode_graph_mutation_ops_ndjson(&encoded).unwrap();
        let receipt = verify_graph_mutation_ops_ndjson(&encoded);

        assert_eq!(decoded, ops);
        assert!(receipt.is_self_consistent());
        assert_eq!(receipt.verdict, GraphMutationVerdict::Pass);
        assert_eq!(receipt.op_count, 2);
        assert_eq!(receipt.valid_row_count, 2);
        assert_eq!(receipt.invalid_row_count, 0);
    }

    #[test]
    fn graph_mutation_ops_verifier_rejects_tampered_row_hash() {
        let ops = vec![GraphMutationOp::RemoveNode {
            path: "demo::remove".into(),
            file: "src/lib.rs".into(),
            lo: 13,
            hi: 45,
        }];
        let encoded = encode_graph_mutation_ops_ndjson(&ops);
        let tampered = encoded.replacen("|", "|999", 1);

        assert!(decode_graph_mutation_ops_ndjson(&tampered).is_none());
        let receipt = verify_graph_mutation_ops_ndjson(&tampered);
        assert!(receipt.is_self_consistent());
        assert_eq!(receipt.verdict, GraphMutationVerdict::Fail);
        assert_eq!(receipt.valid_row_count, 0);
        assert_eq!(receipt.invalid_row_count, 1);
    }
}
