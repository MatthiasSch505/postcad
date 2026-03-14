# PostCAD Pilot Bundle

## Quick Start — Canonical Demo

Check the environment first, then start the service and run the demo:

```bash
./examples/pilot/preflight.sh           # check tools + fixtures (no side effects)
cargo run -p postcad-service &          # start service on :8080
./examples/pilot/demo.sh                # route → dispatch → approve → export → verify
```

`demo.sh` also runs preflight automatically at startup, so the explicit step above is optional.

Expected final output:

```
  DEMO COMPLETE
  Candidate:    pilot-de-001
  Receipt hash: 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb
  Verification: VERIFIED
```

The export packet is written to `examples/pilot/export_packet.json`.
The reviewer shell is at `http://localhost:8080/reviewer`.

---

## What PostCAD Solves

After a dental CAD design is complete, someone still has to decide which manufacturer
gets the case. Today that step is manual, opaque, and relationship-driven: a coordinator
checks a spreadsheet, picks a lab, and hopes the choice was compliant. There is no
audit trail and no way for a third party to verify the decision after the fact.

PostCAD replaces that step with deterministic, rule-based routing. Every routing
decision is cryptographically committed to a receipt. The receipt can be independently
verified — by the clinic, the manufacturer, or a regulator — without trusting the
platform.

---

## Operator Demo Flow

### Step 1 — Route

Routes the pilot case against the manufacturer registry. Self-verification runs
automatically inside the routing step; the command exits non-zero if the receipt
fails to verify.

```bash
./examples/pilot/run_pilot.sh
```

Expected output:

```
Result:               routed
Selected candidate:   pilot-de-001
Receipt hash:         0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb
Kernel version:       postcad-routing-v1

Receipt written to:   examples/pilot/receipt.json

Verification: OK
```

### Step 2 — Verify

Independently verifies the receipt produced in Step 1 against the original inputs.
No routing is re-run. The kernel replays the decision deterministically and confirms
every hash field in the receipt.

```bash
./examples/pilot/verify.sh
```

Expected output:

```
VERIFIED
```

The same inputs always produce the same receipt hash. The receipt will not verify
if the case, the registry snapshot, or the routing policy has drifted from the
values committed at routing time.

---

## Files

| File | Description |
|------|-------------|
| `case.json` | Dental case input — procedure, material, jurisdiction, routing policy |
| `registry_snapshot.json` | Three active manufacturers in Germany, all capable of zirconia crowns |
| `config.json` | Routing configuration — jurisdiction `DE`, policy `allow_domestic_and_cross_border` |
| `derived_policy.json` | Full routing policy bundle derived from `registry_snapshot.json` — required for `verify-receipt` |
| `candidates.json` | Eligible candidate list extracted from `derived_policy.json` — required for `verify-receipt` |
| `expected_routed.json` | Locked routing receipt — canonical output for these inputs |
| `expected_verify.json` | Locked verification result — `{"result":"VERIFIED"}` |
| `receipt.json` | Written by `run_pilot.sh` at runtime |

---

## Prerequisites

```bash
cargo build --bin postcad-cli
```

Binary is written to `target/debug/postcad-cli`.

---

## CLI: Route (raw)

Routes the pilot case and self-verifies the receipt in one step. Exits non-zero if
self-verification fails.

```bash
./target/debug/postcad-cli route-case-from-registry --json \
  --case     examples/pilot/case.json \
  --registry examples/pilot/registry_snapshot.json \
  --config   examples/pilot/config.json
```

Expected output matches `expected_routed.json`.

---

## CLI: Verify (raw)

```bash
./target/debug/postcad-cli verify-receipt \
  --receipt    examples/pilot/receipt.json \
  --case       examples/pilot/case.json \
  --policy     examples/pilot/derived_policy.json \
  --candidates examples/pilot/candidates.json
```

Expected output: `VERIFIED`. Expected exit code: `0`.

Note: `--policy` takes `derived_policy.json` (full bundle including `snapshots`).
`--candidates` takes `candidates.json` (flat array). They are separate files.

---

## Service Flow

Start the service:

```bash
cargo run -p postcad-service
# listening on 0.0.0.0:8080
```

Run the canonical demo (route → dispatch → approve → export → verify):

```bash
./examples/pilot/demo.sh
```

Independent verify against the locked receipt:

```bash
curl -s -X POST http://localhost:8080/verify \
  -H 'Content-Type: application/json' \
  -d "{
    \"receipt\": $(cat examples/pilot/expected_routed.json),
    \"case\":    $(cat examples/pilot/case.json),
    \"policy\":  $(cat examples/pilot/derived_policy.json)
  }"
```

Expected response: `{"result":"VERIFIED"}`.

---

## Locked Outputs

`expected_routed.json`:
- `outcome`: `"routed"`
- `selected_candidate_id`: `"pilot-de-001"`
- `receipt_hash`: `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb`

`expected_verify.json`:
```json
{"result": "VERIFIED"}
```

Running the same inputs twice produces the same `receipt_hash`. The receipt will not
change unless the inputs change.

---

## Pilot Run Bundle

A pilot run bundle is the complete, shareable artifact set produced by a single PostCAD run. It contains every artifact needed for external review or lab handoff:

| Bundle file | Contents |
|---|---|
| `route.json` | Routing result — the receipt produced by the routing kernel |
| `receipt.json` | Same receipt (canonical copy) |
| `verification.json` | Verification result confirming the receipt hash |
| `export_packet.json` | Approved dispatch packet bound to the receipt |
| `reproducibility.json` | Reproducibility check result confirming determinism |
| `bundle_manifest.json` | Generated index with artifact list and timestamp |

### Generating a bundle

Complete the full pilot run first (route → verify → export → reproducibility check), then:

```bash
./examples/pilot/package_run.sh
```

By default this reads artifacts from `examples/pilot/` and writes the bundle to `pilot_bundle/`.

Override source and output directories:

```bash
./examples/pilot/package_run.sh <source_dir> <output_dir>
```

The script is idempotent: running it twice produces the same output. It exits non-zero with a clear message if any required artifact is missing.

### Validating a bundle

```bash
./examples/pilot/validate_bundle.sh
```

Checks that all required files are present, non-empty, and valid JSON. Exits non-zero with a clear per-file error if any check fails. Accepts an optional path argument:

```bash
./examples/pilot/validate_bundle.sh <bundle_dir>
```

### Replaying a run

```bash
./examples/pilot/replay_run.sh
```

Reads `route.json`, `receipt.json`, and `verification.json` from the bundle and prints a concise human-readable run summary. Performs a cross-artifact consistency check to confirm all artifacts belong to the same run (receipt hash match, outcome/verification agreement). Exits non-zero if the check fails.

```bash
./examples/pilot/replay_run.sh <bundle_dir>
```

### Inspecting a bundle manually

```bash
cat pilot_bundle/bundle_manifest.json   # artifact index + timestamp
cat pilot_bundle/receipt.json           # routing decision + receipt hash
cat pilot_bundle/verification.json      # VERIFIED / FAILED
cat pilot_bundle/export_packet.json     # dispatch packet
cat pilot_bundle/reproducibility.json   # reproducibility result
```

Generated bundles are excluded from version control via `.gitignore`.

---

## Trial Receipt Ledger

Each pilot run accumulates an append-only lifecycle record in:

```
examples/pilot/reports/ledger_<run-id>.txt
```

The ledger is written automatically by the pilot scripts as the workflow progresses.

### Where entries are written

| Script / command | Ledger event |
|---|---|
| `run_pilot.sh` | `outbound_bundle_created` |
| `lab_simulator.sh --handoff-pack` | `handoff_pack_created` |
| `verify.sh --inbound` | `inbound_artifact_processed`, `verification_recorded`, `operator_decision_recorded` |
| `verify.sh --batch-inbound` | `inbound_artifact_processed`, `operator_decision_recorded` (per artifact) |

### Ledger entry format

Each entry is a plain-text block:

```
sequence: 001
event: outbound_bundle_created
run_id: f1000001-0000-0000-0000-000000000001
artifact: examples/pilot/receipt.json
result: recorded
timestamp: 2026-03-14T10:00:00Z
```

### Complete trial flow with ledger

```bash
# 1. Route — writes ledger entry: outbound_bundle_created
./examples/pilot/run_pilot.sh

# 2. Export bundle + generate handoff pack — writes: handoff_pack_created
./examples/pilot/package_run.sh
./examples/pilot/lab_simulator.sh --handoff-pack handoff/ --bundle pilot_bundle

# 3. Receive lab response + verify — writes: inbound_artifact_processed,
#    verification_recorded, operator_decision_recorded
./examples/pilot/verify.sh --inbound inbound/lab_response.json --bundle pilot_bundle

# 4. Inspect the complete ledger
cat examples/pilot/reports/ledger_f1000001-0000-0000-0000-000000000001.txt
```

Expected ledger after complete flow:

```
sequence: 001
event: outbound_bundle_created
run_id: f1000001-0000-0000-0000-000000000001
...

sequence: 002
event: handoff_pack_created
...

sequence: 003
event: inbound_artifact_processed
...

sequence: 004
event: verification_recorded
...

sequence: 005
event: operator_decision_recorded
result: accepted
...
```

The ledger is stored in `reports/` and excluded from version control.

---

## External Lab Trial

For real external trials, generate a handoff pack instead of simulating a lab response locally.

### Generate a handoff pack

Complete the full pilot run and export a bundle, then:

```bash
./examples/pilot/lab_simulator.sh --handoff-pack handoff/ --bundle pilot_bundle
```

**Expected output:**

```
  ✓  Handoff pack written: handoff/f1000001-0000-0000-0000-000000000001

  Run ID       : f1000001-0000-0000-0000-000000000001
  Receipt hash : 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb

  Contents:
    artifacts/receipt.json
    artifacts/export_packet.json
    manifest.txt
    operator_instructions.txt
    lab_response_instructions.txt
```

### Inspect the pack

```bash
cat handoff/<run-id>/manifest.txt
cat handoff/<run-id>/operator_instructions.txt
cat handoff/<run-id>/lab_response_instructions.txt
ls  handoff/<run-id>/artifacts/
```

### Send to real lab

Send the `handoff/<run-id>/` directory to the external lab.

The lab reads `lab_response_instructions.txt` to understand the required response format and returns a `lab_response.json` file.

### Receive response and verify

```bash
# Place the returned response in the inbound directory
cp lab_response.json inbound/lab_response_<run-id>.json

# Verify single response
./examples/pilot/verify.sh --inbound inbound/lab_response_<run-id>.json \
                           --bundle pilot_bundle

# Or run batch intake triage for all inbound responses
./examples/pilot/verify.sh --batch-inbound inbound/ --bundle pilot_bundle
```

### Handoff pack structure

```
handoff/<run-id>/
  manifest.txt                   index of included files + run identifiers
  operator_instructions.txt      instructions for the sending operator
  lab_response_instructions.txt  instructions for the receiving lab
  artifacts/
    receipt.json                 routing receipt — source of truth
    export_packet.json           approved dispatch packet (if present)
```

Generated handoff packs are excluded from version control via `.gitignore`.

---

## Pilot Operator Workflow

End-to-end sequence from outbound bundle to finalized decision record.

**Step 1 — Export a run bundle**

```bash
./examples/pilot/package_run.sh
```

**Step 2 — Simulate a lab response**

```bash
./examples/pilot/lab_simulator.sh pilot_bundle inbound/response_a.json
```

**Step 3 — Verify inbound response and generate decision artifact**

```bash
./examples/pilot/verify.sh --inbound inbound/response_a.json --bundle pilot_bundle
```

Expected output (success):

```
  response verified for current run

  Receipt hash : 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb
  Case ID      : f1000001-0000-0000-0000-000000000001

  Operator decision: ACCEPTED
  Decision record:   examples/pilot/reports/decision_response_a.txt
```

Expected output (mismatch):

```
  response belongs to different run

  Receipt hash mismatch:
    bundle   : 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb
    response : 0000000000000000000000000000000000000000000000000000000000000000

  Operator decision: REJECTED
  Reason:            run_mismatch
  Decision record:   examples/pilot/reports/decision_response_b.txt
```

**Step 4 — Inspect decision record**

```bash
cat examples/pilot/reports/decision_response_a.txt
```

```
run_id: 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb
artifact: response_a.json
verification_result: verified_for_current_run
operator_decision: accepted
timestamp: 2026-03-14T10:00:00Z
```

### Decision mapping

| Verification result | Operator decision | Reason |
|---|---|---|
| `verified_for_current_run` | `accepted` | — |
| `belongs_to_different_run` | `rejected` | `run_mismatch` |
| `malformed` | `rejected` | `malformed` |
| `unverifiable` | `rejected` | `unverifiable` |

Decision artifacts are written to `examples/pilot/reports/` and excluded from version control.

---

## Inbound Lab Response Verification

After exporting a run bundle, operators can simulate a lab response and verify
it belongs to the exact current run.

### Roundtrip

**Step 1 — Export a run bundle**

Complete the full pilot run (route → verify → export → reproducibility check), then:

```bash
./examples/pilot/package_run.sh
```

**Step 2 — Simulate a lab response**

```bash
./examples/pilot/lab_simulator.sh pilot_bundle lab_response.json
```

This reads the bundle and writes `lab_response.json` bound to the exact run
(receipt_hash, dispatch_id, case_id).

**Step 3 — Verify the inbound response**

```bash
./examples/pilot/verify.sh --inbound lab_response.json --bundle pilot_bundle
```

**Expected success output:**

```
  response verified for current run

  Receipt hash : 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb
  Case ID      : f1000001-0000-0000-0000-000000000001
  Dispatch ID  : <dispatch_id>
```

**Expected failure output (stale or mismatched response):**

```
  response belongs to different run

  Receipt hash mismatch:
    bundle   : 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb
    response : 0000000000000000000000000000000000000000000000000000000000000000
```

### Verification outcomes

| Outcome | Meaning |
|---|---|
| `response verified for current run` | Receipt hash matches — response belongs to this run |
| `response belongs to different run` | Receipt hash or dispatch ID mismatch |
| `response missing required artifact/field` | `receipt_hash` field absent from lab response |
| `response cannot be verified` | File missing, not valid JSON, or no bundle receipt |

### Using a specific bundle directory

```bash
./examples/pilot/verify.sh --inbound lab_response.json --bundle /path/to/bundle
```

### Test fixtures

| File | Description |
|---|---|
| `testdata/lab_response_valid.json` | Valid response — matches locked pilot receipt hash |
| `testdata/lab_response_stale.json` | Stale response — different receipt hash |
| `testdata/lab_response_malformed.json` | Malformed response — missing receipt_hash field |

---

## Operator Intake Triage

For batch processing of inbound lab response artifacts, use the `--batch-inbound`
mode. This acts as an operator inbox: classify every artifact in a drop directory
in one deterministic run.

### Batch triage roundtrip

**Step 1 — Export a run bundle**

```bash
./examples/pilot/package_run.sh
```

**Step 2 — Populate an inbound directory**

Drop lab response artifacts (e.g. from `lab_simulator.sh`) into an `inbound/` directory:

```bash
./examples/pilot/lab_simulator.sh pilot_bundle inbound/response_a.json
# copy other responses into inbound/ as needed
```

**Step 3 — Run intake triage**

```bash
./examples/pilot/verify.sh --batch-inbound inbound/ --bundle pilot_bundle
```

With a written report:

```bash
./examples/pilot/verify.sh --batch-inbound inbound/ --bundle pilot_bundle \
  --report reports/intake_report.txt
```

**Expected output:**

```
  accepted        response_a.json
                  Reason  : receipt_hash matches current run
                  Hash    : 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb

  mismatch        response_b.json
                  Reason  : receipt_hash does not match current run
                  Hash    : 0000000000000000000000000000000000000000000000000000000000000000

  malformed       response_c.json
                  Reason  : missing required field: receipt_hash

  duplicate       response_dup.json
                  Reason  : receipt_hash already accepted in this batch

  ────────────────────────────────────────
  Intake Summary

  Total processed:     4
  Accepted:            1
  Mismatched:          1
  Malformed:           1
  Unverifiable:        0
  Duplicate:           1
```

### Batch triage classifications

| Classification | Meaning |
|---|---|
| `accepted` | Receipt hash matches current run |
| `mismatch` | Receipt hash or dispatch ID belongs to a different run |
| `malformed` | Missing required field (`receipt_hash`) |
| `unverifiable` | File is not valid JSON |
| `duplicate` | Identical receipt hash already accepted in this batch |

### Inbound test fixtures

| File | Description |
|---|---|
| `testdata/inbound/response_a.json` | Accepted — matches locked receipt hash |
| `testdata/inbound/response_b.json` | Mismatch — wrong receipt hash |
| `testdata/inbound/response_c.json` | Malformed — missing receipt_hash |
| `testdata/inbound/response_dup.json` | Duplicate — same hash as response_a |

---

## Smoke Test

```bash
cargo test -p postcad-service --test pilot_bundle_smoke_test
```

The test routes using the pilot fixtures, compares the receipt to `expected_routed.json`
value-for-value, then verifies and compares to `expected_verify.json`.
