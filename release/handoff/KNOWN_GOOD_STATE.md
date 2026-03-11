# PostCAD Pilot — Known Good State

Description of what a known-good local pilot state looks like and how to verify you are in it. This file describes the state by what to check, not by hardcoding values that will age badly.

---

## How to verify the current commit

```bash
git log --oneline -1          # show the current HEAD commit and message
git status                    # confirm working tree is clean
git diff                      # confirm no unstaged changes
git diff --cached             # confirm no staged changes
```

A known-good state requires:
- Working tree is clean (`nothing to commit`)
- No untracked files under `crates/`, `release/review/`, `release/acceptance/`, `examples/`, `tests/`
- `examples/pilot/` fixtures are at their committed state (no local modifications)

---

## Expected release bundle structure

The following directories and files must be present (verify with `ls`):

```
release/
  README.md
  start_pilot.sh              (executable)
  reset_pilot_data.sh         (executable)
  smoke_test.sh               (executable)
  generate_evidence_bundle.sh (executable)
  evidence/README.md
  review/README.md
  review/SYSTEM_OVERVIEW.md
  review/OPERATOR_FLOW.md
  review/ARTIFACT_GUIDE.md
  review/BOUNDARIES.md
  acceptance/README.md
  acceptance/PILOT_ACCEPTANCE_CHECKLIST.md
  acceptance/REVIEW_WORKSHEET.md
  acceptance/print_acceptance_summary.sh
  handoff/README.md
  handoff/HANDOFF_CHECKLIST.md
  handoff/FIRST_HOUR_GUIDE.md
  handoff/KNOWN_GOOD_STATE.md

demo/
  run_demo.sh                 (executable)

examples/pilot/
  case.json
  registry_snapshot.json
  config.json
```

Quick structural check:
```bash
./release/acceptance/print_acceptance_summary.sh
```
All lines should show `[OK]`.

---

## Expected test suite state

```bash
cargo test --workspace
```

All test suites pass with 0 failures. This does not require a running service.

---

## Expected evidence bundle state

`release/evidence/current/` is gitignored (runtime output). It either:

1. Exists from a previous run — check with:
   ```bash
   cat release/evidence/current/summary.txt
   ```
   Expected last line: `All 7 steps passed.`

2. Does not exist yet — generate it by running the local pilot flow:
   ```bash
   ./release/start_pilot.sh          # Terminal A
   ./release/smoke_test.sh           # Terminal B
   ./release/generate_evidence_bundle.sh  # Terminal B
   ```

In either case, when the evidence bundle is present and the run was successful:
- `04_receipt.json` contains `"outcome": "routed"` and `"selected_candidate_id": "pilot-de-001"`
- `06_verify.json` contains `"result": "VERIFIED"`
- The `receipt_hash` value is deterministic for the canonical pilot inputs

To confirm the `receipt_hash` matches what is expected:
```bash
python3 -c "
import json
d = json.load(open('release/evidence/current/04_receipt.json'))
print(d['receipt_hash'])
"
```
Compare against the value documented in `release/review/ARTIFACT_GUIDE.md`.

---

## What is frozen

The following are not expected to change in a known-good state:

| What | How to check |
|---|---|
| `examples/pilot/` fixtures | `git diff examples/pilot/` — must be empty |
| `tests/protocol_vectors/` | `git diff tests/protocol_vectors/` — must be empty |
| Protocol version | `"postcad-v1"` in service responses |
| Routing kernel version | `"postcad-routing-v1"` in receipt `04_receipt.json` |
| Receipt schema version | `"1"` in receipt `04_receipt.json` |
| Deterministic `receipt_hash` | Matches value in `release/review/ARTIFACT_GUIDE.md` |

---

## What is NOT expected to be stable across runs

The following change between runs and do not indicate a problem:

- `release/evidence/current/` — regenerated each time; not committed
- `data/` directories — runtime state; cleared by `reset_pilot_data.sh`
- `release/evidence/current/git_head.txt` — reflects the HEAD at time of generation
- Timestamps in `07_route_history.json` route entries (service-side timestamps are not part of the protocol)

---

## Summary: known-good local pilot state

| Condition | Check |
|---|---|
| Working tree clean | `git status` |
| All tests pass | `cargo test --workspace` |
| Canonical fixtures unchanged | `git diff examples/pilot/` |
| Release scripts present and executable | `ls -l release/*.sh` |
| Review and acceptance packets present | `ls release/review/ release/acceptance/` |
| Evidence bundle passes (or can be generated) | `cat release/evidence/current/summary.txt` |
| Acceptance pre-check clean | `./release/acceptance/print_acceptance_summary.sh` |
