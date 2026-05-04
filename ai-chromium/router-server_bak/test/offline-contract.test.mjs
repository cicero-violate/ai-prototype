import test from "node:test";
import assert from "node:assert/strict";

import { makeReceiptStore } from "../src/capability/receipts.mjs";
import { assertSupports, CAPABILITIES } from "../src/provider/capability-contract.mjs";
import { isSensitivePath, redactRequest } from "../src/evidence/redaction.mjs";
import { classifyStructureOnly } from "../src/data/privacy-classifier.mjs";

test("redaction removes message content and file paths", () => {
  const redacted = redactRequest({
    model: "x",
    stream: false,
    messages: [{ role: "user", content: "secret text" }],
    browser: { provider: "chatgpt", files: [{ purpose: "source", path: "/tmp/a" }] },
  });

  assert.equal(redacted.messages[0].content_redacted, true);
  assert.equal(redacted.messages[0].content_length, 11);
  assert.equal(redacted.browser.files[0].path_redacted, true);
  assert.equal("content" in redacted.messages[0], false);
});

test("receipt store emits monotonic action receipts", () => {
  const store = makeReceiptStore("turn-1");
  const first = store.add({ provider: "p", capability: "send_message", status: "ok" });
  const second = store.add({ provider: "p", capability: "read_response", status: "ok" });

  assert.equal(first.seq, 1);
  assert.equal(second.seq, 2);
  assert.equal(store.all().length, 2);
  assert.equal(store.all()[0].schema, "ai_chromium.action_receipt.v1");
});

test("capability contract rejects unsupported actions", () => {
  const adapter = { provider: "local", capabilities: [CAPABILITIES.READ_RESPONSE] };

  assert.doesNotThrow(() => assertSupports(adapter, CAPABILITIES.READ_RESPONSE));
  assert.throws(() => assertSupports(adapter, CAPABILITIES.SEND_MESSAGE), /does not support/);
});

test("structure-only classification is stable", () => {
  assert.equal(isSensitivePath("Authorization"), true);
  assert.equal(isSensitivePath("public_title"), false);
  assert.equal(classifyStructureOnly(), "structure_only");
});