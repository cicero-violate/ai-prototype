//! Patch generator: GraphMutationOp list → unified diff + intent overrides.
//!
//! Source patches are expressed as standard unified diffs suitable for
//! `patch(1)` or the `apply_patch` MCP tool.  Intent overrides are a separate
//! map written directly into the mutated graph.json.
//!
//! Algorithm
//! ---------
//! 1. Validate each op against the current graph (stale-op guard on lo/hi).
//! 2. Overlap guard: reject any op whose byte range intersects another op's
//!    range in the same file (e.g. removing both an impl block and one of its
//!    methods).  Rejected ops are reported in `stale_ops`.
//! 3. Group valid ops by target file.
//! 4. For each file: read content, convert byte offsets to line numbers,
//!    build `EditOp`s, sort ascending, merge overlapping context windows into
//!    hunks, render as unified diff.
//! 5. Collect `RetypeIntent` ops into a separate override map.

use crate::graph::CrateGraph;
use crate::ops::GraphMutationOp;
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const CTX: usize = 3;

// ─── public output ───────────────────────────────────────────────────────────

pub struct PatchSet {
    /// Unified diff text for all source file changes.  Empty if no source ops.
    pub source_patch: String,
    /// Intent label overrides to apply directly to graph.json.
    pub intent_overrides: BTreeMap<String, String>,
    /// Ops rejected because their expected_lo/hi did not match the graph.
    pub stale_ops: Vec<String>,
}

// ─── line-level edit ─────────────────────────────────────────────────────────

struct EditOp {
    /// First line to affect in the old file (0-indexed).
    old_start: usize,
    /// Number of old lines to remove (0 for pure insertion).
    old_count: usize,
    /// Lines to insert in their place.
    new_lines: Vec<String>,
}

/// A resolved byte range for overlap detection.
struct ResolvedRange {
    path: String,
    file: String,
    lo: u32,
    hi: u32,
}

// ─── entry point ─────────────────────────────────────────────────────────────

pub fn generate(graph: &CrateGraph, ops: &[GraphMutationOp], source_root: &Path) -> Result<PatchSet> {
    let mut file_ops: BTreeMap<String, Vec<EditOp>> = BTreeMap::new();
    let mut intent_overrides = BTreeMap::new();
    let mut stale_ops = Vec::new();

    // Pre-flight: resolve byte ranges for all source-touching ops and detect
    // overlaps.  An impl block and one of its methods will both have spans that
    // overlap; patching both would produce an invalid diff.
    let ranges: Vec<ResolvedRange> = ops
        .iter()
        .filter_map(|op| match op {
            GraphMutationOp::RemoveNode { path, .. } | GraphMutationOp::AddAttribute { path, .. } => {
                let span = graph.nodes.get(path.as_str())?.def.as_ref()?;
                Some(ResolvedRange {
                    path: path.clone(),
                    file: span.file.clone(),
                    lo: span.lo,
                    hi: span.hi,
                })
            }
            GraphMutationOp::RetypeIntent { .. } => None,
        })
        .collect();

    let overlapping: std::collections::BTreeSet<String> = {
        let mut bad = std::collections::BTreeSet::new();
        for i in 0..ranges.len() {
            for j in (i + 1)..ranges.len() {
                let a = &ranges[i];
                let b = &ranges[j];
                if a.file == b.file && a.lo < b.hi && b.lo < a.hi {
                    bad.insert(a.path.clone());
                    bad.insert(b.path.clone());
                }
            }
        }
        bad
    };

    for path in &overlapping {
        stale_ops.push(format!(
            "overlap: {path} byte range intersects another op in the same file — \
             remove either the container (impl) or the member (fn), not both"
        ));
    }

    for op in ops {
        if overlapping.contains(op.path()) {
            continue; // already recorded in stale_ops above
        }
        match op {
            GraphMutationOp::RemoveNode { path, expected_lo, expected_hi } => {
                let span = resolve_span(graph, path)?;
                if span.lo != *expected_lo || span.hi != *expected_hi {
                    stale_ops.push(format!(
                        "RemoveNode({path}): expected lo={expected_lo}/hi={expected_hi}, \
                         found lo={}/hi={}",
                        span.lo, span.hi
                    ));
                    continue;
                }
                let content = read_file(source_root, &span.file)?;
                let (start, end) = span_to_line_range(&content, span.lo as usize, span.hi as usize);
                file_ops.entry(span.file.clone()).or_default().push(EditOp {
                    old_start: start,
                    old_count: end - start + 1,
                    new_lines: vec![],
                });
            }

            GraphMutationOp::AddAttribute { path, attr, expected_lo } => {
                let span = resolve_span(graph, path)?;
                if span.lo != *expected_lo {
                    stale_ops.push(format!(
                        "AddAttribute({path}): expected lo={expected_lo}, found lo={}",
                        span.lo
                    ));
                    continue;
                }
                let content = read_file(source_root, &span.file)?;
                let insert_line = lo_to_line(&content, span.lo as usize);
                let indent = line_indent(&content, insert_line);
                file_ops.entry(span.file.clone()).or_default().push(EditOp {
                    old_start: insert_line,
                    old_count: 0,
                    new_lines: vec![format!("{indent}{attr}")],
                });
            }

            GraphMutationOp::RetypeIntent { path, new_label } => {
                if !graph.nodes.contains_key(path.as_str()) {
                    anyhow::bail!("RetypeIntent: node not found in graph: {path}");
                }
                intent_overrides.insert(path.clone(), new_label.clone());
            }
        }
    }

    let mut diff_parts = Vec::new();
    for (file, mut edits) in file_ops {
        edits.sort_by_key(|e| e.old_start);
        let content = read_file(source_root, &file)?;
        let lines: Vec<&str> = content.lines().collect();
        let diff = render_file_diff(&file, &lines, &edits);
        if !diff.is_empty() {
            diff_parts.push(diff);
        }
    }

    Ok(PatchSet {
        source_patch: diff_parts.join(""),
        intent_overrides,
        stale_ops,
    })
}

// ─── helpers ─────────────────────────────────────────────────────────────────

fn resolve_span<'g>(
    graph: &'g CrateGraph,
    path: &str,
) -> Result<&'g crate::graph::SourceSpan> {
    let node = graph
        .nodes
        .get(path)
        .with_context(|| format!("node not found in graph: {path}"))?;
    node.def
        .as_ref()
        .with_context(|| format!("node has no source span (generated/external?): {path}"))
}

fn read_file(root: &Path, rel: &str) -> Result<String> {
    let path = root.join(rel);
    fs::read_to_string(&path)
        .with_context(|| format!("cannot read source file: {}", path.display()))
}

/// Convert a byte offset to its 0-indexed line number.
fn lo_to_line(content: &str, lo: usize) -> usize {
    let lo = lo.min(content.len());
    content.as_bytes()[..lo]
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
}

/// Return the [start_line, end_line] (inclusive, 0-indexed) that fully cover
/// the byte span [lo, hi].
fn span_to_line_range(content: &str, lo: usize, hi: usize) -> (usize, usize) {
    (lo_to_line(content, lo), lo_to_line(content, hi))
}

/// Return the leading whitespace of the given 0-indexed line.
fn line_indent(content: &str, line_idx: usize) -> String {
    let line = content.lines().nth(line_idx).unwrap_or("");
    let trimmed = line.trim_start();
    line[..line.len() - trimmed.len()].to_string()
}

// ─── unified diff renderer ───────────────────────────────────────────────────

fn render_file_diff(filename: &str, old_lines: &[&str], edits: &[EditOp]) -> String {
    if edits.is_empty() {
        return String::new();
    }

    // Group edits whose context windows overlap into single hunks.
    let groups = group_edits(edits);
    let mut result = format!("--- a/{filename}\n+++ b/{filename}\n");
    let mut new_line_delta: isize = 0;

    for group in &groups {
        let first = &group[0];
        let last = &group[group.len() - 1];

        let ctx_start = first.old_start.saturating_sub(CTX);
        let raw_ctx_end = last.old_start + last.old_count + CTX;
        let ctx_end = raw_ctx_end.min(old_lines.len());

        // Count lines in the hunk for the @@ header.
        let old_count = ctx_end - ctx_start;
        let delta_this_hunk: isize = group
            .iter()
            .map(|e| e.new_lines.len() as isize - e.old_count as isize)
            .sum();
        let new_count = (old_count as isize + delta_this_hunk) as usize;

        let old_start_1 = ctx_start + 1;
        let new_start_1 = (ctx_start as isize + 1 + new_line_delta) as usize;

        result.push_str(&format!(
            "@@ -{old_start_1},{old_count} +{new_start_1},{new_count} @@\n"
        ));

        let mut cursor = ctx_start;
        for edit in group {
            // context lines up to this edit
            for i in cursor..edit.old_start {
                if i < old_lines.len() {
                    result.push_str(&format!(" {}\n", old_lines[i]));
                }
            }
            // removed lines
            for i in edit.old_start..edit.old_start + edit.old_count {
                if i < old_lines.len() {
                    result.push_str(&format!("-{}\n", old_lines[i]));
                }
            }
            // inserted lines
            for line in &edit.new_lines {
                result.push_str(&format!("+{line}\n"));
            }
            cursor = edit.old_start + edit.old_count;
        }
        // trailing context
        for i in cursor..ctx_end {
            if i < old_lines.len() {
                result.push_str(&format!(" {}\n", old_lines[i]));
            }
        }

        new_line_delta += delta_this_hunk;
    }

    result
}

/// Group edits into runs whose expanded context windows overlap.
fn group_edits(edits: &[EditOp]) -> Vec<Vec<&EditOp>> {
    let mut groups: Vec<Vec<&EditOp>> = Vec::new();
    for edit in edits {
        if let Some(last_group) = groups.last_mut() {
            let prev = last_group.last().unwrap();
            let prev_ctx_end = prev.old_start + prev.old_count + CTX;
            let this_ctx_start = edit.old_start.saturating_sub(CTX);
            if this_ctx_start <= prev_ctx_end {
                last_group.push(edit);
                continue;
            }
        }
        groups.push(vec![edit]);
    }
    groups
}

// ─── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lo_to_line_counts_newlines() {
        let src = "line0\nline1\nline2\n";
        assert_eq!(lo_to_line(src, 0), 0);
        assert_eq!(lo_to_line(src, 6), 1);
        assert_eq!(lo_to_line(src, 12), 2);
    }

    #[test]
    fn span_to_line_range_covers_span() {
        let src = "fn a() {}\nfn b() {}\nfn c() {}\n";
        // "fn b" starts at byte 10, ends at byte 19
        let (s, e) = span_to_line_range(src, 10, 19);
        assert_eq!(s, 1);
        assert_eq!(e, 1);
    }

    #[test]
    fn render_remove_hunk() {
        let lines = vec!["a", "b", "c", "d", "e", "f", "g"];
        let edits = vec![EditOp { old_start: 3, old_count: 1, new_lines: vec![] }];
        let diff = render_file_diff("src/foo.rs", &lines, &edits);
        assert!(diff.contains("--- a/src/foo.rs"));
        assert!(diff.contains("-d\n"));
        assert!(diff.contains(" c\n"));
        assert!(diff.contains(" e\n"));
    }

    #[test]
    fn render_insert_hunk() {
        let lines = vec!["fn foo() {}", "fn bar() {}"];
        let edits = vec![EditOp {
            old_start: 1,
            old_count: 0,
            new_lines: vec!["#[must_use]".into()],
        }];
        let diff = render_file_diff("src/lib.rs", &lines, &edits);
        assert!(diff.contains("+#[must_use]\n"));
        assert!(diff.contains(" fn bar() {}"));
    }

    fn count_hunks(diff: &str) -> usize {
        diff.lines().filter(|l| l.starts_with("@@")).count()
    }

    #[test]
    fn nearby_edits_merged_into_one_hunk() {
        let lines: Vec<&str> = (0..20).map(|_| "x").collect();
        // Two edits 4 lines apart — within 2*CTX=6, should merge.
        let edits = vec![
            EditOp { old_start: 5, old_count: 1, new_lines: vec![] },
            EditOp { old_start: 9, old_count: 1, new_lines: vec![] },
        ];
        let diff = render_file_diff("f.rs", &lines, &edits);
        let n = count_hunks(&diff);
        assert_eq!(n, 1, "expected 1 merged hunk, got {n}");
    }

    #[test]
    fn distant_edits_produce_separate_hunks() {
        let lines: Vec<&str> = (0..40).map(|_| "x").collect();
        // Two edits 20 lines apart — well outside 2*CTX=6, separate hunks.
        let edits = vec![
            EditOp { old_start: 2, old_count: 1, new_lines: vec![] },
            EditOp { old_start: 30, old_count: 1, new_lines: vec![] },
        ];
        let diff = render_file_diff("f.rs", &lines, &edits);
        let n = count_hunks(&diff);
        assert_eq!(n, 2, "expected 2 hunks, got {n}");
    }

    #[test]
    fn stale_op_is_reported_not_panicked() {
        use crate::graph::{CrateGraph, GraphMeta, GraphNode, SourceSpan};
        use crate::ops::GraphMutationOp;
        use std::collections::BTreeMap;

        let mut nodes = BTreeMap::new();
        nodes.insert(
            "foo::bar".to_string(),
            GraphNode {
                def_id: "0:1".into(),
                path: "foo::bar".into(),
                kind: "fn".into(),
                def: Some(SourceSpan { file: "src/lib.rs".into(), line: 1, col: 0, lo: 0, hi: 10 }),
                source_text: None,
                sig: None,
                fields: vec![],
            },
        );
        let graph = CrateGraph {
            meta: GraphMeta { schema_version: 11, ..Default::default() },
            nodes,
            ..Default::default()
        };

        let ops = vec![GraphMutationOp::RemoveNode {
            path: "foo::bar".into(),
            expected_lo: 999, // wrong — stale
            expected_hi: 10,
        }];

        let root = std::env::temp_dir();
        let patch_set = generate(&graph, &ops, &root).unwrap();
        assert_eq!(patch_set.stale_ops.len(), 1);
        assert!(patch_set.source_patch.is_empty());
    }

    #[test]
    fn overlapping_ops_are_both_rejected() {
        use crate::graph::{CrateGraph, GraphMeta, GraphNode, SourceSpan};
        use crate::ops::GraphMutationOp;
        use std::collections::BTreeMap;

        // impl node at 0..100, fn node at 20..60 — nested, so overlapping.
        let mut nodes = BTreeMap::new();
        nodes.insert("foo::MyImpl".into(), GraphNode {
            def_id: "0:1".into(), path: "foo::MyImpl".into(), kind: "impl".into(),
            def: Some(SourceSpan { file: "src/lib.rs".into(), line: 1, col: 0, lo: 0, hi: 100 }),
            source_text: None, sig: None, fields: vec![],
        });
        nodes.insert("foo::MyImpl::method".into(), GraphNode {
            def_id: "0:2".into(), path: "foo::MyImpl::method".into(), kind: "fn".into(),
            def: Some(SourceSpan { file: "src/lib.rs".into(), line: 3, col: 4, lo: 20, hi: 60 }),
            source_text: None, sig: None, fields: vec![],
        });
        let graph = CrateGraph {
            meta: GraphMeta { schema_version: 11, ..Default::default() },
            nodes, ..Default::default()
        };

        let ops = vec![
            GraphMutationOp::RemoveNode { path: "foo::MyImpl".into(),        expected_lo: 0,  expected_hi: 100 },
            GraphMutationOp::RemoveNode { path: "foo::MyImpl::method".into(), expected_lo: 20, expected_hi: 60  },
        ];

        let root = std::env::temp_dir();
        let patch_set = generate(&graph, &ops, &root).unwrap();
        // Both ops should be flagged — neither should produce a patch.
        assert_eq!(patch_set.stale_ops.len(), 2, "expected both ops rejected: {:?}", patch_set.stale_ops);
        assert!(patch_set.source_patch.is_empty());
    }
}
