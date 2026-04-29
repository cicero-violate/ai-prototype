import crypto from "node:crypto";

export function makeDatasetId({ turnId, provider, capability }) {
  const raw = `${turnId}:${provider}:${capability}`;
  const digest = crypto.createHash("sha256").update(raw).digest("hex").slice(0, 12);
  return `dataset_${digest}`;
}

export function buildDatasetRecord({ turnId, provider, capability, privacyClass, redactionPass, recordCount }) {
  return {
    schema: "ai_chromium.dataset_record.v1",
    dataset_id: makeDatasetId({ turnId, provider, capability }),
    source_turn_ids: [turnId],
    provider,
    capability,
    privacy_class: privacyClass,
    record_count: recordCount,
    redaction_pass: redactionPass,
    retention_class: "derived_structural",
  };
}
