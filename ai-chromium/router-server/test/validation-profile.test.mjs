import assert from "node:assert/strict";
import test from "node:test";
import { classifyStep } from "../src/tools/report-validation.mjs";

test("validation classifier separates source failures from CDP environment blocks", () => {
  assert.equal(classifyStep({ status: 0, text: "ok" }), "pass");
  assert.equal(classifyStep({ status: 1, text: "connect ECONNREFUSED 127.0.0.1:9221" }), "fail_environment");
  assert.equal(classifyStep({ status: 1, text: "healthz status=502; body={code:'cdp_unavailable'}" }), "fail_environment");
  assert.equal(classifyStep({ status: 1, text: "assertion failed" }), "fail_source");
});