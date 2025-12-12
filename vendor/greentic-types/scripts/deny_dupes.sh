#!/usr/bin/env bash
set -euo pipefail

# Patterns to guard against: if another crate/files re-define these shared structs/enums,
# fail with a helpful diagnostic. Comma-separated allow lists point to the canonical files.
declare -a PATTERNS=(
  "struct RunResult::src/run.rs"
  "struct NodeSummary::src/run.rs"
  "struct NodeFailure::src/run.rs"
  "enum RunStatus::src/run.rs"
  "enum NodeStatus::src/run.rs"
  "struct Capabilities::src/capabilities.rs"
  "struct Limits::src/capabilities.rs"
  "struct TelemetrySpec::src/capabilities.rs"
)

STATUS=0

for entry in "${PATTERNS[@]}"; do
  pattern=${entry%%::*}
  allow=${entry#*::}
  IFS=',' read -r -a allowed_files <<<"$allow"

  matches=$(rg --files-with-matches -g'*.rs' -F "$pattern" --glob '!target/**' --glob '!dist/**' --glob '!greentic-types-macros/**' || true)
  for file in $matches; do
    skip=false
    for allowed_file in "${allowed_files[@]}"; do
      if [[ "$file" == "$allowed_file" ]]; then
        skip=true
        break
      fi
    done

    if [[ "$skip" == false ]]; then
      echo "Found duplicate definition ('$pattern') in $file. Use greentic-types instead of redefining shared structs." >&2
      STATUS=1
    fi
  done
 done

if [[ $STATUS -ne 0 ]]; then
  echo "Duplicate struct definitions detected. Greentic repos should depend on greentic-types instead." >&2
fi

exit $STATUS
