//! Supervisor process configuration.
//!
//! Owns environment-derived settings for the supervisor runtime, worker process,
//! MCP transport, workspace, and OAuth store.

use std::env;
use std::net::SocketAddr;
use std::path::{Component, Path, PathBuf};

use crate::service::endpoints::{mcp_connector_url_from_env, supervisor_port_from_env};

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
        let project_dir = env::var("PROJECT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| env::current_dir().unwrap_or_default());
        let tlog_dir = env::var("AI_TLOG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| project_dir.join("state/tlog"));
        let tlog_dir = if tlog_dir.is_relative() {
            project_dir.join(&tlog_dir)
        } else {
            tlog_dir
        };
        let worker_bin = match env::var("AI_KERNEL_TLOG_BIN") {
            Ok(path) => resolve_worker_bin_override(PathBuf::from(path), &project_dir),
            Err(_) => default_worker_bin()?,
        };
        let mcp_worker_url = env::var("AI_MCP_WORKER_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:38469/mcp_worker".to_string());
        let router_url = env::var("CANON_OPENAI_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8082/v1".to_string());
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
            .unwrap_or_else(|_| default_oauth_store_file(&project_dir));
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

fn default_oauth_store_file(project_dir: &std::path::Path) -> PathBuf {
    project_dir.join("state").join("oauth-store.enc")
}

fn default_worker_bin() -> Result<PathBuf, String> {
    let exe = env::current_exe().map_err(|err| format!("current_exe failed: {err}"))?;
    let dir = exe
        .parent()
        .ok_or_else(|| "supervisor binary has no parent directory".to_string())?;
    let mut sibling = dir.join("kernel_tlog");
    if cfg!(windows) {
        sibling.set_extension("exe");
    }
    if sibling.exists() {
        return Ok(sibling);
    }

    let mut path = workspace_target_dir().join(profile_from_exe(&exe));
    path.push("kernel_tlog");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    Ok(path)
}

fn resolve_worker_bin_override(path: PathBuf, project_dir: &Path) -> PathBuf {
    if path.is_absolute() {
        return path;
    }

    if let Some(target_relative) = strip_parent_target_prefix(&path) {
        return project_dir.join("target").join(target_relative);
    }

    let project_relative = project_dir.join(&path);
    if project_relative.exists() {
        return project_relative;
    }

    project_relative
}

fn strip_parent_target_prefix(path: &Path) -> Option<PathBuf> {
    let mut components = path.components();
    match (components.next(), components.next()) {
        (Some(Component::ParentDir), Some(Component::Normal(target))) if target == "target" => {
            Some(components.as_path().to_path_buf())
        }
        _ => None,
    }
}

fn workspace_target_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|path| path.join("target"))
        .unwrap_or_else(|| PathBuf::from("target"))
}

fn profile_from_exe(exe: &Path) -> &'static str {
    if exe
        .components()
        .any(|component| component.as_os_str() == "release")
    {
        "release"
    } else {
        "debug"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_oauth_store_file_is_workspace_local() {
        let project_dir = PathBuf::from("/workspace/project");
        assert_eq!(
            default_oauth_store_file(&project_dir),
            PathBuf::from("/workspace/project/state/oauth-store.enc")
        );
    }

    #[test]
    fn worker_bin_override_legacy_parent_target_is_workspace_local() {
        let project_dir = PathBuf::from("/workspace/project");
        assert_eq!(
            resolve_worker_bin_override(PathBuf::from("../target/debug/kernel_tlog"), &project_dir),
            PathBuf::from("/workspace/project/target/debug/kernel_tlog")
        );
    }

    #[test]
    fn worker_bin_override_relative_path_stays_project_relative() {
        let project_dir = PathBuf::from("/workspace/project");
        assert_eq!(
            resolve_worker_bin_override(PathBuf::from("bin/kernel_tlog"), &project_dir),
            PathBuf::from("/workspace/project/bin/kernel_tlog")
        );
    }

    #[test]
    fn default_worker_bin_falls_back_to_workspace_target() {
        let path = default_worker_bin().expect("default worker bin path");
        assert!(
            path.ends_with("target/debug/kernel_tlog")
                || path.ends_with("target/release/kernel_tlog")
        );
        assert!(
            !path.ends_with("ai/target/debug/kernel_tlog")
                && !path.ends_with("ai/target/release/kernel_tlog"),
            "default path should use the workspace target dir when no sibling binary exists: {}",
            path.display()
        );
    }
}
