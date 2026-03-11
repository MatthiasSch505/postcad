#!/usr/bin/env bash
# PostCAD pilot release — reset local runtime data.
#
# Usage (from repo root or release/ directory):
#   ./release/reset_pilot_data.sh
#
# Removes only the five runtime data directories written by postcad-service.
# Stop the service before running this script.
#
# Touches:          data/cases/  data/receipts/  data/policies/
#                   data/dispatch/  data/verification/
# Does NOT touch:   source code, compiled binaries, canonical fixtures

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DATA_DIR="${POSTCAD_DATA:-$REPO_ROOT/data}"

RUNTIME_DIRS=(
  "$DATA_DIR/cases"
  "$DATA_DIR/receipts"
  "$DATA_DIR/policies"
  "$DATA_DIR/dispatch"
  "$DATA_DIR/verification"
)

echo ""
echo "[reset_pilot_data] Repo root : $REPO_ROOT"
echo "[reset_pilot_data] Data root : $DATA_DIR"
echo ""

removed=0
skipped=0
for dir in "${RUNTIME_DIRS[@]}"; do
  if [[ -d "$dir" ]]; then
    file_count=$(find "$dir" -maxdepth 1 -type f | wc -l | tr -d ' ')
    rm -rf "$dir"
    echo "[reset_pilot_data] REMOVED  $dir  ($file_count file(s))"
    removed=$((removed + 1))
  else
    echo "[reset_pilot_data] skipped  $dir  (not present)"
    skipped=$((skipped + 1))
  fi
done

echo ""
echo "[reset_pilot_data] ── NOT touched ──────────────────────────────────────"
echo "[reset_pilot_data]   source code, compiled artifacts, canonical fixtures"
echo ""

if [[ "$removed" -gt 0 ]]; then
  echo "[reset_pilot_data] Done. $removed director(ies) removed, $skipped skipped."
else
  echo "[reset_pilot_data] Nothing to remove. All runtime directories were already absent."
fi
