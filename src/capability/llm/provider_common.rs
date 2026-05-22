//! Shared helpers for local LLM provider adapters.
//!
//! Keep provider-specific wire schemas in the provider modules. This module only
//! contains byte/string primitives that were duplicated across providers.

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub(crate) fn split_http_response(response: &str) -> Option<(u16, &str)> {
    let (head, body) = response.split_once("\r\n\r\n")?;
    let status = head
        .lines()
        .next()?
        .split_whitespace()
        .nth(1)?
        .parse::<u16>()
        .ok()?;
    Some((status, body))
}

pub(crate) fn encode_u64_fields_ndjson(fields: &[u64]) -> String {
    format!(
        "[{}]",
        fields
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(crate) fn parse_u64_fields(line: &str) -> Option<Vec<u64>> {
    let body = line.trim().strip_prefix('[')?.strip_suffix(']')?;
    if body.trim().is_empty() {
        return Some(Vec::new());
    }
    body.split(',')
        .map(|raw| raw.trim().parse::<u64>().ok())
        .collect()
}

pub(crate) fn append_ndjson_record(path: &Path, encoded_record: &str) -> io::Result<()> {
    ensure_record_parent(path)?;

    {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(file, "{}", encoded_record)?;
        file.sync_all()?;
    }

    sync_parent_dir(path)
}

pub(crate) fn load_ndjson_records<T, E, F>(
    path: &Path,
    record_tag: u64,
    parse_fields: impl Fn(&str) -> Result<Vec<u64>, E>,
    decode: F,
) -> Result<Vec<T>, E>
where
    E: From<io::Error>,
    F: Fn(&[u64]) -> Result<T, E>,
{
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let fields = parse_fields(&line)?;
        if fields.len() >= 2 && fields[1] == record_tag {
            records.push(decode(&fields)?);
        }
    }
    Ok(records)
}

fn ensure_record_parent(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn sync_parent_dir(path: &Path) -> io::Result<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() {
        return Ok(());
    }
    let dir = File::open(parent)?;
    dir.sync_all()
}

pub(crate) fn json_escape(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_http_response_extracts_status_and_body() {
        assert_eq!(
            split_http_response("HTTP/1.1 200 OK\r\nX: y\r\n\r\nbody"),
            Some((200, "body"))
        );
        assert_eq!(split_http_response("not-http"), None);
    }

    #[test]
    fn u64_fields_round_trip() {
        let encoded = encode_u64_fields_ndjson(&[1, 2, 3]);
        assert_eq!(encoded, "[1,2,3]");
        assert_eq!(parse_u64_fields(&encoded), Some(vec![1, 2, 3]));
    }

    #[test]
    fn json_escape_handles_common_sequences() {
        assert_eq!(json_escape("a\"b\\c\n"), "a\\\"b\\\\c\\n");
    }
}
