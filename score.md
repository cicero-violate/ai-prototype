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

Current date: 2026-05-11.

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

The graph-derived structural score report now reports `G = 7.93 / 10` after the scorer excludes synthetic derived trait shims from structural duplicate-pressure accounting. This is a measurement-quality improvement, not a claim that runtime capability increased by itself. Full-suite validation remains separately gated by item 50.

## Rationale
Correctness, determinism, scalability, and coherency are strongest because the kernel, receipts, replay boundaries, graph fixture validation, validation evidence paths, and current planning direction agree on the same evidence-first architecture. Robustness, intelligence, benefit, and maintainability remain lower because live graph editing, self-modification, domain intelligence, and safe-change ergonomics are not yet proven end to end.

Fast validation on 2026-05-11 passed after the score-report fix: semantic spine static validation passed, the score report regenerated with `G = 7.93 / 10`, Simplicity improved to `7.1`, Maintainability improved to `10.0`, and the post-validation supervisor reload succeeded on port 9100. Item 50 full-suite validation remains a separate release gate.

## Score History

### 2026-05-11 — graph-derived score improved after scorer noise reduction

- Graph-derived `SCORE_REPORT.md` aggregate improved from `G = 7.14 / 10` to `G = 7.93 / 10`.
- Simplicity improved from `4.9` to `7.1`; Maintainability improved from `7.7` to `10.0`; Determinism remained `10.0`.
- Reason: duplicate-pressure scoring now applies the same synthetic derived trait shim filter used for structural function counting, preventing generated `Clone`/`Debug`/`Eq`/`PartialEq` methods from distorting duplicate-pressure metrics.
- Validation: fast validation passed and post-validation supervisor reload reported `supervisor reload: pass port=9100`.

### 2026-05-10 — baseline retained after status split

- Scores unchanged at `G ~= 8.14 / 10`.
- Reason: file split changed documentation structure only; it did not add implementation, validation, or graph evidence.
- Supporting evidence belongs in `status.md`.

### 2026-05-10 — added maintainability axis

- Added `M Maintainability = 7.8` as an explicit score axis for safe modification, low coupling, low technical debt, and refactorability.
- Updated aggregate score from `G ~= 8.14 / 10` to `G ~= 8.12 / 10` because the scoring surface now includes one additional axis.
- This is a scoring-surface change only; it does not claim new capability evidence.
