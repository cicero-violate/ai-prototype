# Code Quality Score Report

Generated: 2026-05-18  |  Schema version: 17  |  Crates: 5

## Aggregate Scores

```text
Architecture         = 9.3
Structure            = 5.8
Simplicity           = 6.6
Maintainability      = 10.0
Determinism          = 10.0
Coherency            = 8.2

G (geometric mean)   = 8.14 / 10
```

## Per-Crate Breakdown

| Crate          | Nodes | Edges | Fns  | Arch | Struct | Simple | Maint | Determ | Coher |
| :------------- | ----: | ----: | ---: | ---: | -----: | -----: | ----: | -----: | ----: |
| ai             |  6499 | 43268 | 2758 |  9.5 |    6.2 |    6.7 |  10.0 |   10.0 |   8.2 |
| browser_router |   640 |  6731 |  318 |  8.0 |    5.1 |    4.0 |  10.0 |   10.0 |   8.5 |
| browser_router |     2 |    74 |    2 |  5.0 |    6.1 |    0.7 |  10.0 |   10.0 |   7.0 |
| canon_rustc_v3 |   694 |  3708 |  336 |  8.3 |    3.6 |    8.5 |  10.0 |   10.0 |   8.0 |
| canon_rustc_v3 |     2 |    69 |    2 |  5.0 |    6.1 |    0.9 |  10.0 |   10.0 |   8.1 |

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
