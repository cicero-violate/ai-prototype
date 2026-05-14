use std::env;
use std::path::PathBuf;

/// Runtime config for the agent loop, sourced from environment variables.
/// Mirrors chatgpt-agent-loop/config.mjs, extended with certification fields.
#[derive(Clone, Debug)]
pub struct AgentLoopConfig {
    // ── Loop ─────────────────────────────────────────────────────────────────
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
    // ── Certification (tlog / receipts) ───────────────────────────────────────
    /// Worker HTTP port. When set, each completed loop cycle runs an AgentCycle
    /// certification pass that stamps evidence to the tlog.
    pub worker_port: Option<u16>,
    /// Supervisor HTTP port. When set, the supervisor's /reload endpoint is
    /// called before each certification to give the cycle a fresh worker state.
    pub supervisor_port: Option<u16>,
    /// Maximum steps for the certification AgentCycle (default 30).
    pub cert_max_steps: u64,
    /// Specific sub-task domain for this agent (injected into the planning prompt).
    /// When None the agent uses only GOAL.md.
    pub domain: Option<String>,
    /// Success criterion matching `domain` (injected into the planning prompt).
    pub metric: Option<String>,
}

impl AgentLoopConfig {
    pub fn from_env() -> Self {
        let project_dir: PathBuf =
            env::var("PROJECT_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    // Derive from binary path: target/release/agent → ../../.. = project root.
                    // Falls back to current_dir() if exe path is unavailable.
                    env::current_exe()
                        .ok()
                        .and_then(|p| {
                            p.parent() // release/
                                .and_then(|p| p.parent()) // target/
                                .and_then(|p| p.parent()) // ai/
                                .map(|p| p.to_path_buf())
                        })
                        .unwrap_or_else(|| env::current_dir().unwrap_or_default())
                });
        let working_dir = project_dir.clone();
        let sse_chunks_dir = env::var("SSE_CHUNKS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| project_dir.join("agent_state").join("sse-chunks"));
        let worker_port = env::var("AI_WORKER_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok());
        let supervisor_port = env::var("SUPERVISOR_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok());
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
            worker_port,
            supervisor_port,
            cert_max_steps: env_u64("AI_CERT_MAX_STEPS", 30),
            domain: env::var("AI_AGENT_DOMAIN").ok().filter(|s| !s.is_empty()),
            metric: env::var("AI_AGENT_METRIC").ok().filter(|s| !s.is_empty()),
        }
    }
}

fn env_u32(key: &str, default: u32) -> u32 {
    env_parsed(key, default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    env_parsed(key, default)
}

fn env_parsed<T>(key: &str, default: T) -> T
where
    T: std::str::FromStr,
{
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
