export function scoreCapability({ provider, capability, receipts }) {
  const sampleCount = receipts.length;
  const successCount = receipts.filter((r) => r.status === "completed").length;
  const failureCount = receipts.filter((r) => r.status === "failed").length;
  const successRate = sampleCount > 0 ? successCount / sampleCount : 0;
  return {
    schema: "ai_chromium.capability_score.v1",
    provider,
    capability,
    strategy_id: "baseline_v1",
    sample_count: sampleCount,
    failure_count: failureCount,
    success_rate: Number(successRate.toFixed(4)),
    score: Number(successRate.toFixed(4)),
  };
}
