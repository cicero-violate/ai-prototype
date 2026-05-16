//! `canon_spawn_agent` MCP tool argument contract.
//!
//! The capability layer owns the MCP tool schema and argument parsing. The
//! supervisor host performs actual worker spawning through `StatefulMcpHost`.

use serde_json::Value;

pub const CANON_SPAWN_AGENT_TOOL: &str = "canon_spawn_agent";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpawnAgentToolRequest {
    pub domain: String,
    pub metric: String,
    pub max_steps: u64,
}

impl SpawnAgentToolRequest {
    pub fn parse(args: &Value) -> Result<Self, String> {
        let domain = args
            .get("domain")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "domain is required".to_string())?
            .to_string();
        let metric = args
            .get("metric")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| "metric is required".to_string())?
            .to_string();
        let max_steps = args
            .get("max_steps")
            .and_then(Value::as_u64)
            .unwrap_or(20)
            .max(1)
            .min(100);
        Ok(Self {
            domain,
            metric,
            max_steps,
        })
    }
}
