function mergeTypes(a, b) {
  const vals = [];
  for (const t of [a, b]) {
    if (Array.isArray(t)) vals.push(...t);
    else if (t) vals.push(t);
  }
  const merged = [...new Set(vals)].sort();
  return merged.length <= 1 ? merged[0] : merged;
}

function hasType(t, name) {
  return Array.isArray(t) ? t.includes(name) : t === name;
}

export function mergeSchema(a, b) {
  if (!a) return b;
  if (!b) return a;
  const out = { type: mergeTypes(a.type, b.type) };

  if (hasType(out.type, "object")) {
    out.properties = {};
    const keys = new Set([...Object.keys(a.properties ?? {}), ...Object.keys(b.properties ?? {})]);
    for (const key of [...keys].sort()) {
      out.properties[key] = mergeSchema(a.properties?.[key], b.properties?.[key]);
    }
    const reqA = new Set(a.required ?? []);
    const reqB = new Set(b.required ?? []);
    out.required = [...reqA].filter((k) => reqB.has(k)).sort();
    out.additionalProperties = true;
  }

  if (hasType(out.type, "array")) {
    out.items = mergeSchema(a.items, b.items) ?? {};
  }
  return out;
}

function inferSchema(value, depth = 0, maxDepth = 8) {
  if (value === null) return { type: "null" };
  if (Array.isArray(value)) {
    if (depth >= maxDepth) return { type: "array", items: {} };
    let items = null;
    for (const v of value) items = mergeSchema(items, inferSchema(v, depth + 1, maxDepth));
    return { type: "array", items: items ?? {} };
  }
  const kind = typeof value;
  if (kind === "string") return { type: "string" };
  if (kind === "boolean") return { type: "boolean" };
  if (kind === "number") return { type: Number.isInteger(value) ? "integer" : "number" };
  if (kind !== "object") return { type: "string" };
  if (depth >= maxDepth) return { type: "object", additionalProperties: true };
  const properties = {};
  const required = [];
  for (const key of Object.keys(value).sort()) {
    properties[key] = inferSchema(value[key], depth + 1, maxDepth);
    required.push(key);
  }
  return { type: "object", properties, required, additionalProperties: true };
}

function schemaKey(value) {
  const op = typeof value?.o === "string" ? value.o : "obj";
  const p = typeof value?.p === "string" ? value.p.split("/").slice(1, 3).join("_") : "";
  return p ? `${op}__${p}` : op;
}

export function makeSchemaObserver() {
  const byKey = new Map();
  const FULL_KEY = "raw_payload_json__unclassified";

  function mergeIntoKey(key, value) {
    const curr = byKey.get(key) ?? { key, count: 0, schema: null, samples: [] };
    curr.count += 1;
    curr.schema = mergeSchema(curr.schema, inferSchema(value));
    curr.samples.push(value);
    byKey.set(key, curr);
  }

  function observe(value) {
    if (!value || typeof value !== "object") return;
    mergeIntoKey(FULL_KEY, value);
    mergeIntoKey(schemaKey(value), value);
  }

  function snapshot() {
    const keys = [...byKey.values()].sort((a, b) => a.key.localeCompare(b.key));
    return {
      index: {
        schema: "ai_chromium.schema_index.v1",
        created_at: new Date().toISOString(),
        keys: keys.map((k) => ({ key: k.key, count: k.count, schema_path: `${k.key}.schema.json` })),
      },
      docs: keys.map((k) => ({
        file: `${k.key}.schema.json`,
        json: { $schema: "https://json-schema.org/draft/2020-12/schema", title: k.key, ...k.schema },
      })),
      samples: keys.map((k) => ({
        file: `${k.key}.samples.json`,
        json: {
          key: k.key,
          count: k.count,
          samples: k.samples,
        },
      })),
      count: keys.length,
    };
  }

  return { observe, snapshot };
}
