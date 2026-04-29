import { parseSseFrames } from "../extraction/frame-parser.mjs";
import { makeTextCandidateExtractor } from "../extraction/candidate-walker.mjs";
import { makeEmbeddedJsonReassembler } from "../extraction/embedded-json.mjs";
import { makeTextAccumulator } from "../extraction/accumulator.mjs";
import { CAPABILITIES } from "../provider/capability-contract.mjs";
import { makeSchemaGuidedTextExtractor } from "../extraction/schema-guided-extraction.mjs";
import { makeSchemaObserver } from "../extraction/schema-derivation.mjs";

function tryJson(text) {
  try { return JSON.parse(text); } catch { return null; }
}

function looksLikeConversationList(text) {
  return text.includes('"items"') && text.includes('"GROUP_DM"') && text.includes('"magic_link_url"');
}

function cleanOutputText(text) {
  let out = String(text ?? "");
  const denyTokens = [
    "model_editable_context",
    "model_set_context",
    // Keep internal-tool transcript tokens intact; only strip explicit prompt wrappers.
    "USER:",
    "SYSTEM:",
  ];
  for (const token of denyTokens) {
    out = out.split(token).join("");
  }
  return out.trim();
}

function extractAggregateResultSummary(agg) {
  if (!agg || typeof agg !== "object") return null;
  const stdout = Array.isArray(agg.messages)
    ? agg.messages.filter((m) => m?.stream_name === "stdout").map((m) => String(m?.text ?? "")).join("") || null
    : null;
  return {
    status: agg.status ?? null,
    run_id: agg.run_id ?? null,
    start_time: typeof agg.start_time === "number" ? agg.start_time : null,
    end_time: typeof agg.end_time === "number" ? agg.end_time : null,
    update_time: typeof agg.update_time === "number" ? agg.update_time : null,
    stdout,
    final_expression_output: agg.final_expression_output ?? null,
    in_kernel_exception: agg.in_kernel_exception ?? null,
    system_exception: agg.system_exception ?? null,
    timeout_triggered: agg.timeout_triggered ?? null,
  };
}

function extractEventMetadata(parsed) {
  const message = parsed?.v?.message ?? null;
  const content = message?.content ?? null;
  const metadata = message?.metadata ?? null;
  return {
    conversation_id: parsed?.conversation_id ?? parsed?.v?.conversation_id ?? null,
    message_id: parsed?.message_id ?? message?.id ?? null,
    type: parsed?.type ?? null,
    content_type: content?.content_type ?? null,
    author_role: message?.author?.role ?? null,
    author_name: message?.author?.name ?? null,
    recipient: message?.recipient ?? null,
    channel: message?.channel ?? null,
    status: message?.status ?? null,
    end_turn: typeof message?.end_turn === "boolean" ? message.end_turn : null,
    model_slug: metadata?.model_slug ?? null,
    resolved_model_slug: metadata?.resolved_model_slug ?? null,
    request_id: metadata?.request_id ?? null,
    turn_exchange_id: metadata?.turn_exchange_id ?? null,
    parent_id: metadata?.parent_id ?? null,
    message_type: metadata?.message_type ?? null,
    marker: parsed?.marker ?? null,
    event: parsed?.event ?? null,
    create_time: typeof message?.create_time === "number" ? message.create_time : null,
    update_time: typeof message?.update_time === "number" ? message.update_time : null,
    content_language: content?.language ?? null,
    response_format_name: content?.response_format_name ?? null,
    reasoning_status: metadata?.reasoning_status ?? null,
    reasoning_title: metadata?.reasoning_title ?? null,
    reasoning_start_time: typeof metadata?.reasoning_start_time === "number" ? metadata.reasoning_start_time : null,
    reasoning_end_time: typeof metadata?.reasoning_end_time === "number" ? metadata.reasoning_end_time : null,
    finished_duration_sec: typeof metadata?.finished_duration_sec === "number" ? metadata.finished_duration_sec : null,
    aggregate_result: extractAggregateResultSummary(metadata?.aggregate_result),
    is_thinking_preamble: metadata?.is_thinking_preamble_message ?? null,
    is_visually_hidden: metadata?.is_visually_hidden_from_conversation ?? null,
    citations: Array.isArray(metadata?.citations) && metadata.citations.length > 0 ? metadata.citations : null,
    search_result_groups: Array.isArray(metadata?.search_result_groups) && metadata.search_result_groups.length > 0 ? metadata.search_result_groups : null,
    content_references: Array.isArray(metadata?.content_references) && metadata.content_references.length > 0 ? metadata.content_references : null,
  };
}

function extractMessageText(content) {
  if (!content || typeof content !== "object") return "";
  if (typeof content.text === "string") return content.text;
  if (typeof content.content === "string") return content.content;
  if (Array.isArray(content.parts)) return content.parts.filter((p) => typeof p === "string").join("");
  return "";
}

function extractCalpicoTurnMeta(parsed) {
  // Harvests turn-level IDs that only appear on user calpico messages (not in raw_messages metadata).
  const events = Array.isArray(parsed) ? parsed : [parsed];
  const out = { request_id: null, conversation_id: null, offset: null };
  for (const event of events) {
    const payload = event?.payload?.payload;
    if (event?.payload?.type !== "calpico-message-add" || !payload?.message) continue;
    if (payload.room_id) out.conversation_id = out.conversation_id ?? payload.room_id;
    // request_id lives on the outer (user) calpico message, not in raw_messages metadata
    if (payload.message.request_id) out.request_id = out.request_id ?? payload.message.request_id;
    // Redis stream offset — encodes wall-clock arrival time
    if (event.offset) out.offset = out.offset ?? event.offset;
  }
  return out;
}

function extractCalpicoCandidates(parsed) {
  const events = Array.isArray(parsed) ? parsed : [parsed];
  const out = [];
  for (const event of events) {
    const payload = event?.payload?.payload;
    if (event?.payload?.type !== "calpico-message-add" || !payload?.message) continue;
    const roomId = payload.room_id ?? null;
    const groupMessage = payload.message;
    // Fall back to outer-message request_id (available on both user and assistant events)
    const outerRequestId = groupMessage.request_id ?? null;
    const outerCreatedAt = groupMessage.created_at ?? null;
    const calpicoOffset = event.offset ?? null;
    const rawMessages = Array.isArray(groupMessage.raw_messages) ? groupMessage.raw_messages : [];
    for (const message of rawMessages) {
      if (message?.author?.role !== "assistant") continue;
      if (message?.metadata?.is_visually_hidden_from_conversation === true) continue;
      const text = extractMessageText(message.content);
      if (!text) continue;
      const metadata = message.metadata ?? {};
      out.push({
        candidate: {
          text,
          rule: { kind: "calpico_raw_message", source: "websocket_frame" },
        },
        meta: {
          conversation_id: roomId,
          message_id: message.id ?? groupMessage.id ?? null,
          type: event?.payload?.type ?? null,
          content_type: message.content?.content_type ?? null,
          author_role: message.author?.role ?? null,
          recipient: message.recipient ?? null,
          channel: message.channel ?? null,
          status: message.status ?? null,
          end_turn: typeof message.end_turn === "boolean" ? message.end_turn : null,
          model_slug: metadata.model_slug ?? groupMessage.model_slug ?? null,
          resolved_model_slug: metadata.resolved_model_slug ?? null,
          // Prefer inner metadata, fall back to outer calpico message (user events carry it)
          request_id: metadata.request_id ?? groupMessage.request_id ?? null,
          turn_exchange_id: metadata.turn_exchange_id ?? null,
          message_type: metadata.message_type ?? null,
          author_name: message.author?.name ?? null,
          parent_id: metadata.parent_id ?? null,
          aggregate_result: extractAggregateResultSummary(metadata.aggregate_result),
          marker: null,
          event: null,
          // Prefer inner message timestamps; fall back to outer calpico created_at
          create_time: typeof message.create_time === "number" ? message.create_time
            : outerCreatedAt ? new Date(outerCreatedAt).getTime() / 1000 : null,
          update_time: typeof message.update_time === "number" ? message.update_time : null,
          content_language: message.content?.language ?? null,
          response_format_name: message.content?.response_format_name ?? null,
          reasoning_status: metadata.reasoning_status ?? null,
          reasoning_title: metadata.reasoning_title ?? null,
          reasoning_start_time: typeof metadata.reasoning_start_time === "number" ? metadata.reasoning_start_time : null,
          reasoning_end_time: typeof metadata.reasoning_end_time === "number" ? metadata.reasoning_end_time : null,
          finished_duration_sec: typeof metadata.finished_duration_sec === "number" ? metadata.finished_duration_sec : null,
          calpico_offset: calpicoOffset,
          outer_request_id: outerRequestId,
          weight: typeof message.weight === "number" ? message.weight : null,
          is_thinking_preamble: metadata.is_thinking_preamble_message ?? null,
          is_visually_hidden: metadata.is_visually_hidden_from_conversation ?? null,
          citations: Array.isArray(metadata.citations) && metadata.citations.length > 0 ? metadata.citations : null,
          search_result_groups: Array.isArray(metadata.search_result_groups) && metadata.search_result_groups.length > 0 ? metadata.search_result_groups : null,
          content_references: Array.isArray(metadata.content_references) && metadata.content_references.length > 0 ? metadata.content_references : null,
          is_proactively_sent: typeof groupMessage.is_proactively_sent === "boolean" ? groupMessage.is_proactively_sent : null,
          reply_to: groupMessage.reply_to ?? null,
        },
      });
    }
  }
  return out;
}

function isCalpicoMessageEvent(parsed) {
  const events = Array.isArray(parsed) ? parsed : [parsed];
  return events.some((event) => event?.payload?.type === "calpico-message-add");
}

function isPresent(value) {
  return value !== null && value !== undefined;
}

function buildGroupKey(entry) {
  const conv = entry?.conversation_id ?? "";
  const msg = entry?.message_id ?? "";
  const role = entry?.author_role ?? "";
  const channel = entry?.channel ?? "";
  const ctype = entry?.content_type ?? "";
  const recipient = entry?.recipient ?? "";
  return [conv, msg, role, channel, ctype, recipient].join("|");
}

function classifyPhase(entry) {
  if (entry?.channel === "final" && entry?.author_role === "assistant") return "final";
  if (entry?.content_type === "code" || entry?.recipient === "python") return "tool_call";
  if (entry?.content_type === "execution_output" || entry?.author_role === "tool") return "tool_output";
  if (entry?.content_type === "reasoning_recap") return "reasoning_recap";
  if (entry?.type === "message_marker") return "message_marker";
  return "message";
}

export function makeResponseProcessor({ onDelta, onActivity, onDone }) {
  const accumulator = makeTextAccumulator();
  const extractor = makeTextCandidateExtractor();
  const schemaExtractor = makeSchemaGuidedTextExtractor();
  const schemaObserver = makeSchemaObserver();
  const reassembler = makeEmbeddedJsonReassembler();
  const ruleEvidence = [];
  const modelResponses = [];
  const groupPatches = [];
  const rawCapture = [];
  const streamMetaByRequestId = new Map();
  let rawSequence = 0;
  const lifecycleMarkers = [];
  let messageStreamComplete = false;
  const streamOpsCounts = {};
  const turnMetaFields = { conversation_id: null, request_id: null, turn_exchange_id: null, turn_trace_id: null, calpico_offsets: [] };
  const thoughtsByMessageId = new Map();
  const hiddenMessageIds = new Set();
  const serverSteMeta = {
    turn_trace_id: null, turn_mode: null, turn_use_case: null,
    tool_invoked: null, tool_name: null, did_auto_switch_to_reasoning: null,
    cluster_region: null, plan_type: null, plan_type_bucket: null,
    model_slug: null, is_first_turn: null, fast_convo: null,
  };
  let limitsProgress = [];
  let defaultModelSlug = null;
  let blockedFeatures = [];

  function trackThoughts(parsed, mergedMeta) {
    // Add event: thoughts message with initial thoughts array
    const message = parsed?.v?.message;
    if (message?.id && message?.content?.content_type === "thoughts" && Array.isArray(message.content.thoughts)) {
      const existing = thoughtsByMessageId.get(message.id) ?? [];
      thoughtsByMessageId.set(message.id, [...existing, ...message.content.thoughts]);
    }
    // Patch event: append to /message/content/thoughts
    if (parsed?.o === "patch" && Array.isArray(parsed?.v)) {
      const msgId = mergedMeta.message_id;
      if (!msgId) return;
      for (const patch of parsed.v) {
        if (patch?.p === "/message/content/thoughts" && patch?.o === "append" && Array.isArray(patch?.v)) {
          const existing = thoughtsByMessageId.get(msgId) ?? [];
          thoughtsByMessageId.set(msgId, [...existing, ...patch.v]);
        }
      }
    }
  }

  function trackTurnMeta(parsed, eventMeta) {
    if (eventMeta.is_visually_hidden === true && eventMeta.message_id) hiddenMessageIds.add(eventMeta.message_id);
    if (eventMeta.conversation_id && !turnMetaFields.conversation_id) turnMetaFields.conversation_id = eventMeta.conversation_id;
    if (eventMeta.request_id && !turnMetaFields.request_id) turnMetaFields.request_id = eventMeta.request_id;
    if (eventMeta.turn_exchange_id && !turnMetaFields.turn_exchange_id) turnMetaFields.turn_exchange_id = eventMeta.turn_exchange_id;
    if (typeof parsed?.o === "string") {
      streamOpsCounts[parsed.o] = (streamOpsCounts[parsed.o] ?? 0) + 1;
    }
    if (parsed?.type === "message_marker") {
      lifecycleMarkers.push({
        marker: parsed.marker ?? null,
        event: parsed.event ?? null,
        message_id: parsed.message_id ?? null,
        conversation_id: parsed.conversation_id ?? null,
        observed_at: new Date().toISOString(),
      });
    }
    if (parsed?.type === "message_stream_complete") {
      messageStreamComplete = true;
    }
  }

  function trackServerSteMeta(parsed) {
    if (parsed?.type !== "server_ste_metadata") return;
    const m = parsed.metadata ?? {};
    const set = (key, val) => { if (val != null && serverSteMeta[key] == null) serverSteMeta[key] = val; };
    const setBool = (key, val) => { if (typeof val === "boolean" && serverSteMeta[key] === null) serverSteMeta[key] = val; };
    set("turn_trace_id", m.turn_trace_id); set("turn_mode", m.turn_mode);
    set("turn_use_case", m.turn_use_case); set("tool_name", m.tool_name);
    set("cluster_region", m.cluster_region); set("plan_type", m.plan_type);
    set("plan_type_bucket", m.plan_type_bucket); set("model_slug", m.model_slug);
    setBool("tool_invoked", m.tool_invoked); setBool("did_auto_switch_to_reasoning", m.did_auto_switch_to_reasoning);
    setBool("is_first_turn", m.is_first_turn); setBool("fast_convo", m.fast_convo);
    if (m.turn_trace_id && !turnMetaFields.turn_trace_id) turnMetaFields.turn_trace_id = m.turn_trace_id;
    if (parsed.conversation_id && !turnMetaFields.conversation_id) turnMetaFields.conversation_id = parsed.conversation_id;
    if (m.request_id && !turnMetaFields.request_id) turnMetaFields.request_id = m.request_id;
    if (m.turn_exchange_id && !turnMetaFields.turn_exchange_id) turnMetaFields.turn_exchange_id = m.turn_exchange_id;
  }

  function trackConversationDetailMeta(parsed) {
    if (parsed?.type !== "conversation_detail_metadata") return;
    if (Array.isArray(parsed.limits_progress) && limitsProgress.length === 0) limitsProgress = parsed.limits_progress;
    if (parsed.default_model_slug && !defaultModelSlug) defaultModelSlug = parsed.default_model_slug;
    if (Array.isArray(parsed.blocked_features) && blockedFeatures.length === 0 && parsed.blocked_features.length > 0) blockedFeatures = parsed.blocked_features;
    if (parsed.conversation_id && !turnMetaFields.conversation_id) turnMetaFields.conversation_id = parsed.conversation_id;
  }

  function trackInputMessageMeta(parsed) {
    if (parsed?.type !== "input_message") return;
    const m = parsed.input_message?.metadata ?? {};
    if (m.turn_trace_id) { if (!serverSteMeta.turn_trace_id) serverSteMeta.turn_trace_id = m.turn_trace_id; if (!turnMetaFields.turn_trace_id) turnMetaFields.turn_trace_id = m.turn_trace_id; }
    if (m.turn_exchange_id && !turnMetaFields.turn_exchange_id) turnMetaFields.turn_exchange_id = m.turn_exchange_id;
    if (m.request_id && !turnMetaFields.request_id) turnMetaFields.request_id = m.request_id;
  }

  function mergeMeta(source, parsedMeta = null) {
    const requestId = source?.request_id ?? null;
    const prior = requestId ? (streamMetaByRequestId.get(requestId) ?? {}) : {};
    const current = parsedMeta ?? {};
    // Only carry forward stable context; ephemeral event tags must remain event-local.
    const merged = {
      conversation_id: current.conversation_id ?? prior.conversation_id ?? null,
      message_id: current.message_id ?? prior.message_id ?? null,
      content_type: current.content_type ?? prior.content_type ?? null,
      author_role: current.author_role ?? prior.author_role ?? null,
      recipient: current.recipient ?? prior.recipient ?? null,
      channel: current.channel ?? prior.channel ?? null,
      status: current.status ?? prior.status ?? null,
      end_turn: current.end_turn ?? prior.end_turn ?? null,
      model_slug: current.model_slug ?? prior.model_slug ?? null,
      resolved_model_slug: current.resolved_model_slug ?? prior.resolved_model_slug ?? null,
      request_id: current.request_id ?? prior.request_id ?? null,
      turn_exchange_id: current.turn_exchange_id ?? prior.turn_exchange_id ?? null,
      // parent_id is safe to carry: add events set it, append events inherit it from prior
      parent_id: current.parent_id ?? prior.parent_id ?? null,
      message_type: current.message_type ?? prior.message_type ?? null,
      create_time: current.create_time ?? prior.create_time ?? null,
      update_time: current.update_time ?? prior.update_time ?? null,
      content_language: current.content_language ?? prior.content_language ?? null,
      response_format_name: current.response_format_name ?? prior.response_format_name ?? null,
      reasoning_status: current.reasoning_status ?? prior.reasoning_status ?? null,
      reasoning_title: current.reasoning_title ?? prior.reasoning_title ?? null,
      reasoning_start_time: current.reasoning_start_time ?? prior.reasoning_start_time ?? null,
      reasoning_end_time: current.reasoning_end_time ?? prior.reasoning_end_time ?? null,
      finished_duration_sec: current.finished_duration_sec ?? prior.finished_duration_sec ?? null,
      // is_thinking_preamble can be carried; is_visually_hidden is NOT carried — tracked via hiddenMessageIds Set
      is_thinking_preamble: current.is_thinking_preamble ?? prior.is_thinking_preamble ?? null,
      // author_name, aggregate_result, citations are message-specific; do not carry from prior
      author_name: current.author_name ?? null,
      aggregate_result: current.aggregate_result ?? null,
      citations: current.citations ?? null,
      search_result_groups: current.search_result_groups ?? null,
      content_references: current.content_references ?? null,
      // Event-local tags (do not carry forward)
      type: isPresent(current.type) ? current.type : null,
      marker: isPresent(current.marker) ? current.marker : null,
      event: isPresent(current.event) ? current.event : null,
    };
    if (requestId) streamMetaByRequestId.set(requestId, merged);
    return merged;
  }

  function emitCandidates(candidates, source, meta = null) {
    const mergedMeta = mergeMeta(source, meta);
    if (mergedMeta.message_id && hiddenMessageIds.has(mergedMeta.message_id)) return;
    for (const c of candidates) {
      modelResponses.push({
        text: String(c?.text ?? ""),
        source_event_kind: c?.rule?.kind ?? "unknown",
        source_request_id: source?.request_id ?? null,
        conversation_id: mergedMeta.conversation_id,
        message_id: mergedMeta.message_id,
        type: mergedMeta.type,
        content_type: mergedMeta.content_type,
        author_role: mergedMeta.author_role,
        author_name: mergedMeta.author_name,
        recipient: mergedMeta.recipient,
        channel: mergedMeta.channel,
        status: mergedMeta.status,
        end_turn: mergedMeta.end_turn,
        model_slug: mergedMeta.model_slug,
        resolved_model_slug: mergedMeta.resolved_model_slug,
        request_id: mergedMeta.request_id,
        turn_exchange_id: mergedMeta.turn_exchange_id,
        parent_id: mergedMeta.parent_id,
        message_type: mergedMeta.message_type,
        marker: mergedMeta.marker,
        event: mergedMeta.event,
        create_time: mergedMeta.create_time,
        update_time: mergedMeta.update_time,
        content_language: mergedMeta.content_language,
        response_format_name: mergedMeta.response_format_name,
        reasoning_status: mergedMeta.reasoning_status,
        reasoning_title: mergedMeta.reasoning_title,
        reasoning_start_time: mergedMeta.reasoning_start_time,
        reasoning_end_time: mergedMeta.reasoning_end_time,
        finished_duration_sec: mergedMeta.finished_duration_sec,
        aggregate_result: mergedMeta.aggregate_result,
        is_thinking_preamble: mergedMeta.is_thinking_preamble,
        citations: mergedMeta.citations,
        search_result_groups: mergedMeta.search_result_groups,
        content_references: mergedMeta.content_references,
        observed_at: new Date().toISOString(),
      });
      const delta = accumulator.append(c.text);
      if (delta) onDelta?.(delta);
    }
  }

  function recordRawCapture({ raw, parsed, source, frame = null }) {
    const meta = extractEventMetadata(parsed);
    rawCapture.push({
      sequence: rawSequence++,
      source_request_id: source?.request_id ?? null,
      source: source?.source ?? null,
      event_name: frame?.event_name ?? null,
      raw_length: typeof raw === "string" ? raw.length : null,
      parsed,
      meta,
      observed_at: new Date().toISOString(),
    });
  }

  function recordGroupPatch(parsed, source, meta) {
    const patches = parsed?.o === "patch" && Array.isArray(parsed?.v) ? parsed.v : null;
    if (!patches) return;
    const status = patches.find((p) => p?.p === "/message/status" && p?.o === "replace")?.v;
    const endTurn = patches.find((p) => p?.p === "/message/end_turn" && p?.o === "replace")?.v;
    if (status === undefined && endTurn === undefined) return;
    groupPatches.push({
      group_key: buildGroupKey(meta),
      status: typeof status === "string" ? status : undefined,
      end_turn: typeof endTurn === "boolean" ? endTurn : undefined,
      source_request_id: source?.request_id ?? null,
      observed_at: new Date().toISOString(),
    });
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
        recordRawCapture({ raw: frame.data, parsed, source, frame });
        const eventMeta = extractEventMetadata(parsed);
        const mergedMeta = mergeMeta(source, eventMeta);
        recordGroupPatch(parsed, source, mergedMeta);
        schemaObserver.observe(parsed);
        trackTurnMeta(parsed, eventMeta);
        trackThoughts(parsed, mergedMeta);
        trackServerSteMeta(parsed);
        trackConversationDetailMeta(parsed);
        trackInputMessageMeta(parsed);

        const calpicoCandidates = extractCalpicoCandidates(parsed);
        if (calpicoCandidates.length > 0) {
          for (const item of calpicoCandidates) {
            emitCandidates([item.candidate], source, item.meta);
            // Harvest turn-level IDs from calpico metas (SSE trackTurnMeta got nulls for these)
            if (item.meta.conversation_id && !turnMetaFields.conversation_id) turnMetaFields.conversation_id = item.meta.conversation_id;
            if (item.meta.request_id && !turnMetaFields.request_id) turnMetaFields.request_id = item.meta.request_id;
            if (item.meta.turn_exchange_id && !turnMetaFields.turn_exchange_id) turnMetaFields.turn_exchange_id = item.meta.turn_exchange_id;
            if (item.meta.calpico_offset) turnMetaFields.calpico_offsets.push(item.meta.calpico_offset);
          }
          continue;
        }
        if (isCalpicoMessageEvent(parsed)) {
          // User calpico messages produce no candidates but carry the turn request_id and offset
          const calpicoMeta = extractCalpicoTurnMeta(parsed);
          if (calpicoMeta.conversation_id && !turnMetaFields.conversation_id) turnMetaFields.conversation_id = calpicoMeta.conversation_id;
          if (calpicoMeta.request_id && !turnMetaFields.request_id) turnMetaFields.request_id = calpicoMeta.request_id;
          if (calpicoMeta.offset) turnMetaFields.calpico_offsets.push(calpicoMeta.offset);
          continue;
        }

        // Patch stream reassembly (ChatGPT /f/conversation format)
        if (source?.request_id && parsed.o === "append" && typeof parsed.p === "string" && typeof parsed.v === "string") {
          const assembled = reassembler.feedPatch({
            requestId: source.request_id,
            op: "append",
            path: parsed.p,
            value: parsed.v,
          });
          if (assembled) emitCandidates(extractor.extractFromValue(assembled.parsed), source, eventMeta);
        }

        const schemaExtract = schemaExtractor.extract(frame.data);
        if (schemaExtract.candidates.length > 0) {
          emitCandidates(schemaExtract.candidates, source, eventMeta);
          ruleEvidence.push(...schemaExtract.candidates.map((c) => ({
            ...c.rule,
            source_request_id: source?.request_id ?? null,
            observed_at: new Date().toISOString(),
          })));
        } else {
          emitCandidates(extractor.extractFromValue(parsed), source, eventMeta);
        }
      }
      return;
    }

    const parsed = tryJson(text.trim());
    if (parsed) {
      recordRawCapture({ raw: text.trim(), parsed, source });
      const eventMeta = extractEventMetadata(parsed);
      const mergedMeta = mergeMeta(source, eventMeta);
      recordGroupPatch(parsed, source, mergedMeta);
      schemaObserver.observe(parsed);
      trackTurnMeta(parsed, eventMeta);
      trackThoughts(parsed, mergedMeta);
      trackServerSteMeta(parsed);
      trackConversationDetailMeta(parsed);
      trackInputMessageMeta(parsed);
      const calpicoCandidates = extractCalpicoCandidates(parsed);
      if (calpicoCandidates.length > 0) {
        for (const item of calpicoCandidates) {
          emitCandidates([item.candidate], source, item.meta);
          if (item.meta.conversation_id && !turnMetaFields.conversation_id) turnMetaFields.conversation_id = item.meta.conversation_id;
          if (item.meta.request_id && !turnMetaFields.request_id) turnMetaFields.request_id = item.meta.request_id;
          if (item.meta.turn_exchange_id && !turnMetaFields.turn_exchange_id) turnMetaFields.turn_exchange_id = item.meta.turn_exchange_id;
          if (item.meta.calpico_offset) turnMetaFields.calpico_offsets.push(item.meta.calpico_offset);
        }
        return;
      }
      if (isCalpicoMessageEvent(parsed)) {
        const calpicoMeta = extractCalpicoTurnMeta(parsed);
        if (calpicoMeta.conversation_id && !turnMetaFields.conversation_id) turnMetaFields.conversation_id = calpicoMeta.conversation_id;
        if (calpicoMeta.request_id && !turnMetaFields.request_id) turnMetaFields.request_id = calpicoMeta.request_id;
        if (calpicoMeta.offset) turnMetaFields.calpico_offsets.push(calpicoMeta.offset);
        return;
      }
      const schemaExtract = schemaExtractor.extract(text.trim());
      if (schemaExtract.candidates.length > 0) {
        emitCandidates(schemaExtract.candidates, source, eventMeta);
        ruleEvidence.push(...schemaExtract.candidates.map((c) => ({
          ...c.rule,
          source_request_id: source?.request_id ?? null,
          observed_at: new Date().toISOString(),
        })));
      } else {
        emitCandidates(extractor.extractFromValue(parsed), source, eventMeta);
      }
    }
  }

  return {
    processChunk,
    accumulator,
    getCleanContent() { return cleanOutputText(accumulator.value()); },
    getFinalAssistantContent() {
      const final = modelResponses
        .filter((r) =>
          r?.author_role === "assistant" &&
          r?.channel === "final" &&
          typeof r?.text === "string" &&
          r.text.length > 0
        )
        .map((r) => r.text)
        .join("");
      return cleanOutputText(final);
    },
    getSchemaSnapshot() { return schemaObserver.snapshot(); },
    getRuleEvidence() { return [...ruleEvidence]; },
    getModelResponses() { return [...modelResponses]; },
    getRawCapture() { return [...rawCapture]; },
    getGroupedModelResponses() {
      const groups = [];
      const byKey = new Map();
      for (const item of modelResponses) {
        const key = buildGroupKey(item);
        let group = byKey.get(key);
        if (!group) {
          const phase = classifyPhase(item);
          group = {
            group_key: key,
            sequence_index: groups.length,
            phase,
            conversation_id: item.conversation_id ?? null,
            message_id: item.message_id ?? null,
            content_type: item.content_type ?? null,
            author_role: item.author_role ?? null,
            author_name: item.author_name ?? null,
            recipient: item.recipient ?? null,
            channel: item.channel ?? null,
            status: item.status ?? null,
            end_turn: item.end_turn ?? null,
            model_slug: item.model_slug ?? null,
            resolved_model_slug: item.resolved_model_slug ?? null,
            parent_id: item.parent_id ?? null,
            turn_exchange_id: item.turn_exchange_id ?? null,
            aggregate_result: item.aggregate_result ?? null,
            is_thinking_preamble: item.is_thinking_preamble ?? null,
            citations: item.citations ?? null,
            search_result_groups: item.search_result_groups ?? null,
            content_references: item.content_references ?? null,
            first_observed_at: item.observed_at ?? null,
            last_observed_at: item.observed_at ?? null,
            chunk_count: 0,
            text_length: 0,
            text: "",
          };
          byKey.set(key, group);
          groups.push(group);
        }
        group.text += item.text ?? "";
        group.chunk_count += 1;
        group.text_length = group.text.length;
        group.last_observed_at = item.observed_at ?? group.last_observed_at;
        // Update nullable fields if we get better data from later items
        group.author_name = group.author_name ?? item.author_name ?? null;
        group.aggregate_result = group.aggregate_result ?? item.aggregate_result ?? null;
        group.parent_id = group.parent_id ?? item.parent_id ?? null;
        group.turn_exchange_id = group.turn_exchange_id ?? item.turn_exchange_id ?? null;
        group.model_slug = group.model_slug ?? item.model_slug ?? null;
        group.resolved_model_slug = group.resolved_model_slug ?? item.resolved_model_slug ?? null;
        group.citations = group.citations ?? item.citations ?? null;
        group.search_result_groups = group.search_result_groups ?? item.search_result_groups ?? null;
        group.content_references = group.content_references ?? item.content_references ?? null;
      }
      for (const patch of groupPatches) {
        const group = byKey.get(patch.group_key);
        if (!group) continue;
        if (patch.status !== undefined) group.status = patch.status;
        if (patch.end_turn !== undefined) group.end_turn = patch.end_turn;
        group.last_observed_at = patch.observed_at ?? group.last_observed_at;
      }
      // Attach captured thoughts to their groups
      for (const group of groups) {
        if (group.message_id && thoughtsByMessageId.has(group.message_id)) {
          group.thoughts = thoughtsByMessageId.get(group.message_id);
        }
      }
      return groups;
    },
    getTurnMeta() {
      return {
        conversation_id: turnMetaFields.conversation_id,
        request_id: turnMetaFields.request_id,
        turn_exchange_id: turnMetaFields.turn_exchange_id,
        turn_trace_id: turnMetaFields.turn_trace_id,
        calpico_offsets: [...turnMetaFields.calpico_offsets],
        lifecycle_markers: [...lifecycleMarkers],
        message_stream_complete: messageStreamComplete,
        stream_ops: { ...streamOpsCounts },
        server_meta: { ...serverSteMeta },
        limits_progress: [...limitsProgress],
        default_model_slug: defaultModelSlug,
        blocked_features: blockedFeatures.length > 0 ? [...blockedFeatures] : [],
      };
    },
  };
}

export function buildReadReceipt({ adapter, receipts, targetId, status = "completed" }) {
  return receipts.add({
    provider: adapter.provider,
    capability: CAPABILITIES.READ_RESPONSE,
    status,
    targetId,
  });
}
