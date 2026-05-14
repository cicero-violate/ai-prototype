use ai::{
    api_transport_receipt_replay_classification,
    api_transport_receipt_replay_classification_with_expected_count,
    append_api_transport_receipt_ndjson, decode_api_transport_receipt_ndjson,
    encode_api_transport_receipt_ndjson, handle_transport_frame_once,
    load_api_transport_ledger_ndjson, load_api_transport_receipts_ndjson, tick,
    verify_api_transport_receipt_chain, verify_api_transport_receipts, verify_tlog,
    ApiTransportDisposition, ApiTransportFrame, ApiTransportLedger, ApiTransportReceipt,
    ApiTransportSession, CanonError, Command, CommandEnvelope, CommandLedger, ObservationRecord,
    Phase, RuntimeConfig, State, TLog,
};

fn transport_receipt_path(name: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let dir = std::path::PathBuf::from("target/test-tmp/api-transport-receipts");
    std::fs::create_dir_all(&dir).expect("api transport receipt fixture dir should exist");
    dir.join(format!(
        "ai-api-transport-{name}-{}-{nanos}.ndjson",
        std::process::id(),
    ))
}

fn observation_frame() -> ApiTransportFrame {
    let record = ObservationRecord::new(1, 1, 0xabc, 1);
    let envelope = CommandEnvelope::new(11, Command::SubmitEvidence(record.submission()));
    ApiTransportFrame::new(101, envelope)
}

fn accepted_transport_receipts(count: u64) -> (TLog, Vec<ApiTransportReceipt>) {
    let cfg = RuntimeConfig::default();
    let mut state = State::default();
    let mut tlog = TLog::default();
    let mut command_ledger = CommandLedger::default();
    let mut transport_ledger = ApiTransportLedger::default();
    assert!(tick(&mut state, &mut tlog, cfg).is_ok());

    for seq in 1..=count {
        let frame = ApiTransportFrame::new(
            100 + seq,
            CommandEnvelope::new(
                200 + seq,
                Command::SubmitEvidence(
                    ObservationRecord::new(seq, 1, 0xabc + seq, 1).submission(),
                ),
            ),
        );
        assert!(handle_transport_frame_once(
            &mut state,
            &mut tlog,
            cfg,
            &mut command_ledger,
            &mut transport_ledger,
            frame,
        )
        .is_ok());
    }

    (tlog, transport_ledger.receipts().to_vec())
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
fn transport_frame_receipt_match_is_payload_scoped() {
    let frame = observation_frame();
    let matching = ApiTransportReceipt::new(
        frame.request_id,
        frame.payload_hash,
        frame.envelope.command_id,
        frame.envelope.command_hash,
        55,
    );
    let different_payload = ApiTransportReceipt::new(
        frame.request_id,
        frame.payload_hash ^ 1,
        frame.envelope.command_id,
        frame.envelope.command_hash,
        55,
    );

    assert!(frame.matches_receipt(matching));
    assert!(!frame.matches_receipt(different_payload));
}

#[test]
fn api_transport_receipt_contract_rejects_zero_fields_and_tampered_hash() {
    let valid = ApiTransportReceipt::new(101, 202, 303, 404, 505);
    assert!(valid.is_contract_valid());

    let mut zero_request_id = valid;
    zero_request_id.request_id = 0;
    assert!(!zero_request_id.is_contract_valid());

    let mut zero_payload_hash = valid;
    zero_payload_hash.payload_hash = 0;
    assert!(!zero_payload_hash.is_contract_valid());

    let mut zero_command_id = valid;
    zero_command_id.command_id = 0;
    assert!(!zero_command_id.is_contract_valid());

    let mut zero_command_hash = valid;
    zero_command_hash.command_hash = 0;
    assert!(!zero_command_hash.is_contract_valid());

    let mut zero_event_hash = valid;
    zero_event_hash.event_hash = 0;
    assert!(!zero_event_hash.is_contract_valid());

    let mut zero_receipt_hash = valid;
    zero_receipt_hash.receipt_hash = 0;
    assert!(!zero_receipt_hash.is_contract_valid());

    let mut tampered_receipt_hash = valid;
    tampered_receipt_hash.receipt_hash = tampered_receipt_hash.receipt_hash.wrapping_add(1);
    assert!(!tampered_receipt_hash.is_contract_valid());
}

#[test]
fn transport_ledger_exposes_request_id_membership() {
    let receipt = ApiTransportReceipt::new(101, 202, 303, 404, 505);
    let ledger = match ApiTransportLedger::from_receipts(vec![receipt]) {
        Ok(ledger) => ledger,
        Err(err) => panic!("transport ledger should accept unique receipt: {err:?}"),
    };

    assert!(ledger.contains_request_id(101));
    assert!(!ledger.contains_request_id(102));
}

#[test]
fn transport_ledger_request_id_lookup_distinguishes_membership_from_payload_conflict() {
    let frame = observation_frame();
    let receipt = ApiTransportReceipt::new(
        frame.request_id,
        frame.payload_hash,
        frame.envelope.command_id,
        frame.envelope.command_hash,
        55,
    );
    let ledger = match ApiTransportLedger::from_receipts(vec![receipt]) {
        Ok(ledger) => ledger,
        Err(err) => panic!("transport ledger should accept unique receipt: {err:?}"),
    };

    assert!(ledger.contains_request_id(frame.request_id));
    assert!(!ledger.contains_request_id(frame.request_id + 1));
    assert!(!ledger.has_conflicting_request(&frame));

    let mut conflicting_frame = frame.clone();
    conflicting_frame.payload_hash ^= 1;

    assert!(ledger.has_conflicting_request(&conflicting_frame));
}

#[test]
fn transport_frame_classifies_same_payload_request_id_collision_as_invalid_replay() {
    let cfg = RuntimeConfig::default();
    let mut state = State::default();
    let before = state;
    let mut tlog = TLog::default();
    let mut command_ledger = CommandLedger::default();
    let frame = observation_frame();
    let receipt_with_same_request_and_payload_but_different_command = ApiTransportReceipt::new(
        frame.request_id,
        frame.payload_hash,
        frame.envelope.command_id.wrapping_add(1),
        frame.envelope.command_hash,
        55,
    );
    let mut transport_ledger = match ApiTransportLedger::from_receipts(vec![
        receipt_with_same_request_and_payload_but_different_command,
    ]) {
        Ok(ledger) => ledger,
        Err(err) => panic!("syntactically valid receipt should build a ledger: {err:?}"),
    };

    assert!(!frame.matches_receipt(receipt_with_same_request_and_payload_but_different_command));
    assert!(!transport_ledger.has_conflicting_request(&frame));
    assert!(transport_ledger.contains_request_id(frame.request_id));

    assert_eq!(
        handle_transport_frame_once(
            &mut state,
            &mut tlog,
            cfg,
            &mut command_ledger,
            &mut transport_ledger,
            frame,
        ),
        Err(CanonError::InvalidReplay)
    );
    assert_eq!(state, before);
    assert!(tlog.is_empty());
    assert!(command_ledger.is_empty());
    assert_eq!(transport_ledger.len(), 1);
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
fn transport_ledger_loads_persisted_receipts_for_process_restart_replay() {
    let cfg = RuntimeConfig::default();
    let mut state = State::default();
    let mut tlog = TLog::default();
    let mut command_ledger = CommandLedger::default();
    let mut transport_ledger = ApiTransportLedger::default();
    let frame = observation_frame();
    assert!(tick(&mut state, &mut tlog, cfg).is_ok());

    let first = match handle_transport_frame_once(
        &mut state,
        &mut tlog,
        cfg,
        &mut command_ledger,
        &mut transport_ledger,
        frame.clone(),
    ) {
        Ok(response) => response,
        Err(err) => panic!("transport frame should be accepted before restart: {err:?}"),
    };
    let receipt = transport_ledger.receipts()[0];
    let state_after_first = state;
    let tlog_len_after_first = tlog.len();

    let path = transport_receipt_path("restart-replay");
    let _ = std::fs::remove_file(&path);
    assert!(append_api_transport_receipt_ndjson(&path, receipt).is_ok());
    let mut restored_transport_ledger = match load_api_transport_ledger_ndjson(&path) {
        Ok(ledger) => ledger,
        Err(err) => panic!("transport ledger should load from receipts: {err:?}"),
    };
    let _ = std::fs::remove_file(&path);
    let mut restored_command_ledger = CommandLedger::default();

    let replay = match handle_transport_frame_once(
        &mut state,
        &mut tlog,
        cfg,
        &mut restored_command_ledger,
        &mut restored_transport_ledger,
        frame,
    ) {
        Ok(response) => response,
        Err(err) => panic!("loaded transport receipt should replay: {err:?}"),
    };

    assert_eq!(replay.disposition, ApiTransportDisposition::Replayed);
    assert_eq!(replay.event_hash, first.event_hash);
    assert_eq!(state, state_after_first);
    assert_eq!(tlog.len(), tlog_len_after_first);
    assert!(restored_command_ledger.is_empty());
    assert_eq!(restored_transport_ledger.len(), 1);
    assert!(verify_api_transport_receipts(&tlog, restored_transport_ledger.receipts()).is_ok());
}

#[test]
fn transport_ledger_load_rejects_duplicate_request_ids() {
    let receipt = ApiTransportReceipt::new(101, 202, 303, 404, 505);
    let path = transport_receipt_path("duplicate-request-id");
    let _ = std::fs::remove_file(&path);
    assert!(append_api_transport_receipt_ndjson(&path, receipt).is_ok());
    assert!(append_api_transport_receipt_ndjson(&path, receipt).is_ok());

    assert_eq!(
        load_api_transport_ledger_ndjson(&path),
        Err(CanonError::InvalidApiCommand)
    );
    let _ = std::fs::remove_file(&path);
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

#[test]
fn transport_receipt_chain_reports_valid_replay() {
    let (tlog, receipts) = accepted_transport_receipts(2);
    let report = match verify_api_transport_receipt_chain(&tlog, &receipts) {
        Ok(report) => report,
        Err(err) => panic!("transport receipts should verify: {err:?}"),
    };

    assert_eq!(report.receipt_count, 2);
    assert_eq!(report.first_seq, 1);
    assert_eq!(report.last_seq, 2);
    assert_ne!(report.final_hash, 0);
    assert_eq!(
        api_transport_receipt_replay_classification(&tlog, &receipts),
        None
    );
}

#[test]
fn transport_receipt_chain_classifies_missing_receipt() {
    let (tlog, mut receipts) = accepted_transport_receipts(2);
    receipts.pop();

    assert_eq!(
        api_transport_receipt_replay_classification_with_expected_count(&tlog, &receipts, 2),
        Some("missing_receipt")
    );
    assert!(verify_api_transport_receipts(&tlog, &receipts).is_ok());
}

#[test]
fn transport_receipt_chain_classifies_stale_receipt() {
    let (mut tlog, receipts) = accepted_transport_receipts(2);
    tlog.pop();

    assert_eq!(
        api_transport_receipt_replay_classification(&tlog, &receipts),
        Some("stale_receipt")
    );
    assert_eq!(
        verify_api_transport_receipts(&tlog, &receipts),
        Err(CanonError::InvalidReplay)
    );
}

#[test]
fn transport_receipt_chain_classifies_duplicated_receipt() {
    let (tlog, mut receipts) = accepted_transport_receipts(2);
    receipts[1] = receipts[0];

    assert_eq!(
        api_transport_receipt_replay_classification(&tlog, &receipts),
        Some("duplicated_receipt")
    );
    assert_eq!(
        verify_api_transport_receipts(&tlog, &receipts),
        Err(CanonError::InvalidApiCommand)
    );
}

#[test]
fn transport_receipt_chain_classifies_reordered_receipts() {
    let (tlog, mut receipts) = accepted_transport_receipts(2);
    receipts.swap(0, 1);

    assert_eq!(
        api_transport_receipt_replay_classification(&tlog, &receipts),
        Some("reordered_receipt")
    );
    assert_eq!(
        verify_api_transport_receipts(&tlog, &receipts),
        Err(CanonError::InvalidReplay)
    );
}

#[test]
fn transport_receipt_chain_classifies_forged_receipt() {
    let (tlog, mut receipts) = accepted_transport_receipts(2);
    receipts[0].receipt_hash = receipts[0].receipt_hash.wrapping_add(1);

    assert_eq!(
        api_transport_receipt_replay_classification(&tlog, &receipts),
        Some("forged_receipt")
    );
    assert_eq!(
        verify_api_transport_receipts(&tlog, &receipts),
        Err(CanonError::InvalidApiCommand)
    );
}

#[test]
fn transport_session_handles_replay_and_verifies_receipts() {
    let cfg = RuntimeConfig::default();
    let mut state = State::default();
    let mut tlog = TLog::default();
    assert!(tick(&mut state, &mut tlog, cfg).is_ok());
    let mut session = match ApiTransportSession::from_parts(
        state,
        tlog,
        cfg,
        CommandLedger::default(),
        ApiTransportLedger::default(),
    ) {
        Ok(session) => session,
        Err(err) => panic!("transport session should initialize from valid parts: {err:?}"),
    };
    let frame = observation_frame();

    let first = match session.handle_frame(frame.clone()) {
        Ok(response) => response,
        Err(err) => panic!("session should accept first transport frame: {err:?}"),
    };
    let state_after_first = *session.state();
    let tlog_len_after_first = session.tlog().len();
    assert_eq!(first.disposition, ApiTransportDisposition::Accepted);
    assert_eq!(session.transport_ledger().len(), 1);
    assert_eq!(session.command_ledger().len(), 1);
    assert!(session.verify().is_ok());

    let replay = match session.handle_frame(frame) {
        Ok(response) => response,
        Err(err) => panic!("session should replay persisted transport frame: {err:?}"),
    };
    assert_eq!(replay.disposition, ApiTransportDisposition::Replayed);
    assert_eq!(replay.event_hash, first.event_hash);
    assert_eq!(*session.state(), state_after_first);
    assert_eq!(session.tlog().len(), tlog_len_after_first);
    assert_eq!(session.transport_ledger().len(), 1);
    assert_eq!(session.command_ledger().len(), 1);
    assert!(session.verify().is_ok());
}

#[test]
fn transport_session_rejects_stale_command_ledger_on_restart() {
    let cfg = RuntimeConfig::default();
    let mut state = State::default();
    let mut tlog = TLog::default();
    assert!(tick(&mut state, &mut tlog, cfg).is_ok());
    let mut session = match ApiTransportSession::from_parts(
        state,
        tlog,
        cfg,
        CommandLedger::default(),
        ApiTransportLedger::default(),
    ) {
        Ok(session) => session,
        Err(err) => panic!("transport session should initialize from valid empty parts: {err:?}"),
    };
    assert!(session.handle_frame(observation_frame()).is_ok());

    let (state, tlog, cfg, command_ledger, transport_ledger) = session.clone().into_parts();
    assert!(ApiTransportSession::from_parts(
        state,
        tlog.clone(),
        cfg,
        command_ledger,
        transport_ledger.clone(),
    )
    .is_ok());

    assert_eq!(
        ApiTransportSession::from_parts(
            state,
            tlog,
            cfg,
            CommandLedger::default(),
            transport_ledger,
        ),
        Err(CanonError::InvalidReplay)
    );
}

#[test]
fn transport_session_rejects_receipts_not_backed_by_tlog_events() {
    let receipt = ApiTransportReceipt::new(101, 202, 303, 404, 505);
    let transport_ledger = match ApiTransportLedger::from_receipts(vec![receipt]) {
        Ok(ledger) => ledger,
        Err(err) => panic!("syntactically valid receipt should build a ledger: {err:?}"),
    };

    assert_eq!(
        ApiTransportSession::from_parts(
            State::default(),
            TLog::default(),
            RuntimeConfig::default(),
            CommandLedger::default(),
            transport_ledger,
        ),
        Err(CanonError::InvalidReplay)
    );
}
