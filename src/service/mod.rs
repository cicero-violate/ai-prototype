//! Service-level orchestration surfaces.
//!
//! This layer owns autonomous agent loops and supervisor process lifecycle.
//! HTTP/MCP edges stay under `api`; reducer/TLog execution stays under `runtime`.
//! Scheduler and worker code stays as adapter code: runtime provides event and
//! recovery decisions, and the supervisor service owns lease-backed task state
//! plus projected plan status and evidence changes.

pub mod agent;
pub mod dispatch;
pub mod endpoints;
pub mod recovery;
pub mod scheduler;
pub mod supervisor;
