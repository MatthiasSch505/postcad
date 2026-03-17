#!/usr/bin/env bash
# ops/night_tail.sh — tail ops/queue_status.log for night-mode monitoring
# Usage: bash ops/night_tail.sh [--lines N]
#
# Read-only. No writes. No network calls.
# Refuses to run outside the PostCAD repository.

set -euo pipefail

# ── Locate canonical repo root ─────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Refuse if not inside a git repo
if ! git -C "${REPO_ROOT}" rev-parse --git-dir > /dev/null 2>&1; then
    echo "ERROR: ${REPO_ROOT} is not a git repository." >&2
    echo "Run this script from within the PostCAD repo." >&2
    exit 1
fi

# Refuse if not the PostCAD repo (check for a known marker)
if [[ ! -f "${REPO_ROOT}/CLAUDE.md" ]] || ! grep -q "Post-CAD Layer" "${REPO_ROOT}/CLAUDE.md" 2>/dev/null; then
    echo "ERROR: ${REPO_ROOT} does not appear to be the PostCAD repository." >&2
    echo "Expected CLAUDE.md with 'Post-CAD Layer' marker." >&2
    exit 1
fi

# ── Defaults ───────────────────────────────────────────────────────────────────

LOG_FILE="${REPO_ROOT}/ops/queue_status.log"
LINES=20

# ── Argument parsing ───────────────────────────────────────────────────────────

usage() {
    echo "Usage: $0 [--lines N]" >&2
    echo "  --lines N   Number of log lines to show (default: ${LINES})" >&2
    exit 1
}

while [[ $# -gt 0 ]]; do
    case "${1:-}" in
        --lines)
            shift
            if [[ $# -eq 0 || ! "${1:-}" =~ ^[0-9]+$ ]]; then
                echo "error: --lines requires a positive integer" >&2
                usage
            fi
            LINES="${1}"
            shift
            ;;
        *)
            echo "error: unknown argument: ${1}" >&2
            usage
            ;;
    esac
done

# ── Main ───────────────────────────────────────────────────────────────────────

echo "# night_tail — last ${LINES} lines of ops/queue_status.log"
echo "# Full logs: ${REPO_ROOT}/ops/"
echo ""

if [[ ! -f "${LOG_FILE}" ]]; then
    echo "ERROR: log file not found: ${LOG_FILE}" >&2
    exit 1
fi

tail -n "${LINES}" "${LOG_FILE}"
