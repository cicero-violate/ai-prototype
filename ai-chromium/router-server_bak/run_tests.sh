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

printf 'tests %d\n' "${#files[@]}"