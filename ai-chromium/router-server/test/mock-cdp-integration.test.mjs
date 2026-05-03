import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

const ARTIFACTS_DIR = fs.mkdtempSync(path.join(os.tmpdir(), "router-mock-cdp-artifacts-"));
process.env.ARTIFACTS_DIR = ARTIFACTS_DIR;
delete process.env.ARTIFACT_RAW_CAPTURE;
delete process.env.RAW_CAPTURE_MODE;

await import("../src/provider/gemini-private.adapter.mjs");
await import("../src/provider/chatgpt-group.adapter.mjs");
await import("../src/provider/chatgpt-project.adapter.mjs");
await import("../src/provider/chatgpt-private.adapter.mjs");
const { handleChatCompletions } = await import("../src/api/openai-compatible.mjs");

function b64(text) {
  return Buffer.from(text, "utf8").toString("base64");
}

function assistantSsePayload(text) {
  const payload = {
    conversation_id: "conv_mock",
    v: {
      message: {
        id: "msg_mock",
        author: { role: "assistant", name: "mock-assistant" },
        channel: "final",
        status: "finished_successfully",
        end_turn: true,
        content: { content_type: "text", parts: [text] },
        metadata: {
          request_id: "req_mock",
          turn_exchange_id: "exchange_mock",
          model_slug: "mock-browser-model",
        },
      },
    },
  };
  return [
    `data: ${JSON.stringify(payload)}\n\n`,
    `data: ${JSON.stringify({ type: "message_stream_complete", conversation_id: "conv_mock" })}\n\n`,
    "data: [DONE]\n\n",
  ].join("");
}

class MockCdpSocket {
  constructor({ target, reply, transcript }) {
    this.target = target;
    this.reply = reply;
    this.transcript = transcript;
    this.listeners = new Set();
    this.closed = false;
    this.didScheduleCapture = false;
  }

  async connect() {
    this.transcript.connects += 1;
  }

  onEvent(fn) {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  }

  emit(method, params) {
    for (const fn of [...this.listeners]) fn(method, params);
  }

  scheduleCapture() {
    if (this.didScheduleCapture) return;
    this.didScheduleCapture = true;
    setTimeout(() => {
      this.emit("Network.responseReceived", {
        requestId: "mock_request_1",
        response: {
          url: "https://chatgpt.com/backend-api/conversation",
          mimeType: "text/event-stream",
          headers: { "content-type": "text/event-stream" },
        },
      });
      setTimeout(() => {
        this.emit("Network.loadingFinished", { requestId: "mock_request_1" });
      }, 2);
    }, 2);
  }

  async send(method, params = {}) {
    this.transcript.commands.push({ method, params });

    if (method === "Network.streamResourceContent") {
      return { bufferedData: b64(assistantSsePayload(this.reply)) };
    }

    if (method === "Runtime.evaluate") {
      const expression = String(params.expression ?? "");
      if (expression === "1") return { result: { value: 1 } };
      if (expression.includes("location.href")) return { result: { value: this.target.url } };
      const promptMatch = expression.match(/const prompt = ([\s\S]*?);\n/);
      if (promptMatch) {
        this.transcript.prompts.push(JSON.parse(promptMatch[1]));
        this.scheduleCapture();
        return { result: { value: { ok: true, method: "mock_button" } } };
      }
      return { result: { value: true } };
    }

    return {};
  }

  close() {
    this.closed = true;
    this.transcript.closes += 1;
  }
}

class MockResponse {
  constructor() {
    this.statusCode = null;
    this.headers = {};
    this.chunks = [];
    this.headersSent = false;
    this.destroyed = false;
    this.ended = false;
  }

  writeHead(statusCode, headers = {}) {
    this.statusCode = statusCode;
    this.headers = { ...this.headers, ...headers };
    this.headersSent = true;
  }

  write(chunk) {
    this.chunks.push(Buffer.isBuffer(chunk) ? chunk.toString("utf8") : String(chunk));
  }

  end(chunk = "") {
    if (chunk) this.write(chunk);
    this.ended = true;
  }

  text() {
    return this.chunks.join("");
  }
}

function parseSse(text) {
  return String(text).split(/\n\n/).filter(Boolean).map((frame) => {
    const line = frame.split(/\n/).find((entry) => entry.startsWith("data: "));
    const data = line?.slice(6) ?? "";
    return data === "[DONE]" ? data : JSON.parse(data);
  });
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function turnDirs() {
  const base = path.join(ARTIFACTS_DIR, "turns");
  if (!fs.existsSync(base)) return [];
  return fs.readdirSync(base).map((name) => path.join(base, name)).sort();
}

async function captureNewTurn(fn) {
  const before = new Set(turnDirs());
  const result = await fn();
  const after = turnDirs().filter((dir) => !before.has(dir));
  assert.equal(after.length, 1, `expected one new turn artifact, got ${after.length}`);
  return { ...result, turnDir: after[0] };
}

function makeContext({ reply = "MOCK-A MOCK-B" } = {}) {
  const transcript = { connects: 0, closes: 0, commands: [], prompts: [], findOrCreateCalls: 0 };
  const target = {
    id: "target_mock_1",
    url: "https://chatgpt.com/",
    webSocketDebuggerUrl: "ws://mock.invalid/devtools/page/target_mock_1",
  };
  return {
    transcript,
    ctx: {
      readBody: async (req) => req.body,
      parseJsonObject: (body) => JSON.parse(body),
      jsonResponse: (res, status, body) => {
        res.writeHead(status, { "content-type": "application/json" });
        res.end(`${JSON.stringify(body)}\n`);
      },
      errorResponse: (res, status, message, extra = {}) => {
        res.writeHead(status, { "content-type": "application/json" });
        res.end(`${JSON.stringify({ error: { message, ...extra } })}\n`);
      },
      targetManager: {
        async findOrCreate() {
          transcript.findOrCreateCalls += 1;
          return target;
        },
      },
      config: {
        idleMs: 20,
        firstCaptureMs: 2_000,
        maxMs: 5_000,
        wsEnabled: true,
        cdpHost: "mock",
        cdpPort: 0,
        uploadScript: "",
        defaultProjectId: "",
        cdpSocketFactory: (_wsUrl, context) => new MockCdpSocket({ target: context.target, reply, transcript }),
      },
    },
  };
}

test("mocked CDP non-streaming route writes OpenAI envelope and quality artifacts", async () => {
  const { ctx, transcript } = makeContext({ reply: "MOCK-A MOCK-B" });
  const request = {
    model: "chatgpt-cdp",
    stream: false,
    messages: [
      { role: "system", content: "Follow system contract." },
      { role: "developer", content: "Preserve terse output." },
      { role: "user", content: "Return mock token." },
    ],
    browser: { files: [{ path: "/tmp/private/input.txt", purpose: "assistants" }] },
  };

  const { res, turnDir } = await captureNewTurn(async () => {
    const res = new MockResponse();
    await handleChatCompletions({ body: JSON.stringify(request) }, res, ctx);
    return { res };
  });

  assert.equal(res.statusCode, 200);
  const body = JSON.parse(res.text());
  assert.equal(body.object, "chat.completion");
  assert.equal(body.choices[0].message.role, "assistant");
  assert.equal(body.choices[0].message.content, "MOCK-A MOCK-B");
  assert.equal(body.browser.target_id, "target_mock_1");

  assert.equal(transcript.findOrCreateCalls, 1);
  assert.equal(transcript.connects, 1);
  assert.equal(transcript.closes, 1);
  assert.equal(transcript.prompts.length, 1);
  assert.match(transcript.prompts[0], /System\/developer instructions/);
  assert.match(transcript.prompts[0], /\[system\] Follow system contract/);
  assert.match(transcript.prompts[0], /\[developer\] Preserve terse output/);
  assert.match(transcript.prompts[0], /Return mock token/);

  const redacted = readJson(path.join(turnDir, "request.redacted.json"));
  assert.equal(redacted.messages_redacted, true);
  assert.equal(redacted.browser.files_redacted, true);
  assert.equal("content" in redacted.messages[0], false);
  assert.equal("path" in redacted.browser.files[0], false);

  const manifest = readJson(path.join(turnDir, "manifest.json"));
  assert.equal(manifest.schema, "ai_chromium.turn_manifest.v1");
  assert.equal(manifest.target_id, "target_mock_1");

  const replay = readJson(path.join(turnDir, "replay.json"));
  assert.equal(replay.replay_match, true);
  const evaluation = readJson(path.join(turnDir, "evaluation.json"));
  assert.equal(evaluation.redaction_pass, true);
  assert.equal(evaluation.quality, 1);

  const rawPolicy = readJson(path.join(turnDir, "raw-capture.blocked.json"));
  assert.equal(rawPolicy.raw_capture_persisted, false);
  assert.equal(rawPolicy.record_count > 0, true);
});

test("mocked CDP streaming route emits ordered SSE chunks and final DONE", async () => {
  const { ctx } = makeContext({ reply: "STREAM-A STREAM-B" });
  const request = {
    model: "chatgpt-cdp",
    stream: true,
    messages: [{ role: "user", content: "Stream mock token." }],
  };

  const { res } = await captureNewTurn(async () => {
    const res = new MockResponse();
    await handleChatCompletions({ body: JSON.stringify(request) }, res, ctx);
    return { res };
  });

  assert.equal(res.statusCode, 200);
  assert.match(res.headers["content-type"], /text\/event-stream/);
  const events = parseSse(res.text());
  assert.equal(events[0].choices[0].delta.role, "assistant");
  const streamed = events
    .filter((event) => event !== "[DONE]" && event.object === "chat.completion.chunk")
    .map((event) => event.choices[0].delta.content ?? "")
    .join("");
  assert.equal(streamed, "STREAM-A STREAM-B");
  const finish = events.find((event) => event !== "[DONE]" && event.choices?.[0]?.finish_reason === "stop");
  assert.ok(finish);
  assert.equal(events.at(-1), "[DONE]");
});

test("unsupported tool calling fails before mocked browser mutation", async () => {
  const { ctx, transcript } = makeContext();
  const res = new MockResponse();
  await handleChatCompletions({ body: JSON.stringify({
    model: "chatgpt-cdp",
    messages: [{ role: "user", content: "Use a tool." }],
    tools: [{ type: "function", function: { name: "x" } }],
  }) }, res, ctx);

  assert.equal(res.statusCode, 400);
  assert.match(res.text(), /tools/);
  assert.equal(transcript.findOrCreateCalls, 0);
  assert.equal(transcript.connects, 0);
  assert.equal(transcript.prompts.length, 0);
});