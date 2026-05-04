import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { validateTurnArtifacts } from '../src/tools/validate-turn-artifacts.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const fixtureRoot = path.join(__dirname, 'fixtures', 'artifacts');

const cases = [
  ['valid', true, null],
  ['missing-manifest', false, /missing manifest\.json/],
  ['replay-mismatch', false, /replay\.replay_match must be true/],
  ['redaction-fail', false, /evaluation\.redaction_pass must be true/],
  ['malformed-evidence', false, /malformed JSON/],
];

for (const [name, pass, pattern] of cases) {
  test(`artifact fixture: ${name}`, () => {
    const result = validateTurnArtifacts(path.join(fixtureRoot, name));
    assert.equal(result.scope, 'strict');
    assert.equal(result.pass, pass);
    assert.equal(result.turn_count, 1);
    if (pattern) {
      assert.match(result.failures[0].errors.join('\n'), pattern);
      assert.equal(result.failure_summaries.length, 1);
    }
    else assert.equal(result.fail_count, 0);
  });
}
