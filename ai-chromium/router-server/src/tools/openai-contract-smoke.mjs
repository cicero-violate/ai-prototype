#!/usr/bin/env node
import assert from "node:assert/strict";
import {
  estimateUsage,
  makeChatCompletionResponse,
  makeOpenAiChunk,
  messagesToPrompt,
  validateChatCompletionRequest,
} from "../api/openai-contract.mjs";

const request = {
  model: "chatgpt-cdp",
  messages: [
    { role: "system", content: "Be precise." },
    { role: "user", content: "Say OK." },
  ],
  stream: false,
};

const validation = validateChatCompletionRequest(request);
assert.equal(validation.ok, true);

const prompt = messagesToPrompt(request.messages);
assert.match(prompt, /System\/developer instructions/);
assert.match(prompt, /Say OK/);

const usage = estimateUsage({ prompt, completion: "OK" });
assert.equal(usage.total_tokens, usage.prompt_tokens + usage.completion_tokens);
assert.ok(usage.total_tokens > 0);

const response = makeChatCompletionResponse({
  id: "chatcmpl-test",
  model: request.model,
  content: "OK",
  usage,
});
assert.equal(response.object, "chat.completion");
assert.equal(response.choices[0].message.role, "assistant");
assert.equal(response.usage.total_tokens, usage.total_tokens);
assert.equal(response.system_fingerprint, "browser-cdp-router");

const firstChunk = makeOpenAiChunk({ id: "chatcmpl-test", model: request.model, role: "assistant" });
assert.equal(firstChunk.object, "chat.completion.chunk");
assert.equal(firstChunk.choices[0].delta.role, "assistant");

console.log("openai_contract_smoke_ok");