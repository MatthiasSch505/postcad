# PostCAD Pilot Readiness — Out of Scope

Explicit statement of what the current local pilot package does not claim and does not provide.

---

## Protocol and system design

**Not in scope:**
- Protocol redesign or versioning beyond `postcad-v1`
- Routing algorithm changes or improvements
- Schema changes to the routing receipt
- Kernel behavior changes
- Hash function changes or commitment structure changes
- Any modification to frozen protocol values

The protocol, kernel, schemas, and hashing behavior are frozen. See `release/freeze/FROZEN_BOUNDARIES.md` for the complete frozen scope.

---

## Semantic acceptance engine

**Not in scope:**
- Automated acceptance decisions
- Machine-readable pass/fail verdicts
- CI-integrated acceptance gates
- Automated regulatory scoring

The acceptance bundle (`release/acceptance/`) provides a checklist and worksheet for human review. It does not implement an automated acceptance engine. Filling in the worksheet is a human task.

---

## Certification and regulatory claims

**Not in scope:**
- Regulatory approval (CE mark, FDA clearance, MHLW, or equivalent)
- Standards certification (ISO 13485, ISO 9001, or equivalent)
- Legal compliance claims of any kind
- Third-party audit or attestation
- Claims of production-grade quality or safety

The pilot demonstrates a deterministic routing and verification flow for evaluation purposes. It makes no regulatory or certification claims.

---

## Production deployment

**Not in scope:**
- Deployment to any hosted environment
- Cloud infrastructure or container orchestration
- Network-accessible services beyond `localhost`
- Multi-tenant or multi-operator operation
- Persistent storage beyond local `data/` directories
- Backup, recovery, or high-availability configuration

The service runs locally on `localhost:8080` only. All data is local. There is no remote or hosted component.

---

## Coverage beyond the canonical pilot case

**Not in scope:**
- Multiple simultaneous cases
- Non-canonical input fixtures
- Edge cases not covered by `examples/pilot/`
- Performance or load testing
- Failure injection beyond the smoke test's accepted 409 idempotent re-run

The pilot covers exactly one canonical case: `case_id = f1000001-0000-0000-0000-000000000001`, jurisdiction DE, zirconia crown. Coverage beyond this case is not claimed.

---

## Guarantees about external systems

**Not in scope:**
- Integration with external CAD systems
- Integration with external manufacturer systems
- Real-time dispatch or verification with any external party
- Network transport security or encryption
- Authentication or authorization

---

## Where to look for actual scope

| Question | Where to look |
|---|---|
| What is frozen? | `release/freeze/FROZEN_BOUNDARIES.md` |
| What does the system do? | `release/review/SYSTEM_OVERVIEW.md` |
| What does the acceptance checklist cover? | `release/acceptance/PILOT_ACCEPTANCE_CHECKLIST.md` |
| What endpoints exist? | `release/freeze/FROZEN_BOUNDARIES.md` — "Frozen: endpoint surface" |
| What are the canonical inputs? | `examples/pilot/` |
| What does "ready for external review" mean? | `release/readiness/READINESS_SNAPSHOT.md` — final section |
