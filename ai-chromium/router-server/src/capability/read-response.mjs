import { parseSseFrames } from "../extraction/frame-parser.mjs";
import { makeTextCandidateExtractor } from "../extraction/candidate-walker.mjs";
import { makeEmbeddedJsonReassembler } from "../extraction/embedded-json.mjs";
import { makeTextAccumulator } from "../extraction/accumulator.mjs";
import { CAPABILITIES } from "../provider/capability-contract.mjs";

function tryJson(text) {
  try { return JSON.parse(text); } catch { return null; }
}

function looksLikeConversationList(text) {
  return text.includes('"items"') && text.includes('"GROUP_DM"') && text.includes('"magic_link_url"');
}

export function makeResponseProcessor({ onDelta, onActivity, onDone }) {
  const accumulator = makeTextAccumulator();
  const extractor = makeTextCandidateExtractor();
  const reassembler = makeEmbeddedJsonReassembler();

  function emitCandidates(candidates) {
    for (const c of candidates) {
      const delta = accumulator.append(c.text);
      if (delta) onDelta?.(delta);
    }
  }

  function processChunk(text, source) {
    if (!text || looksLikeConversationList(text)) return;
    onActivity?.();

    const frames = parseSseFrames(text);
    if (frames.length > 0) {
      for (const frame of frames) {
        if (!frame.data) continue;
        if (frame.data === "[DONE]") {
          onDone?.(source?.request_id);
          continue;
        }
        const parsed = tryJson(frame.data);
        if (!parsed) continue;

        // Patch stream reassembly (ChatGPT /f/conversation format)
        if (source?.request_id && parsed.o === "append" && typeof parsed.p === "string" && typeof parsed.v === "string") {
          const assembled = reassembler.feedPatch({
            requestId: source.request_id,
            op: "append",
            path: parsed.p,
            value: parsed.v,
          });
          if (assembled) emitCandidates(extractor.extractFromValue(assembled.parsed));
        }

        emitCandidates(extractor.extractFromValue(parsed));
      }
      return;
    }

    const parsed = tryJson(text.trim());
    if (parsed) emitCandidates(extractor.extractFromValue(parsed));
  }

  return { processChunk, accumulator };
}

export function buildReadReceipt({ adapter, receipts, targetId, status = "completed" }) {
  return receipts.add({
    provider: adapter.provider,
    capability: CAPABILITIES.READ_RESPONSE,
    status,
    targetId,
  });
}
