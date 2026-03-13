#!/usr/bin/env bash
# PostCAD — Canonical Pilot Demo
#
# Runs the full PostCAD pilot protocol end-to-end:
#   route → dispatch → approve → export → verify
#
# Requirements:
#   - postcad-service running on http://localhost:8080
#     Start with: cargo run -p postcad-service
#   - curl and python3 installed
#
# Usage:
#   ./examples/pilot/demo.sh
#
# Override the service address:
#   POSTCAD_ADDR=http://localhost:9000 ./examples/pilot/demo.sh

set -euo pipefail

BASE_URL="${POSTCAD_ADDR:-http://localhost:8080}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXPORT_FILE="$SCRIPT_DIR/export_packet.json"

pass() { printf "  [OK]  %s\n" "$*"; }
fail() { printf "  [FAIL] %s\n" "$*" >&2; exit 1; }
sep()  { printf "\n── %s ──\n" "$*"; }

# Portable JSON field extraction via python3 (no jq required).
# jget <json_string> <dotted.key>   → string value
# jobj <json_string> <key>          → compact JSON object/array
jget() { python3 -c "import sys,json; d=json.loads(sys.argv[1]); v=d; [v:=v[k] for k in sys.argv[2].split('.')]; print(v)" "$1" "$2"; }
jobj() { python3 -c "import sys,json; d=json.loads(sys.argv[1]); print(json.dumps(d[sys.argv[2]],separators=(',',':')))" "$1" "$2"; }

# ── Preflight ────────────────────────────────────────────────────────────────
"${SCRIPT_DIR}/preflight.sh" || exit 1

REGISTRY=$(cat "$SCRIPT_DIR/registry_snapshot.json")
CONFIG=$(cat "$SCRIPT_DIR/config.json")
CASE=$(cat "$SCRIPT_DIR/case.json")

printf "══════════════════════════════════════════\n"
printf "  PostCAD — Canonical Pilot Demo\n"
printf "  Service: %s\n" "$BASE_URL"
printf "══════════════════════════════════════════\n"

# ── 0: Health check ───────────────────────────────────────────────────────────

sep "0 — Health"
curl -sf "$BASE_URL/health" > /dev/null \
  || fail "Service not reachable at $BASE_URL — start it with: cargo run -p postcad-service"
pass "Service is up"

# ── 1: Route normalized pilot case ────────────────────────────────────────────

sep "1 — POST /pilot/route-normalized"
ROUTE=$(curl -sf -X POST "$BASE_URL/pilot/route-normalized" \
  -H "Content-Type: application/json" \
  -d "{
    \"pilot_case\": {
      \"case_id\":          \"f1000001-0000-0000-0000-000000000001\",
      \"restoration_type\": \"crown\",
      \"material\":         \"zirconia\",
      \"jurisdiction\":     \"DE\"
    },
    \"registry_snapshot\": $REGISTRY,
    \"routing_config\":    $CONFIG
  }") || fail "POST /pilot/route-normalized failed"

OUTCOME=$(jget "$ROUTE" "receipt.outcome")
SELECTED=$(jget "$ROUTE" "receipt.selected_candidate_id")
RECEIPT_HASH=$(jget "$ROUTE" "receipt.receipt_hash")
RECEIPT_OBJ=$(jobj "$ROUTE" "receipt")
POLICY_OBJ=$(jobj "$ROUTE" "derived_policy")

[[ "$OUTCOME" == "routed" ]] || fail "Expected outcome=routed, got: $OUTCOME"
pass "Routed → candidate=$SELECTED"
printf "  receipt_hash: %s\n" "$RECEIPT_HASH"
printf "  ↳ Next: bind receipt to a dispatch commitment\n"

# ── 2: Create dispatch (re-runs verification inline) ─────────────────────────

sep "2 — POST /dispatch/create"
DISPATCH=$(curl -sf -X POST "$BASE_URL/dispatch/create" \
  -H "Content-Type: application/json" \
  -d "{
    \"receipt\": $RECEIPT_OBJ,
    \"case\":    $CASE,
    \"policy\":  $POLICY_OBJ
  }") || fail "POST /dispatch/create failed"

DISPATCH_ID=$(jget "$DISPATCH" "dispatch_id")
DISPATCH_STATUS=$(jget "$DISPATCH" "status")
[[ "$DISPATCH_STATUS" == "draft" ]] || fail "Expected status=draft, got: $DISPATCH_STATUS"
pass "Dispatch created, verification_passed=true  id=$DISPATCH_ID"
printf "  ↳ Next: reviewer approval\n"

# ── 3: Approve ────────────────────────────────────────────────────────────────

sep "3 — POST /dispatch/$DISPATCH_ID/approve"
APPROVE=$(curl -sf -X POST "$BASE_URL/dispatch/$DISPATCH_ID/approve" \
  -H "Content-Type: application/json" \
  -d '{"approved_by": "reviewer"}') || fail "POST /dispatch/approve failed"

APPROVE_STATUS=$(jget "$APPROVE" "status")
[[ "$APPROVE_STATUS" == "approved" ]] || fail "Expected status=approved, got: $APPROVE_STATUS"
pass "Dispatch approved by reviewer"
printf "  ↳ Next: export canonical handoff packet\n"

# ── 4: Export ─────────────────────────────────────────────────────────────────

sep "4 — GET /dispatch/$DISPATCH_ID/export"
EXPORT=$(curl -sf "$BASE_URL/dispatch/$DISPATCH_ID/export") \
  || fail "GET /dispatch/export failed"

EXPORT_STATUS=$(jget "$EXPORT" "status")
EXPORT_HASH=$(jget "$EXPORT" "receipt_hash")
[[ "$EXPORT_STATUS" == "exported" ]] || fail "Expected status=exported, got: $EXPORT_STATUS"
python3 -c "import sys,json; print(json.dumps(json.loads(sys.argv[1]),indent=2))" "$EXPORT" > "$EXPORT_FILE"
pass "Export packet saved → examples/pilot/export_packet.json"
printf "  receipt_hash: %s\n" "$EXPORT_HASH"
printf "  ↳ Next: independent verification replay\n"

# ── 5: Verify (independent replay — no stored state trusted) ──────────────────

sep "5 — POST /verify  (independent replay)"
VERIFY=$(curl -sf -X POST "$BASE_URL/verify" \
  -H "Content-Type: application/json" \
  -d "{
    \"receipt\": $RECEIPT_OBJ,
    \"case\":    $CASE,
    \"policy\":  $POLICY_OBJ
  }") || fail "POST /verify failed"

RESULT=$(jget "$VERIFY" "result")
[[ "$RESULT" == "VERIFIED" ]] || fail "Verification failed: $VERIFY"
pass "Independent replay → VERIFIED"

# ── Summary ───────────────────────────────────────────────────────────────────

printf "\n══════════════════════════════════════════\n"
printf "  DEMO COMPLETE\n"
printf "  Candidate:    %s\n" "$SELECTED"
printf "  Receipt hash: %s\n" "$RECEIPT_HASH"
printf "  Dispatch ID:  %s\n" "$DISPATCH_ID"
printf "  Export file:  examples/pilot/export_packet.json\n"
printf "  Verification: VERIFIED\n"
printf "══════════════════════════════════════════\n"
printf "\n"
printf "  To inspect the export packet:\n"
printf "    cat examples/pilot/export_packet.json\n"
printf "\n"
printf "  To use the interactive reviewer shell:\n"
printf "    open http://localhost:8080/reviewer\n"
printf "\n"
