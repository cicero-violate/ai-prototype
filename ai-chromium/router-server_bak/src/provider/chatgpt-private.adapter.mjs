import { CAPABILITIES } from "./capability-contract.mjs";
import { register } from "./registry.mjs";

export const chatgptPrivateAdapter = {
  schema: "ai_chromium.provider_adapter.v1",
  provider: "chatgpt_private",
  originPatterns: ["https://chatgpt.com/*"],
  capabilities: [CAPABILITIES.SEND_MESSAGE, CAPABILITIES.READ_RESPONSE, CAPABILITIES.UPLOAD_FILE],
  stability: "ui_drift_expected",
  requiresAuthenticatedProfile: true,
  providerUrl: "https://chatgpt.com/",
  isGemini: false,
  isGroupChat: false,

  matches(model, _request) {
    const name = String(model ?? "").toLowerCase();
    return (name.includes("chatgpt") || name.includes("cdp") || name.includes("browser")) &&
      !name.includes("gemini") && !name.includes("group") && !name.includes("project");
  },

  async prepare(_cdp) {
    return { ok: true };
  },
};

register(chatgptPrivateAdapter);
