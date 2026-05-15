# Code Quality Score Report

Generated: 2026-05-15  |  Schema version: 16  |  Crates: 18

## Aggregate Scores

```text
Architecture         = 9.0
Structure            = 4.9
Simplicity           = 6.9
Maintainability      = 10.0
Determinism          = 10.0
Coherency            = 8.2

G (geometric mean)   = 7.93 / 10
```

## Per-Crate Breakdown

| Crate                      | Nodes | Edges | Fns  | Arch | Struct | Simple | Maint | Determ | Coher |
| :------------------------- | ----: | ----: | ---: | ---: | -----: | -----: | ----: | -----: | ----: |
| agent                      |     6 |   208 |    6 |  5.0 |    8.8 |    0.8 |  10.0 |   10.0 |   7.7 |
| ai                         |  5628 | 35770 | 2284 |  9.4 |    6.1 |    7.1 |  10.0 |   10.0 |   8.2 |
| ai                         |     1 |    18 |    1 |  5.0 |    4.5 |    4.5 |  10.0 |   10.0 |   7.0 |
| browser_router             |   656 |  5986 |  316 |  9.0 |    4.6 |    4.5 |  10.0 |   10.0 |   8.5 |
| browser_router             |     2 |    74 |    2 |  5.0 |    6.1 |    0.7 |  10.0 |   10.0 |   7.0 |
| chatgpt_mcp_connector      |  3714 | 21661 | 1704 |  9.0 |    3.6 |    7.4 |  10.0 |   10.0 |   8.3 |
| graph_mutation             |    10 |   249 |   10 |  5.0 |    7.8 |    2.2 |  10.0 |   10.0 |   8.0 |
| kernel_tlog                |    18 |   180 |    7 |  5.0 |    8.5 |    2.7 |   9.3 |   10.0 |   9.1 |
| loop_trace                 |     1 |    36 |    1 |  5.0 |    4.5 |    0.8 |  10.0 |   10.0 |   7.0 |
| ollama_judgment            |     4 |   239 |    4 |  5.0 |    7.7 |    0.1 |  10.0 |   10.0 |   8.6 |
| ollama_loop_trace          |     2 |    70 |    2 |  5.0 |    6.1 |    0.8 |  10.0 |   10.0 |   7.0 |
| ollama_tool_loop_trace     |    12 |   238 |    9 |  5.0 |    9.2 |    2.0 |  10.0 |   10.0 |   8.1 |
| ollama_tool_mcp_loop_trace |    17 |   311 |   11 |  5.0 |    9.3 |    1.7 |  10.0 |   10.0 |   8.1 |
| openai_tool_loop_trace     |    15 |   308 |   12 |  5.0 |    8.9 |    2.2 |  10.0 |   10.0 |   7.8 |
| root_validate              |   163 |  1990 |  161 |  3.3 |    1.5 |    6.1 |  10.0 |   10.0 |   7.3 |
| score                      |    90 |   685 |   54 |  8.3 |    4.4 |    6.5 |  10.0 |   10.0 |   8.2 |
| supervisor                 |   115 |   778 |   38 |  9.8 |    5.7 |    4.1 |   9.9 |   10.0 |   8.6 |
| tlog_introspect            |     1 |    49 |    1 |  5.0 |    4.5 |    0.2 |  10.0 |   10.0 |   7.0 |

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
