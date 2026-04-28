import fs from "node:fs";
import path from "node:path";

function mergeSchemaTypes(left, right) {
  const values = [];
  for (const type of [left, right]) {
    if (Array.isArray(type)) values.push(...type);
    else if (type) values.push(type);
  }
  const merged = [...new Set(values)].sort();
  return merged.length === 1 ? merged[0] : merged;
}

function mergeSchema(left, right) {
  if (!left) return right;
  if (!right) return left;

  const mergedType = mergeSchemaTypes(left.type, right.type);
  const merged = { type: mergedType };

  if (left.description || right.description) merged.description = left.description ?? right.description;
  if (left.format || right.format) merged.format = left.format ?? right.format;

  const leftHasObject = Array.isArray(left.type) ? left.type.includes("object") : left.type === "object";
  const rightHasObject = Array.isArray(right.type) ? right.type.includes("object") : right.type === "object";
  if (leftHasObject || rightHasObject) {
    merged.properties = {};
    const names = new Set([...Object.keys(left.properties ?? {}), ...Object.keys(right.properties ?? {})]);
    for (const name of [...names].sort()) {
      merged.properties[name] = mergeSchema(left.properties?.[name], right.properties?.[name]);
    }
    const leftRequired = new Set(left.required ?? []);
    const rightRequired = new Set(right.required ?? []);
    merged.required = [...leftRequired].filter((name) => rightRequired.has(name)).sort();
    merged.additionalProperties = true;
  }

  const leftHasArray = Array.isArray(left.type) ? left.type.includes("array") : left.type === "array";
  const rightHasArray = Array.isArray(right.type) ? right.type.includes("array") : right.type === "array";
  if (leftHasArray || rightHasArray) {
    merged.items = mergeSchema(left.items, right.items) ?? {};
  }

  return merged;
}

function inferJsonSchema(value, depth = 0, maxDepth = 8) {
  if (value === null) return { type: "null" };
  if (Array.isArray(value)) {
    if (depth >= maxDepth) return { type: "array", items: {} };
    let itemSchema = null;
    for (const item of value) {
      itemSchema = mergeSchema(itemSchema, inferJsonSchema(item, depth + 1, maxDepth));
    }
    return { type: "array", items: itemSchema ?? {} };
  }
  const kind = typeof value;
  if (kind === "string") return { type: "string" };
  if (kind === "boolean") return { type: "boolean" };
  if (kind === "number") return { type: Number.isInteger(value) ? "integer" : "number" };
  if (kind === "bigint") return { type: "string", description: "bigint serialized as string" };
  if (kind === "object") {
    if (depth >= maxDepth) return { type: "object", additionalProperties: true };
    const properties = {};
    const required = [];
    for (const name of Object.keys(value).sort()) {
      properties[name] = inferJsonSchema(value[name], depth + 1, maxDepth);
      required.push(name);
    }
    return { type: "object", properties, required, additionalProperties: true };
  }
  return { type: "string", description: `non-json ${kind} serialized as string` };
}

function makeNoopSchemaDumpWriter(error = null) {
  return {
    path: null,
    observe() {},
    stats() { return { path: null, entries: 0, keys: 0, bytes: 0, error }; },
    close() { return this.stats(); },
  };
}

function schemaKeyForRecord(record, safeFileComponent) {
  const event = safeFileComponent(record?.event ?? "unknown_event");
  const method = record?.cdp_method ? `__${safeFileComponent(record.cdp_method)}` : "";
  const suffix = record?.schema_key_suffix ? `__${safeFileComponent(record.schema_key_suffix)}` : "";
  return `${event}${method}${suffix}`;
}

export function makeSchemaDumpWriter({
  completionId,
  model,
  target,
  enabled,
  dirPath,
  sampleLimit,
  maxDepth,
  safeFileComponent,
}) {
  if (!enabled) return makeNoopSchemaDumpWriter("disabled");
  try {
    fs.mkdirSync(dirPath, { recursive: true });
    const stamp = new Date().toISOString().replace(/[:.]/g, "-");
    const outputDirPath = path.join(
      dirPath,
      `${stamp}-${safeFileComponent(completionId)}-${safeFileComponent(model)}`,
    );
    fs.mkdirSync(outputDirPath, { recursive: true });
    const registry = new Map();
    let entries = 0;
    let bytes = 0;
    let writeError = null;
    let closed = false;

    function observe(record) {
      if (closed || !record || typeof record !== "object") return;
      entries += 1;
      const key = schemaKeyForRecord(record, safeFileComponent);
      const current = registry.get(key) ?? {
        key,
        event: record.event ?? null,
        cdp_method: record.cdp_method ?? null,
        count: 0,
        first_seen: record.ts ?? new Date().toISOString(),
        last_seen: null,
        schema: null,
        samples: [],
      };
      current.count += 1;
      current.last_seen = record.ts ?? new Date().toISOString();
      current.schema = mergeSchema(current.schema, inferJsonSchema(record, 0, maxDepth));
      if (current.samples.length < sampleLimit) current.samples.push(record);
      registry.set(key, current);
    }

    function writeJsonFile(filePath, value) {
      const json = `${JSON.stringify(value, null, 2)}\n`;
      fs.writeFileSync(filePath, json);
      bytes += Buffer.byteLength(json, "utf8");
    }

    function flush() {
      if (writeError) return;
      try {
        const index = {
          meta: {
            completion_id: completionId,
            model,
            target_id: target?.id ?? null,
            target_url: target?.url ?? null,
            created_at: new Date().toISOString(),
            source: "cdp-browser-llm-router",
            schema: "cdp.schema_dump.v1",
          },
          stats: {
            entries,
            keys: registry.size,
            sample_limit: sampleLimit,
            max_depth: maxDepth,
          },
          keys: [...registry.values()]
            .sort((a, b) => a.key.localeCompare(b.key))
            .map((item) => ({
              key: item.key,
              event: item.event,
              cdp_method: item.cdp_method,
              count: item.count,
              first_seen: item.first_seen,
              last_seen: item.last_seen,
              schema_path: `${item.key}.schema.json`,
              samples_path: `${item.key}.samples.json`,
            })),
        };
        writeJsonFile(path.join(outputDirPath, "index.json"), index);
        for (const item of [...registry.values()].sort((a, b) => a.key.localeCompare(b.key))) {
          const schemaDocument = {
            $schema: "https://json-schema.org/draft/2020-12/schema",
            title: item.key,
            description: `Inferred from ${item.count} streamed CDP dump record(s).`,
            ...item.schema,
          };
          writeJsonFile(path.join(outputDirPath, `${item.key}.schema.json`), schemaDocument);
          writeJsonFile(path.join(outputDirPath, `${item.key}.samples.json`), {
            key: item.key,
            event: item.event,
            cdp_method: item.cdp_method,
            count: item.count,
            samples: item.samples,
          });
        }
      } catch (err) {
        writeError = String(err?.message ?? err);
      }
    }

    return {
      path: outputDirPath,
      observe,
      stats() {
        return { path: outputDirPath, entries, keys: registry.size, bytes, error: writeError };
      },
      close() {
        if (!closed) {
          closed = true;
          flush();
        }
        return this.stats();
      },
    };
  } catch (err) {
    return makeNoopSchemaDumpWriter(String(err?.message ?? err));
  }
}

export function tryParseEmbeddedJson(text) {
  if (typeof text !== "string") return null;
  const trimmed = text.trim();
  if (!trimmed) return null;
  const startsLikeJson = (trimmed.startsWith("{") && trimmed.endsWith("}")) || (trimmed.startsWith("[") && trimmed.endsWith("]"));
  if (!startsLikeJson) return null;
  try {
    const parsed = JSON.parse(trimmed);
    return parsed && typeof parsed === "object" ? parsed : null;
  } catch {
    return null;
  }
}

export function collectEmbeddedJsonCandidates(value, {
  path = "$",
  depth = 0,
  maxDepth = 6,
  maxCandidates = 8,
  out = [],
} = {}) {
  if (out.length >= maxCandidates || depth > maxDepth) return out;
  if (typeof value === "string") {
    const parsed = tryParseEmbeddedJson(value);
    if (parsed && typeof parsed === "object") out.push({ path, parsed, text: value });
    return out;
  }
  if (Array.isArray(value)) {
    value.forEach((item, index) => collectEmbeddedJsonCandidates(item, {
      path: `${path}[${index}]`,
      depth: depth + 1,
      maxDepth,
      maxCandidates,
      out,
    }));
    return out;
  }
  if (value && typeof value === "object") {
    Object.entries(value).forEach(([key, nested]) => {
      collectEmbeddedJsonCandidates(nested, {
        path: `${path}.${key}`,
        depth: depth + 1,
        maxDepth,
        maxCandidates,
        out,
      });
    });
  }
  return out;
}
