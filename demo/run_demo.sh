#!/usr/bin/env bash
# PostCAD deterministic external demo — 8-step full flow
#
# Usage:
#   ./demo/run_demo.sh
#
# Requires: cargo build --bin postcad-service has been run (or is done here).
# The script starts the service, runs all 8 steps, then stops the service.
# Every curl call asserts the expected HTTP status code.

set -euo pipefail

BASE_URL="http://localhost:8080"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
SERVICE_BIN="$ROOT_DIR/target/debug/postcad-service"
REGISTRY="$ROOT_DIR/examples/pilot/registry_snapshot.json"
CONFIG="$ROOT_DIR/examples/pilot/config.json"
CASE_FILE="$SCRIPT_DIR/case_demo.json"

SERVICE_PID=""

cleanup() {
  if [[ -n "$SERVICE_PID" ]]; then
    echo ""
    echo "--- Stopping service (PID $SERVICE_PID) ---"
    kill "$SERVICE_PID" 2>/dev/null || true
    wait "$SERVICE_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

# ── helpers ──────────────────────────────────────────────────────────────────

assert_status() {
  local label="$1"
  local expected="$2"
  local actual="$3"
  if [[ "$actual" != "$expected" ]]; then
    echo "FAIL [$label]: expected HTTP $expected, got HTTP $actual" >&2
    exit 1
  fi
}

hr() { echo ""; echo "════════════════════════════════════════"; echo "  $*"; echo "════════════════════════════════════════"; }

# ── build if needed ───────────────────────────────────────────────────────────

if [[ ! -x "$SERVICE_BIN" ]]; then
  hr "Building postcad-service..."
  cargo build --bin postcad-service --manifest-path "$ROOT_DIR/Cargo.toml" 2>&1
fi

# ── STEP 1 — Start the service ────────────────────────────────────────────────

hr "STEP 1 — Starting postcad-service"
DATA_DIR=$(mktemp -d)
echo "  Data directory: $DATA_DIR"
POSTCAD_DATA="$DATA_DIR" "$SERVICE_BIN" &
SERVICE_PID=$!
echo "  Service PID: $SERVICE_PID"

# ── STEP 2 — Wait until /health is ready ──────────────────────────────────────

hr "STEP 2 — Waiting for /health"
for i in $(seq 1 20); do
  STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/health" 2>/dev/null || echo "000")
  if [[ "$STATUS" == "200" ]]; then
    echo "  Service is up (attempt $i)"
    break
  fi
  if [[ "$i" == "20" ]]; then
    echo "FAIL: service did not start within 10 seconds" >&2
    exit 1
  fi
  sleep 0.5
done
curl -s "$BASE_URL/health" | python3 -m json.tool

# ── STEP 3 — Store the demo case ──────────────────────────────────────────────

hr "STEP 3 — Store demo case (POST /cases)"
INTAKE_RESP=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/cases" \
  -H "Content-Type: application/json" \
  -d "@$CASE_FILE")
INTAKE_BODY=$(echo "$INTAKE_RESP" | head -n -1)
INTAKE_STATUS=$(echo "$INTAKE_RESP" | tail -n 1)
assert_status "case intake" "200" "$INTAKE_STATUS"
echo "$INTAKE_BODY" | python3 -m json.tool
CASE_ID=$(echo "$INTAKE_BODY" | python3 -c "import json,sys; print(json.load(sys.stdin)['case_id'])")
echo "  Stored case_id: $CASE_ID"

# ── STEP 4 — Route the stored case ────────────────────────────────────────────

hr "STEP 4 — Route stored case (POST /cases/$CASE_ID/route)"
ROUTE_BODY=$(python3 -c "
import json, sys
registry = json.load(open('$REGISTRY'))
config   = json.load(open('$CONFIG'))
print(json.dumps({'registry_snapshot': registry, 'routing_config': config}))
")
ROUTE_RESP=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/cases/$CASE_ID/route" \
  -H "Content-Type: application/json" \
  -d "$ROUTE_BODY")
ROUTE_BODY_TEXT=$(echo "$ROUTE_RESP" | head -n -1)
ROUTE_STATUS=$(echo "$ROUTE_RESP" | tail -n 1)
assert_status "route stored case" "200" "$ROUTE_STATUS"
echo "$ROUTE_BODY_TEXT" | python3 -c "
import json, sys
d = json.load(sys.stdin)
r = d.get('receipt', d)
print(json.dumps({'outcome': r.get('outcome'), 'selected_candidate_id': r.get('selected_candidate_id'), 'receipt_hash': r.get('receipt_hash')}, indent=2))
"
RECEIPT_HASH=$(echo "$ROUTE_BODY_TEXT" | python3 -c "
import json, sys
d = json.load(sys.stdin)
r = d.get('receipt', d)
print(r['receipt_hash'])
")
echo "  Receipt hash: $RECEIPT_HASH"

# ── STEP 5 — Retrieve the routing receipt ─────────────────────────────────────

hr "STEP 5 — Retrieve receipt (GET /receipts/$RECEIPT_HASH)"
RECEIPT_RESP=$(curl -s -w "\n%{http_code}" "$BASE_URL/receipts/$RECEIPT_HASH")
RECEIPT_BODY=$(echo "$RECEIPT_RESP" | head -n -1)
RECEIPT_STATUS=$(echo "$RECEIPT_RESP" | tail -n 1)
assert_status "get receipt" "200" "$RECEIPT_STATUS"
echo "$RECEIPT_BODY" | python3 -c "
import json, sys
d = json.load(sys.stdin)
print(json.dumps({'outcome': d.get('outcome'), 'selected_candidate_id': d.get('selected_candidate_id'), 'receipt_hash': d.get('receipt_hash')}, indent=2))
"

# ── STEP 6 — Dispatch the receipt ─────────────────────────────────────────────

hr "STEP 6 — Dispatch receipt (POST /dispatch/$RECEIPT_HASH)"
DISPATCH_RESP=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/dispatch/$RECEIPT_HASH")
DISPATCH_BODY=$(echo "$DISPATCH_RESP" | head -n -1)
DISPATCH_STATUS=$(echo "$DISPATCH_RESP" | tail -n 1)
assert_status "dispatch" "200" "$DISPATCH_STATUS"
echo "$DISPATCH_BODY" | python3 -m json.tool

# ── STEP 7 — Verify the dispatched receipt ────────────────────────────────────

hr "STEP 7 — Verify dispatch (POST /dispatch/$RECEIPT_HASH/verify)"
VERIFY_RESP=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/dispatch/$RECEIPT_HASH/verify")
VERIFY_BODY=$(echo "$VERIFY_RESP" | head -n -1)
VERIFY_STATUS=$(echo "$VERIFY_RESP" | tail -n 1)
assert_status "dispatch verify" "200" "$VERIFY_STATUS"
echo "$VERIFY_BODY" | python3 -m json.tool
VERIFY_RESULT=$(echo "$VERIFY_BODY" | python3 -c "import json,sys; print(json.load(sys.stdin)['result'])")
if [[ "$VERIFY_RESULT" != "VERIFIED" ]]; then
  echo "FAIL: expected result=VERIFIED, got $VERIFY_RESULT" >&2
  exit 1
fi
echo "  Verification: $VERIFY_RESULT"

# ── STEP 8 — Show route history ───────────────────────────────────────────────

hr "STEP 8 — Route history (GET /routes)"
HISTORY_RESP=$(curl -s -w "\n%{http_code}" "$BASE_URL/routes")
HISTORY_BODY=$(echo "$HISTORY_RESP" | head -n -1)
HISTORY_STATUS=$(echo "$HISTORY_RESP" | tail -n 1)
assert_status "route history" "200" "$HISTORY_STATUS"
echo "$HISTORY_BODY" | python3 -m json.tool

hr "DEMO COMPLETE — all 8 steps passed"
