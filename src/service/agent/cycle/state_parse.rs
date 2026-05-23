//! Minimal worker-state JSON field extraction helpers.

// ---- State parsing -------------------------------------------------------

pub(super) fn extract_phase(state_json: &str) -> Option<String> {
    extract_json_string(state_json, "phase")
}

pub(super) fn extract_json_string(json: &str, field: &str) -> Option<String> {
    let key = format!("\"{field}\":");
    let start = json.find(&key)?;
    let rest = json[start + key.len()..].trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

pub(super) fn extract_tlog_len(state_json: &str) -> Option<usize> {
    let key = "\"tlog_len\":";
    let start = state_json.find(key)?;
    let rest = state_json[start + key.len()..].trim_start();
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}
