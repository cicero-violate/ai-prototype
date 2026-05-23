//! Legacy MCP-named transcript compatibility facade.
//!
//! The implementation lives under `runtime::action_transcript`. This module
//! preserves old imports and the legacy transcript path helper during the
//! migration.

use std::path::{Path, PathBuf};

pub use crate::runtime::action_transcript::{
    append_action_transcript as append_mcp_transcript,
    replay_action_transcripts as replay_mcp_transcripts,
    ActionTranscriptRecord as McpTranscriptRecord,
    ACTION_TRANSCRIPT_RECORD_CALL_RESULT as MCP_TRANSCRIPT_RECORD_CALL_RESULT,
    ACTION_TRANSCRIPT_SCHEMA_VERSION as MCP_TRANSCRIPT_SCHEMA_VERSION,
};

pub fn mcp_transcript_path(workspace_root: &Path) -> PathBuf {
    crate::runtime::action_transcript::legacy_mcp_transcript_path(workspace_root)
}
