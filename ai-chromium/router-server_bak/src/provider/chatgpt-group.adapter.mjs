import { CAPABILITIES } from "./capability-contract.mjs";
import { register } from "./registry.mjs";

export const chatgptGroupAdapter = {
  schema: "ai_chromium.provider_adapter.v1",
  provider: "chatgpt_group",
  originPatterns: ["https://chatgpt.com/*"],
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

  async prepare(cdp, { request } = {}) {
    const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
    const { buildPrepareGroupChatExpression } = await import("../browser/dom-actions.mjs");

    const createNew = Boolean(request?.browser?.create_group_chat ?? false);

    if (!createNew) {
      const first = await cdp.send("Runtime.evaluate", {
        expression: buildPrepareGroupChatExpression(),
        awaitPromise: true,
        returnByValue: true,
      });
      const value = first?.result?.value ?? { ok: true };
      if (!value?.ok) return value;
      if (value.clicked_create) {
        await new Promise((r) => setTimeout(r, 900));
        const second = await cdp.send("Runtime.evaluate", {
          expression: `(() => {
            const editor = document.querySelector('div[contenteditable="true"]') || document.querySelector('textarea');
            const sendBtn = document.querySelector('button[data-testid="send-button"]') ||
              document.querySelector('button[aria-label="Send prompt"]') ||
              document.querySelector('button[aria-label="Send message"]');
            return { ok: Boolean(editor && sendBtn && !sendBtn.disabled), editor: Boolean(editor), send_enabled: Boolean(sendBtn && !sendBtn.disabled) };
          })()`,
          returnByValue: true,
        });
        return second?.result?.value ?? { ok: true };
      }
      return value;
    }

    // Force group-chat creation flow from ChatGPT home for this provider.
    await cdp.send("Page.navigate", { url: "https://chatgpt.com/" });
    await sleep(1800);

    const step = await cdp.send("Runtime.evaluate", {
      expression: `
(async () => {
  const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
  const norm = (s) => (s || '').replace(/\\s+/g, ' ').trim().toLowerCase();
  const visible = (el) => {
    const st = getComputedStyle(el);
    const r = el.getBoundingClientRect();
    return st.display !== 'none' && st.visibility !== 'hidden' && r.width > 0 && r.height > 0;
  };
  const clickish = (el) => {
    try { el.scrollIntoView({ block: 'center', inline: 'center' }); } catch {}
    const r = el.getBoundingClientRect();
    const cx = r.left + r.width / 2;
    const cy = r.top + r.height / 2;
    const t = document.elementFromPoint(cx, cy) || el;
    const fire = (node, type) => node.dispatchEvent(new MouseEvent(type, { bubbles: true, cancelable: true, clientX: cx, clientY: cy }));
    fire(t, 'mousedown'); fire(t, 'mouseup'); fire(t, 'click');
    try { el.click(); } catch {}
  };

  const start = document.querySelector('button[aria-label="Start a group chat"]');
  if (!start) return { ok: false, reason: "start_group_chat_button_missing", href: location.href };
  clickish(start);

  const deadline = Date.now() + 10000;
  while (Date.now() < deadline) {
    await sleep(150);
    const controls = Array.from(document.querySelectorAll('button,[role="button"],a')).filter(visible);
    const startGroup = controls.find((el) => norm(el.innerText || el.textContent).includes('start group chat'));
    if (startGroup) {
      clickish(startGroup);
      await sleep(300);
    }
    const hrefNow = String(location.href || '');
    if (hrefNow.includes('/gg/')) {
      const closeCandidates = Array.from(document.querySelectorAll('button,[role="button"],a')).filter(visible);
      const closeBtn = closeCandidates.find((el) => {
        const text = norm(el.innerText || el.textContent);
        const aria = norm(el.getAttribute('aria-label') || '');
        const all = (text + ' ' + aria).trim();
        return all === 'close' || all.includes('close') || all.includes('dismiss') || all === 'x';
      });
      if (closeBtn) {
        clickish(closeBtn);
      } else {
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', bubbles: true }));
        document.dispatchEvent(new KeyboardEvent('keyup', { key: 'Escape', code: 'Escape', bubbles: true }));
      }
      return { ok: true, href: hrefNow };
    }
    const link = controls.find((el) => {
      if (el.tagName !== 'A') return false;
      const href = el.getAttribute('href') || '';
      return href.startsWith('/gg/');
    });
    if (link) {
      const closeCandidates = Array.from(document.querySelectorAll('button,[role="button"],a')).filter(visible);
      const closeBtn = closeCandidates.find((el) => {
        const text = norm(el.innerText || el.textContent);
        const aria = norm(el.getAttribute('aria-label') || '');
        const all = (text + ' ' + aria).trim();
        return all === 'close' || all.includes('close') || all.includes('dismiss') || all === 'x';
      });
      if (closeBtn) {
        clickish(closeBtn);
      } else {
        document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', bubbles: true }));
        document.dispatchEvent(new KeyboardEvent('keyup', { key: 'Escape', code: 'Escape', bubbles: true }));
      }
      return { ok: true, href: 'https://chatgpt.com' + (link.getAttribute('href') || '') };
    }
  }
  return { ok: false, reason: "group_chat_link_not_found", href: location.href };
})()`,
      awaitPromise: true,
      returnByValue: true,
    });
    const created = step?.result?.value ?? { ok: false, reason: "group_chat_create_eval_failed" };
    if (!created.ok) return created;

    if (typeof created.href === "string" && created.href.startsWith("https://chatgpt.com/gg/")) {
      await cdp.send("Page.navigate", { url: created.href });
      await sleep(1200);
    }

    const first = await cdp.send("Runtime.evaluate", {
      expression: buildPrepareGroupChatExpression(),
      awaitPromise: true,
      returnByValue: true,
    });
    const value = first?.result?.value ?? { ok: true };
    if (!value?.ok) return value;
    if (value.clicked_create) {
      await new Promise((r) => setTimeout(r, 900));
      const second = await cdp.send("Runtime.evaluate", {
        expression: `(() => {
          const editor = document.querySelector('div[contenteditable="true"]') || document.querySelector('textarea');
          const sendBtn = document.querySelector('button[data-testid="send-button"]') ||
            document.querySelector('button[aria-label="Send prompt"]') ||
            document.querySelector('button[aria-label="Send message"]');
          return { ok: Boolean(editor && sendBtn && !sendBtn.disabled), editor: Boolean(editor), send_enabled: Boolean(sendBtn && !sendBtn.disabled) };
        })()`,
        returnByValue: true,
      });
      return second?.result?.value ?? { ok: true };
    }
    return value;
  },
};

register(chatgptGroupAdapter);
