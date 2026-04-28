import { CAPABILITIES } from "./capability-contract.mjs";
import { register } from "./registry.mjs";

export const geminiPrivateAdapter = {
  schema: "ai_chromium.provider_adapter.v1",
  provider: "gemini_private",
  originPatterns: ["https://gemini.google.com/*"],
  capabilities: [CAPABILITIES.SEND_MESSAGE, CAPABILITIES.READ_RESPONSE, CAPABILITIES.UPLOAD_FILE],
  stability: "ui_drift_expected",
  requiresAuthenticatedProfile: true,
  providerUrl: "https://gemini.google.com/app",
  isGemini: true,
  isGroupChat: false,

  matches(model, _request) {
    return String(model ?? "").toLowerCase().includes("gemini");
  },

  async prepare(_cdp) {
    return { ok: true };
  },
};

register(geminiPrivateAdapter);
