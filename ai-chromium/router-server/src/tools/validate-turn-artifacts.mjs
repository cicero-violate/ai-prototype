#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const REQUIRED_JSON = ['manifest.json', 'replay.json', 'evaluation.json'];

function readJson(filePath) {
  try {
    return JSON.parse(fs.readFileSync(filePath, 'utf8'));
  } catch (error) {
    throw new Error(`${filePath}: malformed JSON: ${error.message}`);
  }
}

function validateNdjson(filePath) {
  const text = fs.readFileSync(filePath, 'utf8');
  const lines = text.split(/\r?\n/).filter((line) => line.trim().length > 0);
  for (const [index, line] of lines.entries()) {
    try {
      JSON.parse(line);
    } catch (error) {
      throw new Error(`${filePath}:${index + 1}: malformed NDJSON: ${error.message}`);
    }
  }
}

function looksLikeTurnDirectory(dirPath) {
  if (!fs.statSync(dirPath).isDirectory()) return false;
  return fs.readdirSync(dirPath).some((name) => {
    return REQUIRED_JSON.includes(name) || name === 'response.json' || name === 'request.redacted.json';
  });
}

function resolveTurnRoot(inputPath) {
  const directTurns = path.join(inputPath, 'artifacts', 'turns');
  if (fs.existsSync(directTurns) && fs.statSync(directTurns).isDirectory()) return directTurns;
  return inputPath;
}

function findTurnDirectories(rootPath) {
  const turnRoot = resolveTurnRoot(rootPath);
  return fs.readdirSync(turnRoot)
    .map((name) => path.join(turnRoot, name))
    .filter((entry) => fs.statSync(entry).isDirectory())
    .filter(looksLikeTurnDirectory)
    .sort();
}

export function validateTurnDirectory(turnDir) {
  const errors = [];

  for (const fileName of REQUIRED_JSON) {
    const filePath = path.join(turnDir, fileName);
    if (!fs.existsSync(filePath)) errors.push(`${turnDir}: missing ${fileName}`);
  }

  let manifest;
  let replay;
  let evaluation;

  for (const fileName of REQUIRED_JSON) {
    const filePath = path.join(turnDir, fileName);
    if (!fs.existsSync(filePath)) continue;
    try {
      const parsed = readJson(filePath);
      if (fileName === 'manifest.json') manifest = parsed;
      if (fileName === 'replay.json') replay = parsed;
      if (fileName === 'evaluation.json') evaluation = parsed;
    } catch (error) {
      errors.push(error.message);
    }
  }

  for (const fileName of fs.readdirSync(turnDir).filter((name) => name.endsWith('.ndjson'))) {
    try {
      validateNdjson(path.join(turnDir, fileName));
    } catch (error) {
      errors.push(error.message);
    }
  }

  if (manifest && typeof manifest.turn_id !== 'string') {
    errors.push(`${turnDir}: manifest.turn_id must be a string`);
  }
  if (replay && replay.replay_match !== true) {
    errors.push(`${turnDir}: replay.replay_match must be true`);
  }
  if (evaluation && evaluation.replay_match !== true) {
    errors.push(`${turnDir}: evaluation.replay_match must be true`);
  }
  if (evaluation && evaluation.redaction_pass !== true) {
    errors.push(`${turnDir}: evaluation.redaction_pass must be true`);
  }
  if (manifest?.turn_id && replay?.turn_id && manifest.turn_id !== replay.turn_id) {
    errors.push(`${turnDir}: replay.turn_id does not match manifest.turn_id`);
  }
  if (manifest?.turn_id && evaluation?.turn_id && manifest.turn_id !== evaluation.turn_id) {
    errors.push(`${turnDir}: evaluation.turn_id does not match manifest.turn_id`);
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