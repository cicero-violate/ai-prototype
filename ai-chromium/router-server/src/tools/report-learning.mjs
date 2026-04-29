#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

const base = process.env.ARTIFACTS_DIR ?? path.resolve(process.cwd(), "artifacts");
const schemasDir = path.join(base, "schemas");
const rulesFile = path.join(base, "rules", "lifecycle.json");
const policyFile = path.join(base, "data", "policy", "policy.current.json");

function readJson(filePath, fallback) {
  try { return JSON.parse(fs.readFileSync(filePath, "utf8")); } catch { return fallback; }
}

const master = readJson(path.join(schemasDir, "master-index.json"), { keys: [] });
const rules = readJson(rulesFile, { providers: {} });
const policy = readJson(policyFile, {});

const providerSummaries = Object.entries(rules.providers ?? {}).map(([provider, v]) => {
  const rows = Object.values(v.rules ?? {});
  const counts = rows.reduce((acc, r) => {
    acc.total += 1;
    acc[r.state] = (acc[r.state] ?? 0) + 1;
    return acc;
  }, { total: 0, candidate: 0, promoted: 0, quarantined: 0 });
  return { provider, ...counts };
});

const out = {
  schema: "ai_chromium.learning_report.v1",
  generated_at: new Date().toISOString(),
  artifacts_dir: base,
  master_schema_keys: master.keys?.length ?? 0,
  master_keys: (master.keys ?? []).map((k) => ({
    key: k.key,
    count: k.count,
    schema_path: k.schema_path,
    samples_path: k.samples_path ?? null,
  })),
  provider_rule_lifecycle: providerSummaries,
  route_policy: policy.route_policy ?? {},
};

process.stdout.write(`${JSON.stringify(out, null, 2)}\n`);
