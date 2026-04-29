#!/usr/bin/env node
import http from "node:http";
import { URL } from "node:url";

// Register provider adapters (order = specificity, most-specific first)
import "./provider/gemini-private.adapter.mjs";
import "./provider/chatgpt-group.adapter.mjs";
import "./provider/chatgpt-project.adapter.mjs";
import "./provider/chatgpt-private.adapter.mjs";

import { makeTargetManager } from "./browser/target-manager.mjs";
import { handleChatCompletions } from "./api/openai-compatible.mjs";
import { createUploadActionHandler } from "./api/upload-action.mjs";
import { createGroupChatActionHandler } from "./api/group-chat-action.mjs";
import { listAdapters } from "./provider/registry.mjs";

const HTTP_HOST = process.env.HTTP_HOST ?? "127.0.0.1";
const HTTP_PORT = Number(process.env.HTTP_PORT ?? 8081);
const CDP_HOST = process.env.CDP_HOST ?? "127.0.0.1";
const CDP_PORT = Number(process.env.CDP_PORT ?? 9221);
const MAX_BODY_BYTES = Number(process.env.MAX_BODY_BYTES ?? 1_000_000);
const DEFAULT_PROJECT_ID = process.env.CHATGPT_PROJECT_ID ?? "";
const CDP_UPLOAD_SCRIPT = process.env.CDP_UPLOAD_SCRIPT ?? "";

const config = {
  idleMs: Number(process.env.TURN_IDLE_MS ?? 2500),
  firstCaptureMs: Number(process.env.TURN_FIRST_CAPTURE_MS ?? 45000),
  maxMs: Number(process.env.TURN_MAX_MS ?? 120000),
  wsEnabled: String(process.env.CDP_WS_FALLBACK ?? "1") !== "0",
  cdpHost: CDP_HOST,
  cdpPort: CDP_PORT,
  uploadScript: CDP_UPLOAD_SCRIPT,
  defaultProjectId: DEFAULT_PROJECT_ID,
};

const targetManager = makeTargetManager({ cdpHost: CDP_HOST, cdpPort: CDP_PORT });

function readBody(req) {
  return new Promise((resolve, reject) => {
    let size = 0;
    const chunks = [];
    req.on("data", (chunk) => {
      size += chunk.length;
      if (size > MAX_BODY_BYTES) { reject(new Error(`request body too large; max=${MAX_BODY_BYTES}`)); req.destroy(); return; }
      chunks.push(chunk);
    });
    req.on("end", () => resolve(Buffer.concat(chunks).toString("utf8")));
    req.on("error", reject);
  });
}

function parseJsonObject(text) {
  if (!text.trim()) return {};
  const value = JSON.parse(text);
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error("expected JSON object");
  return value;
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
  jsonResponse(res, status, { error: { message, type: extra.type ?? "cdp_browser_router_error", code: extra.code ?? null, ...extra } });
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

const ctx = { readBody, parseJsonObject, jsonResponse, errorResponse, targetManager, config };

const handleUploadAction = createUploadActionHandler({
  parseJsonObject, readBody, errorResponse, jsonResponse,
  cdpHost: CDP_HOST, cdpPort: CDP_PORT,
  defaultProjectId: DEFAULT_PROJECT_ID,
  uploadScript: CDP_UPLOAD_SCRIPT,
});

const handleGroupChatAction = createGroupChatActionHandler({
  parseJsonObject, readBody, errorResponse, jsonResponse,
  targetManager,
});

const server = http.createServer(async (req, res) => {
  const url = new URL(req.url ?? "/", `http://${HTTP_HOST}:${HTTP_PORT}`);
  if (req.method === "OPTIONS") return sendCorsOptions(res);

  if (req.method === "GET" && url.pathname === "/healthz") {
    try {
      const version = await targetManager.getVersion();
      return jsonResponse(res, 200, {
        ok: true,
        backend: "cdp",
        http_url: `http://${HTTP_HOST}:${HTTP_PORT}`,
        cdp_url: `http://${CDP_HOST}:${CDP_PORT}`,
        browser: version.Browser ?? null,
        protocol_version: version["Protocol-Version"] ?? null,
        ws_fallback: config.wsEnabled,
        providers: listAdapters(),
      });
    } catch (err) {
      return errorResponse(res, 502, err.message, { code: "cdp_unavailable" });
    }
  }

  if (req.method === "GET" && url.pathname === "/v1/models") {
    const adapters = listAdapters();
    return jsonResponse(res, 200, {
      object: "list",
      data: adapters.map((a) => ({ id: `${a.provider}-cdp`, object: "model", owned_by: "browser-cdp", provider: a.provider })),
    });
  }

  if (req.method === "GET" && url.pathname === "/tabs") {
    try {
      const targets = await targetManager.listTargets();
      return jsonResponse(res, 200, {
        cdp: `http://${CDP_HOST}:${CDP_PORT}`,
        targets: targets.filter((t) => t.type === "page").map((t) => ({ id: t.id, title: t.title, url: t.url })),
      });
    } catch (err) {
      return errorResponse(res, 502, err.message, { code: "cdp_unavailable" });
    }
  }

  if (req.method === "POST" && url.pathname === "/v1/chat/completions") {
    return handleChatCompletions(req, res, ctx);
  }

  if (req.method === "POST" && url.pathname === "/actions/upload") {
    return handleUploadAction(req, res);
  }

  if (req.method === "POST" && url.pathname === "/actions/group-chat") {
    return handleGroupChatAction(req, res);
  }

  errorResponse(res, 404, `not found: ${req.method} ${url.pathname}`, { code: "not_found" });
});

server.listen(HTTP_PORT, HTTP_HOST, () => {
  console.log(`[http] CDP OpenAI-compatible API: http://${HTTP_HOST}:${HTTP_PORT}`);
  console.log(`[cdp]  Chrome DevTools endpoint:   http://${CDP_HOST}:${CDP_PORT}`);
  console.log(`[providers] ${listAdapters().map((a) => a.provider).join(", ")}`);
});

process.on("SIGINT", shutdown);
process.on("SIGTERM", shutdown);

function shutdown() {
  console.log("\n[shutdown]");
  server.close(() => process.exit(0));
  setTimeout(() => process.exit(0), 500).unref();
}
