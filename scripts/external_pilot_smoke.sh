#!/usr/bin/env bash
# PostCAD — External Pilot Smoke Run
#
# One-command end-to-end pilot validation from a clean checkout.
# No prior operator knowledge required.
#
# Usage:
#   scripts/external_pilot_smoke.sh
#   POSTCAD_SMOKE_PORT=9090 scripts/external_pilot_smoke.sh
#
# Prerequisites: cargo, curl, python3
# Exit: 0 = PASS (all stages), nonzero = FAIL
#
# What it does:
#   Preflights the environment, builds the service if needed, starts it in
#   the background, runs the canonical 7-step pilot flow against the frozen
#   fixtures, and confirms the frozen receipt_hash.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PILOT_DIR="${REPO_ROOT}/examples/pilot"
SVC_BIN="${REPO_ROOT}/target/debug/postcad-service"
PORT="${POSTCAD_SMOKE_PORT:-8080}"
BASE_URL="http://127.0.0.1:${PORT}"
PILOT_VERSION="pilot-local-v1"

SVC_PID=""
PASS_COUNT=0
WORK_DIR="$(mktemp -d)"

# ── Cleanup ────────────────────────────────────────────────────────────────────

cleanup() {
    if [[ -n "${SVC_PID}" ]]; then
        kill "${SVC_PID}" 2>/dev/null || true
        wait "${SVC_PID}" 2>/dev/null || true
    fi
    rm -rf "${WORK_DIR}"
}
trap cleanup EXIT

# ── Helpers ────────────────────────────────────────────────────────────────────

banner() {
    echo ""
    echo "══════════════════════════════════════════"
    echo "  $*"
    echo "══════════════════════════════════════════"
}

stage() { echo ""; echo "── $* ──────────────────────────────────────"; }
ok()    { echo "  [OK]   $*"; }
pass()  { echo "  [PASS] $*"; PASS_COUNT=$((PASS_COUNT + 1)); }
fail()  { echo "" >&2; echo "  [FAIL] $*" >&2; exit 1; }

# Extract a top-level JSON field from a string.
json_field() {
    python3 -c "
import json, sys
print(json.loads(sys.argv[1]).get(sys.argv[2]) or '')
" "$1" "$2"
}

# ── Banner ─────────────────────────────────────────────────────────────────────

banner "PostCAD External Pilot Smoke Run"
echo "  repo     : ${REPO_ROOT}"
echo "  fixtures : ${PILOT_DIR}"
echo "  port     : ${PORT}"

# ── A. PREFLIGHT ───────────────────────────────────────────────────────────────

stage "A. Preflight"

# Required tools
for tool in cargo curl python3; do
    if ! command -v "${tool}" &>/dev/null; then
        fail "Required tool not found: '${tool}' — install it before running this script"
    fi
    ok "${tool}: $(command -v "${tool}")"
done

# Repo root sanity
[[ -f "${REPO_ROOT}/Cargo.toml" ]] \
    || fail "Cargo.toml not found at ${REPO_ROOT} — run from repo root or scripts/ directory"
ok "repo root: ${REPO_ROOT}"

# Clean working tree
cd "${REPO_ROOT}"
UNCLEAN=$(git status --porcelain 2>/dev/null || true)
if [[ -n "${UNCLEAN}" ]]; then
    echo "" >&2
    echo "  [FAIL] Working tree is not clean. Commit or stash changes first:" >&2
    echo "${UNCLEAN}" | sed 's/^/           /' >&2
    exit 1
fi
ok "working tree: clean"

# Print commit and pilot label
GIT_COMMIT="$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")"
GIT_BRANCH="$(git branch --show-current 2>/dev/null || echo "unknown")"
echo ""
echo "  commit : ${GIT_COMMIT}  (${GIT_BRANCH})"
echo "  pilot  : ${PILOT_VERSION}"

# Required fixtures
for f in \
    "${PILOT_DIR}/case.json" \
    "${PILOT_DIR}/registry_snapshot.json" \
    "${PILOT_DIR}/config.json" \
    "${PILOT_DIR}/derived_policy.json" \
    "${PILOT_DIR}/expected_routed.json"; do
    [[ -f "$f" ]] || fail "Pilot fixture not found: $f"
    ok "fixture: $(basename "$f")"
done

pass "preflight"

# ── B.1 BUILD ─────────────────────────────────────────────────────────────────

stage "B.1 Build"

if [[ -x "${SVC_BIN}" ]]; then
    ok "binary already built: ${SVC_BIN}"
else
    echo "  Building postcad-service (this may take a minute)..."
    cargo build --bin postcad-service --quiet --manifest-path "${REPO_ROOT}/Cargo.toml"
    ok "build complete"
fi

pass "binary ready"

# ── B.2 START SERVICE ─────────────────────────────────────────────────────────

stage "B.2 Start service"

POSTCAD_ADDR="127.0.0.1:${PORT}" "${SVC_BIN}" \
    >"${WORK_DIR}/service.stdout" 2>"${WORK_DIR}/service.stderr" &
SVC_PID=$!
ok "service started (pid=${SVC_PID})"

MAX_WAIT=20
echo "  Waiting for /health (up to ${MAX_WAIT}s)..."
for i in $(seq 1 "${MAX_WAIT}"); do
    HEALTH="$(curl -sf "${BASE_URL}/health" 2>/dev/null || true)"
    if [[ "${HEALTH}" == '{"status":"ok"}' ]]; then
        ok "service reachable at ${BASE_URL}"
        break
    fi
    if [[ "${i}" -eq "${MAX_WAIT}" ]]; then
        echo "  service stderr:" >&2
        cat "${WORK_DIR}/service.stderr" >&2
        fail "service did not respond at ${BASE_URL}/health after ${MAX_WAIT}s (port ${PORT} in use?)"
    fi
    sleep 1
done

pass "service up"

# ── B.3 PILOT FLOW ────────────────────────────────────────────────────────────

# Canonical pilot identifiers — these never change for this pilot release.
CANONICAL_CASE_ID="f1000001-0000-0000-0000-000000000001"
FROZEN_RECEIPT_HASH="0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb"

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

# Step 1 — Health
stage "1/7  GET /health"
resp=$(curl -s -w "\n%{http_code}" "${BASE_URL}/health")
body=$(echo "$resp" | head -n -1)
status=$(echo "$resp" | tail -n 1)
[[ "$status" == "200" ]] || fail "health: expected HTTP 200, got $status"
pass "health (${body})"

# Step 2 — Store case
stage "2/7  POST /cases  (store pilot case)"
resp=$(curl -s -w "\n%{http_code}" -X POST "${BASE_URL}/cases" \
    -H "Content-Type: application/json" -d "${PILOT_CASE}")
body=$(echo "$resp" | head -n -1)
status=$(echo "$resp" | tail -n 1)
[[ "$status" == "200" || "$status" == "201" ]] \
    || fail "store case: expected HTTP 200 or 201, got $status"
CASE_ID=$(json_field "$body" "case_id")
[[ "$CASE_ID" == "$CANONICAL_CASE_ID" ]] \
    || fail "store case: unexpected case_id: '$CASE_ID' (expected '${CANONICAL_CASE_ID}')"
pass "case stored (case_id=${CASE_ID})"

# Step 3 — Route stored case
stage "3/7  POST /cases/${CASE_ID}/route  (route stored case)"
ROUTE_PAYLOAD=$(python3 -c "
import json
registry = json.load(open('${PILOT_DIR}/registry_snapshot.json'))
config   = json.load(open('${PILOT_DIR}/config.json'))
print(json.dumps({'registry': registry, 'config': config}))
")
resp=$(curl -s -w "\n%{http_code}" -X POST "${BASE_URL}/cases/${CASE_ID}/route" \
    -H "Content-Type: application/json" -d "${ROUTE_PAYLOAD}")
body=$(echo "$resp" | head -n -1)
status=$(echo "$resp" | tail -n 1)
[[ "$status" == "200" ]] || fail "route: expected HTTP 200, got $status"
RECEIPT_HASH=$(json_field "$body" "receipt_hash")
SELECTED=$(json_field "$body" "selected_candidate_id")
[[ -n "$SELECTED" ]] || fail "route: selected_candidate_id is empty (routing refused?)"
echo "  selected : ${SELECTED}"
echo "  hash     : ${RECEIPT_HASH}"
pass "case routed (selected=${SELECTED})"

# Step 4 — Confirm frozen receipt hash
stage "4/7  Receipt hash  (frozen value check)"
if [[ "$RECEIPT_HASH" == "$FROZEN_RECEIPT_HASH" ]]; then
    pass "receipt_hash matches frozen value"
else
    echo "  got      : ${RECEIPT_HASH}" >&2
    echo "  expected : ${FROZEN_RECEIPT_HASH}" >&2
    fail "receipt_hash mismatch — routing output does not match the frozen pilot receipt"
fi

# Step 5 — Retrieve receipt
stage "5/7  GET /receipts/${RECEIPT_HASH}"
resp=$(curl -s -w "\n%{http_code}" "${BASE_URL}/receipts/${RECEIPT_HASH}")
body=$(echo "$resp" | head -n -1)
status=$(echo "$resp" | tail -n 1)
[[ "$status" == "200" ]] || fail "get receipt: expected HTTP 200, got $status"
OUTCOME=$(json_field "$body" "outcome")
[[ "$OUTCOME" == "routed" ]] \
    || fail "get receipt: expected outcome=routed, got '${OUTCOME}'"
pass "receipt retrieved (outcome=${OUTCOME})"

# Step 6 — Dispatch receipt
stage "6/7  POST /dispatch/${RECEIPT_HASH}"
resp=$(curl -s -w "\n%{http_code}" -X POST "${BASE_URL}/dispatch/${RECEIPT_HASH}")
body=$(echo "$resp" | head -n -1)
status=$(echo "$resp" | tail -n 1)
if [[ "$status" == "409" ]]; then
    ok "already dispatched (idempotent)"
elif [[ "$status" == "200" ]]; then
    ok "dispatched"
else
    fail "dispatch: expected HTTP 200 or 409, got $status"
fi
pass "receipt dispatched"

# Step 7 — Verify dispatched receipt
stage "7/7  POST /dispatch/${RECEIPT_HASH}/verify"
resp=$(curl -s -w "\n%{http_code}" -X POST "${BASE_URL}/dispatch/${RECEIPT_HASH}/verify")
body=$(echo "$resp" | head -n -1)
status=$(echo "$resp" | tail -n 1)
[[ "$status" == "200" ]] || fail "verify dispatch: expected HTTP 200, got $status"
RESULT=$(json_field "$body" "result")
[[ "$RESULT" == "VERIFIED" ]] \
    || fail "verify dispatch: expected VERIFIED, got '${RESULT}'"
pass "dispatch verified (result=VERIFIED)"

# ── C. SUCCESS SUMMARY ────────────────────────────────────────────────────────

banner "SMOKE RUN PASSED — ${PASS_COUNT} stages OK"
echo ""
echo "  commit        : ${GIT_COMMIT}  (${GIT_BRANCH})"
echo "  pilot label   : ${PILOT_VERSION}"
echo "  case_id       : ${CASE_ID}"
echo "  receipt_hash  : ${RECEIPT_HASH}"
echo "  verify result : VERIFIED"
echo ""
echo "  Fixtures used:"
echo "    ${PILOT_DIR}/case.json"
echo "    ${PILOT_DIR}/registry_snapshot.json"
echo "    ${PILOT_DIR}/config.json"
echo "    ${PILOT_DIR}/derived_policy.json"
echo ""
echo "  Endpoints exercised:"
echo "    GET  ${BASE_URL}/health"
echo "    POST ${BASE_URL}/cases"
echo "    POST ${BASE_URL}/cases/${CASE_ID}/route"
echo "    GET  ${BASE_URL}/receipts/${RECEIPT_HASH}"
echo "    POST ${BASE_URL}/dispatch/${RECEIPT_HASH}"
echo "    POST ${BASE_URL}/dispatch/${RECEIPT_HASH}/verify"
echo ""
