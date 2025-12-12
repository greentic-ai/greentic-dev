#!/usr/bin/env bash
set -euo pipefail

SCHEMAS=(
  "https://greentic-ai.github.io/greentic-types/schemas/v1/pack-id.schema.json"
  "https://greentic-ai.github.io/greentic-types/schemas/v1/component-id.schema.json"
  "https://greentic-ai.github.io/greentic-types/schemas/v1/flow-id.schema.json"
  "https://greentic-ai.github.io/greentic-types/schemas/v1/node-id.schema.json"
  "https://greentic-ai.github.io/greentic-types/schemas/v1/tenant-context.schema.json"
  "https://greentic-ai.github.io/greentic-types/schemas/v1/hash-digest.schema.json"
  "https://greentic-ai.github.io/greentic-types/schemas/v1/semver-req.schema.json"
  "https://greentic-ai.github.io/greentic-types/schemas/v1/redaction-path.schema.json"
  "https://greentic-ai.github.io/greentic-types/schemas/v1/capabilities.schema.json"
  "https://greentic-ai.github.io/greentic-types/schemas/v1/limits.schema.json"
  "https://greentic-ai.github.io/greentic-types/schemas/v1/telemetry-spec.schema.json"
  "https://greentic-ai.github.io/greentic-types/schemas/v1/node-summary.schema.json"
  "https://greentic-ai.github.io/greentic-types/schemas/v1/node-failure.schema.json"
  "https://greentic-ai.github.io/greentic-types/schemas/v1/node-status.schema.json"
  "https://greentic-ai.github.io/greentic-types/schemas/v1/run-status.schema.json"
  "https://greentic-ai.github.io/greentic-types/schemas/v1/transcript-offset.schema.json"
  "https://greentic-ai.github.io/greentic-types/schemas/v1/tools-caps.schema.json"
  "https://greentic-ai.github.io/greentic-types/schemas/v1/secrets-caps.schema.json"
  "https://greentic-ai.github.io/greentic-types/schemas/v1/otlp-keys.schema.json"
  "https://greentic-ai.github.io/greentic-types/schemas/v1/run-result.schema.json"
)

for url in "${SCHEMAS[@]}"; do
  echo "Checking schema $url"
  tmp=$(mktemp)
  status=$(curl -sS -w "%{http_code}" -o "$tmp" "$url") || {
    echo "Failed to download $url" >&2
    rm -f "$tmp"
    exit 1
  }
  if [[ "$status" != "200" ]]; then
    echo "Schema $url responded with HTTP $status" >&2
    rm -f "$tmp"
    exit 1
  fi

  schema_id=$(python3 - "$tmp" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
try:
    data = json.loads(path.read_text())
except json.JSONDecodeError as exc:
    print(f"Failed to parse {path}: {exc}", file=sys.stderr)
    sys.exit(1)

print(data.get("$id", ""))
PY
  )
  rm -f "$tmp"

  if [[ "$schema_id" != "$url" ]]; then
    echo "Schema $url has mismatched \$id ('$schema_id')" >&2
    exit 1
  fi
  echo "✔ $url"
done
