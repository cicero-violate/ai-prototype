import { CdpSocket } from "../browser/cdp-socket.mjs";
import { resolve as resolveProvider } from "../provider/registry.mjs";
import { buildPlan } from "../capability/plan.mjs";
import { makeReceiptStore } from "../capability/receipts.mjs";
import { executeSendMessage } from "../capability/send-message.mjs";
import { makeResponseProcessor, buildReadReceipt } from "../capability/read-response.mjs";
import { executeSelectProject } from "../capability/select-project.mjs";
import { executeUploadFile } from "../capability/upload-file.mjs";
import { makeNetworkCapture } from "../capture/network-capture.mjs";
import { makeArtifactWriter } from "../evidence/artifact-writer.mjs";
import { redactRequest } from "../evidence/redaction.mjs";
import { CAPABILITIES } from "../provider/capability-contract.mjs";
import { classifyStructureOnly, redactPassFromRequest } from "../data/privacy-classifier.mjs";
import { buildDatasetRecord } from "../data/dataset-registry.mjs";
import { buildFeatureVector } from "../data/feature-extraction.mjs";
import { scoreCapability } from "../mining/score-capabilities.mjs";
import { compareProviderFromScore } from "../mining/compare-providers.mjs";
import { readPolicy, writePolicy, makePolicySnapshot } from "../policy/policy-store.mjs";
import { evaluatePolicyDelta } from "../feedback/evaluate-policy-delta.mjs";
import { buildFeedbackRecord } from "../feedback/record-feedback.mjs";
import { buildReplayRecord, buildEvaluationRecord } from "../replay/replay-turn.mjs";
import { updateMasterSchema } from "../extraction/schema-master-store.mjs";
import { scoreRulesByProvider } from "../mining/score-rules.mjs";
import { updateRuleLifecycle } from "../policy/rule-lifecycle-store.mjs";

let nextCompletionId = 1;

function nowUnix() { return Math.floor(Date.now() / 1000); }
function sleep(ms) { return new Promise((r) => setTimeout(r, ms)); }

function normalizeMessageContent(content) {
  if (typeof content === "string") return content;
  if (Array.isArray(content)) {
    return content.map((p) => {
      if (typeof p === "string") return p;
      if (typeof p?.text === "string") return p.text;
      if (typeof p?.content === "string") return p.content;
      return "";
    }).filter(Boolean).join("\n");
  }
  if (content == null) return "";
  return String(content);
}

function messagesToPrompt(messages) {
  if (!Array.isArray(messages)) return "";
  return messages.map((msg) => {
    const role = typeof msg?.role === "string" ? msg.role : "user";
    const content = normalizeMessageContent(msg?.content);
    return `${role.toUpperCase()}: ${content}`;
  }).filter((line) => line.trim().length > 0).join("\n\n");
}

function makeTurnId() {
  const ts = Date.now().toString(36);
  const rand = Math.random().toString(36).slice(2, 8);
  return `turn_${ts}_${rand}`;
}

function sseWrite(res, value) {
  res.write(`data: ${JSON.stringify(value)}\n\n`);
}

function makeOpenAiChunk({ id, model, content, finishReason = null }) {
  return {
    id,
    object: "chat.completion.chunk",
    created: nowUnix(),
    model,
    choices: [{ index: 0, delta: content == null ? {} : { content }, finish_reason: finishReason }],
  };
}

function setupSse(res) {
  res.writeHead(200, {
    "content-type": "text/event-stream; charset=utf-8",
    "cache-control": "no-cache, no-transform",
    connection: "keep-alive",
    "x-accel-buffering": "no",
    "access-control-allow-origin": "*",
  });
}

export async function runTurn({ request, stream, res, targetManager, config }) {
  const model = request.model ?? "chatgpt-cdp";
  const prompt = typeof request.prompt === "string"
    ? request.prompt
    : messagesToPrompt(request.messages);
  if (!prompt.trim()) throw new Error("empty prompt; provide prompt or messages[]");

  const adapter = resolveProvider(model, request);
  const turnId = makeTurnId();
  const completionId = `chatcmpl-cdp-${nextCompletionId++}`;
  const receipts = makeReceiptStore(turnId);
  const artifacts = makeArtifactWriter(turnId);

  const idleMs = Number(request.browser?.idle_ms ?? request.idle_ms ?? config.idleMs);
  const firstCaptureMs = Number(request.browser?.first_capture_ms ?? request.first_capture_ms ?? config.firstCaptureMs);
  const maxMs = Number(request.browser?.max_ms ?? request.max_ms ?? config.maxMs);
  const reset = Boolean(request.browser?.reset_chat ?? request.reset_chat ?? false);
  const wsEnabled = config.wsEnabled;
  const redactedRequest = redactRequest(request);

  const plan = buildPlan(adapter, request, turnId);
  artifacts.writeRedactedRequest(redactedRequest);
  artifacts.writeCapabilityPlan(plan);

  const target = await targetManager.findOrCreate({
    providerUrl: adapter.providerUrl,
    provider: adapter.provider,
    reset,
  });
  if (!target?.webSocketDebuggerUrl) {
    throw new Error("CDP target has no webSocketDebuggerUrl");
  }

  const cdp = new CdpSocket(target.webSocketDebuggerUrl);
  await cdp.connect();

  // Wait until the page has a JS execution context (new tabs start navigating)
  async function waitForContext(timeoutMs = 15000) {
    const deadline = Date.now() + timeoutMs;
    let attempts = 0;
    while (Date.now() < deadline) {
      try {
        await cdp.send("Runtime.evaluate", { expression: "1", returnByValue: true });
        if (attempts > 0) console.log(`[turn ${turnId}] context found after ${attempts} retries`);
        return;
      } catch (err) {
        const msg = String(err?.message ?? "");
        if (!msg.includes("Cannot find default execution context")) throw err;
        attempts++;
        if (attempts === 1) console.log(`[turn ${turnId}] context not ready, polling...`);
        await sleep(400);
      }
    }
    console.warn(`[turn ${turnId}] waitForContext timed out after ${timeoutMs}ms`);
  }

  let done = false;
  let idleTimer = null;
  let firstCaptureTimer = null;
  let maxTimer = null;
  let resolve, reject;
  const finalPromise = new Promise((res, rej) => { resolve = res; reject = rej; });
  finalPromise.catch(() => {});

  function armIdle() {
    clearTimeout(idleTimer);
    idleTimer = setTimeout(() => {
      if (capture.activeRequestCount() > 0) { armIdle(); return; }
      complete("stop");
    }, idleMs);
  }

  function clearFirstCapture() {
    clearTimeout(firstCaptureTimer);
    firstCaptureTimer = null;
  }

  function complete(reason = "stop") {
    if (done) return;
    done = true;
    clearTimeout(idleTimer);
    clearFirstCapture();
    clearTimeout(maxTimer);
    if (stream && !res.destroyed) {
      sseWrite(res, makeOpenAiChunk({ id: completionId, model, content: null, finishReason: reason }));
      // Emit structured turn envelope for clients that consume it
      try {
        const turnMeta = processor.getTurnMeta();
        const groupedEvents = processor.getGroupedModelResponses().reduce((acc, g) => {
          const phase = g.phase ?? "message";
          acc[phase] = acc[phase] ?? [];
          acc[phase].push(g);
          return acc;
        }, {});
        sseWrite(res, {
          object: "x-turn",
          turn: {
            conversation_id: turnMeta.conversation_id,
            request_id: turnMeta.request_id,
            turn_exchange_id: turnMeta.turn_exchange_id,
            turn_trace_id: turnMeta.turn_trace_id,
            events: groupedEvents,
            lifecycle_markers: turnMeta.lifecycle_markers,
            message_stream_complete: turnMeta.message_stream_complete,
            stream_ops: turnMeta.stream_ops,
            server_meta: turnMeta.server_meta,
            limits_progress: turnMeta.limits_progress,
            default_model_slug: turnMeta.default_model_slug,
          },
        });
      } catch {}
      res.write("data: [DONE]\n\n");
      res.end();
    }
    resolve({ reason });
  }

  const processor = makeResponseProcessor({
    onDelta(delta) {
      clearFirstCapture();
      armIdle();
      if (stream && !res.destroyed) {
        sseWrite(res, makeOpenAiChunk({ id: completionId, model, content: delta }));
      }
    },
    onActivity() {
      clearFirstCapture();
      armIdle();
    },
    onDone() {
      clearFirstCapture();
      armIdle();
    },
  });

  const capture = makeNetworkCapture({
    cdp,
    onChunk(text, source) { processor.processChunk(text, source); },
    onActivity() { clearFirstCapture(); armIdle(); },
    wsEnabled,
  });

  try {
    console.log(`[turn ${turnId}] enabling CDP domains on target ${target.id}`);
    await cdp.send("Page.enable");
    await cdp.send("Runtime.enable");
    await cdp.send("Network.enable", { maxTotalBufferSize: 100_000_000, maxResourceBufferSize: 50_000_000 });
    await cdp.send("Network.setCacheDisabled", { cacheDisabled: true }).catch(() => {});
    console.log(`[turn ${turnId}] waiting for execution context`);
    await waitForContext();
    console.log(`[turn ${turnId}] context ready`);

    if (reset) {
      await cdp.send("Page.navigate", { url: adapter.providerUrl });
      await sleep(1500);
      await waitForContext();
    }

    if (stream) {
      setupSse(res);
      sseWrite(res, makeOpenAiChunk({ id: completionId, model, content: "" }));
    }

    maxTimer = setTimeout(() => complete("length"), maxMs);
    firstCaptureTimer = setTimeout(() => complete("stop"), firstCaptureMs);

    // Execute capability plan.
    for (const step of plan.steps) {
      if (step.capability === CAPABILITIES.SELECT_PROJECT) {
        await executeSelectProject({
          cdp,
          adapter,
          receipts,
          targetId: target.id,
          projectHint: step.project_hint,
          defaultProjectId: config.defaultProjectId,
        });
      } else if (step.capability === CAPABILITIES.UPLOAD_FILE || step.capability === CAPABILITIES.ATTACH_ARTIFACT) {
        await executeUploadFile({
          adapter,
          receipts,
          targetId: target.id,
          cdpHost: config.cdpHost,
          cdpPort: config.cdpPort,
          uploadScript: config.uploadScript,
          defaultProjectId: config.defaultProjectId,
          projectHint: request?.browser?.project_hint,
          files: step.files,
        });
      } else if (step.capability === CAPABILITIES.SEND_MESSAGE) {
        await executeSendMessage({ cdp, prompt, adapter, receipts, targetId: target.id, request });
      }
      // READ_RESPONSE is handled by capture + timers.
    }

    await finalPromise;

    buildReadReceipt({ adapter, receipts, targetId: target.id });
    let finalTargetUrl = target.url;
    try {
      const loc = await cdp.send("Runtime.evaluate", {
        expression: "location.href",
        returnByValue: true,
      });
      finalTargetUrl = String(loc?.result?.value ?? finalTargetUrl);
    } catch {}

    const finalAssistantContent = processor.getFinalAssistantContent();
    const content = finalAssistantContent || processor.getCleanContent();
    const groupedModelResponses = processor.getGroupedModelResponses();
    const groupedByPhase = groupedModelResponses.reduce((acc, group) => {
      const phase = group.phase ?? "message";
      acc[phase] = acc[phase] ?? [];
      acc[phase].push(group);
      return acc;
    }, {});
    const schemaSnapshot = processor.getSchemaSnapshot();
    const masterSchema = updateMasterSchema({ turnId, provider: adapter.provider, schemaSnapshot });
    const ruleEvidence = processor.getRuleEvidence();
    const responseRecord = {
      schema: "ai_chromium.turn.v1",
      turn_id: turnId,
      completion_id: completionId,
      provider: adapter.provider,
      status: "completed",
      content_length: content.length,
      content,
      llm_model_responses_by_phase: groupedByPhase,
      finish_reason: "stop",
      created_at: new Date().toISOString(),
    };
    artifacts.writeResponse(responseRecord);
    artifacts.writeRawCapture(processor.getRawCapture());
    artifacts.writeActionReceipts(receipts.all());
    artifacts.writeSchemaArtifacts(schemaSnapshot);
    artifacts.writeRuleEvidence(ruleEvidence);
    artifacts.writeManifest({
      schema: "ai_chromium.turn_manifest.v1",
      turn_id: turnId,
      completion_id: completionId,
      provider: adapter.provider,
      target_id: target.id,
      target_url: finalTargetUrl,
      master_schema_updated: masterSchema.updated,
      master_schema_key_count: masterSchema.key_count,
      created_at: new Date().toISOString(),
    });

    const providerCapability = plan.steps[0]?.capability ?? CAPABILITIES.READ_RESPONSE;
    const privacyClass = classifyStructureOnly();
    const redactionPass = redactPassFromRequest(redactedRequest);
    const datasetRecord = buildDatasetRecord({
      turnId,
      provider: adapter.provider,
      capability: providerCapability,
      privacyClass,
      redactionPass,
      recordCount: receipts.all().length,
    });
    const feature = buildFeatureVector({
      datasetId: datasetRecord.dataset_id,
      provider: adapter.provider,
      capability: providerCapability,
      receipts: receipts.all(),
      contentLength: content.length,
    });
    const capabilityScore = scoreCapability({
      provider: adapter.provider,
      capability: providerCapability,
      receipts: receipts.all(),
    });
    const providerComparison = compareProviderFromScore(capabilityScore);

    const policy = readPolicy();
    const prevScore = Number(policy?.route_policy?.[adapter.provider]?.score ?? 0);
    policy.route_policy = policy.route_policy ?? {};
    policy.route_policy[adapter.provider] = {
      score: capabilityScore.score,
      updated_at: new Date().toISOString(),
      provider_comparison: providerComparison.rationale,
    };
    writePolicy(policy);
    const snapshot = makePolicySnapshot(policy);
    const delta = evaluatePolicyDelta({ previousScore: prevScore, currentScore: capabilityScore.score });
    const feedback = buildFeedbackRecord({
      policyVersion: snapshot.policy_version,
      turnId,
      provider: adapter.provider,
      capability: providerCapability,
      measuredDelta: delta.measured_delta,
      regression: delta.regression,
    });
    const replay = buildReplayRecord({ turnId, content });
    const evaluation = buildEvaluationRecord({ replayRecord: replay, redactionPass });
    const ruleScores = scoreRulesByProvider({
      provider: adapter.provider,
      ruleEvidence,
      replayPass: Boolean(replay.replay_match),
    });
    updateRuleLifecycle({
      provider: adapter.provider,
      turnId,
      replayPass: Boolean(replay.replay_match),
      ruleEvidence,
    });

    artifacts.writeDiscoverySignals([{
      schema: "ai_chromium.discovery_signal.v1",
      turn_id: turnId,
      provider: adapter.provider,
      capability: providerCapability,
      signal_kind: "turn_completion",
      source_refs: ["action-receipts.ndjson"],
      privacy_class: privacyClass,
      confidence: 0.99,
      schema_keys_observed: schemaSnapshot.count,
    }]);
    artifacts.writeDatasetRecords([datasetRecord]);
    artifacts.writeFeatureVectors([feature]);
    artifacts.writeCapabilityScores([capabilityScore]);
    artifacts.writePolicySnapshot(snapshot);
    artifacts.writeFeedback([feedback]);
    artifacts.writeReplay(replay);
    artifacts.writeEvaluation(evaluation);
    artifacts.writeRuleScores(ruleScores);

    const turnMeta = processor.getTurnMeta();
    return { id: completionId, model, content, finish_reason: "stop", target_id: target.id, target_url: finalTargetUrl, groupedByPhase, turnMeta };
  } catch (err) {
    const schemaSnapshot = processor.getSchemaSnapshot?.() ?? {
      index: { schema: "ai_chromium.schema_index.v1", created_at: new Date().toISOString(), keys: [] },
      docs: [],
      samples: [],
      count: 0,
    };
    artifacts.writeSchemaArtifacts(schemaSnapshot);
    artifacts.writeManifest({
      schema: "ai_chromium.turn_manifest.v1",
      turn_id: turnId,
      completion_id: completionId,
      provider: adapter.provider,
      target_id: target?.id ?? null,
      target_url: target?.url ?? null,
      status: "failed",
      error: String(err?.message ?? err),
      created_at: new Date().toISOString(),
    });
    throw err;
  } finally {
    clearTimeout(idleTimer);
    clearTimeout(maxTimer);
    capture.dispose();
    cdp.close();
  }
}

export async function handleChatCompletions(req, res, ctx) {
  const { readBody, parseJsonObject, errorResponse, jsonResponse } = ctx;
  let body;
  try {
    body = parseJsonObject(await readBody(req));
  } catch (err) {
    errorResponse(res, 400, err.message, { code: "invalid_request" });
    return;
  }

  const stream = Boolean(body.stream);
  try {
    const result = await runTurn({ request: body, stream, res, ...ctx });
    if (stream) return;
    jsonResponse(res, 200, {
      id: result.id,
      object: "chat.completion",
      created: nowUnix(),
      model: result.model,
      choices: [{ index: 0, message: { role: "assistant", content: result.content }, finish_reason: result.finish_reason ?? "stop" }],
      usage: { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 },
      browser: { backend: "cdp", target_id: result.target_id, target_url: result.target_url },
      turn: {
        conversation_id: result.turnMeta.conversation_id,
        request_id: result.turnMeta.request_id,
        turn_exchange_id: result.turnMeta.turn_exchange_id,
        turn_trace_id: result.turnMeta.turn_trace_id,
        calpico_offsets: result.turnMeta.calpico_offsets,
        events: result.groupedByPhase,
        lifecycle_markers: result.turnMeta.lifecycle_markers,
        message_stream_complete: result.turnMeta.message_stream_complete,
        stream_ops: result.turnMeta.stream_ops,
        server_meta: result.turnMeta.server_meta,
        limits_progress: result.turnMeta.limits_progress,
        default_model_slug: result.turnMeta.default_model_slug,
      },
    });
  } catch (err) {
    console.error(`[turn] failed:`, err.message);
    if (stream && !res.headersSent) {
      setupSse(res);
      sseWrite(res, { error: { message: err.message, type: "cdp_browser_router_error" } });
      res.write("data: [DONE]\n\n");
      res.end();
      return;
    }
    if (!res.destroyed && !res.headersSent) {
      errorResponse(res, 502, err.message, { code: "cdp_turn_failed" });
    }
  }
}
