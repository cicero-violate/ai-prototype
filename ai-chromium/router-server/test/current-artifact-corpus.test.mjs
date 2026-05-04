import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

const ARTIFACTS_DIR = fs.mkdtempSync(path.join(os.tmpdir(), "router-current-artifacts-"));
process.env.ARTIFACTS_DIR = ARTIFACTS_DIR;

const { makeArtifactWriter } = await import("../src/evidence/artifact-writer.mjs");
const { validateTurnArtifacts } = await import("../src/tools/validate-turn-artifacts.mjs");

test("current generated artifact corpus validates manifest, replay, evaluation, and NDJSON", () => {
  const turnId = "turn_current_gate";
  const writer = makeArtifactWriter(turnId);
  writer.writeManifest({ schema: "ai_chromium.turn_manifest.v1", turn_id: turnId });
  writer.writeReplay({ schema: "ai_chromium.replay_turn.v1", turn_id: turnId, replay_match: true });
  writer.writeEvaluation({
    schema: "ai_chromium.evaluation.v1",
    turn_id: turnId,
    replay_match: true,
    redaction_pass: true,
    quality: 1,
  });
  writer.writeActionReceipts([{ schema: "ai_chromium.receipt.v1", turn_id: turnId, status: "ok" }]);

  const result = validateTurnArtifacts(ARTIFACTS_DIR, { scope: "current" });
  assert.equal(result.scope, "current");
  assert.equal(result.turn_count, 1);
  assert.equal(result.pass_count, 1);
  assert.equal(result.fail_count, 0);
  assert.equal(result.pass, true);
});
