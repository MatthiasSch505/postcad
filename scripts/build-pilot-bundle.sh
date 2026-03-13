#!/usr/bin/env bash
# PostCAD — Build Pilot Bundle
#
# Copies canonical scripts, fixtures, and docs into release/pilot/.
# Deterministic: same source files always produce the same bundle.
# No network calls. No compilation. No side effects outside release/pilot/.
#
# Usage:
#   ./scripts/build-pilot-bundle.sh
#
# Output: release/pilot/ populated with all bundle assets + MANIFEST.txt

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BUNDLE="${REPO_ROOT}/release/pilot"
PILOT="${REPO_ROOT}/examples/pilot"
DOCS="${REPO_ROOT}/docs"

put() {
    local src="$1" dst="$2"
    cp "${REPO_ROOT}/${src}" "${BUNDLE}/${dst}"
    printf "  copied  %-45s → release/pilot/%s\n" "${src}" "${dst}"
}

printf "\nPostCAD — Build Pilot Bundle\n"
printf "==============================\n"
printf "  bundle: %s\n\n" "${BUNDLE}"

# ── Directories ───────────────────────────────────────────────────────────────

mkdir -p "${BUNDLE}/docs"

# ── Scripts ───────────────────────────────────────────────────────────────────

put "examples/pilot/preflight.sh"  "preflight.sh"
put "examples/pilot/demo.sh"       "demo.sh"
chmod +x "${BUNDLE}/preflight.sh" "${BUNDLE}/demo.sh"

# ── Fixtures ──────────────────────────────────────────────────────────────────

for f in \
    case.json \
    registry_snapshot.json \
    config.json \
    derived_policy.json \
    candidates.json \
    expected_routed.json \
    expected_verify.json; do
    put "examples/pilot/${f}" "${f}"
done

# ── Docs ──────────────────────────────────────────────────────────────────────

put "docs/openapi.yaml"         "docs/openapi.yaml"
put "docs/protocol_diagram.md"  "docs/protocol_diagram.md"

# ── MANIFEST.txt (source provenance, human-readable) ─────────────────────────

{
    printf "# PostCAD Pilot Bundle — File Manifest\n"
    printf "# Generated: %s\n" "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    printf "# Commit:    %s\n" "$(git -C "${REPO_ROOT}" rev-parse HEAD 2>/dev/null || echo 'unknown')"
    printf "#\n"
    printf "# Format: <bundle path>  <source path>\n"
    printf "#\n"
    for entry in \
        "INVENTORY.md              release/pilot/INVENTORY.md" \
        "README.md                 release/pilot/README.md" \
        "candidates.json           examples/pilot/candidates.json" \
        "case.json                 examples/pilot/case.json" \
        "config.json               examples/pilot/config.json" \
        "demo.sh                   examples/pilot/demo.sh" \
        "derived_policy.json       examples/pilot/derived_policy.json" \
        "docs/openapi.yaml         docs/openapi.yaml" \
        "docs/protocol_diagram.md  docs/protocol_diagram.md" \
        "expected_routed.json      examples/pilot/expected_routed.json" \
        "expected_verify.json      examples/pilot/expected_verify.json" \
        "preflight.sh              examples/pilot/preflight.sh" \
        "registry_snapshot.json    examples/pilot/registry_snapshot.json"; do
        printf "%s\n" "${entry}"
    done
} > "${BUNDLE}/MANIFEST.txt"
printf "  wrote   MANIFEST.txt\n"

# ── manifest.sha256 (machine-verifiable, sha256sum -c compatible) ─────────────

# Enumerate all bundle files, sorted, excluding the manifest itself and runtime artifacts.
(
    cd "${BUNDLE}"
    find . -type f \
        | grep -v '^./manifest\.sha256$' \
        | grep -v '^./export_packet\.json$' \
        | sort \
        | sed 's|^\./||' \
        | xargs sha256sum \
        > manifest.sha256
)
printf "  wrote   manifest.sha256\n"

# ── Summary ───────────────────────────────────────────────────────────────────

FILE_COUNT=$(find "${BUNDLE}" -type f | wc -l | tr -d ' ')

printf "\nBundle complete — %s file(s) in release/pilot/\n" "${FILE_COUNT}"
printf "\nContents:\n"
find "${BUNDLE}" -type f | sort | sed "s|${BUNDLE}/||" | sed 's/^/  /'
printf "\n"
