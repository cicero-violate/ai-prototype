export function classifyStructureOnly() {
  return "structure_only";
}

export function redactPassFromRequest(redactedRequest) {
  return Boolean(redactedRequest?.messages_redacted || redactedRequest?.browser?.files_redacted);
}
