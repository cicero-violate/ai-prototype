const SENSITIVE_PATTERNS = [
  /token/i, /authorization/i, /cookie/i, /secret/i, /cursor/i,
  /magic_link/i, /avatar_url/i, /access_token/i, /refresh_token/i,
  /session/i,
];

export function isSensitivePath(path) {
  return SENSITIVE_PATTERNS.some((re) => re.test(String(path ?? "")));
}

export function redactRequest(request) {
  const { model, stream } = request;
  const out = { model, stream };

  if (Array.isArray(request.messages)) {
    out.messages = request.messages.map((msg) => ({
      role: msg?.role ?? "user",
      content_length: typeof msg?.content === "string" ? msg.content.length : null,
      content_redacted: true,
    }));
  }

  const b = request.browser;
  if (b && typeof b === "object") {
    out.browser = {
      provider: b.provider ?? null,
      capabilities: b.capabilities ?? null,
      project_hint: b.project_hint ?? null,
      conversation_hint: b.conversation_hint ?? null,
      files: Array.isArray(b.files)
        ? b.files.map((f) => ({ purpose: f?.purpose ?? null, path_redacted: true }))
        : null,
    };
  }

  return out;
}
