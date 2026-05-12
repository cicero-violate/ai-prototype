# Code Quality Score Report

Generated: 2026-05-11  |  Schema version: 16  |  Crates: 15

## Aggregate Scores

```text
Architecture         = 9.1
Structure            = 1.6
Simplicity           = 6.0
Maintainability      = 7.7
Determinism          = 10.0
Coherency            = 8.2

G (geometric mean)   = 6.15 / 10
```

## Per-Crate Breakdown

| Crate                      | Nodes | Edges | Fns  | Arch | Struct | Simple | Maint | Determ | Coher |
| :------------------------- | ----: | ----: | ---: | ---: | -----: | -----: | ----: | -----: | ----: |
| agent                      |     2 |    66 |    2 |  5.0 |    8.0 |    0.9 |  10.0 |   10.0 |   8.1 |
| ai                         |  5297 | 33821 | 3423 |  9.5 |    1.6 |    4.8 |   6.3 |   10.0 |   8.1 |
| ai                         |     1 |    18 |    1 |  5.0 |    7.8 |    4.5 |  10.0 |   10.0 |   7.0 |
| chatgpt_mcp_connector      |  3651 | 20573 | 2127 |  8.9 |    0.9 |    8.0 |   9.6 |   10.0 |   8.3 |
| graph_mutation             |    10 |   249 |   10 |  5.0 |    6.4 |    2.2 |  10.0 |   10.0 |   8.0 |
| loop_trace                 |     1 |    36 |    1 |  5.0 |    8.0 |    0.8 |  10.0 |   10.0 |   7.0 |
| ollama_judgment            |     4 |   239 |    4 |  5.0 |    7.5 |    0.1 |  10.0 |   10.0 |   8.6 |
| ollama_loop_trace          |     2 |    70 |    2 |  5.0 |    7.9 |    0.8 |  10.0 |   10.0 |   7.0 |
| ollama_tool_loop_trace     |    10 |   220 |    8 |  5.0 |    7.9 |    1.7 |  10.0 |   10.0 |   8.0 |
| ollama_tool_mcp_loop_trace |    15 |   293 |   11 |  5.0 |    7.4 |    1.9 |  10.0 |   10.0 |   8.1 |
| openai_tool_loop_trace     |    13 |   283 |   11 |  5.0 |    7.0 |    2.1 |  10.0 |   10.0 |   7.7 |
| root_validate              |   159 |  1938 |  157 |  3.3 |    5.7 |    6.1 |  10.0 |   10.0 |   7.3 |
| score                      |    88 |   627 |   53 |  8.4 |    6.0 |    7.0 |  10.0 |   10.0 |   8.2 |
| supervisor                 |   111 |   630 |   60 |  9.0 |    5.1 |    8.1 |  10.0 |   10.0 |   8.6 |
| worker                     |    18 |   180 |   11 |  5.0 |    7.0 |    4.6 |  10.0 |   10.0 |   9.1 |

## Axis Definitions

| Axis            | Formula                                                              | Graph signal                         |
| :-------------- | :------------------------------------------------------------------- | :----------------------------------- |
| Architecture    | edges/nodes coupling (peak 7) + trait+impl/total abstraction ratio   | coupling density, abstraction ratio  |
| Structure       | 10 × (1 − gini coefficient of call in-degree distribution)           | call in-degree gini                  |
| Simplicity      | exponential fanout penalty (>5) × (1 − duplicate-pair ratio)         | mean call fanout, similar-edge ratio |
| Maintainability | phase-decomposition coverage × (1 − duplication pressure)            | phase edge coverage, similar ratio   |
| Determinism     | intent coverage × (1 − 2×violation rate for pure+risk conflicts)     | pure fns with risk edges             |
| Coherency       | 0.7×coverage + 0.3×normalized Shannon entropy of intent classes      | intent coverage, intent entropy      |

*Scores are structural proxies from graph.json topology and intent classification. They do not capture test coverage, runtime correctness, or domain semantics.*
