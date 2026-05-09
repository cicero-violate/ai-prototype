use ai::{
    build_router, resume_durable_runtime, tick, ApiTransportLedger, ApiTransportSession, Command,
    CommandEnvelope, CommandLedger, EvidenceSubmission, EvidenceSubmissionDto, McpCallReceipt,
    McpCallRequest, RuntimeConfig, State, StateDto, TLog, WorkerAppState,
};
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

fn tlog_path(name: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let dir = std::path::PathBuf::from("target/test-tmp/api-server-tlogs");
    std::fs::create_dir_all(&dir).expect("api server tlog fixture dir should exist");
    dir.join(format!(
        "ai-api-server-{name}-{}-{nanos}.ndjson",
        std::process::id(),
    ))
}

fn missing_parent_tlog_path(name: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let dir = std::path::PathBuf::from("target/test-tmp/api-server-tlogs").join(format!(
        "missing-parent-{name}-{}-{nanos}",
        std::process::id(),
    ));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("stale missing-parent fixture dir should remove");
    }
    dir.join("tlog.ndjson")
}

fn invariant_payload(payload_hash: u64) -> EvidenceSubmissionDto {
    EvidenceSubmissionDto {
        gate: "Invariant".to_string(),
        evidence: "InvariantProof".to_string(),
        passed: true,
        effect: Some("None".to_string()),
        payload_hash,
    }
}

fn execution_receipt_payload(payload_hash: u64) -> EvidenceSubmissionDto {
    EvidenceSubmissionDto {
        gate: "Execution".to_string(),
        evidence: "ExecutionReceipt".to_string(),
        passed: true,
        effect: Some("None".to_string()),
        payload_hash,
    }
}

fn command_body(command_id: u64, submission: EvidenceSubmission) -> serde_json::Value {
    let envelope = CommandEnvelope::new(command_id, Command::SubmitEvidence(submission));
    serde_json::json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": "SubmitEvidence",
        "payload": invariant_payload(submission.payload_hash),
    })
}

fn evidence_batch_body(command_id: u64, submissions: &[EvidenceSubmission]) -> serde_json::Value {
    let envelope = CommandEnvelope::new(
        command_id,
        Command::SubmitEvidenceBatch(submissions.to_vec()),
    );
    let payload: Vec<_> = submissions
        .iter()
        .map(|submission| invariant_payload(submission.payload_hash))
        .collect();
    serde_json::json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": "SubmitEvidenceBatch",
        "payload": payload,
    })
}

fn command_body_with_payload(
    command_id: u64,
    submission: EvidenceSubmission,
    payload: EvidenceSubmissionDto,
) -> serde_json::Value {
    let envelope = CommandEnvelope::new(command_id, Command::SubmitEvidence(submission));
    serde_json::json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": "SubmitEvidence",
        "payload": payload,
    })
}

async fn response_json<T: serde::de::DeserializeOwned>(response: axum::response::Response) -> T {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    serde_json::from_slice(&body).expect("body should deserialize")
}

async fn ok_response_json<T: serde::de::DeserializeOwned>(response: axum::response::Response) -> T {
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    serde_json::from_slice(&body).expect("body should deserialize")
}

#[tokio::test]
async fn worker_health_route_returns_ok() {
    let path = tlog_path("health");
    let app = build_router(WorkerAppState::new_default(&path));
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health/worker")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&body[..], b"ok");
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn state_route_is_read_only() {
    let path = tlog_path("state");
    let state = WorkerAppState::new_default(&path);
    let app = build_router(state.clone());

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/state")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let second = app
        .oneshot(
            Request::builder()
                .uri("/v1/state")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::OK);
    let first: StateDto = response_json(first).await;
    let second: StateDto = response_json(second).await;
    assert_eq!(first, second);
    assert_eq!(state.snapshot().unwrap().tlog_len, 0);
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn command_route_uses_transport_session_and_persists_tlog() {
    let path = tlog_path("command");
    let _ = std::fs::remove_file(&path);
    let cfg = RuntimeConfig::default();
    let mut initial_state = State::default();
    let mut initial_tlog = TLog::default();
    assert!(tick(&mut initial_state, &mut initial_tlog, cfg).is_ok());
    let session = ApiTransportSession::from_parts(
        initial_state,
        initial_tlog,
        cfg,
        CommandLedger::default(),
        ApiTransportLedger::default(),
    )
    .expect("initialized session should verify");
    let state = WorkerAppState::new(session, &path);
    let app = build_router(state.clone());
    let submission = EvidenceSubmission::with_payload(
        ai::GateId::Invariant,
        ai::Evidence::InvariantProof,
        true,
        0xabc,
    );
    let body = command_body(11, submission);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/command")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let response: ai::CommandResponseDto = ok_response_json(response).await;
    assert!(response.ok);
    assert_eq!(response.request_id, 1);
    assert_eq!(response.disposition, "accepted");
    assert_eq!(state.snapshot().unwrap().tlog_len, 3);
    assert!(std::fs::metadata(&path).unwrap().len() > 0);
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn command_route_maps_tlog_persistence_failure_to_internal_server_error() {
    let path = missing_parent_tlog_path("persistence-failure");
    let parent = path.parent().expect("fixture path should have parent");
    assert!(!parent.exists());

    let cfg = RuntimeConfig::default();
    let mut initial_state = State::default();
    let mut initial_tlog = TLog::default();
    assert!(tick(&mut initial_state, &mut initial_tlog, cfg).is_ok());
    let session = ApiTransportSession::from_parts(
        initial_state,
        initial_tlog,
        cfg,
        CommandLedger::default(),
        ApiTransportLedger::default(),
    )
    .expect("initialized session should verify");
    let state = WorkerAppState::new(session, &path);
    let app = build_router(state.clone());
    let submission = EvidenceSubmission::with_payload(
        ai::GateId::Invariant,
        ai::Evidence::InvariantProof,
        true,
        0xabc,
    );
    let body = command_body(91, submission);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/command")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let error: ai::ErrorDto = response_json(response).await;
    assert!(!error.ok);
    assert_eq!(error.error, "TlogIo");
    assert!(!path.exists());
    assert_eq!(state.snapshot().unwrap().tlog_len, 3);
}

#[tokio::test]
async fn command_route_replays_after_durable_resume_without_appending_tlog() {
    let path = tlog_path("durable-resume-replay");
    let _ = std::fs::remove_file(&path);
    let cfg = RuntimeConfig::default();
    let mut initial_state = State::default();
    let mut initial_tlog = TLog::default();
    assert!(tick(&mut initial_state, &mut initial_tlog, cfg).is_ok());
    let initial_session = ApiTransportSession::from_parts(
        initial_state,
        initial_tlog,
        cfg,
        CommandLedger::default(),
        ApiTransportLedger::default(),
    )
    .expect("initialized session should verify");
    let first_state = WorkerAppState::new(initial_session, &path);
    let first_app = build_router(first_state.clone());
    let submission = EvidenceSubmission::with_payload(
        ai::GateId::Invariant,
        ai::Evidence::InvariantProof,
        true,
        0xabc,
    );
    let body = command_body(71, submission);

    let first_response = first_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/command")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let first_response: ai::CommandResponseDto = ok_response_json(first_response).await;
    assert_eq!(first_response.disposition, "accepted");
    let persisted_snapshot = first_state.snapshot().unwrap();
    let persisted_disk_len = std::fs::metadata(&path).unwrap().len();

    let resumed = resume_durable_runtime(State::default(), &path).unwrap();
    assert_eq!(resumed.state.phase, ai::Phase::Analysis);
    assert_eq!(resumed.tlog.len(), persisted_snapshot.tlog_len);
    let resumed_session = ApiTransportSession::from_parts(
        resumed.state,
        resumed.tlog,
        cfg,
        resumed.command_ledger,
        ApiTransportLedger::default(),
    )
    .expect("resumed session should verify");
    let resumed_state = WorkerAppState::new(resumed_session, &path);
    let resumed_app = build_router(resumed_state.clone());

    let replay_response = resumed_app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/command")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let replay_response: ai::CommandResponseDto = ok_response_json(replay_response).await;

    assert_eq!(replay_response.disposition, "replayed");
    assert_eq!(replay_response.event_hash, first_response.event_hash);
    assert_eq!(replay_response.event_seq, first_response.event_seq);
    assert_eq!(
        resumed_state.snapshot().unwrap().tlog_len,
        persisted_snapshot.tlog_len
    );
    assert_eq!(std::fs::metadata(&path).unwrap().len(), persisted_disk_len);
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn command_route_accepts_mcp_receipt_submission_and_persists_tlog() {
    let path = tlog_path("mcp-receipt-submission");
    let _ = std::fs::remove_file(&path);
    let mut execute_state = State::ready();
    execute_state.phase = ai::Phase::Execute;
    let session = ApiTransportSession::from_parts(
        execute_state,
        TLog::default(),
        RuntimeConfig::default(),
        CommandLedger::default(),
        ApiTransportLedger::default(),
    )
    .expect("execute-phase session should verify");
    let state = WorkerAppState::new(session, &path);
    let app = build_router(state.clone());
    let request = McpCallRequest::new(
        ai::CapabilityRegistry::canonical(),
        "http://127.0.0.1:38469/mcp_worker",
        "shell",
        r#"{"cwd":".","command":"true"}"#,
        1000,
        4096,
    );
    let receipt = McpCallReceipt::from_response(
        &request,
        br#"{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"ok"}]}}"#,
        0,
        false,
    );
    assert!(receipt.is_success());
    assert!(receipt.is_valid_for(&request));

    let submission = receipt.submission();
    assert!(submission.is_contract_valid());
    let body = command_body_with_payload(
        41,
        submission,
        execution_receipt_payload(submission.payload_hash),
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/command")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let response: ai::CommandResponseDto = ok_response_json(response).await;
    assert!(response.ok);
    assert_eq!(response.request_id, 1);
    assert_eq!(response.disposition, "accepted");
    assert_eq!(state.snapshot().unwrap().tlog_len, 2);
    assert!(std::fs::metadata(&path).unwrap().len() > 0);
    let _ = std::fs::remove_file(path);
}

#[tokio::test]
async fn invalid_command_does_not_mutate_state() {
    let path = tlog_path("invalid");
    let _ = std::fs::remove_file(&path);
    let state = WorkerAppState::new_default(&path);
    let before = state.snapshot().unwrap();
    let app = build_router(state.clone());
    let body = serde_json::json!({
        "command_id": 22,
        "command_hash": 1,
        "payload_tag": "Unknown",
        "payload": {}
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/command")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(state.snapshot().unwrap(), before);
    assert!(!path.exists());
}

#[tokio::test]
async fn oversized_batch_command_does_not_mutate_state_or_disk() {
    let path = tlog_path("oversized-batch");
    let _ = std::fs::remove_file(&path);
    let state = WorkerAppState::new_default(&path);
    let before = state.snapshot().unwrap();
    let app = build_router(state.clone());
    let submissions: Vec<_> = (0..=ai::api::protocol::API_COMMAND_BATCH_LIMIT)
        .map(|idx| {
            EvidenceSubmission::with_payload(
                ai::GateId::Invariant,
                ai::Evidence::InvariantProof,
                true,
                0x9000 + idx as u64,
            )
        })
        .collect();
    let body = evidence_batch_body(61, &submissions);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/command")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(state.snapshot().unwrap(), before);
    assert!(!path.exists());
}

#[tokio::test]
async fn malformed_batch_payload_does_not_mutate_state_or_disk() {
    let path = tlog_path("malformed-batch");
    let _ = std::fs::remove_file(&path);
    let state = WorkerAppState::new_default(&path);
    let before = state.snapshot().unwrap();
    let app = build_router(state.clone());
    let valid = EvidenceSubmission::with_payload(
        ai::GateId::Invariant,
        ai::Evidence::InvariantProof,
        true,
        0xabc,
    );
    let mut body = evidence_batch_body(62, &[valid]);
    body["payload"] = serde_json::json!({ "not": "a batch array" });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/command")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(state.snapshot().unwrap(), before);
    assert!(!path.exists());
}

#[tokio::test]
async fn tampered_batch_envelope_does_not_mutate_state_or_disk() {
    let path = tlog_path("tampered-batch-envelope");
    let _ = std::fs::remove_file(&path);
    let state = WorkerAppState::new_default(&path);
    let before = state.snapshot().unwrap();
    let app = build_router(state.clone());
    let valid = EvidenceSubmission::with_payload(
        ai::GateId::Invariant,
        ai::Evidence::InvariantProof,
        true,
        0xabc,
    );
    let mut body = evidence_batch_body(63, &[valid]);
    body["command_hash"] =
        serde_json::json!(body["command_hash"].as_u64().unwrap().wrapping_add(1));

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/command")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(state.snapshot().unwrap(), before);
    assert!(!path.exists());
}
