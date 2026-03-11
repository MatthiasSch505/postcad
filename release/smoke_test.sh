#!/usr/bin/env bash
# PostCAD pilot release — smoke test.
#
# Usage (from repo root or release/ directory):
#   ./release/smoke_test.sh
#
# Assumes postcad-service is already running on localhost:8080.
# Start it first with:  ./release/start_pilot.sh
#
# Runs a deterministic 7-step pilot smoke flow and exits nonzero on failure.
#
# Flow:
#   1. GET  /health                      — service is up
#   2. POST /cases                       — store pilot case
#   3. POST /cases/:id/route             — route the stored case
#   4. GET  /receipts/:hash              — retrieve routing receipt
#   5. POST /dispatch/:hash              — dispatch the receipt
#   6. POST /dispatch/:hash/verify       — verify dispatched receipt
#   7. GET  /routes                      — route history is non-empty
#
# Uses the canonical pilot fixture (examples/pilot/) as input.
# Does not start or stop the service.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BASE_URL="${POSTCAD_ADDR:-http://localhost:8080}"
REGISTRY="$REPO_ROOT/examples/pilot/registry_snapshot.json"
CONFIG="$REPO_ROOT/examples/pilot/config.json"

# Canonical pilot case — deterministic; same case_id every run.
PILOT_CASE='{
  "case_id": "f1000001-0000-0000-0000-000000000001",
  "jurisdiction": "DE",
  "routing_policy": "allow_domestic_and_cross_border",
  "patient_country": "germany",
  "manufacturer_country": "germany",
  "material": "zirconia",
  "procedure": "crown",
  "file_type": "stl"
}'

# ── helpers ───────────────────────────────────────────────────────────────────

pass() { echo "  [PASS] $*"; }
fail() { echo "  [FAIL] $*" >&2; exit 1; }

assert_status() {
  local label="$1" expected="$2" actual="$3"
  [[ "$actual" == "$expected" ]] || fail "Phase $label: expected HTTP $expected, got HTTP $actual"
}

assert_field() {
  local label="$1" field="$2" json="$3"
  python3 -c "
import json, sys
d = json.loads(sys.argv[1])
keys = sys.argv[2].split('.')
v = d
for k in keys:
    v = v[k]
assert v is not None and v != '', f'field {sys.argv[2]} is null/empty'
" "$json" "$field" || fail "Phase $label: field '$field' missing or empty in response"
}

hr() { echo ""; echo "── $* ──────────────────────────────────────────────"; }

# ── PRE-FLIGHT — required tools ───────────────────────────────────────────────

hr "Pre-flight checks"
for tool in curl python3; do
  if ! command -v "$tool" &>/dev/null; then
    fail "Pre-flight: '$tool' not found on PATH — install it before running this script"
  fi
  echo "  [OK] $tool found: $(command -v "$tool")"
done

for fixture in "$REGISTRY" "$CONFIG"; do
  [[ -f "$fixture" ]] || fail "Pre-flight: fixture not found: $fixture"
  echo "  [OK] fixture: $fixture"
done

echo "  [OK] Base URL: $BASE_URL"

# ── PRE-FLIGHT — service reachable ────────────────────────────────────────────

echo ""
echo "  Checking service reachability..."
REACH_STATUS=$(curl -s -o /dev/null -w "%{http_code}" --max-time 3 "$BASE_URL/health" 2>/dev/null || echo "000")
if [[ "$REACH_STATUS" == "000" ]]; then
  echo "" >&2
  echo "  [FAIL] Cannot reach $BASE_URL/health (connection refused or timeout)" >&2
  echo "" >&2
  echo "  Is the service running? Start it with:" >&2
  echo "    ./release/start_pilot.sh" >&2
  echo "" >&2
  exit 1
fi
echo "  [OK] Service reachable (HTTP $REACH_STATUS)"

# ── STEP 1 — health ───────────────────────────────────────────────────────────

hr "1. Health"
resp=$(curl -s -w "\n%{http_code}" "$BASE_URL/health")
body=$(echo "$resp" | head -n -1)
status=$(echo "$resp" | tail -n 1)
assert_status "health" "200" "$status"
echo "$body" | python3 -m json.tool
pass "health"

# ── STEP 2 — store pilot case ─────────────────────────────────────────────────

hr "2. Store pilot case (POST /cases)"
resp=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/cases" \
  -H "Content-Type: application/json" -d "$PILOT_CASE")
body=$(echo "$resp" | head -n -1)
status=$(echo "$resp" | tail -n 1)
# 200 on first store, also accept 200 if idempotent (case already exists → still 200)
assert_status "case intake" "200" "$status"
echo "$body" | python3 -m json.tool
assert_field "case intake" "case_id" "$body"
CASE_ID=$(python3 -c "import json,sys; print(json.loads(sys.argv[1])['case_id'])" "$body")
pass "case stored (case_id=$CASE_ID)"

# ── STEP 3 — route stored case ────────────────────────────────────────────────

hr "3. Route stored case (POST /cases/$CASE_ID/route)"
ROUTE_PAYLOAD=$(python3 -c "
import json
registry = json.load(open('$REGISTRY'))
config   = json.load(open('$CONFIG'))
print(json.dumps({'registry_snapshot': registry, 'routing_config': config}))
")
resp=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/cases/$CASE_ID/route" \
  -H "Content-Type: application/json" -d "$ROUTE_PAYLOAD")
body=$(echo "$resp" | head -n -1)
status=$(echo "$resp" | tail -n 1)
assert_status "route stored case" "200" "$status"
python3 -c "
import json,sys
d = json.loads(sys.argv[1])
r = d.get('receipt', d)
print(json.dumps({'outcome': r.get('outcome'), 'selected_candidate_id': r.get('selected_candidate_id'), 'receipt_hash': r.get('receipt_hash')}, indent=2))
" "$body"
assert_field "route stored case" "receipt.receipt_hash" "$body"
RECEIPT_HASH=$(python3 -c "
import json,sys
d = json.loads(sys.argv[1])
print(d['receipt']['receipt_hash'])
" "$body")
OUTCOME=$(python3 -c "
import json,sys
d = json.loads(sys.argv[1])
print(d['receipt']['outcome'])
" "$body")
[[ "$OUTCOME" == "routed" ]] || fail "route stored case: expected outcome=routed, got $OUTCOME"
pass "case routed (receipt_hash=${RECEIPT_HASH:0:16}…)"

# ── STEP 4 — retrieve receipt ─────────────────────────────────────────────────

hr "4. Retrieve receipt (GET /receipts/$RECEIPT_HASH)"
resp=$(curl -s -w "\n%{http_code}" "$BASE_URL/receipts/$RECEIPT_HASH")
body=$(echo "$resp" | head -n -1)
status=$(echo "$resp" | tail -n 1)
assert_status "get receipt" "200" "$status"
python3 -c "
import json,sys
d = json.loads(sys.argv[1])
print(json.dumps({'outcome': d.get('outcome'), 'selected_candidate_id': d.get('selected_candidate_id'), 'receipt_hash': d.get('receipt_hash')}, indent=2))
" "$body"
assert_field "get receipt" "receipt_hash" "$body"
pass "receipt retrieved"

# ── STEP 5 — dispatch receipt ─────────────────────────────────────────────────

hr "5. Dispatch receipt (POST /dispatch/$RECEIPT_HASH)"
resp=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/dispatch/$RECEIPT_HASH")
body=$(echo "$resp" | head -n -1)
status=$(echo "$resp" | tail -n 1)
# 200 on first dispatch; if smoke test is re-run, dispatch already exists → 409
# Both are acceptable: the receipt was dispatched.
if [[ "$status" == "409" ]]; then
  echo "  (already dispatched — idempotent re-run)"
elif [[ "$status" == "200" ]]; then
  echo "$body" | python3 -m json.tool
  assert_field "dispatch" "receipt_hash" "$body"
else
  assert_status "dispatch" "200" "$status"
fi
pass "receipt dispatched"

# ── STEP 6 — verify dispatched receipt ───────────────────────────────────────

hr "6. Verify dispatch (POST /dispatch/$RECEIPT_HASH/verify)"
resp=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/dispatch/$RECEIPT_HASH/verify")
body=$(echo "$resp" | head -n -1)
status=$(echo "$resp" | tail -n 1)
assert_status "dispatch verify" "200" "$status"
echo "$body" | python3 -m json.tool
RESULT=$(python3 -c "import json,sys; print(json.loads(sys.argv[1])['result'])" "$body")
[[ "$RESULT" == "VERIFIED" ]] || fail "dispatch verify: expected VERIFIED, got $RESULT"
pass "dispatch verified (result=VERIFIED)"

# ── STEP 7 — route history ────────────────────────────────────────────────────

hr "7. Route history (GET /routes)"
resp=$(curl -s -w "\n%{http_code}" "$BASE_URL/routes")
body=$(echo "$resp" | head -n -1)
status=$(echo "$resp" | tail -n 1)
assert_status "route history" "200" "$status"
echo "$body" | python3 -m json.tool
ROUTE_COUNT=$(python3 -c "
import json,sys
d = json.loads(sys.argv[1])
routes = d.get('routes', d.get('route_history', []))
print(len(routes))
" "$body")
[[ "$ROUTE_COUNT" -ge 1 ]] || fail "route history: expected at least 1 route, got $ROUTE_COUNT"
pass "route history ($ROUTE_COUNT route(s))"

# ── summary ───────────────────────────────────────────────────────────────────

echo ""
echo "════════════════════════════════════════"
echo "  SMOKE TEST PASSED — all 7 steps OK"
echo "════════════════════════════════════════"
