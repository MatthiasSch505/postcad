#!/usr/bin/env bash
# PostCAD pilot walkthrough — print orientation summary.
#
# Usage (from repo root or release/ directory):
#   ./release/walkthrough/print_walkthrough.sh
#
# Read-only. Does not start the service, run the smoke test,
# generate evidence, or modify any file.
# Prints: HEAD, git status, key file presence, and the exact
# local pilot sequence with commands and output locations.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

ok()      { echo "  [OK]  $*"; }
missing() { echo "  [--]  MISSING: $*"; }
hr()      { echo ""; echo "── $* ──────────────────────────────────────────────"; }

# ── header ────────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════"
echo "  PostCAD Pilot — Walkthrough"
echo "══════════════════════════════════════════"
echo "  Repo root : $REPO_ROOT"
echo ""

# ── git state ─────────────────────────────────────────────────────────────────

hr "Git state"
branch=$(git -C "$REPO_ROOT" branch --show-current 2>/dev/null || echo "(unknown)")
head=$(git -C "$REPO_ROOT" rev-parse HEAD 2>/dev/null || echo "(unknown)")
clean=$(git -C "$REPO_ROOT" status --short 2>/dev/null || echo "?")
echo "  Branch : $branch"
echo "  HEAD   : $head"
[[ -z "$clean" ]] && echo "  Status : clean" || echo "  Status : DIRTY (run 'git status' for details)"

# ── key file presence ─────────────────────────────────────────────────────────

hr "Key files"
for f in \
  "release/start_pilot.sh" \
  "release/reset_pilot_data.sh" \
  "release/smoke_test.sh" \
  "release/generate_evidence_bundle.sh" \
  "demo/run_demo.sh" \
  "examples/pilot/case.json" \
  "examples/pilot/registry_snapshot.json" \
  "examples/pilot/config.json" \
  "release/review/README.md" \
  "release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md" \
  "release/acceptance/print_acceptance_summary.sh" \
  "release/handoff/FIRST_HOUR_GUIDE.md" \
  "release/handoff/print_handoff_index.sh" \
  "release/walkthrough/PILOT_WALKTHROUGH.md"; do
  [[ -f "$REPO_ROOT/$f" ]] && ok "$f" || missing "$f"
done

evidence="$REPO_ROOT/release/evidence/current"
if [[ -d "$evidence" ]] && [[ -f "$evidence/summary.txt" ]]; then
  grep -q "All 7 steps passed" "$evidence/summary.txt" \
    && ok "release/evidence/current/ — summary: All 7 steps passed" \
    || ok "release/evidence/current/ — present (summary check failed or incomplete)"
else
  echo "  [--]  release/evidence/current/ — not present (generate with Step 6 below)"
fi

# ── pilot sequence ────────────────────────────────────────────────────────────

hr "Local pilot sequence"
cat <<'SEQUENCE'

  Two terminals required for steps 3–6.
  Run all commands from the repo root.

  ┌─ Terminal A ───────────────────────────────────────────┐
  │                                                        │
  │  Step 3 — Start service (leave running)                │
  │    ./release/start_pilot.sh                            │
  │                                                        │
  └────────────────────────────────────────────────────────┘

  ┌─ Terminal B ───────────────────────────────────────────┐
  │                                                        │
  │  Step 1 — Inspect repo state                           │
  │    git status                                          │
  │    cargo test --workspace                              │
  │                                                        │
  │  Step 2 — Reset pilot data                             │
  │    ./release/reset_pilot_data.sh                       │
  │                                                        │
  │  Step 4 — Smoke test (service must be running)         │
  │    ./release/smoke_test.sh                             │
  │                                                        │
  │  Step 5 — Demo (self-contained, no service needed)     │
  │    ./demo/run_demo.sh                                  │
  │                                                        │
  │  Step 6 — Generate evidence (service must be running)  │
  │    ./release/generate_evidence_bundle.sh               │
  │    cat release/evidence/current/summary.txt            │
  │                                                        │
  │  Step 7 — Inspect review packet (read-only)            │
  │    cat release/review/README.md                        │
  │                                                        │
  │  Step 8 — Acceptance pre-check (read-only)             │
  │    ./release/acceptance/print_acceptance_summary.sh    │
  │                                                        │
  │  Step 9 — Handoff index (read-only)                    │
  │    ./release/handoff/print_handoff_index.sh            │
  │                                                        │
  └────────────────────────────────────────────────────────┘

SEQUENCE

# ── expected outputs ──────────────────────────────────────────────────────────

hr "Expected outputs"
cat <<'OUTPUTS'

  Step 4 smoke test final line:
    SMOKE TEST PASSED — all 7 steps OK

  Step 5 demo final line:
    DEMO COMPLETE — all 8 steps passed

  Step 6 evidence final line:
    EVIDENCE BUNDLE COMPLETE

  Step 6 summary.txt final line:
    All 7 steps passed.

  Step 6 key receipt value (deterministic):
    receipt_hash = 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb
    selected_candidate_id = pilot-de-001
    result (verify) = VERIFIED

OUTPUTS

# ── inspection locations ──────────────────────────────────────────────────────

hr "Key inspection locations"
cat <<'INSPECT'

  release/evidence/current/summary.txt       overall pass/fail
  release/evidence/current/04_receipt.json   routing receipt (all hash fields)
  release/evidence/current/06_verify.json    verification result
  release/review/ARTIFACT_GUIDE.md           field-by-field receipt guide
  release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md  33-item acceptance criteria
  release/handoff/KNOWN_GOOD_STATE.md        known-good state reference

INSPECT

# ── further reading ───────────────────────────────────────────────────────────

hr "Further reading"
cat <<'READING'

  release/walkthrough/PILOT_WALKTHROUGH.md   full step-by-step narrative
  release/README.md                          operator runbook
  release/review/SYSTEM_OVERVIEW.md          system layer description
  release/review/BOUNDARIES.md              frozen scope
  release/handoff/FIRST_HOUR_GUIDE.md        first-hour guide for new operators

READING

# ── done ──────────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════"
echo "  Walkthrough orientation complete."
echo "  Read PILOT_WALKTHROUGH.md for full detail."
echo "══════════════════════════════════════════"
echo ""
