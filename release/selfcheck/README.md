# PostCAD Pilot Release Self-Check

A read-only structural check of the local pilot release package.

---

## What it is for

Confirms that the expected release surfaces are present and that key paths referenced in the release docs resolve to real files. Intended as a quick first check before running the pilot or handing off to a reviewer.

This is a structural file-presence check only. It does not run the service, verify routing behavior, or make acceptance decisions.

---

## Who should use it

- An operator who wants to confirm the release package is intact before starting
- A reviewer who wants to verify nothing is missing before inspecting artifacts
- Anyone who received this repo and needs a fast orientation on what is present

---

## Command

Run from the repo root or from `release/selfcheck/`:

```bash
./release/selfcheck/run_release_selfcheck.sh
```

---

## What it checks (at a high level)

- Git state: branch, HEAD, clean/dirty status
- Operator scripts: present and executable
- Release docs: present
- Each release surface (walkthrough, evidence, review, acceptance, handoff, selfcheck): all expected files present
- Cross-references: a sample of paths cited in `release/INDEX.md` resolve to real files
- Evidence current folder: present or noted as missing (not an error — it is gitignored)

---

## What it does NOT check

- Protocol correctness
- Routing or kernel behavior
- Endpoint responses
- Whether the service is currently running
- Semantic validity of any JSON artifact
- Acceptance criteria
- Compliance or certification of any kind

See `SELFCHECK_SCOPE.md` for the complete scope definition.

---

## Related

| Resource | Purpose |
|---|---|
| `release/INDEX.md` | Top-level reference table for the whole release/ tree |
| `release/print_release_index.sh` | Top-level release index with surface listing |
| `release/acceptance/print_acceptance_summary.sh` | Structural pre-check for acceptance inputs |
| `release/handoff/print_handoff_index.sh` | Handoff resource index |
| `release/walkthrough/print_walkthrough.sh` | Walkthrough orientation |
