#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

{
  echo "======================================"
  echo "POSTCAD SNAPSHOT"
  echo "======================================"
  echo
  echo "DATE"
  date
  echo
  echo "--------------------------------------"
  echo "GIT STATUS --SHORT"
  echo "--------------------------------------"
  git status --short
  echo
  echo "--------------------------------------"
  echo "LATEST COMMIT"
  echo "--------------------------------------"
  git log --oneline -1
  echo
  echo "--------------------------------------"
  echo "CHANGED FILES"
  echo "--------------------------------------"
  git diff --name-only
  echo
  echo "--------------------------------------"
  echo "DIFF STAT"
  echo "--------------------------------------"
  git diff --stat
} > ops/current_snapshot.md

cat ops/current_snapshot.md
