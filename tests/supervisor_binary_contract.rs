use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    listener.local_addr().expect("local addr").port()
}

fn temp_tlog_dir(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("ai-supervisor-bin-{name}-{}", std::process::id()))
}

fn wait_for_json(url: &str) -> serde_json::Value {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        match reqwest::blocking::get(url) {
            Ok(response) if response.status().is_success() => {
                return response.json().expect("json response should decode");
            }
            _ => thread::sleep(Duration::from_millis(100)),
        }
    }
    panic!("endpoint did not become ready: {url}");
}

#[test]
fn supervisor_help_does_not_require_environment() {
    let output = Command::new(env!("CARGO_BIN_EXE_supervisor"))
        .arg("--help")
        .output()
        .expect("supervisor --help should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("usage: supervisor"));
    assert!(stdout.contains("/reload"));
    assert!(stdout.contains("/health"));
}

#[test]
fn supervisor_spawns_worker_and_reloads_generation() {
    let supervisor_port = free_port();
    let tlog_dir = temp_tlog_dir("reload");
    let _ = std::fs::remove_dir_all(&tlog_dir);

    let mut child = Command::new(env!("CARGO_BIN_EXE_supervisor"))
        .env("SUPERVISOR_PORT", supervisor_port.to_string())
        .env("AI_TLOG_DIR", &tlog_dir)
        .env("AI_WORKER_BIN", env!("CARGO_BIN_EXE_worker"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("supervisor should spawn");

    let health_url = format!("http://127.0.0.1:{supervisor_port}/health");
    let first = wait_for_json(&health_url);
    assert_eq!(first["ok"], true);
    assert_eq!(first["generation"], 1);
    let first_port = first["worker_port"].as_u64().expect("worker port") as u16;
    assert!(
        reqwest::blocking::get(format!("http://127.0.0.1:{first_port}/health/worker"))
            .expect("worker health response")
            .status()
            .is_success()
    );

    let reload_url = format!("http://127.0.0.1:{supervisor_port}/reload");
    let reload: serde_json::Value = reqwest::blocking::Client::new()
        .post(&reload_url)
        .send()
        .expect("reload response")
        .json()
        .expect("reload json");
    assert_eq!(reload["ok"], true);
    assert_eq!(reload["active"]["generation"], 2);
    let second_port = reload["active"]["worker_port"]
        .as_u64()
        .expect("new worker port") as u16;
    assert_ne!(first_port, second_port);

    let second = wait_for_json(&health_url);
    assert_eq!(second["ok"], true);
    assert_eq!(second["generation"], 2);
    assert_eq!(second["worker_port"].as_u64().unwrap() as u16, second_port);
    assert!(
        reqwest::blocking::get(format!("http://127.0.0.1:{second_port}/health/worker"))
            .expect("new worker health response")
            .status()
            .is_success()
    );

    #[cfg(unix)]
    {
        let status = Command::new("kill")
            .arg("-TERM")
            .arg(child.id().to_string())
            .status()
            .expect("kill should run");
        assert!(status.success());
    }
    #[cfg(not(unix))]
    {
        child.kill().expect("supervisor should kill");
    }
    let _ = child.wait().expect("supervisor should wait");
    let _ = std::fs::remove_dir_all(&tlog_dir);
}
