use std::error::Error;
use std::io;
use std::path::Path;

use ai::api::routes::handle_envelope;
use ai::{
    append_ollama_judgment_proof_event_ndjson, append_ollama_llm_effect_receipt_ndjson,
    load_ollama_judgment_proof_events_ndjson, load_ollama_llm_effect_receipts_ndjson,
    load_tlog_ndjson, verify_ollama_judgment_proof_event_order_ndjson,
    verify_ollama_judgment_proof_events, verify_ollama_judgment_proof_events_ndjson,
    verify_ollama_judgment_tlog_ndjson, verify_ollama_llm_effect_receipts, verify_tlog,
    write_tlog_ndjson, Command, CommandEnvelope, ContextRecord, EventKind, Evidence, Gate,
    GateId, GateStatus, MemoryFact, MemoryIndex, OllamaClient, OllamaError, OllamaJudgmentProofEvent,
    OllamaLlmEffectReceipt, OLLAMA_JUDGMENT_PROOF_LINE, OLLAMA_LLM_EFFECT_RECEIPT_RECORD,
    OLLAMA_LLM_EFFECT_RECEIPT_SCHEMA_VERSION, Phase, PolicyStore, RuntimeConfig, State,
};

fn critical_ollama_receipt_tamper_fields() -> [(&'static str, usize); 17] {
    [
        ("provider", 2),
        ("base_url", 3),
        ("model", 4),
        ("request_hash", 5),
        ("timeout_ms", 6),
        ("retry_count", 7),
        ("max_retries", 8),
        ("attempt_budget", 9),
        ("request_identity_hash", 10),
        ("retry_budget_hash", 11),
        ("budget_exhausted", 12),
        ("duplicate_request", 13),
        ("response_hash", 14),
        ("raw_response_hash", 15),
        ("proof_event_seq", 22),
        ("proof_hash", 23),
        ("receipt_hash", 24),
    ]
}

fn tamper_first_ollama_receipt_field(
    source: &Path,
    target: &Path,
    field_index: usize,
) -> Result<(), Box<dyn Error>> {
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
                    if fields.len() >= 2
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
        return Err(io::Error::other("missing ollama receipt line to tamper").into());
    }

    std::fs::write(target, output)?;
    Ok(())
}

fn tamper_rejected_count(path: &Path, receipt: OllamaLlmEffectReceipt) -> Result<usize, Box<dyn Error>> {
    let mut rejected = 0usize;
    for (field_name, field_index) in critical_ollama_receipt_tamper_fields() {
        let tampered_path = path.with_file_name(format!(
            "ollama_judgment-{}-{}-{field_name}.tampered.tlog.ndjson",
            std::process::id(),
            receipt.receipt_hash
        ));
        std::fs::remove_file(&tampered_path).ok();
        tamper_first_ollama_receipt_field(path, &tampered_path, field_index)?;
        if matches!(
            verify_ollama_judgment_tlog_ndjson(&tampered_path),
            Err(OllamaError::InvalidReplay)
        ) {
            rejected += 1;
        }
        std::fs::remove_file(&tampered_path).ok();
    }
    Ok(rejected)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut state = State::default();
    state.phase = Phase::Judgment;
    state.gates.invariant = Gate::pass(Evidence::InvariantProof);
    state.gates.analysis = Gate::pass(Evidence::AnalysisReport);

    let mut memory = MemoryIndex::default();
    let _inserted = memory.insert(MemoryFact::new(state.packet.objective_id, 0xfeed, 7, 1));
    let lookup = memory.lookup(state.packet.objective_id, 8);
    let context = ContextRecord::from_packet_memory(state.packet, 0xabc, &lookup);
    let policy = PolicyStore::default();
    let client = OllamaClient::from_env()?;
    let call = client.call_from_context(&context, &policy)?;
    let envelope = CommandEnvelope::new(
        call.request_hash,
        Command::SubmitEvidence(call.submission()),
    );

    let mut tlog = Vec::new();
    handle_envelope(&mut state, &mut tlog, RuntimeConfig::default(), envelope.clone())?;
    verify_tlog(&tlog)?;

    let persisted_event = tlog
        .iter()
        .find(|event| {
            event.kind == EventKind::Persisted
                && event.evidence == Evidence::JudgmentRecord
                && event.affected_gate == Some(GateId::Judgment)
                && event.api_command_hash == envelope.command_hash
        })
        .copied()
        .ok_or_else(|| io::Error::other("missing persisted judgment event"))?;

    let base_receipt = call
        .receipt_for_configured_event(client.config(), envelope.command_hash, &persisted_event)
        .ok_or_else(|| io::Error::other("missing ollama receipt"))?;
    let receipt_verified = base_receipt.replay_verified(&tlog);
    let endpoint_verified = base_receipt.base_url_provenance_verified(client.config());
    let (receipt, proof_event) = OllamaJudgmentProofEvent::finalize_receipt_after_tlog(
        base_receipt,
        &tlog,
        receipt_verified,
        true,
        endpoint_verified,
        state.phase == Phase::Plan,
    )
    .ok_or_else(|| io::Error::other("failed to finalize ollama proof"))?;

    let tlog_path = Path::new("ollama_judgment.tlog.ndjson");
    let tampered_tlog_path = Path::new("ollama_judgment.tampered.tlog.ndjson");
    std::fs::remove_file(tlog_path).ok();
    std::fs::remove_file(tampered_tlog_path).ok();

    write_tlog_ndjson(tlog_path, &tlog)?;
    append_ollama_llm_effect_receipt_ndjson(tlog_path, &receipt)?;
    append_ollama_judgment_proof_event_ndjson(tlog_path, &proof_event)?;

    let loaded_tlog = load_tlog_ndjson(tlog_path)?;
    let loaded_receipts = load_ollama_llm_effect_receipts_ndjson(tlog_path)?;
    let loaded_proofs = load_ollama_judgment_proof_events_ndjson(tlog_path)?;
    let receipt_replay_count = verify_ollama_llm_effect_receipts(&loaded_tlog, &loaded_receipts)?;
    let proof_replay_count =
        verify_ollama_judgment_proof_events(&loaded_tlog, &loaded_receipts, &loaded_proofs)?;
    let proof_order_count = verify_ollama_judgment_proof_event_order_ndjson(tlog_path)?;
    let durable_proof_count = verify_ollama_judgment_proof_events_ndjson(tlog_path)?;
    let durable_receipt_count = verify_ollama_judgment_tlog_ndjson(tlog_path)?;

    tamper_first_ollama_receipt_field(tlog_path, tampered_tlog_path, 24)?;
    let tamper_rejected = matches!(
        verify_ollama_judgment_tlog_ndjson(tampered_tlog_path),
        Err(OllamaError::InvalidReplay)
    );
    let tampered_total = critical_ollama_receipt_tamper_fields().len();
    let tampered_rejected = tamper_rejected_count(tlog_path, receipt)?;

    println!("provider=ollama");
    println!("base_url={}", client.config().base_url);
    println!("model={}", client.config().model);
    println!("prompt_hash={}", call.prompt_hash);
    println!("request_hash={}", call.request_hash);
    println!("response_hash={}", call.response_hash);
    println!("raw_response_hash={}", call.raw_response_hash);
    println!("base_url_hash={}", receipt.base_url_hash);
    println!("base_url_expected_hash={}", client.config().base_url_id());
    println!(
        "base_url_provenance_verified={}",
        receipt.base_url_provenance_verified(client.config())
    );
    println!("timeout_ms={}", receipt.timeout_ms);
    println!("retry_count={}", receipt.retry_count);
    println!("max_retries={}", receipt.max_retries);
    println!("attempt_budget={}", receipt.attempt_budget);
    println!("request_identity_hash={}", receipt.request_identity_hash);
    println!("retry_budget_hash={}", receipt.retry_budget_hash);
    println!("budget_exhausted={}", receipt.budget_exhausted);
    println!("duplicate_request={}", receipt.duplicate_request);
    println!("token_count={}", receipt.token_count);
    println!("receipt_proof_event_seq={}", receipt.proof_event_seq);
    println!("receipt_proof_hash={}", receipt.proof_hash);
    println!("receipt_hash={}", receipt.receipt_hash);
    println!("receipt_event_seq={}", receipt.event_seq);
    println!("receipt_verified={}", receipt_verified && receipt_replay_count == 1);
    println!("tlog_path={}", tlog_path.display());
    println!("tampered_tlog_path={}", tampered_tlog_path.display());
    println!("tamper_rejected={}", tamper_rejected);
    println!("tampered_fields_rejected={tampered_rejected}/{tampered_total}");
    println!("judgment_passed={}", state.gates.judgment.status == GateStatus::Pass);
    println!("event_from={:?}", persisted_event.from);
    println!("event_to={:?}", persisted_event.to);
    println!("phase_is_plan={}", state.phase == Phase::Plan);
    println!("proof_event_seq={}", proof_event.proof_event_seq);
    println!("proof_line_hash={}", proof_event.proof_line_hash);
    println!("proof_hash={}", proof_event.proof_hash);
    println!(
        "proof_order_verified={}",
        proof_order_count == proof_replay_count && proof_order_count == 1
    );
    println!(
        "durable_proof_verified={}",
        durable_proof_count == 1 && durable_receipt_count == 1
    );
    println!(
        "receipt_proof_matches={}",
        proof_event.matches_receipt(receipt, &loaded_tlog)
    );
    println!(
        "ollama_judgment_proof={}",
        OLLAMA_JUDGMENT_PROOF_LINE
    );

    std::fs::remove_file(tampered_tlog_path).ok();
    Ok(())
}