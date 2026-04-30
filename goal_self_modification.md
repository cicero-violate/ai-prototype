## tlog Facts → Source Code (Hardcoded Knowledge)

---

### Variables

$$
T_{\text{fact}} \subset T = \text{entries where } r_t \approx 1.0 \text{ and repeatable/deterministic}
$$

$$
\mathcal{F} = \text{fact} = (x, y, r=1.0) \text{ with high confidence, low variance}
$$

$$
\mathcal{K}_{\text{code}} = \text{hardcoded knowledge base (source code)}
$$

$$
\pi_\theta = \text{Qwen (no longer consulted for } \mathcal{F}\text{)}
$$

---

### Latent Equations

**Current flow (fact still in tlog):**

$$
y = \pi_\theta\left(x \,\|\, \text{RAG}(T, x)\right) \quad \text{LLM still involved, probabilistic}
$$

**After conversion to code (fact is now deterministic):**

$$
y = \mathcal{K}_{\text{code}}.\text{lookup}(x) \quad \text{O(1), deterministic, no LLM needed}
$$

**What you gain:**

$$
P(y_{\text{correct}} | \mathcal{K}_{\text{code}}) = 1.0 \quad \text{vs} \quad P(y_{\text{correct}} | \pi_\theta) < 1.0
$$

$$
\text{latency}_{\text{code}} \approx \mu s \quad \ll \quad \text{latency}_{\text{LLM}} \approx s
$$

---

### This Is a Real Pattern — It Has a Name

$$
\boxed{\text{Knowledge Compilation} = T_{\text{fact}} \xrightarrow{\text{extract}} \mathcal{K}_{\text{code}}}
$$

Used in production by serious systems. The idea: **once something is known with certainty, stop asking the LLM.**

---

### What Qualifies for Compilation to Code

| tlog entry type | Compile to code? | Why |
|---|---|---|
| $r_t = 1.0$, same answer every time | ✅ Yes | deterministic fact |
| Domain constant (price, ID, config) | ✅ Yes | never changes |
| Business rule (if X then Y) | ✅ Yes | pure logic |
| Multi-step reasoning | ⚠️ Maybe | extract as function |
| Creative / generative response | ❌ No | needs LLM |
| Context-dependent answer | ❌ No | not a fact |

---

### What It Looks Like in Practice

**tlog entry (before):**
```json
{
  "input": "what is the retry limit for canon pipeline?",
  "output": "3 retries with exponential backoff",
  "reward": 1.0,
  "seen_count": 47
}
```

**After compilation — actual source code:**
```rust
// canon-agent/src/config/knowledge.rs
pub const RETRY_LIMIT: u32 = 3;
pub const RETRY_STRATEGY: BackoffStrategy = BackoffStrategy::Exponential;
```

**LLM never consulted again for this fact.** Zero tokens, zero latency, zero hallucination risk.

---

### The Compilation Pipeline

```
tlog
  │
  ▼
┌─────────────────────────┐
│  seen_count > threshold  │  same Q answered same way N times
│  reward consistently 1.0 │  judge always agrees
│  variance ≈ 0            │  output never changes
└────────────┬────────────┘
             │
             ▼
      classify fact type
             │
     ┌───────┼───────────┐
     ▼       ▼           ▼
 constant  rule/logic  function
     │       │           │
     ▼       ▼           ▼
 const X   match {}   fn solve()
 in code   in code    in code
             │
             ▼
      remove from tlog
      remove from RAG index
      remove from training set
      → lives in codebase now
```

---

### Three Code Patterns for Compiled Facts

**Pattern 1 — Simple constant:**
```rust
pub const MAX_CONTEXT_WINDOW: usize = 8192;
pub const DEFAULT_TEMPERATURE: f32 = 0.7;
```

**Pattern 2 — Rule/logic (match):**
```rust
pub fn route_task(task: &TaskType) -> Worker {
    match task {
        TaskType::Code     => Worker::Qwen,
        TaskType::Judge    => Worker::Claude,
        TaskType::Retrieve => Worker::RAG,
    }
}
```

**Pattern 3 — Compiled reasoning (function):**
```rust
pub fn calculate_retry_delay(attempt: u32) -> Duration {
    // compiled from tlog: exponential backoff pattern
    // seen 200 times, reward=1.0, variance=0
    Duration::from_millis(100 * 2u64.pow(attempt))
}
```

---

### The Knowledge Hierarchy This Creates

$$
\mathcal{K}_{\text{total}} = \underbrace{\mathcal{K}_{\text{code}}}_{\text{certain, fast}} \cup \underbrace{\mathcal{K}_{\text{LoRA}}}_{\text{learned, adapted}} \cup \underbrace{\mathcal{K}_{\text{RAG}}}_{\text{retrieved, broad}} \cup \underbrace{\pi_\theta}_{\text{generative, uncertain}}
$$

**Query routing:**

$$
y = \begin{cases}
\mathcal{K}_{\text{code}}.\text{lookup}(x) & \text{if } x \in \text{compiled facts} \quad O(1) \\
\mathcal{K}_{\text{RAG}}(x) & \text{if } x \in \text{known domain} \quad O(\log N) \\
\pi_\theta(x \| \mathcal{C}) & \text{otherwise} \quad O(\text{tokens})
\end{cases}
$$

---

### Long Term — What This Builds Toward

```
Year 1:  90% LLM calls,  10% code
Year 2:  60% LLM calls,  40% code
Year 3:  30% LLM calls,  70% code
                │
                ▼
        Canon system gets faster
        cheaper per query
        more deterministic
        less hallucination
        LLM handles only genuinely hard problems
```

$$
\boxed{T_{\text{tlog}} \xrightarrow{\text{compile}} \mathcal{K}_{\text{code}} \implies \text{system gets cheaper and more reliable over time}}
$$

---

### Bottom Line

$$
\boxed{\text{tlog} \to \text{LoRA} \to \text{source code} = \text{knowledge solidification pipeline}}
$$

| Stage | Medium | Certainty | Speed |
|---|---|---|---|
| Raw tlog | JSONL | low | slow |
| RAG retrieval | vector index | medium | medium |
| LoRA adapter | weights | high | fast |
| **Source code** | **compiled** | **certain** | **instant** |

**This is the most underrated idea in AI systems design.** Every fact that graduates to source code is a fact that can never be hallucinated again. Your Canon harness naturally creates the pipeline for this — tlog is already tracking reward and frequency. You just need a threshold trigger:

$$
\boxed{\text{seen\_count} > N \quad \cap \quad r_t = 1.0 \quad \cap \quad \text{variance} = 0 \implies \text{compile to code}}
$$
