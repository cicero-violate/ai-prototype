function isObject(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function walkJson(value, visit, path = "$", ancestors = []) {
  visit(value, path, ancestors);
  if (Array.isArray(value)) {
    value.forEach((item, i) => walkJson(item, visit, `${path}[${i}]`, [...ancestors, value]));
    return;
  }
  if (isObject(value)) {
    for (const [key, nested] of Object.entries(value)) {
      walkJson(nested, visit, `${path}.${key}`, [...ancestors, value]);
    }
  }
}

function stringLooksLikePointer(value) {
  return value === "" || (typeof value === "string" && value.startsWith("/") && value.length <= 512);
}

function stringLooksLikeOperation(value) {
  return typeof value === "string" &&
    /^(add|append|replace|patch|delta|insert|set|remove|delete)$/i.test(value);
}

function pathLooksTextBearing(pathName) {
  const lower = String(pathName ?? "").toLowerCase();
  if (/(token|authorization|cookie|secret|cursor|magic_link|avatar|url|id|uuid|digest|hash)/.test(lower)) return false;
  if (/(input_message|user_editable_context|developer_content|context_scopes|scope_namespace|content_type|conversation_id|message_id)/.test(lower)) return false;
  const segments = lower.split(/[^a-z0-9_]+/).filter(Boolean);
  const leaf = segments.at(-1) ?? "";
  if (segments.some((s) => /^(metadata|thoughts|reasoning|inspector|citation|citations)$/.test(s))) return false;
  if (/^(type|content_type|status|role|name|namespace|scope|scope_namespace|kind|language)$/.test(leaf)) return false;
  const has = (name) => segments.includes(name);
  if (has("content") && (has("message") || has("delta") || has("choice") || has("choices") || has("part") || has("parts") || has("text"))) return true;
  return /^(text|body|answer|response|output|markdown|code)$/.test(leaf);
}

function valueLooksHumanText(value) {
  const text = String(value ?? "");
  if (!text.trim()) return false;
  if (/^(global|text|user|assistant|system|developer)$/i.test(text.trim())) return false;
  if (/^\d+\.\d+$/.test(text.trim())) return false;
  if (text.length > 256 && /^[A-Za-z0-9+/_=.-]+$/.test(text) && !/\s/.test(text)) return false;
  if (/^https?:\/\//i.test(text)) return false;
  return true;
}

function detectPatchLikeObject(value) {
  if (!isObject(value)) return null;
  const entries = Object.entries(value);
  const pointer = entries.find(([, v]) => stringLooksLikePointer(v));
  const operation = entries.find(([, v]) => stringLooksLikeOperation(v));
  if (!operation) return null;
  let payload = null;
  if (pointer) {
    payload = entries.find(([k]) => k !== pointer[0] && k !== operation[0]);
  } else {
    payload = entries.find(([k]) => k !== operation[0]);
  }
  if (!payload) return null;
  return {
    pointer_key: pointer?.[0] ?? null,
    pointer: pointer?.[1] ?? null,
    operation_key: operation[0],
    operation: String(operation[1]).toLowerCase(),
    value_key: payload[0],
    value: payload[1],
  };
}

function extractTextCandidates(value, state, out, source = "payload") {
  const patch = detectPatchLikeObject(value);
  if (patch) {
    if (patch.operation === "patch" && Array.isArray(patch.value)) {
      for (const nested of patch.value) extractTextCandidates(nested, state, out, "patch_array");
      return;
    }
    if (typeof patch.value === "string") {
      if (pathLooksTextBearing(patch.pointer) && valueLooksHumanText(patch.value)) {
        state.active_pointer = patch.pointer;
        state.active_operation = patch.operation;
        out.push({ text: patch.value, rule: { source, kind: "schema_patch_string", pointer: patch.pointer, operation: patch.operation } });
        return;
      }
    }
    if ((patch.operation === "add" || patch.operation === "replace") && isObject(patch.value)) {
      extractTextCandidates(patch.value, state, out, "patch_object_value");
      return;
    }
  }

  if (isObject(value)) {
    const entries = Object.entries(value);
    const stringEntries = entries.filter(([, v]) => typeof v === "string");
    if (stringEntries.length === 1 && entries.length <= 2 && state.active_pointer) {
      const [valueKey, text] = stringEntries[0];
      if (valueLooksHumanText(text)) {
        out.push({ text, rule: { source, kind: "schema_continuation_string", value_key: valueKey, active_pointer: state.active_pointer } });
        return;
      }
    }
  }

  walkJson(value, (node, pathName) => {
    if (typeof node !== "string") return;
    if (!pathLooksTextBearing(pathName) || !valueLooksHumanText(node)) return;
    out.push({ text: node, rule: { source, kind: "schema_text_leaf", path: pathName } });
  });
}

export function makeTextCandidateExtractor() {
  const state = { active_pointer: null, active_operation: null };
  return {
    extractFromValue(value) {
      const candidates = [];
      extractTextCandidates(value, state, candidates);
      return candidates;
    },
  };
}
