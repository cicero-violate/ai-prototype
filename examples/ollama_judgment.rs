#![forbid(unsafe_code)]

use ai::{
    api::routes::{handle_command, handle_envelope},
    append_ollama_llm_effect_receipt_ndjson, load_ollama_llm_effect_receipts_ndjson,
    load_tlog_ndjson, tick, verify_ollama_llm_effect_receipts, verify_tlog, write_tlog_ndjson,
    Command, CommandEnvelope, ContextRecord, EventKind, Evidence, GateStatus, MemoryFact,
    MemoryIndex, ObservationRecord, OllamaClient, Phase, PolicyStore, RuntimeConfig, State,
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
    let receipt = call
        .receipt_for_event(envelope.command_hash, persisted_event)
        .ok_or_else(|| std::io::Error::other("failed to bind ollama receipt to tlog event"))?;

    let tlog_path = std::env::var("CANON_OLLAMA_TLOG")
        .unwrap_or_else(|_| "ollama_judgment.tlog.ndjson".to_string());
    let tlog_path_buf = std::path::PathBuf::from(&tlog_path);
    write_tlog_ndjson(&tlog_path_buf, &tlog)?;
    append_ollama_llm_effect_receipt_ndjson(&tlog_path_buf, &receipt)?;

    let disk_tlog = load_tlog_ndjson(&tlog_path_buf)?;
    let disk_receipts = load_ollama_llm_effect_receipts_ndjson(&tlog_path_buf)?;
    verify_tlog(&disk_tlog)?;
    verify_ollama_llm_effect_receipts(&disk_tlog, &disk_receipts)?;

    verify_tlog(&tlog)?;

    println!("provider=ollama");
    println!("base_url={}", client.config().base_url);
    println!("model={}", client.config().model);
    println!("prompt_hash={}", llm.prompt.prompt_hash);
    println!("request_hash={}", call.request_hash);
    println!("response_hash={}", llm.response.response_hash);
    println!("raw_response_hash={}", call.raw_response_hash);
    println!("token_count={}", llm.response.token_count);
    println!("receipt_hash={}", receipt.receipt_hash);
    println!("receipt_event_seq={}", receipt.event_seq);
    println!("receipt_verified=true");
    println!("tlog_path={}", tlog_path);
    println!(
        "judgment_passed={}",
        state.gates.judgment.status == GateStatus::Pass
    );
    println!("event_from={:?}", response.event.from);
    println!("event_to={:?}", response.event.to);
    println!("phase_is_plan={}", state.phase == Phase::Plan);

    Ok(())
}