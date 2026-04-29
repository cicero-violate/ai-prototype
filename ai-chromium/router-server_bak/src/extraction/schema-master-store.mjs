import fs from "node:fs";
import path from "node:path";
import { mergeSchema } from "./schema-derivation.mjs";

const BASE_DIR = process.env.ARTIFACTS_DIR ?? path.resolve(process.cwd(), "artifacts");
const MASTER_DIR = path.join(BASE_DIR, "schemas");
const MASTER_INDEX_PATH = path.join(MASTER_DIR, "master-index.json");
const MASTER_SAMPLE_LIMIT = Number(process.env.MASTER_SCHEMA_SAMPLE_LIMIT ?? 200);

function readJson(filePath, fallback) {
  try {
    return JSON.parse(fs.readFileSync(filePath, "utf8"));
  } catch {
    return fallback;
  }
}

function writeJson(filePath, value) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

export function updateMasterSchema({ turnId, provider, schemaSnapshot }) {
  if (!schemaSnapshot || !Array.isArray(schemaSnapshot.docs)) return { updated: false, key_count: 0 };

  const index = readJson(MASTER_INDEX_PATH, {
    schema: "ai_chromium.master_schema_index.v1",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    keys: [],
  });
  const keyMap = new Map((index.keys ?? []).map((k) => [k.key, k]));

  for (const doc of schemaSnapshot.docs) {
    const key = String(doc?.json?.title ?? doc?.file ?? "").replace(/\.schema\.json$/, "");
    if (!key) continue;
    const schemaPath = path.join(MASTER_DIR, `${key}.schema.json`);
    const current = readJson(schemaPath, null);
    const nextSchema = doc?.json ?? {};
    const mergedCore = current ? mergeSchema(current, nextSchema) : nextSchema;
    const merged = {
      $schema: "https://json-schema.org/draft/2020-12/schema",
      title: key,
      ...mergedCore,
    };
    writeJson(schemaPath, merged);

    const prev = keyMap.get(key);
    const sampleDoc = (schemaSnapshot.samples ?? []).find((s) => s.file === `${key}.samples.json`);
    const masterSamplesPath = path.join(MASTER_DIR, `${key}.samples.json`);
    const existingSamples = readJson(masterSamplesPath, { key, count: 0, samples: [] });
    const mergedSamples = [
      ...(existingSamples.samples ?? []),
      ...((sampleDoc?.json?.samples ?? []).map((s) => ({
        observed_at: new Date().toISOString(),
        turn_id: turnId,
        provider,
        payload: s,
      }))),
    ];
    const trimmed = mergedSamples.slice(-MASTER_SAMPLE_LIMIT);
    writeJson(masterSamplesPath, {
      key,
      count: Number(prev?.count ?? 0) + Number((schemaSnapshot.index?.keys ?? []).find((k) => k.key === key)?.count ?? 0),
      sample_limit: MASTER_SAMPLE_LIMIT,
      samples: trimmed,
    });

    keyMap.set(key, {
      key,
      count: Number(prev?.count ?? 0) + Number((schemaSnapshot.index?.keys ?? []).find((k) => k.key === key)?.count ?? 0),
      schema_path: `${key}.schema.json`,
      samples_path: `${key}.samples.json`,
      first_seen_turn_id: prev?.first_seen_turn_id ?? turnId,
      last_seen_turn_id: turnId,
      last_provider: provider,
      updated_at: new Date().toISOString(),
    });
  }

  index.updated_at = new Date().toISOString();
  index.keys = [...keyMap.values()].sort((a, b) => a.key.localeCompare(b.key));
  writeJson(MASTER_INDEX_PATH, index);
  return { updated: true, key_count: index.keys.length, path: MASTER_INDEX_PATH };
}
