use ai_delta_overlay::api::transport::*;

#[test]
fn placeholder_transport_contract() {
    assert!(verify_tlog(&vec![ControlEvent { state_after: State(1), valid: true }]).is_ok());
}

#[test]
fn transport_session_rejects_state_not_matching_tlog_tail_on_restart() {
    let tlog = vec![ControlEvent { state_after: State(7), valid: true }];
    assert_eq!(
        ApiTransportSession::from_parts(RuntimeConfig::default(), State(8), tlog),
        Err(CanonError::InvalidStateContinuity)
    );
}

#[test]
fn transport_session_rejects_invalid_runtime_config_on_restart() {
    let tlog = vec![ControlEvent { state_after: State(7), valid: true }];
    assert_eq!(
        ApiTransportSession::from_parts(RuntimeConfig::invalid(), State(7), tlog),
        Err(CanonError::InvalidReplay)
    );
}

#[test]
fn transport_receipt_verifier_rejects_invalid_tlog_even_when_receipt_event_matches() {
    let tlog = vec![ControlEvent { state_after: State(7), valid: false }];
    assert_eq!(verify_api_transport_receipts(&tlog), Err(CanonError::InvalidReplay));
}
