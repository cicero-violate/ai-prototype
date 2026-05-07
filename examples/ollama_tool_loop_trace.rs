use ai::api::routes::handle_envelope;
use ai::{
    append_sandbox_process_receipt_ndjson, tick, verify_tlog, write_tlog_ndjson, Command,
    CommandEnvelope, ContextRecord, ControlEvent, GateStatus, LiveSandboxProcessExecutor,
    MemoryFact, MemoryIndex, OllamaClient, OllamaMessage, Phase, PolicyStore, RuntimeConfig, State,
    TLog,
};
use std::path::Path;

const TOOL_CALL_TARGET: usize = 5;

#[derive(Clone, Copy)]
struct ToolSpec {
    intent: &'static str,
    command: &'static str,
    args: &'static [&'static str],
    aliases: &'static [&'static str],
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = RuntimeConfig::default();
    let client = OllamaClient::from_env()?;
    let policy = PolicyStore::default();
    let mut state = State::default();
    let mut tlog: TLog = Vec::new();
    let mut tool_calls = 0usize;
    let tlog_dir = Path::new("tlog");
    std::fs::create_dir_all(tlog_dir)?;
    let tlog_path = tlog_dir.join("ollama_tool_loop_trace.tlog.ndjson");
    let process_receipt_path = tlog_dir.join("ollama_tool_loop_trace.process_receipts.ndjson");
    std::fs::remove_file(&tlog_path).ok();
    std::fs::remove_file(&process_receipt_path).ok();

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
            submit_ollama_judgment(&client, &policy, &mut state, &mut tlog, cfg)?;
        } else if state.phase == Phase::Execute && tool_calls == 0 {
            tool_calls = submit_ollama_tool_calls(
                &client,
                &process_receipt_path,
                &mut state,
                &mut tlog,
                cfg,
            )?;
        } else {
            tick(&mut state, &mut tlog, cfg)?;
        }

        for event in &tlog[before_len..] {
            print_event(event);
        }
    }

    verify_tlog(&tlog)?;
    write_tlog_ndjson(&tlog_path, &tlog)?;
    println!(
        "done phase={:?} success={} events={} tool_calls={} tlog_path={} process_receipt_path={}",
        state.phase,
        state.is_success(),
        tlog.len(),
        tool_calls,
        tlog_path.display(),
        process_receipt_path.display()
    );

    Ok(())
}

fn submit_ollama_judgment(
    client: &OllamaClient,
    policy: &PolicyStore,
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
) -> Result<(), Box<dyn std::error::Error>> {
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
    let call = client.call_from_context(&context, policy)?;

    println!(
        "llm_receipt request_hash={} response_hash={} raw_response_hash={} token_count={}",
        call.request_hash, call.response_hash, call.raw_response_hash, call.token_count
    );

    let envelope = CommandEnvelope::new(
        call.request_hash,
        Command::SubmitEvidence(call.submission()),
    );
    handle_envelope(state, tlog, cfg, envelope)?;
    Ok(())
}

fn submit_ollama_tool_calls(
    client: &OllamaClient,
    process_receipt_path: &Path,
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
) -> Result<usize, Box<dyn std::error::Error>> {
    let sandbox_root =
        std::env::temp_dir().join(format!("canon-ollama-tool-loop-{}", std::process::id()));
    let executor = LiveSandboxProcessExecutor::new(&sandbox_root)
        .with_allowed_command("/usr/bin/printf")
        .with_allowed_command("/usr/bin/pwd")
        .with_allowed_command("/usr/bin/uname")
        .with_allowed_command("/usr/bin/whoami")
        .with_allowed_command("/usr/bin/true")
        .with_locked_env("CANON_SANDBOX", "1")
        .with_timeout_ms(1000)
        .with_max_output_bytes(4096);

    let mut receipts = Vec::new();
    for tool_call_index in 1..=TOOL_CALL_TARGET {
        let spec = tool_spec(tool_call_index)?;
        println!("llm_tool_intent_call index={tool_call_index} phase=Execute provider=ollama");
        let response = client.chat(&[
            OllamaMessage::system("Return exactly the requested tool intent token. No markdown, no punctuation, no explanation."),
            OllamaMessage::user(format!(
                "Safe tool menu: RUN_PRINTF, RUN_PWD, RUN_UNAME, RUN_WHOAMI, RUN_TRUE. For tool call number {tool_call_index}, return exactly {}.",
                spec.intent
            )),
        ])?;
        let normalized = response.content.trim().to_ascii_uppercase();
        println!(
            "llm_tool_intent index={} content={:?} response_hash={} raw_hash={} token_count={}",
            tool_call_index,
            response.content.trim(),
            response.response_hash,
            response.raw_hash,
            response.total_tokens
        );

        if !matches_tool_intent(&normalized, spec) {
            return Err(format!("unsupported tool intent: {}", response.content.trim()).into());
        }
        println!(
            "tool_intent_mapped index={} mapped={} command={} args={:?}",
            tool_call_index, spec.intent, spec.command, spec.args
        );

        let receipt = executor
            .execute_process(spec.command, spec.args, "")
            .map_err(|err| format!("tool execution failed: {err:?}"))?;

        println!(
            "tool_receipt index={} command={} exit_status={} timed_out={} stdout_bytes={} stdout_hash={} receipt_hash={}",
            tool_call_index,
            spec.command,
            receipt.exit_status,
            receipt.timed_out,
            receipt.stdout_bytes,
            receipt.stdout_hash,
            receipt.receipt_hash
        );
        append_sandbox_process_receipt_ndjson(process_receipt_path, &receipt)
            .map_err(|err| format!("failed to persist process receipt: {err:?}"))?;
        receipts.push(receipt);
    }

    let batch_hash = receipts
        .iter()
        .fold(0x5_7001_ca11_u64, |hash, receipt| {
            hash ^ receipt.receipt_hash
        })
        .max(1);
    let envelope = CommandEnvelope::new(batch_hash, Command::SubmitProcessReceiptBatch(receipts));
    handle_envelope(state, tlog, cfg, envelope)?;
    Ok(TOOL_CALL_TARGET)
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

fn tool_spec(index: usize) -> Result<ToolSpec, Box<dyn std::error::Error>> {
    let spec = match index {
        1 => ToolSpec {
            intent: "RUN_PRINTF",
            command: "/usr/bin/printf",
            args: &["tool 1: printf\n"],
            aliases: &["PRINTF"],
        },
        2 => ToolSpec {
            intent: "RUN_PWD",
            command: "/usr/bin/pwd",
            args: &[],
            aliases: &["PWD"],
        },
        3 => ToolSpec {
            intent: "RUN_UNAME",
            command: "/usr/bin/uname",
            args: &["-s"],
            aliases: &["UNAME"],
        },
        4 => ToolSpec {
            intent: "RUN_WHOAMI",
            command: "/usr/bin/whoami",
            args: &[],
            aliases: &["WHOAMI"],
        },
        5 => ToolSpec {
            intent: "RUN_TRUE",
            command: "/usr/bin/true",
            args: &[],
            aliases: &["TRUE"],
        },
        _ => return Err(format!("missing tool spec for index {index}").into()),
    };
    Ok(spec)
}

fn matches_tool_intent(normalized: &str, spec: ToolSpec) -> bool {
    normalized.contains(spec.intent) || spec.aliases.iter().any(|alias| normalized.contains(alias))
}
