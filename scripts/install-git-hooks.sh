#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GIT_DIR="$(git -C "$ROOT" rev-parse --git-dir)"
case "$GIT_DIR" in
  /*) HOOK_DIR="$GIT_DIR/hooks" ;;
  *) HOOK_DIR="$ROOT/$GIT_DIR/hooks" ;;
esac
mkdir -p "$HOOK_DIR"

cat > "$HOOK_DIR/pre-commit" <<'HOOK'
#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"
exec scripts/validate-fast.sh
HOOK

cat > "$HOOK_DIR/pre-push" <<'HOOK'
#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"
exec scripts/validate-full.sh
HOOK

chmod +x "$HOOK_DIR/pre-commit" "$HOOK_DIR/pre-push"
echo "installed hooks: $HOOK_DIR/pre-commit $HOOK_DIR/pre-push"
