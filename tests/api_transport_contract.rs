use ai::{
    append_api_transport_receipt_ndjson, decode_api_transport_receipt_ndjson,
    encode_api_transport_receipt_ndjson, handle_transport_frame_once,
    load_api_transport_receipts_ndjson, tick, verify_api_transport_receipts, verify_tlog,
    ApiTransportDisposition, ApiTransportFrame, ApiTransportLedger, ApiTransportReceipt, CanonError, Command,
    CommandEnvelope, CommandLedger, ObservationRecord, Phase, RuntimeConfig, State, TLog,
};

fn transport_receipt_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "ai-api-transport-{name}-{}.ndjson",
        std::process::id()
    ))
}

fn observation_frame() -> ApiTransportFrame {
    let record = ObservationRecord::new(1, 1, 0xabc, 1);
    let envelope = CommandEnvelope::new(11, Command::SubmitEvidence(record.submission()));
    ApiTransportFrame::new(101, envelope)
}

#[test]
fn transport_frame_replays_same_request_without_state_mutation() {
    let cfg = RuntimeConfig::default();
    let mut state = State::default();
    let mut tlog = TLog::default();
    let mut command_ledger = CommandLedger::default();
    let mut transport_ledger = ApiTransportLedger::default();
    let frame = observation_frame();
    assert!(tick(&mut state, &mut tlog, cfg).is_ok());
    assert_eq!(state.phase, Phase::Invariant);

    let first_result = handle_transport_frame_once(
        &mut state,
        &mut tlog,
        cfg,
        &mut command_ledger,
        &mut transport_ledger,
        frame.clone(),
    );
    assert!(first_result.is_ok());
    let first = match first_result {
        Ok(response) => response,
        Err(_) => return,
    };

    assert_eq!(first.disposition, ApiTransportDisposition::Accepted);
    assert_eq!(command_ledger.len(), 1);
    assert_eq!(transport_ledger.len(), 1);

    let state_after_first = state;
    let tlog_len_after_first = tlog.len();
    let second_result = handle_transport_frame_once(
        &mut state,
        &mut tlog,
        cfg,
        &mut command_ledger,
        &mut transport_ledger,
        frame,
    );
    assert!(second_result.is_ok());
    let second = match second_result {
        Ok(response) => response,
        Err(_) => return,
    };

    assert_eq!(second.disposition, ApiTransportDisposition::Replayed);
    assert_eq!(second.event_hash, first.event_hash);
    assert_eq!(state, state_after_first);
    assert_eq!(tlog.len(), tlog_len_after_first);
    assert_eq!(command_ledger.len(), 1);
    assert_eq!(transport_ledger.len(), 1);
    assert!(verify_tlog(&tlog).is_ok());
}

#[test]
fn transport_ledger_rejects_request_id_reuse_with_different_payload() {
    let cfg = RuntimeConfig::default();
    let mut state = State::default();
    let mut tlog = TLog::default();
    let mut command_ledger = CommandLedger::default();
    let mut transport_ledger = ApiTransportLedger::default();
    let first_frame = observation_frame();
    let conflicting_frame = ApiTransportFrame::new(
        first_frame.request_id,
        CommandEnvelope::new(
            12,
            Command::SubmitEvidence(ObservationRecord::new(2, 1, 0xdef, 1).submission()),
        ),
    );
    assert_ne!(first_frame.payload_hash, conflicting_frame.payload_hash);
    assert!(tick(&mut state, &mut tlog, cfg).is_ok());

    let first_result = handle_transport_frame_once(
        &mut state,
        &mut tlog,
        cfg,
        &mut command_ledger,
        &mut transport_ledger,
        first_frame,
    );
    assert!(first_result.is_ok());

    let state_after_first = state;
    let tlog_len_after_first = tlog.len();
    let conflict = handle_transport_frame_once(
        &mut state,
        &mut tlog,
        cfg,
        &mut command_ledger,
        &mut transport_ledger,
        conflicting_frame,
    );

    assert_eq!(conflict, Err(CanonError::InvalidApiCommand));
    assert_eq!(state, state_after_first);
    assert_eq!(tlog.len(), tlog_len_after_first);
    assert_eq!(command_ledger.len(), 1);
    assert_eq!(transport_ledger.len(), 1);
}

#[test]
fn transport_frame_rejects_tampered_payload_without_state_mutation() {
    let cfg = RuntimeConfig::default();
    let mut state = State::default();
    let before = state;
    let mut tlog = TLog::default();
    let mut command_ledger = CommandLedger::default();
    let mut transport_ledger = ApiTransportLedger::default();
    let mut frame = observation_frame();
    frame.payload_hash ^= 1;

    let result = handle_transport_frame_once(
        &mut state,
        &mut tlog,
        cfg,
        &mut command_ledger,
        &mut transport_ledger,
        frame,
    );

    assert_eq!(result, Err(CanonError::InvalidApiCommand));
    assert_eq!(state, before);
    assert!(tlog.is_empty());
    assert!(command_ledger.is_empty());
    assert!(transport_ledger.is_empty());
}

#[test]
fn transport_receipt_persists_and_verifies_against_tlog() {
    let cfg = RuntimeConfig::default();
    let mut state = State::default();
    let mut tlog = TLog::default();
    let mut command_ledger = CommandLedger::default();
    let mut transport_ledger = ApiTransportLedger::default();
    let frame = observation_frame();
    assert!(tick(&mut state, &mut tlog, cfg).is_ok());

    let response = match handle_transport_frame_once(
        &mut state,
        &mut tlog,
        cfg,
        &mut command_ledger,
        &mut transport_ledger,
        frame,
    ) {
        Ok(response) => response,
        Err(err) => panic!("transport frame should be accepted: {err:?}"),
    };

    assert_eq!(response.disposition, ApiTransportDisposition::Accepted);
    let receipt = transport_ledger.receipts()[0];
    let encoded = encode_api_transport_receipt_ndjson(receipt);
    let decoded = match decode_api_transport_receipt_ndjson(&encoded) {
        Ok(decoded) => decoded,
        Err(err) => panic!("transport receipt should decode: {err:?}"),
    };
    assert_eq!(decoded, receipt);
    assert!(verify_api_transport_receipts(&tlog, &[decoded]).is_ok());

    let path = transport_receipt_path("persist");
    let _ = std::fs::remove_file(&path);
    assert!(append_api_transport_receipt_ndjson(&path, decoded).is_ok());
    let loaded = match load_api_transport_receipts_ndjson(&path) {
        Ok(loaded) => loaded,
        Err(err) => panic!("transport receipt should load: {err:?}"),
    };
    let _ = std::fs::remove_file(&path);

    assert_eq!(loaded, vec![receipt]);
    assert!(verify_api_transport_receipts(&tlog, &loaded).is_ok());
}

#[test]
fn transport_receipt_verifier_rejects_tampered_command_hash() {
    let cfg = RuntimeConfig::default();
    let mut state = State::default();
    let mut tlog = TLog::default();
    let mut command_ledger = CommandLedger::default();
    let mut transport_ledger = ApiTransportLedger::default();
    let frame = observation_frame();
    assert!(tick(&mut state, &mut tlog, cfg).is_ok());
    assert!(handle_transport_frame_once(
        &mut state,
        &mut tlog,
        cfg,
        &mut command_ledger,
        &mut transport_ledger,
        frame,
    )
    .is_ok());

    let receipt = transport_ledger.receipts()[0];
    let tampered = ApiTransportReceipt::new(
        receipt.request_id,
        receipt.payload_hash,
        receipt.command_id,
        receipt.command_hash ^ 1,
        receipt.event_hash,
    );

    assert_eq!(
        verify_api_transport_receipts(&tlog, &[tampered]),
        Err(CanonError::InvalidReplay)
    );
}

#[test]
fn transport_receipt_decode_rejects_payload_hash_tamper() {
    let receipt = ApiTransportReceipt::new(101, 202, 303, 404, 505);
    let line = encode_api_transport_receipt_ndjson(receipt);
    let tampered = line.replacen(",202,", ",203,", 1);

    assert_eq!(
        decode_api_transport_receipt_ndjson(&tampered),
        Err(CanonError::InvalidTlogRecord)
    );
}