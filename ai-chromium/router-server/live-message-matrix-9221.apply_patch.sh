#!/usr/bin/env bash
set -euo pipefail

apply_patch <<'PATCH'
*** Begin Patch
*** Delete File: test/live-cdp-9221.test.mjs
*** End Patch
PATCH

apply_patch <<'PATCH'
*** Begin Patch
*** Add File: test/live-cdp-9221.test.mjs
+import assert from "node:assert/strict";
+import { spawn } from "node:child_process";
+import test from "node:test";
+
+const REPO_ROOT = new URL("../", import.meta.url);
+const HOST = process.env.LIVE_HTTP_HOST ?? "127.0.0.1";
+const HTTP_PORT = Number(process.env.LIVE_HTTP_PORT ?? 18081);
+const CDP_HOST = process.env.LIVE_CDP_HOST ?? process.env.CDP_HOST ?? "127.0.0.1";
+const CDP_PORT = Number(process.env.LIVE_CDP_PORT ?? 9221);
+const BASE_URL = process.env.LIVE_ROUTER_URL ?? `http://${HOST}:${HTTP_PORT}`;
+const USE_EXISTING_ROUTER = Boolean(process.env.LIVE_ROUTER_URL);
+const LIVE_MODEL = process.env.LIVE_MODEL ?? "chatgpt-cdp";
+const LIVE_MATRIX_LIMIT = Number(process.env.LIVE_MATRIX_LIMIT ?? 0);
+
+function sleep(ms) {
+  return new Promise((resolve) => setTimeout(resolve, ms));
+}
+
+async function fetchJson(url, options = {}) {
+  const response = await fetch(url, options);
+  const text = await response.text();
+  let body = null;
+  try {
+    body = text ? JSON.parse(text) : null;
+  } catch {
+    body = { parse_error: "non_json_response", text };
+  }
+  return { response, body, text };
+}
+
+async function fetchSseEvents(url, options = {}) {
+  const response = await fetch(url, options);
+  const text = await response.text();
+  const events = [];
+  for (const block of text.split(/\n\n+/)) {
+    const lines = block.split(/\n/).filter((line) => line.startsWith("data: "));
+    if (lines.length === 0) continue;
+    const payload = lines.map((line) => line.slice("data: ".length)).join("\n");
+    if (payload === "[DONE]") {
+      events.push({ done: true });
+      continue;
+    }
+    try {
+      events.push(JSON.parse(payload));
+    } catch {
+      events.push({ parse_error: true, payload });
+    }
+  }
+  return { response, text, events };
+}
+
+function startRouter() {
+  const logs = [];
+  const child = spawn(process.execPath, ["src/server.mjs"], {
+    cwd: REPO_ROOT,
+    env: {
+      ...process.env,
+      HTTP_HOST: HOST,
+      HTTP_PORT: String(HTTP_PORT),
+      CDP_HOST,
+      CDP_PORT: String(CDP_PORT),
+      TURN_IDLE_MS: process.env.TURN_IDLE_MS ?? "500",
+      TURN_FIRST_CAPTURE_MS: process.env.TURN_FIRST_CAPTURE_MS ?? "20000",
+      TURN_MAX_MS: process.env.TURN_MAX_MS ?? "90000",
+    },
+    stdio: ["ignore", "pipe", "pipe"],
+  });
+
+  const collect = (streamName) => (chunk) => {
+    const text = chunk.toString("utf8");
+    logs.push(`[${streamName}] ${text}`);
+  };
+  child.stdout.on("data", collect("stdout"));
+  child.stderr.on("data", collect("stderr"));
+  return { child, logs };
+}
+
+async function waitForHealth({ child, logs }, timeoutMs = 20_000) {
+  const deadline = Date.now() + timeoutMs;
+  let lastError = null;
+  while (Date.now() < deadline) {
+    if (child?.exitCode != null) {
+      throw new Error(`router exited before health check passed; code=${child.exitCode}\n${logs.join("")}`);
+    }
+    try {
+      const { response, body, text } = await fetchJson(`${BASE_URL}/healthz`);
+      if (response.status === 200) return body;
+      lastError = new Error(`healthz status=${response.status}; body=${text}`);
+    } catch (err) {
+      lastError = err;
+    }
+    await sleep(250);
+  }
+  throw new Error(`router health check did not pass on ${BASE_URL}; last=${lastError?.message ?? "none"}\n${logs.join("")}`);
+}
+
+function killRouter(child) {
+  if (!child || child.exitCode != null || child.killed) return;
+  child.kill("SIGTERM");
+}
+
+function compactBrowserRequest(extra = {}) {
+  const browser = {
+    provider: process.env.LIVE_PROVIDER || undefined,
+    target_url: process.env.LIVE_TARGET_URL || undefined,
+    reset_chat: process.env.LIVE_RESET_CHAT === "1",
+    idle_ms: Number(process.env.LIVE_IDLE_MS ?? 500),
+    first_capture_ms: Number(process.env.LIVE_FIRST_CAPTURE_MS ?? 20_000),
+    max_ms: Number(process.env.LIVE_MAX_MS ?? 90_000),
+    ...extra,
+  };
+  return Object.fromEntries(Object.entries(browser).filter(([, value]) => value !== undefined));
+}
+
+function expectedReplyInstruction(token) {
+  return [
+    `This is a router live-test case.`,
+    `Reply with exactly this token and no other text: ${token}`,
+  ].join(" ");
+}
+
+function messageMatrixCases() {
+  const cases = [
+    {
+      name: "user-only string content",
+      token: "LIVE-MSG-USER-ONLY",
+      messages: [
+        { role: "user", content: expectedReplyInstruction("LIVE-MSG-USER-ONLY") },
+      ],
+    },
+    {
+      name: "system plus user",
+      token: "LIVE-MSG-SYSTEM-USER",
+      messages: [
+        { role: "system", content: "Follow the final user instruction exactly." },
+        { role: "user", content: expectedReplyInstruction("LIVE-MSG-SYSTEM-USER") },
+      ],
+    },
+    {
+      name: "developer plus user",
+      token: "LIVE-MSG-DEVELOPER-USER",
+      messages: [
+        { role: "developer", content: "Do not explain. Return only the requested token." },
+        { role: "user", content: expectedReplyInstruction("LIVE-MSG-DEVELOPER-USER") },
+      ],
+    },
+    {
+      name: "system plus developer plus user",
+      token: "LIVE-MSG-SYSTEM-DEVELOPER-USER",
+      messages: [
+        { role: "system", content: "You are validating a browser router." },
+        { role: "developer", content: "Return only the token supplied by the user." },
+        { role: "user", content: expectedReplyInstruction("LIVE-MSG-SYSTEM-DEVELOPER-USER") },
+      ],
+    },
+    {
+      name: "assistant prior plus user",
+      token: "LIVE-MSG-ASSISTANT-PRIOR",
+      messages: [
+        { role: "user", content: "Prior setup: remember the next request wins." },
+        { role: "assistant", content: "Acknowledged." },
+        { role: "user", content: expectedReplyInstruction("LIVE-MSG-ASSISTANT-PRIOR") },
+      ],
+    },
+    {
+      name: "system plus assistant prior plus user",
+      token: "LIVE-MSG-SYSTEM-ASSISTANT-USER",
+      messages: [
+        { role: "system", content: "Ignore prior assistant text when the user asks for a token." },
+        { role: "assistant", content: "Old answer should not be repeated." },
+        { role: "user", content: expectedReplyInstruction("LIVE-MSG-SYSTEM-ASSISTANT-USER") },
+      ],
+    },
+    {
+      name: "assistant tool result plus user",
+      token: "LIVE-MSG-TOOL-RESULT",
+      messages: [
+        { role: "assistant", content: "I will use the tool result." },
+        { role: "tool", name: "fixture_tool", tool_call_id: "call_live_fixture", content: "tool_state=ok" },
+        { role: "user", content: expectedReplyInstruction("LIVE-MSG-TOOL-RESULT") },
+      ],
+    },
+    {
+      name: "multipart text content",
+      token: "LIVE-MSG-MULTIPART-TEXT",
+      messages: [
+        {
+          role: "user",
+          content: [
+            { type: "text", text: "Part A: live multipart test." },
+            { type: "text", text: expectedReplyInstruction("LIVE-MSG-MULTIPART-TEXT") },
+          ],
+        },
+      ],
+    },
+    {
+      name: "multipart input_text content",
+      token: "LIVE-MSG-INPUT-TEXT",
+      messages: [
+        {
+          role: "user",
+          content: [
+            { type: "input_text", text: "Part A: input_text bridge." },
+            { type: "input_text", text: expectedReplyInstruction("LIVE-MSG-INPUT-TEXT") },
+          ],
+        },
+      ],
+    },
+    {
+      name: "mixed array string and text parts",
+      token: "LIVE-MSG-MIXED-PARTS",
+      messages: [
+        {
+          role: "user",
+          content: [
+            "Part A: mixed message array.",
+            { type: "text", text: expectedReplyInstruction("LIVE-MSG-MIXED-PARTS") },
+          ],
+        },
+      ],
+    },
+    {
+      name: "prompt fallback without messages",
+      token: "LIVE-MSG-PROMPT-FALLBACK",
+      prompt: expectedReplyInstruction("LIVE-MSG-PROMPT-FALLBACK"),
+    },
+  ];
+  return LIVE_MATRIX_LIMIT > 0 ? cases.slice(0, LIVE_MATRIX_LIMIT) : cases;
+}
+
+function makeRequest(caseDef, overrides = {}) {
+  const request = {
+    model: LIVE_MODEL,
+    stream: false,
+    browser: compactBrowserRequest(overrides.browser),
+  };
+  if (caseDef.prompt) request.prompt = caseDef.prompt;
+  if (caseDef.messages) request.messages = caseDef.messages;
+  return { ...request, ...overrides };
+}
+
+function assertLiveCompletion({ caseDef, response, body, text, logs }) {
+  assert.equal(response.status, 200, `${caseDef.name}: live completion failed; body=${text}\n${logs.join("")}`);
+  assert.equal(body.object, "chat.completion", `${caseDef.name}: expected chat.completion object`);
+  assert.equal(body.system_fingerprint, "browser-cdp-router", `${caseDef.name}: expected router fingerprint`);
+  assert.equal(body.browser?.backend, "cdp", `${caseDef.name}: expected cdp backend`);
+  assert.ok(body.browser?.target_id, `${caseDef.name}: expected live CDP target_id`);
+  assert.ok(body.browser?.target_url, `${caseDef.name}: expected live CDP target_url`);
+  assert.equal(body.turn?.message_stream_complete, true, `${caseDef.name}: expected captured stream completion marker`);
+
+  const content = String(body.choices?.[0]?.message?.content ?? "");
+  assert.ok(content.length > 0, `${caseDef.name}: expected non-empty assistant content`);
+  assert.match(
+    content,
+    new RegExp(caseDef.token.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")),
+    `${caseDef.name}: expected response to contain token ${caseDef.token}; content=${content}`,
+  );
+}
+
+async function postChatCompletion(caseDef, request, logs) {
+  const { response, body, text } = await fetchJson(`${BASE_URL}/v1/chat/completions`, {
+    method: "POST",
+    headers: { "content-type": "application/json" },
+    body: JSON.stringify(request),
+  });
+  assertLiveCompletion({ caseDef, response, body, text, logs });
+  return body;
+}
+
+test("live OpenAI-compatible message matrix runs through CDP on port 9221", { timeout: 900_000 }, async (t) => {
+  const router = USE_EXISTING_ROUTER ? { child: null, logs: [] } : startRouter();
+  t.after(() => killRouter(router.child));
+
+  const health = await waitForHealth(router);
+  assert.equal(health.backend, "cdp");
+  assert.equal(health.cdp_url, `http://${CDP_HOST}:${CDP_PORT}`);
+
+  let priorTargetUrl = null;
+  for (const caseDef of messageMatrixCases()) {
+    await t.test(caseDef.name, async () => {
+      const request = makeRequest(caseDef);
+      const body = await postChatCompletion(caseDef, request, router.logs);
+      if (priorTargetUrl && !process.env.LIVE_TARGET_URL && process.env.LIVE_RESET_CHAT !== "1") {
+        assert.equal(
+          body.browser?.target_url,
+          priorTargetUrl,
+          `${caseDef.name}: expected target reuse when reset_chat=false and no explicit target_url is supplied`,
+        );
+      }
+      priorTargetUrl = body.browser?.target_url ?? priorTargetUrl;
+    });
+  }
+});
+
+test("live streaming message request emits OpenAI chunks and final turn envelope", { timeout: 150_000 }, async (t) => {
+  const router = USE_EXISTING_ROUTER ? { child: null, logs: [] } : startRouter();
+  t.after(() => killRouter(router.child));
+
+  const health = await waitForHealth(router);
+  assert.equal(health.backend, "cdp");
+  assert.equal(health.cdp_url, `http://${CDP_HOST}:${CDP_PORT}`);
+
+  const caseDef = {
+    name: "streaming system plus user",
+    token: "LIVE-MSG-STREAM",
+    messages: [
+      { role: "system", content: "Return only the requested token." },
+      { role: "user", content: expectedReplyInstruction("LIVE-MSG-STREAM") },
+    ],
+  };
+  const request = makeRequest(caseDef, { stream: true });
+
+  const { response, text, events } = await fetchSseEvents(`${BASE_URL}/v1/chat/completions`, {
+    method: "POST",
+    headers: { "content-type": "application/json" },
+    body: JSON.stringify(request),
+  });
+
+  assert.equal(response.status, 200, `${caseDef.name}: streaming request failed; body=${text}\n${router.logs.join("")}`);
+  assert.match(response.headers.get("content-type") ?? "", /text\/event-stream/);
+  assert.ok(events.some((event) => event.object === "chat.completion.chunk"), "expected OpenAI-compatible chunks");
+  assert.ok(events.some((event) => event.object === "x-turn"), "expected final structured turn envelope");
+  assert.ok(events.some((event) => event.done), "expected [DONE] sentinel");
+
+  const streamedText = events
+    .filter((event) => event.object === "chat.completion.chunk")
+    .map((event) => event.choices?.[0]?.delta?.content ?? "")
+    .join("");
+  assert.match(streamedText, /LIVE-MSG-STREAM/, `expected streamed content to contain token; content=${streamedText}`);
+});
*** End Patch
PATCH
