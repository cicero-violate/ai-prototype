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
scripts/validate-fast.sh
scripts/reload-supervisor-after-validation.sh
HOOK

cat > "$HOOK_DIR/pre-push" <<'HOOK'
#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"
scripts/validate-full.sh
scripts/reload-supervisor-after-validation.sh
HOOK

chmod +x "$ROOT/scripts/reload-supervisor-after-validation.sh"
chmod +x "$HOOK_DIR/pre-commit" "$HOOK_DIR/pre-push"
echo "installed hooks: $HOOK_DIR/pre-commit $HOOK_DIR/pre-push"
