import { CAPABILITIES } from "../provider/capability-contract.mjs";

function buildProjectUrl({ projectHint, defaultProjectId }) {
  if (typeof projectHint === "string" && projectHint.startsWith("http")) return projectHint;
  if (typeof projectHint === "string" && projectHint.trim().length > 0) {
    return `https://chatgpt.com/g/${projectHint.trim()}/project?tab=sources`;
  }
  if (defaultProjectId) {
    return `https://chatgpt.com/g/${defaultProjectId}/project?tab=sources`;
  }
  return "https://chatgpt.com/";
}

export async function executeSelectProject({ cdp, adapter, receipts, targetId, projectHint, defaultProjectId }) {
  const targetUrl = buildProjectUrl({ projectHint, defaultProjectId });
  await cdp.send("Page.navigate", { url: targetUrl });
  const result = await cdp.send("Runtime.evaluate", {
    expression: "location.href",
    returnByValue: true,
  });

  const finalUrl = String(result?.result?.value ?? targetUrl);
  return receipts.add({
    provider: adapter.provider,
    capability: CAPABILITIES.SELECT_PROJECT,
    status: "completed",
    targetId,
    evidenceRefs: [finalUrl],
  });
}
