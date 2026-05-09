# Future Capability Integration

This file describes how the domain layer should eventually connect to existing
capabilities. It is not wired.

## Rule

Domain output must flow through capabilities. Domain code should not directly
advance phases, set gates, mutate packets, append runtime events, or bypass
verification.

```text
Domain record → Capability record → EvidenceSubmission → API/Runtime → TLog
```

## Mapping Table

| Domain concept       | Future capability target                 | Notes                                               |
| ---                  | ---                                      | ---                                                 |
| `DomainSignal`       | `ObservationRecord`                      | Raw or normalized signal becomes observed evidence. |
| `DomainContext`      | `ContextRecord`, `MemoryLookupRecord`    | Assembled context and retrieved memory.             |
| `DomainJudgment`     | `JudgmentRecord`, `PolicyJudgmentRecord` | Domain verdict becomes bounded judgment.            |
| `DomainPlan`         | `PlanRecord`                             | Future action path becomes plan evidence.           |
| Tool/research action | `ToolExecutionRecord`                    | Effects need receipts.                              |
| Claim/proof check    | `VerificationRecord`                     | Claims and artifacts need verification.             |
| Result scorecard     | `EvalRecord`                             | Result quality becomes eval evidence.               |
| Repeatable pattern   | `PolicyPromotion`                        | Only after replay-verified eval success.            |

## Future Flow

```text
DomainSignal
  ↓
ObservationRecord
  ↓ EvidenceSubmission(Invariant or Analysis path)
ContextRecord + MemoryLookupRecord
  ↓
JudgmentRecord / PolicyJudgmentRecord
  ↓
PlanRecord
  ↓
ToolExecutionRecord / LlmRecord
  ↓
VerificationRecord
  ↓
EvalRecord
  ↓
PolicyPromotion candidate
```

## Integration TODO

- [ ] Decide whether domain records get their own receipt family.
- [ ] Decide whether domain records are serialized as NDJSON.
- [ ] Define domain schema versions.
- [ ] Define domain hash functions using existing `mix` style.
- [ ] Define domain-to-capability conversion boundaries.
- [ ] Add tests that prove domain records cannot bypass capability registry.
- [ ] Add fixtures for finance/business/trading paths.
- [ ] Keep trading execution blocked unless sandbox policy is explicit.

## Non-Goals For Now

- No `pub mod domain` in `lib.rs`.
- No changes to `kernel` enums.
- No changes to `runtime::reduce`.
- No new live trading execution path.
- No domain-specific exception path around verification.
