use ai::{
    append_mcp_call_receipt_ndjson, decode_mcp_call_receipt_ndjson, encode_mcp_call_receipt_ndjson,
    load_mcp_call_receipts_ndjson, verify_mcp_call_receipts, CapabilityRegistry, Effect,
    LiveActionExecutor, ActionReceipt, ActionCallRequest, ToolSandboxError,
};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn receipt_path(name: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::path::PathBuf::from("target/test-tmp/action-receipts").join(format!(
        "ai-action-receipt-{name}-{}-{nonce}.ndjson",
        std::process::id(),
    ))
}

fn request() -> ActionCallRequest {
    ActionCallRequest::new(
        CapabilityRegistry::canonical(),
        "http://127.0.0.1:38469/mcp_worker",
        "shell",
        r#"{"cwd":".","command":"true"}"#,
        5000,
        65536,
    )
}

#[test]
fn action_request_hash_is_stable_and_admissible() {
    let first = request();
    let second = request();

    assert!(first.is_admissible());
    assert_ne!(first.contract_hash(), 0);
    assert_eq!(first.contract_hash(), second.contract_hash());
}

#[test]
fn action_receipt_normalizes_process_effect() {
    let request = request();
    let receipt = ActionReceipt::from_response(&request, br#"{"ok":true}"#, 0, false);

    assert!(receipt.is_contract_valid());
    assert!(receipt.is_success());
    assert!(receipt.effect_is_normalized());
    assert_eq!(
        receipt.effect,
        Effect::process(
            receipt.response_hash,
            0,
            receipt.response_bytes,
            0,
            receipt.exit_status,
            receipt.timed_out,
        )
    );
    assert!(receipt.is_valid_for(&request));
}

#[test]
fn action_receipt_roundtrips_ndjson_and_persists() {
    let request = request();
    let receipt = ActionReceipt::from_response(&request, br#"{"ok":true}"#, 0, false);
    let encoded = encode_mcp_call_receipt_ndjson(&receipt);
    let decoded = match decode_mcp_call_receipt_ndjson(&encoded) {
        Ok(decoded) => decoded,
        Err(err) => panic!("receipt should decode: {err:?}"),
    };

    assert_eq!(decoded, receipt);
    assert_eq!(decoded.receipt_hash, receipt.receipt_hash);
    assert_eq!(
        verify_mcp_call_receipts(std::slice::from_ref(&decoded)),
        Ok(1)
    );

    let path = receipt_path("persist");
    let _ = std::fs::remove_file(&path);
    assert!(append_mcp_call_receipt_ndjson(&path, &decoded).is_ok());
    let loaded = match load_mcp_call_receipts_ndjson(&path) {
        Ok(loaded) => loaded,
        Err(err) => panic!("receipt should load: {err:?}"),
    };
    assert_eq!(loaded, vec![receipt]);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn action_receipt_rejects_tampered_hash_and_shape() {
    let request = request();
    let mut receipt = ActionReceipt::from_response(&request, br#"{"ok":true}"#, 0, false);
    receipt.receipt_hash ^= 1;

    assert_eq!(
        decode_mcp_call_receipt_ndjson(&encode_mcp_call_receipt_ndjson(&receipt)),
        Err(ToolSandboxError::InvalidToolReceipt)
    );
    assert_eq!(
        decode_mcp_call_receipt_ndjson("[1,4,5]"),
        Err(ToolSandboxError::InvalidToolReceiptRecord)
    );
}

#[test]
fn action_executor_enforces_allowlist_and_args_bound() {
    let executor = LiveActionExecutor::new("http://127.0.0.1:38469/mcp_worker")
        .with_allowed_tool("shell")
        .with_timeout_ms(100)
        .with_max_output_bytes(8);

    assert_eq!(
        executor.request_for("apply_patch", "{}"),
        Err(ToolSandboxError::CommandDenied)
    );
    assert_eq!(
        executor.request_for("shell", r#"{"long":true}"#),
        Err(ToolSandboxError::ArtifactTooLarge)
    );
    assert!(executor.request_for("shell", "{}").is_ok());
}

#[test]
fn action_executor_calls_local_worker_and_records_receipt() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local mcp worker");
    let port = listener.local_addr().expect("local addr").port();
    let worker_url = format!("http://127.0.0.1:{port}/mcp_worker");
    let response_body =
        br#"{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"ok"}]}}"#;
    let expected_response_bytes = response_body.len() as u64;

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept mcp request");
        let mut request_bytes = [0u8; 4096];
        let read = stream.read(&mut request_bytes).expect("read mcp request");
        let request = String::from_utf8_lossy(&request_bytes[..read]);

        assert!(request.contains("POST /mcp_worker HTTP/1.1"));
        assert!(request.contains("\"method\":\"tools/call\""));
        assert!(request.contains("\"name\":\"shell\""));
        assert!(request.contains("\"command\":\"true\""));

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            response_body.len(),
            String::from_utf8_lossy(response_body)
        );
        stream
            .write_all(response.as_bytes())
            .expect("write mcp response");
    });

    let args = r#"{"cwd":".","command":"true"}"#;
    let executor = LiveActionExecutor::new(worker_url)
        .with_allowed_tool("shell")
        .with_timeout_ms(1000)
        .with_max_output_bytes(4096);
    let request = executor
        .request_for("shell", args)
        .expect("request should be admissible");
    let receipt = executor
        .execute_call("shell", args)
        .expect("local mcp call should produce receipt");

    server.join().expect("server thread should finish");

    assert!(receipt.is_success());
    assert!(receipt.is_contract_valid());
    assert!(receipt.effect_is_normalized());
    assert!(receipt.is_valid_for(&request));
    assert_eq!(receipt.exit_status, 0);
    assert!(!receipt.timed_out);
    assert_eq!(receipt.response_bytes, expected_response_bytes);
    assert_ne!(receipt.response_hash, 0);
    assert_eq!(executor.replay_receipt(&receipt, "shell", args), Ok(true));

    let encoded = encode_mcp_call_receipt_ndjson(&receipt);
    let decoded = decode_mcp_call_receipt_ndjson(&encoded).expect("receipt should decode");
    assert_eq!(decoded, receipt);
    assert_eq!(verify_mcp_call_receipts(&[decoded]), Ok(1));
    assert_eq!(
        executor.execute_call("apply_patch", "{}"),
        Err(ToolSandboxError::CommandDenied)
    );
}

#[test]
fn action_executor_records_connection_failure_as_receipt() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("reserve local unused mcp port");
    let port = listener.local_addr().expect("local addr").port();
    drop(listener);

    let worker_url = format!("http://127.0.0.1:{port}/mcp_worker");
    let args = r#"{"cwd":".","command":"true"}"#;
    let executor = LiveActionExecutor::new(worker_url)
        .with_allowed_tool("shell")
        .with_timeout_ms(250)
        .with_max_output_bytes(4096);
    let request = executor
        .request_for("shell", args)
        .expect("request should be admissible");
    let receipt = executor
        .execute_call("shell", args)
        .expect("connection failure should still produce a typed receipt");

    assert!(!receipt.is_success());
    assert!(receipt.is_contract_valid());
    assert!(receipt.effect_is_normalized());
    assert!(receipt.is_valid_for(&request));
    assert_eq!(receipt.exit_status, 1);
    assert!(!receipt.timed_out);
    assert_eq!(receipt.response_bytes, 0);
    assert_ne!(receipt.response_hash, 0);
    assert_eq!(executor.replay_receipt(&receipt, "shell", args), Ok(true));
    assert_eq!(verify_mcp_call_receipts(&[receipt]), Ok(1));
}

#[test]
fn action_executor_records_worker_timeout_as_receipt() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local mcp worker");
    let port = listener.local_addr().expect("local addr").port();
    let worker_url = format!("http://127.0.0.1:{port}/mcp_worker");
    let (release_tx, release_rx) = mpsc::channel::<()>();

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept mcp request");
        let mut request_bytes = [0u8; 4096];
        let read = stream.read(&mut request_bytes).expect("read mcp request");
        let request = String::from_utf8_lossy(&request_bytes[..read]);

        assert!(request.contains("POST /mcp_worker HTTP/1.1"));
        assert!(request.contains("\"method\":\"tools/call\""));
        assert!(request.contains("\"name\":\"shell\""));

        let _ = release_rx.recv_timeout(Duration::from_secs(2));
    });

    let args = r#"{"cwd":".","command":"sleep"}"#;
    let executor = LiveActionExecutor::new(worker_url)
        .with_allowed_tool("shell")
        .with_timeout_ms(100)
        .with_max_output_bytes(4096);
    let request = executor
        .request_for("shell", args)
        .expect("request should be admissible");
    let receipt = executor
        .execute_call("shell", args)
        .expect("worker timeout should still produce a typed receipt");

    let _ = release_tx.send(());
    server.join().expect("server thread should finish");

    assert!(!receipt.is_success());
    assert!(receipt.is_contract_valid());
    assert!(receipt.effect_is_normalized());
    assert!(receipt.is_valid_for(&request));
    assert_eq!(receipt.exit_status, 1);
    assert!(receipt.timed_out);
    assert_eq!(receipt.response_bytes, 0);
    assert_ne!(receipt.response_hash, 0);
    assert_eq!(executor.replay_receipt(&receipt, "shell", args), Ok(true));
    assert_eq!(verify_mcp_call_receipts(&[receipt]), Ok(1));
}

#[test]
fn action_executor_records_worker_http_failure_as_receipt() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind local mcp worker");
    let port = listener.local_addr().expect("local addr").port();
    let worker_url = format!("http://127.0.0.1:{port}/mcp_worker");
    let response_body =
        br#"{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"worker failed"}}"#;
    let expected_response_bytes = response_body.len() as u64;

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept mcp request");
        let mut request_bytes = [0u8; 4096];
        let read = stream.read(&mut request_bytes).expect("read mcp request");
        let request = String::from_utf8_lossy(&request_bytes[..read]);

        assert!(request.contains("POST /mcp_worker HTTP/1.1"));
        assert!(request.contains("\"method\":\"tools/call\""));
        assert!(request.contains("\"name\":\"shell\""));

        let response = format!(
            "HTTP/1.1 500 Internal Server Error\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            response_body.len(),
            String::from_utf8_lossy(response_body)
        );
        stream
            .write_all(response.as_bytes())
            .expect("write mcp error response");
    });

    let args = r#"{"cwd":".","command":"false"}"#;
    let executor = LiveActionExecutor::new(worker_url)
        .with_allowed_tool("shell")
        .with_timeout_ms(1000)
        .with_max_output_bytes(4096);
    let request = executor
        .request_for("shell", args)
        .expect("request should be admissible");
    let receipt = executor
        .execute_call("shell", args)
        .expect("worker HTTP failure should still produce a typed receipt");

    server.join().expect("server thread should finish");

    assert!(!receipt.is_success());
    assert!(receipt.is_contract_valid());
    assert!(receipt.effect_is_normalized());
    assert!(receipt.is_valid_for(&request));
    assert_eq!(receipt.exit_status, 1);
    assert!(!receipt.timed_out);
    assert_eq!(receipt.response_bytes, expected_response_bytes);
    assert_ne!(receipt.response_hash, 0);
    assert_eq!(executor.replay_receipt(&receipt, "shell", args), Ok(true));
    assert_eq!(verify_mcp_call_receipts(&[receipt]), Ok(1));
}
