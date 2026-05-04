import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { validateTurnArtifacts } from '../src/tools/validate-turn-artifacts.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const fixtureRoot = path.join(__dirname, 'fixtures', 'artifacts');

test('accepts a completed turn with manifest, replay, redaction, and parseable NDJSON evidence', () => {
  const result = validateTurnArtifacts(path.join(fixtureRoot, 'valid'));
  assert.equal(result.pass, true);
  assert.equal(result.turn_count, 1);
  assert.equal(result.fail_count, 0);
});

test('rejects a completed turn missing manifest evidence', () => {
  const result = validateTurnArtifacts(path.join(fixtureRoot, 'missing-manifest'));
  assert.equal(result.pass, false);
  assert.match(result.failures[0].errors.join('\n'), /missing manifest\.json/);
});

test('rejects replay mismatch evidence', () => {
  const result = validateTurnArtifacts(path.join(fixtureRoot, 'replay-mismatch'));
  assert.equal(result.pass, false);
  assert.match(result.failures[0].errors.join('\n'), /replay\.replay_match must be true/);
});

test('rejects redaction failure evidence', () => {
  const result = validateTurnArtifacts(path.join(fixtureRoot, 'redaction-fail'));
  assert.equal(result.pass, false);
  assert.match(result.failures[0].errors.join('\n'), /evaluation\.redaction_pass must be true/);
});

test('rejects malformed JSON evidence deterministically', () => {
  const result = validateTurnArtifacts(path.join(fixtureRoot, 'malformed-evidence'));
  assert.equal(result.pass, false);
  assert.match(result.failures[0].errors.join('\n'), /malformed JSON/);
});