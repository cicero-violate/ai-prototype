//! Reloadable worker binary placeholder.
//!
//! Step 1 wires the binary target and dependencies. HTTP serving is implemented
//! in the later worker/server steps from `plan.md`.

#![forbid(unsafe_code)]

use std::env;

fn main() {
    if env::args().any(|arg| arg == "--help" || arg == "-h") {
        println!("usage: worker [--help]");
        println!("environment: PORT, AI_TLOG_DIR, AI_MCP_WORKER_URL, AI_WORKER_GENERATION");
        return;
    }

    println!("worker target is declared; HTTP implementation is pending");
}
