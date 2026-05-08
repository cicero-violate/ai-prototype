use ai::{
    append_mcp_call_receipt_ndjson, decode_mcp_call_receipt_ndjson, encode_mcp_call_receipt_ndjson,
    load_mcp_call_receipts_ndjson, verify_mcp_call_receipts, CapabilityRegistry, Effect,
    LiveMcpCallExecutor, McpCallReceipt, McpCallRequest, ToolSandboxError,
};

fn receipt_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "ai-mcp-receipt-{name}-{}.ndjson",
        std::process::id()
    ))
}

fn request() -> McpCallRequest {
    McpCallRequest::new(
        CapabilityRegistry::canonical(),
        "http://127.0.0.1:38469/mcp_worker",
        "shell",
        r#"{"cwd":".","command":"true"}"#,
        5000,
        65536,
    )
}

#[test]
fn mcp_request_hash_is_stable_and_admissible() {
    let first = request();
    let second = request();

    assert!(first.is_admissible());
    assert_ne!(first.contract_hash(), 0);
    assert_eq!(first.contract_hash(), second.contract_hash());
}

#[test]
fn mcp_receipt_normalizes_process_effect() {
    let request = request();
    let receipt = McpCallReceipt::from_response(&request, br#"{"ok":true}"#, 0, false);

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
fn mcp_receipt_roundtrips_ndjson_and_persists() {
    let request = request();
    let receipt = McpCallReceipt::from_response(&request, br#"{"ok":true}"#, 0, false);
    let encoded = encode_mcp_call_receipt_ndjson(&receipt);
    let decoded = match decode_mcp_call_receipt_ndjson(&encoded) {
        Ok(decoded) => decoded,
        Err(err) => panic!("receipt should decode: {err:?}"),
    };

    assert_eq!(decoded, receipt);
    assert_eq!(decoded.receipt_hash, receipt.receipt_hash);
    assert_eq!(verify_mcp_call_receipts(&[decoded.clone()]), Ok(1));

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
fn mcp_receipt_rejects_tampered_hash_and_shape() {
    let request = request();
    let mut receipt = McpCallReceipt::from_response(&request, br#"{"ok":true}"#, 0, false);
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
fn mcp_executor_enforces_allowlist_and_args_bound() {
    let executor = LiveMcpCallExecutor::new("http://127.0.0.1:38469/mcp_worker")
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
