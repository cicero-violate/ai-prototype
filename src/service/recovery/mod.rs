//! Self-healing recovery — event bus subscription loop.
//!
//! `event_loop` polls the kernel TLog for `GateFailed` and `LeaseExpired`
//! wakeups and resets stale running nodes to Pending via the supervisor.
//! Bus loss is safe: replay(TLog) reconstructs every wakeup.

pub mod event_loop;
