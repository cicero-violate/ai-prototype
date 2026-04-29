const HARD_UNSUPPORTED_PARAMS = new Map([
  ["tools", "tool calling is not implemented by the browser router"],
  ["tool_choice", "tool calling is not implemented by the browser router"],
  ["functions", "legacy function calling is not implemented by the browser router"],
  ["function_call", "legacy function calling is not implemented by the browser router"],
]);

const SOFT_IGNORED_PARAMS = new Set([
  "temperature",
  "top_p",
  "max_tokens",
  "max_completion_tokens",
  "presence_penalty",
  "frequency_penalty",
  "logit_bias",
  "seed",
  "stop",
  "user",
  "metadata",
  "store",
  "parallel_tool_calls",
  "logprobs",
  "top_logprobs",
]);

function isPlainObject(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

export function normalizeMessageContent(content) {
  if (typeof content === "string") return content;
  if (Array.isArray(content)) {
    return content.map((part) => {
      if (typeof part === "string") return part;
      if (!isPlainObject(part)) return "";
      if (typeof part.text === "string") return part.text;
      if (typeof part.content === "string") return part.content;
      if (part.type === "input_text" && typeof part.text === "string") return part.text;
      if (part.type === "text" && typeof part.text === "string") return part.text;
      if (part.type === "image_url") return "[image_url omitted by browser-router prompt bridge]";
      if (part.type === "input_image") return "[input_image omitted by browser-router prompt bridge]";
      return "";
    }).filter(Boolean).join("\n");
  }
  if (content == null) return "";
  return String(content);
}

export function messagesToPrompt(messages) {
  if (!Array.isArray(messages)) return "";
  const system = [];
  const conversation = [];

  for (const msg of messages) {
    const role = typeof msg?.role === "string" ? msg.role : "user";
    const content = normalizeMessageContent(msg?.content).trim();
    if (!content) continue;

    if (role === "system" || role === "developer") {
      system.push(`[${role}] ${content}`);
    } else if (role === "assistant") {
      conversation.push(`Assistant prior message:\n${content}`);
    } else if (role === "tool") {
      const toolName = msg?.name ?? msg?.tool_call_id ?? "unknown_tool";
      conversation.push(`Tool result (${toolName}):\n${content}`);
    } else {
      conversation.push(content);
    }
  }

  if (system.length === 0) return conversation.join("\n\n");
  return [
    "System/developer instructions:",
    system.join("\n\n"),
    conversation.join("\n\n"),
  ].filter((part) => String(part).trim().length > 0).join("\n\n");
}

export function validateChatCompletionRequest(body) {
  const errors = [];
  const warnings = [];

  if (!isPlainObject(body)) {
    return { ok: false, errors: ["request body must be a JSON object"], warnings };
  }

  if (body.model != null && typeof body.model !== "string") {
    errors.push("model must be a string when provided");
  }

  if (body.prompt == null && !Array.isArray(body.messages)) {
    errors.push("provide messages[] or prompt");
  }

  if (Array.isArray(body.messages)) {
    body.messages.forEach((msg, idx) => {
      if (!isPlainObject(msg)) errors.push(`messages[${idx}] must be an object`);
      if (isPlainObject(msg) && msg.role != null && typeof msg.role !== "string") {
        errors.push(`messages[${idx}].role must be a string`);
      }
      if (isPlainObject(msg) && !("content" in msg) && msg.role !== "assistant") {
        errors.push(`messages[${idx}].content is required`);
      }
    });
  }

  if (body.n != null && Number(body.n) !== 1) {
    errors.push("n values other than 1 are unsupported by the browser router");
  }

  for (const [key, reason] of HARD_UNSUPPORTED_PARAMS.entries()) {
    if (body[key] != null) errors.push(`${key}: ${reason}`);
  }

  for (const key of SOFT_IGNORED_PARAMS) {
    if (body[key] != null) warnings.push({ param: key, behavior: "accepted_but_browser_provider_controls_generation" });
  }

  if (body.response_format != null) {
    const type = body.response_format?.type;
    if (type && type !== "text") {
      warnings.push({ param: "response_format", behavior: "not_enforced_by_browser_router" });
    }
  }

  return { ok: errors.length === 0, errors, warnings };
}

export function estimateTokenCount(text) {
  const s = String(text ?? "").trim();
  if (!s) return 0;
  const lexical = s.match(/[\p{L}\p{N}_]+|[^\s]/gu)?.length ?? 0;
  const charEstimate = Math.ceil(s.length / 4);
  return Math.max(1, Math.ceil((lexical + charEstimate) / 2));
}

export function estimateUsage({ prompt, completion }) {
  const promptTokens = estimateTokenCount(prompt);
  const completionTokens = estimateTokenCount(completion);
  return {
    prompt_tokens: promptTokens,
    completion_tokens: completionTokens,
    total_tokens: promptTokens + completionTokens,
  };
}

export function makeOpenAiChunk({ id, model, content, role = null, finishReason = null, created = null }) {
  const delta = {};
  if (role) delta.role = role;
  if (content != null) delta.content = content;
  return {
    id,
    object: "chat.completion.chunk",
    created: created ?? Math.floor(Date.now() / 1000),
    model,
    choices: [{ index: 0, delta, finish_reason: finishReason }],
  };
}

export function makeChatCompletionResponse({
  id,
  model,
  content,
  finishReason = "stop",
  created = null,
  usage,
  warnings = [],
  browser,
  turn,
}) {
  const response = {
    id,
    object: "chat.completion",
    created: created ?? Math.floor(Date.now() / 1000),
    model,
    choices: [{
      index: 0,
      message: {
        role: "assistant",
        content: String(content ?? ""),
        refusal: null,
        annotations: [],
      },
      finish_reason: finishReason,
    }],
    usage: usage ?? estimateUsage({ prompt: "", completion: content }),
    system_fingerprint: "browser-cdp-router",
  };

  if (warnings.length > 0) response.warnings = warnings;
  if (browser) response.browser = browser;
  if (turn) response.turn = turn;
  return response;
}