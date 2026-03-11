# PostCAD External Review Packet

Entry point for external reviewers. This packet describes the current local pilot system — what it does, how the operator runs it, what it produces, and what is intentionally out of scope.

---

## Who this is for

Anyone reviewing the PostCAD pilot from outside the build loop: technical reviewers, auditors, acceptance testers.

---

## Recommended reading order

1. **`SYSTEM_OVERVIEW.md`** — what the system is and what the current pilot covers
2. **`OPERATOR_FLOW.md`** — exact steps an operator takes to run the pilot locally
3. **`ARTIFACT_GUIDE.md`** — what the evidence bundle contains and how to inspect it
4. **`BOUNDARIES.md`** — what is frozen, what is not claimed, what is out of scope

---

## Key references

| Resource | Purpose |
|---|---|
| `release/README.md` | Operator runbook (prerequisites, commands, troubleshooting) |
| `release/evidence/README.md` | Evidence bundle structure and inspection guide |
| `release/evidence/current/` | Actual evidence output from the last local run |
| `release/evidence/current/summary.txt` | Human-readable confirmation that all 7 steps passed |
| `examples/pilot/` | Canonical input fixtures used by the pilot flow |

---

## Current maturity

The local pilot demonstrates a complete, deterministic routing and verification flow:

- Case intake → routing decision → receipt issuance → dispatch → verification
- All steps are deterministic: same inputs always produce the same outputs
- The routing receipt is cryptographically self-verifiable without the routing engine
- All evidence is captured locally in `release/evidence/current/`

This is a local pilot only. No external services, no production deployment, no cloud infrastructure.
