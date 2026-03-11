#!/usr/bin/env bash
# PostCAD Pilot Acceptance Runner
#
# Validates the full locked pilot flow automatically:
#   /health → /version → /route (compare to expected_routed.json)
#                      → /verify (compare to expected_verify.json)
#
# Usage:
#   scripts/pilot_acceptance.sh
#   POSTCAD_ACCEPTANCE_PORT=9090 scripts/pilot_acceptance.sh
#
# Exit: 0 on PASS, nonzero on any FAIL.
# Temp: written to a mktemp directory; deleted on exit.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PILOT_DIR="${REPO_ROOT}/examples/pilot"
SVC_BIN="${REPO_ROOT}/target/debug/postcad-service"
PORT="${POSTCAD_ACCEPTANCE_PORT:-8080}"
BASE_URL="http://127.0.0.1:${PORT}"

WORK_DIR="$(mktemp -d)"
SVC_PID=""
PASS_COUNT=0
FAIL_COUNT=0

# ── Cleanup ───────────────────────────────────────────────────────────────────

cleanup() {
    if [[ -n "${SVC_PID}" ]]; then
        kill "${SVC_PID}" 2>/dev/null || true
        wait "${SVC_PID}" 2>/dev/null || true
    fi
    rm -rf "${WORK_DIR}"
}
trap cleanup EXIT

# ── Helpers ───────────────────────────────────────────────────────────────────

step()  { echo; echo "── ${*}"; }
ok()    { echo "  [PASS] ${*}"; PASS_COUNT=$((PASS_COUNT + 1)); }
fail()  { echo "  [FAIL] ${*}" >&2; FAIL_COUNT=$((FAIL_COUNT + 1)); exit 1; }

# Compare two JSON files for value equality using python3.
# Arguments: <actual_file> <expected_file> <label>
json_eq() {
    local actual="$1" expected="$2" label="$3"
    python3 - "${actual}" "${expected}" <<'PYEOF'
import json, sys
actual   = json.load(open(sys.argv[1]))
expected = json.load(open(sys.argv[2]))
if actual != expected:
    print("--- actual (truncated) ---",   file=sys.stderr)
    print(json.dumps(actual,   sort_keys=True)[:500], file=sys.stderr)
    print("--- expected (truncated) ---", file=sys.stderr)
    print(json.dumps(expected, sort_keys=True)[:500], file=sys.stderr)
    sys.exit(1)
PYEOF
    ok "${label}"
}

# ── Banner ────────────────────────────────────────────────────────────────────

echo "PostCAD Pilot Acceptance Runner"
echo "================================"
echo "  fixtures : ${PILOT_DIR}"
echo "  binary   : ${SVC_BIN}"
echo "  port     : ${PORT}"
echo "  work dir : ${WORK_DIR}"

# ── Step 1: Build ─────────────────────────────────────────────────────────────

step "1/8  build"
if [[ ! -x "${SVC_BIN}" ]]; then
    echo "  building postcad-service..."
    cargo build --bin postcad-service --quiet \
        --manifest-path "${REPO_ROOT}/Cargo.toml"
fi
ok "binary present"

# ── Step 2: Start service ─────────────────────────────────────────────────────

step "2/8  start service"
POSTCAD_ADDR="127.0.0.1:${PORT}" "${SVC_BIN}" \
    >"${WORK_DIR}/service.stdout" 2>"${WORK_DIR}/service.stderr" &
SVC_PID=$!
echo "  pid: ${SVC_PID}"

# ── Step 3: Wait for /health ──────────────────────────────────────────────────

step "3/8  health"
MAX_WAIT=20
for i in $(seq 1 "${MAX_WAIT}"); do
    HEALTH="$(curl -sf "${BASE_URL}/health" 2>/dev/null || true)"
    if [[ "${HEALTH}" == '{"status":"ok"}' ]]; then
        ok "/health → ${HEALTH}"
        break
    fi
    if [[ "${i}" -eq "${MAX_WAIT}" ]]; then
        echo "  service stderr:" >&2
        cat "${WORK_DIR}/service.stderr" >&2
        fail "/health did not respond after ${MAX_WAIT}s"
    fi
    sleep 1
done

# ── Step 4: Version ───────────────────────────────────────────────────────────

step "4/8  version"
curl -sf "${BASE_URL}/version" >"${WORK_DIR}/version.json"
python3 - "${WORK_DIR}/version.json" <<'PYEOF'
import json, sys
v = json.load(open(sys.argv[1]))
want = {
    "protocol_version":       "postcad-v1",
    "routing_kernel_version": "postcad-routing-v1",
    "service":                "postcad-service",
}
for key, val in want.items():
    if v.get(key) != val:
        print(f"  expected {key}={val!r}, got {v.get(key)!r}", file=sys.stderr)
        sys.exit(1)
PYEOF
ok "/version fields verified"

# ── Step 5: Route ─────────────────────────────────────────────────────────────

step "5/8  route"
curl -sf -X POST "${BASE_URL}/route" \
    -H 'Content-Type: application/json' \
    -d "{
          \"case\":              $(cat "${PILOT_DIR}/case.json"),
          \"registry_snapshot\": $(cat "${PILOT_DIR}/registry_snapshot.json"),
          \"routing_config\":    $(cat "${PILOT_DIR}/config.json")
        }" \
    >"${WORK_DIR}/route_response.json"

# Extract .receipt from the route response for comparison.
python3 -c "
import json, sys
resp = json.load(open('${WORK_DIR}/route_response.json'))
json.dump(resp['receipt'], open('${WORK_DIR}/actual_receipt.json', 'w'), indent=2, sort_keys=True)
"

# Normalise expected fixture.
python3 -c "
import json
data = json.load(open('${PILOT_DIR}/expected_routed.json'))
json.dump(data, open('${WORK_DIR}/expected_receipt.json', 'w'), indent=2, sort_keys=True)
"

json_eq "${WORK_DIR}/actual_receipt.json" \
        "${WORK_DIR}/expected_receipt.json" \
        "/route receipt matches expected_routed.json"

# ── Step 6: Verify ────────────────────────────────────────────────────────────

step "6/8  verify"
curl -sf -X POST "${BASE_URL}/verify" \
    -H 'Content-Type: application/json' \
    -d "{
          \"receipt\": $(cat "${PILOT_DIR}/expected_routed.json"),
          \"case\":    $(cat "${PILOT_DIR}/case.json"),
          \"policy\":  $(cat "${PILOT_DIR}/derived_policy.json")
        }" \
    >"${WORK_DIR}/verify_response.json"

# Normalise both sides for comparison.
python3 -c "
import json
data = json.load(open('${WORK_DIR}/verify_response.json'))
json.dump(data, open('${WORK_DIR}/actual_verify.json', 'w'), indent=2, sort_keys=True)
"
python3 -c "
import json
data = json.load(open('${PILOT_DIR}/expected_verify.json'))
json.dump(data, open('${WORK_DIR}/expected_verify.json', 'w'), indent=2, sort_keys=True)
"

json_eq "${WORK_DIR}/actual_verify.json" \
        "${WORK_DIR}/expected_verify.json" \
        "/verify result matches expected_verify.json"

# ── Step 7: Locked fixture check ──────────────────────────────────────────────

step "7/8  locked file integrity"
cd "${REPO_ROOT}"
DIRTY=$(git diff -- examples/pilot/ 2>/dev/null || true)
if [[ -n "${DIRTY}" ]]; then
    echo "  examples/pilot/ has uncommitted changes:" >&2
    echo "${DIRTY}" >&2
    fail "locked fixtures were modified during acceptance run"
fi
ok "examples/pilot/ is clean"

# ── Step 8: Summary ───────────────────────────────────────────────────────────

step "8/8  summary"
echo
echo "================================"
echo "  RESULT : PASS"
echo "  checks : ${PASS_COUNT} passed, ${FAIL_COUNT} failed"
echo "================================"
