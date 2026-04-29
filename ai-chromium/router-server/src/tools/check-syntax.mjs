#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

const ROOT = process.cwd();
const SKIP_DIRS = new Set([".git", "node_modules", "artifacts", "_old"]);

function collectMjs(dir, out = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      if (!SKIP_DIRS.has(entry.name)) collectMjs(path.join(dir, entry.name), out);
    } else if (entry.isFile() && entry.name.endsWith(".mjs")) {
      out.push(path.join(dir, entry.name));
    }
  }
  return out;
}

let failures = 0;
for (const file of collectMjs(ROOT)) {
  const rel = path.relative(ROOT, file);
  const result = spawnSync(process.execPath, ["--check", file], { encoding: "utf8" });
  if (result.status !== 0) {
    failures += 1;
    process.stderr.write(`node --check failed: ${rel}\n`);
    if (result.stderr) process.stderr.write(result.stderr);
  }
}

if (failures > 0) process.exit(1);
console.log(`syntax_ok files=${collectMjs(ROOT).length}`);