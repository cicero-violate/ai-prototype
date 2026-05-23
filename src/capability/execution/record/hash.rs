//! Hash and NDJSON helpers for execution receipt records.

use std::fs::File;
use std::path::Path;

use crate::capability::execution::record::types::{ToolEffectKind, ToolSandboxError};
use crate::kernel::mix;

pub(crate) fn sync_dir(path: &Path) -> Result<(), ToolSandboxError> {
    let dir = File::open(path).map_err(|_| ToolSandboxError::SandboxIo)?;
    dir.sync_all().map_err(|_| ToolSandboxError::SandboxIo)
}

pub(crate) fn bytes_hash(bytes: &[u8]) -> u64 {
    let mut h = 0x6a09e667f3bcc909u64;
    h = mix(h, bytes.len() as u64);
    for byte in bytes {
        h = mix(h, *byte as u64);
    }
    h.max(1)
}

pub(crate) fn string_hash(value: &str) -> u64 {
    bytes_hash(value.as_bytes())
}

pub(crate) fn parse_u64_ndjson_fields(line: &str) -> Result<Vec<u64>, ToolSandboxError> {
    let trimmed = line.trim();
    let body = trimmed
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or(ToolSandboxError::InvalidToolReceiptRecord)?;

    if body.trim().is_empty() {
        return Ok(Vec::new());
    }

    body.split(',')
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| ToolSandboxError::InvalidToolReceiptRecord)
        })
        .collect()
}

pub(crate) fn validate_u64_ndjson_header(
    fields: &[u64],
    expected_len: usize,
    schema_version: u64,
    record_kind: u64,
) -> Result<(), ToolSandboxError> {
    if fields.len() == expected_len && fields[0] == schema_version && fields[1] == record_kind {
        Ok(())
    } else {
        Err(ToolSandboxError::InvalidToolReceiptRecord)
    }
}

pub(crate) fn tool_effect_kind_from_u64(value: u64) -> Result<ToolEffectKind, ToolSandboxError> {
    match value {
        0 => Ok(ToolEffectKind::None),
        1 => Ok(ToolEffectKind::Artifact),
        2 => Ok(ToolEffectKind::Process),
        _ => Err(ToolSandboxError::InvalidToolReceiptRecord),
    }
}
