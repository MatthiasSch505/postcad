#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

CAMPAIGN_FILE="${1:-ops/current_campaign.md}"

if [[ ! -f "$CAMPAIGN_FILE" ]]; then
  echo "Campaign file not found: $CAMPAIGN_FILE" >&2
  exit 1
fi

echo "======================================"
echo "POSTCAD BATCHED CAMPAIGN RUNNER"
echo "======================================"
echo
echo "Repo: $ROOT"
echo
echo "Campaign file:"
echo "--------------------------------------"
cat "$CAMPAIGN_FILE"
echo
echo "--------------------------------------"
echo

PROMPT="$(printf '%s\n\n%s\n' \
  "$(cat "$CAMPAIGN_FILE")" \
  "Execute the full campaign above in this repo. Do not stop after one subtask. Run the defined test command before finishing. Stay strictly within the allowed files and constraints from the campaign. If the task cannot be completed exactly within those bounds, stop and explain why in one concise final message.")"

SYSTEM_PROMPT="$(cat <<'SYS'
You are executing a bounded PostCAD lane-1 campaign inside a regulated infrastructure repository.

Non-negotiable rules:
- Do not modify kernel crates unless the campaign explicitly allows it.
- Treat crates/core, crates/routing, crates/compliance, crates/audit, and crates/registry as forbidden unless explicitly allowed.
- Respect allowed-files boundaries exactly.
- Do not widen scope.
- Do not make architecture changes.
- Do not ask for confirmation if the campaign is clear; execute it fully.
- Run the specified test command before finishing.
- Keep behavior deterministic and auditable.
SYS
)"

claude \
  --print \
  --model sonnet \
  --permission-mode acceptEdits \
  --output-format text \
  --system-prompt "$SYSTEM_PROMPT" \
  "$PROMPT"
