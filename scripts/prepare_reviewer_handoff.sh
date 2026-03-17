#!/usr/bin/env bash
# PostCAD — Prepare Reviewer Handoff
#
# Single command that runs the acceptance gate, exports the reviewer pack,
# and generates a ready-to-send handoff message.
#
# Usage:
#   scripts/prepare_reviewer_handoff.sh
#
# Output:
#   release/out/reviewer-pack-pilot-local-v1/
#   release/out/reviewer-pack-pilot-local-v1/HANDOFF_MESSAGE.txt
#
# Exit: 0 = ready to send, nonzero = FAIL

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PILOT_VERSION="pilot-local-v1"
PACK_DIR="${REPO_ROOT}/release/out/reviewer-pack-${PILOT_VERSION}"
HANDOFF_MSG="${PACK_DIR}/HANDOFF_MESSAGE.txt"

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

banner "PostCAD Prepare Reviewer Handoff"
echo "  pilot  : ${PILOT_VERSION}"
echo "  repo   : ${REPO_ROOT}"

# ── 1. PREFLIGHT ──────────────────────────────────────────────────────────────

stage "1. Preflight"

cd "${REPO_ROOT}"

[[ -f "${REPO_ROOT}/Cargo.toml" ]] \
    || fail "Cargo.toml not found — run from repo root or scripts/ directory"
ok "repo root: ${REPO_ROOT}"

UNCLEAN=$(git status --porcelain 2>/dev/null || true)
if [[ -n "${UNCLEAN}" ]]; then
    echo "" >&2
    echo "  [FAIL] Working tree is not clean. Commit or stash changes first:" >&2
    echo "${UNCLEAN}" | sed 's/^/           /' >&2
    exit 1
fi
ok "working tree: clean"

GIT_COMMIT="$(git rev-parse HEAD 2>/dev/null || echo "unknown")"
GIT_COMMIT_SHORT="$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")"
GIT_BRANCH="$(git branch --show-current 2>/dev/null || echo "unknown")"
echo ""
echo "  commit : ${GIT_COMMIT}"
echo "  branch : ${GIT_BRANCH}"
echo "  pilot  : ${PILOT_VERSION}"

for script in \
    "${REPO_ROOT}/scripts/pilot_acceptance_check.sh" \
    "${REPO_ROOT}/scripts/export_reviewer_pack.sh"; do
    [[ -f "${script}" ]] || fail "Required script not found: ${script}"
    [[ -x "${script}" ]] || fail "Script not executable: ${script}"
    ok "found: $(basename "${script}")"
done

# ── 2. ACCEPTANCE GATE ────────────────────────────────────────────────────────

stage "2. Acceptance gate"

echo "  Running: scripts/pilot_acceptance_check.sh"
echo ""
if ! "${REPO_ROOT}/scripts/pilot_acceptance_check.sh"; then
    echo "" >&2
    fail "Acceptance check failed — handoff aborted. Fix the issues above and retry."
fi
echo ""
ok "acceptance gate passed"

# ── 3. REVIEWER PACK EXPORT ───────────────────────────────────────────────────

stage "3. Reviewer pack export"

echo "  Running: scripts/export_reviewer_pack.sh"
if ! "${REPO_ROOT}/scripts/export_reviewer_pack.sh" > /dev/null 2>&1; then
    fail "export_reviewer_pack.sh failed"
fi

[[ -d "${PACK_DIR}" ]] \
    || fail "Pack folder not found after export: ${PACK_DIR}"
ok "pack folder: ${PACK_DIR}"

# ── 4. HANDOFF MESSAGE ────────────────────────────────────────────────────────

stage "4. Handoff message"

cat > "${HANDOFF_MSG}" <<MSGEOF
PostCAD Pilot Package — ${PILOT_VERSION}
Commit: ${GIT_COMMIT}

Hi,

I'm sharing the frozen local pilot package for PostCAD. Here is what's
included and how to run it.

What this is:
  A self-contained local pilot of the PostCAD routing and verification
  service. It demonstrates deterministic case routing, cryptographic receipt
  verification, and a complete intake-to-dispatch flow using fixed canonical
  inputs. Everything runs locally — no external services or cloud
  infrastructure required.

To reproduce the pilot run (one command, from repo root):
  scripts/external_pilot_smoke.sh

  Prerequisites: cargo (Rust toolchain), curl, python3
  The script builds the service, runs the full pilot flow, and shuts down
  on exit. Expected final output: "SMOKE RUN PASSED — 10 stages OK"

Entry point for review:
  release/out/reviewer-pack-${PILOT_VERSION}/README.md

  The README lists every included file with its purpose and the frozen
  reference values (protocol version, receipt hash, expected outcome).

Frozen reference values:
  Protocol version : postcad-v1
  Routing kernel   : postcad-routing-v1
  Case ID          : f1000001-0000-0000-0000-000000000001
  Receipt hash     : 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb
  Verify result    : VERIFIED

The package does not include production deployment, regulatory claims, or
coverage beyond the single canonical pilot case.

Let me know if you have any questions or run into any issues.
MSGEOF

ok "HANDOFF_MESSAGE.txt written: ${HANDOFF_MSG}"

# ── 5. SUMMARY ────────────────────────────────────────────────────────────────

banner "HANDOFF READY"
echo ""
echo "  pilot label  : ${PILOT_VERSION}"
echo "  commit       : ${GIT_COMMIT}"
echo "  pack folder  : ${PACK_DIR}"
echo "  handoff msg  : ${HANDOFF_MSG}"
echo ""
echo "  Reviewer rerun command:"
echo "    scripts/external_pilot_smoke.sh"
echo ""
echo "  Pack contents:"
find "${PACK_DIR}" -type f | sort | sed "s|${PACK_DIR}/|    |"
echo ""
echo "  ── HANDOFF_MESSAGE.txt ──────────────────────"
cat "${HANDOFF_MSG}" | sed 's/^/  /'
echo "  ─────────────────────────────────────────────"
echo ""
