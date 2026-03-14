# Reviewer Shell — Operator Guide

The reviewer shell (`GET /reviewer`) is the human review surface on top of the PostCAD routing kernel.
It exercises the full operator path against real protocol endpoints with no mocked decisions.

## Golden path

```
Open reviewer → Run route → Inspect receipt → Verify replay → Dispatch
```

**One-glance summary:** Run route → Inspect artifacts → Verify replay → Dispatch after verification succeeds.

When routing via the CLI first, substitute step 2 with `./examples/pilot/run_pilot.sh`
(writes `examples/pilot/receipt.json`); then open the reviewer for steps 3–5.

## Artifacts

Four artifacts are produced or confirmed in the pilot flow:

| Artifact | Location | Purpose |
|---|---|---|
| **Receipt** | `examples/pilot/receipt.json` · reviewer "Receipt JSON" section | Routing decision audit record. **Inspect this first** before verifying or dispatching. |
| **Receipt Hash** | `receipt.receipt_hash` field | Cryptographic commitment to all receipt fields. **Verification source of truth** — the verifier recomputes this hash from the original inputs. |
| **Verification result** | Reviewer "Verification result" section · `POST /verify` | Confirms the receipt hash is authentic. Required before dispatch. |
| **Dispatch packet** | `examples/pilot/export_packet.json` · reviewer "Export packet" section | Manufacturer handoff commitment bound to the receipt hash. Irreversible once approved. |

## CLI companion scripts

Two helper scripts exist for the CLI path (no service required):

| Script | What it does |
|---|---|
| `./examples/pilot/run_pilot.sh` | Route the pilot case + self-verify. Writes `examples/pilot/receipt.json`. |
| `./examples/pilot/verify.sh` | Replay-verify the receipt against the original inputs. No stored state trusted. |

The reviewer shell calls the same kernel and verifier over HTTP (`POST /pilot/route-normalized`,
`POST /verify`). Use the CLI scripts for headless CI or independent verification; use this page
for human review and dispatch.

## Panel structure

The results panel is divided into four readable sections:

| Section | Purpose |
|---|---|
| **Quick path / Workflow status** | Always-visible cheat sheet + four-column state block |
| **Routing decision** | Inspect generated audit artifacts. Verify before dispatching. |
| **Verify before dispatch** | Run replay verification — kernel re-derives the receipt from original inputs |
| **Dispatch commitment** | Dispatch after verification succeeds — irreversible once approved |

Each section carries a short subtitle so a first-time viewer can scan the screen without reading the full docs.

## Reviewer shell — 5 steps

### Step 1 — Open reviewer

Navigate to `/reviewer`. Pilot fixtures (`examples/pilot/`) load automatically. A compact
**Quick path** strip and the workflow status block at the top of the results panel show all four
stages as **not-run**.

If fixtures fail to load, the page shows an explicit "cannot review" state and all dispatch
actions remain blocked. Start the service from the repo root:

```bash
cargo run -p postcad-service
```

### Step 2 — Run route

Fill in the four normalized pilot fields (or click **⊕ Load sample** for the canonical values)
and click **▶ Submit for Review**.

The kernel evaluates eligibility against the manufacturer registry and routing policy. On success:

- Workflow status: **Routing → available**, **Receipt → available**, **Dispatch → available**
- The receipt JSON and artifact summary appear on the right.
- A guidance note appears: _"Verification pending. Run verify before dispatch."_

If routing fails, **Routing → failed** and **Receipt → missing** are shown with an inline error
and operator-readable hint.

### Step 3 — Inspect receipt

Review the receipt artifact summary:

| Field | What to check |
|---|---|
| Outcome | Must be `routed` (green pill) |
| Selected Manufacturer | The candidate chosen by the kernel |
| Receipt Hash | **Verification source of truth.** Deterministic — same inputs always produce the same hash. Copy it for reference before verifying. |
| Kernel Version | Records the exact kernel version used |

The full receipt JSON is shown below the summary. This is the artifact to inspect before proceeding. Use the **Copy artifact** button below the receipt JSON to copy it to the clipboard for use in external tools or documentation.

If the routing decision looks wrong, stop here. Do not proceed to verification or dispatch.

### Step 4 — Verify replay

Click **↩ Replay Verification**. The kernel re-derives every hash field in the receipt from the
original inputs — no stored state is trusted.

On success:

- Workflow status: **Verification → verified**
- The guidance note "Verification pending" clears.
- Banner: `✓ VERIFIED — receipt replay matched`

On failure:

- Workflow status: **Verification → failed**
- A dispatch-blocked note appears: _"Dispatch blocked until verification succeeds."_
- Do not dispatch until the root cause is resolved.

The **⚠ Tamper + Verify** button demonstrates tamper detection — it intentionally mutates the
receipt client-side before submitting to `/verify`. This always fails and does not affect the
verification state indicator.

### Step 5 — Dispatch

After verifying the receipt in Step 4, create the dispatch commitment. The server independently
re-verifies the receipt at dispatch creation — this is a protocol guarantee, not just a UI check.
Do not skip Step 4: if verification fails in the browser, the server will also reject the dispatch.

1. Click **⬦ Create Dispatch** — the server binds the dispatch record to the receipt hash.
2. Review the dispatch ID and status (`draft`).
3. Click **✓ Approve Dispatch** — the commitment becomes immutable.
4. Click **↓ Export Dispatch Packet** — the deterministic export record is shown.

**Dispatch is irreversible once approved.** If at any point the evidence is insufficient or the
jurisdiction fit is unclear, stop and do not proceed.

The **Dispatch readiness** panel immediately above the Create Dispatch button shows the current
decision state — see the next section for details. Once the export packet is produced, the panel
shows **Dispatch completed** and no further action is required for the current run.

## Dispatch readiness panel

A compact **Dispatch readiness** panel appears above the Create Dispatch button. It shows one of
three states derived from existing verification signals — no new backend states are introduced:

| State | Meaning |
|---|---|
| `Not ready for dispatch` | Verification has not run, is pending, or failed |
| `Ready for dispatch` | Verification succeeded — dispatch commitment can be created |
| `Dispatch completed` | Export packet produced — current run is complete |

Blocking reason shown when not ready:

- _"Required artifact not yet generated."_ — routing has not run yet
- _"Verification pending. Run verify before dispatch."_ — routing done, verify not yet run
- _"Verification failed. Resolve before dispatching."_ — verification failed

A pre-dispatch checklist shows three visual indicators: **Receipt reviewed** · **Verification
succeeded** · **Dispatch action confirmed**. These are presentational only and do not enforce
workflow behavior.

## Workflow status block

A four-column status block is always visible at the top of the results panel:

| Column | Possible states |
|---|---|
| Routing | `not-run` · `available` · `failed` |
| Receipt | `not-run` · `available` · `missing` |
| Verification | `not-run` · `verified` · `failed` |
| Dispatch | `not-run` · `available` · `failed` |

States derive from real kernel responses only — no synthetic signals.

## Integrity badges

Each artifact panel (routing decision, receipt JSON, verification result, export packet) displays a small integrity badge derived from the current verification state:

| Badge | Meaning |
|---|---|
| `UNVERIFIED` | Artifact exists but verification has not been run (neutral) |
| `VERIFIED` | Verification passed — receipt replay matched (green) |
| `FAILED` | Verification failed — do not dispatch (red) |

Badges update automatically as the operator progresses through the workflow. The dispatch export panel always shows `VERIFIED` because the server independently re-verifies the receipt before creating the dispatch record. Artifacts can be copied to the clipboard using the **Copy artifact** button below each panel.

## Artifact missing messages

- If a receipt has not been generated: _"Artifact not yet generated. Run route to create."_
- If verification has not run after routing: _"Verification pending. Run verify before dispatch."_
- If verification failed: _"Dispatch blocked until verification succeeds."_

## Endpoints exercised

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/pilot-fixtures` | Load pilot case, registry, config |
| `POST` | `/pilot/route-normalized` | Run routing kernel |
| `POST` | `/verify` | Replay verification |
| `POST` | `/dispatch/create` | Create dispatch commitment |
| `POST` | `/dispatch/:id/approve` | Approve (irreversible) |
| `GET` | `/dispatch/:id/export` | Export dispatch packet |

## What the reviewer does not do

- It does not make routing decisions.
- It does not modify the receipt or any kernel output.
- It does not bypass compliance or regulatory checks.
- It does not dispatch without a receipt (all dispatch actions are blocked in that state).
