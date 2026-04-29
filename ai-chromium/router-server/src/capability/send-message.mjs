import {
  buildWaitForEditorExpression,
  buildSubmitExpression,
} from "../browser/dom-actions.mjs";
import { CAPABILITIES } from "../provider/capability-contract.mjs";

export async function executeSendMessage({ cdp, prompt, adapter, receipts, targetId, request }) {
  const { isGemini, isGroupChat } = adapter;

  async function evalWithContextRetry(expression, attempts = 8) {
    let lastErr = null;
    for (let i = 0; i < attempts; i += 1) {
      try {
        return await cdp.send("Runtime.evaluate", {
          expression,
          awaitPromise: true,
          returnByValue: true,
        });
      } catch (err) {
        lastErr = err;
        const msg = String(err?.message ?? "");
        if (!msg.includes("Cannot find default execution context")) throw err;
        await new Promise((r) => setTimeout(r, 350));
      }
    }
    throw lastErr ?? new Error("execution context unavailable");
  }

  const readyResult = await evalWithContextRetry(buildWaitForEditorExpression(isGemini));

  if (!readyResult?.result?.value) {
    receipts.add({ provider: adapter.provider, capability: CAPABILITIES.SEND_MESSAGE, status: "failed", targetId });
    throw new Error("page editor not found; ensure browser is logged in and target page is loaded");
  }

  if (isGroupChat) {
    const prep = await adapter.prepare(cdp, { request });
    if (!prep?.ok) {
      receipts.add({ provider: adapter.provider, capability: CAPABILITIES.SEND_MESSAGE, status: "failed", targetId });
      throw new Error(`group chat not ready: ${prep?.reason ?? "manual setup required"}`);
    }
  }

  const submitResult = await evalWithContextRetry(buildSubmitExpression(prompt, { isGemini, isGroupChat }))
    .catch((err) => {
      const msg = String(err?.message ?? "");
      // Group chat: clicking send can destroy the JS context (ChatGPT React reloads the runner).
      // Treat as optimistic success — the editor-state check below verifies.
      if (isGroupChat && msg.includes("Promise was collected")) {
        return { result: { value: { ok: true, method: "send_context_destroyed" } } };
      }
      throw err;
    });

  const submitValue = submitResult?.result?.value;
  if (!submitValue?.ok) {
    receipts.add({ provider: adapter.provider, capability: CAPABILITIES.SEND_MESSAGE, status: "failed", targetId });
    throw new Error(`prompt submit failed: ${JSON.stringify(submitValue)}`);
  }

  // In some ChatGPT surfaces, synthetic DOM KeyboardEvents don't trigger send.
  // If text remains in editor after submit, press a real Enter via CDP Input.dispatchKeyEvent.
  if (isGroupChat) {
    const editorState = await evalWithContextRetry(`(() => {
      const editor = document.querySelector('div[contenteditable="true"]') || document.querySelector('textarea');
      const text = String(editor?.textContent || editor?.value || '');
      return { has_editor: Boolean(editor), text_len: text.trim().length };
    })()`);
    const textLen = Number(editorState?.result?.value?.text_len ?? 0);
    if (textLen > 0) {
      await evalWithContextRetry(`(() => {
        const editor = document.querySelector('div[contenteditable="true"]') || document.querySelector('textarea');
        if (editor) {
          editor.focus();
          const r = editor.getBoundingClientRect();
          const x = r.left + Math.min(10, Math.max(1, r.width / 2));
          const y = r.top + Math.min(10, Math.max(1, r.height / 2));
          try { editor.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true, clientX: x, clientY: y })); } catch {}
          try { editor.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, cancelable: true, clientX: x, clientY: y })); } catch {}
          try { editor.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true, clientX: x, clientY: y })); } catch {}
        }
        return Boolean(editor);
      })()`);
      await cdp.send("Input.dispatchKeyEvent", {
        type: "rawKeyDown",
        key: "Enter",
        code: "Enter",
        windowsVirtualKeyCode: 13,
        nativeVirtualKeyCode: 13,
        unmodifiedText: "\r",
        text: "\r",
      }).catch(() => {});
      await cdp.send("Input.dispatchKeyEvent", {
        type: "char",
        key: "Enter",
        code: "Enter",
        windowsVirtualKeyCode: 13,
        nativeVirtualKeyCode: 13,
        unmodifiedText: "\r",
        text: "\r",
      }).catch(() => {});
      await cdp.send("Input.dispatchKeyEvent", {
        type: "keyUp",
        key: "Enter",
        code: "Enter",
        windowsVirtualKeyCode: 13,
        nativeVirtualKeyCode: 13,
      }).catch(() => {});
      await new Promise((r) => setTimeout(r, 250));
    }
  }

  return receipts.add({
    provider: adapter.provider,
    capability: CAPABILITIES.SEND_MESSAGE,
    status: "completed",
    targetId,
  });
}
