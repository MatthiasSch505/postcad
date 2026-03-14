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
echo "Launch Claude now in this repo and execute the full campaign."
echo "Do not stop after one subtask."
echo "Run the defined test command before returning."
echo
claude
