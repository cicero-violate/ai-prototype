use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use crate::capability::CapabilityId;
use crate::kernel::{mix, Packet};

use super::request::ToolRequest;
use super::types::{ToolEffectKind, ToolSandboxError};

pub(crate) fn tool_command_hash(packet: Packet) -> u64 {
    let mut h = 0xbb67ae8584caa73bu64;
    h = mix(h, packet.objective_id);
    h = mix(h, packet.active_task_id);
    h.max(1)
}

pub(crate) fn tool_input_hash(packet: Packet) -> u64 {
    let mut h = 0x13198a2e03707344u64;
    h = mix(h, packet.objective_id);
    h = mix(h, packet.active_task_id);
    h = mix(h, packet.ready_tasks as u64);
    h = mix(h, packet.revision);
    h.max(1)
}

pub(crate) fn tool_output_hash(request: ToolRequest) -> u64 {
    let mut h = 0x243f6a8885a308d3u64;
    h = mix(h, request.objective_id);
    h = mix(h, request.task_id);
    h = mix(h, request.command_hash);
    h = mix(h, request.input_hash);
    h.max(1)
}

pub(crate) fn tool_effect_output_hash(before: Packet, after: Packet) -> u64 {
    let mut h = 0x4528_21e6_38d0_1377u64;
    h = mix_packet(h, before);
    h = mix_packet(h, after);
    h.max(1)
}

pub(crate) fn mix_packet(mut h: u64, packet: Packet) -> u64 {
    h = mix(h, packet.objective_id);
    h = mix(h, packet.objective_required_tasks as u64);
    h = mix(h, packet.objective_done_tasks as u64);
    h = mix(h, packet.ready_tasks as u64);
    h = mix(h, packet.active_task_id);
    h = mix(h, packet.artifact_id);
    h = mix(h, packet.parent_artifact_id);
    h = mix(h, packet.artifact_bytes);
    h = mix(h, packet.artifact_receipt_hash);
    h = mix(h, packet.artifact_lineage_hash);
    h = mix(h, packet.revision);
    h
}

pub(crate) fn tool_failure_hash(request: ToolRequest, exit_code: u8) -> u64 {
    let mut h = 0x9e3779b97f4a7c15u64;
    h = mix(h, request.contract_hash());
    h = mix(h, exit_code as u64);
    h.max(1)
}

pub(crate) fn artifact_relative_name(request: ToolRequest) -> String {
    format!(
        "artifact-{:016x}-{:016x}.canon",
        request.contract_hash(),
        request.input_hash
    )
}

pub(crate) fn sandbox_artifact_body(request: ToolRequest, before: Packet, after: Packet) -> Vec<u8> {
    format!(
        concat!(
            "canon-tool-artifact-v1\n",
            "capability={}\n",
            "registry_policy_hash={}\n",
            "request_hash={}\n",
            "objective_id={}\n",
            "task_id={}\n",
            "input_hash={}\n",
            "effect=MaterializeArtifact\n",
            "before_revision={}\n",
            "after_revision={}\n",
            "artifact_id={}\n",
            "artifact_receipt_hash={}\n"
        ),
        request.capability as u64,
        request.registry_policy_hash,
        request.contract_hash(),
        request.objective_id,
        request.task_id,
        request.input_hash,
        before.revision,
        after.revision,
        after.artifact_id,
        after.artifact_receipt_hash
    )
    .into_bytes()
}

pub(crate) fn ensure_relative_name(name: &str) -> Result<(), ToolSandboxError> {
    let path = Path::new(name);
    if path.components().count() == 1 && !name.is_empty() && !name.contains(std::path::MAIN_SEPARATOR)
    {
        Ok(())
    } else {
        Err(ToolSandboxError::PathEscapesSandbox)
    }
}

pub(crate) fn ensure_under(root: &Path, path: &Path) -> Result<(), ToolSandboxError> {
    let parent = path.parent().ok_or(ToolSandboxError::SandboxIo)?;
    fs::create_dir_all(parent).map_err(|_| ToolSandboxError::SandboxIo)?;
    let parent = parent
        .canonicalize()
        .map_err(|_| ToolSandboxError::SandboxIo)?;
    if parent.starts_with(root) {
        Ok(())
    } else {
        Err(ToolSandboxError::PathEscapesSandbox)
    }
}

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

pub(crate) fn path_hash(path: impl AsRef<[u8]>) -> u64 {
    bytes_hash(path.as_ref())
}


pub(crate) fn string_hash(value: &str) -> u64 {
    bytes_hash(value.as_bytes())
}

pub(crate) fn argv_hash(command: &str, args: &[&str]) -> u64 {
    let mut h = 0x1f83_d9ab_fb41_bd6bu64;
    h = mix(h, string_hash(command));
    h = mix(h, args.len() as u64);
    for arg in args {
        h = mix(h, string_hash(arg));
    }
    h.max(1)
}

pub(crate) fn locked_env_hash(env: &[(String, String)]) -> u64 {
    let mut h = 0x5be0_cd19_137e_2179u64;
    h = mix(h, env.len() as u64);
    for (key, value) in env {
        h = mix(h, string_hash(key));
        h = mix(h, string_hash(value));
    }
    h.max(1)
}

pub(crate) fn ensure_process_token(value: &str) -> Result<(), ToolSandboxError> {
    if value.is_empty() || value.contains('\0') || value.contains('=') {
        Err(ToolSandboxError::InvalidCommand)
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_relative_process_path(path: &str) -> Result<(), ToolSandboxError> {
    let path = Path::new(path);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(ToolSandboxError::PathEscapesSandbox);
    }
    Ok(())
}

pub(crate) fn bounded_file_bytes(path: &Path, max_bytes: u64) -> Result<Vec<u8>, ToolSandboxError> {
    let metadata = fs::metadata(path).map_err(|_| ToolSandboxError::SandboxIo)?;
    if metadata.len() > max_bytes {
        return Err(ToolSandboxError::OutputTooLarge);
    }

    let mut file = File::open(path).map_err(|_| ToolSandboxError::SandboxIo)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|_| ToolSandboxError::SandboxIo)?;
    if bytes.len() as u64 > max_bytes {
        return Err(ToolSandboxError::OutputTooLarge);
    }
    Ok(bytes)
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

pub(crate) fn capability_from_u64(value: u64) -> Result<CapabilityId, ToolSandboxError> {
    match value {
        7 => Ok(CapabilityId::Tooling),
        _ => Err(ToolSandboxError::InvalidToolReceiptRecord),
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