#!/usr/bin/env node
import http from "node:http";
import { pathToFileURL, URL } from "node:url";

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

export function configFromEnv(env = process.env) {
  return {
    httpHost: env.HTTP_HOST ?? "127.0.0.1",
    httpPort: Number(env.HTTP_PORT ?? 8081),
    cdpHost: env.CDP_HOST ?? "127.0.0.1",
    cdpPort: Number(env.CDP_PORT ?? 9221),
    maxBodyBytes: Number(env.MAX_BODY_BYTES ?? 1_000_000),
    defaultProjectId: env.CHATGPT_PROJECT_ID ?? "",
    uploadScript: env.CDP_UPLOAD_SCRIPT ?? "",
    idleMs: Number(env.TURN_IDLE_MS ?? 2500),
    firstCaptureMs: Number(env.TURN_FIRST_CAPTURE_MS ?? 45000),
    maxMs: Number(env.TURN_MAX_MS ?? 120000),
    wsEnabled: String(env.CDP_WS_FALLBACK ?? "1") !== "0",
  };
}

function readBody(req, limitBytes) {
  return new Promise((resolve, reject) => {
    let size = 0;
    const chunks = [];
    req.on("data", (chunk) => {
      size += chunk.length;
      if (size > limitBytes) { reject(new Error(`request body too large; max=${limitBytes}`)); req.destroy(); return; }
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

export function createRouterServer({
  config = configFromEnv(),
  targetManager = makeTargetManager({ cdpHost: config.cdpHost, cdpPort: config.cdpPort }),
} = {}) {
  const scopedReadBody = (req) => readBody(req, config.maxBodyBytes);
  const ctx = { readBody: scopedReadBody, parseJsonObject, jsonResponse, errorResponse, targetManager, config };
  const handleUploadAction = createUploadActionHandler({
    parseJsonObject, readBody: scopedReadBody, errorResponse, jsonResponse,
    cdpHost: config.cdpHost, cdpPort: config.cdpPort,
    defaultProjectId: config.defaultProjectId,
    uploadScript: config.uploadScript,
  });
  const handleGroupChatAction = createGroupChatActionHandler({
    parseJsonObject, readBody: scopedReadBody, errorResponse, jsonResponse, targetManager,
  });

  return http.createServer(async (req, res) => {
    const url = new URL(req.url ?? "/", `http://${config.httpHost}:${config.httpPort}`);
    if (req.method === "OPTIONS") return sendCorsOptions(res);

    if (req.method === "GET" && url.pathname === "/healthz") {
      try {
        const version = await targetManager.getVersion();
        return jsonResponse(res, 200, {
          ok: true,
          backend: "cdp",
          http_url: `http://${config.httpHost}:${config.httpPort}`,
          cdp_url: `http://${config.cdpHost}:${config.cdpPort}`,
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
      return jsonResponse(res, 200, {
        object: "list",
        data: listAdapters().map((a) => ({ id: `${a.provider}-cdp`, object: "model", owned_by: "browser-cdp", provider: a.provider })),
      });
    }

    if (req.method === "GET" && url.pathname === "/tabs") {
      try {
        const targets = await targetManager.listTargets();
        return jsonResponse(res, 200, {
          cdp: `http://${config.cdpHost}:${config.cdpPort}`,
          targets: targets.filter((t) => t.type === "page").map((t) => ({ id: t.id, title: t.title, url: t.url })),
        });
      } catch (err) {
        return errorResponse(res, 502, err.message, { code: "cdp_unavailable" });
      }
    }

    if (req.method === "POST" && url.pathname === "/v1/chat/completions") return handleChatCompletions(req, res, ctx);
    if (req.method === "POST" && url.pathname === "/actions/upload") return handleUploadAction(req, res);
    if (req.method === "POST" && url.pathname === "/actions/group-chat") return handleGroupChatAction(req, res);
    return errorResponse(res, 404, `not found: ${req.method} ${url.pathname}`, { code: "not_found" });
  });
}

export function startServer({ config = configFromEnv(), targetManager } = {}) {
  const server = createRouterServer({ config, targetManager });
  server.listen(config.httpPort, config.httpHost, () => {
    console.log(`[http] CDP OpenAI-compatible API: http://${config.httpHost}:${config.httpPort}`);
    console.log(`[cdp]  Chrome DevTools endpoint:   http://${config.cdpHost}:${config.cdpPort}`);
    console.log(`[providers] ${listAdapters().map((a) => a.provider).join(", ")}`);
  });
  return server;
}

const server = process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href ? startServer() : null;

if (server) {
  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);
}

function shutdown() {
  console.log("\n[shutdown]");
  server.close(() => process.exit(0));
  setTimeout(() => process.exit(0), 500).unref();
}
