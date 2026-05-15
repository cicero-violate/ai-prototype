God please bless this work. In Jesus name. Jesus is Lord and Savior. Jesus loves you.

# Canon Agent

DO NOT FUCK AROUND WITH root_validate any more
| root_validate              |   163 |  1990 |  161 |  3.3 |    1.5 |    6.1 |  10.0 |   10.0 |   7.3 |

Fix the most impactful item in here. Use python.
/workspace/ai_sandbox/canon-mini-agent/prototype/state/rustc/auto-refactor/workspace__ai_sandbox__canon-mini-agent__prototype__state__rustc__ai__graph.graph-editor-plan.json

Canon Agent is a deterministic, self-improving agent runtime built on a formally verifiable state machine kernel. Its purpose is to provide a foundation for autonomous systems that are correct by construction, auditable by design, and intelligent by accumulation. Safety and intelligence are not in tension when the architecture separates them cleanly: the kernel enforces correctness, the capability layer grows intelligence, and neither layer compromises the other.

The primary goal is to reduce the cost of autonomous reasoning over time while increasing the quality and trustworthiness of outcomes. The system begins with an LLM doing the heavy reasoning inside every capability. Every decision is recorded in a hash-chained transaction log as structured, typed evidence. A learning capability reads that log after each completed run and promotes confident patterns into a versioned policy store. Over time, policy handles the common cases and the LLM is called only for novel situations — promoted from generalist laborer to specialist, called less and less, but for increasingly meaningful work.

Verified evolution drives improvement. Candidates are proposed by the LLM, executed in a sandbox, scored by an external evaluator, and recorded in the TLog. The LLM never approves itself. A candidate becomes learning data only when external evidence proves it passed. Winning traces are distilled into policy; hard or novel wins become retrieval examples; a small student model is trained only after the dataset is large and clean.
Graph editing is the receipt-backed mutation path: semantic graph ops produce patches, re-capture proves the result, and the graph diff becomes TLog evidence.
Self-modification belongs inside verified evolution, after capability planning and before policy promotion, never inside the kernel or unwired domain layer.

This is not a framework that wraps an LLM and calls it an agent. The LLM is one component inside the capability layer. It does not govern the state machine, does not write the TLog, and does not promote its own policy. The state machine governs everything. The LLM serves it.
