#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "======================================"
echo "POSTCAD BATCHED CAMPAIGN RUNNER"
echo "======================================"
echo
echo "Repo: $ROOT"
echo
echo "Campaign file:"
echo "--------------------------------------"
cat ops/current_campaign.md
echo
echo "--------------------------------------"
echo

PROMPT="$(printf '%s\n\n%s\n' \
    "$(cat ops/current_campaign.md)" \
    "Execute the full campaign above in this repo. Do not stop after one subtask. Run the defined test command before finishing.")"

if [ -t 0 ] && [ -t 1 ]; then
    # Interactive: print context then launch claude normally
    echo "Launch Claude now in this repo and execute the full campaign."
    echo "Do not stop after one subtask."
    echo "Run the defined test command before returning."
    echo
    claude
else
    # Non-interactive (piped/logged): feed prompt via stdin
    echo "$PROMPT" | claude --print
fi
