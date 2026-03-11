#!/usr/bin/env bash
# PostCAD pilot release — generate deterministic evidence bundle.
#
# Usage (from repo root or release/ directory):
#   ./release/generate_evidence_bundle.sh
#
# Requires: postcad-service is already running on localhost:8080.
# Start it first with: ./release/start_pilot.sh
#
# Produces: release/evidence/current/
#   Replaces any existing current/ folder cleanly.
#   Contains captured API responses, copied inputs, and a summary.
#
# Does NOT start or stop the service.
# Does NOT modify any canonical fixtures or protocol artifacts.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BASE_URL="${POSTCAD_ADDR:-http://localhost:8080}"
REGISTRY="$REPO_ROOT/examples/pilot/registry_snapshot.json"
CONFIG="$REPO_ROOT/examples/pilot/config.json"
CASE_FIXTURE="$REPO_ROOT/examples/pilot/case.json"
OUT="$SCRIPT_DIR/evidence/current"
DATA_DIR="${POSTCAD_DATA:-$REPO_ROOT/data}"

# Canonical pilot case — same case_id as smoke_test.sh; deterministic every run.
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

step()  { echo ""; echo "  [evidence] $*"; }
ok()    { echo "  [OK]  $*"; }
skip()  { echo "  [--]  $*"; }
abort() { echo ""; echo "  [FAIL] $*" >&2; exit 1; }

pretty_json() {
  python3 -c "import json,sys; print(json.dumps(json.loads(sys.argv[1]), indent=2))" "$1" 2>/dev/null \
    || echo "$1"
}

# ── pre-flight ────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════"
echo "  PostCAD Pilot — Evidence Bundle"
echo "══════════════════════════════════════════"
echo "  Repo root : $REPO_ROOT"
echo "  Base URL  : $BASE_URL"
echo "  Output    : $OUT"
echo "══════════════════════════════════════════"

for tool in curl python3; do
  command -v "$tool" &>/dev/null || abort "Pre-flight: '$tool' not found on PATH"
done

for fixture in "$REGISTRY" "$CONFIG" "$CASE_FIXTURE"; do
  [[ -f "$fixture" ]] || abort "Pre-flight: fixture not found: $fixture"
done

REACH=$(curl -s -o /dev/null -w "%{http_code}" --max-time 3 "$BASE_URL/health" 2>/dev/null || echo "000")
if [[ "$REACH" != "200" ]]; then
  echo "" >&2
  abort "Cannot reach $BASE_URL/health (got HTTP $REACH). Start the service first:
  ./release/start_pilot.sh"
fi
ok "Service reachable (HTTP $REACH)"

# ── prepare output folder ─────────────────────────────────────────────────────

step "Preparing output folder: $OUT"
rm -rf "$OUT"
mkdir -p "$OUT/inputs"
ok "Output folder ready"

# ── git context ───────────────────────────────────────────────────────────────

step "Capturing git context"
git -C "$REPO_ROOT" rev-parse HEAD > "$OUT/git_head.txt" 2>/dev/null \
  || echo "(not a git repo or git not available)" > "$OUT/git_head.txt"
ok "git_head.txt written"

# ── copy inputs ───────────────────────────────────────────────────────────────

step "Copying pilot input fixtures"
cp "$CASE_FIXTURE"  "$OUT/inputs/case.json"
cp "$REGISTRY"      "$OUT/inputs/registry_snapshot.json"
cp "$CONFIG"        "$OUT/inputs/config.json"
ok "Copied: case.json, registry_snapshot.json, config.json → inputs/"

# ── record commands used ──────────────────────────────────────────────────────

cat > "$OUT/commands.txt" <<'CMDS'
Evidence captured using the following HTTP calls against postcad-service:

01  GET  /health
02  POST /cases                       (store canonical pilot case)
03  POST /cases/:id/route             (route the stored case)
04  GET  /receipts/:hash              (retrieve routing receipt)
05  POST /dispatch/:hash              (dispatch the receipt)
06  POST /dispatch/:hash/verify       (verify dispatched receipt)
07  GET  /routes                      (route history)

Fixture inputs used:
  examples/pilot/case.json
  examples/pilot/registry_snapshot.json
  examples/pilot/config.json

Pilot case_id: f1000001-0000-0000-0000-000000000001
CMDS
ok "commands.txt written"

# ── step 01 — health ──────────────────────────────────────────────────────────

step "01 — GET /health"
BODY=$(curl -s "$BASE_URL/health")
pretty_json "$BODY" > "$OUT/01_health.json"
ok "01_health.json saved"

# ── step 02 — store case ──────────────────────────────────────────────────────

step "02 — POST /cases (store pilot case)"
RESP=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/cases" \
  -H "Content-Type: application/json" -d "$PILOT_CASE")
BODY=$(echo "$RESP" | head -n -1)
STATUS=$(echo "$RESP" | tail -n 1)
# 200 = already stored (idempotent re-run), 201 = freshly created
[[ "$STATUS" == "200" || "$STATUS" == "201" ]] || abort "Step 02 /cases returned HTTP $STATUS"
pretty_json "$BODY" > "$OUT/02_store_case.json"
CASE_ID=$(python3 -c "import json,sys; print(json.loads(sys.argv[1])['case_id'])" "$BODY")
ok "02_store_case.json saved (case_id=$CASE_ID)"

# ── step 03 — route case ──────────────────────────────────────────────────────

# Endpoint: POST /cases/:id/route
# Request:  {"registry": [...], "config": {...}}
# Response: {"case_id": "...", "receipt_hash": "...", "selected_candidate_id": "..."}

step "03 — POST /cases/$CASE_ID/route"
ROUTE_PAYLOAD=$(python3 -c "
import json
registry = json.load(open('$REGISTRY'))
config   = json.load(open('$CONFIG'))
print(json.dumps({'registry': registry, 'config': config}))
")
RESP=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/cases/$CASE_ID/route" \
  -H "Content-Type: application/json" -d "$ROUTE_PAYLOAD")
BODY=$(echo "$RESP" | head -n -1)
STATUS=$(echo "$RESP" | tail -n 1)
[[ "$STATUS" == "200" ]] || abort "Step 03 /cases/:id/route returned HTTP $STATUS"
pretty_json "$BODY" > "$OUT/03_route_case.json"
RECEIPT_HASH=$(python3 -c "
import json,sys
d = json.loads(sys.argv[1])
print(d['receipt_hash'])
" "$BODY")
SELECTED=$(python3 -c "
import json,sys
d = json.loads(sys.argv[1])
v = d.get('selected_candidate_id')
print(v if v else '')
" "$BODY")
[[ -n "$SELECTED" ]] || abort "Step 03: selected_candidate_id is empty (routing refused?)"
ok "03_route_case.json saved (selected=$SELECTED, receipt_hash=${RECEIPT_HASH:0:16}…)"

# ── step 04 — retrieve receipt ────────────────────────────────────────────────

step "04 — GET /receipts/$RECEIPT_HASH"
RESP=$(curl -s -w "\n%{http_code}" "$BASE_URL/receipts/$RECEIPT_HASH")
BODY=$(echo "$RESP" | head -n -1)
STATUS=$(echo "$RESP" | tail -n 1)
[[ "$STATUS" == "200" ]] || abort "Step 04 /receipts/:hash returned HTTP $STATUS"
pretty_json "$BODY" > "$OUT/04_receipt.json"
ok "04_receipt.json saved"

# ── step 05 — dispatch receipt ────────────────────────────────────────────────

step "05 — POST /dispatch/$RECEIPT_HASH"
RESP=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/dispatch/$RECEIPT_HASH")
BODY=$(echo "$RESP" | head -n -1)
STATUS=$(echo "$RESP" | tail -n 1)
if [[ "$STATUS" == "200" ]]; then
  pretty_json "$BODY" > "$OUT/05_dispatch.json"
  ok "05_dispatch.json saved"
elif [[ "$STATUS" == "409" ]]; then
  echo '{"note": "already dispatched — 409 accepted as idempotent", "receipt_hash": "'"$RECEIPT_HASH"'"}' \
    > "$OUT/05_dispatch.json"
  ok "05_dispatch.json saved (already dispatched — idempotent)"
else
  abort "Step 05 /dispatch/:hash returned HTTP $STATUS"
fi

# ── step 06 — verify dispatched receipt ───────────────────────────────────────

step "06 — POST /dispatch/$RECEIPT_HASH/verify"
RESP=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/dispatch/$RECEIPT_HASH/verify")
BODY=$(echo "$RESP" | head -n -1)
STATUS=$(echo "$RESP" | tail -n 1)
[[ "$STATUS" == "200" ]] || abort "Step 06 /dispatch/:hash/verify returned HTTP $STATUS"
pretty_json "$BODY" > "$OUT/06_verify.json"
RESULT=$(python3 -c "import json,sys; print(json.loads(sys.argv[1])['result'])" "$BODY")
[[ "$RESULT" == "VERIFIED" ]] || abort "Step 06: expected VERIFIED, got $RESULT"
ok "06_verify.json saved (result=$RESULT)"

# ── step 07 — route history ───────────────────────────────────────────────────

step "07 — GET /routes"
RESP=$(curl -s -w "\n%{http_code}" "$BASE_URL/routes")
BODY=$(echo "$RESP" | head -n -1)
STATUS=$(echo "$RESP" | tail -n 1)
[[ "$STATUS" == "200" ]] || abort "Step 07 /routes returned HTTP $STATUS"
pretty_json "$BODY" > "$OUT/07_route_history.json"
ROUTE_COUNT=$(python3 -c "
import json,sys
d = json.loads(sys.argv[1])
routes = d.get('routes', d.get('route_history', []))
print(len(routes))
" "$BODY")
ok "07_route_history.json saved ($ROUTE_COUNT route(s))"

# ── copy local data artifacts if present ──────────────────────────────────────

step "Copying local data artifacts (if present)"
copied_data=0
for subdir in cases receipts policies dispatch verification; do
  src="$DATA_DIR/$subdir"
  if [[ -d "$src" ]]; then
    dst="$OUT/data_artifacts/$subdir"
    mkdir -p "$dst"
    # copy all .json files one level deep
    while IFS= read -r -d '' f; do
      cp "$f" "$dst/"
      copied_data=$((copied_data + 1))
    done < <(find "$src" -maxdepth 1 -name "*.json" -print0 2>/dev/null)
  fi
done
if [[ "$copied_data" -gt 0 ]]; then
  ok "Copied $copied_data data artifact file(s) → data_artifacts/"
else
  skip "No local data artifacts found (data/ not yet written, or POSTCAD_DATA overridden)"
fi

# ── write summary ─────────────────────────────────────────────────────────────

step "Writing summary.txt"
GIT_HEAD=$(cat "$OUT/git_head.txt")
cat > "$OUT/summary.txt" <<SUMMARY
PostCAD Pilot Evidence Bundle
=============================

Repository : $REPO_ROOT
Git HEAD   : $GIT_HEAD
Base URL   : $BASE_URL

Pilot inputs
------------
  inputs/case.json               canonical pilot case (case_id=$CASE_ID)
  inputs/registry_snapshot.json  manufacturer registry used for routing
  inputs/config.json             routing config (jurisdiction + policy)

API evidence (7 steps)
-----------------------
  01_health.json         GET /health → HTTP 200
  02_store_case.json     POST /cases → HTTP 200 (case_id=$CASE_ID)
  03_route_case.json     POST /cases/:id/route → HTTP 200 (selected=$SELECTED)
  04_receipt.json        GET /receipts/:hash → HTTP 200 (hash=${RECEIPT_HASH:0:16}…)
  05_dispatch.json       POST /dispatch/:hash → HTTP 200 or 409 (idempotent)
  06_verify.json         POST /dispatch/:hash/verify → HTTP 200 (result=$RESULT)
  07_route_history.json  GET /routes → HTTP 200 ($ROUTE_COUNT route(s))

Context
-------
  git_head.txt     commit hash at time of evidence capture
  commands.txt     full list of commands and fixture paths used

Data artifacts
--------------
  data_artifacts/  copies of files written by postcad-service under data/
                   (empty if service used a non-default POSTCAD_DATA path)

Verification check
------------------
  selected  = $SELECTED
  result    = $RESULT  (expected: VERIFIED)
  routes    = $ROUTE_COUNT route(s) in history

All 7 steps passed.
SUMMARY
ok "summary.txt written"

# ── done ─────────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════"
echo "  EVIDENCE BUNDLE COMPLETE"
echo "  Output: $OUT"
echo "══════════════════════════════════════════"
echo ""
echo "  Inspect the bundle:"
echo "    cat $OUT/summary.txt"
echo "    ls  $OUT/"
echo ""
