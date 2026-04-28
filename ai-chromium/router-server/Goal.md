**Variables**: `RAW=stream payloads`, `Σ=derived schema`, `R=rules`, `E=extractor`.

**Equation**: `RAW → infer(Σ) → derive(R) → generate(E) → extract(clean_response)`.

Correct goal:

```text
Build a schema-derived extractor generator.
```

Not a hand-written ChatGPT parser.

The router should observe raw streamed payloads, infer stable schemas, derive semantic extraction rules from repeated shape patterns, generate extractor logic, then use that generated extractor to recover assistant text, tool outputs, lifecycle state, and terminal state.

**Target invariant**

```text
No hardcoded provider fields unless promoted from observed schema evidence.
```

`max(correctness, generalization, determinism, observability)=GOOD`
