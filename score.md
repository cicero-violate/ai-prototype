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
base_commit = 8a60ea0
restored_head_before_changes = 8a60ea0
tracked_files = 123
rust_files = 78
runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_ndjson_files = 0
runtime_audit_events = 0
runtime_largest_message_ledger_lines = 0
validation = PASS: cargo -Znext-lockfile-bump run --bin root_validate --locked
validation_note = root_validate passed root cargo check, bounded score_contract tests, and canon-rustc-v3 graph telemetry receipt generation
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
| I    |   8.1 |
| E    |   7.2 |
| C    |   7.6 |
| A    |   8.8 |
| R    |   7.6 |
| P    |   6.4 |
| S    |   7.2 |
| D    |   8.6 |
| T    |   8.7 |
| Co   |   7.1 |
| Em   |   7.2 |
| B    |   7.5 |
| L    |   7.6 |
| Si   |   6.2 |
| F    |   7.9 |

```text
G = 7.54
max(G) = good
```
