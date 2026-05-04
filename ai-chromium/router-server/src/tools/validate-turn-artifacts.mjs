#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const REQUIRED_JSON = ['manifest.json', 'replay.json', 'evaluation.json'];
const TURN_MARKERS = new Set([...REQUIRED_JSON, 'response.json', 'request.redacted.json']);

const isDir = (filePath) => fs.existsSync(filePath) && fs.statSync(filePath).isDirectory();

function readJson(filePath) {
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch (error) {
    throw new Error(`${filePath}: malformed JSON: ${error.message}`);
  }
}

function validateNdjson(filePath) {
  for (const [index, line] of fs.readFileSync(filePath, 'utf8').split(/\r?\n/).entries()) {
    if (!line.trim()) continue;
    try {
      JSON.parse(line);
    } catch (error) {
      throw new Error(`${filePath}:${index + 1}: malformed JSON: ${error.message}`);
    }
  }
}

function looksLikeTurnDirectory(dirPath) {
  return isDir(dirPath) && fs.readdirSync(dirPath).some((name) => TURN_MARKERS.has(name));
}

function resolveTurnRoot(inputPath) {
  const directTurns = path.join(inputPath, 'artifacts', 'turns');
  return isDir(directTurns) ? directTurns : inputPath;
}

function findTurnDirectories(rootPath) {
  const turnRoot = resolveTurnRoot(rootPath);
  return fs.readdirSync(turnRoot)
    .map((name) => path.join(turnRoot, name))
    .filter(looksLikeTurnDirectory)
    .sort();
}

export function validateTurnDirectory(turnDir) {
  const errors = [];
  const docs = {};

  for (const fileName of REQUIRED_JSON) {
    const filePath = path.join(turnDir, fileName);
    if (!fs.existsSync(filePath)) {
      errors.push(`${turnDir}: missing ${fileName}`);
      continue;
    }
    try {
      docs[fileName] = readJson(filePath);
    } catch (error) {
      errors.push(error.message);
    }
  }

  for (const fileName of fs.readdirSync(turnDir).filter((name) => name.endsWith('.ndjson'))) {
    try { validateNdjson(path.join(turnDir, fileName)); } catch (error) { errors.push(error.message); }
  }

  const manifest = docs['manifest.json'];

  const checks = [
    [docs['replay.json'], 'replay.replay_match must be true', (doc) => doc.replay_match === true],
    [docs['evaluation.json'], 'evaluation.replay_match must be true', (doc) => doc.replay_match === true],
    [docs['evaluation.json'], 'evaluation.redaction_pass must be true', (doc) => doc.redaction_pass === true],
  ];

  if (manifest && typeof manifest.turn_id !== 'string') {
    errors.push(`${turnDir}: manifest.turn_id must be a string`);
  }
  for (const [doc, message, predicate] of checks) {
    if (doc && !predicate(doc)) errors.push(`${turnDir}: ${message}`);
  }
  for (const [doc, label] of [[docs['replay.json'], 'replay'], [docs['evaluation.json'], 'evaluation']]) {
    if (manifest?.turn_id && doc?.turn_id && manifest.turn_id !== doc.turn_id) {
      errors.push(`${turnDir}: ${label}.turn_id does not match manifest.turn_id`);
    }
  }

  return { turnDir, pass: errors.length === 0, errors };
}

export function validateTurnArtifacts(rootPath) {
  const turnDirs = findTurnDirectories(rootPath);
  const results = turnDirs.map(validateTurnDirectory);
  const failures = results.filter((result) => !result.pass);
  return {
    root: rootPath,
    turn_count: results.length,
    pass_count: results.length - failures.length,
    fail_count: failures.length,
    pass: results.length > 0 && failures.length === 0,
    failures,
  };
}

function main() {
  const rootPath = process.argv[2] ?? 'artifacts/turns';
  const result = validateTurnArtifacts(rootPath);
  console.log(JSON.stringify(result, null, 2));
  if (!result.pass) process.exitCode = 1;
}

if (import.meta.url === `file://${process.argv[1]}`) main();