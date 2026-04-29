import { parseSseFrames } from "./frame-parser.mjs";

function tryJson(text) {
  try { return JSON.parse(text); } catch { return null; }
}

function parsePayloadValues(raw) {
  const values = [];
  const frames = parseSseFrames(raw);
  let done = false;
  if (frames.length > 0) {
    for (const frame of frames) {
      if (!frame.data) continue;
      if (frame.data === "[DONE]") { done = true; continue; }
      const parsed = tryJson(frame.data);
      if (parsed) values.push({ parsed, event_name: frame.event_name ?? null });
    }
    return { values, done };
  }
  const parsed = tryJson(String(raw ?? "").trim());
  if (parsed) values.push({ parsed, event_name: null });
  return { values, done };
}

function pathLooksAssistant(pathName) {
  const p = String(pathName ?? "").toLowerCase();
  if (/(prompt|instruction|system|metadata|reasoning|thought|debug|policy|profile|auth|token|cookie)/.test(p)) return false;
  if (/(input_message|user_editable_context|developer_content|context_scopes|scope_namespace|content_type|conversation_id|message_id)/.test(p)) return false;
  // Prefer known assistant-bearing payload locations and deltas.
  return /(\.v\.message\.content\.parts\[[0-9]+\]$|\.message\.content\.parts\[[0-9]+\]$|\.delta(\.|$)|\.output_text(\.|$)|\.response_text(\.|$)|\.text$)/.test(p);
}

function looksLikeAssistantText(node) {
  const text = String(node ?? "").trim();
  if (!text) return false;
  if (/^(global|text|user|assistant|system|developer)$/i.test(text)) return false;
  if (/^\d+\.\d+$/.test(text)) return false;
  return true;
}

function walk(value, visit, path = "$") {
  visit(value, path);
  if (Array.isArray(value)) value.forEach((item, idx) => walk(item, visit, `${path}[${idx}]`));
  else if (value && typeof value === "object") {
    for (const [k, v] of Object.entries(value)) walk(v, visit, `${path}.${k}`);
  }
}

function schemaKeyForValue(value) {
  const op = typeof value?.o === "string" ? value.o : "obj";
  const p = typeof value?.p === "string" ? value.p.split("/").slice(1, 3).join("_") : "";
  return p ? `${op}__${p}` : op;
}

function extractCandidates(value, sourceRef = {}) {
  const out = [];
  const schemaKey = schemaKeyForValue(value);
  const schemaPath = `schemas/${schemaKey}.schema.json`;
  walk(value, (node, path) => {
    if (typeof node !== "string") return;
    const text = node.trim();
    if (!looksLikeAssistantText(text)) return;
    if (!pathLooksAssistant(path)) return;
    if (text.startsWith("You are ") || text.includes("USER PROFILE") || text.includes("The user provided")) return;
    const ruleId = `schema_path_text:${schemaKey}:${path}`;
    out.push({
      text: node,
      rule: {
        kind: "schema_path_text",
        path,
        schema_key: schemaKey,
        schema_path: schemaPath,
        rule_id: ruleId,
        source_ref: sourceRef,
      },
    });
  });
  return out;
}

export function makeSchemaGuidedTextExtractor() {
  return {
    extract(raw) {
      const { values, done } = parsePayloadValues(raw);
      const all = [];
      for (let i = 0; i < values.length; i += 1) {
        const item = values[i];
        all.push(...extractCandidates(item.parsed, { payload_index: i, event_name: item.event_name }));
      }
      return { candidates: all, done, values: values.map((v) => v.parsed) };
    },
  };
}
