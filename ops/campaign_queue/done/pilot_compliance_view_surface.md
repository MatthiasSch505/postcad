campaign name

pilot: add compliance view surface

objective

Add --compliance-view to run_pilot.sh — a read-only inspection surface that
reads receipt.json and prints the compliance rules that were evaluated during
the pilot run, each with a pass/fail indicator, the applicable jurisdiction,
and the manufacturer that was assessed. Falls back gracefully when no
receipt.json is present. Shell/docs/test only.

files allowed to change

examples/pilot/run_pilot.sh
examples/pilot/README.md
crates/service/tests/pilot_compliance_view_surface_tests.rs

Claude prompt

Add --compliance-view to examples/pilot/run_pilot.sh and document it in
examples/pilot/README.md, then write source-inspection tests in
crates/service/tests/pilot_compliance_view_surface_tests.rs.

The --compliance-view block must:

1. Check for receipt.json. If absent, print an error to stderr ("no pilot
   receipt found — run ./run_pilot.sh first") and exit 1.

2. Resolve run context from receipt.json using python3 (CV_ prefix vars):
     CV_RUN_ID   — routing_input.case_id
     CV_MFR_ID   — routing_decision.manufacturer_id
     CV_COUNTRY  — routing_input.case.destination_country
     CV_OUTCOME  — routing_decision.outcome (or "unknown")
   All vars default to "not detected" on parse failure.

3. Print the following sections in order:

   COMPLIANCE VIEW
   ---------------
   RUN
     Run ID   : <CV_RUN_ID>
     Country  : <CV_COUNTRY>
     Outcome  : <CV_OUTCOME>

   RULES EVALUATED
     [pass] ManufacturerActive   — manufacturer is active in registry
     [pass] Capability           — supports required material and procedure
     [pass] Iso13485             — baseline quality certification present
     [info] EuMdr                — CE Mark required for EU destinations
     [info] FdaClearance         — FDA 510k required for United States
     [info] MhlwApproval         — MHLW approval required for Japan

   The [info] lines are always printed; they are labelled info because
   the pilot currently evaluates all rules for the fixed demo case.

   MANUFACTURER
     Selected : <CV_MFR_ID>

   NOTE
     Compliance rules are stateless and deterministic. Each rule carries
     a ReasonCode. No AI evaluation. Rules are defined in
     crates/compliance/src/rules/.

4. Exit 0 after printing.

5. The block must not write any files, must not invoke $(date, must not
   perform filesystem checks beyond the initial receipt.json guard.

6. Add --compliance-view to the --help-surface listing.

README: add ## Compliance View section (after ## Artifact Index, before
## Command Map) with the command, a description of the four sections, and a
note that compliance rules are stateless and carry a ReasonCode.

Tests (crates/service/tests/pilot_compliance_view_surface_tests.rs):
Use a compliance_view_block() helper anchored on "--compliance-view\"" and
ending at "exit 0\nfi\n" (ASCII-safe, same pattern as other surface tests).
Write at least 35 tests covering:
  - flag present in script
  - flag appears in --help-surface
  - block exits 0
  - clean failure when receipt missing (exit 1, stderr, run_pilot.sh hint)
  - COMPLIANCE VIEW header
  - RUN section: Run ID, Country, Outcome fields
  - RULES EVALUATED section: all six rule names present
  - [pass] and [info] markers present
  - MANUFACTURER section: Selected field
  - NOTE section: mentions stateless, deterministic, ReasonCode
  - stable section ordering: RUN before RULES EVALUATED before MANUFACTURER before NOTE
  - no filesystem writes (> "${SCRIPT_DIR})
  - no date command ($(date)
  - README: ## Compliance View section, --compliance-view command, stateless/ReasonCode mention

test command

cd ~/projects/postcad && cargo test -p service pilot_compliance_view_surface_tests -- --nocapture

commit message

pilot: add compliance view surface
