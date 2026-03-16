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

## Command Guardrails

Pilot commands validate their arguments and print structured guidance when used incorrectly.

| Situation | Output |
|---|---|
| `--inspect-inbound-reply` called without a file | `INSPECT INBOUND REPLY — USAGE` block with example |
| `--export-dispatch` called with no current pilot run | `DISPATCH EXPORT — PRECONDITION NOT MET` block with recommended steps |
| Unknown or unrecognised flag | `UNKNOWN COMMAND` block directing to `--help-surface` |

All guardrail messages are plain text, deterministic, and non-interactive. They exit non-zero so scripts can detect the failure.

---

## Run Summary

To see the current pilot run state and recommended next step:

```bash
./examples/pilot/run_pilot.sh --run-summary
```

Prints the current run ID (if a receipt exists), the presence status of each key artifact, and the suggested next operator action. No commands are executed. No files are written.

---

## Run Fingerprint

To print a deterministic identifier derived from the protocol artifacts of the current run:

```bash
./examples/pilot/run_pilot.sh --run-fingerprint
```

Shows the run context, the artifact files contributing to the fingerprint (receipt, inbound reply, verification decision, dispatch packet), and the computed SHA-256 fingerprint. The fingerprint is stable as long as artifact content is unchanged and grows as more artifacts are added to the run. No commands are executed. No files are written.

---

## Lab Entrypoint

The fastest introduction for a lab or manufacturer evaluating the production-side workflow of the pilot:

```bash
./examples/pilot/run_pilot.sh --lab-entrypoint
```

Shows what the lab receives, what they are expected to do, why the workflow matters to a lab, the four commands to explore first, a one-line explanation per command, and the current run context. No commands are executed. No files are written.

---

## Business Entrypoint

The fastest introduction for non-technical external viewers evaluating the workflow and business meaning of the pilot:

```bash
./examples/pilot/run_pilot.sh --business-entrypoint
```

Shows what the pilot does in plain language, why it matters operationally, the four commands to explore first, a one-line explanation per command, and the current run context. Use this as the starting point for investors, operators, commercial partners, or lab owners. No commands are executed. No files are written.

---

## Engineer Entrypoint

The fastest technical introduction for engineers evaluating the protocol and workflow surfaces:

```bash
./examples/pilot/run_pilot.sh --engineer-entrypoint
```

Shows what to look at first, the recommended command order, one-line explanations for each command, and the current run context. Use this as the starting point before exploring receipts, dispatch packets, or the protocol chain. No commands are executed. No files are written.

---

## Protocol Chain

To see the ordered chain of protocol artifacts in the pilot workflow:

```bash
./examples/pilot/run_pilot.sh --protocol-chain
```

Shows the run context, the four-stage artifact chain (receipt → inbound reply → verification → dispatch packet), the current detection state of each stage, why the chain matters, and the commands to inspect each artifact further. No commands are executed. No files are written.

---

## Dispatch Packet

To understand how the dispatch packet functions as the execution-side artifact of the pilot workflow:

```bash
./examples/pilot/run_pilot.sh --dispatch-packet
```

Shows the run context, what the dispatch packet represents (execution-side handoff artifact, follows verified workflow state), why it matters, the commands to use it, and an engineer interpretation summary. If no dispatch artifact is present yet, shows guidance to run `--export-dispatch` first. No commands are executed. No files are written.

---

## Receipt Replay

To understand how the current receipt functions as the replayable routing commitment for the pilot workflow:

```bash
./examples/pilot/run_pilot.sh --receipt-replay
```

Shows the run context (run ID and receipt path), what the receipt commits (selected candidate, deterministic outcome, receipt hash as verification source of truth), the replay idea, the commands to use it, and an engineer interpretation summary. If a receipt exists, confirms it is available for replay-oriented inspection. No commands are executed. No files are written.

---

## Default Inbound Path Resolution

When `--inspect-inbound-reply` is called without a file argument, it tries to resolve the inbound reply automatically:

```bash
./examples/pilot/run_pilot.sh --inspect-inbound-reply
```

If `receipt.json` is present, the run ID is extracted from `case_id` (falling back to the first 12 characters of `receipt_hash`). The expected inbound reply path is then computed as `inbound/lab_reply_<run-id>.json`. If that file exists, inspection proceeds as if it had been provided explicitly.

If the file does not yet exist, the command prints a structured error:

```
INBOUND REPLY NOT FOUND

  Current run : <run-id>
  Expected    : <path>/inbound/lab_reply_<run-id>.json

  Next step:
    generate simulated reply:
      ./examples/pilot/run_pilot.sh --simulate-inbound
    or provide the file explicitly:
      ./examples/pilot/run_pilot.sh --inspect-inbound-reply <file>
```

When `verify.sh` is called with `--bundle` but without `--inbound`, the same resolution logic applies: the expected inbound reply path is computed from the bundle's `receipt.json` and used automatically if the file exists.

```bash
./examples/pilot/verify.sh --bundle examples/pilot
```

If the file is not yet present, the same `INBOUND REPLY NOT FOUND` message is shown with guidance to run `--simulate-inbound` first.

---

## Trace View

To see the workflow event trace for the current pilot run:

```bash
./examples/pilot/run_pilot.sh --trace-view
```

Shows the current run ID and the detection status of each workflow event — whether each artifact has been observed or is not yet present. Events are inferred from existing filesystem artifacts only. No commands are executed. No files are written.

---

## Inbound Reply Simulator

To demonstrate the pilot workflow end-to-end without a real external lab:

```bash
./examples/pilot/run_pilot.sh --simulate-inbound
```

Creates a deterministic simulated lab reply in the inbound directory. If a current run exists (`receipt.json` present), the file is named `inbound/lab_reply_<run-id>.json`. Without a current run it writes `inbound/lab_reply_simulated.json`.

The simulated reply is copied from `examples/pilot/testdata/lab_reply_simulated.json` — a minimal valid reply consistent with the pilot fixture structure.

After generating, the next steps are shown: inspect and verify.

---

## Demo Surface

The fastest single-command introduction to the PostCAD pilot:

```bash
./examples/pilot/run_pilot.sh --demo-surface
```

Prints a compact end-to-end narrative — what PostCAD is, the end-to-end flow, what the operator sees, why it matters, and the commands to explore further. No commands are executed. No files are written.

---

## System Overview

To understand what the PostCAD pilot system is and how it works, run:

```bash
./examples/pilot/run_pilot.sh --system-overview
```

This prints a short, deterministic explanation of the system — what PostCAD does, the core idea, the pilot workflow, key artifacts, operator tools, and system properties. No commands are executed. No files are written.

---

## Help Surface

The best starting point for a first-time operator — shows every available pilot mode and when to use each one:

```bash
./examples/pilot/run_pilot.sh --help-surface
```

Prints a consolidated, deterministic overview of all pilot operator modes, their purpose, and a recommended order for the normal workflow. No commands are executed. No files are written.

---

## Quickstart

The fastest way for a new operator to understand the pilot commands:

```bash
./examples/pilot/run_pilot.sh --quickstart
```

This prints the minimum command sheet for the complete pilot workflow — one exact command per step, one line of explanation each. No commands are executed. No files are written.

---

## Artifact Index

When you want to orient yourself in the pilot workflow — where are the files, what do I inspect next — run:

```bash
./examples/pilot/run_pilot.sh --artifact-index
```

This prints a compact, deterministic map of every artifact location in the current pilot workflow. No commands are executed. No files are written.

If `receipt.json` is present, the current run ID is shown and the inbound/outbound/ledger paths are resolved to the specific run. If no receipt exists, generic patterns are shown instead.

Expected output (with a current run):

```
PostCAD — Pilot Artifact Index
════════════════════════════════════════════════════════════

Current run : f1000001-0000-0000-0000-000000000001

Pilot bundle
  receipt.json       examples/pilot/receipt.json
  export_packet.json examples/pilot/export_packet.json

Inbound replies
  directory  examples/pilot/inbound/
  current    examples/pilot/inbound/lab_reply_f1000001-0000-0000-0000-000000000001.json

Outbound packages
  directory  examples/pilot/outbound/
  current    examples/pilot/outbound/lab_trial_f1000001-0000-0000-0000-000000000001/

Decision records
  directory  examples/pilot/reports/
  ledger     examples/pilot/reports/ledger_f1000001-0000-0000-0000-000000000001.txt

Verification
  command    ./examples/pilot/verify.sh --inbound .../inbound/lab_reply_<run-id>.json --bundle examples/pilot

────────────────────────────────────────
Operator flow reminder

  1. inspect inbound reply  — ./examples/pilot/run_pilot.sh --inspect-inbound-reply inbound/lab_reply_<run-id>.json
  2. verify inbound reply   — ./examples/pilot/verify.sh --inbound inbound/lab_reply_<run-id>.json --bundle examples/pilot
  3. export dispatch packet — ./examples/pilot/run_pilot.sh --export-dispatch

════════════════════════════════════════════════════════════
```

---

## Dispatch Export Outcomes

After the dispatch packet has been approved via the reviewer shell, the operator
can check export readiness and get a structured summary:

```bash
./examples/pilot/run_pilot.sh --export-dispatch
```

### DISPATCH EXPORT READY

```
  ════════════════════════════════════════
  DISPATCH EXPORT READY
  ════════════════════════════════════════
  Run ID  : <run-id>
  File    : examples/pilot/export_packet.json
  Result  : dispatch packet exported
  Next    : send packet to manufacturer / lab contact
  ════════════════════════════════════════
```

Meaning: `export_packet.json` is present and bound to the current run. The operator may send the dispatch packet to the manufacturer or lab contact.

### DISPATCH EXPORT FAILED

```
  ════════════════════════════════════════
  DISPATCH EXPORT FAILED
  ════════════════════════════════════════
  Result  : dispatch export failed
  Reason  : <reason>
  Next    : <operator guidance>
  ════════════════════════════════════════
```

### Failure guidance

| Failure cause | Reason shown | Next action |
|---|---|---|
| No `receipt.json` | no current pilot run found | generate or load a current pilot run before exporting |
| No `export_packet.json` | dispatch packet not present | verify the current route, then approve dispatch via reviewer shell |
| Other precondition | export precondition not met | confirm the pilot bundle and current artifacts are present |

The dispatch packet (`export_packet.json`) is created by the reviewer shell after
human approval of the routing decision. Run `cargo run -p postcad-service` and open
`http://localhost:8080/reviewer` to create and approve a dispatch commitment.

---

## Pilot Walkthrough

A new operator can see the complete pilot workflow in one command:

```bash
./examples/pilot/run_pilot.sh --walkthrough
```

This prints a 4-step guide — no commands are executed, no files are written.

Expected output:

```
POSTCAD PILOT WALKTHROUGH
════════════════════════════════════════════════════════════

Step 1 — Generate pilot bundle
  Command : ./examples/pilot/run_pilot.sh
  Creates : examples/pilot/receipt.json
  What    : Routes the dental case against the manufacturer registry.
            A cryptographic receipt is written and self-verified.
            The receipt hash is the verification source of truth for this run.

Step 2 — Inspect inbound lab reply
  Command : ./examples/pilot/run_pilot.sh --inspect-inbound-reply inbound/lab_reply_<run-id>.json
  Reads   : inbound/lab_reply_<run-id>.json
  What    : Checks that all required fields are present in the returned reply
            before running full cryptographic verification.
            Prints: reply structurally readable / reply missing required field(s)

Step 3 — Verify inbound reply
  Command : ./examples/pilot/verify.sh --inbound inbound/lab_reply_<run-id>.json --bundle examples/pilot
  Reads   : inbound/lab_reply_<run-id>.json + examples/pilot/receipt.json
  What    : Cryptographically binds the inbound reply to the current run.
            Writes a decision record to examples/pilot/reports/.
            Prints: VERIFICATION PASSED / VERIFICATION FAILED

Step 4 — Export dispatch packet
  Command : ./examples/pilot/run_pilot.sh --export-lab-trial-package
  Creates : examples/pilot/outbound/lab_trial_<run-id>/
  What    : Packages the routing receipt and lab reply template
            into a sendable directory for the external lab.
            Includes operator instructions, message kit, and receipt.

════════════════════════════════════════════════════════════
Run this walkthrough at any time:
  ./examples/pilot/run_pilot.sh --walkthrough
```

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

## Running a full PostCAD pilot trial

The `--trial-run` flag executes the complete external pilot workflow in a single command:

```bash
./examples/pilot/run_pilot.sh --trial-run
```

This command sequentially runs:

1. Route the pilot case → write `receipt.json`
2. Generate an external handoff pack → write `handoff/<run-id>/`
3. Simulate a lab response → write `inbound/trial_response.json`
4. Verify the inbound response against the current run
5. Record the operator decision

**Expected output:**

```
PostCAD — Full Trial Run
  ────────────────────────────────────────

  Starting PostCAD trial run

  Outbound bundle created
  External handoff pack created
  Simulated lab response generated
  Inbound response verified
  Operator decision: ACCEPTED
  Trial ledger updated

  Trial run completed

  Run ID : f1000001-0000-0000-0000-000000000001
  Ledger : examples/pilot/reports/ledger_f1000001-0000-0000-0000-000000000001.txt
  Receipt: examples/pilot/receipt.json
```

The trial run exits 0 if the operator decision is ACCEPTED, 1 if REJECTED. The ledger is updated at each step and stored in `reports/ledger_<run-id>.txt`.

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

## Verification Verdict Output

Every `verify.sh` run ends with a structured operator verdict block.

### VERIFICATION PASSED

```
  ════════════════════════════════════════
  VERIFICATION PASSED
  ════════════════════════════════════════
  Inbound : inbound/lab_reply_<run-id>.json
  Bundle  : examples/pilot
  Result  : verification passed
  Next    : operator may export dispatch packet
  ════════════════════════════════════════
```

Meaning: the inbound reply is cryptographically bound to the current run. The operator may proceed to export the dispatch packet.

### VERIFICATION FAILED

```
  ════════════════════════════════════════
  VERIFICATION FAILED
  ════════════════════════════════════════
  Inbound : inbound/lab_reply_<run-id>.json
  Bundle  : examples/pilot
  Result  : verification failed
  Next    : <operator guidance>
  ════════════════════════════════════════
```

Meaning: the inbound reply could not be verified against the current run. The `Next` line tells the operator what to do.

### Failure guidance

| Failure cause | Next action shown |
|---|---|
| Inbound file not found | check inbound reply file path and rerun |
| Inbound file not valid JSON | inspect inbound reply before verifying |
| Bundle directory missing receipt | confirm the pilot bundle path is correct |
| Reply missing required field | ask the lab to resend a complete reply if fields are unreadable |
| Receipt hash mismatch | confirm the lab returned the reply for the current run |

---

## Inbound Reply Inspection

Before running full verification, inspect a returned lab reply to confirm it is structurally complete.

### Inspect the reply

```bash
./examples/pilot/run_pilot.sh --inspect-inbound-reply inbound/lab_reply_<run-id>.json
```

### Expected output (structurally readable)

```
  Artifact : lab_reply_<run-id>.json

  Case ID          : f1000001-0000-0000-0000-000000000001
  Receipt hash     : 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb
  Lab ID           : dental-lab-berlin-001
  Status           : accepted
  Acknowledged at  : 2026-03-15T10:00:00Z
  Notes            : not present

  Required fields:
    present  receipt_hash
    present  lab_id
    present  status
    present  lab_acknowledged_at

  reply structurally readable
```

### Expected output (missing required field)

```
  Required fields:
    MISSING  receipt_hash
    present  lab_id
    ...

  reply missing required field(s): receipt_hash
```

### After inspection — run verification and decision

```bash
./examples/pilot/verify.sh \
  --inbound inbound/lab_reply_<run-id>.json \
  --bundle  examples/pilot
```

---

## Package Self-Check

Before sending the lab trial package, run the self-check to confirm it is complete.

### Workflow

```bash
# 1. Export the package
./examples/pilot/run_pilot.sh --export-lab-trial-package

# 2. Run the self-check
./examples/pilot/run_pilot.sh --check-lab-trial-package

# 3a. If ready — zip and send
zip -r lab_trial_<run-id>.zip outbound/lab_trial_<run-id>/

# 3b. If failed — regenerate
./examples/pilot/run_pilot.sh --export-lab-trial-package
```

### Expected output (ready)

```
  File check:

  present  manifest.txt
  present  operator_instructions.txt
  present  lab_instructions.txt
  present  lab_reply_template.json
  present  email_to_lab.txt
  present  short_message_to_lab.txt
  present  operator_send_note.txt
  present  receipt.json

  package ready for external lab send
```

### Expected output (failed)

```
  present  manifest.txt
  missing  lab_reply_template.json
  ...

  package check failed
  Regenerate the package:
    ./examples/pilot/run_pilot.sh --export-lab-trial-package
```

---

## First-Contact Send Flow

The sendable lab trial package includes a ready-to-use message kit. The operator does not need to write any message — everything is pre-generated.

### Generate the package (includes message kit)

```bash
./examples/pilot/run_pilot.sh --export-lab-trial-package
```

The package at `outbound/lab_trial_<run-id>/` now includes:

| File | Purpose |
|---|---|
| `email_to_lab.txt` | Draft email to paste and send |
| `short_message_to_lab.txt` | Short WhatsApp/Signal/LinkedIn message |
| `operator_send_note.txt` | Operator checklist: zip → send → receive → verify |

### Send using the email draft

Open `email_to_lab.txt`, copy the content, paste into your email client, attach the zip, and send.

### Send using the short message

Open `short_message_to_lab.txt`, copy the one-paragraph message, send via WhatsApp, Signal, or LinkedIn.

### After the lab returns the reply

Follow `operator_send_note.txt` step by step:

```
[ ] 1. Zip the package
[ ] 2. Send to lab with email_to_lab.txt or short_message_to_lab.txt
[ ] 3. Wait for lab_reply_<run-id>.json
[ ] 4. Place in inbound/
[ ] 5. Run verification and decision
[ ] 6. Inspect decision record
```

---

## Sendable Lab Trial Package

The quickest way to prepare a real external trial. Generates one directory with all files the lab needs to complete and return a reply.

### Generate the package

Run the pilot flow first, then:

```bash
./examples/pilot/run_pilot.sh --export-lab-trial-package
```

Expected output:

```
  Package written: examples/pilot/outbound/lab_trial_<run-id>

  Run ID      : <run-id>
  Receipt hash: <hash>

  Contents:
    manifest.txt
    operator_instructions.txt
    lab_instructions.txt
    lab_reply_template.json
    receipt.json
```

### Package structure

```
outbound/lab_trial_<run-id>/
  manifest.txt                  index of included files + run identifiers
  operator_instructions.txt     what to send, what to expect back, how to verify
  lab_instructions.txt          what the lab must fill in and return
  lab_reply_template.json       pre-filled template — lab fills 2 fields and returns
  receipt.json                  routing receipt (source of truth)
  export_packet.json            dispatch packet (if present)
```

### Zip and send to real lab

```bash
zip -r lab_trial_<run-id>.zip outbound/lab_trial_<run-id>/
```

Send the zip to the external lab. The lab reads `lab_instructions.txt` and returns the completed `lab_reply_template.json`.

### Receive completed reply

When the lab returns the filled template:

```bash
# Place it in your inbound directory
cp lab_reply_returned.json examples/pilot/inbound/lab_reply_<run-id>.json

# Verify and generate decision record
./examples/pilot/verify.sh \
  --inbound examples/pilot/inbound/lab_reply_<run-id>.json \
  --bundle  examples/pilot
```

Expected output (success):

```
  response verified for current run

  Operator decision: ACCEPTED
  Decision record:   examples/pilot/reports/decision_lab_reply_<run-id>.txt
```

Generated packages are excluded from version control via `.gitignore`.

---

## Real Manual External Trial

For trials with a real external lab (no simulator), use the manual reply template workflow. The handoff pack now includes a pre-filled `lab_reply_template.json` the lab can edit and return directly.

### Step 1 — Route and generate handoff pack

```bash
./examples/pilot/run_pilot.sh
./examples/pilot/lab_simulator.sh --handoff-pack handoff/ --bundle examples/pilot
```

### Step 2 — Send pack to real lab

Send `handoff/<run-id>/` to the external lab. The lab receives:

| File | Purpose |
|---|---|
| `lab_reply_template.json` | Pre-filled JSON — lab fills in `lab_acknowledged_at` and `lab_id` |
| `lab_response_instructions.txt` | Instructions for completing and returning the reply |
| `artifacts/receipt.json` | Routing receipt for reference |
| `manifest.txt` | Index of included files |

### Step 3 — Prepare reply template locally (optional)

To copy the template into the inbound directory as a starting point:

```bash
./examples/pilot/run_pilot.sh --prepare-manual-reply
```

Expected output:

```
  Reply template prepared for manual completion:
    examples/pilot/inbound/lab_reply_<run-id>.json

  Run ID      : <run-id>
  Receipt hash: <hash>

  The lab must fill in:
    lab_acknowledged_at  — ISO 8601 timestamp
    lab_id               — lab identifier

  Fields that must not be changed:
    lab_response_schema, receipt_hash, dispatch_id, case_id, status
```

### Step 4 — Lab fills and returns the template

The lab opens `lab_reply_template.json`, fills in two fields, and returns the file:

```json
{
  "lab_response_schema": "1",
  "receipt_hash": "<unchanged — pre-filled>",
  "dispatch_id": "<unchanged — pre-filled>",
  "case_id": "<unchanged — pre-filled>",
  "lab_acknowledged_at": "2026-03-15T14:00:00Z",
  "lab_id": "dental-lab-berlin-001",
  "status": "accepted"
}
```

`receipt_hash`, `dispatch_id`, and `case_id` must not be changed. The response will be rejected if `receipt_hash` does not match.

### Step 5 — Place returned file into inbound directory

```bash
cp lab_response_returned.json examples/pilot/inbound/lab_reply_<run-id>.json
```

### Step 6 — Verify and generate decision record

```bash
./examples/pilot/verify.sh \
  --inbound examples/pilot/inbound/lab_reply_<run-id>.json \
  --bundle  examples/pilot
```

Expected output (success):

```
  response verified for current run

  Operator decision: ACCEPTED
  Decision record:   examples/pilot/reports/decision_lab_reply_<run-id>.txt
```

Expected output (malformed or wrong run):

```
  response missing required artifact/field

  Operator decision: REJECTED
  Reason:            malformed
```

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

## Verification View

To inspect the current verification decision artifact as a clean, read-only operator-facing view:

```bash
./examples/pilot/run_pilot.sh --verification-view
```

Prints a read-only operator-facing view of the current verification artifact — status (detected or not detected), artifact path, and a summary of the verification outcome (run ID, decision, manufacturer ID, jurisdiction, reason/notes). No files are written. If no verification artifact is present, a clear fallback is shown and the command exits 0.

---

## Audit Receipt View

To inspect the current audit receipt as a clean, read-only operator-facing view:

```bash
./examples/pilot/run_pilot.sh --audit-receipt-view
```

Prints the receipt status (detected or not detected), the receipt path, the receipt summary (run ID, decision, jurisdiction, manufacturer ID, profile), why the receipt matters for audit and handoff, and the commands to use next. No commands are executed. No files are written. If `receipt.json` is not present, a clear fallback is shown and the command still exits 0.

---

## Smoke Test

```bash
cargo test -p postcad-service --test pilot_bundle_smoke_test
```

The test routes using the pilot fixtures, compares the receipt to `expected_routed.json`
value-for-value, then verifies and compares to `expected_verify.json`.

---

## Campaign Queue Runner

The campaign queue runner executes lane-1 campaigns sequentially in unattended mode.

```bash
# Preview the queue without executing anything
bash ops/run_campaign_queue.sh --dry-run

# Run all queued campaigns
bash ops/run_campaign_queue.sh

# Run at most 3 campaigns
bash ops/run_campaign_queue.sh --max 3
```

Campaign files are placed in `ops/campaign_queue/` as numbered markdown files
(e.g. `001_my_campaign.md`). They are processed in lexicographic order.

The runner enforces lane-1 safety before executing any campaign:
- Kernel crates (`crates/core`, `crates/routing`, `crates/compliance`, `crates/audit`,
  `crates/registry`) are forbidden — the campaign is rejected before execution.
- All file paths must fall within `examples/pilot/`, `docs/`, `ops/`, or
  `crates/service/tests/*surface_tests.rs`.

On success the campaign file is moved to `ops/campaign_queue/done/`. On failure after
one repair retry, the runner stops and writes a blocker report to `ops/last_result.md`.
Per-campaign logs are written to `ops/campaign_queue/logs/`.

Status entries are appended to `ops/queue_status.log` with ISO timestamps.

### Reading ops/last_result.md

`ops/last_result.md` is the single-file morning report. It is written before the first
campaign starts (status `RUNNING`) and updated at every terminal state. After a run it
contains:

- **Status** — `NOT_RUN` / `RUNNING` / `PASSED` / `BLOCKED` / `PARTIAL`
- Start and end times (ISO 8601 UTC)
- Campaigns discovered, executed, passed, passed-on-retry, blocked
- Last successful campaign name and blocked campaign name (if any)
- Latest commit hash and latest per-campaign log path
- Ordered list of campaigns still pending in the queue

### Alert hooks

Set environment variables before invoking the runner to receive a shell callback at
each terminal state. Hooks are optional, best-effort, and non-fatal — a hook failure
is logged but never changes the queue exit code.

```bash
# Example: log queue completion to a local file
export POSTCAD_QUEUE_ON_SUCCESS="echo '[postcad] queue passed' >> /tmp/postcad_alerts.log"
export POSTCAD_QUEUE_ON_BLOCKED="echo '[postcad] BLOCKED: $POSTCAD_QUEUE_LAST_CAMPAIGN' >> /tmp/postcad_alerts.log"
export POSTCAD_QUEUE_ON_PARTIAL="echo '[postcad] partial run: $POSTCAD_QUEUE_EXECUTED executed' >> /tmp/postcad_alerts.log"

bash ops/run_campaign_queue.sh
```

The following variables are exported into the hook's environment:

| Variable | Description |
|---|---|
| `POSTCAD_QUEUE_STATUS` | Terminal status string |
| `POSTCAD_QUEUE_EXECUTED` | Number of campaigns executed |
| `POSTCAD_QUEUE_PASSED` | Number of campaigns passed |
| `POSTCAD_QUEUE_BLOCKED` | Number of campaigns blocked |
| `POSTCAD_QUEUE_LAST_CAMPAIGN` | Name of last successful campaign |
| `POSTCAD_QUEUE_LOG_PATH` | Path to latest per-campaign log |
