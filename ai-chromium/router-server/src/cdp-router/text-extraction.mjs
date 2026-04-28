import { makeSchemaGuidedTextExtractor as makeSchemaGuidedTextExtractorImpl } from "./schema-guided-extraction.mjs";

function tryJson(text) {
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
}

function looksLikeConversationList(text) {
  return text.includes('"items"') && text.includes('"GROUP_DM"') && text.includes('"magic_link_url"');
}


export function parseSseFrames(raw) {
  const frames = [];
  const lines = String(raw ?? "").split(/\r?\n/);
  let eventName = null;
  let dataLines = [];

  function flush() {
    if (eventName == null && dataLines.length === 0) return;
    frames.push({
      event_name: eventName,
      data: dataLines.join("\n"),
    });
    eventName = null;
    dataLines = [];
  }

  for (const line of lines) {
    if (line === "") {
      flush();
      continue;
    }
    if (line.startsWith("event:")) {
      eventName = line.slice(6).trim();
      continue;
    }
    if (line.startsWith("data:")) {
      dataLines.push(line.slice(5).trimStart());
    }
  }
  flush();
  return frames;
}

function parsePayloadJsonValues(raw) {
  const text = String(raw ?? "");
  const values = [];
  let done = false;
  if (!text) return { values, done, frame_count: 0, parse_errors: 0 };

  let parseErrors = 0;
  const frames = parseSseFrames(text);
  if (frames.length > 0) {
    for (const frame of frames) {
      if (!frame.data) continue;
      if (frame.data === "[DONE]") {
        done = true;
        continue;
      }
      const parsed = tryJson(frame.data);
      if (parsed === null) {
        parseErrors += 1;
        continue;
      }
      values.push(parsed);
    }
    return { values, done, frame_count: frames.length, parse_errors: parseErrors };
  }

  const parsed = tryJson(text);
  if (parsed !== null) values.push(parsed);
  return { values, done, frame_count: 0, parse_errors: parseErrors };
}

export function makeSchemaGuidedTextExtractor() {
  return makeSchemaGuidedTextExtractorImpl({
    parsePayloadJsonValues,
    looksLikeConversationList,
  });
}

function jsonShape(value) {
  if (Array.isArray(value)) {
    return {
      type: "array",
      length: value.length,
      item_types: [...new Set(value.slice(0, 10).map((item) => Array.isArray(item) ? "array" : typeof item))],
    };
  }
  if (value && typeof value === "object") {
    return {
      type: "object",
      keys: Object.keys(value).slice(0, 40),
    };
  }
  return { type: typeof value };
}

function summarizePayloadJson(value) {
  if (value && typeof value === "object") {
    return {
      shape: jsonShape(value),
      type: value.type ?? null,
      op: value.o ?? value.op ?? null,
      patch_path: value.p ?? null,
      message_role: value.message?.author?.role ?? value.author?.role ?? null,
      content_type: value.message?.content?.content_type ?? value.content?.content_type ?? null,
      channel: value.message?.channel ?? value.channel ?? null,
    };
  }
  return {
    shape: jsonShape(value),
    type: null,
    op: null,
    patch_path: null,
    message_role: null,
    content_type: null,
    channel: null,
  };
}

export function extractPayloadJsonDumpFields(raw) {
  const text = String(raw ?? "");
  if (!text) return {};

  const payloadItems = [];
  const payloadJson = [];
  let parseErrors = 0;

  function addPayloadItem(base, data) {
    const item = {
      ...base,
      data_bytes: Buffer.byteLength(String(data ?? ""), "utf8"),
    };
    if (data === "[DONE]") {
      item.done = true;
      payloadItems.push(item);
      return;
    }
    const parsed = tryJson(data);
    if (parsed === null) {
      parseErrors += 1;
      item.payload_parse_error = "invalid_json";
      payloadItems.push(item);
      return;
    }
    item.payload_json = parsed;
    Object.assign(item, summarizePayloadJson(parsed));
    payloadJson.push(parsed);
    payloadItems.push(item);
  }

  const frames = parseSseFrames(text);
  if (frames.length > 0) {
    frames.forEach((frame, frameIndex) => {
      addPayloadItem({
        frame_index: frameIndex,
        event_name: frame.event_name,
      }, frame.data);
    });
  } else {
    const parsed = tryJson(text);
    if (parsed !== null) {
      const item = {
        frame_index: null,
        event_name: null,
        data_bytes: Buffer.byteLength(text, "utf8"),
        payload_json: parsed,
        ...summarizePayloadJson(parsed),
      };
      payloadItems.push(item);
      payloadJson.push(parsed);
    }
  }

  if (payloadItems.length === 0) return {};
  return {
    payload_items: payloadItems,
    payload_item_count: payloadItems.length,
    payload_json: payloadJson,
    payload_json_count: payloadJson.length,
    payload_json_parse_errors: parseErrors,
  };
}

function classifyTextPath(pathName, value) {
  const lowerPath = String(pathName ?? "").toLowerCase();
  const text = String(value ?? "");
  if (!text.trim()) return { keep: false, reason: "empty" };
  if (/(token|authorization|cookie|secret|cursor|magic_link|avatar_url|access_token|refresh_token)/i.test(lowerPath)) {
    return { keep: false, reason: "sensitive_path" };
  }
  if (text.length > 256 && /^[A-Za-z0-9+/_=.-]+$/.test(text) && !/\s/.test(text)) {
    return { keep: false, reason: "opaque_encoded_string" };
  }
  if (/^(id|message_id|conversation_id|request_id|target_id)$/i.test(lowerPath.split(/[.[\]]+/).filter(Boolean).at(-1) ?? "")) {
    return { keep: false, reason: "identifier" };
  }
  return { keep: true, reason: null };
}

function truncateInnerText(text, maxBytes) {
  const value = String(text ?? "");
  const originalBytes = Buffer.byteLength(value, "utf8");
  if (maxBytes > 0 && originalBytes > maxBytes) {
    return {
      text: Buffer.from(value, "utf8").subarray(0, maxBytes).toString("utf8"),
      text_bytes: originalBytes,
      text_truncated: true,
    };
  }
  return { text: value, text_bytes: originalBytes, text_truncated: false };
}

function collectInnerTextCandidates(value, { prefix = "$", out = [], depth = 0, maxStringBytes = 4096 } = {}) {
  if (depth > 40) return out;
  if (typeof value === "string") {
    const classification = classifyTextPath(prefix, value);
    if (classification.keep) {
      out.push({
        path: prefix,
        ...truncateInnerText(value, maxStringBytes),
      });
    }
    return out;
  }
  if (Array.isArray(value)) {
    value.forEach((item, index) => collectInnerTextCandidates(item, {
      prefix: `${prefix}[${index}]`,
      out,
      depth: depth + 1,
      maxStringBytes,
    }));
    return out;
  }
  if (value && typeof value === "object") {
    for (const [key, nested] of Object.entries(value)) {
      collectInnerTextCandidates(nested, {
        prefix: `${prefix}.${key}`,
        out,
        depth: depth + 1,
        maxStringBytes,
      });
    }
  }
  return out;
}

export function extractInnerTextEntries(raw, source = {}, { maxStringBytes = 4096 } = {}) {
  const text = String(raw ?? "");
  if (!text) return [];
  const entries = [];

  function addJsonShape({ frameIndex = null, eventName = null, dataIndex = null, value }) {
    entries.push({
      kind: "payload_json",
      source,
      frame_index: frameIndex,
      event_name: eventName,
      data_index: dataIndex,
      payload_json: value,
      ...summarizePayloadJson(value),
    });
    entries.push({
      kind: "json_shape",
      source,
      frame_index: frameIndex,
      event_name: eventName,
      data_index: dataIndex,
      shape: jsonShape(value),
      type: value?.type ?? null,
      op: value?.o ?? value?.op ?? null,
      patch_path: value?.p ?? null,
      message_role: value?.message?.author?.role ?? value?.author?.role ?? null,
      content_type: value?.message?.content?.content_type ?? value?.content?.content_type ?? null,
    });
  }

  function addTextCandidates({ frameIndex = null, eventName = null, dataIndex = null, value }) {
    const candidates = collectInnerTextCandidates(value, { maxStringBytes });
    for (const candidate of candidates) {
      entries.push({
        kind: "text_candidate",
        source,
        frame_index: frameIndex,
        event_name: eventName,
        data_index: dataIndex,
        type: value?.type ?? null,
        op: value?.o ?? value?.op ?? null,
        patch_path: value?.p ?? null,
        message_role: value?.message?.author?.role ?? value?.author?.role ?? null,
        content_type: value?.message?.content?.content_type ?? value?.content?.content_type ?? null,
        ...candidate,
      });
    }
  }

  const frames = parseSseFrames(text);
  if (frames.length > 0) {
    frames.forEach((frame, frameIndex) => {
      if (!frame.data || frame.data === "[DONE]") return;
      const json = tryJson(frame.data);
      if (!json) {
        const trimmed = frame.data.trim();
        if (trimmed) {
          entries.push({
            kind: "raw_sse_data",
            source,
            frame_index: frameIndex,
            event_name: frame.event_name,
            ...truncateInnerText(trimmed, maxStringBytes),
          });
        }
        return;
      }
      addJsonShape({ frameIndex, eventName: frame.event_name, value: json });
      addTextCandidates({ frameIndex, eventName: frame.event_name, value: json });
    });
    return entries;
  }

  const json = tryJson(text);
  if (json) {
    addJsonShape({ value: json });
    addTextCandidates({ value: json });
    return entries;
  }

  entries.push({
    kind: "raw_text",
    source,
    ...truncateInnerText(text, maxStringBytes),
  });
  return entries;
}

export function makeTextAccumulator() {
  let full = "";
  return {
    append(candidate) {
      if (!candidate) return "";
      if (candidate === full) return "";
      if (candidate.startsWith(full)) {
        const delta = candidate.slice(full.length);
        full = candidate;
        return delta;
      }
      if (full.endsWith(candidate)) return "";
      full += candidate;
      return candidate;
    },
    value() { return full; },
  };
}
