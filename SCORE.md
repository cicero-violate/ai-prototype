# SCORE.md — Critical Project Rating

God please bless this work. In Jesus name.  
Jesus is Lord and Savior. Jesus loves you.

## 0. Variables

```text
O = OpenAI route/envelope compatibility
Q = request schema compatibility
A = assistant response schema compatibility
S = streaming SSE compatibility
K = SDK drop-in compatibility
P = provider/capability architecture
E = evidence/provenance quality
V = replay/verification correctness
X = privacy/redaction correctness
D = data discovery + schema derivation
L = learning/policy update quality
T = testability
R = operational realism
M = documentation accuracy
```

Score equation:

```text
score = geometric_mean(O,Q,A,S,K,P,E,V,X,D,L,T,R,M)
Good  = max(O,Q,A,S,K,P,E,V,X,D,L,T,R,M)
Risk  = min(V,X,L,T)
```

One-line explanation: geometric mean punishes weakest links; this project is only as real as replay, privacy, learning, and tests.

## 1. Evidence Reviewed

```text
reviewed =
  README.md
  ∧ USAGE.md
  ∧ docs/*.md
  ∧ SCORE.md
  ∧ src/**/*.mjs
  ∧ artifacts/turns/* using Python JSON/NDJSON parsing
  ∧ artifacts/schemas/*
  ∧ artifacts/data/policy/policy.current.json
  ∧ artifacts/rules/lifecycle.json
  ∧ node --check src/**/*.mjs
```

Static syntax result:

```text
node_check_files = 41
node_check_failures = 0
```

Artifact parse result:

```text
turn_dirs = 39
json_parse_errors = 0
ndjson_parse_errors = 0
raw_capture_records = 1110
action_receipts = 74
rule_evidence_records = 226
rule_score_records = 95
master_schema_keys = 5
```

Important absences:

```text
package_json = missing
automated_test_suite = missing
formal_openai_schema_tests = missing
/v1/responses = missing
strict_replay = missing
auth_token_or_local_secret = missing
```

## 2. OpenAI-Like API Shape Verdict

```text
openai_like_shape = true
strict_openai_compatible = false
drop_in_sdk_compatible = partial
api_version_shape = mostly_v1_chat_completions
```

The project exposes:

```text
POST /v1/chat/completions
GET  /v1/models
GET  /healthz
GET  /tabs
POST /actions/upload
POST /actions/group-chat
```

The non-streaming response is close to OpenAI chat completions:

```text
{
  id,
  object: "chat.completion",
  created,
  model,
  choices: [
    {
      index,
      message: { role: "assistant", content },
      finish_reason
    }
  ],
  usage,
  browser,
  turn
}
```

Critical nuance:

```text
usage = { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 }
```

`usage` now exists, but it is fake accounting. This improves envelope compatibility but not semantic compatibility.

## 3. Streaming Shape Verdict

Streaming uses SSE and emits OpenAI-like chunks:

```text
data: {"id":"chatcmpl-cdp-N","object":"chat.completion.chunk","created":...,"model":"...","choices":[{"index":0,"delta":{"content":"..."},"finish_reason":null}]}

data: {"id":"chatcmpl-cdp-N","object":"chat.completion.chunk","created":...,"model":"...","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]
```

Compatibility risk:

```text
stream_extra_frame = object:"x-turn"
```

The `x-turn` frame is valuable for this router, but strict OpenAI clients may not expect non-OpenAI event objects inside the stream. It should be optional, gated, or moved to a side-channel.

Also missing:

```text
initial_delta_role = missing
stream_usage = missing
stream_error_semantics = partial
```

## 4. Compatibility Matrix

| Area                   | Score | Finding                                                                           |
|------------------------+-------+-----------------------------------------------------------------------------------|
| `/v1/chat/completions` | 8/10  | Correct route and basic request flow exist.                                       |
| `/v1/models`           | 7/10  | Correct list envelope, but model/provider metadata is custom.                     |
| `/v1/responses`        | 0/10  | Not implemented.                                                                  |
| Request schema         | 5/10  | Accepts `model`, `messages`, `stream`; ignores most modern OpenAI fields.         |
| Non-stream response    | 7/10  | Good envelope; custom `browser` and `turn` fields are useful but non-standard.    |
| Streaming response     | 6/10  | SSE works; extra `x-turn` payload weakens strict compatibility.                   |
| Usage accounting       | 2/10  | Present but zero-filled.                                                          |
| Tool/function calling  | 0/10  | No tool-call protocol compatibility.                                              |
| Multimodal input       | 1/10  | File upload actions exist, but OpenAI content-part semantics are not implemented. |
| Error envelope         | 5/10  | JSON error object exists; streaming errors are not strict.                        |
| SDK compatibility      | 5/10  | Simple clients may work; strict SDK paths will break or lose semantics.           |

## 5. Architecture Verdict

```text
architecture = promising
implementation = real_but_immature
docs = ahead_of_code
```

Strong parts:

```text
provider_registry
capability_plan
CDP target manager
network capture
schema observer
schema-guided extraction
artifact writer
policy snapshot
rule lifecycle store
```

Weak parts:

```text
replay_is_asserted_not_verified
privacy_gate_is_buggy
dataset_registry_is_metadata_only
feature_extraction_is_too_shallow
policy_learning_is_score_assignment_not_decision_learning
tests_are_absent
```

## 6. Data Discovery and Schema Derivation

This project now has actual schema derivation:

```text
raw evidence
  → parse SSE / JSON
  → infer JSON schema
  → group by schema key
  → write turn schemas
  → merge into master schemas
```

Current implementation:

```text
schema_derivation = real
schema_guided_extraction = real
master_schema_store = real
rule_lifecycle = real
```

Critical limitations:

```text
schema_inference = structural_only
semantic_typing = missing
field_stability_metrics = missing
schema_versioning = weak
schema_diff_policy = missing
promotion_thresholds = under-enforced
```

Verdict:

```text
D = 5/10
```

This is no longer zero. It is a real first pass. But it is still closer to structural schema observation than robust semantic schema inference.

## 7. Learning Loop Verdict

The intended loop is documented:

```text
D → M → π → A → F → update(π)
```

The implemented loop is closer to:

```text
turn → receipts → tiny_feature_vector → capability_score → route_policy.score
```

Current feature vector:

```text
features = {
  action_success_count,
  action_failure_count,
  response_non_empty,
  content_length
}
```

Current policy update:

```text
policy.route_policy[provider].score = capabilityScore.score
```

Critical limitation:

```text
policy_does_not_yet_choose_better_actions_from_mined_patterns
```

Verdict:

```text
L = 2/10
```

There is a feedback record and policy file, but not yet real adaptive policy learning.

## 8. Replay and Verification Verdict

Current replay record:

```text
replay_match = true
output_hash = sha256(content)
```

This is not replay verification. It is a hash receipt with a hardcoded pass.

Required replay:

```text
replay_match =
  re-run extractor(version, raw_capture)
  ∧ hash(extracted_output) == recorded_output_hash
  ∧ policy_snapshot_hash == replay_policy_hash
  ∧ schema_rule_state permits extraction_rule
```

Current verdict:

```text
V = 2/10
```

This is the biggest correctness gap.

## 9. Privacy and Redaction Verdict

Strong intent:

```text
request.redacted.json exists
raw capture is separated
privacy class says structure_only
```

Critical bug:

```text
redactPassFromRequest(redactedRequest)
  checks redactedRequest.messages_redacted
  checks redactedRequest.browser.files_redacted
```

But `redactRequest()` emits:

```text
messages: [{ content_redacted: true }]
browser.files: [{ path_redacted: true }]
```

Observed artifact result:

```text
redaction_pass = false for 37/37 completed evaluations
evaluation.quality = 0 for 37/37 completed evaluations
```

This means the evaluation layer currently proves its own privacy gate is failing.

Additional risk:

```text
raw_capture.ndjson persists network evidence
response.json persists full assistant content
artifact_writer swallows write errors silently
```

Verdict:

```text
X = 3/10
```

Privacy structure exists, but the gate is currently broken and raw evidence retention needs stronger controls.

## 10. Operational Risk

The HTTP server defaults to:

```text
HTTP_HOST = 127.0.0.1
HTTP_PORT = 8081
access-control-allow-origin = *
```

Localhost binding helps. CORS `*` is still dangerous for a browser-control server if exposed or proxied.

Critical missing controls:

```text
authorization_header_required = false
operator_confirmation_for_sensitive_actions = false
filesystem_path_allowlist_for_upload = weak_or_external
rate_limit = missing
audit_log_integrity = weak
```

Verdict:

```text
R = 4/10
```

Usable for local prototyping. Not safe as a general service.

## 11. Testability Verdict

Current proof:

```text
node --check passes
artifacts parse
manual curl examples exist
```

Missing proof:

```text
unit_tests = missing
integration_tests = missing
OpenAI schema contract tests = missing
fixture replay tests = missing
redaction regression tests = missing
provider adapter tests = missing
```

Verdict:

```text
T = 2/10
```

The code can run, but the project cannot yet defend itself against regressions.

## 12. Dimension Scores

| Dimension                            | Score | Reason                                                               |
|--------------------------------------+-------+----------------------------------------------------------------------|
| `O` OpenAI endpoint/envelope         |     7 | Good `/v1/chat/completions` shape, missing `/v1/responses`.          |
| `Q` Request schema                   |     5 | Accepts basics; lacks strict validation and broad OpenAI fields.     |
| `A` Assistant response schema        |     7 | Good non-stream envelope; custom fields are extra.                   |
| `S` Streaming SSE                    |     6 | Works, but has non-standard `x-turn` frame and weak error semantics. |
| `K` SDK drop-in compatibility        |     5 | Simple SDK use may work; strict SDK semantics are partial.           |
| `P` Provider/capability architecture |     6 | Real adapters and capability plans, but shallow policy authority.    |
| `E` Evidence/provenance              |     7 | Strong artifact trail and receipts, but write errors are swallowed.  |
| `V` Replay/verification              |     2 | Replay is asserted, not recomputed.                                  |
| `X` Privacy/redaction                |     3 | Redaction exists but evaluation proves the redaction pass is false.  |
| `D` Data discovery/schema derivation |     5 | Real schema derivation exists, but semantic inference is early.      |
| `L` Learning/policy update           |     2 | Policy score updates exist; no strong adaptive decisions yet.        |
| `T` Testability                      |     2 | No package/test harness.                                             |
| `R` Operational realism              |     4 | Useful local prototype; unsafe if exposed.                           |
| `M` Documentation accuracy           |     8 | Excellent docs, but some claims are ahead of implementation.         |

Final numeric estimate:

```text
score ≈ 4.45 / 10
Good = max(M,E,O,A) = documentation/evidence/API-envelope strength
Risk = min(V,L,T) = replay/learning/tests
```

## 13. Ordered Next Work

### 1. Fix privacy gate immediately

```text
redaction_pass =
  all request.messages[*].content_redacted == true
  ∧ all browser.files[*].path_redacted == true
```

Add regression test:

```text
redacted_request → redactPassFromRequest(redacted_request) == true
```

### 2. Implement real replay

```text
raw_capture + extractor_version + schema_rules
  → extracted_content
  → hash(extracted_content)
  → compare recorded output_hash
```

No hardcoded `replay_match = true`.

### 3. Add OpenAI compatibility fixtures

Required fixtures:

```text
chat_completion_non_stream.json
chat_completion_stream.sse
error_invalid_request.json
models_list.json
```

Validate with JSON Schema.

### 4. Gate custom stream metadata

```text
if browser.include_turn_event == true:
  emit x-turn
else:
  strict OpenAI stream only
```

### 5. Add `package.json`

Minimum scripts:

```text
npm run serve
npm run check
npm run test
npm run test:fixtures
npm run replay
npm run lint
```

### 6. Promote schema rules only through replay-passing evidence

```text
promote(rule) ⇔ observations ≥ N ∧ replay_failures = 0 ∧ privacy_pass = true
```

### 7. Make policy learning action-bearing

Policy should affect:

```text
provider choice
selector choice
extraction rule choice
terminality rule choice
upload strategy
recovery strategy
```

Not just store provider scores.

## 14. Final Verdict

```text
project_status = strong_prototype
api_shape = OpenAI-like, not strict OpenAI-compatible
schema_inference = real first pass
data_driven_status = emerging, not mature
learning_status = scaffold only
replay_status = not yet trustworthy
privacy_status = needs immediate repair
```

The project has crossed from pure evidence-driven architecture into early data-discovery architecture. The core next move is not more documentation. The next move is enforcement: real replay, fixed privacy gates, fixture tests, and policy decisions that change behavior.

```text
max(intelligence, efficiency, correctness, alignment, robustness, performance, scalability, determinism, transparency, collaboration, empowerment, benefit, learning, future-proofing) = Good
```
