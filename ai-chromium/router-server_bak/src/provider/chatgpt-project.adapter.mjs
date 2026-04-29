import { CAPABILITIES } from "./capability-contract.mjs";
import { register } from "./registry.mjs";

export const chatgptProjectAdapter = {
  schema: "ai_chromium.provider_adapter.v1",
  provider: "chatgpt_project",
  originPatterns: ["https://chatgpt.com/*"],
  capabilities: [
    CAPABILITIES.SELECT_PROJECT,
    CAPABILITIES.UPLOAD_FILE,
    CAPABILITIES.ATTACH_ARTIFACT,
    CAPABILITIES.SEND_MESSAGE,
    CAPABILITIES.READ_RESPONSE,
  ],
  stability: "ui_drift_expected",
  requiresAuthenticatedProfile: true,
  providerUrl: "https://chatgpt.com/",
  isGemini: false,
  isGroupChat: false,

  matches(model, _request) {
    const name = String(model ?? "").toLowerCase();
    return name.includes("project") && !name.includes("gemini");
  },

  async prepare(_cdp) {
    return { ok: true };
  },
};

register(chatgptProjectAdapter);
