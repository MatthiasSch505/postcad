#!/usr/bin/env bash
# PostCAD — Normalized Pilot Flow
#
# Runs the full normalized pilot protocol from the command line using curl.
#
# Usage:
#   ./examples/pilot/run_normalized_pilot.sh
#
# Requirements:
#   - postcad-service running (default: http://localhost:3000)
#   - curl and jq installed
#
# Override the service address:
#   POSTCAD_ADDR=http://localhost:8080 ./examples/pilot/run_normalized_pilot.sh
#
# Flow:
#   Step 1 — POST /pilot/route-normalized   → receipt + derived_policy
#   Step 2 — POST /dispatch/create          → dispatch_id (draft)
#   Step 3 — POST /dispatch/:id/approve     → status: approved
#   Step 4 — GET  /dispatch/:id/export      → canonical export packet

set -euo pipefail

BASE_URL="${POSTCAD_ADDR:-http://localhost:3000}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

pass() { echo "  [OK]  $*"; }
fail() { echo "  [FAIL] $*" >&2; exit 1; }

# ── Load pilot registry and config from the same directory ───────────────────

REGISTRY=$(cat "$SCRIPT_DIR/registry_snapshot.json")
CONFIG=$(cat "$SCRIPT_DIR/config.json")

echo ""
echo "══════════════════════════════════════════"
echo "  PostCAD — Normalized Pilot Flow"
echo "  Service: $BASE_URL"
echo "══════════════════════════════════════════"
echo ""

# ── Step 1: Route normalized pilot case ──────────────────────────────────────

echo "Step 1 — POST /pilot/route-normalized"

# case_id c0000001-... is the canonical UUID form of "case-001"
ROUTE_RESPONSE=$(curl -sf -X POST "$BASE_URL/pilot/route-normalized" \
  -H "Content-Type: application/json" \
  -d "{
    \"pilot_case\": {
      \"case_id\":          \"c0000001-0000-0000-0000-000000000001\",
      \"restoration_type\": \"crown\",
      \"material\":         \"zirconia\",
      \"jurisdiction\":     \"DE\"
    },
    \"registry_snapshot\": $REGISTRY,
    \"routing_config\":    $CONFIG
  }") || fail "Step 1: POST /pilot/route-normalized failed"

OUTCOME=$(echo "$ROUTE_RESPONSE"  | jq -r '.receipt.outcome')
SELECTED=$(echo "$ROUTE_RESPONSE" | jq -r '.receipt.selected_candidate_id')
RECEIPT_HASH=$(echo "$ROUTE_RESPONSE" | jq -r '.receipt.receipt_hash')

[[ "$OUTCOME" == "routed" ]] || fail "Step 1: expected outcome=routed, got: $OUTCOME"
pass "Step 1 — routed"

echo ""
echo "  Selected Candidate:  $SELECTED"
echo "  Receipt Hash:        $RECEIPT_HASH"
echo ""

# ── Step 2: Create dispatch ───────────────────────────────────────────────────

echo "Step 2 — POST /dispatch/create"

# The receipt's routing_input contains the full CaseInput shape needed for
# dispatch verification; extract it directly instead of re-specifying.
RECEIPT_OBJ=$(echo "$ROUTE_RESPONSE"  | jq '.receipt')
CASE_OBJ=$(echo "$ROUTE_RESPONSE"     | jq '.receipt.routing_input')
POLICY_OBJ=$(echo "$ROUTE_RESPONSE"   | jq '.derived_policy')

DISPATCH_RESPONSE=$(curl -sf -X POST "$BASE_URL/dispatch/create" \
  -H "Content-Type: application/json" \
  -d "{
    \"receipt\": $RECEIPT_OBJ,
    \"case\":    $CASE_OBJ,
    \"policy\":  $POLICY_OBJ
  }") || fail "Step 2: POST /dispatch/create failed"

DISPATCH_ID=$(echo "$DISPATCH_RESPONSE" | jq -r '.dispatch_id')
DISPATCH_STATUS=$(echo "$DISPATCH_RESPONSE" | jq -r '.status')

[[ "$DISPATCH_STATUS" == "draft" ]] || fail "Step 2: expected status=draft, got: $DISPATCH_STATUS"
pass "Step 2 — dispatch created"

echo ""
echo "  Dispatch ID:         $DISPATCH_ID"
echo "  Status:              $DISPATCH_STATUS"
echo ""

# ── Step 3: Approve dispatch ──────────────────────────────────────────────────

echo "Step 3 — POST /dispatch/$DISPATCH_ID/approve"

APPROVE_RESPONSE=$(curl -sf -X POST "$BASE_URL/dispatch/$DISPATCH_ID/approve" \
  -H "Content-Type: application/json" \
  -d '{"approved_by": "operator"}') || fail "Step 3: approve failed"

APPROVE_STATUS=$(echo "$APPROVE_RESPONSE" | jq -r '.status')

[[ "$APPROVE_STATUS" == "approved" ]] || fail "Step 3: expected status=approved, got: $APPROVE_STATUS"
pass "Step 3 — dispatch approved"
echo ""

# ── Step 4: Export dispatch packet ────────────────────────────────────────────

echo "Step 4 — GET /dispatch/$DISPATCH_ID/export"

EXPORT_RESPONSE=$(curl -sf "$BASE_URL/dispatch/$DISPATCH_ID/export") \
  || fail "Step 4: export failed"

EXPORT_STATUS=$(echo "$EXPORT_RESPONSE" | jq -r '.status')

[[ "$EXPORT_STATUS" == "exported" ]] || fail "Step 4: expected status=exported, got: $EXPORT_STATUS"
pass "Step 4 — dispatch exported"

echo ""
echo "  Export Packet:"
echo "$EXPORT_RESPONSE" | jq .
echo ""

# ── Summary ───────────────────────────────────────────────────────────────────

echo "══════════════════════════════════════════"
echo "  FLOW COMPLETE"
echo "  Selected Candidate:  $SELECTED"
echo "  Receipt Hash:        $RECEIPT_HASH"
echo "  Dispatch ID:         $DISPATCH_ID"
echo "══════════════════════════════════════════"
echo ""
