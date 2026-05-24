use std::env;
use std::path::PathBuf;

use crate::runtime::workspace::workspace_state_dir;
use crate::service::endpoints::{
    mcp_connector_url_from_env, supervisor_port_from_env, DEFAULT_SUPERVISOR_PORT,
};

pub const DEFAULT_EXECUTOR_COUNT: u32 = 1;
pub const MAX_EXECUTOR_COUNT: u32 = 5;

/// Runtime config for the agent loop, sourced from environment variables.
/// Mirrors chatgpt-agent-loop/config.mjs, extended with certification fields.
#[derive(Clone, Debug)]
pub struct AgentLoopConfig {
    // ── Loop ─────────────────────────────────────────────────────────────────
    pub execute_turns: u32,
    pub turn_retry_limit: u32,
    pub loop_sleep_ms: u64,
    pub agent_count: u32,
    pub executor_count: u32,
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
    /// Browser-router base URL (from CANON_OPENAI_BASE_URL, /v1 suffix stripped).
    /// Used by the DAG scheduler to cap the wave size by available CDP tabs.
    pub browser_router_url: Option<String>,
    /// Maximum steps for the certification AgentCycle (default 30).
    pub cert_max_steps: u64,
    /// Specific sub-task domain for this agent (injected into the planning prompt).
    /// When None the agent uses only GOAL.md.
    pub domain: Option<String>,
    /// Success criterion matching `domain` (injected into the planning prompt).
    pub metric: Option<String>,
    /// Plan node this spawned agent owns. DAG-scheduled children use this to
    /// append evidence to their node without editing shared planning files.
    pub plan_node_id: Option<String>,
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
            .unwrap_or_else(|_| {
                workspace_state_dir(&project_dir)
                    .join("agent_state")
                    .join("sse-chunks")
            });
        let worker_port = env::var("AI_WORKER_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok());
        let supervisor_port = supervisor_port_from_env().unwrap_or(DEFAULT_SUPERVISOR_PORT);
        let browser_router_url = env::var("CANON_OPENAI_BASE_URL").ok().map(|url| {
            let url = url.trim_end_matches('/').to_string();
            url.strip_suffix("/v1").unwrap_or(&url).to_string()
        });
        Self {
            execute_turns: env_parsed::<u32>("EXECUTE_TURNS", 2),
            turn_retry_limit: env_parsed::<u32>("TURN_RETRY_LIMIT", 2),
            loop_sleep_ms: env_parsed::<u64>("LOOP_SLEEP_MS", 5000),
            agent_count: env_parsed::<u32>("AGENT_COUNT", 1),
            executor_count: env_parsed::<u32>("CANON_EXECUTOR_COUNT", DEFAULT_EXECUTOR_COUNT)
                .clamp(1, MAX_EXECUTOR_COUNT),
            project_dir,
            working_dir,
            sse_chunks_dir,
            mcp_connector_url: mcp_connector_url_from_env(supervisor_port),
            router_turn_max_ms: env_parsed::<u64>("ROUTER_TURN_MAX_MS", 600_000),
            router_first_capture_ms: env_parsed::<u64>("ROUTER_FIRST_CAPTURE_MS", 60_000),
            router_idle_ms: env_parsed::<u64>("ROUTER_IDLE_MS", 2_500),
            worker_port,
            supervisor_port: Some(supervisor_port),
            browser_router_url,
            cert_max_steps: env_parsed::<u64>("AI_CERT_MAX_STEPS", 30),
            domain: env::var("AI_AGENT_DOMAIN").ok().filter(|s| !s.is_empty()),
            metric: env::var("AI_AGENT_METRIC").ok().filter(|s| !s.is_empty()),
            plan_node_id: env::var("AI_AGENT_PLAN_NODE_ID")
                .ok()
                .filter(|s| !s.is_empty()),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    const NUMERIC_ENV_KEYS: [&str; 8] = [
        "EXECUTE_TURNS",
        "TURN_RETRY_LIMIT",
        "AGENT_COUNT",
        "LOOP_SLEEP_MS",
        "ROUTER_TURN_MAX_MS",
        "ROUTER_FIRST_CAPTURE_MS",
        "ROUTER_IDLE_MS",
        "AI_CERT_MAX_STEPS",
    ];

    fn env_test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn restore_env(snapshot: &[(&str, Option<String>)]) {
        for (key, value) in snapshot {
            match value {
                Some(value) => env::set_var(key, value),
                None => env::remove_var(key),
            }
        }
    }

    #[test]
    fn agent_loop_config_from_env_preserves_typed_numeric_defaults() {
        let _guard = env_test_lock()
            .lock()
            .expect("env test lock should not be poisoned");
        let snapshot: Vec<_> = NUMERIC_ENV_KEYS
            .iter()
            .map(|key| (*key, env::var(key).ok()))
            .collect();

        for key in NUMERIC_ENV_KEYS {
            env::remove_var(key);
        }

        env::set_var("EXECUTE_TURNS", "11");
        env::set_var("TURN_RETRY_LIMIT", "12");
        env::set_var("AGENT_COUNT", "13");
        env::set_var("LOOP_SLEEP_MS", "1400");
        env::set_var("ROUTER_TURN_MAX_MS", "1500");
        env::set_var("ROUTER_FIRST_CAPTURE_MS", "1600");
        env::set_var("ROUTER_IDLE_MS", "1700");
        env::set_var("AI_CERT_MAX_STEPS", "18");

        let parsed = AgentLoopConfig::from_env();
        assert_eq!(parsed.execute_turns, 11);
        assert_eq!(parsed.turn_retry_limit, 12);
        assert_eq!(parsed.agent_count, 13);
        assert_eq!(parsed.loop_sleep_ms, 1400);
        assert_eq!(parsed.router_turn_max_ms, 1500);
        assert_eq!(parsed.router_first_capture_ms, 1600);
        assert_eq!(parsed.router_idle_ms, 1700);
        assert_eq!(parsed.cert_max_steps, 18);

        env::set_var("EXECUTE_TURNS", "not-a-u32");
        env::set_var("LOOP_SLEEP_MS", "not-a-u64");

        let defaults = AgentLoopConfig::from_env();
        assert_eq!(defaults.execute_turns, 2);
        assert_eq!(defaults.loop_sleep_ms, 5000);

        restore_env(&snapshot);
    }
}
