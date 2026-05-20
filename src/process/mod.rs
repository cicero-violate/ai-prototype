//! Process-level orchestration surfaces.
//!
//! This layer owns autonomous agent loops and supervisor process lifecycle.
//! HTTP/MCP edges stay under `api`; reducer/TLog execution stays under `runtime`.

pub mod agent;
pub mod endpoints;
pub mod supervisor;
