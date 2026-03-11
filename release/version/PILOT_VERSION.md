# PostCAD Pilot Version Marker

---

## Pilot label

```
pilot-local-v1
```

---

## What this label refers to

The frozen local pilot package committed to the `main` branch of this repository, consisting of:

- The PostCAD routing and verification service (`crates/service/`)
- The complete operator script surface (`release/`)
- The evidence bundle infrastructure (`release/generate_evidence_bundle.sh`, `release/evidence/`)
- The review, acceptance, handoff, walkthrough, selfcheck, freeze, review-trace, and readiness bundles
- The canonical pilot fixtures (`examples/pilot/`)
- The protocol test vectors (`tests/protocol_vectors/`)

This label identifies the current state of the package at the time of the pilot review. It does not imply a software release version, a production milestone, or any regulatory or certification status.

---

## Current commit verification

To confirm you are on the reviewed pilot state:

```bash
git branch --show-current    # expected: main
git log --oneline -1         # review current HEAD commit
git status                   # expected: nothing to commit, working tree clean
cargo test --workspace       # all suites must pass, 0 failures
./release/selfcheck/run_release_selfcheck.sh  # expected: Missing items: 0
```

---

## What this label does NOT imply

- No software version increment (the protocol version remains `postcad-v1`)
- No semver versioning (this is a local pilot label, not a release tag)
- No production readiness
- No regulatory approval
- No claim beyond the local pilot scope defined in `release/freeze/FROZEN_BOUNDARIES.md`
- No guarantee that a git tag with this name exists unless one is explicitly created

---

## Optional git tag command (documentation only — not run by any script)

To anchor the current HEAD as the named pilot state, an operator may run:

```bash
git tag -a pilot-local-v1 $(git rev-parse HEAD) -m "PostCAD local pilot v1"
```

This command is provided as documentation. No script in this repository creates this tag automatically. Run it only when explicitly instructed and after confirming the commit is the correct reviewed state.

To verify a tag after creation:

```bash
git tag -l pilot-local-v1
git show pilot-local-v1 --stat
```
