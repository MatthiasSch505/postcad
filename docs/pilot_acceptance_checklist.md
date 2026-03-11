# Pilot Acceptance Checklist

Run path: `docs/operator_handoff.md`

---

- [ ] Service boots without error (`docker compose up` or `./target/debug/postcad-service`)
- [ ] `GET /health` returns `{"status":"ok"}`
- [ ] `GET /version` returns `protocol_version: "postcad-v1"` and `routing_kernel_version: "postcad-routing-v1"`
- [ ] `POST /route` output matches `examples/pilot/expected_routed.json` (no diff)
- [ ] `POST /verify` output matches `examples/pilot/expected_verify.json` (no diff)
- [ ] Docker path works (`docker compose up/down`) **or** cargo path works (`cargo build --bin postcad-service`)
- [ ] Stop command executed (`docker compose down` or `Ctrl-C`)
- [ ] No unexpected diffs in locked files (`git diff examples/pilot/` is clean)

---

All boxes checked = acceptance run **PASSED**.
Any unchecked box = acceptance run **FAILED** — see failure handling in `docs/operator_handoff.md`.
