export function compareProviderFromScore(score) {
  return {
    schema: "ai_chromium.provider_comparison.v1",
    provider: score.provider,
    capability: score.capability,
    rank_signal: score.score,
    rationale: score.score >= 0.8 ? "preferred" : "fallback",
  };
}
