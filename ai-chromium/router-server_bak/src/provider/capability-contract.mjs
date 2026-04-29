export const CAPABILITIES = Object.freeze({
  SEND_MESSAGE: "send_message",
  READ_RESPONSE: "read_response",
  UPLOAD_FILE: "upload_file",
  SELECT_PROJECT: "select_project",
  ATTACH_ARTIFACT: "attach_artifact",
  REPLAY_TURN: "replay_turn",
});

export function assertSupports(adapter, capability) {
  if (!adapter.capabilities.includes(capability)) {
    throw new Error(`provider "${adapter.provider}" does not support capability "${capability}"`);
  }
}
