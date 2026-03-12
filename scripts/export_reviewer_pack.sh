#!/usr/bin/env bash
# PostCAD — Reviewer Pack Export
#
# Collects the minimum external-facing evidence from a frozen pilot smoke run
# into one clean folder that can be handed to an outside reviewer.
#
# Usage:
#   scripts/export_reviewer_pack.sh
#
# Output:
#   release/out/reviewer-pack-pilot-local-v1/
#
# Prerequisites: clean working tree, all fixtures present
# Exit: 0 = pack exported, nonzero = FAIL

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PILOT_VERSION="pilot-local-v1"
PACK_NAME="reviewer-pack-${PILOT_VERSION}"
OUT_DIR="${REPO_ROOT}/release/out/${PACK_NAME}"

# ── Helpers ────────────────────────────────────────────────────────────────────

banner() {
    echo ""
    echo "══════════════════════════════════════════"
    echo "  $*"
    echo "══════════════════════════════════════════"
}
stage() { echo ""; echo "── $* ──────────────────────────────────────"; }
ok()    { echo "  [OK]   $*"; }
fail()  { echo "" >&2; echo "  [FAIL] $*" >&2; exit 1; }

# ── Banner ─────────────────────────────────────────────────────────────────────

banner "PostCAD Reviewer Pack Export"
echo "  pilot  : ${PILOT_VERSION}"
echo "  output : ${OUT_DIR}"

# ── A. PREFLIGHT ───────────────────────────────────────────────────────────────

stage "A. Preflight"

cd "${REPO_ROOT}"

# Repo root sanity
[[ -f "${REPO_ROOT}/Cargo.toml" ]] \
    || fail "Cargo.toml not found — run from repo root or scripts/ directory"
ok "repo root: ${REPO_ROOT}"

# Clean working tree
UNCLEAN=$(git status --porcelain 2>/dev/null || true)
if [[ -n "${UNCLEAN}" ]]; then
    echo "" >&2
    echo "  [FAIL] Working tree is not clean. Commit or stash changes first:" >&2
    echo "${UNCLEAN}" | sed 's/^/           /' >&2
    exit 1
fi
ok "working tree: clean"

# Commit and pilot label
GIT_COMMIT="$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")"
GIT_COMMIT_FULL="$(git rev-parse HEAD 2>/dev/null || echo "unknown")"
GIT_BRANCH="$(git branch --show-current 2>/dev/null || echo "unknown")"
echo ""
echo "  commit : ${GIT_COMMIT_FULL}"
echo "  branch : ${GIT_BRANCH}"
echo "  pilot  : ${PILOT_VERSION}"

# Required source files
declare -A SOURCES
SOURCES["release/RELEASE_NOTES_PILOT.md"]="release/RELEASE_NOTES_PILOT.md"
SOURCES["release/version/PILOT_VERSION.md"]="release/version/PILOT_VERSION.md"
SOURCES["release/external/EXTERNAL_DELIVERY_OVERVIEW.md"]="release/external/EXTERNAL_DELIVERY_OVERVIEW.md"
SOURCES["release/external/EXTERNAL_REVIEW_PATH.md"]="release/external/EXTERNAL_REVIEW_PATH.md"
SOURCES["release/external/EXTERNAL_BOUNDARIES.md"]="release/external/EXTERNAL_BOUNDARIES.md"
SOURCES["docs/external_pilot_smoke.md"]="docs/external_pilot_smoke.md"
SOURCES["examples/pilot/case.json"]="examples/pilot/case.json"
SOURCES["examples/pilot/registry_snapshot.json"]="examples/pilot/registry_snapshot.json"
SOURCES["examples/pilot/config.json"]="examples/pilot/config.json"
SOURCES["examples/pilot/expected_routed.json"]="examples/pilot/expected_routed.json"
SOURCES["scripts/external_pilot_smoke.sh"]="scripts/external_pilot_smoke.sh"

for path in "${!SOURCES[@]}"; do
    [[ -f "${REPO_ROOT}/${path}" ]] || fail "Required file not found: ${path}"
    ok "found: ${path}"
done

echo ""
echo "  [PASS] preflight"

# ── B. BUILD PACK ─────────────────────────────────────────────────────────────

stage "B. Build pack"

# Replace output folder cleanly
if [[ -d "${OUT_DIR}" ]]; then
    rm -rf "${OUT_DIR}"
    ok "removed existing: ${OUT_DIR}"
fi
mkdir -p "${OUT_DIR}/fixtures"
ok "created: ${OUT_DIR}"

# Copy release and review docs
cp "${REPO_ROOT}/release/RELEASE_NOTES_PILOT.md"                       "${OUT_DIR}/RELEASE_NOTES_PILOT.md"
cp "${REPO_ROOT}/release/version/PILOT_VERSION.md"                     "${OUT_DIR}/PILOT_VERSION.md"
cp "${REPO_ROOT}/release/external/EXTERNAL_DELIVERY_OVERVIEW.md"       "${OUT_DIR}/EXTERNAL_DELIVERY_OVERVIEW.md"
cp "${REPO_ROOT}/release/external/EXTERNAL_REVIEW_PATH.md"             "${OUT_DIR}/EXTERNAL_REVIEW_PATH.md"
cp "${REPO_ROOT}/release/external/EXTERNAL_BOUNDARIES.md"              "${OUT_DIR}/EXTERNAL_BOUNDARIES.md"
cp "${REPO_ROOT}/docs/external_pilot_smoke.md"                         "${OUT_DIR}/external_pilot_smoke.md"

# Copy canonical pilot fixtures
cp "${REPO_ROOT}/examples/pilot/case.json"                             "${OUT_DIR}/fixtures/case.json"
cp "${REPO_ROOT}/examples/pilot/registry_snapshot.json"                "${OUT_DIR}/fixtures/registry_snapshot.json"
cp "${REPO_ROOT}/examples/pilot/config.json"                           "${OUT_DIR}/fixtures/config.json"
cp "${REPO_ROOT}/examples/pilot/expected_routed.json"                  "${OUT_DIR}/fixtures/expected_routed.json"

ok "files copied"

# ── C. GENERATE README ────────────────────────────────────────────────────────

stage "C. Generate README.md"

cat > "${OUT_DIR}/README.md" <<READMEEOF
# PostCAD Reviewer Pack — ${PILOT_VERSION}

**Commit:** ${GIT_COMMIT_FULL}
**Branch:** ${GIT_BRANCH}
**Pilot label:** \`${PILOT_VERSION}\`

---

## What this pack proves

- The PostCAD routing service can be built from source and run locally with no external dependencies.
- The canonical pilot case (\`f1000001-0000-0000-0000-000000000001\`, DE, zirconia crown) routes deterministically to \`pilot-de-001\`.
- The routing receipt hash is frozen: \`0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb\`
- The receipt passes independent cryptographic verification (\`result: VERIFIED\`) without re-routing.
- The full intake → route → dispatch → verify flow completes cleanly end-to-end.
- An outside reviewer with \`cargo\`, \`curl\`, and \`python3\` can reproduce this independently from a clean checkout.

---

## Included files

| File | Purpose |
|---|---|
| \`README.md\` | This file — entry point for the pack |
| \`MANIFEST.txt\` | Inventory of all files in this pack |
| \`RELEASE_NOTES_PILOT.md\` | What the pilot release includes; frozen protocol values; out-of-scope statement |
| \`PILOT_VERSION.md\` | Pilot label definition and commit verification steps |
| \`EXTERNAL_DELIVERY_OVERVIEW.md\` | What the package demonstrates; key properties; evidence surfaces |
| \`EXTERNAL_REVIEW_PATH.md\` | 8-step external inspection path with stop conditions |
| \`EXTERNAL_BOUNDARIES.md\` | Explicit scope boundaries — what this package is and is not |
| \`external_pilot_smoke.md\` | Quickstart for running the one-command end-to-end smoke run |
| \`fixtures/case.json\` | Canonical pilot case input |
| \`fixtures/registry_snapshot.json\` | Manufacturer registry used for routing |
| \`fixtures/config.json\` | Routing configuration |
| \`fixtures/expected_routed.json\` | Frozen expected routing receipt (anchor for hash verification) |

---

## Reproduce the pilot run

From the repo root (not this folder), with a clean working tree:

\`\`\`bash
scripts/external_pilot_smoke.sh
\`\`\`

No prior setup beyond \`cargo\`, \`curl\`, and \`python3\`. The script builds the service, runs the full pilot flow, and shuts down on exit.

See \`external_pilot_smoke.md\` in this pack for prerequisites, expected output, and failure fixes.

---

## What success looks like

\`\`\`
SMOKE RUN PASSED — 10 stages OK

  commit        : ${GIT_COMMIT}  (${GIT_BRANCH})
  pilot label   : ${PILOT_VERSION}
  case_id       : f1000001-0000-0000-0000-000000000001
  receipt_hash  : 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb
  verify result : VERIFIED
\`\`\`

---

## Frozen reference values

| Item | Value |
|---|---|
| Protocol version | \`postcad-v1\` |
| Routing kernel | \`postcad-routing-v1\` |
| Receipt schema version | \`1\` |
| Canonical case ID | \`f1000001-0000-0000-0000-000000000001\` |
| Deterministic receipt hash | \`0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb\` |
| Expected verify result | \`VERIFIED\` |

Fixture paths in the live repo:
- \`examples/pilot/case.json\`
- \`examples/pilot/registry_snapshot.json\`
- \`examples/pilot/config.json\`
- \`examples/pilot/expected_routed.json\`
READMEEOF

ok "README.md written"

# ── D. GENERATE MANIFEST ──────────────────────────────────────────────────────

stage "D. Generate MANIFEST.txt"

cat > "${OUT_DIR}/MANIFEST.txt" <<MANIFESTEOF
PostCAD Reviewer Pack — ${PILOT_VERSION}
commit: ${GIT_COMMIT_FULL}
branch: ${GIT_BRANCH}
generated: $(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date)

files:
README.md
MANIFEST.txt
RELEASE_NOTES_PILOT.md
PILOT_VERSION.md
EXTERNAL_DELIVERY_OVERVIEW.md
EXTERNAL_REVIEW_PATH.md
EXTERNAL_BOUNDARIES.md
external_pilot_smoke.md
fixtures/case.json
fixtures/registry_snapshot.json
fixtures/config.json
fixtures/expected_routed.json
MANIFESTEOF

ok "MANIFEST.txt written"

# ── E. SUMMARY ────────────────────────────────────────────────────────────────

banner "REVIEWER PACK EXPORTED"
echo ""
echo "  location : ${OUT_DIR}"
echo "  pilot    : ${PILOT_VERSION}"
echo "  commit   : ${GIT_COMMIT_FULL}"
echo ""
echo "  Contents:"
find "${OUT_DIR}" -type f | sort | sed "s|${OUT_DIR}/|    |"
echo ""
echo "  To rerun the pilot smoke from the repo root:"
echo "    scripts/external_pilot_smoke.sh"
echo ""
