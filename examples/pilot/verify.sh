#!/usr/bin/env bash
# PostCAD Protocol v1 — Standalone Verification
#
# Runs the end-to-end demo using embedded frozen protocol-vector v01 fixtures.
# This command routes the case AND verifies the receipt in a single step,
# printing a VERIFIED result.
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

JSON_FLAG=""
if [[ "${1:-}" == "--json" ]]; then
  JSON_FLAG="--json"
fi

echo "PostCAD Protocol v1 — Verification"
echo "====================================="
echo ""
echo "Running demo route + verify..."
echo ""

"$BIN" demo-run $JSON_FLAG

echo ""
echo "Verification complete."
