# Code Quality Score Report

Generated: 2026-05-13  |  Schema version: 16  |  Crates: 14

## Aggregate Scores

```text
Architecture         = 8.8
Structure            = 4.8
Simplicity           = 7.5
Maintainability      = 10.0
Determinism          = 10.0
Coherency            = 8.7

G (geometric mean)   = 8.05 / 10
```

## Per-Crate Breakdown

| Crate                 | Nodes | Edges | Fns  | Arch | Struct | Simple | Maint | Determ | Coher |
| :-------------------- | ----: | ----: | ---: | ---: | -----: | -----: | ----: | -----: | ----: |
| agent                 |     5 |   177 |    5 |  5.0 |    8.5 |    0.8 |  10.0 |   10.0 |   7.0 |
| ai                    |  5472 | 33868 | 2208 |  9.3 |    6.0 |    7.6 |  10.0 |   10.0 |   8.6 |
| ai                    |     1 |    17 |    1 |  5.0 |    4.5 |    5.0 |  10.0 |   10.0 |   7.0 |
| browser_router        |   642 |  5471 |  304 |  9.2 |    4.5 |    4.9 |  10.0 |   10.0 |   8.8 |
| browser_router        |     2 |    73 |    2 |  5.0 |    6.1 |    0.7 |  10.0 |   10.0 |   7.0 |
| canon_rustc_v3        |   352 |  2176 |  177 |  8.9 |    4.3 |    7.6 |  10.0 |   10.0 |   8.1 |
| canon_rustc_v3        |     2 |    69 |    2 |  5.0 |    6.1 |    0.9 |  10.0 |   10.0 |   8.1 |
| chatgpt_mcp_connector |  3693 | 19733 | 1683 |  8.6 |    3.5 |    8.0 |  10.0 |   10.0 |   8.8 |
| graph_mutation        |    10 |   232 |   10 |  5.0 |    7.8 |    2.6 |  10.0 |   10.0 |   8.0 |
| root_validate         |   162 |  1872 |  160 |  3.7 |    1.5 |    6.8 |   9.8 |   10.0 |   8.3 |
| score                 |    90 |   685 |   54 |  8.3 |    4.4 |    6.5 |  10.0 |   10.0 |   8.2 |
| supervisor            |   115 |   739 |   38 |  9.5 |    5.7 |    4.5 |  10.0 |   10.0 |   8.7 |
| tlog_introspect       |     1 |    43 |    1 |  5.0 |    4.5 |    0.4 |  10.0 |   10.0 |   7.0 |
| worker                |    18 |   175 |    7 |  5.0 |    8.5 |    3.1 |  10.0 |   10.0 |   8.7 |

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
