#![forbid(unsafe_code)]

use std::path::Path;

use ai::{
    api::routes::{handle_command, handle_envelope},
    append_ollama_judgment_proof_event_ndjson, append_ollama_llm_effect_receipt_ndjson,
    load_ollama_judgment_proof_events_ndjson, load_ollama_llm_effect_receipts_ndjson,
    load_tlog_ndjson, tick, verify_ollama_judgment_proof_event_order_ndjson,
    verify_ollama_judgment_proof_events_ndjson,
    verify_ollama_judgment_tlog_ndjson, verify_ollama_llm_effect_receipts, verify_tlog,
    write_tlog_ndjson, Command, CommandEnvelope, ContextRecord, EventKind, Evidence,
    GateStatus, MemoryFact, MemoryIndex, ObservationRecord, OllamaClient, OllamaError,
    OllamaJudgmentProofEvent, Phase, PolicyStore, RuntimeConfig, State,
    OLLAMA_JUDGMENT_PROOF_LINE, OLLAMA_LLM_EFFECT_RECEIPT_RECORD,
    OLLAMA_LLM_EFFECT_RECEIPT_SCHEMA_VERSION,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = State::default();
    let cfg = RuntimeConfig::default();
    let mut tlog = Vec::new();

    tick(&mut state, &mut tlog, cfg)?;

    let observation = ObservationRecord::new(7, 1, 0xabc, 1);
    handle_command(
        &mut state,
        &mut tlog,
        cfg,
        Command::SubmitEvidence(observation.submission()),
    )?;

    let mut memory = MemoryIndex::default();
    memory.insert(MemoryFact::new(state.packet.objective_id, 0xfeed, 7, 1));
    let lookup = memory.lookup(state.packet.objective_id, 8);
    let context =
        ContextRecord::from_packet_memory(state.packet, observation.observed_hash, &lookup);
    handle_command(
        &mut state,
        &mut tlog,
        cfg,
        Command::SubmitEvidence(context.submission()),
    )?;

    let policy = PolicyStore::default();
    let client = OllamaClient::from_env()?;
    let call = client.call_from_context(&context, &policy)?;
    let llm = call.record.clone();
    let envelope = CommandEnvelope::new(call.request_hash, Command::SubmitEvidence(call.submission()));
    let response = handle_envelope(&mut state, &mut tlog, cfg, envelope.clone())?;

    let persisted_event = tlog
        .iter()
        .find(|event| {
            event.kind == EventKind::Persisted
                && event.evidence == Evidence::JudgmentRecord
                && event.api_command_hash == envelope.command_hash
        })
        .ok_or_else(|| std::io::Error::other("missing persisted llm judgment event"))?;
    let base_receipt = call
        .receipt_for_configured_event(client.config(), envelope.command_hash, persisted_event)
        .ok_or_else(|| std::io::Error::other("failed to prove local ollama provenance before receipt construction"))?;
    let base_url_provenance_verified = base_receipt.base_url_provenance_verified(client.config());
    if !base_url_provenance_verified {
        return Err(std::io::Error::other("ollama receipt does not prove configured base_url").into());
    }
    let judgment_passed = state.gates.judgment.status == GateStatus::Pass;
    let phase_is_plan = state.phase == Phase::Plan;
    let (receipt, proof_event) = OllamaJudgmentProofEvent::finalize_receipt_after_tlog(
        base_receipt,
        &tlog,
        true,
        true,
        base_url_provenance_verified,
        phase_is_plan,
    )
    .ok_or_else(|| std::io::Error::other("ollama judgment proof event did not close"))?;

    let tlog_path = std::env::var("CANON_OLLAMA_TLOG")
        .unwrap_or_else(|_| "ollama_judgment.tlog.ndjson".to_string());
    let tampered_tlog_path = std::env::var("CANON_OLLAMA_TAMPERED_TLOG")
        .unwrap_or_else(|_| "ollama_judgment.tampered.tlog.ndjson".to_string());
    let tlog_path_buf = std::path::PathBuf::from(&tlog_path);
    let tampered_tlog_path_buf = std::path::PathBuf::from(&tampered_tlog_path);
    std::fs::remove_file(&tampered_tlog_path_buf).ok();
    write_tlog_ndjson(&tlog_path_buf, &tlog)?;
    append_ollama_llm_effect_receipt_ndjson(&tlog_path_buf, &receipt)?;

    let disk_tlog = load_tlog_ndjson(&tlog_path_buf)?;
    let disk_receipts = load_ollama_llm_effect_receipts_ndjson(&tlog_path_buf)?;
    verify_tlog(&disk_tlog)?;
    verify_ollama_llm_effect_receipts(&disk_tlog, &disk_receipts)?;

    let receipt_verified = verify_ollama_judgment_tlog_ndjson(&tlog_path_buf)? == disk_receipts.len();
    let critical_fields = critical_ollama_receipt_tamper_fields();
    let mut tamper_rejections = 0usize;
    for (idx, (field_name, field_index)) in critical_fields.iter().enumerate() {
        let field_tampered_tlog_path = if idx == 0 {
            tampered_tlog_path_buf.clone()
        } else {
            let mut path = tampered_tlog_path_buf.clone();
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("ollama_judgment.tampered.tlog.ndjson");
            path.set_file_name(format!("{file_name}.{field_name}.ndjson"));
            path
        };
        std::fs::remove_file(&field_tampered_tlog_path).ok();
        tamper_first_ollama_receipt_field(&tlog_path_buf, &field_tampered_tlog_path, *field_index)?;
        if matches!(
            verify_ollama_judgment_tlog_ndjson(&field_tampered_tlog_path),
            Err(OllamaError::InvalidReplay)
        ) {
            tamper_rejections += 1;
        } else {
            return Err(std::io::Error::other(format!(
                "tampered ollama receipt field {field_name} unexpectedly passed replay",
            ))
            .into());
        }
    }
    let tamper_rejected = tamper_rejections == critical_fields.len();
    if !receipt_verified || !tamper_rejected || !proof_event.matches_receipt(receipt, &disk_tlog) {
        return Err(std::io::Error::other("ollama receipt/proof bidirectional binding did not close").into());
    }
    append_ollama_judgment_proof_event_ndjson(&tlog_path_buf, &proof_event)?;
    let disk_proof_events = load_ollama_judgment_proof_events_ndjson(&tlog_path_buf)?;
    let durable_proof_verified =
        verify_ollama_judgment_proof_events_ndjson(&tlog_path_buf)? == disk_proof_events.len();
    let proof_order_verified =
        verify_ollama_judgment_proof_event_order_ndjson(&tlog_path_buf)? == disk_proof_events.len();
    if !durable_proof_verified {
        return Err(std::io::Error::other("durable ollama judgment proof failed replay").into());
    }
    if !proof_order_verified {
        return Err(std::io::Error::other("ollama judgment proof ordering failed replay").into());
    }

    verify_tlog(&tlog)?;

    println!("provider=ollama");
    println!("base_url={}", client.config().base_url);
    println!("model={}", client.config().model);
    println!("prompt_hash={}", llm.prompt.prompt_hash);
    println!("request_hash={}", call.request_hash);
    println!("response_hash={}", llm.response.response_hash);
    println!("raw_response_hash={}", call.raw_response_hash);
    println!("base_url_hash={}", call.base_url_hash);
    println!("base_url_expected_hash={}", client.config().base_url_id());
    println!("base_url_provenance_verified={}", base_url_provenance_verified);
    println!("token_count={}", llm.response.token_count);
    println!("receipt_proof_event_seq={}", receipt.proof_event_seq);
    println!("receipt_proof_hash={}", receipt.proof_hash);
    println!("receipt_hash={}", receipt.receipt_hash);
    println!("receipt_event_seq={}", receipt.event_seq);
    println!("receipt_verified={}", receipt_verified);
    println!("tlog_path={}", tlog_path);
    println!("tampered_tlog_path={}", tampered_tlog_path);
    println!("tamper_rejected={}", tamper_rejected);
    println!(
        "tampered_fields_rejected={}/{}",
        tamper_rejections,
        critical_fields.len()
    );
    let ollama_judgment_proof = receipt_verified
        && tamper_rejected
        && base_url_provenance_verified
        && phase_is_plan
        && durable_proof_verified
        && proof_order_verified;
    if !ollama_judgment_proof {
        return Err(std::io::Error::other("ollama judgment proof did not close").into());
    }

    println!("judgment_passed={}", judgment_passed);
    println!("event_from={:?}", response.event.from);
    println!("event_to={:?}", response.event.to);
    println!("phase_is_plan={}", phase_is_plan);
    println!("proof_event_seq={}", proof_event.proof_event_seq);
    println!("proof_line_hash={}", proof_event.proof_line_hash);
    println!("proof_hash={}", proof_event.proof_hash);
    println!("proof_order_verified={}", proof_order_verified);
    println!("durable_proof_verified={}", durable_proof_verified);
    println!("ollama_judgment_proof={}", OLLAMA_JUDGMENT_PROOF_LINE);

    Ok(())
}

fn critical_ollama_receipt_tamper_fields() -> [(&'static str, usize); 9] {
    [
        ("provider", 2),
        ("base_url", 3),
        ("model", 4),
        ("request_hash", 5),
        ("response_hash", 6),
        ("raw_response_hash", 7),
        ("proof_event_seq", 14),
        ("proof_hash", 15),
        ("receipt_hash", 16),
    ]
}

fn tamper_first_ollama_receipt_field(
    source: &Path,
    target: &Path,
    field_index: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let input = std::fs::read_to_string(source)?;
    let mut output = String::new();
    let mut changed = false;

    for line in input.lines() {
        let trimmed = line.trim();
        let maybe_body = trimmed
            .strip_prefix('[')
            .and_then(|value| value.strip_suffix(']'));
        if !changed {
            if let Some(body) = maybe_body {
                let parsed = body
                    .split(',')
                    .map(|raw| raw.trim().parse::<u64>())
                    .collect::<Result<Vec<_>, _>>();
                if let Ok(mut fields) = parsed {
                    if fields.len() == 17
                        && fields[0] == OLLAMA_LLM_EFFECT_RECEIPT_SCHEMA_VERSION
                        && fields[1] == OLLAMA_LLM_EFFECT_RECEIPT_RECORD
                        && field_index < fields.len()
                    {
                        fields[field_index] ^= 1;
                        output.push('[');
                        output.push_str(
                            &fields
                                .iter()
                                .map(u64::to_string)
                                .collect::<Vec<_>>()
                                .join(","),
                        );
                        output.push_str("]\n");
                        changed = true;
                        continue;
                    }
                }
            }
        }
        output.push_str(line);
        output.push('\n');
    }

    if !changed {
        return Err(std::io::Error::other("missing ollama receipt line to tamper").into());
    }
    std::fs::write(target, output)?;
    Ok(())
}
