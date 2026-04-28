import crypto from "node:crypto";
import net from "node:net";
import { URL } from "node:url";

export class CdpSocket {
  constructor(wsUrl) {
    this.wsUrl = new URL(wsUrl);
    this.socket = null;
    this.buffer = Buffer.alloc(0);
    this.nextId = 1;
    this.pending = new Map();
    this.listeners = new Set();
    this.open = false;
  }

  connect() {
    return new Promise((resolve, reject) => {
      if (this.wsUrl.protocol !== "ws:") {
        reject(new Error(`unsupported CDP websocket protocol: ${this.wsUrl.protocol}`));
        return;
      }
      const port = Number(this.wsUrl.port || 80);
      const host = this.wsUrl.hostname;
      const path = `${this.wsUrl.pathname}${this.wsUrl.search}`;
      const key = crypto.randomBytes(16).toString("base64");
      const socket = net.createConnection({ host, port }, () => {
        socket.write([
          `GET ${path} HTTP/1.1`,
          `Host: ${host}:${port}`,
          "Upgrade: websocket",
          "Connection: Upgrade",
          `Sec-WebSocket-Key: ${key}`,
          "Sec-WebSocket-Version: 13",
          "\r\n",
        ].join("\r\n"));
      });
      this.socket = socket;
      let handshake = Buffer.alloc(0);
      const onHandshakeData = (chunk) => {
        handshake = Buffer.concat([handshake, chunk]);
        const marker = handshake.indexOf("\r\n\r\n");
        if (marker < 0) return;
        const head = handshake.subarray(0, marker + 4).toString("latin1");
        if (!head.startsWith("HTTP/1.1 101") && !head.startsWith("HTTP/1.0 101")) {
          reject(new Error(`CDP websocket handshake failed: ${head.split("\r\n")[0]}`));
          socket.destroy();
          return;
        }
        socket.off("data", onHandshakeData);
        socket.on("data", (data) => this.onData(data));
        socket.on("close", () => this.close(new Error("CDP websocket closed")));
        socket.on("error", (err) => this.close(err));
        this.open = true;
        const rest = handshake.subarray(marker + 4);
        if (rest.length > 0) this.onData(rest);
        resolve();
      };
      socket.on("data", onHandshakeData);
      socket.on("error", reject);
      socket.setTimeout(15000, () => {
        reject(new Error("CDP websocket handshake timeout"));
        socket.destroy();
      });
    });
  }

  send(method, params = {}) {
    if (!this.open) return Promise.reject(new Error("CDP websocket is not open"));
    const id = this.nextId++;
    const message = JSON.stringify({ id, method, params });
    this.socket.write(encodeClientWsFrame(message));
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`CDP command timed out: ${method}`));
      }, 30000);
      this.pending.set(id, { resolve, reject, timer, method });
    });
  }

  onEvent(fn) {
    this.listeners.add(fn);
    return () => this.listeners.delete(fn);
  }

  onData(chunk) {
    this.buffer = Buffer.concat([this.buffer, chunk]);
    while (true) {
      const frame = decodeWsFrame(this.buffer);
      if (!frame) return;
      this.buffer = this.buffer.subarray(frame.consumed);
      if (frame.opcode === 0x8) {
        this.socket.end();
        this.close(new Error("CDP websocket closed"));
        return;
      }
      if (frame.opcode === 0x9) {
        this.socket.write(encodeClientWsFrame(frame.payload, 0xA));
        continue;
      }
      if (frame.opcode !== 0x1) continue;
      const text = frame.payload.toString("utf8");
      let msg;
      try {
        msg = JSON.parse(text);
      } catch {
        continue;
      }
      if (msg.id != null) {
        const pending = this.pending.get(msg.id);
        if (!pending) continue;
        clearTimeout(pending.timer);
        this.pending.delete(msg.id);
        if (msg.error) {
          const errorMessage = msg.error.message ?? JSON.stringify(msg.error);
          if (pending.method === "Network.streamResourceContent" && String(errorMessage).includes("already finished loading")) {
            pending.resolve({ __stale_stream_resource: true, error: msg.error });
          } else {
            pending.reject(new Error(`${pending.method}: ${errorMessage}`));
          }
        } else {
          pending.resolve(msg.result ?? {});
        }
        continue;
      }
      if (msg.method) {
        for (const fn of this.listeners) {
          try { fn(msg.method, msg.params ?? {}); } catch {}
        }
      }
    }
  }

  close(err = null) {
    if (!this.open && !this.socket) return;
    this.open = false;
    try { this.socket?.destroy(); } catch {}
    this.socket = null;
    for (const [id, pending] of this.pending) {
      clearTimeout(pending.timer);
      pending.reject(err ?? new Error("CDP websocket closed"));
      this.pending.delete(id);
    }
  }
}

function encodeClientWsFrame(value, opcode = 0x1) {
  const payload = Buffer.isBuffer(value) ? value : Buffer.from(String(value), "utf8");
  const mask = crypto.randomBytes(4);
  let header;
  if (payload.length < 126) {
    header = Buffer.from([0x80 | opcode, 0x80 | payload.length]);
  } else if (payload.length < 65536) {
    header = Buffer.alloc(4);
    header[0] = 0x80 | opcode;
    header[1] = 0x80 | 126;
    header.writeUInt16BE(payload.length, 2);
  } else {
    header = Buffer.alloc(10);
    header[0] = 0x80 | opcode;
    header[1] = 0x80 | 127;
    header.writeBigUInt64BE(BigInt(payload.length), 2);
  }
  const masked = Buffer.from(payload);
  for (let i = 0; i < masked.length; i++) masked[i] ^= mask[i % 4];
  return Buffer.concat([header, mask, masked]);
}

function decodeWsFrame(buffer) {
  if (buffer.length < 2) return null;
  const b0 = buffer[0];
  const opcode = b0 & 0x0f;
  const masked = Boolean(buffer[1] & 0x80);
  let length = buffer[1] & 0x7f;
  let offset = 2;
  if (length === 126) {
    if (buffer.length < offset + 2) return null;
    length = buffer.readUInt16BE(offset);
    offset += 2;
  } else if (length === 127) {
    if (buffer.length < offset + 8) return null;
    const big = buffer.readBigUInt64BE(offset);
    if (big > BigInt(Number.MAX_SAFE_INTEGER)) throw new Error("websocket frame too large");
    length = Number(big);
    offset += 8;
  }
  let mask = null;
  if (masked) {
    if (buffer.length < offset + 4) return null;
    mask = buffer.subarray(offset, offset + 4);
    offset += 4;
  }
  if (buffer.length < offset + length) return null;
  const payload = Buffer.from(buffer.subarray(offset, offset + length));
  if (mask) {
    for (let i = 0; i < payload.length; i++) payload[i] ^= mask[i % 4];
  }
  return { opcode, payload, consumed: offset + length };
}
