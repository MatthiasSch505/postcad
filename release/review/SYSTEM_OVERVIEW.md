# PostCAD — System Overview

Current local pilot system. This document describes what is actually present and running, not a roadmap.

---

## What the system does

PostCAD is a deterministic routing and verification layer for dental CAD manufacturing decisions.

Given a case (material, procedure, jurisdiction) and a set of candidate manufacturers, it:

1. Evaluates regulatory compliance rules by destination country
2. Selects a manufacturer from the eligible set using a deterministic kernel
3. Issues a routing receipt binding the decision to its inputs via SHA-256 commitments
4. Records the decision in an append-only audit chain
5. Allows any party to independently verify the receipt without the routing engine

Every decision carries a machine-readable reason code. No AI, no randomness, no timestamps in routing logic.

---

## Layers

### 1. Kernel / Protocol (`postcad-core`, `postcad-compliance`, `postcad-routing`, `postcad-audit`)

- Stateless compliance rules: EU MDR, FDA 510k, MHLW, ISO 13485
- Deterministic routing selector (HighestPriority / DeterministicHash)
- Receipt assembly: all fields bound by SHA-256 commitments; `receipt_hash` computed last
- Append-only audit chain: each entry contains the previous entry's hash
- Protocol version: `postcad-v1`, routing kernel: `postcad-routing-v1`

### 2. Service layer (`postcad-service`)

An HTTP wrapper around the kernel. Exposes a local API on `localhost:8080`. Handles:

- Case storage (`POST /cases`, `GET /cases/:id`)
- Routing (`POST /cases/:id/route`)
- Receipt retrieval (`GET /receipts/:hash`)
- Dispatch (`POST /dispatch/:hash`)
- Dispatch verification (`POST /dispatch/:hash/verify`)
- Route history (`GET /routes`)
- Health (`GET /health`)
- Embedded single-page operator UI (`GET /`)

Runtime data is written to `data/` relative to the working directory. All five subdirectories (`cases/`, `receipts/`, `policies/`, `dispatch/`, `verification/`) are created lazily.

### 3. Release / Operator layer (`release/`)

Scripts for local operation:

| Script | Purpose |
|---|---|
| `release/reset_pilot_data.sh` | Removes all runtime data; leaves source and fixtures untouched |
| `release/start_pilot.sh` | Builds if needed, starts the service in the foreground |
| `release/smoke_test.sh` | Runs the 7-step deterministic flow; exits 0 on success |
| `release/generate_evidence_bundle.sh` | Captures the flow as an inspectable folder |
| `demo/run_demo.sh` | Self-contained 8-step demo; starts and stops the service itself |

### 4. Evidence / Review layer (`release/evidence/`, `release/review/`)

- `release/evidence/current/` — output folder from the last evidence capture run
- `release/review/` — this review packet

---

## Current pilot scope

- Local machine only
- Single operator, single case (`case_id: f1000001-0000-0000-0000-000000000001`)
- Single jurisdiction (DE), single material (zirconia), single procedure (crown)
- One candidate registry (`examples/pilot/registry_snapshot.json`)
- Expected routing outcome: `selected_candidate_id = pilot-de-001`
- Expected verification outcome: `result = VERIFIED`

---

## What the pilot demonstrates

- The full case-intake → routing → receipt → dispatch → verification path works end-to-end
- The receipt is deterministic: same inputs always produce `receipt_hash = 0db54077cff0fbc4…`
- The receipt passes independent verification via `POST /dispatch/:hash/verify`
- Route history is persisted and retrievable
- The operator can reset, re-run, and re-verify without any state leaking across runs
