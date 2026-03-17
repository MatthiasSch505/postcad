# PostCAD Constitution

**Status: FROZEN — do not edit without a core-truth campaign with explicit founder approval.**

---

## 1. Mission

PostCAD is a deterministic post-CAD routing and audit infrastructure layer for regulated manufacturing.

It verifies manufacturer certifications, enforces regulatory constraints by jurisdiction, routes cases to eligible manufacturers through rule-driven logic, and records an immutable, hash-chained audit trail.

PostCAD does not own clinical or manufacturing decisions. Every decision the platform makes must carry a `ReasonCode`. No silent pass/fail is permitted.

---

## 2. Frozen Kernel Invariants

The following behaviors are non-negotiable and must never drift:

- **routing semantics are deterministic and rule-driven** — same case + same eligible manufacturer list always produces the same routing outcome; no probabilistic or heuristic selection
- **refusal semantics must remain explicit** — every refusal carries a `ReasonCode`; no silent failure, no swallowed rejection
- **receipt/dispatch schema changes are controlled and not incidental** — schema modifications require a deliberate core-truth campaign, not a convenience edit
- **audit/integrity behavior must remain stable and reviewable** — hash-chained append-only log entries are never mutated; `verify_chain()` must always be correct and testable
- **compliance and jurisdiction logic are constraints, not guesses** — compliance rules are stateless and deterministic; they receive all context as arguments and produce auditable pass/fail with `ReasonCode`

---

## 3. Liability Boundary

- PostCAD does not become manufacturer, clinician, or legal decision-maker
- The platform acts as routing/audit/compliance infrastructure only
- No clinical outcome, no manufacturing commitment, and no legal determination is owned or asserted by this platform
- Responsibility for downstream clinical and manufacturing decisions rests entirely with the qualified parties in the supply chain

---

## 4. Forbidden Expansions

The following are permanently out of scope for lane-1 campaigns and require explicit founder approval to revisit:

- **no chairside operational workflow in v1** — PostCAD is a back-end infrastructure layer; chairside UX is out of scope
- **no hidden AI heuristics in core routing** — routing decisions must be explainable, rule-based, and fully traceable; no ML model may influence routing without a `ReasonCode`
- **no silent schema drift** — receipt, dispatch, and audit schemas may not change as a side-effect of surface or helper work
- **no convenience edits that cross into kernel truth** — operator UX improvements, documentation, and pilot surface work must not alter the semantics of core crates
- **no ownership of clinical/manufacturing liability** — the platform must never assert, imply, or accept liability for clinical or manufacturing outcomes

---

## 5. Campaign Lane Model

Campaigns are classified into two lanes:

| Lane | Scope | Speed | Test bar |
|------|-------|-------|----------|
| **core truth lane** | kernel / protocol / schema / audit / refusal / routing / compliance semantics | slow, surgical | strict; all invariants must pass |
| **surface lane** | docs / helpers / operator UX / read-only observability / pilot presentation | faster | must not mutate core meaning |

Rules:
- **core truth lane requires surgical scope and strict tests** — every changed invariant must have a test; no untested semantic change is permitted
- **surface lane may move faster but must not mutate core meaning** — surface campaigns touch only non-kernel files; any file that affects routing, compliance, audit, or schema logic is off-limits

---

## 6. Campaign Rule

- every campaign must declare allowed files, forbidden layers, definition of done, and exact test command

Every campaign — core truth or surface — must declare:

- **allowed files** — the exact set of files the campaign may create or modify
- **forbidden layers** — which kernel crates and protocol files are off-limits
- **definition of done** — the observable outcome that marks the campaign complete
- **exact test command** — the full shell command that must pass before the campaign is closed

No campaign may be considered complete without a passing test command.

---

## 7. Review Doctrine

- **prefer refusal over cleverness** — when a decision is ambiguous, the platform must refuse and surface a `ReasonCode` rather than guess
- **prefer boring determinism over speed if they conflict** — a slower, fully deterministic outcome is always preferred over a faster probabilistic one
- **treat ambiguous core edits as failures** — if a proposed change touches kernel semantics in a way that is unclear or contested, it is treated as a failure; the campaign must be redesigned with an explicit scope

---

*This document is the single canonical source of truth for what PostCAD may never drift from. Consult it before creating or running any campaign.*
