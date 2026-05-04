base_commit: 52d402c00e2ace712d22a3cabf2ca1176b67e7c0
head_commit: unavailable-no-valid-source-worktree

status: failed_to_create_valid_delta_bundle
bundle_path: /mnt/data/repo-delta-002.bundle
bundle_valid_git_bundle: false

reason:
- The required restored ai source repository is unavailable in the current sandbox state.
- /mnt/data/ai.bundle is absent.
- file_search retrieval for uploaded files failed repeatedly with RetrievalClientResponseError.
- api_tool fallback confirmed the public GitHub repository exists, but the required base commit 52d402c00e2ace712d22a3cabf2ca1176b67e7c0 is not present there.
- /mnt/data/ai-runtime.tar.gz was inspected with Python tarfile and JSON readers; it contains runtime logs, ledgers, manifests, and downloaded reports, but intentionally excludes source apply-worktrees and generated git bundles.
- A synthetic git bundle from partial runtime/GitHub data would be unsafe because the receiver command `git fetch ./repo-delta-002.bundle HEAD && git merge --ff-only FETCH_HEAD` would replace the worktree with an incomplete reconstructed tree.

runtime_archive_inspection:
- runtime_archive: /mnt/data/ai-runtime.tar.gz
- parsed_with: Python tarfile + Python json
- runtime_manifest_base_commit: 52d402c00e2ace712d22a3cabf2ca1176b67e7c0
- runtime_member_count: 64
- runtime_download_history_count: 131
- runtime_included_count: 63
- runtime_excluded_apply_worktree_files_detected: true
- generated_git_bundles_excluded_by_runtime_archive: true

rust_toolchain_status:
- shell_tar_used_for_rust: false
- python_tarfile_extraction_used: true
- installed_prefix: /mnt/data/rustc-python-install-prefix
- cargo_home: /mnt/data/rustc-python-cargo-home
- rustc_available: true
- cargo_available: true
- note: dependency-free cargo probe cannot substitute for repository validation because the source repository is unavailable.

validation_results:
- source_repo_restore: failed_source_bundle_absent
- runtime_archive_parse: pass
- github_fallback_search: partial
- github_required_base_commit_lookup: not_found
- valid_git_bundle_create: failed_no_source_worktree

changed_files_B_to_H:
- none_validated

receiver_apply_commands:
- git fetch ./repo-delta-002.bundle HEAD && git merge --ff-only FETCH_HEAD

receiver_warning:
- Do not apply this artifact as a git bundle. It is a diagnostic placeholder created to avoid fabricating a destructive synthetic delta.

recovery_required:
- Re-upload the ai git bundle containing base commit 52d402c00e2ace712d22a3cabf2ca1176b67e7c0, or restore /mnt/data/ai.bundle in the sandbox, then rerun Phase 2.
