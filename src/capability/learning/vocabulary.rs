//! Canonical vocabulary families for learning artifacts.
//!
//! Pure data — no I/O. Shared by the runtime writer and the Python training pipeline.
//! Placement: capability/learning owns vocabulary normalization; kernel owns only
//! durable event primitives; api and refactor/compiler must not carry this vocabulary.

pub const TASK_WORDS: &[&str] = &[
    "fix", "refactor", "move", "rename", "extract", "validate", "generate", "rollback",
];

pub const SYMBOL_WORDS: &[&str] = &[
    "scheduler",
    "route",
    "mailbox",
    "kernel",
    "dispatch",
    "codec",
    "runtime",
];

pub const TOOL_WORDS: &[&str] = &[
    "apply_patch",
    "shell",
    "cargo_check",
    "cargo_test",
    "inspect",
    "validate",
];

pub const OUTCOME_WORDS: &[&str] = &[
    "succeeded",
    "failed",
    "reverted",
    "blocked",
    "timed_out",
    "accepted",
];

pub const COST_WORDS: &[&str] = &[
    "fan_in",
    "fan_out",
    "churn",
    "penalty",
    "risk",
    "barrier",
    "rollback_cost",
];

/// Extract task vocabulary words that appear in a task title.
pub fn task_words_from_title(title: &str) -> Vec<&'static str> {
    TASK_WORDS
        .iter()
        .copied()
        .filter(|&word| title.contains(word))
        .collect()
}

/// Extract tool vocabulary words present in a tool call sequence.
pub fn tool_words_from_sequence<S: AsRef<str>>(tools: &[S]) -> Vec<&'static str> {
    TOOL_WORDS
        .iter()
        .copied()
        .filter(|&word| tools.iter().any(|t| t.as_ref() == word))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_words_extracted_from_title() {
        let words = task_words_from_title("fix HirRecord deserialization and refactor codec");
        assert!(words.contains(&"fix"));
        assert!(words.contains(&"refactor"));
        assert!(!words.contains(&"move"));
    }

    #[test]
    fn tool_words_extracted_from_sequence() {
        let seq = ["apply_patch", "cargo_check", "get_landmarks"];
        let words = tool_words_from_sequence(&seq);
        assert!(words.contains(&"apply_patch"));
        assert!(words.contains(&"cargo_check"));
        assert!(!words.contains(&"shell"));
    }
}
