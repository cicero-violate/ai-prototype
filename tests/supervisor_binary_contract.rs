use ai::{Command, CommandEnvelope, EvidenceSubmission};
use std::net::TcpListener;
use std::process::{Command as ProcessCommand, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    listener.local_addr().expect("local addr").port()
}

fn temp_tlog_dir(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("ai-supervisor-bin-{name}-{}", std::process::id()))
}

fn worker_command_body(command_id: u64, payload_hash: u64) -> serde_json::Value {
    let submission = EvidenceSubmission::with_payload(
        ai::GateId::Invariant,
        ai::Evidence::InvariantProof,
        true,
        payload_hash,
    );
    let envelope = CommandEnvelope::new(command_id, Command::SubmitEvidence(submission));
    serde_json::json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": "SubmitEvidence",
        "payload": {
            "gate": "Invariant",
            "evidence": "InvariantProof",
            "passed": true,
            "effect": "None",
            "payload_hash": payload_hash
        }
    })
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
    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_supervisor"))
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

    let mut child = ProcessCommand::new(env!("CARGO_BIN_EXE_supervisor"))
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

    let first_state_url = format!("http://127.0.0.1:{first_port}/v1/state");
    let initial_state = wait_for_json(&first_state_url);
    let initial_tlog_len = initial_state["tlog_len"]
        .as_u64()
        .expect("initial tlog len");

    let command_url = format!("http://127.0.0.1:{first_port}/v1/command");
    let command_response: serde_json::Value = reqwest::blocking::Client::new()
        .post(&command_url)
        .json(&worker_command_body(31, 0x7a11))
        .send()
        .expect("worker command response")
        .json()
        .expect("worker command json");
    assert_eq!(command_response["ok"], true);
    assert_eq!(command_response["request_id"], 1);
    assert_eq!(command_response["disposition"], "accepted");

    let after_command = wait_for_json(&first_state_url);
    assert!(after_command["tlog_len"].as_u64().expect("tlog len") > initial_tlog_len);
    let after_command_tlog_len = after_command["tlog_len"].as_u64().expect("tlog len");
    let tlog_path = tlog_dir.join("worker-tlog.ndjson");
    assert!(std::fs::read_dir(&tlog_dir)
        .expect("tlog dir should exist")
        .any(|entry| entry.expect("tlog entry").path().is_file()));
    let tlog_len_after_first_command = std::fs::metadata(&tlog_path)
        .expect("worker tlog should exist")
        .len();

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
    assert!(
        reqwest::blocking::get(format!("http://127.0.0.1:{first_port}/health/worker"))
            .expect("retired worker should remain reachable during drain")
            .status()
            .is_success(),
        "hot reload should keep the retired worker alive during the drain window"
    );

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
    let second_state_url = format!("http://127.0.0.1:{second_port}/v1/state");
    let second_state = wait_for_json(&second_state_url);
    assert_eq!(
        second_state["tlog_len"].as_u64().expect("second tlog len"),
        after_command_tlog_len
    );

    let second_command_url = format!("http://127.0.0.1:{second_port}/v1/command");
    let replay_response: serde_json::Value = reqwest::blocking::Client::new()
        .post(&second_command_url)
        .json(&worker_command_body(31, 0x7a11))
        .send()
        .expect("replayed worker command response")
        .json()
        .expect("replayed worker command json");
    assert_eq!(replay_response["ok"], true);
    assert_eq!(replay_response["disposition"], "replayed");
    assert_eq!(replay_response["event_seq"], command_response["event_seq"]);
    assert_eq!(
        replay_response["event_hash"],
        command_response["event_hash"]
    );
    assert_eq!(
        wait_for_json(&second_state_url)["tlog_len"]
            .as_u64()
            .expect("second tlog len after replay"),
        after_command_tlog_len
    );
    assert_eq!(
        std::fs::metadata(&tlog_path)
            .expect("worker tlog should still exist")
            .len(),
        tlog_len_after_first_command
    );

    #[cfg(unix)]
    {
        let status = ProcessCommand::new("kill")
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
