function isObject(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function walkJson(value, visit, path = "$", ancestors = []) {
  visit(value, path, ancestors);
  if (Array.isArray(value)) {
    value.forEach((item, index) => walkJson(item, visit, `${path}[${index}]`, [...ancestors, value]));
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
  return typeof value === "string" && /^(add|append|replace|patch|delta|insert|set|remove|delete)$/i.test(value);
}

function pathLooksTextBearing(pathName) {
  const lower = String(pathName ?? "").toLowerCase();
  if (/(token|authorization|cookie|secret|cursor|magic_link|avatar|url|id|uuid|digest|hash)/.test(lower)) return false;
  const segments = lower.split(/[^a-z0-9_]+/).filter(Boolean);
  const leaf = segments.at(-1) ?? "";
  if (segments.some((segment) => /^(metadata|thoughts|reasoning|inspector|citation|citations)$/.test(segment))) return false;
  if (/^(type|content_type|status|role|name|namespace|scope|scope_namespace|kind|language)$/.test(leaf)) return false;
  const has = (name) => segments.includes(name);
  if (has("content") && (has("message") || has("delta") || has("choice") || has("choices") || has("part") || has("parts") || has("text"))) return true;
  return /^(text|body|answer|response|output|markdown|code)$/.test(leaf);
}

function valueLooksHumanText(value) {
  const text = String(value ?? "");
  if (!text.trim()) return false;
  if (text.length > 256 && /^[A-Za-z0-9+/_=.-]+$/.test(text) && !/\s/.test(text)) return false;
  if (/^https?:\/\//i.test(text)) return false;
  return true;
}

function detectPatchLikeObject(value) {
  if (!isObject(value)) return null;
  const entries = Object.entries(value);
  const pointer = entries.find(([_key, nested]) => stringLooksLikePointer(nested));
  const operation = entries.find(([_key, nested]) => stringLooksLikeOperation(nested));
  if (!operation) return null;

  let payload = null;
  if (pointer) {
    payload = entries.find(([key]) => key !== pointer[0] && key !== operation[0]);
  } else {
    payload = entries.find(([key]) => key !== operation[0]);
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

function extractSchemaTextCandidates(value, state, out, source = "payload") {
  const patch = detectPatchLikeObject(value);
  if (patch) {
    if (patch.operation === "patch" && Array.isArray(patch.value)) {
      for (const nested of patch.value) extractSchemaTextCandidates(nested, state, out, "patch_array");
      return;
    }

    if (typeof patch.value === "string") {
      const textBearing = pathLooksTextBearing(patch.pointer);
      const humanText = valueLooksHumanText(patch.value);
      if (textBearing && humanText) {
        state.active_pointer = patch.pointer;
        state.active_operation = patch.operation;
        out.push({
          text: patch.value,
          rule: {
            source,
            kind: "schema_patch_string",
            pointer_key: patch.pointer_key,
            operation_key: patch.operation_key,
            value_key: patch.value_key,
            pointer: patch.pointer,
            operation: patch.operation,
          },
        });
        return;
      }
    }

    if ((patch.operation === "add" || patch.operation === "replace") && isObject(patch.value)) {
      extractSchemaTextCandidates(patch.value, state, out, "patch_object_value");
      return;
    }
  }

  if (isObject(value)) {
    const entries = Object.entries(value);
    const stringEntries = entries.filter(([_key, nested]) => typeof nested === "string");
    if (stringEntries.length === 1 && entries.length <= 2 && state.active_pointer) {
      const [valueKey, text] = stringEntries[0];
      if (valueLooksHumanText(text)) {
        out.push({
          text,
          rule: {
            source,
            kind: "schema_continuation_string",
            value_key: valueKey,
            active_pointer: state.active_pointer,
            active_operation: state.active_operation,
          },
        });
        return;
      }
    }
  }

  walkJson(value, (node, pathName) => {
    if (typeof node !== "string") return;
    if (!pathLooksTextBearing(pathName) || !valueLooksHumanText(node)) return;
    out.push({
      text: node,
      rule: {
        source,
        kind: "schema_text_leaf",
        path: pathName,
      },
    });
  });
}

export function makeSchemaGuidedTextExtractor({ parsePayloadJsonValues, looksLikeConversationList }) {
  const state = {
    active_pointer: null,
    active_operation: null,
  };

  return {
    extract(raw) {
      if (typeof raw !== "string" || raw.length === 0) return { text: "", done: false, activity: false, rules: [] };
      if (looksLikeConversationList(raw)) return { text: "", done: false, activity: false, rules: [] };

      const parsed = parsePayloadJsonValues(raw);
      const candidates = [];
      for (const value of parsed.values) extractSchemaTextCandidates(value, state, candidates);

      const text = candidates.map((candidate) => candidate.text).join("");
      return {
        text,
        done: parsed.done,
        activity: candidates.length > 0,
        payload_count: parsed.values.length,
        frame_count: parsed.frame_count,
        parse_errors: parsed.parse_errors,
        rules: candidates.map((candidate) => candidate.rule),
      };
    },
  };
}
