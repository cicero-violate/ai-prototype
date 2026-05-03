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

## Equations

```text
score = geometric_mean(O, Q, A, S, K, P, E, V, X, D, L, T, R, M)
Good = max(O, Q, A, S, K, P, E, V, X, D, L, T, R, M)
```

One-line explanation: `score` measures weakest-link reliability; `Good` identifies the strongest current leverage axis.

## Evidence Reviewed

```text
base_commit = 050b9997b04e83f9ac8d86d8c83e41c7daa035b3
repository = router-server bundle restored from /mnt/data/router-server.bundle
runtime_archive = /mnt/data/router-server-runtime.tar.gz
goal_file = GOAL.md present
package_json = present
source_files = 47 syntax-checked JavaScript modules
artifact_turns = 88 committed turn artifact directories sampled with Python JSON/NDJSON parsing
```

Reviewed surfaces:

```text
GOAL.md
USAGE.md
docs/*.md
package.json
src/**/*.mjs
test/*.mjs
artifacts/turns/*/{manifest,response,replay,evaluation,request.redacted}.json
artifacts/turns/*/*.ndjson
router-server-runtime/RUNTIME_MANIFEST.json
router-server-runtime/log/chatgpt_project_agent.ndjson
router-server-runtime/.repo-agent-runtime/audit.ndjson
```

Runtime archive findings:

```text
runtime_schema_version = 1
runtime_mode = runtime-artifacts
runtime_download_history = []
runtime_leak_scan_findings = 0
runtime_integrity_scan_findings = 0
runtime_schema_scan_findings = 0
runtime_included_files = 2
runtime_excluded_generated_bundle = true
runtime_excluded_generated_archive = true
```

Validation results:

```text
node --version = v22.16.0
npm --version = 10.9.2
npm run check = pass; syntax_ok files=47
npm run test:unit = pass; 7/7 node:test cases passed
npm run test:mock = pass; 3/3 node:test cases passed
npm run smoke = pass; openai_contract_smoke_ok
npm run test:live = fail_environment; CDP health check returned 502 because 127.0.0.1:9221 refused connection
```

The live failure is still a project risk because the most important behavior is browser-mediated and cannot be validated without an authenticated CDP browser target.

## OBSERVE Pass — Repository, Runtime, and Validation Evidence

This pass inspected the restored repository without source-code changes. The only intended mutation is this evidence-backed scorecard update.

Recent git history observed:

```text
955dd51 Evaluate router-server scorecard
050b999 ready for agent run
7c556c7 ready for agent run
6461f90 first commit
```

Repository/build metadata observed:

```text
package_name = ai-chromium-router
package_type = module
declared_scripts = check, serve, smoke, test, test:unit, test:live
check_command = node src/tools/check-syntax.mjs
unit_command = node --test test/openai-contract.test.mjs
live_command = node --test test/live-cdp-9221.test.mjs
source_mjs_files_under_src = 44
syntax_checked_mjs_files_total = 46
test_files = 3
test_lines = 746
```

Uploaded runtime archive evidence:

```text
archive = /mnt/data/router-server-runtime.tar.gz
included_files = RUNTIME_MANIFEST.json, log/chatgpt_project_agent.ndjson, .repo-agent-runtime/audit.ndjson
runtime_manifest_schema_version = 1
runtime_manifest_mode = runtime-artifacts
runtime_manifest_base_commit = 050b9997b04e83f9ac8d86d8c83e41c7daa035b3
runtime_download_history = []
runtime_download_history_by_classification = {}
runtime_log_events = 1
runtime_audit_events = 1
runtime_audit_head_before_archive = 19fec548fcd512e611402ba05bf7a1faff8428f8c378511531d8217a4eab913e
runtime_leak_scan_finding_count = 0
runtime_integrity_scan_finding_count = 0
runtime_schema_scan_finding_count = 0
excluded_runtime_payloads = generated-bundle, generated-runtime-archive
```

Committed artifact corpus observed with Python JSON/NDJSON parsing:

```text
turn_artifact_dirs = 88
request_redacted_json_files = 88
response_json_files = 80
response_json_openai_chat_completion_shape = 0
response_json_internal_turn_record_shape = 80
schema_dirs = 86
raw_capture_ndjson_files = 26
raw_capture_blocked_json_files = 43
replay_json_files = 80
replay_match_true = 39
replay_match_false = 41
evaluation_json_files = 80
evaluation_replay_match_true = 39
evaluation_replay_match_false = 41
```

Validation re-run in this environment:

```text
node --version = v22.16.0
npm --version = 10.9.2
npm run check = pass; syntax_ok files=47
npm run test:unit = pass; 7/7 node:test cases passed
npm run test:mock = pass; 3/3 node:test cases passed
npm run smoke = pass; openai_contract_smoke_ok
npm run test:live = fail_environment; healthz status=502; code=cdp_unavailable; connect ECONNREFUSED 127.0.0.1:9221
```

Concrete constraints:

```text
authenticated_browser_session_required = true
local_CDP_port_required = 9221
live_test_starts_router_http_port = 18081
current_environment_has_no_reachable_CDP_browser = true
strict_browser_behavior_not_proven_by_current_validation = true
```

Missing validation signals:

```text
missing_live_CDP_pass = true
missing_no_browser_CDP_mock_integration = false
missing_artifact_quality_threshold_test = true
missing_SDK_client_smoke_test = true
missing_auth_boundary_test = true
missing_tool_calling_contract_test = true
missing_response_format_contract_test = true
missing_streaming_error_contract_test = true
missing_receipt_tamper_rejection_test = true
missing_policy_promotion_regression_test = true
```

Risk update:

```text
risk = browser_dependency ∨ weak_artifact_quality_gate ∨ shallow_receipts ∨ policy_learning_unproven ∨ OpenAI_semantic_gap
```

One-line explanation: local tests prove the envelope helpers and syntax are coherent, but current evidence still does not prove authenticated browser execution, strict OpenAI semantics, durable replay, or measured learning improvement.

## EXECUTE Pass — Mocked Browser Boundary Integration Evidence

This pass implemented the highest-leverage planned improvement: a deterministic no-browser CDP fixture that exercises the real `/v1/chat/completions` handler, CDP command seam, network capture path, response extraction, replay, redaction, artifact writing, and streaming SSE output without requiring an authenticated browser.

Source changes made:

```text
src/api/openai-compatible.mjs = adds injectable cdpSocketFactory seam while preserving real CdpSocket default
test/mock-cdp-integration.test.mjs = adds offline mocked CDP integration tests
package.json = makes npm test run offline unit + mock integration; keeps test:live as manual/environment-gated
score.md = records this evidence and residual risk
```

New mocked integration evidence:

```text
mock_non_streaming_route = pass; returned OpenAI chat.completion envelope with assistant content
mock_prompt_bridge = pass; captured system/developer/user bridge in submitted browser prompt
mock_streaming_route = pass; emitted assistant role chunk, ordered content deltas, finish chunk, and [DONE]
mock_tool_rejection_gate = pass; unsupported tools returned 400 before targetManager/CDP mutation
mock_artifact_manifest = pass; completed turn wrote ai_chromium.turn_manifest.v1
mock_replay = pass; replay_match=true for extracted mock assistant content
mock_redaction = pass; prompt content and browser file path absent from request.redacted.json
mock_raw_capture_policy = pass; raw-capture.blocked.json written with raw_capture_persisted=false by default
```

Validation results after implementation:

```text
node --version = v22.16.0
npm --version = 10.9.2
npm run check = pass; syntax_ok files=47
npm run test:unit = pass; 7/7 node:test cases passed
npm run test:mock = pass; 3/3 node:test cases passed
npm run smoke = pass; openai_contract_smoke_ok
npm test = pass; test:unit + test:mock
npm run test:live = fail_environment/timeout; healthz status=502; code=cdp_unavailable; connect ECONNREFUSED 127.0.0.1:9221
```

Residual constraint:

```text
mocked_CDP_pass ≠ authenticated_browser_pass
```

One-line explanation: offline validation now proves the route and artifact pipeline through a mocked browser boundary, but it still does not prove live authenticated ChatGPT/Gemini behavior under UI/network drift.

## Goal Alignment

GOAL.md defines the project as a local browser-control server exposing an OpenAI-compatible API over authenticated browser sessions, with target flow:

```text
request
  → provider selection
  → capability plan
  → browser action
  → evidence capture
  → dataset registry
  → feature extraction
  → pattern mining
  → policy decision
  → response extraction
  → feedback + replay/privacy verification
  → policy update
  → OpenAI-compatible response
```

Current implementation partially matches that flow:

```text
implemented = provider_registry
  ∧ capability_plan
  ∧ CDP_target_manager
  ∧ OpenAI-like /v1/chat/completions route
  ∧ /v1/models route
  ∧ redacted_request_artifacts
  ∧ receipts
  ∧ schema_observation
  ∧ dataset_records
  ∧ feature_vectors
  ∧ replay_records
  ∧ policy_snapshot
```

Missing or weak:

```text
weak = strict_OpenAI_semantics
  ∨ authenticated_API_behavior
  ∨ durable_replay_from_persisted_safe_evidence
  ∨ browser_live_validation_in_current_environment
  ∨ measured_policy_learning
  ∨ schema_promotion_regression_gates
```

## OpenAI-Like API Shape Verdict

```text
openai_like_shape = true
strict_openai_compatible = false
drop_in_sdk_compatible = partial
```

Evidence:

```text
GET  /v1/models exists in src/server.mjs
POST /v1/chat/completions exists in src/server.mjs
non_stream_response includes id, object, created, model, choices, usage, system_fingerprint
stream_response emits text/event-stream chunks and [DONE]
unsupported tools/function calling are rejected explicitly
many generation parameters are accepted but provider-controlled/ignored
```

Critical boundary:

```text
OpenAI-like envelope ≠ OpenAI semantic compatibility
```

The router may satisfy basic SDK calls pointed at `baseURL`, but it does not preserve the full behavior of OpenAI Chat Completions parameters, tool calls, structured outputs, token accounting, authentication, or model execution semantics.

## Current Score

| Axis | Score | Evidence-backed note |
|---|---:|---|
| `O` OpenAI endpoint/envelope compatibility | 7.3 | `/v1/chat/completions` and `/v1/models` exist; response helper adds `usage`, `system_fingerprint`, `refusal:null`, and `annotations:[]`; mocked route test now proves the handler emits the envelope end-to-end. |
| `Q` Request schema compatibility | 6.8 | `messages[]`/`prompt` are validated; mocked integration proves `system`/`developer`/`user` bridging and pre-browser `tools` rejection; many generation controls remain accepted-but-not-enforced. |
| `A` Assistant response schema compatibility | 7.8 | Contract helper produces a plausible chat completion envelope, but committed historical `response.json` turn records are internal turn records, not final OpenAI response envelopes. |
| `S` Streaming SSE compatibility | 7.3 | Mocked integration parses emitted SSE and verifies assistant role chunk, ordered content deltas, terminal finish chunk, and `[DONE]`; streaming error semantics still use SSE data payloads rather than strict HTTP failure behavior. |
| `K` SDK/drop-in compatibility | 5.7 | Basic clients likely work for simple non-tool text turns; advanced SDK surfaces fail, warn, or silently defer to browser provider behavior. |
| `P` Provider/capability architecture | 7.4 | Provider adapters, registry, capability contracts, upload/read/send/select capabilities, and target manager are separated cleanly. |
| `E` Evidence/provenance quality | 6.4 | Receipts, manifests, datasets, feedback, schema artifacts, runtime audit logs, and mocked artifact assertions exist; receipts remain shallow and not cryptographically bound to browser evidence. |
| `V` Replay/verification correctness | 6.0 | Mocked integration proves replay_match=true for extracted assistant content, but this is not yet full deterministic replay from retained, safe artifacts. |
| `X` Privacy/redaction correctness | 7.0 | Mocked integration asserts prompt/file-path absence in redacted request and raw capture blocked by default; committed history still contains 26 `raw-capture.ndjson` files. |
| `D` Data discovery + schema derivation | 6.6 | Schema observer, schema-guided extraction, master schema store, samples, feature vectors, and rule evidence exist; extraction contracts are still heuristic. |
| `L` Policy learning quality | 3.8 | Feedback and policy snapshots exist, but policy updates are driven by simple receipt/capability scores, not proven extraction improvement over replay history. |
| `T` Testability | 7.2 | Syntax, unit, smoke, and offline mocked CDP integration tests pass; live CDP test remains present but failed here due unavailable browser/CDP endpoint. |
| `R` Operational realism | 6.3 | Local browser/CDP model is now testable without a browser and still realistic for operator-owned sessions; actual live behavior remains brittle under tab/UI drift and unavailable CDP state. |
| `M` Documentation accuracy | 6.0 | GOAL/docs explain the intended architecture well; wording still risks overstating strict OpenAI compatibility and data-driven learning maturity. |

```text
current_score ≈ 6.5 / 10
Good = max(P=7.4, A=7.8, O=7.3, S=7.3, T=7.2, X=7.0, Q=6.8) = A
```

One-line explanation: the strongest current surface remains the response-envelope builder, while the weakest strategic surface is still proven policy learning.

## Artifact Evidence

Python JSON/NDJSON parsing of committed turn artifacts found:

```text
turn_artifact_dirs = 88
json_parse_errors = 0
response_json_files = 80
response_json_with_usage = 0
request_redaction_field_count = 87
raw_capture_ndjson_files = 26
raw_capture_blocked_policy_files = 43
```

Replay/evaluation status across parsed turn artifacts:

```text
(replay_match=true,  redaction_pass=false, quality=0) = 37
(replay_match=false, redaction_pass=true,  quality=0) = 40
(replay_match=true,  redaction_pass=true,  quality=1) = 2
(replay_match=false, redaction_pass=false, quality=0) = 1
(missing replay/evaluation fields) = 8
```

Interpretation:

```text
historical_artifacts_are_useful_evidence = true
historical_artifacts_prove_reliable_quality = false
```

The old artifact corpus shows meaningful instrumentation, but only 2 parsed turns achieved both replay and redaction pass. That keeps replay/privacy quality below production confidence even though current code has improved gates.

## Critical Findings

### 1. API compatibility is still shape-first

```text
shape_compatibility = routes + JSON envelope + choices + SSE
semantic_compatibility = shape + auth + exact params + tools + role semantics + errors + usage fidelity
```

The project has useful shape compatibility. It does not have strict semantic compatibility.

### 2. Browser prompt bridging is intentional but not equivalent to API role semantics

Current mapping:

```text
system/developer → literal instruction block
assistant → prior assistant text block
tool → tool result text block
user → prompt text
```

This is better than blind role concatenation, but still not equivalent to the hidden instruction hierarchy of a real Chat Completions API.

Required invariant:

```text
role_semantics_preserved ⇔ role_mapping_explicit ∧ client_expectations_documented ∧ tests_cover_role_sensitive_behavior
```

### 3. Raw capture policy improved, but committed history is mixed

Current code blocks raw capture unless:

```text
ARTIFACT_RAW_CAPTURE=1 ∨ RAW_CAPTURE_MODE=unsafe
```

However committed artifacts include both older raw captures and newer blocked policy files.

Required invariant:

```text
persist(raw_capture) ⇒ explicit_discovery_mode ∧ retention_policy ∧ redaction_or_encryption ∧ no_auth_material
```

### 4. Replay is a first verifier, not durable replay

Current replay:

```text
replay_match = hash(normalize(emitted_content)) == hash(normalize(extract(raw_capture_in_memory)))
```

This catches hardcoded-success replay, but it is still coupled to in-memory raw capture and not yet a replay engine over privacy-safe persisted evidence.

Required invariant:

```text
replay_match ⇔ extractor(safe_evidence, schema_rules, policy_version) == emitted_response_content
```

### 5. Learning records exist, but policy learning is shallow

Implemented pieces:

```text
dataset_records
feature_vectors
capability_scores
feedback_records
policy_snapshot
rule_scores
rule_lifecycle
```

Weakness:

```text
policy_update ≈ capability_score_from_receipts
```

That does not prove that discovered schemas or provider rules improve future extraction accuracy.

Required invariant:

```text
promote_rule ⇔ replay_passes(history) ∧ redaction_passes ∧ regression_rate ≤ threshold ∧ measured_delta > 0
```

### 6. Live behavior is not validated in this environment

The live test failed because `127.0.0.1:9221` refused connection. That is environmental, but it blocks the highest-value validation surface.

Required invariant:

```text
release_confidence ⇒ unit_pass ∧ smoke_pass ∧ live_CDP_pass ∧ artifact_quality_pass
```

## Highest-Leverage Next Fixes

1. Add artifact-quality tests over committed `artifacts/turns/*` requiring replay/redaction pass rates above a declared threshold.
2. Convert raw-capture persistence into an explicit discovery-mode contract with retention metadata and artifact tests.
3. Bind action receipts to evidence hashes and replay records so replay can reject tampered evidence.
4. Add strict OpenAI compatibility tests for error envelopes, ignored-parameter warnings, streaming error behavior, and SDK basic calls.
5. Add policy-promotion tests proving that schema/rule promotion requires replay pass, redaction pass, and non-regression.
6. Add authenticated live-CDP validation once an operator-owned browser is reachable on `127.0.0.1:9221`.
7. Narrow public claims to “OpenAI-like local browser-backed Chat Completions route” until strict semantic parity is tested.

## Correct Next Build Order

```text
artifact_quality_gate
  → raw_capture_retention_contract
  → evidence_hash_bound_receipts
  → durable_replay_from_safe_artifacts
  → strict_OpenAI_error_and_stream_tests
  → policy_promotion_regression_gate
  → live_CDP_CI_profile
```

## Final Assessment

```text
router_server = useful_local_browser_router
  ∧ modular_provider_capability_direction
  ∧ OpenAI_like_basic_shape
  ∧ not_strict_OpenAI_compatible
  ∧ not_yet_durable_replay_verified
  ∧ not_yet_policy_learning_proven
```

The codebase is materially stronger than a one-file scraper: it has provider adapters, capability planning, structured artifacts, schema derivation, replay records, privacy gates, and tests.

The critical truth is that the system should not yet be described as a strict OpenAI-compatible API or a fully data-driven learning router. The accurate current claim is:

```text
"OpenAI-like /v1/chat/completions envelope for local browser-backed text turns, with emerging evidence/replay/privacy infrastructure."
```