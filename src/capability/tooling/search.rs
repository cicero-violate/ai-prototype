//! Repository search capability — fuzzy filename search and BM25 content search.
//!
//! Ported from canon-tools-search. Provides deterministic local search before
//! LLM calls to reduce prompt guessing and improve evidence collection.
//!
//! Results are bounded by `limit` (capped at `MAX_SEARCH_RESULTS`) and confined
//! to paths under the provided root.

use std::cmp::Ordering;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use nucleo_matcher::{
    pattern::{AtomKind, CaseMatching, Normalization, Pattern},
    Config, Matcher, Utf32Str,
};

/// Hard cap on results to keep searches bounded.
pub const MAX_SEARCH_RESULTS: usize = 200;

#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Path relative to the search root.
    pub path: PathBuf,
    /// Absolute path to the match.
    pub full_path: PathBuf,
    pub score: u32,
    /// Optional text snippet for content-based searches.
    pub snippet: Option<String>,
}

/// A receipt produced after a search operation, suitable for logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchReceipt {
    pub query_hash: u64,
    pub result_hash: u64,
    pub result_count: usize,
}

impl SearchReceipt {
    fn new(query: &str, results: &[SearchResult]) -> Self {
        let query_hash = hash_str(query);
        let mut h = 0x9e3779b97f4a7c15u64;
        for r in results {
            h = hash_mix(h, hash_str(r.path.to_string_lossy().as_ref()));
        }
        SearchReceipt {
            query_hash,
            result_hash: h.max(1),
            result_count: results.len(),
        }
    }
}

fn hash_mix(h: u64, v: u64) -> u64 {
    let h = h ^ v;
    let h = h.wrapping_mul(0x9e3779b97f4a7c15);
    h ^ (h >> 31)
}

fn hash_str(s: &str) -> u64 {
    let bytes = s.as_bytes();
    let mut h = 0x6a09e667f3bcc909u64;
    h = hash_mix(h, bytes.len() as u64);
    for &b in bytes {
        h = hash_mix(h, b as u64);
    }
    h.max(1)
}

/// Fuzzy filename search over files under `root` using nucleo-matcher.
/// Returns top `limit` matches (capped at `MAX_SEARCH_RESULTS`) ordered by
/// descending score then path.
pub fn search_files(
    query: &str,
    root: &Path,
    limit: usize,
) -> Result<(Vec<SearchResult>, SearchReceipt), String> {
    if query.trim().is_empty() {
        return Ok((Vec::new(), SearchReceipt::new(query, &[])));
    }

    let limit = limit.min(MAX_SEARCH_RESULTS);
    let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
    let pattern = Pattern::new(
        query,
        CaseMatching::Smart,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );

    let walker = WalkBuilder::new(root)
        .hidden(false)
        .ignore(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build();

    let mut results = Vec::new();
    for entry in walker.flatten() {
        let file_type = match entry.file_type() {
            Some(ft) => ft,
            None => continue,
        };
        if !file_type.is_file() {
            continue;
        }
        let full_path = entry.path().to_path_buf();
        let rel = match full_path.strip_prefix(root) {
            Ok(r) => r.to_path_buf(),
            Err(_) => continue,
        };
        let rel_str = match rel.to_str() {
            Some(s) => s,
            None => continue,
        };
        let mut buf = Vec::new();
        let haystack = Utf32Str::new(rel_str, &mut buf);
        let Some(score) = pattern.score(haystack, &mut matcher) else {
            continue;
        };
        results.push(SearchResult {
            path: rel,
            full_path,
            score,
            snippet: None,
        });
    }

    results.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => a.path.cmp(&b.path),
        other => other,
    });
    if results.len() > limit {
        results.truncate(limit);
    }

    let receipt = SearchReceipt::new(query, &results);
    Ok((results, receipt))
}

/// BM25-style content search (lightweight, in-memory per call).
/// Tokenises text files and ranks by BM25 term frequencies.
/// Returns top `limit` matches (capped at `MAX_SEARCH_RESULTS`).
pub fn search_files_bm25(
    query: &str,
    root: &Path,
    limit: usize,
) -> Result<(Vec<SearchResult>, SearchReceipt), String> {
    if query.trim().is_empty() {
        return Ok((Vec::new(), SearchReceipt::new(query, &[])));
    }

    let limit = limit.min(MAX_SEARCH_RESULTS);
    let query_terms: Vec<String> = tokenize(query);
    if query_terms.is_empty() {
        return Ok((Vec::new(), SearchReceipt::new(query, &[])));
    }

    let mut documents: Vec<(PathBuf, PathBuf, Vec<String>)> = Vec::new();
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .ignore(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build();

    for entry in walker.flatten() {
        let file_type = match entry.file_type() {
            Some(ft) => ft,
            None => continue,
        };
        if !file_type.is_file() {
            continue;
        }
        let full_path = entry.path().to_path_buf();
        let rel = match full_path.strip_prefix(root) {
            Ok(r) => r.to_path_buf(),
            Err(_) => continue,
        };
        if !is_text_candidate(&rel) {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&full_path) else {
            continue;
        };
        let tokens = tokenize(&content);
        if tokens.is_empty() {
            continue;
        }
        documents.push((rel, full_path, tokens));
    }

    if documents.is_empty() {
        return Ok((Vec::new(), SearchReceipt::new(query, &[])));
    }

    use std::collections::HashMap;
    let mut doc_freq: HashMap<String, usize> = HashMap::new();
    for (_, _, tokens) in &documents {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for t in tokens {
            if seen.insert(t) {
                *doc_freq.entry(t.clone()).or_insert(0) += 1;
            }
        }
    }

    let doc_count = documents.len() as f32;
    let k1 = 1.5_f32;
    let b = 0.75_f32;

    let mut scored: Vec<SearchResult> = Vec::new();
    for (rel, full, tokens) in documents {
        let doc_len = tokens.len() as f32;
        let mut tf: HashMap<&str, usize> = HashMap::new();
        for t in &tokens {
            *tf.entry(t).or_insert(0) += 1;
        }
        let avg_dl = doc_len;
        let mut score = 0.0_f32;
        for term in &query_terms {
            let df = *doc_freq.get(term).unwrap_or(&0) as f32;
            if df == 0.0 {
                continue;
            }
            let idf = ((doc_count - df + 0.5) / (df + 0.5) + 1.0).ln();
            let f = *tf.get(term.as_str()).unwrap_or(&0) as f32;
            let numerator = f * (k1 + 1.0);
            let denom = f + k1 * (1.0 - b + b * (doc_len / avg_dl.max(1.0)));
            score += idf * (numerator / denom);
        }
        if score > 0.0 {
            let snippet = make_snippet(&tokens, &query_terms);
            scored.push(SearchResult {
                path: rel,
                full_path: full,
                score: (score * 1000.0) as u32,
                snippet,
            });
        }
    }

    scored.sort_by(|a, b| match b.score.cmp(&a.score) {
        Ordering::Equal => a.path.cmp(&b.path),
        other => other,
    });
    if scored.len() > limit {
        scored.truncate(limit);
    }

    let receipt = SearchReceipt::new(query, &scored);
    Ok((scored, receipt))
}

fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '/' && c != '.')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_ascii_lowercase())
        .collect()
}

fn is_text_candidate(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("rs" | "toml" | "md" | "txt" | "json" | "yaml" | "yml" | "ndjson")
    )
}

fn make_snippet(tokens: &[String], terms: &[String]) -> Option<String> {
    let window = 30;
    for i in 0..tokens.len() {
        if terms.contains(&tokens[i]) {
            let start = i.saturating_sub(3);
            let end = (i + window).min(tokens.len());
            return Some(tokens[start..end].join(" "));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn scratch_dir() -> std::path::PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("ai-search-test-{}-{n}", std::process::id()));
        fs::create_dir_all(&dir).expect("scratch dir creation should succeed");
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn search_files_empty_query_returns_empty() {
        let dir = scratch_dir();
        let (results, receipt) = search_files("", &dir, 10).expect("search should succeed");
        assert!(results.is_empty());
        assert_eq!(receipt.result_count, 0);
        cleanup(&dir);
    }

    #[test]
    fn search_files_finds_matching_rs_file() {
        let dir = scratch_dir();
        fs::write(dir.join("main.rs"), b"fn main() {}").expect("write should succeed");
        fs::write(dir.join("other.toml"), b"[package]").expect("write should succeed");

        let (results, receipt) = search_files("main", &dir, 10).expect("search should succeed");
        // nucleo fuzzy-matches "main" against "main.rs" — must appear in results
        assert!(
            !results.is_empty(),
            "expected at least one result for query 'main'"
        );
        assert!(results
            .iter()
            .any(|r| r.path.to_str().unwrap().contains("main")));
        assert_ne!(receipt.query_hash, 0);
        assert_ne!(receipt.result_hash, 0);
        cleanup(&dir);
    }

    #[test]
    fn search_files_bm25_finds_content_match() {
        let dir = scratch_dir();
        fs::write(dir.join("kernel.rs"), b"pub struct Kernel { state: State }")
            .expect("write should succeed");
        fs::write(dir.join("unrelated.rs"), b"fn noop() {}").expect("write should succeed");

        let (results, receipt) =
            search_files_bm25("Kernel state", &dir, 10).expect("search should succeed");
        assert!(!results.is_empty());
        assert!(results[0].path.to_str().unwrap().contains("kernel.rs"));
        assert_ne!(receipt.query_hash, 0);
        cleanup(&dir);
    }

    #[test]
    fn search_respects_limit_cap() {
        let dir = scratch_dir();
        for i in 0..10 {
            fs::write(dir.join(format!("file{i}.rs")), b"fn foo() {}")
                .expect("write should succeed");
        }
        let (results, _) = search_files("file", &dir, 3).expect("search should succeed");
        assert!(results.len() <= 3);
        cleanup(&dir);
    }

    #[test]
    fn receipt_is_deterministic_for_same_query_and_results() {
        let dir = scratch_dir();
        fs::write(dir.join("foo.rs"), b"fn foo() {}").expect("write should succeed");

        let (r1, receipt1) = search_files("foo", &dir, 10).expect("search should succeed");
        let (r2, receipt2) = search_files("foo", &dir, 10).expect("search should succeed");

        assert_eq!(r1.len(), r2.len());
        assert_eq!(receipt1, receipt2);
        cleanup(&dir);
    }
}
