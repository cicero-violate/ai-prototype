import { CAPABILITIES } from "../provider/capability-contract.mjs";

const DEFAULT_CAPABILITIES = [CAPABILITIES.SEND_MESSAGE, CAPABILITIES.READ_RESPONSE];

export function buildPlan(adapter, request, turnId) {
  const requested = Array.isArray(request?.browser?.capabilities)
    ? request.browser.capabilities
    : DEFAULT_CAPABILITIES;

  const steps = [];
  for (const cap of requested) {
    if (!adapter.capabilities.includes(cap)) {
      throw new Error(`provider "${adapter.provider}" does not support capability "${cap}"`);
    }
    const step = { capability: cap };
    if (cap === CAPABILITIES.SELECT_PROJECT) {
      step.project_hint = request?.browser?.project_hint ?? null;
    }
    if (cap === CAPABILITIES.UPLOAD_FILE || cap === CAPABILITIES.ATTACH_ARTIFACT) {
      step.files = request?.browser?.files ?? [];
    }
    steps.push(step);
  }

  return {
    schema: "ai_chromium.capability_plan.v1",
    turn_id: turnId,
    provider: adapter.provider,
    steps,
  };
}
