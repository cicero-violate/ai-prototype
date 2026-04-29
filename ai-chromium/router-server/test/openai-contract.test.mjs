import assert from "node:assert/strict";
import test from "node:test";

import {
  estimateUsage,
  makeChatCompletionResponse,
  makeOpenAiChunk,
  messagesToPrompt,
  validateChatCompletionRequest,
} from "../src/api/openai-contract.mjs";
import { redactRequest } from "../src/evidence/redaction.mjs";
import { redactPassFromRequest } from "../src/data/privacy-classifier.mjs";
import { buildReplayRecord, replayExtractContent } from "../src/replay/replay-turn.mjs";

test("messagesToPrompt preserves system/developer intent instead of blind role flattening", () => {
  const prompt = messagesToPrompt([
    { role: "system", content: "Follow the contract." },
    { role: "developer", content: "Return terse output." },
    { role: "user", content: [{ type: "text", text: "Say OK." }] },
  ]);

  assert.match(prompt, /System\/developer instructions/);
  assert.match(prompt, /\[system\] Follow the contract/);
  assert.match(prompt, /\[developer\] Return terse output/);
  assert.match(prompt, /Say OK/);
});

test("unsupported tool calling is rejected explicitly", () => {
  const result = validateChatCompletionRequest({
    model: "chatgpt-cdp",
    messages: [{ role: "user", content: "call a tool" }],
    tools: [{ type: "function", function: { name: "x" } }],
  });

  assert.equal(result.ok, false);
  assert.match(result.errors.join("\n"), /tools/);
});

test("usage estimates are deterministic and non-zero for non-empty content", () => {
  const usage = estimateUsage({ prompt: "hello world", completion: "ok" });
  assert.ok(usage.prompt_tokens > 0);
  assert.ok(usage.completion_tokens > 0);
  assert.equal(usage.total_tokens, usage.prompt_tokens + usage.completion_tokens);
});

test("chat completion envelope includes strict client compatibility fields", () => {
  const response = makeChatCompletionResponse({
    id: "chatcmpl-test",
    model: "chatgpt-cdp",
    content: "OK",
    usage: { prompt_tokens: 2, completion_tokens: 1, total_tokens: 3 },
  });

  assert.equal(response.object, "chat.completion");
  assert.equal(response.choices[0].message.role, "assistant");
  assert.equal(response.choices[0].message.refusal, null);
  assert.deepEqual(response.choices[0].message.annotations, []);
  assert.equal(response.usage.total_tokens, 3);
  assert.equal(response.system_fingerprint, "browser-cdp-router");
});

test("streaming first chunk can carry assistant role", () => {
  const chunk = makeOpenAiChunk({ id: "chatcmpl-test", model: "chatgpt-cdp", role: "assistant" });
  assert.equal(chunk.object, "chat.completion.chunk");
  assert.equal(chunk.choices[0].delta.role, "assistant");
  assert.equal("content" in chunk.choices[0].delta, false);
});

test("redaction classifier agrees with redactor schema", () => {
  const redacted = redactRequest({
    model: "chatgpt-cdp",
    messages: [{ role: "user", content: "secret prompt" }],
    browser: { files: [{ path: "/tmp/private.txt", purpose: "assistants" }] },
  });

  assert.equal(redacted.messages_redacted, true);
  assert.equal(redacted.browser.files_redacted, true);
  assert.equal(redactPassFromRequest(redacted), true);
  assert.equal("content" in redacted.messages[0], false);
  assert.equal("path" in redacted.browser.files[0], false);
});

test("replay record compares re-extracted assistant content instead of hardcoding success", () => {
  const rawCapture = [{
    parsed: {
      v: {
        message: {
          author: { role: "assistant" },
          content: { parts: ["OK"] },
        },
      },
    },
    meta: { author_role: "assistant", is_visually_hidden: false },
  }];

  assert.equal(replayExtractContent(rawCapture), "OK");

  const pass = buildReplayRecord({ turnId: "turn_test", content: "OK", rawCapture });
  assert.equal(pass.replay_match, true);

  const fail = buildReplayRecord({ turnId: "turn_test", content: "NO", rawCapture });
  assert.equal(fail.replay_match, false);
});