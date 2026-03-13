# PostCAD — Pilot Bundle Snapshot

**Protocol:** `postcad-v1` · **Kernel:** `postcad-routing-v1`

---

## Bundle purpose

Frozen reproducible snapshot of the PostCAD pilot. Contains the scripts,
fixtures, and specifications needed to reproduce the full routing and
dispatch flow end-to-end against a fixed input set.

---

## Build source

| Field | Value |
|---|---|
| Repository | `postcad` |
| Commit | `c401e73a77f7b2670b4528b04592cb74dbe12eea` |

---

## Canonical flow surfaces

| Surface | Path / Command |
|---|---|
| Preflight | `./release/pilot/preflight.sh` |
| Demo entrypoint | `./release/pilot/demo.sh` |
| Reviewer shell | `http://localhost:8080/reviewer` |
| Verify command | `POST /verify` — see `PROTOCOL_WALKTHROUGH.md` Step 7 |
| OpenAPI spec | `release/pilot/docs/openapi.yaml` |

---

## Shipped artifacts

18 files listed below (plus `BUNDLE_SNAPSHOT.md` and `manifest.sha256` generated after this step).
See `MANIFEST.txt` for full inventory with source paths.

```
AUDIT_CHECKLIST.md
INVENTORY.md
MANIFEST.txt
PROTOCOL_WALKTHROUGH.md
README.md
REVIEWER_HANDOFF.md
SEQUENCE_DIAGRAM.md
candidates.json
case.json
config.json
demo.sh
derived_policy.json
docs/openapi.yaml
docs/protocol_diagram.md
expected_routed.json
expected_verify.json
preflight.sh
registry_snapshot.json
```

---

## Integrity surfaces

| Surface | Path |
|---|---|
| Human-readable manifest | `release/pilot/MANIFEST.txt` |
| SHA-256 hashes | `release/pilot/manifest.sha256` |
| Integrity check script | `./scripts/check-pilot-bundle.sh` |

---

## Reviewer guidance

| Document | Purpose |
|---|---|
| `REVIEWER_HANDOFF.md` | Structured review path: 5-minute and 20-minute tracks |
| `PROTOCOL_WALKTHROUGH.md` | Step-by-step execution flow with artifact references |
| `SEQUENCE_DIAGRAM.md` | One-glance execution order across all five actors |
| `INVENTORY.md` | Maps every bundle file to its purpose and flow step |
