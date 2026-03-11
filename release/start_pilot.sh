#!/usr/bin/env bash
# PostCAD pilot release — start the service locally.
#
# Usage (from repo root or release/ directory):
#   ./release/start_pilot.sh
#
# Starts postcad-service in the foreground on localhost:8080.
# Runtime data is written under ./data/ (override with POSTCAD_DATA=…).
# Stop with Ctrl-C.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SERVICE_BIN="$REPO_ROOT/target/debug/postcad-service"
DATA_DIR="${POSTCAD_DATA:-$REPO_ROOT/data}"
BASE_URL="http://${POSTCAD_ADDR:-localhost:8080}"

echo ""
echo "══════════════════════════════════════════"
echo "  PostCAD Pilot — Local Service Startup"
echo "══════════════════════════════════════════"
echo "  Repo root : $REPO_ROOT"
echo "  Base URL  : $BASE_URL"
echo "  Data root : $DATA_DIR"
echo "  Data dirs : cases/ receipts/ policies/ dispatch/ verification/"
echo "══════════════════════════════════════════"
echo ""

# ── build if binary is absent ─────────────────────────────────────────────────

if [[ ! -x "$SERVICE_BIN" ]]; then
  echo "[start_pilot] Binary not found at: $SERVICE_BIN"
  echo "[start_pilot] Building postcad-service (this may take a minute)..."
  cargo build --bin postcad-service --manifest-path "$REPO_ROOT/Cargo.toml"
  echo "[start_pilot] Build complete."
  echo ""
fi

# ── start ─────────────────────────────────────────────────────────────────────

echo "[start_pilot] Service starting — press Ctrl-C to stop."
echo "[start_pilot] When the service prints its listen address, open a second"
echo "[start_pilot] terminal and run:  ./release/smoke_test.sh"
echo ""

exec "$SERVICE_BIN"
