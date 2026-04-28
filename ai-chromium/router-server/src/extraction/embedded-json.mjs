export function tryParseEmbeddedJson(text) {
  if (typeof text !== "string") return null;
  const trimmed = text.trim();
  if (!trimmed) return null;
  const ok = (trimmed.startsWith("{") && trimmed.endsWith("}")) ||
              (trimmed.startsWith("[") && trimmed.endsWith("]"));
  if (!ok) return null;
  try {
    const parsed = JSON.parse(trimmed);
    return parsed && typeof parsed === "object" ? parsed : null;
  } catch { return null; }
}

export function collectEmbeddedJsonCandidates(value, {
  path = "$", depth = 0, maxDepth = 6, maxCandidates = 8, out = [],
} = {}) {
  if (out.length >= maxCandidates || depth > maxDepth) return out;
  if (typeof value === "string") {
    const parsed = tryParseEmbeddedJson(value);
    if (parsed) out.push({ path, parsed, text: value });
    return out;
  }
  if (Array.isArray(value)) {
    value.forEach((item, i) =>
      collectEmbeddedJsonCandidates(item, { path: `${path}[${i}]`, depth: depth + 1, maxDepth, maxCandidates, out })
    );
    return out;
  }
  if (value && typeof value === "object") {
    Object.entries(value).forEach(([key, nested]) =>
      collectEmbeddedJsonCandidates(nested, { path: `${path}.${key}`, depth: depth + 1, maxDepth, maxCandidates, out })
    );
  }
  return out;
}

export function makeEmbeddedJsonReassembler() {
  const buffers = new Map();

  function feedPatch({ requestId, op, path, value }) {
    if (op !== "append" || typeof path !== "string" || typeof value !== "string") return null;
    const key = `${requestId}::${path}`;
    const next = (buffers.get(key) ?? "") + value;
    if (next.length > 1_000_000) { buffers.delete(key); return null; }
    buffers.set(key, next);
    const parsed = tryParseEmbeddedJson(next);
    if (!parsed) return null;
    buffers.delete(key);
    return { path, parsed };
  }

  function clearRequest(requestId) {
    for (const key of buffers.keys()) {
      if (key.startsWith(`${requestId}::`)) buffers.delete(key);
    }
  }

  return { feedPatch, clearRequest };
}
