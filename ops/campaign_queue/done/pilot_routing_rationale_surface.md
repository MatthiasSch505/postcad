campaign name

pilot: add routing rationale surface

objective

Add --routing-rationale to run_pilot.sh — a read-only inspection surface that
reads receipt.json and explains the routing decision in plain language: why
this manufacturer was selected, what strategy was used, and what the reason
code was. Falls back gracefully when no receipt.json is present.
Shell/docs/test only.

files allowed to change

examples/pilot/run_pilot.sh
examples/pilot/README.md
crates/service/tests/pilot_routing_rationale_surface_tests.rs

Claude prompt

Add --routing-rationale to examples/pilot/run_pilot.sh and document it in
examples/pilot/README.md, then write source-inspection tests in
crates/service/tests/pilot_routing_rationale_surface_tests.rs.

The --routing-rationale block must:

1. Check for receipt.json. If absent, print an error to stderr ("no pilot
   receipt found — run ./run_pilot.sh first") and exit 1.

2. Resolve run context from receipt.json using python3 (RR_ prefix vars):
     RR_RUN_ID       — routing_input.case_id
     RR_MFR_ID       — routing_decision.manufacturer_id
     RR_STRATEGY     — routing_decision.strategy (or "not recorded")
     RR_REASON_CODE  — routing_decision.reason_code (or "not recorded")
     RR_OUTCOME      — routing_decision.outcome (or "unknown")
   All vars default to "not detected" on parse failure.

3. Print the following sections in order:

   ROUTING RATIONALE
   -----------------
   RUN
     Run ID   : <RR_RUN_ID>
     Outcome  : <RR_OUTCOME>

   DECISION
     Manufacturer : <RR_MFR_ID>
     Strategy     : <RR_STRATEGY>
     Reason code  : <RR_REASON_CODE>

   EXPLANATION
     PostCAD selects a manufacturer deterministically from the list of
     compliant candidates. The strategy determines the selection algorithm:
       HighestPriority    — selects the candidate with the lowest priority
                            number; same input always yields same result
       DeterministicHash  — hashes the case ID to distribute load across
                            candidates; same case ID always maps to the
                            same manufacturer

   INVARIANTS
     - Same case + same eligible list = same manufacturer, always
     - No AI or probabilistic selection
     - Every decision carries a ReasonCode
     - Source: crates/routing/src/

4. Exit 0 after printing.

5. The block must not write any files, must not invoke $(date, must not
   perform filesystem checks beyond the initial receipt.json guard.

6. Add --routing-rationale to the --help-surface listing.

README: add ## Routing Rationale section (after ## Compliance View, before
## Command Map) with the command, a description of the four sections, and a
note that the strategy is deterministic and carries a ReasonCode.

Tests (crates/service/tests/pilot_routing_rationale_surface_tests.rs):
Use a routing_rationale_block() helper anchored on "--routing-rationale\""
and ending at "exit 0\nfi\n" (ASCII-safe, same pattern as other surface tests).
Write at least 35 tests covering:
  - flag present in script
  - flag appears in --help-surface
  - block exits 0
  - clean failure when receipt missing (exit 1, stderr, run_pilot.sh hint)
  - ROUTING RATIONALE header
  - RUN section: Run ID, Outcome fields
  - DECISION section: Manufacturer, Strategy, Reason code fields
  - EXPLANATION section: HighestPriority and DeterministicHash both described
  - INVARIANTS section: deterministic, ReasonCode, no AI mentions
  - stable section ordering: RUN before DECISION before EXPLANATION before INVARIANTS
  - no filesystem writes (> "${SCRIPT_DIR})
  - no date command ($(date)
  - README: ## Routing Rationale section, --routing-rationale command,
    deterministic/ReasonCode mention

test command

cd ~/projects/postcad && cargo test -p service pilot_routing_rationale_surface_tests -- --nocapture

commit message

pilot: add routing rationale surface
