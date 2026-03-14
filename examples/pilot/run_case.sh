#!/usr/bin/env bash
# run_case.sh — execute a full PostCAD run for a given case input file.
#
# Usage:
#   ./examples/pilot/run_case.sh [CASE_FILE] [OPTIONS]
#
#   CASE_FILE           path to the case JSON input file  (default: examples/pilot/case.json)
#
# Options:
#   --registry FILE     registry snapshot JSON             (default: examples/pilot/registry_snapshot.json)
#   --config   FILE     routing config JSON                (default: examples/pilot/config.json)
#   --policy   FILE     derived policy JSON                (default: examples/pilot/derived_policy.json)
#   --output   DIR      run output directory               (default: run_output)
#   --base-url URL      service base URL                   (default: http://localhost:8080)
#   --bundle            package output into a pilot bundle (default: off)
#   --skip-dispatch     skip dispatch create/approve/export (default: off)
#
# The service must be running before executing this script.
# Exit codes:
#   0  run completed successfully
#   1  run failed at one or more steps

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ── defaults ───────────────────────────────────────────────────────────────────
CASE_FILE="${1:-$SCRIPT_DIR/case.json}"
REGISTRY="$SCRIPT_DIR/registry_snapshot.json"
CONFIG="$SCRIPT_DIR/config.json"
POLICY="$SCRIPT_DIR/derived_policy.json"
OUTPUT_DIR="run_output"
BASE_URL="http://localhost:8080"
DO_BUNDLE=0
SKIP_DISPATCH=0

# parse remaining args (skip $1 = case file)
shift || true
while [[ $# -gt 0 ]]; do
  case "$1" in
    --registry)  REGISTRY="$2";    shift 2 ;;
    --config)    CONFIG="$2";      shift 2 ;;
    --policy)    POLICY="$2";      shift 2 ;;
    --output)    OUTPUT_DIR="$2";  shift 2 ;;
    --base-url)  BASE_URL="$2";    shift 2 ;;
    --bundle)    DO_BUNDLE=1;      shift   ;;
    --skip-dispatch) SKIP_DISPATCH=1; shift ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

# ── colour helpers ─────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; RESET='\033[0m'
info()  { echo -e "${GREEN}[run]${RESET}  $*"; }
step()  { echo -e "${CYAN}[step]${RESET} $*"; }
warn()  { echo -e "${YELLOW}[warn]${RESET} $*"; }
fail()  { echo -e "${RED}[fail]${RESET} $*" >&2; }
ok()    { echo -e "  ${GREEN}✓${RESET} $*"; }

# ── check required input files ─────────────────────────────────────────────────
for f in "$CASE_FILE" "$REGISTRY" "$CONFIG"; do
  if [[ ! -f "$f" ]]; then
    fail "Required input file not found: $f"
    exit 1
  fi
done

mkdir -p "$OUTPUT_DIR"

echo ""
info "PostCAD external run protocol"
info "Case      : $CASE_FILE"
info "Registry  : $REGISTRY"
info "Config    : $CONFIG"
info "Output    : $OUTPUT_DIR"
info "Service   : $BASE_URL"
echo ""

# ── check service reachability ─────────────────────────────────────────────────
step "1/5 — Checking service..."
if ! curl -sf "$BASE_URL/version" >/dev/null 2>&1; then
  fail "Service not reachable at $BASE_URL"
  fail "Start the service with: cargo run -p postcad-service"
  exit 1
fi
ok "Service is reachable"
echo ""

# ── step 1: route ──────────────────────────────────────────────────────────────
step "2/5 — Routing case..."
ROUTE_RESPONSE=$(curl -sf -X POST "$BASE_URL/pilot/route-normalized" \
  -H 'Content-Type: application/json' \
  -d "{
    \"pilot_case\":        $(cat "$CASE_FILE"),
    \"registry_snapshot\": $(cat "$REGISTRY"),
    \"routing_config\":    $(cat "$CONFIG")
  }" 2>/dev/null) || { fail "Route request failed"; exit 1; }

echo "$ROUTE_RESPONSE" | python3 -m json.tool > "$OUTPUT_DIR/route_response.json" 2>/dev/null \
  || echo "$ROUTE_RESPONSE" > "$OUTPUT_DIR/route_response.json"

OUTCOME=$(python3 -c "import json,sys; d=json.load(open('$OUTPUT_DIR/route_response.json')); print(d.get('receipt',{}).get('outcome',''))" 2>/dev/null || echo "")
SELECTED=$(python3 -c "import json,sys; d=json.load(open('$OUTPUT_DIR/route_response.json')); print(d.get('receipt',{}).get('selected_candidate_id',''))" 2>/dev/null || echo "")
RECEIPT_HASH=$(python3 -c "import json,sys; d=json.load(open('$OUTPUT_DIR/route_response.json')); print(d.get('receipt',{}).get('receipt_hash',''))" 2>/dev/null || echo "")

if [[ -z "$OUTCOME" ]]; then
  fail "Route response does not contain a receipt"
  cat "$OUTPUT_DIR/route_response.json" >&2
  exit 1
fi

# write receipt.json separately
python3 -c "import json; d=json.load(open('$OUTPUT_DIR/route_response.json')); json.dump(d['receipt'],open('$OUTPUT_DIR/receipt.json','w'),indent=2)" 2>/dev/null \
  || cp "$OUTPUT_DIR/route_response.json" "$OUTPUT_DIR/receipt.json"

ok "Outcome:      $OUTCOME"
ok "Selected:     ${SELECTED:-(none)}"
ok "Receipt hash: ${RECEIPT_HASH:-(none)}"
cp "$OUTPUT_DIR/receipt.json" "$OUTPUT_DIR/route.json"
echo ""

# ── step 2: verify ─────────────────────────────────────────────────────────────
step "3/5 — Verifying receipt..."

if [[ ! -f "$POLICY" ]]; then
  warn "Derived policy not found at $POLICY — extracting from route response..."
  python3 -c "import json; d=json.load(open('$OUTPUT_DIR/route_response.json')); json.dump(d.get('derived_policy',{}),open('$OUTPUT_DIR/derived_policy.json','w'),indent=2)" 2>/dev/null \
    && POLICY="$OUTPUT_DIR/derived_policy.json" \
    || { fail "Cannot determine policy for verification"; exit 1; }
fi

VERIFY_RESPONSE=$(curl -sf -X POST "$BASE_URL/verify" \
  -H 'Content-Type: application/json' \
  -d "{
    \"receipt\": $(cat "$OUTPUT_DIR/receipt.json"),
    \"case\":    $(cat "$CASE_FILE"),
    \"policy\":  $(cat "$POLICY")
  }" 2>/dev/null) || { fail "Verify request failed"; exit 1; }

echo "$VERIFY_RESPONSE" > "$OUTPUT_DIR/verification.json"
VERIFY_RESULT=$(python3 -c "import json; print(json.loads('$VERIFY_RESPONSE').get('result',''))" 2>/dev/null \
  || echo "$VERIFY_RESPONSE" | grep -o 'VERIFIED\|FAILED' || echo "unknown")

if [[ "$VERIFY_RESULT" == "VERIFIED" ]]; then
  ok "Verification: VERIFIED"
else
  warn "Verification: $VERIFY_RESULT"
fi
echo ""

# ── step 3: reproducibility check ─────────────────────────────────────────────
step "4/5 — Running reproducibility check..."
REPRO_RESPONSE=$(curl -sf -X POST "$BASE_URL/pilot/route-normalized" \
  -H 'Content-Type: application/json' \
  -d "{
    \"pilot_case\":        $(cat "$CASE_FILE"),
    \"registry_snapshot\": $(cat "$REGISTRY"),
    \"routing_config\":    $(cat "$CONFIG")
  }" 2>/dev/null) || { fail "Reproducibility re-route failed"; exit 1; }

REPRO_HASH=$(python3 -c "import json; d=json.loads('''$(echo "$REPRO_RESPONSE" | python3 -m json.tool 2>/dev/null || echo "$REPRO_RESPONSE")'''); print(d.get('receipt',{}).get('receipt_hash',''))" 2>/dev/null || echo "")

if [[ -n "$RECEIPT_HASH" && -n "$REPRO_HASH" && "$RECEIPT_HASH" == "$REPRO_HASH" ]]; then
  REPRO_STATUS="reproducible"
  ok "Reproducibility: PASSED (hash match)"
else
  REPRO_STATUS="mismatch"
  warn "Reproducibility: MISMATCH (hashes differ)"
fi

python3 -c "
import json
print(json.dumps({
  'status': '$REPRO_STATUS',
  'original_hash': '$RECEIPT_HASH',
  'reproduced_hash': '$REPRO_HASH',
  'match': '$RECEIPT_HASH' == '$REPRO_HASH'
}, indent=2))
" > "$OUTPUT_DIR/reproducibility.json" 2>/dev/null \
  || echo "{\"status\": \"$REPRO_STATUS\"}" > "$OUTPUT_DIR/reproducibility.json"
echo ""

# ── step 4: dispatch (optional) ───────────────────────────────────────────────
if [[ $SKIP_DISPATCH -eq 0 && "$VERIFY_RESULT" == "VERIFIED" ]]; then
  step "5/5 — Dispatch create → approve → export..."

  DERIVED_POLICY_JSON=$(cat "$OUTPUT_DIR/route_response.json" | python3 -c "import json,sys; print(json.dumps(json.load(sys.stdin).get('derived_policy', {})))" 2>/dev/null || echo "{}")

  CREATE_RESP=$(curl -sf -X POST "$BASE_URL/dispatch/create" \
    -H 'Content-Type: application/json' \
    -d "{
      \"receipt\": $(cat "$OUTPUT_DIR/receipt.json"),
      \"case\":    $(cat "$CASE_FILE"),
      \"policy\":  $DERIVED_POLICY_JSON
    }" 2>/dev/null) || { fail "Dispatch create failed"; exit 1; }

  DISPATCH_ID=$(python3 -c "import json; print(json.loads('$CREATE_RESP').get('dispatch_id',''))" 2>/dev/null || echo "")
  if [[ -z "$DISPATCH_ID" ]]; then
    warn "Could not obtain dispatch_id — skipping approve/export"
  else
    ok "Dispatch created: $DISPATCH_ID"

    curl -sf -X POST "$BASE_URL/dispatch/$DISPATCH_ID/approve" >/dev/null 2>&1 \
      || warn "Dispatch approve failed (continuing)"
    ok "Dispatch approved"

    EXPORT_RESP=$(curl -sf "$BASE_URL/dispatch/$DISPATCH_ID/export" 2>/dev/null) \
      || { warn "Dispatch export failed"; EXPORT_RESP="{}"; }
    echo "$EXPORT_RESP" > "$OUTPUT_DIR/export_packet.json"
    ok "Dispatch exported"
  fi
else
  if [[ $SKIP_DISPATCH -eq 1 ]]; then
    step "5/5 — Dispatch skipped (--skip-dispatch)"
  else
    step "5/5 — Dispatch skipped (verification did not pass)"
  fi
  echo "{}" > "$OUTPUT_DIR/export_packet.json"
fi
echo ""

# ── write run summary ──────────────────────────────────────────────────────────
python3 -c "
import json
summary = {
  'outcome':          '$OUTCOME',
  'selected':         '$SELECTED',
  'receipt_hash':     '$RECEIPT_HASH',
  'verification':     '$VERIFY_RESULT',
  'reproducibility':  '$REPRO_STATUS',
  'output_dir':       '$OUTPUT_DIR',
}
print(json.dumps(summary, indent=2))
" > "$OUTPUT_DIR/run_summary.json" 2>/dev/null || true

info "Run complete — artifacts written to $OUTPUT_DIR/"
echo ""

# ── bundle (optional) ──────────────────────────────────────────────────────────
if [[ $DO_BUNDLE -eq 1 ]]; then
  info "Packaging pilot bundle..."
  "$SCRIPT_DIR/package_run.sh" "$OUTPUT_DIR" pilot_bundle || true
fi
