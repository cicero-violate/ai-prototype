import { spawn } from "node:child_process";
import { CAPABILITIES } from "../provider/capability-contract.mjs";

function runCommand(command, args) {
  return new Promise((resolve) => {
    const child = spawn(command, args, { stdio: ["ignore", "pipe", "pipe"] });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (d) => { stdout += d.toString("utf8"); });
    child.stderr.on("data", (d) => { stderr += d.toString("utf8"); });
    child.on("close", (code) => resolve({ code: code ?? 1, stdout, stderr }));
    child.on("error", (err) => resolve({ code: 1, stdout, stderr: `${stderr}\n${String(err?.message ?? err)}`.trim() }));
  });
}

export async function executeUploadFile({
  adapter,
  receipts,
  targetId,
  cdpHost,
  cdpPort,
  uploadScript,
  defaultProjectId,
  projectHint,
  files,
}) {
  if (!uploadScript) {
    throw new Error("upload_file requested but CDP_UPLOAD_SCRIPT is not configured");
  }

  const first = Array.isArray(files) && files.length > 0 ? files[0] : null;
  if (!first?.path) throw new Error("upload_file requested but no browser.files[].path provided");

  const projectId = String(projectHint ?? defaultProjectId ?? "").trim();
  const targetUrl = projectId
    ? `https://chatgpt.com/g/${projectId}/project?tab=sources`
    : "https://chatgpt.com/";
  const cdp = `http://${cdpHost}:${cdpPort}`;
  const args = [
    uploadScript,
    "--cdp", cdp,
    "--match", targetUrl,
    "--file", String(first.path),
    "--open-target-if-missing", "--target-url", targetUrl,
    "--target-wait-timeout-sec", "45",
    "--open-sources-flow", "--scope", "sources", "--force-upload",
    "--confirm-loaded",
    "--confirm-timeout-sec", "120",
    "--confirm-settle-sec", "3",
  ];

  const result = await runCommand("python3", args);
  if (result.code !== 0) {
    receipts.add({
      provider: adapter.provider,
      capability: CAPABILITIES.UPLOAD_FILE,
      status: "failed",
      targetId,
      evidenceRefs: [result.stderr.slice(-500)],
    });
    throw new Error(`upload failed: ${result.stderr.slice(-500) || "unknown error"}`);
  }

  return receipts.add({
    provider: adapter.provider,
    capability: CAPABILITIES.UPLOAD_FILE,
    status: "completed",
    targetId,
    evidenceRefs: [result.stdout.slice(-500)],
  });
}
