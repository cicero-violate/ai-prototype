# Critical Project Rating

## Variables

```text
A = architecture clarity
C = capability coverage
E = evidence/replay design
P = privacy/safety
T = testability
O = operational realism
R = requirements fit
D = data discovery quality
L = policy learning quality
F = feedback loop correctness
```

Score equation:

```text
score = geometric_mean(A, C, E, P, T, O, R, D, L, F)
```

One-line explanation: one weak axis should lower the whole project because browser automation fails at the weakest boundary.

## Original README Rating

| Axis                    | Score | Critical Note                                                                                                                  |
|-------------------------+-------+--------------------------------------------------------------------------------------------------------------------------------|
| Architecture clarity    |   8.0 | Strong evidence/replay framing. Too abstract for real provider execution.                                                      |
| Capability coverage     |   4.5 | Does not directly model ChatGPT private, ChatGPT group chat, ChatGPT Projects upload, or Gemini as first-class adapters.       |
| Evidence/replay design  |   8.5 | Best part of the project. Ordered evidence and deterministic replay are correct.                                               |
| Privacy/safety          |   6.5 | Good redaction intent, but not enough hard boundary around credentials, group participants, uploaded files, and auth material. |
| Testability             |   6.8 | Good test categories, but missing provider contract tests, upload tests, group author tests, and live-smoke isolation.         |
| Operational realism     |   5.2 | Underestimates UI drift, login state, project navigation, file chooser behavior, upload completion, and group-chat provenance. |
| Requirements fit        |   4.8 | The actual requirement is multi-provider browser capability routing; the README mostly describes extraction.                   |
| Data discovery quality  |   2.8 | Does not build datasets, features, capability scores, provider comparisons, or policy deltas.                                  |
| Policy learning quality |   1.5 | No closed-loop policy update. Replay proves past behavior but does not improve future behavior.                                |
| Feedback correctness    |   1.5 | No measured action feedback, regression detection, or policy rollback.                                                         |

```text
original_score ≈ 5.4 / 10
```

## Updated Architecture Rating

| Axis                    | Score | Reason                                                                                                  |
|-------------------------+-------+---------------------------------------------------------------------------------------------------------|
| Architecture clarity    |   8.6 | Provider/capability layers are now explicit.                                                            |
| Capability coverage     |   8.1 | ChatGPT private, group, project upload, and Gemini are first-class surfaces.                            |
| Evidence/replay design  |   8.7 | Original strength preserved.                                                                            |
| Privacy/safety          |   8.0 | Adds auth, CAPTCHA, participant data, and upload retention boundaries.                                  |
| Testability             |   8.3 | Adds adapter, upload, group provenance, and live-smoke test classes.                                    |
| Operational realism     |   7.8 | Better but still constrained by provider UI drift and unofficial browser surfaces.                      |
| Requirements fit        |   8.5 | Now matches the stated product requirement directly.                                                    |
| Data discovery quality  |   8.5 | Adds dataset registry, feature extraction, pattern mining, capability scoring, and provider comparison. |
| Policy learning quality |   8.1 | Adds policy hypotheses, policy update gates, and policy versioning.                                     |
| Feedback correctness    |   7.9 | Adds measured deltas, regression detection, and quarantine/revert requirements.                         |

```text
updated_target_score ≈ 8.3 / 10
```

## Brutal Assessment

The original document was architecturally smart but product-misaligned.

It over-optimized for:

```text
schema_extraction + replay
```

It under-specified:

```text
provider_navigation + uploads + group_chat_identity + capability contracts + data_discovery + policy_learning
```

The fatal missing invariant was:

```text
emit(result) ⇒ provider_capability_succeeded ∧ action_receipts_exist ∧ response_provenance_known
```

Without that, the system could extract text correctly while still failing the user’s actual task.

The second missing invariant was:

```text
data_driven ⇔ D → M → π → A → F → update(π)
```

Without that, the system is evidence-driven, not data-driven.

## Highest-Risk Work Remaining

1. ChatGPT Projects artifact upload verification.
2. Group chat author/provenance classification.
3. Gemini UI drift and upload surface differences.
4. Redaction of browser/session artifacts.
5. Dataset registry and feature extraction.
6. Capability scoring and provider comparison.
7. Policy update from measured feedback.
8. Live browser tests that are opt-in and do not leak credentials.

## Correct Next Build Order

```text
provider_registry
  → capability_contracts
  → chatgpt_private_send_read
  → action_receipts
  → project_upload
  → group_author_classifier
  → gemini_adapter
  → dataset_registry
  → feature_extraction
  → pattern_mining
  → capability_scoring
  → policy_feedback_loop
  → replay_promotion
```

This order minimizes risk because every later feature depends on stable provider and capability contracts.
