import { CAPABILITIES } from "./capability-contract.mjs";
import { register } from "./registry.mjs";

export const chatgptGroupAdapter = {
  schema: "ai_chromium.provider_adapter.v1",
  provider: "chatgpt_group",
  originPatterns: ["https://chatgpt.com/gg/*"],
  capabilities: [CAPABILITIES.SEND_MESSAGE, CAPABILITIES.READ_RESPONSE],
  stability: "ui_drift_expected",
  requiresAuthenticatedProfile: true,
  providerUrl: "https://chatgpt.com/",
  isGemini: false,
  isGroupChat: true,

  matches(model, _request) {
    const name = String(model ?? "").toLowerCase();
    return name.includes("group");
  },

  async prepare(cdp) {
    const { buildPrepareGroupChatExpression } = await import("../browser/dom-actions.mjs");
    const result = await cdp.send("Runtime.evaluate", {
      expression: buildPrepareGroupChatExpression(),
      awaitPromise: true,
      returnByValue: true,
    });
    const value = result?.result?.value ?? { ok: true };
    if (!value.ok) return value;
    if (value.clicked_create) {
      await new Promise((r) => setTimeout(r, 900));
      const check = await cdp.send("Runtime.evaluate", {
        expression: `(() => {
          const editor = document.querySelector('div[contenteditable="true"]') || document.querySelector('textarea');
          const send = document.querySelector('button[data-testid="send-button"]') || document.querySelector('button[aria-label="Send prompt"]');
          return { ok: Boolean(editor && send && !send.disabled) };
        })()`,
        returnByValue: true,
      });
      return check?.result?.value ?? { ok: true };
    }
    return value;
  },
};

register(chatgptGroupAdapter);
