#!/usr/bin/env node
// CDP Browser LLM Router
//
// Exposes an OpenAI-compatible HTTP API and drives an already-running Chrome
// instance through the Chrome DevTools Protocol. It uses
// Network.streamResourceContent for HTTP streaming responses and optionally
// observes WebSocket frames as a fallback.
//
// Start Chrome separately, for example:
//   chromium --remote-debugging-port=9221 --user-data-dir=/tmp/chrome-cdp-llm
//
// Then run:
//   CDP_PORT=9221 HTTP_PORT=8081 node server/cdp-browser-llm-router.mjs

import fs from "node:fs";
import http from "node:http";
import path from "node:path";
import { URL } from "node:url";
import { CdpSocket } from "./cdp-router/cdp-socket.mjs";
import {
  collectEmbeddedJsonCandidates,
  makeSchemaDumpWriter,
  tryParseEmbeddedJson,
} from "./cdp-router/schema-derivation.mjs";
import {
  extractInnerTextEntries,
  extractPayloadJsonDumpFields,
  makeSchemaGuidedTextExtractor,
  makeTextAccumulator,
  parseSseFrames,
} from "./cdp-router/text-extraction.mjs";

const HTTP_HOST = process.env.HTTP_HOST ?? "127.0.0.1";
const HTTP_PORT = Number(process.env.HTTP_PORT ?? 8081);
const CDP_HOST = process.env.CDP_HOST ?? "127.0.0.1";
const CDP_PORT = Number(process.env.CDP_PORT ?? 9221);
const DEFAULT_IDLE_MS = Number(process.env.TURN_IDLE_MS ?? 2500);
const DEFAULT_FIRST_CAPTURE_MS = Number(process.env.TURN_FIRST_CAPTURE_MS ?? 45000);
const DEFAULT_MAX_MS = Number(process.env.TURN_MAX_MS ?? 120000);
const MAX_BODY_BYTES = Number(process.env.MAX_BODY_BYTES ?? 1_000_000);
const ENABLE_WS_FALLBACK = String(process.env.CDP_WS_FALLBACK ?? "1") !== "0";
const STREAM_DUMP_ENABLED = String(process.env.CDP_STREAM_DUMP ?? "1") !== "0";
const STREAM_DUMP_DIR = process.env.CDP_STREAM_DUMP_DIR ?? path.resolve(process.cwd(), "cdp-stream-dumps");
const STREAM_DUMP_MAX_TEXT_BYTES = Number(process.env.CDP_STREAM_DUMP_MAX_TEXT_BYTES ?? 0);
const CDP_METHOD_LOG_ENABLED = String(process.env.CDP_METHOD_LOG ?? "1") !== "0";
const CDP_METHOD_LOG_MAX_PAYLOAD_BYTES = Number(process.env.CDP_METHOD_LOG_MAX_PAYLOAD_BYTES ?? 32768);
const INNER_TEXT_DUMP_ENABLED = String(process.env.CDP_INNER_TEXT_DUMP ?? "1") !== "0";
const INNER_TEXT_DUMP_DIR = process.env.CDP_INNER_TEXT_DUMP_DIR ?? path.resolve(process.cwd(), "cdp-inner-text-dumps");
const INNER_TEXT_MAX_STRING_BYTES = Number(process.env.CDP_INNER_TEXT_MAX_STRING_BYTES ?? 4096);
const INNER_TEXT_MAX_ENTRIES = Number(process.env.CDP_INNER_TEXT_MAX_ENTRIES ?? 5000);
const SCHEMA_DUMP_ENABLED = String(process.env.CDP_SCHEMA_DUMP ?? "1") !== "0";
const SCHEMA_DUMP_DIR = process.env.CDP_SCHEMA_DUMP_DIR ?? path.resolve(process.cwd(), "cdp-schema-dumps");
const SCHEMA_DUMP_SAMPLE_LIMIT = Number(process.env.CDP_SCHEMA_DUMP_SAMPLE_LIMIT ?? 3);
const SCHEMA_DUMP_MAX_DEPTH = Number(process.env.CDP_SCHEMA_DUMP_MAX_DEPTH ?? 8);

const CHATGPT_URL = process.env.CHATGPT_URL ?? "https://chatgpt.com/";
const GEMINI_URL = process.env.GEMINI_URL ?? "https://gemini.google.com/app";

let nextCompletionId = 1;

function nowUnix() {
  return Math.floor(Date.now() / 1000);
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function jsonResponse(res, status, value) {
  const body = JSON.stringify(value, null, 2);
  res.writeHead(status, {
    "content-type": "application/json; charset=utf-8",
    "content-length": Buffer.byteLength(body),
    "access-control-allow-origin": "*",
  });
  res.end(body);
}

function errorResponse(res, status, message, extra = {}) {
  jsonResponse(res, status, {
    error: {
      message,
      type: extra.type ?? "cdp_browser_router_error",
      code: extra.code ?? null,
      ...extra,
    },
  });
}

function sendCorsOptions(res) {
  res.writeHead(204, {
    "access-control-allow-origin": "*",
    "access-control-allow-methods": "GET,POST,OPTIONS",
    "access-control-allow-headers": "authorization,content-type",
    "access-control-max-age": "600",
  });
  res.end();
}

function readBody(req) {
  return new Promise((resolve, reject) => {
    let size = 0;
    const chunks = [];
    req.on("data", (chunk) => {
      size += chunk.length;
      if (size > MAX_BODY_BYTES) {
        reject(new Error(`request body too large; max=${MAX_BODY_BYTES}`));
        req.destroy();
        return;
      }
      chunks.push(chunk);
    });
    req.on("end", () => resolve(Buffer.concat(chunks).toString("utf8")));
    req.on("error", reject);
  });
}

function parseJsonObject(text) {
  if (!text.trim()) return {};
  const value = JSON.parse(text);
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("expected JSON object");
  }
  return value;
}

function normalizeMessageContent(content) {
  if (typeof content === "string") return content;
  if (Array.isArray(content)) {
    return content
      .map((part) => {
        if (typeof part === "string") return part;
        if (typeof part?.text === "string") return part.text;
        if (typeof part?.content === "string") return part.content;
        return "";
      })
      .filter(Boolean)
      .join("\n");
  }
  if (content == null) return "";
  return String(content);
}

function messagesToPrompt(messages) {
  if (!Array.isArray(messages)) return "";
  return messages
    .map((msg) => {
      const role = typeof msg?.role === "string" ? msg.role : "user";
      const content = normalizeMessageContent(msg?.content);
      return `${role.toUpperCase()}: ${content}`;
    })
    .filter((line) => line.trim().length > 0)
    .join("\n\n");
}

function modelToBrowserUrl(model, request) {
  const explicit = request.browser?.url ?? request.browser_url;
  if (typeof explicit === "string" && explicit.startsWith("http")) return explicit;
  if (String(model ?? "").toLowerCase().includes("gemini")) return GEMINI_URL;
  return CHATGPT_URL;
}

function modelMatchesUrl(model, url) {
  const name = String(model ?? "").toLowerCase();
  if (name.includes("gemini")) return String(url ?? "").includes("gemini.google.com");
  return String(url ?? "").includes("chatgpt.com") || String(url ?? "").includes("chat.openai.com");
}

function makeOpenAiChunk({ id, model, content, finishReason = null }) {
  return {
    id,
    object: "chat.completion.chunk",
    created: nowUnix(),
    model,
    choices: [
      {
        index: 0,
        delta: content == null ? {} : { content },
        finish_reason: finishReason,
      },
    ],
  };
}

function setupSse(res) {
  res.writeHead(200, {
    "content-type": "text/event-stream; charset=utf-8",
    "cache-control": "no-cache, no-transform",
    connection: "keep-alive",
    "x-accel-buffering": "no",
    "access-control-allow-origin": "*",
  });
}

function sseWrite(res, value) {
  res.write(`data: ${JSON.stringify(value)}\n\n`);
}

function httpJson(method, path) {
  return new Promise((resolve, reject) => {
    const req = http.request(
      { host: CDP_HOST, port: CDP_PORT, method, path },
      (res) => {
        const chunks = [];
        res.on("data", (chunk) => chunks.push(chunk));
        res.on("end", () => {
          const text = Buffer.concat(chunks).toString("utf8");
          if (res.statusCode < 200 || res.statusCode >= 300) {
            reject(new Error(`CDP HTTP ${method} ${path} failed: ${res.statusCode} ${text.slice(0, 200)}`));
            return;
          }
          try {
            resolve(JSON.parse(text));
          } catch (err) {
            reject(new Error(`CDP HTTP ${method} ${path} returned non-json: ${err.message}`));
          }
        });
      },
    );
    req.on("error", reject);
    req.end();
  });
}

function safeFileComponent(value) {
  return String(value ?? "unknown").replace(/[^A-Za-z0-9_.-]+/g, "-").replace(/^-+|-+$/g, "") || "unknown";
}

function truncateDumpText(text) {
  const originalBytes = Buffer.byteLength(text, "utf8");
  if (STREAM_DUMP_MAX_TEXT_BYTES > 0 && originalBytes > STREAM_DUMP_MAX_TEXT_BYTES) {
    return {
      text: Buffer.from(text, "utf8").subarray(0, STREAM_DUMP_MAX_TEXT_BYTES).toString("utf8"),
      text_bytes: originalBytes,
      text_truncated: true,
    };
  }
  return { text, text_bytes: originalBytes, text_truncated: false };
}

function safeJsonStringify(value) {
  const seen = new WeakSet();
  return JSON.stringify(value, (_key, current) => {
    if (typeof current === "bigint") return current.toString();
    if (current && typeof current === "object") {
      if (seen.has(current)) return "[Circular]";
      seen.add(current);
    }
    return current;
  });
}

function truncateDumpJson(value) {
  let json;
  try {
    json = safeJsonStringify(value);
  } catch (err) {
    json = JSON.stringify({ stringify_error: String(err?.message ?? err) });
  }
  if (typeof json !== "string") json = "null";
  const originalBytes = Buffer.byteLength(json, "utf8");
  if (CDP_METHOD_LOG_MAX_PAYLOAD_BYTES > 0 && originalBytes > CDP_METHOD_LOG_MAX_PAYLOAD_BYTES) {
    return {
      payload_json: Buffer.from(json, "utf8").subarray(0, CDP_METHOD_LOG_MAX_PAYLOAD_BYTES).toString("utf8"),
      payload_json_bytes: originalBytes,
      payload_json_truncated: true,
    };
  }
  return { payload_json: json, payload_json_bytes: originalBytes, payload_json_truncated: false };
}

function makeNoopStreamDumpWriter(error = null) {
  return {
    path: null,
    write() {},
    writeText() {},
    writeJson() {},
    writeCdpMethod() {},
    stats() { return { path: null, entries: 0, bytes: 0, error }; },
    async close() { return this.stats(); },
  };
}

function makeStreamDumpWriter({ completionId, model, target }) {
  if (!STREAM_DUMP_ENABLED) return makeNoopStreamDumpWriter("disabled");
  try {
    fs.mkdirSync(STREAM_DUMP_DIR, { recursive: true });
    const schema = makeSchemaDumpWriter({
      completionId,
      model,
      target,
      enabled: SCHEMA_DUMP_ENABLED,
      dirPath: SCHEMA_DUMP_DIR,
      sampleLimit: SCHEMA_DUMP_SAMPLE_LIMIT,
      maxDepth: SCHEMA_DUMP_MAX_DEPTH,
      safeFileComponent,
    });
    const stamp = new Date().toISOString().replace(/[:.]/g, "-");
    const filePath = path.join(
      STREAM_DUMP_DIR,
      `${stamp}-${safeFileComponent(completionId)}-${safeFileComponent(model)}.ndjson`,
    );
    const stream = fs.createWriteStream(filePath, { flags: "a" });
    let entries = 0;
    let bytes = 0;
    let closed = false;
    let writeError = null;

    stream.on("error", (err) => {
      writeError = String(err?.message ?? err);
    });

    function write(event, fields = {}) {
      if (closed) return;
      entries += 1;
      const record = {
        ts: new Date().toISOString(),
        seq: entries,
        completion_id: completionId,
        model,
        target_id: target?.id ?? null,
        target_url: target?.url ?? null,
        event,
        ...fields,
      };
      schema.observe(record);
      if (Array.isArray(record.payload_items)) {
        record.payload_items.forEach((item, index) => {
          if (!item || typeof item !== "object" || !item.payload_json || typeof item.payload_json !== "object") return;
          const suffixParts = [
            item.event_name ? `event_${item.event_name}` : null,
            item.type ? `type_${item.type}` : null,
            item.op ? `op_${item.op}` : null,
          ].filter(Boolean);
          schema.observe({
            ...record,
            event: "raw_payload_json",
            schema_key_suffix: suffixParts.length > 0 ? suffixParts.join("__") : "unclassified",
            payload_index: index,
            payload_event_name: item.event_name ?? null,
            payload_type: item.type ?? null,
            payload_op: item.op ?? null,
            payload_json: item.payload_json,
            payload_shape: item.shape ?? null,
          });

          const embedded = collectEmbeddedJsonCandidates(item.payload_json);
          embedded.forEach((candidate) => {
            schema.observe({
              ...record,
              event: "raw_payload_embedded_json",
              schema_key_suffix: safeFileComponent(candidate.path),
              payload_index: index,
              payload_event_name: item.event_name ?? null,
              payload_type: item.type ?? null,
              payload_op: item.op ?? null,
              embedded_json_path: candidate.path,
              embedded_json: candidate.parsed,
            });
          });
        });
      }
      const line = `${JSON.stringify(record)}\n`;
      bytes += Buffer.byteLength(line, "utf8");
      stream.write(line);
    }

    function writeText(event, fields = {}, text = "") {
      const rawText = String(text ?? "");
      write(event, { ...fields, ...truncateDumpText(rawText), ...extractPayloadJsonDumpFields(rawText) });
    }

    function writeJson(event, fields = {}, value = null) {
      write(event, { ...fields, ...truncateDumpJson(value) });
    }

    function writeCdpMethod(event, method, payload = null) {
      if (!CDP_METHOD_LOG_ENABLED) return;
      writeJson(event, { cdp_method: method }, payload);
    }

    return {
      path: filePath,
      write,
      writeText,
      writeJson,
      writeCdpMethod,
      stats() { return { path: filePath, entries, bytes, error: writeError, schema: schema.stats() }; },
      close() {
        if (closed) return Promise.resolve(this.stats());
        closed = true;
        return new Promise((resolve) => {
          stream.end(() => {
            schema.close();
            resolve(this.stats());
          });
        });
      },
    };
  } catch (err) {
    return makeNoopStreamDumpWriter(String(err?.message ?? err));
  }
}

function truncateInnerText(text) {
  const value = String(text ?? "");
  const originalBytes = Buffer.byteLength(value, "utf8");
  if (INNER_TEXT_MAX_STRING_BYTES > 0 && originalBytes > INNER_TEXT_MAX_STRING_BYTES) {
    return {
      text: Buffer.from(value, "utf8").subarray(0, INNER_TEXT_MAX_STRING_BYTES).toString("utf8"),
      text_bytes: originalBytes,
      text_truncated: true,
    };
  }
  return { text: value, text_bytes: originalBytes, text_truncated: false };
}

function makeNoopInnerTextDumpWriter(error = null) {
  return {
    path: null,
    writeFromRaw() {},
    stats() { return { path: null, entries: 0, text_candidates: 0, payload_json_entries: 0, bytes: 0, error }; },
    close() { return this.stats(); },
  };
}

function makeInnerTextDumpWriter({ completionId, model, target }) {
  if (!INNER_TEXT_DUMP_ENABLED) return makeNoopInnerTextDumpWriter("disabled");
  try {
    fs.mkdirSync(INNER_TEXT_DUMP_DIR, { recursive: true });
    const stamp = new Date().toISOString().replace(/[:.]/g, "-");
    const filePath = path.join(
      INNER_TEXT_DUMP_DIR,
      `${stamp}-${safeFileComponent(completionId)}-${safeFileComponent(model)}.json`,
    );
    const entries = [];
    let textCandidates = 0;
    let payloadJsonEntries = 0;
    let writeError = null;
    let closed = false;

    function writeFromRaw(source = {}, raw = "") {
      if (closed || entries.length >= INNER_TEXT_MAX_ENTRIES) return;
      const extracted = extractInnerTextEntries(String(raw ?? ""), source, {
        maxStringBytes: INNER_TEXT_MAX_STRING_BYTES,
      });
      for (const entry of extracted) {
        if (entries.length >= INNER_TEXT_MAX_ENTRIES) break;
        entries.push({
          seq: entries.length + 1,
          ts: new Date().toISOString(),
          completion_id: completionId,
          model,
          target_id: target?.id ?? null,
          target_url: target?.url ?? null,
          ...entry,
        });
        if (entry.kind === "text_candidate") textCandidates += 1;
        if (entry.kind === "payload_json") payloadJsonEntries += 1;
      }
    }

    function buildDocument() {
      return {
        meta: {
          completion_id: completionId,
          model,
          target_id: target?.id ?? null,
          target_url: target?.url ?? null,
          created_at: new Date().toISOString(),
          source: "cdp-browser-llm-router",
          schema: "cdp.inner_text_dump.v1",
        },
        stats: {
          entries: entries.length,
          text_candidates: textCandidates,
          payload_json_entries: payloadJsonEntries,
          max_entries: INNER_TEXT_MAX_ENTRIES,
          max_string_bytes: INNER_TEXT_MAX_STRING_BYTES,
          truncated_by_entry_limit: entries.length >= INNER_TEXT_MAX_ENTRIES,
        },
        entries,
      };
    }

    return {
      path: filePath,
      writeFromRaw,
      stats() {
        let bytes = 0;
        try {
          bytes = fs.existsSync(filePath) ? fs.statSync(filePath).size : 0;
        } catch {}
        return {
          path: filePath,
          entries: entries.length,
          text_candidates: textCandidates,
          payload_json_entries: payloadJsonEntries,
          bytes,
          error: writeError,
        };
      },
      close() {
        if (closed) return this.stats();
        closed = true;
        try {
          fs.writeFileSync(filePath, `${JSON.stringify(buildDocument(), null, 2)}\n`);
        } catch (err) {
          writeError = String(err?.message ?? err);
        }
        return this.stats();
      },
    };
  } catch (err) {
    return makeNoopInnerTextDumpWriter(String(err?.message ?? err));
  }
}

async function closeDumpWriter(writer, fallback = null) {
  if (!writer || typeof writer.close !== "function") {
    return typeof writer?.stats === "function" ? writer.stats() : fallback;
  }
  try {
    const result = writer.close();
    return result && typeof result.then === "function" ? await result : result;
  } catch {
    return typeof writer.stats === "function" ? writer.stats() : fallback;
  }
}

async function closeDumpWriterQuietly(writer) {
  try {
    await closeDumpWriter(writer, null);
  } catch {}
}

async function cdpListTargets() {
  return await httpJson("GET", "/json/list");
}

async function cdpVersion() {
  return await httpJson("GET", "/json/version");
}

async function cdpNewTarget(url) {
  const path = `/json/new?${encodeURIComponent(url)}`;
  try {
    return await httpJson("PUT", path);
  } catch {
    return await httpJson("GET", path);
  }
}

async function cdpActivateTarget(id) {
  try {
    await httpJson("GET", `/json/activate/${encodeURIComponent(id)}`);
  } catch {}
}

function targetUrl(target) {
  return target?.url ?? target?.description ?? "";
}

async function ensureTarget(model, request) {
  const url = modelToBrowserUrl(model, request);
  const reset = request.browser?.reset_chat ?? request.reset_chat ?? false;
  const targets = await cdpListTargets();
  let target = null;
  if (!reset) {
    target = targets.find((t) => t.type === "page" && modelMatchesUrl(model, targetUrl(t)) && t.webSocketDebuggerUrl);
  }
  if (!target) target = await cdpNewTarget(url);
  if (target.id) await cdpActivateTarget(target.id);
  return target;
}

function shouldStreamResponse(params) {
  const url = params?.response?.url ?? "";
  const mime = String(params?.response?.mimeType ?? "").toLowerCase();
  const headers = params?.response?.headers ?? {};
  const contentType = String(headers["content-type"] ?? headers["Content-Type"] ?? "").toLowerCase();
  if (url.includes("/backend-api/f/conversation")) return true;
  if (url.includes("/backend-api/conversation")) return true;
  if (url.includes("/backend-api/sentinel/chat-requirements")) return false;
  if (contentType.includes("text/event-stream")) return true;
  if (mime.includes("event-stream")) return true;
  return false;
}

function shouldStreamRequest(params) {
  const url = params?.request?.url ?? "";
  if (url.includes("/backend-api/f/conversation")) return true;
  if (url.includes("/backend-api/conversation")) return true;
  return false;
}

function decodeBase64Text(data) {
  if (typeof data !== "string" || data.length === 0) return "";
  try {
    return Buffer.from(data, "base64").toString("utf8");
  } catch {
    return "";
  }
}

function buildSubmitExpression(prompt, { model }) {
  const promptJson = JSON.stringify(prompt);
  const isGemini = String(model ?? "").toLowerCase().includes("gemini");
  return `
(async () => {
  const prompt = ${promptJson};
  const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
  const deadline = Date.now() + 30000;
  const findEditor = () => {
    if (${isGemini ? "true" : "false"}) {
      return document.querySelector('div[contenteditable="true"]') ||
             document.querySelector('rich-textarea div[contenteditable="true"]') ||
             document.querySelector('textarea');
    }
    return document.querySelector('div[contenteditable="true"]') ||
           document.querySelector('textarea');
  };
  while (!findEditor() && Date.now() < deadline) await sleep(250);
  const editor = findEditor();
  if (!editor) return { ok: false, error: "editor_not_found" };
  editor.focus();
  try {
    document.execCommand('selectAll', false, null);
    document.execCommand('insertText', false, prompt);
  } catch {}
  if ((editor.textContent || editor.value || '').trim().length === 0) {
    if ('value' in editor) editor.value = prompt;
    else editor.textContent = prompt;
  }
  editor.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertText', data: prompt }));
  editor.dispatchEvent(new Event('change', { bubbles: true }));
  await sleep(300);
  const findSend = () => {
    const direct = document.querySelector('button[data-testid="send-button"]') ||
      document.querySelector('button[aria-label="Send prompt"]') ||
      document.querySelector('button[aria-label="Send message"]') ||
      document.querySelector('button[aria-label="Submit"]');
    if (direct && !direct.disabled) return direct;
    const buttons = Array.from(document.querySelectorAll('button'));
    return buttons.find((b) => !b.disabled && /send|submit/i.test(b.getAttribute('aria-label') || b.textContent || '')) || null;
  };
  while (!findSend() && Date.now() < deadline) await sleep(100);
  const send = findSend();
  if (send && !send.disabled) {
    send.click();
    return { ok: true, method: "button" };
  }
  editor.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', code: 'Enter', which: 13, keyCode: 13, bubbles: true, cancelable: true }));
  return { ok: true, method: "enter_fallback" };
})()
`;
}

async function waitForPageReady(cdp, model) {
  const expression = `
(async () => {
  const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
  const deadline = Date.now() + 45000;
  const isGemini = ${String(model ?? "").toLowerCase().includes("gemini") ? "true" : "false"};
  const findEditor = () => isGemini
    ? (document.querySelector('div[contenteditable="true"]') || document.querySelector('rich-textarea div[contenteditable="true"]') || document.querySelector('textarea'))
    : (document.querySelector('div[contenteditable="true"]') || document.querySelector('textarea'));
  while (!findEditor() && Date.now() < deadline) await sleep(250);
  return Boolean(findEditor());
})()
`;
  const result = await cdp.send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
  return Boolean(result?.result?.value);
}

async function runCdpTurn({ request, stream, res }) {
  const model = request.model ?? "chatgpt-cdp";
  const prompt = typeof request.prompt === "string" ? request.prompt : messagesToPrompt(request.messages);
  if (!prompt.trim()) throw new Error("empty prompt; provide prompt or messages[]");

  const idleMs = Number(request.browser?.idle_ms ?? request.idle_ms ?? DEFAULT_IDLE_MS);
  const firstCaptureMs = Number(request.browser?.first_capture_ms ?? request.first_capture_ms ?? DEFAULT_FIRST_CAPTURE_MS);
  const maxMs = Number(request.browser?.max_ms ?? request.max_ms ?? DEFAULT_MAX_MS);
  const reset = request.browser?.reset_chat ?? request.reset_chat ?? false;
  const target = await ensureTarget(model, request);
  if (!target?.webSocketDebuggerUrl) throw new Error("CDP target has no webSocketDebuggerUrl");

  const cdp = new CdpSocket(target.webSocketDebuggerUrl);
  await cdp.connect();

  const completionId = `chatcmpl-cdp-${nextCompletionId++}`;
  const dump = makeStreamDumpWriter({ completionId, model, target });
  const innerDump = makeInnerTextDumpWriter({ completionId, model, target });
  const capturedRequests = new Set();
  const bodyFallbackRequests = new Set();
  const requestsWithData = new Set();
  const activeStreamRequests = new Set();
  const accumulator = makeTextAccumulator();
  const textExtractor = makeSchemaGuidedTextExtractor();
  const rawStats = {
    cdp_commands: 0,
    cdp_command_results: 0,
    cdp_command_errors: 0,
    cdp_events: 0,
    stream_resource_attempts: 0,
    stream_resource_hits: 0,
    stream_resource_stale: 0,
    stream_resource_errors: 0,
    request_candidates: 0,
    response_candidates: 0,
    loading_finished_bodies: 0,
    network_chunks: 0,
    websocket_frames: 0,
    websocket_frames_seen: 0,
    event_source_messages: 0,
  };

  const rawCdpSend = cdp.send.bind(cdp);
  cdp.send = (method, params = {}) => {
    rawStats.cdp_commands += 1;
    dump.writeCdpMethod("cdp_command_sent", method, params);
    return rawCdpSend(method, params).then(
      (result) => {
        rawStats.cdp_command_results += 1;
        dump.writeCdpMethod("cdp_command_result", method, result);
        return result;
      },
      (err) => {
        rawStats.cdp_command_errors += 1;
        dump.write("cdp_command_error", {
          cdp_method: method,
          error: String(err?.message ?? err),
        });
        throw err;
      },
    );
  };

  let finish;
  let fail;
  let idleTimer = null;
  let firstCaptureTimer = null;
  let maxTimer = null;
  let done = false;
  const finalPromise = new Promise((resolve, reject) => { finish = resolve; fail = reject; });
  finalPromise.catch(() => {});

  function armIdle() {
    if (idleTimer) clearTimeout(idleTimer);
    idleTimer = setTimeout(() => {
      if (activeStreamRequests.size > 0) {
        dump.write("idle_deferred_active_streams", {
          active_stream_requests: activeStreamRequests.size,
        });
        armIdle();
        return;
      }
      complete("stop");
    }, idleMs);
  }

  function clearFirstCaptureTimer() {
    if (firstCaptureTimer) clearTimeout(firstCaptureTimer);
    firstCaptureTimer = null;
  }

  function emitCandidate(candidate) {
    const delta = accumulator.append(candidate);
    if (!delta) return;
    clearFirstCaptureTimer();
    if (stream && !res.destroyed) sseWrite(res, makeOpenAiChunk({ id: completionId, model, content: delta }));
    armIdle();
  }

  const doneSignals = new Set();
  const embeddedJsonBuffers = new Map();

  function maybeReconstructEmbeddedJson(text, source = {}) {
    const requestId = source.request_id;
    if (!requestId) return;
    const frames = parseSseFrames(text);
    for (const frame of frames) {
      if (!frame?.data || frame.data === "[DONE]") continue;
      let payload;
      try {
        payload = JSON.parse(frame.data);
      } catch {
        continue;
      }
      if (!payload || typeof payload !== "object") continue;
      if (payload.o !== "append" || typeof payload.p !== "string" || typeof payload.v !== "string") continue;
      const key = `${requestId}::${payload.p}`;
      const next = (embeddedJsonBuffers.get(key) ?? "") + payload.v;
      if (next.length > 1_000_000) {
        embeddedJsonBuffers.delete(key);
        continue;
      }
      embeddedJsonBuffers.set(key, next);
      const parsed = tryParseEmbeddedJson(next);
      if (!parsed) continue;
      dump.write("raw_payload_embedded_json_reconstructed", {
        ...source,
        patch_path: payload.p,
        embedded_json: parsed,
      });
      embeddedJsonBuffers.delete(key);
    }
  }

  function processNetworkText(text, source = {}) {
    dump.writeText("parse_input", source, text);
    maybeReconstructEmbeddedJson(text, source);
    innerDump.writeFromRaw(source, text);
    const parsed = textExtractor.extract(text);
    if (parsed.activity) {
      clearFirstCaptureTimer();
      armIdle();
      if (parsed.rules?.length) {
        dump.write("schema_extraction_rules", {
          ...source,
          rule_count: parsed.rules.length,
          rules: parsed.rules,
        });
      }
    }
    if (parsed.text) {
      dump.writeText("parse_output", source, parsed.text);
      emitCandidate(parsed.text);
    }
    if (parsed.done) {
      if (source.request_id) doneSignals.add(source.request_id);
      dump.write("parse_done", {
        ...source,
        done_signal_count: doneSignals.size,
      });
      // Do not complete immediately on [DONE]; we can receive additional
      // chunks/events from the same turn. Let idle/max timers terminate.
      clearFirstCaptureTimer();
      armIdle();
    }
  }

  function complete(reason = "stop") {
    if (done) return;
    done = true;
    if (idleTimer) clearTimeout(idleTimer);
    clearFirstCaptureTimer();
    if (maxTimer) clearTimeout(maxTimer);
    if (stream && !res.destroyed) {
      sseWrite(res, makeOpenAiChunk({ id: completionId, model, content: null, finishReason: reason }));
      res.write("data: [DONE]\n\n");
      res.end();
    }
    finish({ reason });
  }

  function startStreamResourceContent(requestId) {
    if (!requestId || capturedRequests.has(requestId) || done) return;
    capturedRequests.add(requestId);
    rawStats.stream_resource_attempts += 1;
    cdp.send("Network.streamResourceContent", { requestId })
      .then((result) => {
        if (done) return;
        if (result?.__stale_stream_resource) {
          rawStats.stream_resource_stale += 1;
          dump.write("stream_resource_stale", { request_id: requestId });
          capturedRequests.delete(requestId);
          bodyFallbackRequests.add(requestId);
          return;
        }
        rawStats.stream_resource_hits += 1;
        const buffered = decodeBase64Text(result?.bufferedData);
        dump.write("stream_resource_started", {
          request_id: requestId,
          buffered_data_bytes: Buffer.byteLength(buffered, "utf8"),
        });
        if (buffered) processNetworkText(buffered, {
          source: "Network.streamResourceContent.bufferedData",
          request_id: requestId,
        });
      })
      .catch((err) => {
        if (done) return;
        rawStats.stream_resource_errors += 1;
        capturedRequests.delete(requestId);
        bodyFallbackRequests.add(requestId);
        const message = String(err?.message ?? err);
        dump.write("stream_resource_error", { request_id: requestId, error: message });
        if (!message.includes("already finished loading")) fail(err);
      });
  }

  cdp.onEvent(async (method, params) => {
    try {
      rawStats.cdp_events += 1;
      dump.writeCdpMethod("cdp_event", method, params);
      if (done) return;
      if (method === "Network.requestWillBeSent" && shouldStreamRequest(params)) {
        if (params.requestId) activeStreamRequests.add(params.requestId);
        rawStats.request_candidates += 1;
        dump.write("request_candidate", {
          request_id: params.requestId,
          url: params.request?.url ?? null,
          method: params.request?.method ?? null,
          resource_type: params.type ?? null,
        });
        startStreamResourceContent(params.requestId);
        return;
      }
      if (method === "Network.responseReceived" && shouldStreamResponse(params)) {
        if (params.requestId) activeStreamRequests.add(params.requestId);
        rawStats.response_candidates += 1;
        dump.write("response_candidate", {
          request_id: params.requestId,
          url: params.response?.url ?? null,
          status: params.response?.status ?? null,
          mime_type: params.response?.mimeType ?? null,
          resource_type: params.type ?? null,
          content_type: params.response?.headers?.["content-type"] ?? params.response?.headers?.["Content-Type"] ?? null,
        });
        startStreamResourceContent(params.requestId);
        return;
      }
      if (method === "Network.dataReceived" && capturedRequests.has(params.requestId)) {
        rawStats.network_chunks += 1;
        requestsWithData.add(params.requestId);
        clearFirstCaptureTimer();
        armIdle();
        const text = decodeBase64Text(params.data);
        if (text) processNetworkText(text, {
          source: "Network.dataReceived",
          request_id: params.requestId,
          data_length: params.dataLength ?? null,
          encoded_data_length: params.encodedDataLength ?? null,
        });
        return;
      }
      if (method === "Network.loadingFinished" && (bodyFallbackRequests.has(params.requestId) || (capturedRequests.has(params.requestId) && !requestsWithData.has(params.requestId)))) {
        const body = await cdp.send("Network.getResponseBody", { requestId: params.requestId }).catch(() => null);
        const text = body?.base64Encoded ? decodeBase64Text(body.body) : (typeof body?.body === "string" ? body.body : "");
        if (text) {
          rawStats.loading_finished_bodies += 1;
          processNetworkText(text, {
            source: "Network.getResponseBody",
            request_id: params.requestId,
            base64_encoded: Boolean(body?.base64Encoded),
          });
        }
        capturedRequests.delete(params.requestId);
        bodyFallbackRequests.delete(params.requestId);
        requestsWithData.delete(params.requestId);
        activeStreamRequests.delete(params.requestId);
        for (const key of [...embeddedJsonBuffers.keys()]) {
          if (key.startsWith(`${params.requestId}::`)) embeddedJsonBuffers.delete(key);
        }
        armIdle();
        return;
      }
      if (method === "Network.loadingFinished") {
        activeStreamRequests.delete(params.requestId);
        for (const key of [...embeddedJsonBuffers.keys()]) {
          if (key.startsWith(`${params.requestId}::`)) embeddedJsonBuffers.delete(key);
        }
        armIdle();
        return;
      }
      if (method === "Network.loadingFailed") {
        activeStreamRequests.delete(params.requestId);
        for (const key of [...embeddedJsonBuffers.keys()]) {
          if (key.startsWith(`${params.requestId}::`)) embeddedJsonBuffers.delete(key);
        }
        armIdle();
        return;
      }
      if (method === "Network.eventSourceMessageReceived") {
        rawStats.event_source_messages += 1;
        clearFirstCaptureTimer();
        armIdle();
        processNetworkText(`data: ${params.data}\n\n`, {
          source: "Network.eventSourceMessageReceived",
          request_id: params.requestId ?? null,
          event_name: params.eventName ?? null,
        });
        return;
      }
      if (ENABLE_WS_FALLBACK && method === "Network.webSocketFrameReceived") {
        const payload = params?.response?.payloadData;
        if (typeof payload === "string") {
          rawStats.websocket_frames_seen += 1;
          clearFirstCaptureTimer();
          armIdle();
        }
        if (typeof payload === "string" && (payload.includes("calpico-message-add") || payload.includes('"raw_messages"') || payload.includes('"content_type":"text"'))) {
          rawStats.websocket_frames += 1;
          processNetworkText(payload, {
            source: "Network.webSocketFrameReceived",
            request_id: params.requestId ?? null,
          });
        }
      }
    } catch (err) {
      if (!done) fail(err);
    }
  });

  try {
    await cdp.send("Page.enable");
    await cdp.send("Runtime.enable");
    await cdp.send("Network.enable", { maxTotalBufferSize: 100_000_000, maxResourceBufferSize: 50_000_000 });
    await cdp.send("Network.setCacheDisabled", { cacheDisabled: true }).catch(() => {});

    if (reset) {
      const url = modelToBrowserUrl(model, request);
      await cdp.send("Page.navigate", { url });
      await sleep(1500);
    }

    const ready = await waitForPageReady(cdp, model);
    if (!ready) throw new Error("page editor not found; ensure browser is logged in and target page is loaded");

    if (stream) {
      setupSse(res);
      sseWrite(res, makeOpenAiChunk({ id: completionId, model, content: "" }));
    }

    maxTimer = setTimeout(() => complete("length"), maxMs);
    firstCaptureTimer = setTimeout(() => complete("stop"), firstCaptureMs);

    const submit = await cdp.send("Runtime.evaluate", {
      expression: buildSubmitExpression(prompt, { model }),
      awaitPromise: true,
      returnByValue: true,
    });
    const submitValue = submit?.result?.value;
    if (!submitValue?.ok) throw new Error(`prompt submit failed: ${JSON.stringify(submitValue)}`);

    await finalPromise;
    const innerTextDumpStats = await closeDumpWriter(innerDump, innerDump.stats());
    const streamDumpStats = await closeDumpWriter(dump, dump.stats());
    return {
      id: completionId,
      model,
      target_id: target.id,
      target_url: target.url,
      content: accumulator.value(),
      raw_stats: rawStats,
      stream_dump: streamDumpStats,
      inner_text_dump: innerTextDumpStats,
      finish_reason: "stop",
    };
  } finally {
    if (idleTimer) clearTimeout(idleTimer);
    if (maxTimer) clearTimeout(maxTimer);
    await closeDumpWriterQuietly(innerDump);
    await closeDumpWriterQuietly(dump);
    cdp.close();
  }
}

async function handleChatCompletions(req, res) {
  let body;
  try {
    body = parseJsonObject(await readBody(req));
  } catch (err) {
    errorResponse(res, 400, err.message, { code: "invalid_request" });
    return;
  }

  const stream = Boolean(body.stream);
  try {
    const result = await runCdpTurn({ request: body, stream, res });
    if (stream) return;
    jsonResponse(res, 200, {
      id: result.id,
      object: "chat.completion",
      created: nowUnix(),
      model: result.model,
      choices: [
        {
          index: 0,
          message: { role: "assistant", content: result.content },
          finish_reason: result.finish_reason ?? "stop",
        },
      ],
      browser: {
        backend: "cdp",
        cdp: `http://${CDP_HOST}:${CDP_PORT}`,
        target_id: result.target_id,
        target_url: result.target_url,
        stream_dump_path: result.stream_dump?.path ?? null,
        stream_dump_entries: result.stream_dump?.entries ?? 0,
        stream_dump_bytes: result.stream_dump?.bytes ?? 0,
        stream_dump_error: result.stream_dump?.error ?? null,
        schema_dump_path: result.stream_dump?.schema?.path ?? null,
        schema_dump_entries: result.stream_dump?.schema?.entries ?? 0,
        schema_dump_keys: result.stream_dump?.schema?.keys ?? 0,
        schema_dump_bytes: result.stream_dump?.schema?.bytes ?? 0,
        schema_dump_error: result.stream_dump?.schema?.error ?? null,
        inner_text_dump_path: result.inner_text_dump?.path ?? null,
        inner_text_dump_entries: result.inner_text_dump?.entries ?? 0,
        inner_text_dump_candidates: result.inner_text_dump?.text_candidates ?? 0,
        inner_text_dump_payload_json: result.inner_text_dump?.payload_json_entries ?? 0,
        inner_text_dump_bytes: result.inner_text_dump?.bytes ?? 0,
        inner_text_dump_error: result.inner_text_dump?.error ?? null,
        ...result.raw_stats,
      },
    });
  } catch (err) {
    if (stream && !res.headersSent) {
      setupSse(res);
      sseWrite(res, { error: { message: err.message, type: "cdp_browser_router_error" } });
      res.write("data: [DONE]\n\n");
      res.end();
      return;
    }
    if (!res.destroyed && !res.headersSent) errorResponse(res, 502, err.message, { code: "cdp_turn_failed" });
  }
}

async function handleTabs(_req, res) {
  try {
    const targets = await cdpListTargets();
    jsonResponse(res, 200, {
      cdp: `http://${CDP_HOST}:${CDP_PORT}`,
      targets: targets
        .filter((t) => t.type === "page")
        .map((t) => ({ id: t.id, title: t.title, url: t.url, attached: t.attached ?? null })),
    });
  } catch (err) {
    errorResponse(res, 502, err.message, { code: "cdp_unavailable" });
  }
}

const httpServer = http.createServer(async (req, res) => {
  const url = new URL(req.url ?? "/", `http://${HTTP_HOST}:${HTTP_PORT}`);
  if (req.method === "OPTIONS") return sendCorsOptions(res);
  if (req.method === "GET" && url.pathname === "/healthz") {
    try {
      const version = await cdpVersion();
      return jsonResponse(res, 200, {
        ok: true,
        backend: "cdp",
        http_url: `http://${HTTP_HOST}:${HTTP_PORT}`,
        cdp_url: `http://${CDP_HOST}:${CDP_PORT}`,
        browser: version.Browser ?? version["Browser"] ?? null,
        protocol_version: version["Protocol-Version"] ?? null,
        network_stream_resource_content: true,
        websocket_fallback: ENABLE_WS_FALLBACK,
      });
    } catch (err) {
      return errorResponse(res, 502, err.message, { code: "cdp_unavailable" });
    }
  }
  if (req.method === "GET" && url.pathname === "/v1/models") {
    return jsonResponse(res, 200, {
      object: "list",
      data: [
        { id: "chatgpt-cdp", object: "model", owned_by: "browser-cdp" },
        { id: "gemini-cdp", object: "model", owned_by: "browser-cdp" },
      ],
    });
  }
  if (req.method === "GET" && url.pathname === "/tabs") return handleTabs(req, res);
  if (req.method === "POST" && url.pathname === "/v1/chat/completions") return handleChatCompletions(req, res);
  errorResponse(res, 404, `not found: ${req.method} ${url.pathname}`, { code: "not_found" });
});

httpServer.listen(HTTP_PORT, HTTP_HOST, () => {
  console.log(`[http] CDP OpenAI-compatible API: http://${HTTP_HOST}:${HTTP_PORT}`);
  console.log(`[cdp] Chrome DevTools endpoint: http://${CDP_HOST}:${CDP_PORT}`);
});

process.on("SIGINT", () => shutdown());
process.on("SIGTERM", () => shutdown());

function shutdown() {
  console.log("\n[shutdown]");
  httpServer.close(() => process.exit(0));
  setTimeout(() => process.exit(0), 500).unref();
}
