#!/usr/bin/env bash
set -euo pipefail

# Usage:
#   LOCAL_CHECK_ONLINE=1 LOCAL_CHECK_STRICT=1 ci/local_check.sh
# Defaults: offline, non-strict.

ONLINE=${LOCAL_CHECK_ONLINE:-0}
STRICT=${LOCAL_CHECK_STRICT:-0}
VERBOSE=${LOCAL_CHECK_VERBOSE:-0}

if [[ "$VERBOSE" == "1" ]]; then
  set -x
fi

need() {
  command -v "$1" >/dev/null 2>&1 || return 1
}

has_toolchain() {
  rustup toolchain list | grep -q "$1" >/dev/null 2>&1
}

step() {
  echo ""
  echo "▶ $*"
}

run_required() {
  local desc="$1"
  shift
  step "$desc"
  "$@"
}

run_tool_step() {
  local tool="$1"
  shift
  local desc="$1"
  shift
  if need "$tool"; then
    run_required "$desc" "$@"
  else
    echo "[skip] $desc (missing $tool)"
    if [[ "$STRICT" == "1" ]]; then
      echo "Missing required tool: $tool" >&2
      exit 1
    fi
  fi
}

# Tool versions
run_tool_step rustc "rustc version" rustc --version
run_tool_step cargo "cargo version" cargo --version
run_tool_step rustup "rustup version" rustup --version

# Formatting
run_tool_step cargo "cargo fmt" cargo fmt --all -- --check

# Clippy (mirrors GH Actions flags)
run_tool_step cargo "cargo clippy" cargo clippy --workspace --all-targets --all-features -- -D warnings

# Build (all features)
run_tool_step cargo "cargo build" cargo build --workspace --all-features

# Tests (all features)
run_tool_step cargo "cargo test" cargo test --workspace --all-features -- --nocapture

# Export schemas (pages build job)
run_tool_step cargo "cargo run --bin export-schemas --all-features" \
  cargo run --bin export-schemas --all-features

# deny_dupes script
if [[ -x scripts/deny_dupes.sh ]]; then
  run_required "scripts/deny_dupes.sh" ./scripts/deny_dupes.sh
else
  echo "[skip] scripts/deny_dupes.sh (not executable)"
fi

# Schema ID verification (online)
if [[ "$ONLINE" == "1" ]]; then
  if [[ -x scripts/check_schema_ids.sh ]]; then
    run_tool_step curl "Verify published schema IDs" ./scripts/check_schema_ids.sh
  else
    echo "[skip] schema ID verification (script missing)"
  fi
else
  echo "[skip] schema ID verification (set LOCAL_CHECK_ONLINE=1 to enable)"
fi

# MSRV checks via rustup toolchain 1.85.0
if need rustup; then
  if has_toolchain "1.85.0"; then
    run_required "cargo +1.85.0 check" rustup run 1.85.0 cargo check --workspace
    run_required "cargo +1.85.0 check --features schema" \
      rustup run 1.85.0 cargo check --workspace --features schema
  else
    echo "[skip] MSRV cargo check (install rustup toolchain 1.85.0)"
    if [[ "$STRICT" == "1" ]]; then
      echo "Missing rustup toolchain 1.85.0" >&2
      exit 1
    fi
  fi
else
  echo "[skip] MSRV cargo check (rustup not installed)"
  if [[ "$STRICT" == "1" ]]; then
    echo "Missing rustup for MSRV check" >&2
    exit 1
  fi
fi

echo "Local checks completed successfully."
