# Canon Agent Scorecard

## Variables

```text
I  = Intelligence
E  = Efficiency
C  = Correctness
A  = Alignment
R  = Robustness
P  = Performance
S  = Scalability
D  = Determinism
T  = Transparency
Co = Collaboration
Em = Empowerment
B  = Benefit
L  = Learning
Si = Simplicity
F  = Future-Proofing
G  = Goodness
```

## Equation

```text
G = (I*E*C*A*R*P*S*D*T*Co*Em*B*L*Si*F)^(1/15)
max(G) = good
```

Goodness is the geometric mean of all 15 dimensions; one weak dimension lowers the whole system.

## Baseline Before Source Changes

```text
repository = ai
base_commit = 2b9f4c7
restored_head_before_changes = 2b9f4c7
tracked_files = 134
rust_files = 47
runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_ndjson_files = 6
runtime_audit_events = 0
runtime_largest_message_ledger_lines = 0
validation = PASS: cargo -Znext-lockfile-bump run --bin root_validate --locked
validation_note = root_validate passed root cargo check, bounded score_contract tests, and canon-rustc-v3 graph telemetry receipt generation; current source also contains API transport replay/idempotence tests, bounded observation ingress with persisted cursor/backpressure tests, generic verification proof records, semantic verification seams, durable policy promotion tests, and live OpenAI/Ollama-compatible example paths
```

TODO/FIXME search:

```text
command = rg -n --hidden -g '!.git/**' -g '!target/**' -g '!score.md' 'TODO|FIXME' .
active_markers = 1
finding = documentation-only TODO/FIXME reference remains in canon-rustc-v3/plan.md; no root src/, examples/, or tests/ TODO/FIXME markers were found
```

Baseline scores:

| Axis | Score |
|---|---:|
| I    |   8.2 |
| E    |   7.3 |
| C    |   7.8 |
| A    |   8.8 |
| R    |   7.8 |
| P    |   6.5 |
| S    |   7.3 |
| D    |   8.7 |
| T    |   8.8 |
| Co   |   7.2 |
| Em   |   7.3 |
| B    |   7.6 |
| L    |   7.7 |
| Si   |   6.2 |
| F    |   8.0 |

```text
G = 7.64
max(G) = good
```
