//! Stable supervisor binary placeholder.
//!
//! Step 1 wires the binary target and dependencies. Lifecycle management is
//! implemented in the later supervisor step from `plan.md`.

#![forbid(unsafe_code)]

use std::env;

fn main() {
    if env::args().any(|arg| arg == "--help" || arg == "-h") {
        println!("usage: supervisor [--help]");
        println!("environment: SUPERVISOR_PORT, AI_TLOG_DIR, AI_WORKER_BIN, AI_MCP_WORKER_URL");
        return;
    }

    println!("supervisor target is declared; lifecycle implementation is pending");
}
