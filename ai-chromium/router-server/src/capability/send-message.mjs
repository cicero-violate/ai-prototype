import {
  buildWaitForEditorExpression,
  buildSubmitExpression,
} from "../browser/dom-actions.mjs";
import { CAPABILITIES } from "../provider/capability-contract.mjs";

export async function executeSendMessage({ cdp, prompt, adapter, receipts, targetId }) {
  const { isGemini, isGroupChat } = adapter;

  const readyResult = await cdp.send("Runtime.evaluate", {
    expression: buildWaitForEditorExpression(isGemini),
    awaitPromise: true,
    returnByValue: true,
  });

  if (!readyResult?.result?.value) {
    receipts.add({ provider: adapter.provider, capability: CAPABILITIES.SEND_MESSAGE, status: "failed", targetId });
    throw new Error("page editor not found; ensure browser is logged in and target page is loaded");
  }

  if (isGroupChat) {
    const prep = await adapter.prepare(cdp);
    if (!prep?.ok) {
      receipts.add({ provider: adapter.provider, capability: CAPABILITIES.SEND_MESSAGE, status: "failed", targetId });
      throw new Error(`group chat not ready: ${prep?.reason ?? "manual setup required"}`);
    }
  }

  const submitResult = await cdp.send("Runtime.evaluate", {
    expression: buildSubmitExpression(prompt, { isGemini, isGroupChat }),
    awaitPromise: true,
    returnByValue: true,
  });

  const submitValue = submitResult?.result?.value;
  if (!submitValue?.ok) {
    receipts.add({ provider: adapter.provider, capability: CAPABILITIES.SEND_MESSAGE, status: "failed", targetId });
    throw new Error(`prompt submit failed: ${JSON.stringify(submitValue)}`);
  }

  return receipts.add({
    provider: adapter.provider,
    capability: CAPABILITIES.SEND_MESSAGE,
    status: "completed",
    targetId,
  });
}
