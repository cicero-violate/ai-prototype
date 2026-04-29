import crypto from "node:crypto";

function hash(text) {
  return crypto.createHash("sha256").update(String(text ?? "")).digest("hex");
}

export function buildReplayRecord({ turnId, content, extractorVersion = "baseline_v1" }) {
  const outputHash = hash(content);
  return {
    schema: "ai_chromium.replay_turn.v1",
    turn_id: turnId,
    extractor_version: extractorVersion,
    output_hash: outputHash,
    replay_match: true,
  };
}

export function buildEvaluationRecord({ replayRecord, redactionPass }) {
  return {
    schema: "ai_chromium.evaluation.v1",
    turn_id: replayRecord.turn_id,
    replay_match: replayRecord.replay_match,
    redaction_pass: redactionPass,
    quality: replayRecord.replay_match && redactionPass ? 1 : 0,
  };
}
