client request
└── POST /v1/chat/completions
    └── src/server.mjs
        ├── readBody + parseJsonObject
        ├── validateChatCompletionRequest
        └── runTurn(request)
            ├── normalize prompt
            │   └── messagesToPrompt(messages) or request.prompt
            ├── resolve provider adapter
            │   └── provider/registry.mjs
            │       ├── explicit browser.provider
            │       └── model/request matcher
            ├── create turn state
            │   ├── turn_id
            │   ├── completion_id
            │   ├── receipt store
            │   └── artifact writer
            ├── redact + persist request evidence
            │   ├── request.redacted.json
            │   └── capability-plan.json
            ├── build capability plan
            │   └── capability/plan.mjs
            │       ├── select_project?       optional
            │       ├── upload_file?          optional
            │       ├── attach_artifact?      optional
            │       ├── send_message          default
            │       └── read_response         captured, not direct step
            ├── find/create browser tab
            │   └── browser/target-manager.mjs
            │       ├── list CDP targets
            │       ├── match provider surface
            │       ├── create new target if needed
            │       └── activate target
            ├── connect CDP websocket
            │   └── browser/cdp-socket.mjs
            ├── enable browser/network/runtime domains
            ├── install network capture
            │   └── capture/network-capture.mjs
            │       ├── backend-api/conversation
            │       ├── backend-api/f/conversation
            │       ├── event-stream frames
            │       └── websocket frames when enabled
            ├── execute capability plan
            │   ├── executeSelectProject
            │   ├── executeUploadFile
            │   └── executeSendMessage
            │       ├── wait for editor
            │       ├── prepare group chat if needed
            │       ├── inject prompt through DOM
            │       ├── submit message
            │       └── record send receipt
            ├── process captured response stream
            │   └── capability/read-response.mjs
            │       ├── parse SSE frames
            │       ├── parse JSON payloads
            │       ├── reassemble patch/appends
            │       ├── schema-guided extraction
            │       ├── generic candidate walking fallback
            │       ├── accumulate assistant text deltas
            │       ├── group message/tool/reasoning phases
            │       ├── track turn metadata
            │       └── record read receipt
            ├── derive and update schemas
            │   └── extraction/
            │       ├── schema-derivation.mjs
            │       ├── schema-guided-extraction.mjs
            │       └── schema-master-store.mjs
            ├── persist turn artifacts
            │   └── artifacts/turns/<turn_id>/
            │       ├── response.json
            │       ├── action-receipts.ndjson
            │       ├── schemas/*
            │       ├── raw-capture.blocked.json or raw-capture.ndjson
            │       └── manifest.json
            ├── create derived data records
            │   ├── dataset-records.ndjson
            │   └── feature-vectors.ndjson
            ├── score/mining layer
            │   ├── capability-scores.ndjson
            │   ├── rule-evidence.ndjson
            │   └── rule-scores.ndjson
            ├── policy update
            │   ├── read artifacts/data/policy/policy.current.json
            │   ├── update route_policy[provider].score
            │   ├── write policy.current.json
            │   └── write policy-snapshot.json
            ├── feedback + replay verification
            │   ├── feedback.ndjson
            │   ├── replay.json
            │   └── evaluation.json
            └── OpenAI-compatible response
                ├── non-stream: JSON chat.completion
                └── stream: SSE chunks + [DONE]
