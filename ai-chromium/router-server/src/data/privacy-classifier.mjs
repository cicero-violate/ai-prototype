export function classifyStructureOnly() {
  return "structure_only";
}

export function redactPassFromRequest(redactedRequest) {
  const hasMessages = Array.isArray(redactedRequest?.messages);
  const messagesPass = hasMessages
    ? redactedRequest.messages.every((msg) =>
        msg?.content_redacted === true &&
        !("content" in msg) &&
        typeof msg?.content_length === "number"
      )
    : true;

  const files = redactedRequest?.browser?.files;
  const hasFiles = Array.isArray(files);
  const filesPass = hasFiles
    ? files.every((file) => file?.path_redacted === true && !("path" in file))
    : true;

  const aggregateFlagsPass =
    (redactedRequest?.messages_redacted !== false) &&
    (redactedRequest?.browser?.files_redacted !== false);

  return aggregateFlagsPass && messagesPass && filesPass && (hasMessages || hasFiles);
}
