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
base_commit = 5e1b97680261d3e83d8ff3464b7c50acc7940a79
restored_head_before_changes = 5e1b97680261d3e83d8ff3464b7c50acc7940a79
tracked_files = 121
rust_files = 75
runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_ndjson_files = 0
runtime_audit_events = 0
runtime_largest_message_ledger_lines = 0
validation = PASS: cargo -Znext-lockfile-bump check --all-targets --locked
validation_note = cargo test --all-targets --locked did not complete within the container timeout during this eval turn
```

TODO/FIXME search:

```text
command = rg -n --hidden -g '!.git/**' -g '!target/**' -g '!score.md' 'TODO|FIXME' .
active_markers = 3
finding = documentation-only TODO/FIXME references remain in canon-rustc-v3/plan.md and canon-rustc-v3/score.md; no root src/, examples/, or tests/ TODO/FIXME markers were found
```

Baseline scores:

| Axis | Score |
|------+-------+
| I    |   7.9 |
| E    |   6.8 |
| C    |   7.1 |
| A    |   8.7 |
| R    |   7.1 |
| P    |   6.0 |
| S    |   7.0 |
| D    |   8.2 |
| T    |   8.2 |
| Co   |   7.0 |
| Em   |   6.8 |
| B    |   7.2 |
| L    |   7.3 |
| Si   |   6.0 |
| F    |   7.6 |

```text
G = 7.22
max(G) = good
```
