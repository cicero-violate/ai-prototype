# Verified Evolution Loop

Use an AlphaEvolve-style loop: generate code candidates, run them, score them, keep the winners, and feed the winners back into the next prompt.

```text
seed program
→ prompt sampler
→ LLM mutation/crossover
→ candidate patch
→ sandbox run
→ evaluator
→ fitness score
→ program database
→ winner selection
→ next generation
```

Real names:

- **seed program**      : the current code, plan, policy, or action sequence.
- **candidate patch**   : one proposed mutation of the seed.
- **sandbox run**       : isolated execution of the candidate.
- **evaluator**         : tests, replay, proof checks, benchmarks, and eval rules.
- **fitness score**     : measured quality, not model opinion.
- **program database**  : archive of candidates, scores, receipts, and lineage.
- **selection**         : keep candidates that pass and improve the score.
- **next generation**   : use selected winners as context for more mutations.

Authority rule:

```text
LLM proposes candidates.
Sandbox executes candidates.
Evaluator scores candidates.
TLog records receipts.
Program database stores lineage.
Kernel gates deployment.
```

The LLM never approves itself. A candidate becomes learning data only when external evidence proves it passed. If the evaluator is subjective or can be steered by the same model that proposed the candidate, the loop is not verified evolution; it is only model self-review.

## TLog Distillation Loop

After verified evolution, convert only winning traces into learning data.

```text
winning candidate → receipt/proof → TLog → distill.jsonl → policy/student → gated reuse
```

Dataset rule:

```text
D = { E in TLog |
      E.eval.verdict = pass
  and E.replay.valid = true
  and E.score >= threshold
  and E.inputs are retained
  and E.outputs are semantically inspectable }
```

Hashes, receipts, and proof IDs prove lineage. They are not training signal by themselves. Training rows must retain the semantic input, selected action, observable output, measured score, proof hash, and source event.

Each `distill.jsonl` row must keep:

```text
(instruction, input_state, action, output, score, proof_hash, source_event)
```

Use the data in this order:

1. Promote simple repeated wins into policy.
2. Keep hard or novel wins as examples for retrieval.
3. Train a small student model only after the dataset is large and clean.
4. Serve the student through Ollama or another host.
5. Keep the same verifier, proof, replay, and kernel gates.

Ollama is not the student. Ollama is only a local model host. The student is the trained small model or adapter.

## LLM Promotion Ladder

**Stage one.** Policy is empty. The LLM answers every capability that requires reasoning. The TLog fills with LLM-produced structured evidence records.

**Stage two.** Learning reads the TLog and promotes confident patterns into policy. Capabilities check policy first. On a hit, no LLM call. On a miss, LLM call, record added, policy grows.

**Stage three.** Policy handles common cases. The LLM sees only novel situations. Calls become fewer and more targeted.

**Stage four.** The LLM is called to flag where new capabilities are needed. It operates at the architectural level. A human reviews and builds. The cycle repeats for the new domain.
