import crypto from "node:crypto";

function hash(text) {
  return crypto.createHash("sha256").update(String(text ?? "")).digest("hex");
}

function normalizeText(text) {
  return String(text ?? "")
    .replace(/\r\n/g, "\n")
    .replace(/[ \t]+\n/g, "\n")
    .trim();
}

function extractMessageText(content) {
  if (!content || typeof content !== "object") return "";
  if (typeof content.text === "string") return content.text;
  if (typeof content.content === "string") return content.content;
  if (Array.isArray(content.parts)) return content.parts.filter((part) => typeof part === "string").join("");
  return "";
}

function extractReplayTextFromRecord(record) {
  const parsed = record?.parsed;
  const message = parsed?.v?.message ?? parsed?.message ?? null;
  const meta = record?.meta ?? {};
  if (message?.author?.role && message.author.role !== "assistant") return "";
  if (meta.author_role && meta.author_role !== "assistant") return "";
  if (meta.is_visually_hidden === true) return "";

  const direct = extractMessageText(message?.content);
  if (direct) return direct;

  if (parsed?.o === "patch" && Array.isArray(parsed?.v)) {
    return parsed.v
      .filter((patch) =>
        patch?.o === "append" &&
        typeof patch?.v === "string" &&
        /\/message\/content\/parts\/\d+$/.test(String(patch?.p ?? ""))
      )
      .map((patch) => patch.v)
      .join("");
  }

  return "";
}

export function replayExtractContent(rawCapture) {
  if (!Array.isArray(rawCapture)) return "";
  return rawCapture.map(extractReplayTextFromRecord).filter(Boolean).join("");
}

export function buildReplayRecord({ turnId, content, rawCapture = [], extractorVersion = "raw_capture_replay_v2" }) {
  const expected = normalizeText(content);
  const replayed = normalizeText(replayExtractContent(rawCapture));
  const outputHash = hash(expected);
  const replayedHash = hash(replayed);
  const replayMatch = Boolean(expected) && expected === replayed;
  return {
    schema: "ai_chromium.replay_turn.v1",
    turn_id: turnId,
    extractor_version: extractorVersion,
    output_hash: outputHash,
    replayed_output_hash: replayedHash,
    replay_match: replayMatch,
    replay_evidence: {
      method: "re-extract assistant text from in-memory raw capture and compare normalized output",
      raw_record_count: Array.isArray(rawCapture) ? rawCapture.length : 0,
      expected_length: expected.length,
      replayed_length: replayed.length,
      strict_match: replayMatch,
    },
  };
}

export function buildEvaluationRecord({ replayRecord, redactionPass }) {
  return {
    schema: "ai_chromium.evaluation.v1",
    turn_id: replayRecord.turn_id,
    replay_match: replayRecord.replay_match,
    redaction_pass: redactionPass,
    quality: replayRecord.replay_match && redactionPass ? 1 : 0,
  };
}
