**Variables:**
`G = goodness`, `C = correctness`, `I = insight`, `E = efficiency`, `D = determinism`, `A = alignment`, `R = replayability`, `S = simplicity`.

**Equation:**
[
G=\max(C,I,E,D,A,R,S)
]
One dominant dimension can unlock the rest.

**Maximize rustc-wrapper goodness:**

1. **Truth extractor:** emit canonical facts only: `fn`, `trait`, `impl`, `call`, `mut`, `io`, `unsafe`, `panic`, `alloc`.
2. **Delta engine:** compare `graph_old → graph_new`; score only meaningful semantic changes.
3. **Intent classifier:** tag every function: `pure | io | mutation | orchestration | validation | unsafe | boundary`.
4. **Invariant gate:** deny commits when semantic risk rises without tests/proofs.
5. **Compression layer:** delete derivable edges; store minimal facts + reconstruction rules.
6. **Replay spine:** every compile emits `receipt_hash`, `graph_hash`, `intent_hash`, `risk_hash`.
7. **Leverage ranking:** rank files by `impact / complexity`, so agents patch the highest-yield surface first.
8. **Proof bridge:** send high-risk deltas to Lean/spec checks, not the whole repo.
9. **Agent prompt feed:** inject only top changed facts into planner/executor prompts.
10. **Goodness score:**
    [
    G=\frac{Correctness \times Insight \times Determinism \times Replayability}{Verbosity \times Redundancy \times Guesswork}
    ]

English: make the wrapper less like a logger and more like a semantic compiler witness: it should expose what changed, why it matters, what risk increased, and what proof/test is required next.
