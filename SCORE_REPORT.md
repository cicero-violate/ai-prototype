# Code Quality Score Report

Generated: 2026-05-13  |  Schema version: 16  |  Crates: 1

## Aggregate Scores

```text
Architecture         = 8.7
Structure            = 3.5
Simplicity           = 7.9
Maintainability      = 10.0
Determinism          = 10.0
Coherency            = 8.8

G (geometric mean)   = 7.75 / 10
```

## Per-Crate Breakdown

| Crate                 | Nodes | Edges | Fns  | Arch | Struct | Simple | Maint | Determ | Coher |
| :-------------------- | ----: | ----: | ---: | ---: | -----: | -----: | ----: | -----: | ----: |
| chatgpt_mcp_connector |  3705 | 20084 | 1695 |  8.7 |    3.5 |    7.9 |  10.0 |   10.0 |   8.8 |

## Axis Definitions

| Axis            | Formula                                                              | Graph signal                          |
| :-------------- | :------------------------------------------------------------------- | :------------------------------------ |
| Architecture    | edges/nodes coupling (peak 7) + trait+impl/total abstraction ratio   | coupling density, abstraction ratio   |
| Structure       | blend of call-in coverage and 1 − call in-degree gini                | call-in coverage, call in-degree gini |
| Simplicity      | exponential fanout penalty (>5) × (1 − duplicate-pair ratio)         | mean call fanout, similar-edge ratio  |
| Maintainability | phase-decomposition coverage × (1 − duplication pressure)            | phase edge coverage, similar ratio    |
| Determinism     | intent coverage × (1 − 2×violation rate for pure+risk conflicts)     | pure fns with risk edges              |
| Coherency       | 0.7×coverage + 0.3×normalized Shannon entropy of intent classes      | intent coverage, intent entropy       |

*Scores are structural proxies from graph.json topology and intent classification. They do not capture test coverage, runtime correctness, or domain semantics.*
