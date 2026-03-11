# PostCAD Pilot Version Bundle

Entry point for pilot version information and release anchoring.

---

## What this bundle is for

Records the pilot label for the current frozen local package, provides verification instructions for the checked-out commit, and documents the optional git tag command for anchoring this state externally. It does not change any system behavior.

---

## Who should use it

- A reviewer who wants to confirm which named state they are evaluating
- An operator who wants to anchor the current commit as the reviewed pilot state
- Anyone who needs to refer to this pilot package by a stable label

---

## Relation to other surfaces

| Surface | Relation |
|---|---|
| `release/RELEASE_NOTES_PILOT.md` | Concise release notes listing all included surfaces |
| `release/FREEZE_MANIFEST.md` | Classified surface listing with frozen protocol values |
| `release/readiness/READINESS_SNAPSHOT.md` | Current-state summary with recommended review path |
| `release/freeze/FROZEN_BOUNDARIES.md` | Complete frozen scope definition |

---

## How to verify the current checked-out commit

```bash
git branch --show-current    # expected: main
git log --oneline -1         # review current HEAD
cargo test --workspace       # all suites must pass
./release/selfcheck/run_release_selfcheck.sh  # Missing items: 0
```

---

## Command

```bash
./release/version/print_pilot_version.sh
```
