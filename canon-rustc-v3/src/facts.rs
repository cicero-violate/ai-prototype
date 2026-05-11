//! Canonical fact vocabulary and heuristic callee classification.

pub const NODE_KINDS: &[&str] = &["fn", "trait", "impl", "struct", "enum", "ty_alias"];
pub const EDGE_RELATIONS: &[&str] = &[
    "call", "impl", "mut", "io", "unsafe", "panic", "alloc", "use", "similar", "phase", "provider",
];
pub const RISK_RELATIONS: &[&str] = &["mut", "io", "unsafe", "panic", "alloc", "similar", "phase"];

pub fn allowed_relation(relation: &str) -> bool {
    EDGE_RELATIONS.contains(&relation)
}

pub fn risk_relation(relation: &str) -> bool {
    RISK_RELATIONS.contains(&relation)
}

pub fn callee_relations(callee: &str) -> Vec<&'static str> {
    let lower = callee.to_ascii_lowercase();
    let mut out = Vec::new();

    if contains_any(
        &lower,
        &[
            "::fs::",
            "::io::",
            "::file::",
            "read_to_string",
            "write_all",
            "std::process::",
        ],
    ) {
        out.push("io");
    }
    if contains_any(&lower, &["panic", "unwrap_failed"])
        || lower.ends_with("::unwrap")
        || lower.ends_with("::expect")
    {
        out.push("panic");
    }
    if contains_any(
        &lower,
        &["alloc", "exchange_malloc", "box_new", "vec::", "raw_vec"],
    ) {
        out.push("alloc");
    }

    out
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callee_classifier_marks_current_heuristics() {
        assert_eq!(callee_relations("std::fs::read_to_string"), vec!["io"]);
        assert_eq!(
            callee_relations("core::option::Option::unwrap"),
            vec!["panic"]
        );
        assert_eq!(
            callee_relations("alloc::raw_vec::RawVec::new"),
            vec!["alloc"]
        );
    }

    #[test]
    fn risk_relations_are_canonical_edges() {
        for relation in RISK_RELATIONS {
            assert!(allowed_relation(relation));
        }
    }

    #[test]
    fn relation_vocabularies_match_schema_16_contract() {
        assert_eq!(
            NODE_KINDS,
            &["fn", "trait", "impl", "struct", "enum", "ty_alias"]
        );
        assert_eq!(
            EDGE_RELATIONS,
            &[
                "call", "impl", "mut", "io", "unsafe", "panic", "alloc", "use", "similar", "phase",
                "provider",
            ]
        );
        assert_eq!(
            RISK_RELATIONS,
            &["mut", "io", "unsafe", "panic", "alloc", "similar", "phase"]
        );
    }

    #[test]
    fn auto_refactor_relations_are_canonical_edges() {
        assert!(allowed_relation("similar"));
        assert!(allowed_relation("phase"));
        assert!(allowed_relation("provider"));
        assert!(risk_relation("similar"));
        assert!(risk_relation("phase"));
        assert!(!risk_relation("provider"));
    }
}
