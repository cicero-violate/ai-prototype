use ai::api::routes::handle_envelope;
use ai::{
    tick, verify_tlog, Command, CommandEnvelope, ContextRecord, ControlEvent, GateStatus,
    MemoryFact, MemoryIndex, OllamaClient, Phase, PolicyStore, RuntimeConfig, State, TLog,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = RuntimeConfig::default();
    let client = OllamaClient::from_env()?;
    let policy = PolicyStore::default();
    let mut state = State::default();
    let mut tlog: TLog = Vec::new();

    println!(
        "ollama base_url={} model={}",
        client.config().base_url,
        client.config().model
    );
    println!("start phase={:?}", state.phase);

    for _ in 0..cfg.max_steps {
        if state.phase == Phase::Done {
            break;
        }

        let before_len = tlog.len();

        if state.phase == Phase::Judgment && state.gates.judgment.status != GateStatus::Pass {
            println!("llm_call phase=Judgment provider=ollama");
            let mut memory = MemoryIndex::default();
            let _inserted = memory.insert(MemoryFact::new(state.packet.objective_id, 0xfeed, 7, 1));
            let (lookup, memory_receipt) = memory.lookup_with_receipt(state.packet.objective_id, 8);
            let context = ContextRecord::from_packet_memory_receipt(
                state.packet,
                0xabc,
                &lookup,
                Some(&memory_receipt),
            );
            let call = client.call_from_context(&context, &policy)?;

            println!(
                "llm_receipt request_hash={} response_hash={} raw_response_hash={} token_count={}",
                call.request_hash, call.response_hash, call.raw_response_hash, call.token_count
            );

            let envelope = CommandEnvelope::new(
                call.request_hash,
                Command::SubmitEvidence(call.submission()),
            );
            handle_envelope(&mut state, &mut tlog, cfg, envelope)?;
        } else {
            tick(&mut state, &mut tlog, cfg)?;
        }

        for event in &tlog[before_len..] {
            print_event(event);
        }
    }

    verify_tlog(&tlog)?;
    println!(
        "done phase={:?} success={} events={}",
        state.phase,
        state.is_success(),
        tlog.len()
    );

    Ok(())
}

fn print_event(event: &ControlEvent) {
    println!(
        "seq={} {:?}->{:?} kind={:?} cause={:?} evidence={:?} decision={:?} gate={:?} failure={:?} recovery={:?} command_id={} hash={:016x}",
        event.seq,
        event.from,
        event.to,
        event.kind,
        event.cause,
        event.evidence,
        event.decision,
        event.affected_gate,
        event.failure,
        event.recovery_action,
        event.api_command_id,
        event.self_hash,
    );
}
