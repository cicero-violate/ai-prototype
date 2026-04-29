export function buildFeatureVector({ datasetId, provider, capability, receipts, contentLength }) {
  const successCount = receipts.filter((r) => r.status === "completed").length;
  const failureCount = receipts.filter((r) => r.status === "failed").length;
  return {
    schema: "ai_chromium.feature_vector.v1",
    feature_id: `feat_${Date.now().toString(36)}`,
    dataset_id: datasetId,
    provider,
    capability,
    features: {
      action_success_count: successCount,
      action_failure_count: failureCount,
      response_non_empty: contentLength > 0,
      content_length: contentLength,
    },
  };
}
