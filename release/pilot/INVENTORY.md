# PostCAD — Pilot Bundle Inventory

Every file shipped in this bundle, sorted by path. Runtime artifacts produced
during a run are listed separately at the bottom.

---

## Shipped files

| Path | Purpose | When the reviewer uses it | Required / Reference |
|---|---|---|---|
| `candidates.json` | Eligible candidate list extracted from `derived_policy.json` | Step 7 — independent verification via CLI `verify-receipt` | Required for CLI verify path |
| `case.json` | Canonical dental case input: crown/zirconia, jurisdiction DE, `case_id=f1000001-...` | Steps 3 and 7 — routed by `demo.sh`; replayed in verification | Required |
| `config.json` | Routing configuration: jurisdiction `DE`, policy `allow_domestic_and_cross_border` | Step 3 — loaded by `demo.sh` during the route step | Required |
| `demo.sh` | End-to-end demo script: health → route → dispatch → approve → export → verify | Step 3 — run once to execute the full canonical flow | Required |
| `derived_policy.json` | Full routing policy bundle derived from `registry_snapshot.json`; includes all snapshots | Steps 3 and 7 — passed to `POST /dispatch/create` and `POST /verify` | Required |
| `docs/openapi.yaml` | OpenAPI 3.1 spec for all pilot endpoints | Any step — inspect to understand request/response shapes | Reference only |
| `docs/protocol_diagram.md` | One-page protocol flow and guarantee summary | Before starting — read to understand what the demo proves | Reference only |
| `expected_routed.json` | Frozen canonical routing receipt for this input set | Step 7 — compare `receipt_hash` against demo output to confirm determinism | Reference only |
| `expected_verify.json` | Frozen canonical verification result for this input set | Step 7 — confirms expected verify response is `{"result":"VERIFIED"}` | Reference only |
| `BUNDLE_SNAPSHOT.md` | Deterministic surface summary: build source, canonical flow surfaces, shipped artifact list, integrity and reviewer guidance pointers | Audit — one-file record of the exact bundle state at build time | Reference only |
| `INVENTORY.md` | This file — maps every bundle file to its purpose and flow step | Before starting — consult if unsure what a file does | Reference only |
| `MANIFEST.txt` | Human-readable file inventory with source paths from the repo | Audit — trace any bundle file back to its source location | Reference only |
| `PROTOCOL_WALKTHROUGH.md` | Step-by-step technical explanation of the execution flow using actual pilot artifacts | Before starting — read to understand what each step does and why | Reference only |
| `SEQUENCE_DIAGRAM.md` | ASCII sequence diagram showing the full pilot flow across all five actors | Before starting — one-glance execution order overview | Reference only |
| `manifest.sha256` | SHA-256 hashes of all bundle files (sha256sum -c compatible) | Before starting — consumed by `check-pilot-bundle.sh` | Required |
| `preflight.sh` | Environment check: verifies required tools and fixture files are present | Step 1 — run before anything else | Required |
| `README.md` | Step-by-step operator guide for running the canonical pilot flow | Before starting — primary entry point | Required |
| `REVIEWER_HANDOFF.md` | Structured handoff for external reviewers: fastest review path, files that matter most, determinism and integrity checks | First file to read if you are an external reviewer | Reference only |
| `registry_snapshot.json` | Manufacturer registry snapshot: three active manufacturers in Germany, all capable of zirconia crowns | Step 3 — loaded by `demo.sh` during the route step | Required |

---

## Runtime artifacts

These files are produced during a run and are not part of the frozen bundle.

| Path | Produced by | Purpose |
|---|---|---|
| `export_packet.json` | `demo.sh` step 4 (export) | Canonical handoff packet written at runtime; not committed |
