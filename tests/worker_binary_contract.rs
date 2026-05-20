use std::io::Read;
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    listener.local_addr().expect("local addr").port()
}

fn temp_tlog_dir(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("ai-worker-bin-{name}-{}", std::process::id()))
}

fn kernel_tlog_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kernel_tlog").expect("kernel_tlog binary path")
}

#[test]
fn worker_help_does_not_require_environment() {
    let output = Command::new(kernel_tlog_bin())
        .arg("--help")
        .output()
        .expect("kernel_tlog --help should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("usage: kernel_tlog"));
    assert!(stdout.contains("/health/worker"));
    assert!(stdout.contains("canon-agent.tlog.ndjson"));
}

#[test]
fn worker_process_serves_health_and_initializes_tlog() {
    let port = free_port();
    let tlog_dir = temp_tlog_dir("health");
    let _ = std::fs::remove_dir_all(&tlog_dir);

    let mut child = Command::new(kernel_tlog_bin())
        .env("PORT", port.to_string())
        .env("AI_TLOG_DIR", &tlog_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("worker should spawn");

    let url = format!("http://127.0.0.1:{port}/health/worker");
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut body = None;
    while Instant::now() < deadline {
        match reqwest::blocking::get(&url) {
            Ok(response) if response.status().is_success() => {
                body = Some(response.text().expect("health body should read"));
                break;
            }
            _ => thread::sleep(Duration::from_millis(50)),
        }
    }

    if body.as_deref() != Some("ok") {
        let mut stderr = String::new();
        if let Some(mut pipe) = child.stderr.take() {
            let _ = pipe.read_to_string(&mut stderr);
        }
        let _ = child.kill();
        let _ = child.wait();
        panic!("worker health did not become ready; stderr={stderr}");
    }

    let tlog_path = tlog_dir.join("canon-agent.tlog.ndjson");
    assert!(
        std::fs::metadata(&tlog_path)
            .expect("tlog should exist")
            .len()
            > 0
    );
    assert!(
        !tlog_dir.join("worker-tlog.ndjson").exists(),
        "worker-tlog.ndjson is a legacy compatibility name, not the only runtime log"
    );

    child.kill().expect("worker should kill");
    let status = child.wait().expect("worker should wait");
    assert!(!status.success());
    let _ = std::fs::remove_dir_all(&tlog_dir);
}
