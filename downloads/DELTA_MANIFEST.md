base_commit: c8936ccbe7a4d8a3ca7d251de57fda9afd28b15a
head_commit: 96e3f33c89e29a18dedb1830ef4024f8931c28d4

# Delta Manifest

## Bundle

- path: /mnt/data/repo-delta-002.bundle
- sha256: 407a8dfa4ddd4c8cef177891ce8b9dc282c0388367d21e026c6893e31202b148
- bundle_verify: pass
- bundle_head: 96e3f33c89e29a18dedb1830ef4024f8931c28d4 HEAD
- bundle_required_ref: c8936ccbe7a4d8a3ca7d251de57fda9afd28b15a

## Changed Files in B..H

- plan.md
- score.md
- scripts/observe_validation.sh
- tests/test_observe_validation_contract.py

## Validation Results

- python3 /mnt/data/bootstrap_rustc_session.py: pass; rustc 1.75.0, cargo 1.75.0, offline probe pass, internal registry dependency probe pass.
- cargo test --all-targets: pass; 103 Rust tests passed; binary and examples compiled.
- python3 -m unittest discover -s tests -p test_*.py -v: pass; 23 Python tests passed.
- python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json: pass; 4 check groups passed, missing_count=0.
- python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json: pass; production_total=0, test_total=319, example_total=1.
- bash -n scripts/observe_validation.sh: pass.
- python3 -m py_compile scripts/write_delta_manifest.py scripts/validate_policy_learning_trace.py scripts/validate_rust_panic_surface.py: pass.
- git diff --check: pass.
- observe_validation full run: partial; emitted source evidence, runtime archive metrics, and delta metrics, then timed out before final summary in this environment.

## Runtime Archive Inspection

- archive: /mnt/data/ai-runtime.tar.gz
- member_count: 108
- log_related_files: 25
- download_related_files: 50
- download_index_or_candidate_ledger_files: 20
- ledger_files: 20
- audit_files: 1
- current_run_summary_present: true
- runtime_manifest_present: true
- runtime_manifest_base_matches_delta_base: true
- runtime_performance_budget_status: pass

## Receiver Apply Commands

```bash
git fetch ./repo-delta-002.bundle HEAD && git merge --ff-only FETCH_HEAD
```
