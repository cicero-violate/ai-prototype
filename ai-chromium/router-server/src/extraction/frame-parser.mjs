export function parseSseFrames(raw) {
  const frames = [];
  const lines = String(raw ?? "").split(/\r?\n/);
  let eventName = null;
  let dataLines = [];

  function flush() {
    if (eventName == null && dataLines.length === 0) return;
    frames.push({ event_name: eventName, data: dataLines.join("\n") });
    eventName = null;
    dataLines = [];
  }

  for (const line of lines) {
    if (line === "") { flush(); continue; }
    if (line.startsWith("event:")) { eventName = line.slice(6).trim(); continue; }
    if (line.startsWith("data:")) dataLines.push(line.slice(5).trimStart());
  }
  flush();
  return frames;
}
