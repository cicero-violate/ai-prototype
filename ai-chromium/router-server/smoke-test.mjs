#!/usr/bin/env node
// Smoke test for the local browser API.

const res = await fetch("http://127.0.0.1:8080/v1/chat/completions", {
  method: "POST",
  headers: { "content-type": "application/json" },
  body: JSON.stringify({
    model: "chatgpt-browser",
    messages: [{ role: "user", content: "Say hello in one sentence." }],
    stream: false,
    browser: { reset_chat: true, idle_ms: 2500, max_ms: 120000 },
  }),
});

console.log(await res.text());
