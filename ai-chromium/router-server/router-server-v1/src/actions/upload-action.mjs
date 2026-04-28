import { spawn } from "node:child_process";

export function createUploadActionHandler({
  parseJsonObject,
  readBody,
  errorResponse,
  jsonResponse,
  cdpHost,
  cdpPort,
  defaultProjectId,
  uploadScript,
}) {
  function readUploadRequest(body) {
    const projectId = String(body.project_id ?? defaultProjectId);
    const targetUrl = String(
      body.target_url ?? `https://chatgpt.com/g/${projectId}/project?tab=sources`,
    );
    const match = String(body.match ?? targetUrl);
    const file = typeof body.file === "string" ? body.file : null;
    return {
      buildTar: Boolean(body.build_tar ?? !file),
      file,
      tarScript: String(
        body.tar_script
        ?? "/mnt/data/canon-mini-agent-extracted/canon-mini-agent/prototype/ai/ai-chromium/tar.sh",
      ),
      tarOutput: String(body.tar_output ?? "router-server.tar.gz"),
      cdp: String(body.cdp ?? `http://${cdpHost}:${cdpPort}`),
      match,
      targetUrl,
      targetWaitSec: Number(body.target_wait_timeout_sec ?? 45),
      confirmTimeoutSec: Number(body.confirm_timeout_sec ?? 120),
      confirmSettleSec: Number(body.confirm_settle_sec ?? 3),
    };
  }

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

  return async function handleUploadAction(req, res) {
    let body;
    try {
      body = parseJsonObject(await readBody(req));
    } catch (err) {
      errorResponse(res, 400, err.message, { code: "invalid_request" });
      return;
    }

    const cfg = readUploadRequest(body);
    const args = [uploadScript, "--cdp", cfg.cdp, "--match", cfg.match];
    if (cfg.buildTar) {
      args.push("--build-tar", "--tar-script", cfg.tarScript, "--tar-output", cfg.tarOutput);
    } else if (cfg.file) {
      args.push("--file", cfg.file);
    } else {
      errorResponse(res, 400, "either file or build_tar=true is required", { code: "invalid_request" });
      return;
    }

    args.push(
      "--open-target-if-missing",
      "--target-url", cfg.targetUrl,
      "--target-wait-timeout-sec", String(cfg.targetWaitSec),
      "--open-sources-flow",
      "--scope", "sources",
      "--force-upload",
      "--confirm-loaded",
      "--confirm-timeout-sec", String(cfg.confirmTimeoutSec),
      "--confirm-settle-sec", String(cfg.confirmSettleSec),
    );

    const result = await runCommand("python3", args);
    if (result.code !== 0) {
      errorResponse(res, 502, "upload action failed", {
        code: "upload_failed",
        stdout: result.stdout.slice(-4000),
        stderr: result.stderr.slice(-4000),
        command: ["python3", ...args],
      });
      return;
    }

    jsonResponse(res, 200, {
      ok: true,
      action: "upload",
      cdp: cfg.cdp,
      target_url: cfg.targetUrl,
      script: uploadScript,
      stdout: result.stdout.slice(-4000),
    });
  };
}
