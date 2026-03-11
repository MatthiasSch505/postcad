# PostCAD Pilot Handoff Packet

Entry point for a new operator or technical reviewer receiving this local pilot package.

---

## What this is

A single folder that orients a new person to the current local PostCAD pilot system without requiring them to assemble context from multiple locations first.

It does not replace the detailed docs elsewhere — it points to them in the right order.

---

## Who it is for

- A new operator who needs to run the local pilot
- A technical reviewer evaluating the pilot artifacts
- Anyone receiving this repo who was not involved in its build

---

## Prerequisites

| Requirement | Check |
|---|---|
| Rust toolchain | `cargo --version` |
| `python3` on PATH | `python3 --version` |
| `curl` on PATH | `curl --version` |
| Port 8080 free (or `POSTCAD_ADDR=host:port` override) | `lsof -i :8080` or skip |
| Repo cloned and on `main` branch | `git status` |

---

## Recommended order

1. **This file** — orient yourself
2. `FIRST_HOUR_GUIDE.md` — exact first-hour sequence with commands
3. `KNOWN_GOOD_STATE.md` — what the expected current state is and how to verify it
4. `HANDOFF_CHECKLIST.md` — confirm you have received everything
5. `release/README.md` — operator runbook (reset, start, smoke test, demo, evidence)
6. `release/review/README.md` → full review packet
7. `release/acceptance/README.md` → acceptance checklist and worksheet

---

## What the release folder contains

```
release/
├── README.md                    operator runbook
├── start_pilot.sh               start the local service
├── reset_pilot_data.sh          clean runtime data
├── smoke_test.sh                7-step deterministic smoke test
├── generate_evidence_bundle.sh  capture evidence to release/evidence/current/
├── evidence/
│   ├── README.md                evidence bundle guide
│   └── current/                 output from the last evidence run (gitignored)
├── review/                      external review packet (5 documents)
├── acceptance/                  acceptance checklist, worksheet, pre-check script
└── handoff/                     this packet
```

---

## Current state summary

- Protocol: `postcad-v1`, routing kernel: `postcad-routing-v1`
- Canonical pilot case: `case_id = f1000001-0000-0000-0000-000000000001`, jurisdiction DE
- Expected deterministic receipt: `receipt_hash = 0db54077cff0fbc4…`
- Expected verification result: `result = VERIFIED`
- All tests pass: `cargo test --workspace`
- Local pilot only — no external services, no production deployment
