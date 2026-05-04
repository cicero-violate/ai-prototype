base_commit: 54e73b20e1c6a3e377161305c88eb562e6ed903d
head_commit: 55815a3eb635f05809db8708f9194465d60e781d

status: failed_to_restore_required_ai_source_bundle
bundle_path: /mnt/data/repo-delta-002.bundle
bundle_valid_git_bundle: true
bundle_semantics: diagnostic_placeholder_not_cumulative_delta

reason:
- /mnt/data contains ai-runtime.tar.gz but no ai git bundle or ai.bundle source repository bundle.
- The runtime tarball was inspected for logs, conversation snapshots, download indexes, and prior runtime state.
- The runtime tarball does not contain a source worktree, .git directory, or usable source git bundle.
- GitHub fallback repository cicero-violate/ai is accessible through the connector, but required base commit 54e73b20e1c6a3e377161305c88eb562e6ed903d is not present there.
- Creating the requested cumulative command git bundle create /mnt/data/repo-delta-002.bundle B..H is unsafe/impossible without the source repository containing B.

runtime_archive_inspection:
- runtime_archive: /mnt/data/ai-runtime.tar.gz
- parsed_with: Python tarfile and Python json/text readers
- runtime_archive_present: true
- runtime_archive_sha256: bfa52a6dd75b583e5ac49ab6c348b4c54aa587522b463869cf9a269989950001
- runtime_member_count: 93
- runtime_logs_detected: true
- runtime_conversation_ledgers_detected: true
- runtime_download_indexes_detected: true
- runtime_delta_apply_receipts_detected: true
- runtime_manifest_detected: true

validation_results:
- rust_bootstrap: pass
- rustc_version: rustc 1.75.0
- cargo_version: cargo 1.75.0
- dependency_fetch_probe: pass
- source_repo_restore: failed_source_bundle_absent
- repository_validation: not_run_source_repo_absent
- requested_cumulative_bundle_B_to_H: not_created_base_object_absent

changed_files_B_to_H:
- unavailable_source_repo_absent

receiver_apply_commands:
- git fetch ./repo-delta-002.bundle HEAD && git merge --ff-only FETCH_HEAD

receiver_warning:
- Do not apply this artifact as the requested repo delta. It is a diagnostic placeholder because the required uploaded source bundle is absent.

recovery_required:
- Re-upload or restore the ai git bundle containing base commit 54e73b20e1c6a3e377161305c88eb562e6ed903d, then rerun Phase 2.
