export function scoreRulesByProvider({ provider, ruleEvidence, replayPass }) {
  const rules = {};
  for (const r of ruleEvidence ?? []) {
    const id = String(r.rule_id ?? `${r.kind}:${r.path}`);
    const row = rules[id] ?? {
      schema: "ai_chromium.rule_score.v1",
      provider,
      rule_id: id,
      schema_key: r.schema_key ?? null,
      path: r.path ?? null,
      sample_count: 0,
      replay_pass_count: 0,
      replay_fail_count: 0,
      score: 0,
    };
    row.sample_count += 1;
    if (replayPass) row.replay_pass_count += 1;
    else row.replay_fail_count += 1;
    row.score = Number((row.replay_pass_count / row.sample_count).toFixed(4));
    rules[id] = row;
  }
  return Object.values(rules);
}

