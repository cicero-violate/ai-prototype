use std::env;
use std::path::PathBuf;

/// Runtime config for the agent loop, sourced from environment variables.
/// Mirrors chatgpt-agent-loop/config.mjs.
#[derive(Clone, Debug)]
pub struct AgentLoopConfig {
    pub execute_turns: u32,
    pub turn_retry_limit: u32,
    pub loop_sleep_ms: u64,
    pub agent_count: u32,
    pub project_dir: PathBuf,
    pub working_dir: PathBuf,
    pub sse_chunks_dir: PathBuf,
    pub mcp_connector_url: String,
    pub router_turn_max_ms: u64,
    pub router_first_capture_ms: u64,
    pub router_idle_ms: u64,
}

impl AgentLoopConfig {
    pub fn from_env() -> Self {
        let project_dir: PathBuf = env::var("PROJECT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| env::current_dir().unwrap_or_default());
        let working_dir = project_dir.clone();
        let sse_chunks_dir = env::var("SSE_CHUNKS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| project_dir.join("agent_state").join("sse-chunks"));
        Self {
            execute_turns: env_u32("EXECUTE_TURNS", 5),
            turn_retry_limit: env_u32("TURN_RETRY_LIMIT", 2),
            loop_sleep_ms: env_u64("LOOP_SLEEP_MS", 5000),
            agent_count: env_u32("AGENT_COUNT", 1),
            project_dir,
            working_dir,
            sse_chunks_dir,
            mcp_connector_url: env::var("MCP_CONNECTOR_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:4000".into()),
            router_turn_max_ms: env_u64("ROUTER_TURN_MAX_MS", 600_000),
            router_first_capture_ms: env_u64("ROUTER_FIRST_CAPTURE_MS", 60_000),
            router_idle_ms: env_u64("ROUTER_IDLE_MS", 2_500),
        }
    }
}

fn env_u32(key: &str, default: u32) -> u32 {
    env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}
