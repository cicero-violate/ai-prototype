#!/usr/bin/env bash
set -euo pipefail

# Cargo rustc-wrapper launcher for canon-rustc-v3.
#
# Cargo invokes rustc-wrapper as:
#   <wrapper> <real-rustc> <rustc-args...>
#
# Behavior:
#   - use target/debug/canon-rustc-v3 when it already exists
#   - otherwise use scripts/canon-rustc-v3 as the stable wrapper copy
#   - after a build creates a usable wrapper binary, replace the stable
#     scripts/canon-rustc-v3 copy

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REAL_RUSTC="${1:?missing rustc path}"
shift

SCRIPT_WRAPPER="${CANON_RUSTC_V3_WRAPPER:-$ROOT/scripts/canon-rustc-v3}"
TARGET_WRAPPER="$ROOT/target/debug/canon-rustc-v3"
export CANON_RUSTC_V3_ARTIFACT_DIR="${CANON_RUSTC_V3_ARTIFACT_DIR:-state/rustc}"

prepend_ld_path() {
  local dir="$1"
  [ -d "$dir" ] || return 0
  case ":${LD_LIBRARY_PATH:-}:" in
    *":$dir:"*) ;;
    *) export LD_LIBRARY_PATH="$dir${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" ;;
  esac
}

# rustc_private wrappers link to librustc_driver from the active toolchain.
# Cargo does not set the dynamic linker path for rustc-wrapper processes.
if RUSTC_SYSROOT="$("$REAL_RUSTC" --print sysroot 2>/dev/null)"; then
  prepend_ld_path "$RUSTC_SYSROOT/lib"
fi

usable() {
  local candidate="$1"
  [ -x "$candidate" ] || return 1
  "$candidate" "$REAL_RUSTC" -vV >/dev/null 2>&1
}

install_primary_wrapper() {
  local source="$1"
  local tmp="$SCRIPT_WRAPPER.tmp.$$"
  cp "$source" "$tmp"
  chmod 755 "$tmp"
  mv -f "$tmp" "$SCRIPT_WRAPPER"
}

rustc_output_path() {
  local previous=""
  local arg
  for arg in "$@"; do
    if [ "$previous" = "-o" ]; then
      printf '%s\n' "$arg"
      return 0
    fi
    previous="$arg"
  done

  local crate_name
  local out_dir
  local extra_filename
  crate_name="$(arg_value --crate-name "$@" || true)"
  out_dir="$(arg_value --out-dir "$@" || true)"
  extra_filename="$(codegen_value extra-filename "$@" || true)"
  if [ -n "$crate_name" ] && [ -n "$out_dir" ] && [ -n "$extra_filename" ]; then
    printf '%s\n' "$out_dir/$crate_name$extra_filename"
    return 0
  fi

  return 1
}

arg_value() {
  local flag="$1"
  shift
  local previous=""
  local arg
  for arg in "$@"; do
    if [ "$previous" = "$flag" ]; then
      printf '%s\n' "$arg"
      return 0
    fi
    case "$arg" in
      "$flag="*)
        printf '%s\n' "${arg#"$flag="}"
        return 0
        ;;
    esac
    previous="$arg"
  done
  return 1
}

has_arg_value() {
  local flag="$1"
  local expected="$2"
  shift 2
  local previous=""
  local arg
  for arg in "$@"; do
    if [ "$previous" = "$flag" ] && [ "$arg" = "$expected" ]; then
      return 0
    fi
    [ "$arg" = "$flag=$expected" ] && return 0
    previous="$arg"
  done
  return 1
}

codegen_value() {
  local key="$1"
  shift
  local previous=""
  local arg
  for arg in "$@"; do
    if [ "$previous" = "-C" ]; then
      case "$arg" in
        "$key="*)
          printf '%s\n' "${arg#"$key="}"
          return 0
          ;;
      esac
    fi
    previous="$arg"
  done
  return 1
}

is_wrapper_binary_build() {
  [ "$(arg_value --crate-name "$@" || true)" = "canon_rustc_v3" ] \
    && has_arg_value --crate-type bin "$@"
}

refresh_script_wrapper() {
  local rustc_output=""
  if usable "$TARGET_WRAPPER"; then
    install_primary_wrapper "$TARGET_WRAPPER"
    return 0
  fi

  if is_wrapper_binary_build "$@"; then
    local crate_name
    local out_dir
    local extra_filename
    crate_name="$(arg_value --crate-name "$@" || true)"
    out_dir="$(arg_value --out-dir "$@" || true)"
    extra_filename="$(codegen_value extra-filename "$@" || true)"
    rustc_output="$out_dir/$crate_name$extra_filename"
    if [ -x "$rustc_output" ]; then
      install_primary_wrapper "$rustc_output"
      return 0
    fi
  fi

  return 0
}

schedule_target_refresh() {
  is_wrapper_binary_build "$@" || return 0
  nohup bash -c '
    target="$1"
    script="$2"
    i=0
    while [ "$i" -lt 120 ]; do
      if [ -x "$target" ]; then
        tmp="$script.tmp.refresh.$$"
        cp "$target" "$tmp" && chmod 755 "$tmp" && mv -f "$tmp" "$script"
        exit 0
      fi
      i=$((i + 1))
      sleep 0.5
    done
  ' canon-rustc-v3-refresh "$TARGET_WRAPPER" "$SCRIPT_WRAPPER" >/dev/null 2>&1 &
}

run_wrapper() {
  local wrapper="$1"
  shift
  set +e
  "$wrapper" "$REAL_RUSTC" "$@"
  local status=$?
  set -e
  refresh_script_wrapper "$@"
  schedule_target_refresh "$@"
  exit "$status"
}

if usable "$TARGET_WRAPPER"; then
  install_primary_wrapper "$TARGET_WRAPPER"
  run_wrapper "$SCRIPT_WRAPPER" "$@"
fi

if usable "$SCRIPT_WRAPPER"; then
  run_wrapper "$SCRIPT_WRAPPER" "$@"
fi

echo "canon-rustc-v3 auto-wrapper: no usable wrapper found; falling back to rustc" >&2
exec "$REAL_RUSTC" "$@"
