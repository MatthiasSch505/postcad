#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

STAMP="$(date +%Y%m%d_%H%M%S)"
DIR="ops/archive/$STAMP"
mkdir -p "$DIR"

cp ops/current_campaign.md "$DIR/campaign.md" 2>/dev/null || true
cp ops/current_result.md "$DIR/result.md" 2>/dev/null || true
cp ops/current_snapshot.md "$DIR/snapshot.md" 2>/dev/null || true
cp ops/current_decision.md "$DIR/decision.md" 2>/dev/null || true

echo "Archived to $DIR"
