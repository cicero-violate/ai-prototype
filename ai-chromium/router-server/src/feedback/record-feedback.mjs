export function buildFeedbackRecord({ policyVersion, turnId, provider, capability, measuredDelta, regression }) {
  return {
    schema: "ai_chromium.feedback_record.v1",
    feedback_id: `fb_${Date.now().toString(36)}`,
    policy_version: policyVersion,
    turn_id: turnId,
    provider,
    capability,
    measured_delta: measuredDelta,
    regression,
  };
}
