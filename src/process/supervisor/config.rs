//! Supervisor process configuration.
//!
//! Owns environment-derived settings for the supervisor runtime, worker process,
//! MCP transport, workspace, and OAuth store.

use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

use crate::process::endpoints::{mcp_connector_url_from_env, supervisor_port_from_env};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupervisorConfig {
    pub addr: SocketAddr,
    pub tlog_dir: PathBuf,
    pub worker_bin: PathBuf,
    pub mcp_worker_url: String,
    pub router_url: String,
    pub project_dir: PathBuf,
    pub mcp_connector_url: String,
    pub mcp_base_url: String,
    pub mcp_allowed_workspace_root: PathBuf,
    pub oauth_store_file: PathBuf,
    pub oauth_store_key: String,
}

impl SupervisorConfig {
    pub fn from_env() -> Result<Self, String> {
        let port = supervisor_port_from_env()?;
        let tlog_dir =
            PathBuf::from(env::var("AI_TLOG_DIR").unwrap_or_else(|_| "state/tlog".to_string()));
        let worker_bin = match env::var("AI_KERNEL_TLOG_BIN") {
            Ok(path) => PathBuf::from(path),
            Err(_) => default_worker_bin()?,
        };
        let mcp_worker_url = env::var("AI_MCP_WORKER_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:38469/mcp_worker".to_string());
        let router_url = env::var("CANON_OPENAI_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8082/v1".to_string());
        let project_dir = env::var("PROJECT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| env::current_dir().unwrap_or_default());
        let mcp_connector_url = mcp_connector_url_from_env(port);
        let mcp_base_url = env::var("AI_MCP_BASE_URL").unwrap_or_else(|_| {
            env::var("BASE_URL")
                .map(|base| format!("{}/ai", base.trim_end_matches('/')))
                .unwrap_or_else(|_| "https://cheese-server.duckdns.org/ai".to_string())
        });
        let mcp_allowed_workspace_root = env::var("AI_MCP_ALLOWED_WORKSPACE_ROOT")
            .or_else(|_| env::var("MCP_ALLOWED_WORKSPACE_ROOT"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| project_dir.clone());
        let oauth_store_file = env::var("AI_MCP_OAUTH_STORE_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp/canon-ai-mcp/oauth-store.enc"));
        let oauth_store_key = env::var("AI_MCP_OAUTH_STORE_KEY").unwrap_or_else(|_| {
            eprintln!("AI_MCP_OAUTH_STORE_KEY is not set; using a local development key");
            "canon-ai-mcp-dev-key".to_string()
        });
        Ok(Self {
            addr: SocketAddr::from(([0, 0, 0, 0], port)),
            tlog_dir,
            worker_bin,
            mcp_worker_url,
            router_url,
            project_dir,
            mcp_connector_url,
            mcp_base_url,
            mcp_allowed_workspace_root,
            oauth_store_file,
            oauth_store_key,
        })
    }
}

fn default_worker_bin() -> Result<PathBuf, String> {
    let exe = env::current_exe().map_err(|err| format!("current_exe failed: {err}"))?;
    let dir = exe
        .parent()
        .ok_or_else(|| "supervisor binary has no parent directory".to_string())?;
    let mut path = dir.join("kernel_tlog");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    Ok(path)
}
