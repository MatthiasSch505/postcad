#!/usr/bin/env bash
# PostCAD pilot release — start the service locally.
#
# Usage:
#   ./release/start_pilot.sh
#
# Starts postcad-service in the foreground on localhost:8080.
# Runtime data is written to ./data/ relative to the working directory.
# Stop with Ctrl-C.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SERVICE_BIN="$ROOT_DIR/target/debug/postcad-service"

# ── build if binary is absent ─────────────────────────────────────────────────

if [[ ! -x "$SERVICE_BIN" ]]; then
  echo "[start_pilot] Building postcad-service..."
  cargo build --bin postcad-service --manifest-path "$ROOT_DIR/Cargo.toml"
fi

# ── announce data paths ───────────────────────────────────────────────────────

DATA_DIR="${POSTCAD_DATA:-$ROOT_DIR/data}"
echo "[start_pilot] Data directory : $DATA_DIR"
echo "[start_pilot]   cases        : $DATA_DIR/cases/"
echo "[start_pilot]   receipts     : $DATA_DIR/receipts/"
echo "[start_pilot]   policies     : $DATA_DIR/policies/"
echo "[start_pilot]   dispatch     : $DATA_DIR/dispatch/"
echo "[start_pilot]   verification : $DATA_DIR/verification/"
echo "[start_pilot] Service address: ${POSTCAD_ADDR:-0.0.0.0:8080}"
echo "[start_pilot] Starting service (Ctrl-C to stop)..."
echo ""

# ── start ─────────────────────────────────────────────────────────────────────

exec "$SERVICE_BIN"
