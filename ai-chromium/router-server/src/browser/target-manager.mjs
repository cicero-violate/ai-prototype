import http from "node:http";

function httpJson(method, urlPath, { host, port }) {
  return new Promise((resolve, reject) => {
    const req = http.request({ host, port, method, path: urlPath }, (res) => {
      const chunks = [];
      res.on("data", (chunk) => chunks.push(chunk));
      res.on("end", () => {
        const text = Buffer.concat(chunks).toString("utf8");
        if (res.statusCode < 200 || res.statusCode >= 300) {
          reject(new Error(`CDP HTTP ${method} ${urlPath} => ${res.statusCode}`));
          return;
        }
        try { resolve(JSON.parse(text)); }
        catch (err) { reject(new Error(`CDP HTTP ${method} ${urlPath} non-JSON: ${err.message}`)); }
      });
    });
    req.on("error", reject);
    req.end();
  });
}

export function makeTargetManager({ cdpHost, cdpPort }) {
  const o = { host: cdpHost, port: cdpPort };

  async function listTargets() {
    return httpJson("GET", "/json/list", o);
  }

  async function getVersion() {
    return httpJson("GET", "/json/version", o);
  }

  async function newTarget(url) {
    const urlPath = `/json/new?${encodeURIComponent(url)}`;
    try { return await httpJson("PUT", urlPath, o); }
    catch { return httpJson("GET", urlPath, o); }
  }

  async function activateTarget(id) {
    try { await httpJson("GET", `/json/activate/${encodeURIComponent(id)}`, o); }
    catch {}
  }

  async function findOrCreate({ providerUrl, reset }) {
    const origin = new URL(providerUrl).origin;
    const targets = await listTargets();
    let target = null;
    if (!reset) {
      target = targets.find((t) =>
        t.type === "page" &&
        t.webSocketDebuggerUrl &&
        String(t.url ?? "").startsWith(origin)
      );
    }
    if (!target) target = await newTarget(providerUrl);
    if (target?.id) await activateTarget(target.id);
    return target;
  }

  return { listTargets, getVersion, newTarget, activateTarget, findOrCreate };
}
