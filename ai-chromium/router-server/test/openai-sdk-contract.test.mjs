import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

process.env.ARTIFACTS_DIR = fs.mkdtempSync(path.join(os.tmpdir(), "router-sdk-contract-"));
delete process.env.ARTIFACT_RAW_CAPTURE;
delete process.env.RAW_CAPTURE_MODE;

const { createRouterServer, configFromEnv } = await import("../src/server.mjs");

function b64(text) { return Buffer.from(text, "utf8").toString("base64"); }

function assistantStream(text) {
  const payload = {
    conversation_id: "conv_sdk",
    v: { message: {
      id: "msg_sdk",
      author: { role: "assistant", name: "mock-assistant" },
      channel: "final",
      status: "finished_successfully",
      end_turn: true,
      content: { content_type: "text", parts: [text] },
      metadata: { request_id: "req_sdk", turn_exchange_id: "exchange_sdk", model_slug: "mock-browser-model" },
    } },
  };
  return `data: ${JSON.stringify(payload)}\n\ndata: ${JSON.stringify({ type: "message_stream_complete", conversation_id: "conv_sdk" })}\n\ndata: [DONE]\n\n`;
}

class MockCdpSocket {
  constructor({ target, reply, transcript, failConnect = false }) {
    Object.assign(this, { target, reply, transcript, failConnect, listeners: new Set(), didCapture: false });
  }
  async connect() { this.transcript.connects += 1; if (this.failConnect) throw new Error("mock CDP connect failed"); }
  onEvent(fn) { this.listeners.add(fn); return () => this.listeners.delete(fn); }
  emit(method, params) { for (const fn of this.listeners) fn(method, params); }
  close() { this.transcript.closes += 1; }
  scheduleCapture() {
    if (this.didCapture) return;
    this.didCapture = true;
    setTimeout(() => this.emit("Network.responseReceived", {
      requestId: "sdk_req_1",
      response: { url: "https://chatgpt.com/backend-api/conversation", mimeType: "text/event-stream", headers: { "content-type": "text/event-stream" } },
    }), 2);
    setTimeout(() => this.emit("Network.loadingFinished", { requestId: "sdk_req_1" }), 5);
  }
  async send(method, params = {}) {
    this.transcript.commands.push({ method, params });
    if (method === "Network.streamResourceContent") return { bufferedData: b64(assistantStream(this.reply)) };
    if (method === "Runtime.evaluate") {
      const expression = String(params.expression ?? "");
      if (expression === "1") return { result: { value: 1 } };
      if (expression.includes("location.href")) return { result: { value: this.target.url } };
      const prompt = expression.match(/const\s+prompt\s*=\s*([\s\S]*?);\s*\n/)?.[1];
      if (prompt) {
        this.transcript.prompts.push(JSON.parse(prompt));
        this.scheduleCapture();
        return { result: { value: { ok: true, method: "mock_button" } } };
      }
      return { result: { value: true } };
    }
    return {};
  }
}

function parseSse(text) {
  return text.split(/\n\n/).filter(Boolean).map((frame) => {
    const data = frame.split(/\n/).find((line) => line.startsWith("data: "))?.slice(6) ?? "";
    return data === "[DONE]" ? data : JSON.parse(data);
  });
}

async function withServer(fn, { reply = "SDK-MOCK", failConnect = false } = {}) {
  const transcript = { connects: 0, closes: 0, commands: [], prompts: [], findOrCreateCalls: 0 };
  const target = { id: "target_sdk", type: "page", title: "ChatGPT", url: "https://chatgpt.com/", webSocketDebuggerUrl: "ws://mock.invalid/devtools/page/target_sdk" };
  const config = { ...configFromEnv({}), httpHost: "127.0.0.1", httpPort: 0, cdpHost: "mock", cdpPort: 0, idleMs: 20, firstCaptureMs: 2_000, maxMs: 5_000, cdpSocketFactory: (_wsUrl, context) => new MockCdpSocket({ target: context.target, reply, transcript, failConnect }) };
  const targetManager = {
    async getVersion() { return { Browser: "MockChrome/1", "Protocol-Version": "1.3" }; },
    async listTargets() { return [target]; },
    async findOrCreate() { transcript.findOrCreateCalls += 1; return target; },
  };
  const server = createRouterServer({ config, targetManager });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const baseUrl = `http://127.0.0.1:${server.address().port}`;
  try { return await fn({ baseUrl, transcript }); }
  finally { await new Promise((resolve) => server.close(resolve)); }
}

function postChat(baseUrl, body) {
  return fetch(`${baseUrl}/v1/chat/completions`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: typeof body === "string" ? body : JSON.stringify(body),
  });
}

test("SDK matrix: models, non-streaming envelope, warnings, and response_format caveat", async () => {
  await withServer(async ({ baseUrl, transcript }) => {
    const models = await fetch(`${baseUrl}/v1/models`);
    assert.equal(models.status, 200);
    const modelBody = await models.json();
    assert.equal(modelBody.object, "list");
    assert.ok(modelBody.data.some((m) => m.id === "chatgpt_private-cdp"));

    const res = await postChat(baseUrl, {
      model: "chatgpt-cdp",
      messages: [{ role: "user", content: "Return SDK-MOCK." }],
      temperature: 0,
      response_format: { type: "json_object" },
    });
    const body = await res.json();
    assert.equal(res.status, 200);
    assert.equal(body.object, "chat.completion");
    assert.equal(body.choices[0].message.role, "assistant");
    assert.equal(body.choices[0].message.content, "SDK-MOCK");
    assert.equal(body.system_fingerprint, "browser-cdp-router");
    assert.deepEqual(body.warnings.map((w) => w.param).sort(), ["response_format", "temperature"]);
    assert.equal(transcript.findOrCreateCalls, 1);
  });
});

test("SDK matrix: streaming emits role, content, finish, and DONE in order", async () => {
  await withServer(async ({ baseUrl }) => {
    const res = await postChat(baseUrl, { model: "chatgpt-cdp", stream: true, messages: [{ role: "user", content: "stream" }] });
    assert.equal(res.status, 200);
    assert.match(res.headers.get("content-type") ?? "", /text\/event-stream/);
    const events = parseSse(await res.text());
    assert.equal(events[0].choices[0].delta.role, "assistant");
    assert.equal(events.filter((event) => event !== "[DONE]" && event.object === "chat.completion.chunk").map((event) => event.choices[0].delta.content ?? "").join(""), "SDK-MOCK");
    assert.ok(events.some((event) => event !== "[DONE]" && event.choices?.[0]?.finish_reason === "stop"));
    assert.equal(events.at(-1), "[DONE]");
  });
});

test("SDK matrix: unsupported advanced features reject before browser execution", async () => {
  const cases = [
    { tools: [{ type: "function", function: { name: "x" } }] },
    { tool_choice: "auto" },
    { functions: [{ name: "x" }] },
    { function_call: "auto" },
    { n: 2 },
  ];
  await withServer(async ({ baseUrl, transcript }) => {
    for (const extra of cases) {
      const res = await postChat(baseUrl, { model: "chatgpt-cdp", messages: [{ role: "user", content: "x" }], ...extra });
      const body = await res.json();
      assert.equal(res.status, 400);
      assert.equal(body.error.code, "unsupported_or_invalid_request");
    }
    assert.equal(transcript.findOrCreateCalls, 0);
    assert.equal(transcript.connects, 0);
  });
});

test("SDK matrix: invalid requests and streaming failures keep parseable error envelopes", async () => {
  await withServer(async ({ baseUrl }) => {
    const invalidJson = await postChat(baseUrl, "{");
    assert.equal(invalidJson.status, 400);
    assert.equal((await invalidJson.json()).error.code, "invalid_request");

    const missingPrompt = await postChat(baseUrl, { model: "chatgpt-cdp", messages: "bad" });
    assert.equal(missingPrompt.status, 400);
    assert.equal((await missingPrompt.json()).error.code, "unsupported_or_invalid_request");
  });

  await withServer(async ({ baseUrl }) => {
    const res = await postChat(baseUrl, { model: "chatgpt-cdp", stream: true, messages: [{ role: "user", content: "x" }] });
    const events = parseSse(await res.text());
    assert.equal(res.status, 200);
    assert.equal(events[0].error.type, "cdp_browser_router_error");
    assert.equal(events.at(-1), "[DONE]");
  }, { failConnect: true });
});