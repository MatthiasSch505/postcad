#!/usr/bin/env bash
# PostCAD — Environment Preflight Check
#
# Checks that the local environment has everything needed to run the pilot demo.
# No network calls. No installs. No side effects. Just checks.
#
# Usage:
#   ./examples/pilot/preflight.sh
#
# Exit: 0 = ready, 1 = one or more checks failed.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

FAIL=0

ok()   { printf "  [OK]      %s\n" "$*"; }
miss() { printf "  [MISSING] %s\n" "$*" >&2; FAIL=$((FAIL + 1)); }

printf "\nPostCAD — Environment Preflight\n"
printf "================================\n"

# ── Required tools ────────────────────────────────────────────────────────────

printf "\nRequired tools:\n"

if command -v cargo > /dev/null 2>&1; then
    ok "cargo  ($(cargo --version 2>/dev/null))"
else
    miss "cargo — not found. Install Rust at https://rustup.rs"
fi

if command -v curl > /dev/null 2>&1; then
    ok "curl"
else
    miss "curl — not found. Install: apt install curl  or  brew install curl"
fi

if command -v python3 > /dev/null 2>&1; then
    ok "python3  ($(python3 --version 2>/dev/null))"
else
    miss "python3 — not found. Install Python 3 at https://python.org"
fi

# ── Required fixture files ────────────────────────────────────────────────────

printf "\nPilot fixtures:\n"

for f in \
    "examples/pilot/case.json" \
    "examples/pilot/registry_snapshot.json" \
    "examples/pilot/config.json" \
    "examples/pilot/demo.sh"; do
    if [[ -f "${REPO_ROOT}/${f}" ]]; then
        ok "${f}"
    else
        miss "${f} — file not found"
    fi
done

# ── Output directory writable ────────────────────────────────────────────────

printf "\nOutput location:\n"

if [[ -w "${REPO_ROOT}/examples/pilot" ]]; then
    ok "examples/pilot/  (export_packet.json will be written here)"
else
    miss "examples/pilot/ — directory not writable"
fi

# ── Summary ───────────────────────────────────────────────────────────────────

printf "\n"
if [[ "$FAIL" -eq 0 ]]; then
    printf "  Ready to run.\n"
    printf "\n"
    printf "  Start the service:\n"
    printf "    cargo run -p postcad-service\n"
    printf "\n"
    printf "  Run the demo:\n"
    printf "    ./examples/pilot/demo.sh\n"
    printf "\n"
    exit 0
else
    printf "  %d check(s) failed — fix the issues above before running the demo.\n\n" "$FAIL" >&2
    exit 1
fi
