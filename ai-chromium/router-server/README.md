God please bless this work. In Jesus name.
Jesus is Lord and Savior. Jesus loves you.

# AI Chromium Router

> Browser-backed LLM router for ChatGPT private chats, ChatGPT group chats, ChatGPT Projects artifact upload, and Gemini chats.

## 0. Variables

```text
P = provider surface: chatgpt_private | chatgpt_group | chatgpt_project | gemini_private
C = capability: send_message | read_response | upload_file | select_project | attach_artifact | replay_turn
E = ordered browser evidence
R = deterministic replay result
X = extractor result
A = artifact/upload result
G = executable gate result
```

Core equation:

```text
Good = max(provider_coverage, capability_correctness, replay_determinism, privacy_safety, test_coverage, operator_clarity)
```

One-line explanation: the system is good only when each provider capability is proven by evidence, replay, privacy gates, and tests.

## 1. Contract

AI Chromium Router exposes an OpenAI-compatible API over controlled browser sessions.

It must support:

```text
chatgpt.com private chat
chatgpt.com group chat
chatgpt.com project chat
chatgpt.com project artifact upload
gemini.google.com private chat
gemini.google.com file upload when available
```

The project is not just a response extractor.

The project is a provider-capability router.

```text
request
  → provider resolution
  → capability plan
  → browser action
  → evidence capture
  → extraction
  → replay verification
  → OpenAI-compatible response
```

## 2. Current Reality

This README is the executable architecture target.

Until the repository contains passing tests for provider routing, browser actions, upload flows, replay, extraction, redaction, and OpenAI-compatible responses, the project state is:

```text
status = architecture_defined ∧ implementation_unproven
```

The previous architecture was too centered on schema-derived extraction.

The corrected architecture makes provider capability the root object.

## 3. Non-Negotiable Boundaries

This project must not:

- harvest credentials
- bypass login systems
- bypass CAPTCHA or anti-abuse checks
- evade provider rate limits
- impersonate users without explicit local operator control
- retain cookies, auth headers, tokens, or hidden account identifiers in artifacts
- promise stability for unofficial browser UI surfaces

The browser session is assumed to be operator-owned and already authenticated.

```text
auth_control = external_operator_responsibility
router_control = capability_execution_only
```

## 4. Provider Surfaces

Provider support is capability-scoped, not hardcoded per website.

| Provider Surface  | Required Capabilities                                                                 | Required Risk Control                                    |
|-------------------+---------------------------------------------------------------------------------------+----------------------------------------------------------|
| `chatgpt_private` | send prompt, detect response, extract final answer, attach files when supported       | isolate account/session profile                          |
| `chatgpt_group`   | open group target, send message, distinguish assistant response from human messages   | never scrape participants beyond needed message evidence |
| `chatgpt_project` | select project, create/open project chat, upload artifact/reference file, send prompt | artifact retention and redaction gate                    |
| `gemini_private`  | send prompt, detect response, upload files when supported, extract final answer       | UI-drift quarantine                                      |

Capability equation:

```text
execute(P, C) ⇔ adapter(P).supports(C) ∧ preflight(C) ∧ policy_allows(C)
```

## 5. Architecture Planes

```text
┌──────────────────────────────────────────────────────────────┐
│ API Plane                                                     │
│ OpenAI-compatible HTTP, target selection, file upload routing │
├──────────────────────────────────────────────────────────────┤
│ Provider Plane                                                │
│ ChatGPT private, ChatGPT group, ChatGPT project, Gemini       │
├──────────────────────────────────────────────────────────────┤
│ Capability Plane                                              │
│ Send, read, upload, select project, attach artifact, replay   │
├──────────────────────────────────────────────────────────────┤
│ Browser Plane                                                 │
│ CDP connection, target manager, DOM action executor, file I/O  │
├──────────────────────────────────────────────────────────────┤
│ Evidence Plane                                                │
│ Ordered events, action receipts, upload receipts, manifests   │
├──────────────────────────────────────────────────────────────┤
│ Extraction Plane                                              │
│ Frame parsing, schema observations, candidates, accumulators   │
├──────────────────────────────────────────────────────────────┤
│ Replay + Evaluation Plane                                     │
│ Deterministic replay, quality gates, drift quarantine          │
└──────────────────────────────────────────────────────────────┘
```

Browser execution is nondeterministic.

Replay must be deterministic.

```text
determinism_boundary = replay(evidence_bundle, extractor_version)
```

## 6. Request Model

OpenAI-compatible input remains valid:

```json
{
  "model": "chatgpt-cdp",
  "messages": [
    { "role": "user", "content": "Say hello." }
  ],
  "stream": false
}
```

Extended browser routing fields live under `browser`.

```json
{
  "model": "browser-router",
  "messages": [
    { "role": "user", "content": "Read the attached file and summarize it." }
  ],
  "stream": false,
  "browser": {
    "provider": "chatgpt_project",
    "project_hint": "Research",
    "conversation_hint": "Architecture review",
    "capabilities": ["upload_file", "send_message", "read_response"],
    "files": [
      {
        "path": "./artifacts/spec.md",
        "purpose": "project_reference"
      }
    ]
  }
}
```

Normal response must stay OpenAI-compatible.

```json
{
  "id": "chatcmpl-browser-...",
  "object": "chat.completion",
  "created": 1770000000,
  "model": "browser-router",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello."
      },
      "finish_reason": "stop"
    }
  ]
}
```

Diagnostics are returned only when `debug=true`.

```text
debug = false ⇒ response.choices[0].message.content = assistant_text_only
```

## 7. Core Data Types

### 7.1 ProviderAdapter

```json
{
  "schema": "ai_chromium.provider_adapter.v1",
  "provider": "chatgpt_project",
  "origin_patterns": ["https://chatgpt.com/*"],
  "capabilities": [
    "select_project",
    "upload_file",
    "send_message",
    "read_response"
  ],
  "stability": "ui_drift_expected",
  "requires_authenticated_profile": true
}
```

### 7.2 CapabilityPlan

```json
{
  "schema": "ai_chromium.capability_plan.v1",
  "turn_id": "turn_...",
  "provider": "chatgpt_project",
  "steps": [
    { "capability": "select_project", "project_hint": "Research" },
    { "capability": "upload_file", "path": "./artifacts/spec.md" },
    { "capability": "send_message" },
    { "capability": "read_response" }
  ]
}
```

### 7.3 ActionReceipt

```json
{
  "schema": "ai_chromium.action_receipt.v1",
  "turn_id": "turn_...",
  "seq": 12,
  "provider": "chatgpt_project",
  "capability": "upload_file",
  "status": "completed",
  "target_id": "target_...",
  "evidence_refs": ["events/000012.json"],
  "artifact_refs": ["uploads/spec.md.manifest.json"]
}
```

### 7.4 UploadArtifact

```json
{
  "schema": "ai_chromium.upload_artifact.v1",
  "turn_id": "turn_...",
  "provider": "chatgpt_project",
  "local_path": "./artifacts/spec.md",
  "file_digest": "sha256:...",
  "mime": "text/markdown",
  "size_bytes": 12345,
  "upload_status": "completed",
  "remote_label_digest": "sha256:...",
  "retention_class": "operator_provided_reference"
}
```

### 7.5 TurnRecord

```json
{
  "schema": "ai_chromium.turn.v1",
  "turn_id": "turn_...",
  "completion_id": "chatcmpl-browser-...",
  "created_at": "ISO-8601",
  "provider": "chatgpt_private",
  "mode": "serve",
  "status": "completed",
  "extractor_version": "extractor_..."
}
```

## 8. Artifact Layout

Each completed turn writes one bundle.

```text
artifacts/
└── turns/
    └── <turn-id>/
        ├── manifest.json
        ├── request.redacted.json
        ├── response.json
        ├── capability-plan.json
        ├── action-receipts.ndjson
        ├── browser-events.ndjson
        ├── network-events.ndjson
        ├── frames.ndjson
        ├── candidates.ndjson
        ├── uploads/
        │   └── *.manifest.json
        ├── rule-evidence.ndjson
        ├── replay.json
        ├── evaluation.json
        ├── schemas/
        │   ├── index.json
        │   └── *.schema.json
        └── raw/
            └── *.raw
```

Raw payload retention is denied by default.

```text
persist(raw) ⇔ retention_policy(raw) ∧ redaction_pass(raw) ∧ operator_allows(raw)
```

## 9. Browser Action Contract

Browser actions must be evidence-backed.

```text
browser_action = selector_strategy + pre_state + action + post_state + receipt
```

Action rules:

- Use CDP target discovery before DOM interaction.
- Use semantic selectors before brittle CSS selectors.
- Prefer accessibility labels, stable text anchors, and role-like affordances.
- Do not assume one provider uses the same composer, upload button, or response layout forever.
- Every action must produce an `ActionReceipt`.
- Every failed action must preserve enough redacted evidence to debug drift.

Failure equation:

```text
action_failed ⇒ emit(receipt_failed) ∧ classify_failure ∧ block_false_success
```

## 10. ChatGPT Private Adapter

Minimum capabilities:

```text
chatgpt_private = {
  open_or_create_chat,
  send_message,
  wait_for_assistant_response,
  extract_assistant_response,
  optional_file_attach
}
```

Acceptance:

```text
private_chat_pass ⇔ sent(prompt) ∧ captured(response) ∧ replay_match = 1
```

## 11. ChatGPT Group Adapter

Group chats are not equivalent to private chats.

The adapter must distinguish:

```text
human_message ≠ assistant_message ≠ local_user_message
```

Minimum capabilities:

```text
chatgpt_group = {
  open_group_chat,
  send_group_message,
  detect_chatgpt_participant_response,
  ignore_unrelated_human_messages,
  extract_assistant_response
}
```

Hard rule:

```text
emit(response_text) ⇒ provenance.author = chatgpt_assistant
```

The system must not archive group participant data beyond minimal redacted message evidence required for the turn.

## 12. ChatGPT Project Adapter

Projects require navigation and artifact management, not just chat completion.

Minimum capabilities:

```text
chatgpt_project = {
  locate_project,
  create_or_open_project_chat,
  upload_reference_file,
  wait_for_upload_completion,
  send_project_message,
  extract_assistant_response
}
```

Upload gate:

```text
upload_allowed(file) ⇔ exists(file) ∧ size_ok(file) ∧ type_allowed(file) ∧ redaction_policy_known(file)
```

Project artifact success:

```text
artifact_upload_pass ⇔ file_digest_known ∧ upload_receipt_completed ∧ remote_artifact_visible
```

## 13. Gemini Adapter

Minimum capabilities:

```text
gemini_private = {
  open_or_create_chat,
  send_message,
  optional_upload_file,
  wait_for_model_response,
  extract_model_response
}
```

Gemini support must be isolated from ChatGPT assumptions.

```text
provider_adapter(chatgpt) ∩ provider_adapter(gemini) = shared_interfaces_only
```

## 14. Extraction Contract

Extraction is provider-aware but evidence-first.

```text
raw browser/network evidence
  → frame parser
  → JSON parser
  → embedded JSON reconstructor
  → schema observer
  → candidate walker
  → provider-specific candidate classifier
  → accumulator
  → final response
```

The extractor must support both network-stream evidence and DOM fallback evidence.

```text
extract(response) ⇔ network_extractor(response) ∨ dom_snapshot_extractor(response)
```

DOM fallback is lower trust.

```text
trust(network_stream) > trust(dom_snapshot)
```

## 15. Rule Promotion

Rules are not promoted because they worked once.

```text
promote(rule) ⇔
  observations(rule) ≥ N
  ∧ failures(rule) = 0
  ∧ replay_match(rule) = 1
  ∧ redaction_pass(rule) = 1
  ∧ drift_score(rule) ≥ DRIFT_MIN
  ∧ quality(rule) ≥ QUALITY_MIN
```

Default thresholds:

```text
N = 20
DRIFT_MIN = 0.98
QUALITY_MIN = 0.995
FAILURES_ALLOWED = 0
```

Config path:

```text
config/extraction-policy.json
```

## 16. Redaction Boundary

Browser sessions can expose sensitive data.

No artifact is persistable until classified and redacted.

Must redact or block:

- cookies
- authorization headers
- session tokens
- account identifiers
- hidden URLs
- email addresses from group chats unless explicitly required
- participant metadata unrelated to the turn
- prompt text when retention policy forbids it
- uploaded file content when retention policy forbids it

Invariant:

```text
persistable(artifact) ⇔ classified(artifact) ∧ redacted(artifact) ∧ retention_allowed(artifact)
```

## 17. Evaluation Model

```text
quality = geometric_mean(
  provider_capability_pass,
  action_receipt_completeness,
  extraction_completeness,
  uniqueness,
  terminality,
  replay_match,
  drift_score,
  redaction_pass
)
```

Hard gates:

```text
quality < QUALITY_MIN ⇒ block_promotion
privacy_violation = true ⇒ block_retention
replay_match = false ⇒ block_generated_extractor
missing_action_receipt = true ⇒ fail_turn
```

## 18. Failure Taxonomy

| Failure                    | Severity | Required Behavior                                              |
|----------------------------+----------+----------------------------------------------------------------|
| `provider_not_supported`   | High     | Return controlled error before browser action.                 |
| `capability_not_supported` | High     | Return controlled error and list supported capabilities.       |
| `target_not_found`         | High     | Preserve target diagnostics and abort.                         |
| `login_required`           | High     | Stop; never automate login bypass.                             |
| `captcha_or_abuse_check`   | Critical | Stop; require operator resolution.                             |
| `upload_failed`            | High     | Emit receipt, preserve redacted evidence, block false success. |
| `project_not_found`        | Medium   | Return controlled error with project hint.                     |
| `group_author_confusion`   | Critical | Block response emission.                                       |
| `no_capture`               | High     | Abort turn with diagnostics.                                   |
| `parse_failure`            | Medium   | Preserve raw frame if retention allows; continue if safe.      |
| `schema_drift`             | High     | Quarantine affected rule.                                      |
| `duplicate_output`         | Medium   | Penalize accumulator and block promotion.                      |
| `premature_idle`           | High     | Mark lifecycle failure.                                        |
| `replay_mismatch`          | Critical | Block promotion and generated artifacts.                       |
| `privacy_violation`        | Critical | Block retention and generated artifacts.                       |

## 19. Terminality and Timers

Timers are explicit policy.

```json
{
  "schema": "ai_chromium.lifecycle_policy.v1",
  "first_capture_timeout_ms": 45000,
  "idle_timeout_ms": 2500,
  "max_turn_timeout_ms": 120000,
  "upload_timeout_ms": 180000
}
```

Completion rule:

```text
complete(turn) ⇔
  done_signal
  ∨ (idle_timeout ∧ active_streams = 0 ∧ pending_uploads = 0)
  ∨ max_turn_timeout
```

Forbidden behavior:

```text
idle_timeout ∧ active_streams > 0 ⇒ complete(turn) = false
pending_uploads > 0 ⇒ send_message = false
```

## 20. Module Layout

```text
src/
├── server.mjs
├── api/
│   ├── openai-compatible.mjs
│   ├── files.mjs
│   └── debug.mjs
├── provider/
│   ├── registry.mjs
│   ├── capability-contract.mjs
│   ├── chatgpt-private.adapter.mjs
│   ├── chatgpt-group.adapter.mjs
│   ├── chatgpt-project.adapter.mjs
│   └── gemini-private.adapter.mjs
├── browser/
│   ├── cdp-socket.mjs
│   ├── target-manager.mjs
│   ├── dom-actions.mjs
│   ├── file-chooser.mjs
│   └── lifecycle.mjs
├── capability/
│   ├── plan.mjs
│   ├── send-message.mjs
│   ├── read-response.mjs
│   ├── upload-file.mjs
│   ├── select-project.mjs
│   └── receipts.mjs
├── capture/
│   ├── normalize-event.mjs
│   ├── network-capture.mjs
│   ├── sse-capture.mjs
│   ├── websocket-capture.mjs
│   └── dom-snapshot.mjs
├── evidence/
│   ├── artifact-writer.mjs
│   ├── manifest.mjs
│   ├── digest.mjs
│   └── redaction.mjs
├── extraction/
│   ├── frame-parser.mjs
│   ├── json-parser.mjs
│   ├── embedded-json.mjs
│   ├── schema-observer.mjs
│   ├── candidate-walker.mjs
│   ├── provider-classifier.mjs
│   ├── rule-matcher.mjs
│   └── accumulator.mjs
├── replay/
│   ├── replay-turn.mjs
│   └── replay-suite.mjs
├── evaluation/
│   ├── metrics.mjs
│   ├── drift.mjs
│   └── promotion-gate.mjs
└── generated/
    ├── registry.generated.mjs
    ├── types.generated.mjs
    ├── validators.generated.mjs
    ├── extractors.generated.mjs
    └── manifest.generated.json
```

## 21. CLI Surface

```bash
npm run serve
npm run observe -- --provider chatgpt_private --prompt "Say hello"
npm run observe -- --provider chatgpt_group --prompt "Ask ChatGPT to summarize the thread"
npm run upload -- --provider chatgpt_project --project "Research" --file ./spec.md
npm run observe -- --provider gemini_private --prompt "Say hello"
npm run replay -- --turn artifacts/turns/<turn-id>
npm run replay:suite
npm run promote
npm run generate
npm run test
npm run test:replay
npm run test:redaction
npm run test:adapters
npm run lint
```

Every mutating command must print:

```text
effect_kind
artifact_path
artifact_digest
```

## 22. Test Matrix

| Test Layer        | Required Coverage                                                                |
|-------------------+----------------------------------------------------------------------------------|
| Unit              | frame parser, JSON parser, schema merge, candidate walker, accumulator, redactor |
| Provider Contract | each adapter declares capabilities and rejects unsupported capabilities          |
| Browser Action    | action receipts, selector fallback, target loss, login-required stop             |
| Upload            | file preflight, file chooser, upload receipt, timeout, failure evidence          |
| Group Chat        | assistant/human/local author separation                                          |
| Golden Replay     | stream dump → expected output, schema keys, rule firings                         |
| Drift             | old schema + changed payload → drift classification + fallback                   |
| Redaction         | sensitive browser data → retained artifact contains no forbidden fields          |
| API               | OpenAI-compatible request/response shape                                         |
| Failure           | no_capture, replay_mismatch, privacy_violation, premature_idle, upload_failed    |
| Live Smoke        | opt-in manual browser tests against real authenticated sessions                  |

Live smoke tests must not run in CI by default.

```text
CI_DEFAULT = offline_tests_only
LIVE_BROWSER_TESTS = explicit_operator_opt_in
```

## 23. Acceptance Criteria

```text
A1: every completed turn writes a manifest
A2: every browser action writes an ActionReceipt
A3: every provider declares supported capabilities
A4: unsupported capabilities fail before browser mutation
A5: ChatGPT private chat can send and read a response
A6: ChatGPT group chat can identify assistant response provenance
A7: ChatGPT project upload writes an UploadArtifact receipt
A8: Gemini private chat can send and read a response
A9: every emitted text span has provenance
A10: replay output equals live output for promoted extractors
A11: redaction runs before long-term retention
A12: generated extractors include manifest lineage
A13: generated extractors pass replay before execution
A14: idle completion is blocked while streams/uploads are active
A15: OpenAI-compatible response does not leak diagnostics
```

## 24. Implementation Phases

### Phase 0 — Minimal Router

```text
goal = OpenAI-compatible server + provider registry + capability contracts
```

Deliverables:

- `npm run serve`
- `/v1/chat/completions`
- provider registry
- controlled unsupported-capability errors

### Phase 1 — ChatGPT Private

```text
goal = reliable private ChatGPT send/read path
```

Deliverables:

- target resolution
- prompt injection
- response extraction
- turn manifest
- replay fixture

### Phase 2 — Upload + Project Routing

```text
goal = upload local artifacts into ChatGPT Projects with evidence
```

Deliverables:

- project locator
- file chooser handling
- upload receipt
- upload timeout/failure handling
- artifact manifest

### Phase 3 — ChatGPT Group Chat

```text
goal = group chat responses with correct author provenance
```

Deliverables:

- group target resolver
- author classifier
- human-message exclusion
- group replay fixture

### Phase 4 — Gemini

```text
goal = Gemini private chat send/read and optional upload
```

Deliverables:

- Gemini adapter
- Gemini response extractor
- Gemini upload preflight
- Gemini replay fixture

### Phase 5 — Replay + Promotion

```text
goal = replay-verified extraction rules and generated extractors
```

Deliverables:

- replay suite
- promotion config
- generated extractor manifest
- drift quarantine

### Phase 6 — Privacy Hardening

```text
goal = no unsafe long-term artifact retention
```

Deliverables:

- redaction tests
- forbidden-field scanner
- retention policy
- generated-code leak test

## 25. Risk Register

| Risk                         | Severity | Control                                               |
|------------------------------+----------+-------------------------------------------------------|
| Provider UI drift            | High     | provider adapters, drift tests, live smoke quarantine |
| Browser nondeterminism       | High     | replay boundary and action receipts                   |
| Upload false success         | High     | visible remote artifact check and upload receipt      |
| Group author confusion       | Critical | author provenance gate                                |
| Sensitive artifact retention | Critical | redaction-before-retention gate                       |
| Credential exposure          | Critical | operator-owned auth; never store auth material        |
| CAPTCHA / abuse check        | Critical | stop and require operator action                      |
| Overfit extraction           | High     | observation threshold + replay suite                  |
| Hidden provider assumptions  | High     | capability contracts and provider-specific tests      |
| Weak operator visibility     | Medium   | CLI effects and artifact digests                      |

## 26. Final Definition

```text
AI Chromium Router = OpenAI-compatible API
  + provider capability registry
  + controlled Chromium session
  + ChatGPT private/group/project adapters
  + Gemini adapter
  + artifact upload receipts
  + ordered evidence
  + deterministic replay
  + privacy gates
```

The original project had the right instinct: evidence first.

The corrected project target is stronger:

```text
capability_first ∧ evidence_backed ∧ replay_verified ∧ privacy_gated
```
