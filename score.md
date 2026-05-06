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
base_commit = 449ca9f823e7ed2e24e5d49a5bf1a467d04391ac
restored_head_before_changes = 449ca9f823e7ed2e24e5d49a5bf1a467d04391ac
tracked_files = 121
rust_files = 75
runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_ndjson_files = 15
runtime_audit_events = 48
runtime_largest_message_ledger_lines = 318
```

TODO/FIXME search:

```text
command = rg -n --hidden -g '!.git/**' -g '!target/**' -g '!score.md' 'TODO|FIXME' .
active_markers = 1
finding = canon-rustc-v3/plan.md:39 references TODO/FIXME evidence in prior score work
```

Baseline scores:

| Axis | Score |
|------+-------+
| I    |   7.4 |
| E    |   6.1 |
| C    |   6.6 |
| A    |   8.6 |
| R    |   6.5 |
| P    |   5.7 |
| S    |   6.7 |
| D    |   7.9 |
| T    |   7.7 |
| Co   |   6.8 |
| Em   |   6.5 |
| B    |   7.0 |
| L    |   6.9 |
| Si   |   5.8 |
| F    |   7.2 |

```text
G = 6.85
max(G) = good
```
