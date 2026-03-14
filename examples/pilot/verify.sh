#!/usr/bin/env bash
# PostCAD Protocol v1 — Pilot Receipt Verification
#
# Verifies the routing receipt produced by run_pilot.sh against the original
# pilot inputs. Exits non-zero if verification fails.
#
# Usage:
#   ./verify.sh              # human-readable output
#   ./verify.sh --json       # JSON output

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BIN="${REPO_ROOT}/target/debug/postcad-cli"

if [[ ! -x "$BIN" ]]; then
  echo "Building postcad-cli..."
  cargo build --bin postcad-cli --quiet --manifest-path "${REPO_ROOT}/Cargo.toml"
fi

RECEIPT="${SCRIPT_DIR}/receipt.json"

if [[ ! -f "$RECEIPT" ]]; then
  echo "error: receipt.json not found — run run_pilot.sh first" >&2
  exit 1
fi

JSON_FLAG=""
if [[ "${1:-}" == "--json" ]]; then
  JSON_FLAG="--json"
fi

echo "PostCAD Protocol v1 — Pilot Receipt Verification"
echo "=================================================="
echo ""

"$BIN" verify-receipt $JSON_FLAG \
  --receipt    "${SCRIPT_DIR}/receipt.json" \
  --case       "${SCRIPT_DIR}/case.json" \
  --policy     "${SCRIPT_DIR}/derived_policy.json" \
  --candidates "${SCRIPT_DIR}/candidates.json"

echo ""
echo "Verification complete — receipt is authentic, dispatch is safe to proceed."
echo ""
echo "Next step: open the reviewer shell to create and approve a dispatch commitment."
echo "  cargo run -p postcad-service"
echo "  # then open http://localhost:8080/reviewer"
