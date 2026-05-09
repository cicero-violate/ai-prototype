# Agent Loop Contract

## Contract Status

```text
status: draft
wiring: none
runtime_effect: none
```

## Loop Phases

```text
Observe
  read worker state, TLog, policy, and available capability surfaces

Orient
  identify current phase, failed/missing gate, objective, risk, and context

Decide
  choose next action kind (see Action Kinds below)

Act
  execute the chosen action kind and produce a receipt
  (see receipt rules per action kind below)

Submit
  wrap evidence in CommandEnvelope / ApiTransportFrame

Verify
  verify TLog, command ledger, transport receipt, and effect receipt

Evaluate
  score outcome against objective and risk constraints

Learn
  produce promotion candidate only after verified eval
```

## Action Kinds and Their Receipts

These are not equivalent. Each produces different evidence.

```text
LLM Turn (via router-server)
  POST /v1/chat/completions → OpenAiChatResponse { content, target_url }
  receipt: OpenAiLlmEffectReceipt
  evidence: the LLM call itself, not any tools ChatGPT may have called internally
  note: if ChatGPT uses MCP tools during this turn, those are invisible to the agent.
        the agent receives only the final text content.

Direct Tool Call (via mcp_worker)
  LiveMcpCallExecutor.execute_call(tool, args) → McpCallReceipt
  receipt: McpCallReceipt
  evidence: the specific tool, arguments, output hash, and exit status

Observation Ingress
  ObservationIngressBatch → EvidenceSubmission
  receipt: evidence hash from the batch

Process Execution (sandbox)
  LiveSandboxProcessExecutor.execute_process(command, args) → SandboxProcessReceipt
  receipt: SandboxProcessReceipt
```

The agent must not assume it can receipt tool calls that happened inside a
ChatGPT session. If the agent needs to know what tools ChatGPT used, it must
parse response text — which is unstructured and not a reliable evidence source.

## Conceptual Records

```text
AgentObjective
  objective_id
  objective_hash
  domain_hint
  success_metric_hash
  risk_envelope_hash
  stop_condition_hash

AgentObservation
  state_hash
  tlog_hash
  phase
  missing_gate
  policy_hash
  route_hash

AgentDecision
  selected_path
  selected_capability
  selected_gate
  expected_evidence
  confidence_score
  uncertainty_score
  rationale_hash

AgentAction
  action_kind
  provider_or_tool_hash
  request_hash
  expected_receipt_kind
  timeout_ms
  max_output_bytes

AgentReceipt
  action_hash
  capability_receipt_hash
  command_hash
  event_hash
  verification_hash

AgentRunSummary
  objective_hash
  final_state_hash
  tlog_hash
  event_count
  success
  eval_hash
  promotion_candidate_hash
```

## Stop Conditions

- Runtime reaches `Phase::Done`.
- TLog verification fails.
- Capability receipt verification fails.
- Max agent steps reached.
- Max retry budget reached.
- Human-review boundary reached.
- Risk envelope blocks the action.
- External provider unavailable past retry budget.

## Required Invariants

- Every agent step must be traceable to an objective.
- Every external effect must have a receipt.
- Every command must be replay-safe.
- Every transition must pass `verify_tlog`.
- Every learning candidate must have eval support.
- Every retry must be bounded.
- Every high-risk action must be blocked or require review.

## TODO

- [ ] Define exact hash layout for each conceptual record.
- [ ] Define max agent step budget.
- [ ] Define retry budget.
- [ ] Define provider failure handling.
- [ ] Define command id generation.
- [ ] Define worker API error recovery.
- [ ] Define replay verification cadence.
