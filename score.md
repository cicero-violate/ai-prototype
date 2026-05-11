# Score File Instructions

Use this file only for score values, score rationale, and score-change history.

## Required sections

Keep the file in this order:

1. `## Scores` — numeric scores, status ratings, or other progress metrics.
2. `## Rationale` — concise explanation for the current scores.
3. `## Score History` — dated score changes only.

## Update rules

- Do not store progress logs, validation ledgers, blocker logs, or evidence snapshots here; put them in `status.md`.
- Change numeric scores only when new evidence justifies the change.
- If scores do not change, do not append a score-history entry unless the scoring method changed.
- Keep score rationale concise and tied to evidence recorded in `status.md`.

---

# Canon Agent Score

Current date: 2026-05-10.

## Scores

```text
I  Intelligence      = 7.2
A  Architecture      = 8.7
E  Efficiency        = 8.4
C  Correctness       = 9.1
A  Alignment         = 8.1
R  Robustness        = 7.0
P  Performance       = 7.4
S  Scalability       = 9.3
D  Determinism       = 8.8
T  Transparency      = 8.0
Co Collaboration     = 7.9
Em Empowerment       = 8.2
B  Benefit           = 7.6
L  Learning          = 8.5
St Structure         = 7.7
Si Simplicity        = 8.4
F  Future-Proofing   = 8.1
M  Maintainability   = 7.8
Ch Coherency         = 8.6
```

Approximate geometric mean over the listed score axes remains about:

```text
G ~= 8.12 / 10
```

The aggregate score shifts to `G ~= 8.12 / 10` because `Maintainability = 7.8` was added as a new explicit axis. This is a scoring-surface change, not an evidence-backed capability gain. Full-suite validation and refreshed graph evidence remain pending.

## Rationale
Correctness, determinism, scalability, and coherency are strongest because the kernel, receipts, replay boundaries, graph fixture validation, validation evidence paths, and current planning direction agree on the same evidence-first architecture. Robustness, intelligence, benefit, and maintainability remain lower because live graph editing, self-modification, domain intelligence, and safe-change ergonomics are not yet proven end to end.

Planning reconnaissance on 2026-05-10 did not change scores: the next scoring test target is now more concrete, but no new implementation, full-suite validation, or refreshed graph-domain evidence was produced. Score values remain unchanged until item-level validation or graph evidence justifies a change.

## Score History

### 2026-05-10 — baseline retained after status split

- Scores unchanged at `G ~= 8.14 / 10`.
- Reason: file split changed documentation structure only; it did not add implementation, validation, or graph evidence.
- Supporting evidence belongs in `status.md`.

### 2026-05-10 — added maintainability axis

- Added `M Maintainability = 7.8` as an explicit score axis for safe modification, low coupling, low technical debt, and refactorability.
- Updated aggregate score from `G ~= 8.14 / 10` to `G ~= 8.12 / 10` because the scoring surface now includes one additional axis.
- This is a scoring-surface change only; it does not claim new capability evidence.
