#!/usr/bin/env bash
# PostCAD — Check Pilot Bundle Integrity
#
# Verifies the release/pilot/ bundle is self-consistent:
#   1. All required files are present
#   2. No unexpected extra files exist
#   3. All SHA-256 hashes match manifest.sha256
#
# Usage:
#   ./scripts/check-pilot-bundle.sh
#
# Exit codes:
#   0 — bundle is intact
#   1 — one or more checks failed (details printed to stderr)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUNDLE="${SCRIPT_DIR}/../release/pilot"

pass() { printf "  [OK]    %s\n" "$*"; }
fail() { printf "  [FAIL]  %s\n" "$*" >&2; FAILED=1; }

FAILED=0

printf "\nPostCAD — Pilot Bundle Integrity Check\n"
printf "========================================\n"
printf "  bundle: %s\n\n" "${BUNDLE}"

# ── Required files ─────────────────────────────────────────────────────────────

REQUIRED=(
    INVENTORY.md
    MANIFEST.txt
    PROTOCOL_WALKTHROUGH.md
    README.md
    REVIEWER_HANDOFF.md
    SEQUENCE_DIAGRAM.md
    manifest.sha256
    preflight.sh
    demo.sh
    case.json
    registry_snapshot.json
    config.json
    derived_policy.json
    candidates.json
    expected_routed.json
    expected_verify.json
    docs/openapi.yaml
    docs/protocol_diagram.md
)

printf "Required files:\n"
for f in "${REQUIRED[@]}"; do
    if [[ -f "${BUNDLE}/${f}" ]]; then
        pass "${f}"
    else
        fail "missing required file: ${f}"
    fi
done

# ── No unexpected extras ───────────────────────────────────────────────────────

printf "\nUnexpected extras:\n"

# Build a set of allowed files (required + runtime artifacts).
declare -A ALLOWED
for f in "${REQUIRED[@]}"; do
    ALLOWED["${f}"]=1
done
# Runtime artifacts produced by demo.sh — allowed but not required.
ALLOWED["export_packet.json"]=1

UNEXPECTED=0
while IFS= read -r -d '' path; do
    rel="${path#${BUNDLE}/}"
    if [[ -z "${ALLOWED[${rel}]+x}" ]]; then
        fail "unexpected file: ${rel}"
        UNEXPECTED=1
    fi
done < <(find "${BUNDLE}" -type f -print0 | sort -z)

if [[ "${UNEXPECTED}" -eq 0 ]]; then
    pass "no unexpected files"
fi

# ── Hash verification ──────────────────────────────────────────────────────────

printf "\nHash verification:\n"

if [[ ! -f "${BUNDLE}/manifest.sha256" ]]; then
    fail "manifest.sha256 is missing — run ./scripts/build-pilot-bundle.sh first"
else
    (
        cd "${BUNDLE}"
        if sha256sum --quiet -c manifest.sha256 2>/dev/null; then
            printf "  [OK]    all hashes match manifest.sha256\n"
        else
            printf "  [FAIL]  one or more hashes do not match manifest.sha256\n" >&2
            # Re-run without --quiet to show which files failed.
            sha256sum -c manifest.sha256 2>&1 | grep -v ': OK$' | sed 's/^/          /' >&2
            FAILED=1
        fi
    ) || FAILED=1
fi

# ── Result ─────────────────────────────────────────────────────────────────────

printf "\n"
if [[ "${FAILED}" -eq 0 ]]; then
    printf "Bundle OK — all checks passed.\n\n"
    exit 0
else
    printf "Bundle FAILED — see errors above.\n\n" >&2
    exit 1
fi
