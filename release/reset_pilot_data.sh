#!/usr/bin/env bash
# PostCAD pilot release — reset local runtime data.
#
# Usage:
#   ./release/reset_pilot_data.sh
#
# Removes only the runtime data directories written by postcad-service:
#   data/cases/, data/receipts/, data/policies/,
#   data/dispatch/, data/verification/
#
# Does NOT touch:
#   - source code
#   - canonical fixture directories (frozen inputs and expected outputs)
#   - any compiled artifacts
#
# Safe to run while the service is stopped. Do not run while the service
# is writing to the same data directory.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA_DIR="${POSTCAD_DATA:-$ROOT_DIR/data}"

RUNTIME_DIRS=(
  "$DATA_DIR/cases"
  "$DATA_DIR/receipts"
  "$DATA_DIR/policies"
  "$DATA_DIR/dispatch"
  "$DATA_DIR/verification"
)

echo "[reset_pilot_data] Data root: $DATA_DIR"
echo ""

any_removed=0
for dir in "${RUNTIME_DIRS[@]}"; do
  if [[ -d "$dir" ]]; then
    file_count=$(find "$dir" -maxdepth 1 -type f | wc -l)
    rm -rf "$dir"
    echo "[reset_pilot_data] Removed: $dir  ($file_count file(s))"
    any_removed=1
  else
    echo "[reset_pilot_data] Skipped (not present): $dir"
  fi
done

echo ""
if [[ "$any_removed" == "1" ]]; then
  echo "[reset_pilot_data] Reset complete. Runtime data cleared."
else
  echo "[reset_pilot_data] Nothing to remove. Data directories were already absent."
fi
