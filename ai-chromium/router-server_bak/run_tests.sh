#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$root"

mapfile -t files < <(find src -type f -name '*.mjs' | sort)
if [[ ${#files[@]} -eq 0 ]]; then
  echo "no .mjs files found" >&2
  exit 1
fi

for file in "${files[@]}"; do
  node --check "$file" >/dev/null
done

mapfile -t test_files < <(find test -type f -name '*.test.mjs' 2>/dev/null | sort)
if [[ ${#test_files[@]} -gt 0 ]]; then
  node --test "${test_files[@]}"
fi

behavior_tests=0
if [[ ${#test_files[@]} -gt 0 ]]; then
  behavior_tests="$({ awk '/^test\(/ { count++ } END { print count + 0 }' "${test_files[@]}"; })"
fi

printf 'syntax %d\n' "${#files[@]}"
printf 'behavior %d\n' "$behavior_tests"
printf 'tests %d\n' "$(( ${#files[@]} + behavior_tests ))"