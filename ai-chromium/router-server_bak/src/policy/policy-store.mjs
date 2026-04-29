import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";

const BASE_DIR = process.env.ARTIFACTS_DIR ?? path.resolve(process.cwd(), "artifacts");
const POLICY_DIR = path.join(BASE_DIR, "data", "policy");
const POLICY_FILE = path.join(POLICY_DIR, "policy.current.json");

function digest(value) {
  return `sha256:${crypto.createHash("sha256").update(JSON.stringify(value)).digest("hex")}`;
}

export function readPolicy() {
  try {
    return JSON.parse(fs.readFileSync(POLICY_FILE, "utf8"));
  } catch {
    return {
      route_policy: {},
      selector_policy: {},
      upload_policy: {},
      extraction_policy: {},
      recovery_policy: {},
    };
  }
}

export function writePolicy(policy) {
  fs.mkdirSync(POLICY_DIR, { recursive: true });
  fs.writeFileSync(POLICY_FILE, `${JSON.stringify(policy, null, 2)}\n`);
}

export function makePolicySnapshot(policy) {
  const version = `policy_${Date.now().toString(36)}`;
  return {
    schema: "ai_chromium.policy_snapshot.v1",
    policy_version: version,
    route_policy_digest: digest(policy.route_policy ?? {}),
    selector_policy_digest: digest(policy.selector_policy ?? {}),
    upload_policy_digest: digest(policy.upload_policy ?? {}),
    extraction_policy_digest: digest(policy.extraction_policy ?? {}),
    recovery_policy_digest: digest(policy.recovery_policy ?? {}),
  };
}
