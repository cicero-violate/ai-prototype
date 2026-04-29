import fs from "node:fs";
import path from "node:path";

const BASE_DIR = process.env.ARTIFACTS_DIR ?? path.resolve(process.cwd(), "artifacts");
const RULES_DIR = path.join(BASE_DIR, "rules");
const RULES_FILE = path.join(RULES_DIR, "lifecycle.json");
const MIN_OBS_PROMOTE = Number(process.env.RULE_PROMOTE_MIN_OBS ?? 5);
const MAX_FAIL_RATE_PROMOTE = Number(process.env.RULE_PROMOTE_MAX_FAIL_RATE ?? 0.1);
const QUARANTINE_FAIL_RATE = Number(process.env.RULE_QUARANTINE_FAIL_RATE ?? 0.4);

function readJson(filePath, fallback) {
  try { return JSON.parse(fs.readFileSync(filePath, "utf8")); } catch { return fallback; }
}

function writeJson(filePath, value) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

export function updateRuleLifecycle({ provider, turnId, replayPass, ruleEvidence }) {
  const state = readJson(RULES_FILE, {
    schema: "ai_chromium.rule_lifecycle.v1",
    updated_at: null,
    providers: {},
  });
  state.providers[provider] = state.providers[provider] ?? { rules: {} };
  const rules = state.providers[provider].rules;

  const byRule = new Map();
  for (const r of ruleEvidence ?? []) {
    const id = String(r.rule_id ?? `${r.kind}:${r.path}`);
    byRule.set(id, (byRule.get(id) ?? 0) + 1);
  }

  for (const [ruleId, hits] of byRule.entries()) {
    const row = rules[ruleId] ?? {
      rule_id: ruleId,
      schema_key: null,
      path: null,
      observations: 0,
      failures: 0,
      replay_passes: 0,
      state: "candidate",
      first_seen_turn_id: turnId,
      last_seen_turn_id: turnId,
      updated_at: new Date().toISOString(),
    };
    const sample = (ruleEvidence ?? []).find((r) => String(r.rule_id ?? `${r.kind}:${r.path}`) === ruleId);
    row.schema_key = sample?.schema_key ?? row.schema_key;
    row.path = sample?.path ?? row.path;
    row.observations += hits;
    if (replayPass) row.replay_passes += hits;
    else row.failures += hits;
    row.last_seen_turn_id = turnId;
    row.updated_at = new Date().toISOString();
    const failRate = row.observations > 0 ? row.failures / row.observations : 0;
    if (row.observations >= MIN_OBS_PROMOTE && failRate <= MAX_FAIL_RATE_PROMOTE) row.state = "promoted";
    if (failRate >= QUARANTINE_FAIL_RATE) row.state = "quarantined";
    rules[ruleId] = row;
  }

  state.updated_at = new Date().toISOString();
  writeJson(RULES_FILE, state);
  return state.providers[provider];
}

