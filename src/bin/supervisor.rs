//! Stable supervisor binary.
//!
//! Thin entrypoint for the supervisor runtime.

#![forbid(unsafe_code)]

#[tokio::main]
async fn main() {
    if let Err(err) = ai::supervisor_run().await {
        eprintln!("supervisor error: {err}");
        std::process::exit(1);
    }
}
