import { spawn } from "node:child_process";

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

export function createUploadActionHandler({ parseJsonObject, readBody, errorResponse, jsonResponse, cdpHost, cdpPort, defaultProjectId, uploadScript }) {
  return async function handleUploadAction(req, res) {
    let body;
    try { body = parseJsonObject(await readBody(req)); }
    catch (err) { errorResponse(res, 400, err.message, { code: "invalid_request" }); return; }

    const projectId = String(body.project_id ?? defaultProjectId);
    const targetUrl = String(body.target_url ?? `https://chatgpt.com/g/${projectId}/project?tab=sources`);
    const match = String(body.match ?? targetUrl);
    const cdp = `http://${cdpHost}:${cdpPort}`;

    const args = [uploadScript, "--cdp", cdp, "--match", match];

    const buildTar = Boolean(body.build_tar ?? !body.file);
    if (buildTar) {
      const tarScript = String(body.tar_script ?? "");
      const tarOutput = String(body.tar_output ?? "router-server.tar.gz");
      if (!tarScript) { errorResponse(res, 400, "tar_script required when build_tar=true", { code: "invalid_request" }); return; }
      args.push("--build-tar", "--tar-script", tarScript, "--tar-output", tarOutput);
    } else if (typeof body.file === "string") {
      args.push("--file", body.file);
    } else {
      errorResponse(res, 400, "either file or build_tar=true is required", { code: "invalid_request" });
      return;
    }

    args.push(
      "--open-target-if-missing", "--target-url", targetUrl,
      "--target-wait-timeout-sec", String(body.target_wait_timeout_sec ?? 45),
      "--open-sources-flow", "--scope", "sources", "--force-upload",
      "--confirm-loaded",
      "--confirm-timeout-sec", String(body.confirm_timeout_sec ?? 120),
      "--confirm-settle-sec", String(body.confirm_settle_sec ?? 3),
    );

    const result = await runCommand("python3", args);
    if (result.code !== 0) {
      errorResponse(res, 502, "upload action failed", {
        code: "upload_failed",
        stdout: result.stdout.slice(-4000),
        stderr: result.stderr.slice(-4000),
      });
      return;
    }

    jsonResponse(res, 200, { ok: true, action: "upload", cdp, target_url: targetUrl, stdout: result.stdout.slice(-4000) });
  };
}
