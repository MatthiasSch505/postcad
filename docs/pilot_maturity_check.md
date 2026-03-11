# PostCAD Pilot Maturity Check

**Assessed:** 2026-03-11
**Protocol version:** `postcad-v1`
**Routing kernel:** `postcad-routing-v1`

---

## 1. Purpose

This document is a deterministic readiness check for the current PostCAD pilot
stack. It is intended for use before external demo, operator handoff, or
hire/fundraise trigger evaluation. It records what is implemented, what
conditions are already satisfied, what gaps remain, and what is intentionally
out of scope.

---

## 2. Current Implemented Surfaces

| Surface | Location | Status |
|---------|----------|--------|
| Routing kernel | `crates/routing/` | Locked, deterministic |
| Receipt / refusal artifact schema | `crates/cli/src/receipt.rs` | Schema v1, frozen |
| Verification path | `crates/cli/src/lib.rs` (`verify_receipt_from_policy_json`) | Replay-based, self-contained checks + full replay |
| Registry export | `crates/cli/src/registry_export.rs` | ManufacturerRecord → RoutingPolicyBundle |
| HTTP service | `crates/service/` | axum 0.7, 16+ endpoints |
| Case intake | `POST /cases`, `GET /cases`, `GET /cases/:id` | Filesystem-backed, idempotent |
| Stored-case routing | `POST /cases/:id/route` | Persists receipt + derived policy |
| Receipt retrieval | `GET /receipts`, `GET /receipts/:hash` | Sorted, content-addressed |
| Route history | `GET /routes` | Timestamp-sorted, deterministic tiebreaker |
| Dispatch | `POST /dispatch/:hash` | Idempotent guard, filesystem record |
| Dispatch verification gate | `POST /dispatch/:hash/verify` | Reuses `verify_receipt_from_policy_json`, stores result |
| Operator UI | `GET /` | Embedded single-page app, no external deps |
| Protocol manifest | `GET /protocol-manifest` | Stable schema + version identifiers |
| Acceptance runner | `scripts/pilot_acceptance.sh`, `make acceptance` | Automated, exit-coded |
| Protocol conformance vectors | `tests/protocol_vectors/v01–v05` | 5 scenarios, self-seeding |
| Local deployment | `Dockerfile`, `docker-compose.yml`, `docs/local_service_run.md` | Docker or bare binary |
| Handoff docs | `docs/operator_handoff.md`, `docs/development_bundle.md` | Operator-grade |
| Pilot one-pager | `pilot/PILOT_ONE_PAGER.md` | Partner-facing brief |

---

## 3. Pass Conditions Already Satisfied

- All 760+ tests pass (`cargo test --workspace`).
- Receipt hashes are deterministic: same inputs → same hash, every run, every machine.
- Verification replay is self-contained: no external state required.
- Protocol manifest is frozen and tested against `fixtures/expected_manifest.json`.
- Five protocol conformance vectors lock the routing outcomes for five distinct scenarios.
- Docker deployment boots to `/health → {"status":"ok"}` without source modifications.
- One-command acceptance runner (`make acceptance`) exits 0 on a clean repo.
- Operator UI serves the complete 5-step local workflow without curl.
- Dispatch and dispatch verification are tested with deterministic in-process tests.
- All stable error codes are enumerated in the protocol manifest.

---

## 4. Remaining Gaps

### Usability

- **Operator UI route form requires manual registry JSON paste.** No sample data is
  pre-filled when opening the route modal for a stored case. An external operator
  unfamiliar with the registry schema will be blocked at this step.
- **No inline outcome summary in receipt viewer.** The receipt viewer displays raw
  JSON; a one-line summary ("Routed to: mfr-de-001 · VERIFIED") would reduce
  inspection time.
- **Dispatch has no confirmation step.** A mis-click dispatches immediately with no
  undo. For a supervised pilot this is low risk, but operators should be aware.

### Operational Packaging

- **No `HEALTHCHECK` directive in Dockerfile.** `docker ps` will always show
  "healthy"; actual liveness depends on the operator polling `/health` manually.
- **`data/` directories are created lazily but not documented.** First-write
  creates them, but an operator checking disk layout before first use will see
  no `data/` tree.
- **Acceptance runner covers only the original `/route` + `/verify` endpoints.**
  The newer 4-step flow (intake → stored-case route → dispatch → verify) is not
  exercised by `scripts/pilot_acceptance.sh`.
- **No pre-built binary or Docker image published.** Partners must build from
  source or run `docker build` locally.

### Fixture / Demo Quality

- **`handoff_status.json` acceptance_status is `"pending"`** — never updated after
  the flow was validated. Stale JSON looks broken to an external reviewer.
- **`pilot_acceptance.json` does not reflect dispatch/verification endpoints.**
  Added after the original acceptance spec was written; the JSON is incomplete.
- **No single worked example of the complete 4-step flow** (case store → route →
  dispatch → dispatch verify) as a curl sequence or operator walkthrough.
- **Pilot fixture manufacturer is synthetic (`pilot-de-001`).** A representative
  real-world entry would make the demo more convincing to lab or clinic partners.

### External Handoff Clarity

- **Operator handoff doc (`docs/operator_handoff.md`) describes only the original
  `/route` + `/verify` flow**, not the newer stored-case dispatch flow.
- **No single entry-point README section** that answers "what do I run first" for
  each role (developer, operator, external partner).

### Non-invasive Observability

- **No request/response logging.** Even a single `eprintln!` per request would let
  operators watch the live traffic without a log aggregator.
- **Verification results written to `data/verification/` but not exposed** via any
  `GET /verification/:hash` endpoint; operators must read files directly.
- **No count endpoints.** There is no `GET /status/counts` or similar; the only
  way to see storage state is to call each list endpoint individually.

---

## 5. Explicit Non-Gaps

The following are intentionally out of scope for this pilot phase and do not
constitute gaps:

- **Authentication / authorisation** — not required for a supervised local pilot.
- **Cloud infrastructure** — filesystem-backed storage is correct for local pilot.
- **Database** — routing decisions are stateless; filesystem is sufficient.
- **Billing or metering** — not a pilot deliverable.
- **Analytics platform or dashboard** — operator UI and route history are sufficient.
- **Routing heuristics expansion** — `HighestPriority` + `DeterministicHash` cover
  pilot needs; heuristic changes are a post-pilot decision.
- **Protocol redesign** — `postcad-v1` receipt schema is frozen and correct.
- **AI/ML-based selection** — explicitly out of scope by design; all decisions are
  rule-based with stable reason codes.

---

## 6. Readiness Verdict

**externally demoable**

The routing kernel, verification path, service, operator UI, Docker deployment,
and acceptance runner are all functional and tested. A skilled operator can
give a complete live demo using the frozen fixtures and the operator UI.
However, an external pilot partner operating independently would encounter
friction at the registry JSON input step, find the acceptance spec stale, and
lack a walkthrough of the full 4-step dispatch flow. Supervision is required
for a first external pilot run.

---

## 7. Next Locked Phase

Close the four usability + handoff gaps above, then promote verdict to
**pilot-ready with supervision**.
