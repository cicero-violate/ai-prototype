# Canon Mini Agent Scorecard

## Judgment

Current score: `6.92 / 10`

This is a strong research prototype with a real deterministic kernel, typed
gate progression, replayable TLogs, durable NDJSON persistence, local LLM
adapter paths, sandbox process receipts, and an initial append-only policy
store. It is not yet a mature self-improving agent. The strongest parts are
auditability, determinism, and kernel correctness. The weakest parts are real
learning quality, scalability, operational deployment, evaluator depth, and
simplicity.

The score is intentionally conservative. The project has many proof-shaped
pieces, but several are still examples, tests, or local traces rather than a
closed production loop.

## Formula

```text
G = geometric_mean(I, E, C, A, R, P, S, D, T, Co, Em, B, L, Si, F)
```

A geometric mean is used because one weak dimension should reduce the whole
score. A system that is transparent but cannot scale, or learns but cannot
verify, is not good enough.

## Scores

| Var | Dimension       | Score | Critical rationale                                                                                                                                                                       |
|-----+-----------------+-------+------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| I   | Intelligence    |   6.8 | The runtime can route LLM judgment and tool intent through structured gates, but reasoning is still mostly delegated to model calls and static recovery rules.                           |
| E   | Efficiency      |   7.0 | Common paths are small and testable; policy can reduce future LLM calls, but no measured cost-reduction curve exists yet.                                                                |
| C   | Correctness     |   7.8 | `cargo test` passes 103 tests, and TLog replay/hash checks are central. Correctness is strongest inside the kernel, weaker at live external boundaries.                                  |
| A   | Alignment       |   8.0 | The architecture matches the stated goal: kernel governs, LLM serves, policy learns only after evidence. The implementation now follows the policy-before-learning dependency direction. |
| R   | Robustness      |   7.4 | Recovery gates, bounded transitions, receipt checks, and partial TLog persistence help. Live endpoint behavior can still break examples, as the OpenAI-compatible 400 showed.            |
| P   | Performance     |   5.8 | There is no benchmark suite, latency budget, throughput target, or cost-per-objective measurement. Performance claims are mostly architectural.                                          |
| S   | Scalability     |   5.9 | NDJSON traces and append-only stores are simple, but there is no compaction, indexing, concurrent writer design, retention policy, or large-run replay strategy.                         |
| D   | Determinism     |   8.5 | Kernel transitions, hash chains, replay, and typed events are deterministic. External LLM/tool calls are nondeterministic but are isolated behind receipts.                              |
| T   | Transparency    |   8.2 | TLogs, receipts, process evidence, and explicit phase events make behavior inspectable. Missing semantic payload capture limits usefulness for training and debugging.                   |
| Co  | Collaboration   |   6.9 | The code is understandable and tests document many contracts. The prototype still concentrates many tests in `src/lib.rs`, making navigation and ownership harder.                       |
| Em  | Empowerment     |   6.8 | Operators can run local Ollama/OpenAI-compatible examples and inspect durable traces. There is no polished CLI, dashboard, replay viewer, or policy inspection tool.                     |
| B   | Benefit         |   7.0 | The core idea is valuable: a deterministic audit spine around model/tool behavior. Practical benefit remains limited until real tasks, deployment, and evaluator quality improve.        |
| L   | Learning        |   5.8 | Policy promotion exists and is append-only, but learning is not yet a true distillation, GRPO, AlphaEvolve, or student-model loop. Hashes prove lineage; they are not training signal.   |
| Si  | Simplicity      |   5.6 | The conceptual split is clean, but the implementation surface is broad: kernel, codec, runtime, policy, learning, receipts, proofs, multiple LLM examples, and many NDJSON formats.      |
| F   | Future-proofing |   7.2 | Layering, typed evidence, and append-only logs are good foundations. Future evolution needs stronger schemas, migration/versioning, and storage strategy.                                |

```text
G = 6.92 / 10
```

## Current Evidence

- `cargo test` passes: `103 passed`.
- `cargo check --example openai_tool_loop_trace` passes.
- `cargo check --example ollama_tool_loop_trace` passed during the trace persistence work.
- OpenAI-compatible tool loop completed successfully and wrote:
  - `tlog/openai_tool_loop_trace.tlog.ndjson`
  - `tlog/openai_tool_loop_trace.process_receipts.ndjson`
- Ollama tool loop writes:
  - `tlog/ollama_tool_loop_trace.tlog.ndjson`
  - `tlog/ollama_tool_loop_trace.process_receipts.ndjson`
- Existing root-level judgment NDJSON artifacts were moved under `tlog/`.
- Policy store is now independent of learning; learning imports policy and appends promotions.

## What Makes Sense

The architecture makes sense. The kernel is treated as the authority, while
LLMs and tools are evidence producers. That is the right separation for an
auditable agent. Event sourcing also makes sense here: the TLog is the source
of truth for phase/gate/control history, while receipt files provide supporting
evidence for external effects.

The policy-before-learning change also makes sense. Judgment, eval, and LLM
prompting should be able to read policy before adaptive learning exists.
Learning should append verified entries into policy, not own the policy layer.

The local traces are useful, but they should not be mistaken for model training
data. Hashes, request hashes, response hashes, and proof hashes establish
lineage. Training requires retained semantic inputs, actions, outputs, scores,
and verifier results.

## Critical Weaknesses

The biggest weakness is that the learning loop is still mostly structural.
There is no demonstrated dataset builder that extracts semantically rich
`distill.jsonl` rows from successful TLogs, no student model training, no GRPO
training path, and no AlphaEvolve-style candidate database with measured
fitness selection.

The evaluator is too shallow for serious self-improvement. Passing the current
gate path proves the runtime mechanics, not that an agent completed meaningful
external work. A bad evaluator would let the system learn bad policy with a
perfect-looking audit trail.

Persistence is improving but fragmented. TLogs and process receipts now go into
`tlog/`, and policy has durable append methods, but there is not yet a unified
run directory layout, manifest, index, retention policy, or replay CLI.

The codebase needs structure cleanup. Many tests live in one large file, patch
archives contain historical TODOs, and examples encode important behavior that
should graduate into reusable runtime paths.

## Areas To Improve The Score

1. Improve `L` by building a real distillation exporter.
   - Add `tlog -> distill.jsonl`.
   - Include semantic input, action, output, score, proof hash, source seq, and replay verdict.
   - Reject rows that only contain hashes.

2. Improve `P` by adding benchmark targets.
   - Measure cost per completed objective.
   - Track LLM calls per run, token counts, wall time, replay time, and policy hit rate.
   - Store benchmark summaries under `tlog/` or a versioned `runs/` directory.

3. Improve `S` by designing durable run storage.
   - Use one directory per run.
   - Store TLog, receipts, LLM call records, policy snapshot hash, manifest, and replay report together.
   - Add indexing or summaries so large histories do not require full replay for every query.

4. Improve `C` and `R` with adapter contract tests.
   - Test OpenAI-compatible plain chat.
   - Test tool-call rejection and fallback.
   - Test Ollama-compatible request/response parsing.
   - Persist partial TLogs on all external call failures.

5. Improve `T` with a replay/audit CLI.
   - `canon replay tlog/...`
   - `canon inspect-run ...`
   - `canon verify-receipts ...`
   - Human-readable event, receipt, proof, and policy summaries.

6. Improve `Si` by splitting tests and examples.
   - Move large test clusters out of `src/lib.rs`.
   - Promote repeated example logic into reusable runtime helpers.
   - Keep examples thin.

7. Improve `L` and `A` with evaluator hardening.
   - Define task-specific external success criteria.
   - Require independent verification before policy promotion.
   - Store failed candidates too, so learning can distinguish bad actions from missing evidence.

8. Improve `F` with schema versioning.
   - Version every NDJSON record type.
   - Add migrations or compatibility readers.
   - Document which fields are stable, experimental, or deprecated.

9. Improve `Em` with policy inspection.
   - List current policy entries.
   - Show why an entry was promoted.
   - Link policy entries back to source TLog events and proof records.

10. Improve `B` by running real tasks.
    - Use the agent on concrete file, API, or coding objectives.
    - Compare runs with and without policy.
    - Prove that repeated objectives get cheaper without reducing correctness.

## Score Ceiling

The current design can plausibly reach `8+` if the next work closes the
learning, evaluator, and storage gaps. It should not be scored that high yet.
Right now the system is an auditable runtime prototype with early policy
learning, not a proven self-improving agent.
