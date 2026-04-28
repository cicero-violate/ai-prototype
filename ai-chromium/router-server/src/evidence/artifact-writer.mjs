import fs from "node:fs";
import path from "node:path";

const BASE_DIR = process.env.ARTIFACTS_DIR ?? path.resolve(process.cwd(), "artifacts");

function writeJson(filePath, value) {
  try {
    fs.mkdirSync(path.dirname(filePath), { recursive: true });
    fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
  } catch {}
}

function appendNdjson(filePath, records) {
  if (!records || records.length === 0) return;
  try {
    fs.mkdirSync(path.dirname(filePath), { recursive: true });
    const lines = records.map((r) => JSON.stringify(r)).join("\n") + "\n";
    fs.appendFileSync(filePath, lines);
  } catch {}
}

export function makeArtifactWriter(turnId) {
  const dir = path.join(BASE_DIR, "turns", turnId);

  try { fs.mkdirSync(dir, { recursive: true }); } catch {}

  function p(name) { return path.join(dir, name); }

  return {
    dir,
    writeManifest(record) { writeJson(p("manifest.json"), record); },
    writeRedactedRequest(record) { writeJson(p("request.redacted.json"), record); },
    writeResponse(record) { writeJson(p("response.json"), record); },
    writeCapabilityPlan(plan) { writeJson(p("capability-plan.json"), plan); },
    writeActionReceipts(receipts) { appendNdjson(p("action-receipts.ndjson"), receipts); },
  };
}
