# Critical Project Rating — Router Server

## Variables

```text
O = OpenAI endpoint/envelope compatibility
Q = request schema compatibility
A = assistant response schema compatibility
S = streaming SSE compatibility
K = SDK/drop-in compatibility
P = provider/capability architecture
E = evidence/provenance quality
V = replay/verification correctness
X = privacy/redaction correctness
D = data discovery + schema derivation
L = policy learning quality
T = testability
R = operational realism
M = documentation accuracy
```

Score equation:

```text
score = geometric_mean(O, Q, A, S, K, P, E, V, X, D, L, T, R, M)
Good = max(O, Q, A, S, K, P, E, V, X, D, L, T, R, M)
```

One-line explanation: `score` measures system reliability under weakest-link pressure; `Good` identifies the strongest available leverage axis.

## Evidence Reviewed

```text
reviewed =
  README.md
  ∧ USAGE.md
  ∧ docs/*
  ∧ src/**/*.mjs
  ∧ artifacts/turns/* sampled with Python JSON parsing
  ∧ node --check src/**/*.mjs
```

Static syntax result:

```text
node_check_failures = 0
```

Important absence:

```text
package_json = missing
automated_test_suite = missing
formal_openai_schema_tests = missing
```

## OpenAI-Like API Shape Verdict

```text
openai_like_shape = true
strict_openai_compatible = false
drop_in_sdk_compatible = partial
```

The project exposes the right primary route and response envelope:

```text
POST /v1/chat/completions
GET  /v1/models

non_stream_response =
  { id, object:"chat.completion", created, model, choices:[{index,message:{role,content},finish_reason}] }

stream_response =
  text/event-stream
  data: { object:"chat.completion.chunk", choices:[{index,delta:{content},finish_reason}] }
  data: [DONE]
```

This is **OpenAI-like**, but not strict OpenAI API parity.

## OpenAI Compatibility Matrix

| Area                                     | Status          | Critical Finding                                                                                                                                                                |
|------------------------------------------+-----------------+---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `/v1/chat/completions` route             | Pass            | Correct route exists and accepts `model`, `messages`, and `stream`.                                                                                                             |
| `/v1/models` route                       | Pass            | Returns `object:"list"` and model entries. Provider metadata is extra.                                                                                                          |
| Non-stream response envelope             | Mostly pass     | Correct top-level `id`, `object`, `created`, `model`, `choices`, `message.role`, `message.content`, `finish_reason`.                                                            |
| Streaming envelope                       | Mostly pass     | Uses SSE chunks and `[DONE]`; chunk shape is close.                                                                                                                             |
| Error envelope                           | Partial         | Uses `{error:{message,type,code}}`, but streaming errors are sent as data payloads rather than strict failure semantics.                                                        |
| `usage` object                           | Fail            | Missing `usage.prompt_tokens`, `completion_tokens`, `total_tokens`. Many clients tolerate this, but strict clients expect it.                                                   |
| `system_fingerprint`                     | Missing         | Not fatal, but lowers parity.                                                                                                                                                   |
| `choice.message.refusal` / `annotations` | Missing         | Current OpenAI examples include these fields for chat messages; absence may be tolerated but is not full parity.                                                                |
| Request parameter semantics              | Weak            | `temperature`, `top_p`, `max_tokens`, `stop`, `n`, `tools`, `tool_choice`, `response_format`, `stream_options`, `user`, `store`, and logprob fields are ignored or unsupported. |
| Message role semantics                   | Weak            | `messages[]` are flattened into literal `ROLE: content` text. System/developer/tool semantics are not preserved as API semantics.                                               |
| Tool/function calling                    | Fail            | No `tool_calls`, function-call deltas, tool messages, or structured-output enforcement.                                                                                         |
| Authentication parity                    | Missing         | No Bearer auth enforcement. This is acceptable for local-only use, but not OpenAI-compatible security behavior.                                                                 |
| Response API parity                      | Not implemented | No `/v1/responses`, which is acceptable only if the claim is limited to Chat Completions compatibility.                                                                         |
| SDK drop-in behavior                     | Partial         | Basic OpenAI SDK chat-completion calls may work if pointed at `baseURL`, but advanced SDK features will break or be ignored.                                                    |

## Current Score

| Axis                                        | Score | Critical Note                                                                                                     |
|---------------------------------------------+-------+-------------------------------------------------------------------------------------------------------------------|
| `O` OpenAI endpoint/envelope compatibility  |   7.0 | Route and outer envelope are correct.                                                                             |
| `Q` Request schema compatibility            |   5.2 | Accepts common fields but flattens messages and ignores most generation/tool parameters.                          |
| `A` Assistant response schema compatibility |   7.1 | Basic `choices[0].message.content` path works. Missing usage and newer assistant fields.                          |
| `S` Streaming SSE compatibility             |   6.4 | Basic delta streaming exists. Error semantics and initial role delta are incomplete.                              |
| `K` SDK/drop-in compatibility               |   4.4 | Basic calls likely work; real OpenAI client parity is not guaranteed.                                             |
| `P` Provider/capability architecture        |   7.2 | Good modular split: providers, capabilities, API, extraction, data, policy.                                       |
| `E` Evidence/provenance quality             |   6.2 | Receipts and artifacts exist, but receipts are shallow and raw capture is risky.                                  |
| `V` Replay/verification correctness         |   3.8 | Replay record currently asserts `replay_match: true`; it does not independently replay extraction.                |
| `X` Privacy/redaction correctness           |   3.0 | Raw captures are persisted; sampled artifact has `redaction_pass:false`; classifier and redactor fields disagree. |
| `D` Data discovery + schema derivation      |   6.6 | Real schema observation and guided extraction exist. Still too heuristic and not contract-tested.                 |
| `L` Policy learning quality                 |   3.7 | Policy score updates exist, but they are driven by receipt success counts, not robust measured outcomes.          |
| `T` Testability                             |   2.5 | No package manifest, no test runner, no schema conformance tests, and smoke test uses wrong port.                 |
| `R` Operational realism                     |   5.8 | Handles CDP, tabs, group chat, and UI drift partially. Browser automation remains brittle.                        |
| `M` Documentation accuracy                  |   5.5 | Architecture docs are strong, but they overstate implementation maturity.                                         |

```text
current_score ≈ 5.1 / 10
openai_shape_score ≈ 6.0 / 10
```

## Critical Findings

### 1. The project is OpenAI-shaped, not OpenAI-compatible

```text
shape_compatibility = route + envelope + choices + SSE
semantic_compatibility = shape + parameter behavior + roles + tools + usage + errors + auth
```

The router has `shape_compatibility`; it does not have full `semantic_compatibility`.

### 2. Message handling is the biggest API semantic gap

Current behavior:

```text
messages[] → "ROLE: content\n\nROLE: content" → browser editor
```

This works for browser prompting but it is not equivalent to Chat Completions semantics. A `system` message becomes literal user-visible text unless the target UI interprets it organically.

Required invariant:

```text
role_semantics_preserved ⇔ system/developer/user/tool roles are mapped intentionally, not string-concatenated blindly
```

### 3. Redaction is currently broken as a scored gate

The redactor emits per-message fields:

```text
messages[*].content_redacted = true
browser.files[*].path_redacted = true
```

The classifier checks different fields:

```text
messages_redacted
browser.files_redacted
```

Therefore sampled evaluations show:

```text
redaction_pass = false
quality = 0
```

Required invariant:

```text
redaction_pass = all_private_content_removed ∧ no_raw_secret_retention ∧ classifier_matches_redactor_schema
```

### 4. Replay is not real replay yet

Current replay hashes output and returns `replay_match: true`.

That is a placeholder, not verification.

Required invariant:

```text
replay_match ⇔ extractor(raw_capture, schema_rules, policy_version) == emitted_response_content
```

### 5. Data discovery exists, but learning is still shallow

Good implemented pieces:

```text
schema_observer
schema_guided_extraction
schema_master_store
rule_lifecycle_store
dataset_records
feature_vectors
capability_scores
policy_snapshot
feedback_records
```

Weakness:

```text
policy_update = receipt_success_rate
```

This does not yet prove that a discovered schema or rule improved future extraction accuracy.

Required invariant:

```text
promote_rule ⇔ replay_passes(history) ∧ redaction_passes ∧ regression_rate ≤ threshold
```

### 6. Artifacts are valuable but unsafe by default

The project writes useful artifacts, but raw capture can include sensitive provider/session data.

Required invariant:

```text
persist(raw_capture) ⇒ explicit_discovery_mode ∧ retention_policy ∧ redaction_or_encryption ∧ no_auth_material
```

### 7. Documentation is ahead of implementation

The README and docs describe a strong target architecture. The source implements a meaningful prototype, but the implementation does not yet satisfy the architecture’s claimed gates.

```text
docs_state = target_architecture
code_state = working_browser_router_prototype
gap = tests + replay + redaction + strict_api_parity + policy_promotion
```

## Highest-Leverage Fixes

1. Add an OpenAI conformance test suite:
   ```text
   fixture_request → handler → assert_openai_chat_completion_schema
   fixture_stream → SSE parser → assert_chunk_schema
   error_case → assert_error_schema
   ```
2. Add `usage` with explicit null-safe token estimates or zeros:
   ```text
   usage = { prompt_tokens, completion_tokens, total_tokens }
   ```
3. Fix redaction gate field mismatch:
   ```text
   content_redacted/path_redacted → aggregate redaction_pass
   ```
4. Replace replay placeholder:
   ```text
   raw_capture + extraction_rules + policy_version → deterministic extracted_content
   ```
5. Add `package.json` with `serve`, `check`, `test`, and `smoke` scripts.
6. Fix `smoke-test.mjs` port from `8080` to documented default `8081`.
7. Add formal request filtering:
   ```text
   unsupported_openai_param ⇒ explicit warning/error, not silent ignore
   ```
8. Separate compatibility claims:
   ```text
   OpenAI-like Chat Completions envelope ≠ strict OpenAI API compatibility
   ```
9. Gate raw capture behind opt-in discovery mode.
10. Promote schema rules only after multi-turn replay passes.

## Correct Next Build Order

```text
openai_schema_contract_tests
  → package_json_scripts
  → usage_object
  → redaction_gate_fix
  → replay_extractor_verifier
  → raw_capture_retention_gate
  → unsupported_parameter_policy
  → tool/function_call_rejection_or_support
  → schema_rule_promotion_tests
  → policy_regression_quarantine
```

## Final Assessment

```text
router_server =
  useful_local_browser_router
  ∧ good_modular_direction
  ∧ shallow_OpenAI_like_API_shape
  ∧ not_strict_OpenAI_compatible
  ∧ not_yet_safe_enough_for_long_term_raw_capture
  ∧ not_yet_data_driven_learning_complete
```

The codebase is substantially better than a one-file CDP hack because it has provider adapters, capability planning, receipts, schema derivation, artifacts, and policy files.

The hard truth: the API compatibility claim must be narrowed until tests prove strict behavior. The correct claim today is:

```text
"OpenAI-like /v1/chat/completions envelope for local browser-backed text turns."
```

Not:

```text
"OpenAI-compatible API."
```
