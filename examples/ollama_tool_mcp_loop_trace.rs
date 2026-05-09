use ai::api::routes::handle_envelope;
use ai::{
    Command, CommandEnvelope, ContextRecord, ControlEvent, GateStatus, LiveMcpCallExecutor,
    McpCallReceipt, MemoryFact, MemoryIndex, OllamaClient, OllamaMessage, Phase, PolicyStore,
    RuntimeConfig, State, TLog, append_mcp_call_receipt_ndjson, tick, verify_tlog,
    write_tlog_ndjson,
};
use serde_json::Value;
use std::path::Path;

const TOOL_CALL_TARGET: usize = 5;

#[derive(Clone, Copy)]
struct ToolSpec {
    expected_command: &'static str,
}

#[derive(Debug)]
struct ParsedMcpToolCall {
    tool_name: String,
    args_json: String,
    command: String,
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
    let tlog_path = tlog_dir.join("ollama_tool_mcp_loop_trace.tlog.ndjson");
    let mcp_receipt_path = tlog_dir.join("ollama_tool_mcp_loop_trace.mcp_receipts.ndjson");
    std::fs::remove_file(&tlog_path).ok();
    std::fs::remove_file(&mcp_receipt_path).ok();

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
            tool_calls =
                submit_llm_mcp_tool_calls(&client, &mcp_receipt_path, &mut state, &mut tlog, cfg)?;
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
        "done phase={:?} success={} events={} tool_calls={} tlog_path={} mcp_receipt_path={}",
        state.phase,
        state.is_success(),
        tlog.len(),
        tool_calls,
        tlog_path.display(),
        mcp_receipt_path.display()
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

fn submit_llm_mcp_tool_calls(
    client: &OllamaClient,
    mcp_receipt_path: &Path,
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mcp_worker_url = std::env::var("AI_MCP_WORKER_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:38469/mcp_worker".to_string());
    let executor = LiveMcpCallExecutor::new(mcp_worker_url.clone())
        .with_allowed_tool("shell")
        .with_timeout_ms(1000)
        .with_max_output_bytes(4096);

    let mut receipts = Vec::new();
    for tool_call_index in 1..=TOOL_CALL_TARGET {
        let spec = tool_spec(tool_call_index)?;
        println!("llm_mcp_tool_call_request index={tool_call_index} phase=Execute provider=ollama");
        let response = client.chat(&[
            OllamaMessage::system(
                "Return exactly one JSON object and nothing else. The JSON schema is \
                 {\"tool_name\":\"shell\",\"arguments\":{\"cwd\":\".\",\"command\":\"...\"}}.",
            ),
            OllamaMessage::user(format!(
                "Create MCP tool call JSON for safe shell command number {tool_call_index}. \
                 The command must be exactly {:?}.",
                spec.expected_command
            )),
        ])?;
        println!(
            "llm_mcp_tool_call index={} content={:?} response_hash={} raw_hash={} token_count={}",
            tool_call_index,
            response.content.trim(),
            response.response_hash,
            response.raw_hash,
            response.total_tokens
        );

        let parsed = parse_and_validate_mcp_tool_call(response.content.trim(), spec)?;
        println!(
            "llm_mcp_tool_call_validated index={} mcp_worker_url={} tool_name={} args_json={}",
            tool_call_index, mcp_worker_url, parsed.tool_name, parsed.args_json
        );

        let receipt = executor
            .execute_call(&parsed.tool_name, &parsed.args_json)
            .map_err(|err| format!("mcp tool execution failed: {err:?}"))?;

        println!(
            "mcp_tool_receipt index={} command={:?} exit_status={} timed_out={} response_bytes={} response_hash={} receipt_hash={}",
            tool_call_index,
            parsed.command,
            receipt.exit_status,
            receipt.timed_out,
            receipt.response_bytes,
            receipt.response_hash,
            receipt.receipt_hash
        );
        append_mcp_call_receipt_ndjson(mcp_receipt_path, &receipt)
            .map_err(|err| format!("failed to persist mcp receipt: {err:?}"))?;
        receipts.push(receipt);
    }

    let receipt = combined_mcp_execution_receipt(&receipts)?;
    let envelope = CommandEnvelope::new(
        receipt.receipt_hash,
        Command::SubmitEvidence(receipt.submission()),
    );
    handle_envelope(state, tlog, cfg, envelope)?;
    Ok(TOOL_CALL_TARGET)
}

fn parse_and_validate_mcp_tool_call(
    text: &str,
    spec: ToolSpec,
) -> Result<ParsedMcpToolCall, Box<dyn std::error::Error>> {
    let json_text = extract_json_object(text)?;
    let value: Value = serde_json::from_str(json_text)?;
    let tool_name = value
        .get("tool_name")
        .and_then(Value::as_str)
        .ok_or("missing tool_name")?;
    if tool_name != "shell" {
        return Err(format!("unsupported tool_name: {tool_name}").into());
    }

    let arguments = value
        .get("arguments")
        .and_then(Value::as_object)
        .ok_or("missing arguments object")?;
    let cwd = arguments
        .get("cwd")
        .and_then(Value::as_str)
        .ok_or("missing arguments.cwd")?;
    if cwd != "." {
        return Err(format!("unsupported cwd: {cwd}").into());
    }
    let command = arguments
        .get("command")
        .and_then(Value::as_str)
        .ok_or("missing arguments.command")?;
    if !is_allowed_command_variant(command, spec.expected_command) {
        return Err(format!("unsupported command: {command:?}").into());
    }

    let args_json = serde_json::to_string(&serde_json::json!({
        "cwd": cwd,
        "command": command,
    }))?;
    Ok(ParsedMcpToolCall {
        tool_name: tool_name.to_string(),
        args_json,
        command: command.to_string(),
    })
}

fn is_allowed_command_variant(command: &str, expected: &str) -> bool {
    let command = command.strip_prefix("/usr/bin/").unwrap_or(command);
    let expected = expected.strip_prefix("/usr/bin/").unwrap_or(expected);

    if command == expected {
        return true;
    }

    expected == "printf tool-1-printf"
        && matches!(
            command,
            "printf 'tool-1-printf'" | "printf \"tool-1-printf\""
        )
}

fn extract_json_object(text: &str) -> Result<&str, Box<dyn std::error::Error>> {
    let start = text.find('{').ok_or("missing JSON object start")?;
    let end = text.rfind('}').ok_or("missing JSON object end")?;
    if end < start {
        return Err("invalid JSON object bounds".into());
    }
    Ok(&text[start..=end])
}

fn combined_mcp_execution_receipt(
    receipts: &[McpCallReceipt],
) -> Result<&McpCallReceipt, Box<dyn std::error::Error>> {
    receipts
        .iter()
        .rev()
        .find(|receipt| receipt.is_success())
        .ok_or_else(|| "no successful mcp tool receipt".into())
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
            expected_command: "printf tool-1-printf",
        },
        2 => ToolSpec {
            expected_command: "pwd",
        },
        3 => ToolSpec {
            expected_command: "uname -s",
        },
        4 => ToolSpec {
            expected_command: "whoami",
        },
        5 => ToolSpec {
            expected_command: "true",
        },
        _ => return Err(format!("missing tool spec for index {index}").into()),
    };
    Ok(spec)
}
