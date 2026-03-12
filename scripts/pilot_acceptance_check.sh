#!/usr/bin/env bash
# PostCAD — Pilot Acceptance Check
#
# Verification gate for the frozen pilot-local-v1 state.
# Validates git state, required files, reviewer pack, frozen receipt hash,
# and smoke run entry point before external sharing.
#
# Usage:
#   scripts/pilot_acceptance_check.sh
#
# This script does NOT modify any system state.
# It does NOT run the full smoke flow.
# Exit: 0 = all checks PASS, nonzero = at least one FAIL

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PILOT_VERSION="pilot-local-v1"
FROZEN_RECEIPT_HASH="0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb"
PACK_DIR="${REPO_ROOT}/release/out/reviewer-pack-${PILOT_VERSION}"

OVERALL_PASS=true

# ── Helpers ────────────────────────────────────────────────────────────────────

# Pad a label to a fixed width for aligned output.
pad() {
    local label="$1" width=45
    printf "%-${width}s" "${label}"
}

check_pass() { echo "$(pad "$1")PASS"; }
check_fail() { echo "$(pad "$1")FAIL  — $2" >&2; OVERALL_PASS=false; }

require_file() {
    local path="$1" label="$2"
    if [[ -f "${REPO_ROOT}/${path}" ]]; then
        echo "  [OK]  ${path}"
    else
        echo "  [--]  ${path}  ← NOT FOUND" >&2
        return 1
    fi
}

# ── Banner ─────────────────────────────────────────────────────────────────────

echo ""
echo "PostCAD Pilot Acceptance Check"
echo "==============================="
echo "  pilot   : ${PILOT_VERSION}"
echo "  repo    : ${REPO_ROOT}"
echo ""

cd "${REPO_ROOT}"

# ══════════════════════════════════════════════════════════════════════════════
# CHECK 1 — git state
# ══════════════════════════════════════════════════════════════════════════════

echo "── CHECK 1  git state ──────────────────────────────────"

C1_PASS=true

# Working tree clean
UNCLEAN=$(git status --porcelain 2>/dev/null || true)
if [[ -n "${UNCLEAN}" ]]; then
    echo "  [--]  working tree is dirty:" >&2
    echo "${UNCLEAN}" | sed 's/^/        /' >&2
    C1_PASS=false
else
    echo "  [OK]  working tree: clean"
fi

# HEAD commit
GIT_COMMIT="$(git rev-parse HEAD 2>/dev/null || echo "unknown")"
GIT_COMMIT_SHORT="$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")"
echo "  [OK]  commit: ${GIT_COMMIT}"

# Branch
GIT_BRANCH="$(git branch --show-current 2>/dev/null || echo "unknown")"
echo "  [OK]  branch: ${GIT_BRANCH}"

# Pilot version marker file exists
if [[ -f "${REPO_ROOT}/release/version/PILOT_VERSION.md" ]]; then
    echo "  [OK]  pilot version marker: release/version/PILOT_VERSION.md"
else
    echo "  [--]  pilot version marker not found: release/version/PILOT_VERSION.md" >&2
    C1_PASS=false
fi

if ${C1_PASS}; then check_pass "CHECK 1 — git state"; else check_fail "CHECK 1 — git state" "see above"; OVERALL_PASS=false; fi

# ══════════════════════════════════════════════════════════════════════════════
# CHECK 2 — required pilot files
# ══════════════════════════════════════════════════════════════════════════════

echo ""
echo "── CHECK 2  pilot files ────────────────────────────────"

C2_PASS=true

REQUIRED_FILES=(
    "release/version/PILOT_VERSION.md"
    "release/RELEASE_NOTES_PILOT.md"
    "docs/external_pilot_smoke.md"
    "scripts/external_pilot_smoke.sh"
    "scripts/export_reviewer_pack.sh"
    "examples/pilot/case.json"
    "examples/pilot/config.json"
    "examples/pilot/registry_snapshot.json"
    "examples/pilot/expected_routed.json"
)

for f in "${REQUIRED_FILES[@]}"; do
    require_file "$f" || C2_PASS=false
done

if ${C2_PASS}; then check_pass "CHECK 2 — pilot files"; else check_fail "CHECK 2 — pilot files" "one or more required files missing"; OVERALL_PASS=false; fi

# ══════════════════════════════════════════════════════════════════════════════
# CHECK 3 — reviewer pack export
# ══════════════════════════════════════════════════════════════════════════════

echo ""
echo "── CHECK 3  reviewer pack export ───────────────────────"

C3_PASS=true

# Run the export script
echo "  Running: scripts/export_reviewer_pack.sh"
if "${REPO_ROOT}/scripts/export_reviewer_pack.sh" > /dev/null 2>&1; then
    echo "  [OK]  export script completed"
else
    echo "  [--]  export script failed" >&2
    C3_PASS=false
fi

# Confirm output folder exists
if [[ -d "${PACK_DIR}" ]]; then
    echo "  [OK]  pack folder: ${PACK_DIR}"
else
    echo "  [--]  pack folder not found: ${PACK_DIR}" >&2
    C3_PASS=false
fi

# Confirm required files inside the pack
if ${C3_PASS}; then
    PACK_FILES=(
        "README.md"
        "MANIFEST.txt"
        "PILOT_VERSION.md"
        "RELEASE_NOTES_PILOT.md"
        "external_pilot_smoke.md"
        "fixtures/case.json"
        "fixtures/config.json"
        "fixtures/expected_routed.json"
        "fixtures/registry_snapshot.json"
    )
    for f in "${PACK_FILES[@]}"; do
        if [[ -f "${PACK_DIR}/${f}" ]]; then
            echo "  [OK]  pack: ${f}"
        else
            echo "  [--]  pack missing: ${f}" >&2
            C3_PASS=false
        fi
    done
fi

if ${C3_PASS}; then check_pass "CHECK 3 — reviewer pack export"; else check_fail "CHECK 3 — reviewer pack export" "see above"; OVERALL_PASS=false; fi

# ══════════════════════════════════════════════════════════════════════════════
# CHECK 4 — frozen receipt hash
# ══════════════════════════════════════════════════════════════════════════════

echo ""
echo "── CHECK 4  frozen receipt hash ────────────────────────"

C4_PASS=true

FIXTURE="${REPO_ROOT}/examples/pilot/expected_routed.json"

if [[ ! -f "${FIXTURE}" ]]; then
    echo "  [--]  fixture not found: examples/pilot/expected_routed.json" >&2
    C4_PASS=false
else
    ACTUAL_HASH=$(python3 -c "
import json, sys
d = json.load(open(sys.argv[1]))
print(d.get('receipt_hash', ''))
" "${FIXTURE}")

    echo "  expected : ${FROZEN_RECEIPT_HASH}"
    echo "  actual   : ${ACTUAL_HASH}"

    if [[ "${ACTUAL_HASH}" == "${FROZEN_RECEIPT_HASH}" ]]; then
        echo "  [OK]  receipt_hash matches frozen reference"
    else
        echo "  [--]  receipt_hash MISMATCH" >&2
        C4_PASS=false
    fi
fi

if ${C4_PASS}; then check_pass "CHECK 4 — frozen receipt hash"; else check_fail "CHECK 4 — frozen receipt hash" "hash in expected_routed.json does not match frozen reference"; OVERALL_PASS=false; fi

# ══════════════════════════════════════════════════════════════════════════════
# CHECK 5 — smoke run entry point
# ══════════════════════════════════════════════════════════════════════════════

echo ""
echo "── CHECK 5  smoke run entry point ──────────────────────"

C5_PASS=true

SMOKE_SCRIPT="${REPO_ROOT}/scripts/external_pilot_smoke.sh"

if [[ -f "${SMOKE_SCRIPT}" ]]; then
    echo "  [OK]  file exists: scripts/external_pilot_smoke.sh"
else
    echo "  [--]  not found: scripts/external_pilot_smoke.sh" >&2
    C5_PASS=false
fi

if [[ -x "${SMOKE_SCRIPT}" ]]; then
    echo "  [OK]  executable"
else
    echo "  [--]  not executable — run: chmod +x scripts/external_pilot_smoke.sh" >&2
    C5_PASS=false
fi

if ${C5_PASS}; then check_pass "CHECK 5 — smoke run entry point"; else check_fail "CHECK 5 — smoke run entry point" "see above"; OVERALL_PASS=false; fi

# ══════════════════════════════════════════════════════════════════════════════
# SUMMARY
# ══════════════════════════════════════════════════════════════════════════════

echo ""
echo "══════════════════════════════════════════════════════"
if ${OVERALL_PASS}; then
    echo "  PILOT ACCEPTANCE: PASS"
else
    echo "  PILOT ACCEPTANCE: FAIL" >&2
fi
echo "══════════════════════════════════════════════════════"
echo ""
echo "  commit       : ${GIT_COMMIT}"
echo "  branch       : ${GIT_BRANCH}"
echo "  pilot label  : ${PILOT_VERSION}"
echo "  reviewer pack: ${PACK_DIR}"
echo ""

if ! ${OVERALL_PASS}; then
    exit 1
fi
