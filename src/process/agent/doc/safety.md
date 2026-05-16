# Agent Safety and Correctness

## Rule

The agent can propose and coordinate actions. It cannot bypass runtime,
capability, receipt, verification, or eval boundaries.

## Hard Boundaries

```text
No direct State mutation.
No direct TLog append.
No unreceipted tool execution.
No unstructured LLM authority.
No policy promotion without verified eval.
No live finance/trading execution.
No hidden self-modification.
```

## External Effects

Every external effect needs:

- request hash,
- provider/tool identity hash,
- timeout budget,
- output byte bound,
- effect receipt,
- verification path,
- TLog event binding.

## LLM Output

LLM output is not truth. It is a candidate signal.

Accepted path:

```text
LLM output
→ structured capability record
→ evidence submission
→ runtime event
→ verification/eval
```

Rejected path:

```text
LLM output
→ direct state mutation
```

## Tool Output

Tool output is not accepted without a receipt.

Accepted path:

```text
tool request
→ tool receipt
→ receipt verification
→ EvidenceSubmission
→ TLog
```

Rejected path:

```text
tool request
→ side effect
→ unrecorded success claim
```

## Self-Modification

Self-modification must remain behind the supervisor/worker boundary.

```text
agent proposes patch
tool applies patch with receipt
tests/eval verify patch
worker rebuilds
supervisor reloads worker
new worker resumes from durable TLog
```

## TODO

- [ ] Define human-review-required action classes.
- [ ] Define blocked action classes.
- [ ] Define external effect receipt requirements.
- [ ] Define self-modification review path.
- [ ] Define finance/trading hard blocks.
- [ ] Define stale-data handling.
- [ ] Define contradiction handling.
- [ ] Define prompt-injection handling for observed text.
