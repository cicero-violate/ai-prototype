# Code Quality Score Report

Generated: 2026-05-12  |  Schema version: 16  |  Crates: 16

## Aggregate Scores

```text
Architecture         = 9.0
Structure            = 4.8
Simplicity           = 7.1
Maintainability      = 10.0
Determinism          = 10.0
Coherency            = 8.2

G (geometric mean)   = 7.93 / 10
```

## Per-Crate Breakdown

| Crate                      | Nodes | Edges | Fns  | Arch | Struct | Simple | Maint | Determ | Coher |
| :------------------------- | ----: | ----: | ---: | ---: | -----: | -----: | ----: | -----: | ----: |
| agent                      |     5 |   195 |    5 |  5.0 |    8.5 |    0.5 |  10.0 |   10.0 |   7.8 |
| ai                         |  5419 | 34722 | 2174 |  9.5 |    6.0 |    7.0 |  10.0 |   10.0 |   8.1 |
| ai                         |     1 |    18 |    1 |  5.0 |    4.5 |    4.5 |  10.0 |   10.0 |   7.0 |
| chatgpt_mcp_connector      |  3651 | 20573 | 1655 |  8.9 |    3.4 |    7.5 |  10.0 |   10.0 |   8.3 |
| graph_mutation             |    10 |   249 |   10 |  5.0 |    7.8 |    2.2 |  10.0 |   10.0 |   8.0 |
| loop_trace                 |     1 |    36 |    1 |  5.0 |    4.5 |    0.8 |  10.0 |   10.0 |   7.0 |
| ollama_judgment            |     4 |   239 |    4 |  5.0 |    7.7 |    0.1 |  10.0 |   10.0 |   8.6 |
| ollama_loop_trace          |     2 |    70 |    2 |  5.0 |    6.1 |    0.8 |  10.0 |   10.0 |   7.0 |
| ollama_tool_loop_trace     |    10 |   220 |    7 |  5.0 |    8.9 |    1.2 |  10.0 |   10.0 |   8.0 |
| ollama_tool_mcp_loop_trace |    15 |   293 |    9 |  5.0 |    9.2 |    1.2 |  10.0 |   10.0 |   8.1 |
| openai_tool_loop_trace     |    13 |   283 |   10 |  5.0 |    8.7 |    1.7 |  10.0 |   10.0 |   7.7 |
| root_validate              |   162 |  1971 |  160 |  3.3 |    1.5 |    6.2 |  10.0 |   10.0 |   7.3 |
| score                      |    90 |   685 |   54 |  8.3 |    4.4 |    6.5 |  10.0 |   10.0 |   8.2 |
| supervisor                 |   114 |   709 |   37 |  9.4 |    5.6 |    4.7 |   9.9 |   10.0 |   8.6 |
| tlog_introspect            |     1 |    49 |    1 |  5.0 |    4.5 |    0.2 |  10.0 |   10.0 |   7.0 |
| worker                     |    18 |   180 |    7 |  5.0 |    8.5 |    2.7 |   9.3 |   10.0 |   9.1 |

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
