use ai::{
    build_router, tick, ApiTransportLedger, ApiTransportSession, Command, CommandEnvelope,
    CommandLedger, EvidenceSubmission, EvidenceSubmissionDto, RuntimeConfig, State, StateDto, TLog,
    WorkerAppState,
};
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

fn tlog_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "ai-api-server-{name}-{}.ndjson",
        std::process::id()
    ))
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

fn command_body(command_id: u64, submission: EvidenceSubmission) -> serde_json::Value {
    let envelope = CommandEnvelope::new(command_id, Command::SubmitEvidence(submission));
    serde_json::json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": "SubmitEvidence",
        "payload": invariant_payload(submission.payload_hash),
    })
}

async fn response_json<T: serde::de::DeserializeOwned>(response: axum::response::Response) -> T {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
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

    assert_eq!(response.status(), StatusCode::OK);
    let response: ai::CommandResponseDto = response_json(response).await;
    assert!(response.ok);
    assert_eq!(response.request_id, 1);
    assert_eq!(response.disposition, "accepted");
    assert_eq!(state.snapshot().unwrap().tlog_len, 3);
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
