function decodeBase64(data) {
  if (typeof data !== "string" || !data) return "";
  try { return Buffer.from(data, "base64").toString("utf8"); }
  catch { return ""; }
}

function shouldCaptureResponse(params) {
  const url = params?.response?.url ?? "";
  const mime = String(params?.response?.mimeType ?? "").toLowerCase();
  const headers = params?.response?.headers ?? {};
  const ct = String(headers["content-type"] ?? headers["Content-Type"] ?? "").toLowerCase();
  if (url.includes("/backend-api/f/conversation")) return true;
  if (url.includes("/backend-api/conversation")) return true;
  if (url.includes("/backend-api/sentinel/chat-requirements")) return false;
  if (ct.includes("text/event-stream") || mime.includes("event-stream")) return true;
  return false;
}

function shouldCaptureRequest(params) {
  const url = params?.request?.url ?? "";
  return url.includes("/backend-api/f/conversation") || url.includes("/backend-api/conversation");
}

export function makeNetworkCapture({ cdp, onChunk, onActivity, wsEnabled = true }) {
  const capturedRequests = new Set();
  const bodyFallbackRequests = new Set();
  const requestsWithData = new Set();
  const activeStreamRequests = new Set();

  function deliver(text, source) {
    if (!text) return;
    onChunk?.(text, source);
    onActivity?.();
  }

  function startStream(requestId) {
    if (!requestId || capturedRequests.has(requestId)) return;
    capturedRequests.add(requestId);
    cdp.send("Network.streamResourceContent", { requestId })
      .then((result) => {
        if (result?.__stale_stream_resource) {
          capturedRequests.delete(requestId);
          bodyFallbackRequests.add(requestId);
          return;
        }
        const buffered = decodeBase64(result?.bufferedData);
        if (buffered) deliver(buffered, { source: "stream_buffered", request_id: requestId });
      })
      .catch((err) => {
        capturedRequests.delete(requestId);
        bodyFallbackRequests.add(requestId);
        const msg = String(err?.message ?? err);
        if (msg.includes("already finished loading")) return;
        // non-fatal; body fallback will pick it up on loadingFinished
      });
  }

  function cleanupRequest(id) {
    capturedRequests.delete(id);
    bodyFallbackRequests.delete(id);
    requestsWithData.delete(id);
    activeStreamRequests.delete(id);
  }

  const removeListener = cdp.onEvent(async (method, params) => {
    if (method === "Network.requestWillBeSent" && shouldCaptureRequest(params)) {
      if (params.requestId) activeStreamRequests.add(params.requestId);
      startStream(params.requestId);
      onActivity?.();
      return;
    }
    if (method === "Network.responseReceived" && shouldCaptureResponse(params)) {
      if (params.requestId) activeStreamRequests.add(params.requestId);
      startStream(params.requestId);
      onActivity?.();
      return;
    }
    if (method === "Network.dataReceived" && capturedRequests.has(params.requestId)) {
      requestsWithData.add(params.requestId);
      const text = decodeBase64(params.data);
      if (text) deliver(text, { source: "data_received", request_id: params.requestId });
      onActivity?.();
      return;
    }
    if (method === "Network.loadingFinished") {
      const id = params.requestId;
      const needsBody = bodyFallbackRequests.has(id) ||
        (capturedRequests.has(id) && !requestsWithData.has(id));
      if (needsBody) {
        const body = await cdp.send("Network.getResponseBody", { requestId: id }).catch(() => null);
        const text = body?.base64Encoded
          ? decodeBase64(body.body)
          : (typeof body?.body === "string" ? body.body : "");
        if (text) deliver(text, { source: "response_body", request_id: id });
      }
      cleanupRequest(id);
      onActivity?.();
      return;
    }
    if (method === "Network.loadingFailed") {
      cleanupRequest(params.requestId);
      return;
    }
    if (method === "Network.eventSourceMessageReceived") {
      onActivity?.();
      deliver(`data: ${params.data}\n\n`, {
        source: "event_source",
        request_id: params.requestId ?? null,
      });
      return;
    }
    if (wsEnabled && method === "Network.webSocketFrameReceived") {
      const payload = params?.response?.payloadData;
      if (typeof payload === "string") {
        onActivity?.();
        if (
          payload.includes("calpico-message-add") ||
          payload.includes('"raw_messages"') ||
          payload.includes('"content_type":"text"')
        ) {
          deliver(payload, { source: "websocket_frame", request_id: params.requestId ?? null });
        }
      }
    }
  });

  return {
    activeRequestCount() { return activeStreamRequests.size; },
    dispose() { removeListener(); },
  };
}
