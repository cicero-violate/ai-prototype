# Auto-Refactor Plan Pointer

The authoritative auto-refactor implementation plan for this repository now
lives at the repository root:

```text
plan-autorefactor.md
```

This nested file is intentionally non-authoritative. It is retained only to
prevent stale `canon-rustc-v3/plan-autorefactor.md` guidance from competing with
the root plan.

Current authority decision:

```text
authoritative_plan = plan-autorefactor.md
stale_schema_v13_material = superseded
current_schema_target = schema-v16 graph capture
planner_scope = deterministic advisory operation specs only
source_rewrite_authority = none
```

Any future auto-refactor implementation update should change the root plan or
the implementation files directly, not this pointer.