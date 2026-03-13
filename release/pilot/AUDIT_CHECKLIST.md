# PostCAD — Pilot Audit Checklist

**Protocol:** `postcad-v1` · **Kernel:** `postcad-routing-v1`

---

## Purpose

Structured validation checklist for an external technical reviewer. Each item
maps to a concrete command or file inspection. No item requires trusting the
platform — every check is independently reproducible.

---

## 2-minute audit path

- [ ] Run `./scripts/check-pilot-bundle.sh` → `Bundle OK — all checks passed.`
- [ ] Open `expected_routed.json` — confirm `receipt_hash` is `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb`
- [ ] Open `expected_verify.json` — confirm content is `{"result": "VERIFIED"}`
- [ ] Read `SEQUENCE_DIAGRAM.md` — confirm the flow matches your expectations

---

## 10-minute audit path

Complete the 2-minute path, then:

- [ ] Read `REVIEWER_HANDOFF.md` — full review paths and high-signal file list
- [ ] Read `PROTOCOL_WALKTHROUGH.md` — step-by-step execution with artifact references
- [ ] Inspect `docs/openapi.yaml` — verify the API contract covers all pilot endpoints
- [ ] Start the service and run `./release/pilot/demo.sh` — confirm receipt hash matches
- [ ] Run independent verification (see **Flow checks** below)

---

## Integrity checks

Verify the bundle is complete and unmodified.

```bash
./scripts/check-pilot-bundle.sh
```

Expected: `Bundle OK — all checks passed.`

To verify a single file manually:

```bash
sha256sum release/pilot/expected_routed.json
# compare against release/pilot/manifest.sha256
grep expected_routed.json release/pilot/manifest.sha256
```

To verify all hashes directly with sha256sum:

```bash
(cd release/pilot && sha256sum -c manifest.sha256)
```

Expected: all lines end with `OK`.

---

## Contract checks

Verify the API contract is present and covers the pilot endpoints.

- [ ] `docs/openapi.yaml` exists in the bundle
- [ ] OpenAPI spec contains `POST /pilot/route-normalized`
- [ ] OpenAPI spec contains `POST /verify`
- [ ] OpenAPI spec contains `POST /dispatch/create`
- [ ] OpenAPI spec contains `POST /dispatch/{dispatch_id}/approve`
- [ ] OpenAPI spec contains `GET /dispatch/{dispatch_id}/export`

Quick check:

```bash
grep -c 'post:\|get:' release/pilot/docs/openapi.yaml
# expected: 5 (one per pilot endpoint)
```

---

## Flow checks

Verify the end-to-end pilot flow produces the canonical receipt.

**Prerequisites:** service running on `http://localhost:8080`

```bash
cargo run -p postcad-service &
sleep 2
./release/pilot/demo.sh
```

Expected final output:

```
  DEMO COMPLETE
  Candidate:    pilot-de-001
  Receipt hash: 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb
  Verification: VERIFIED
```

Independent verification against the frozen receipt (no demo run required):

```bash
python3 -c "
import json
print(json.dumps({
  'receipt': json.load(open('release/pilot/expected_routed.json')),
  'case':    json.load(open('release/pilot/case.json')),
  'policy':  json.load(open('release/pilot/derived_policy.json'))
}))" | curl -s -X POST http://localhost:8080/verify \
  -H 'Content-Type: application/json' \
  -d @-
```

Expected: `{"result":"VERIFIED"}`

---

## Determinism checks

Verify that the same inputs always produce the same receipt.

- [ ] `expected_routed.json` contains `"outcome": "routed"` and `"selected_candidate_id": "pilot-de-001"`
- [ ] `receipt_hash` in `expected_routed.json` is `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb`
- [ ] Running `demo.sh` produces the same `receipt_hash` as above
- [ ] Running `demo.sh` a second time (after clearing `data/`) produces the same `receipt_hash`

Automated determinism + verification smoke test:

```bash
cargo test -p postcad-service --test pilot_bundle_smoke_test
```

Expected: all tests pass.

---

## Files to inspect if something fails

| Symptom | File to inspect |
|---|---|
| Wrong receipt hash | `expected_routed.json` — check all 21 committed fields |
| Verify returns FAILED | `derived_policy.json` — policy may have drifted from routing-time snapshot |
| Missing bundle file | `MANIFEST.txt` — check source provenance; run `build-pilot-bundle.sh` |
| Hash mismatch in check script | `manifest.sha256` — regenerate with `build-pilot-bundle.sh` |
| Demo fails at routing step | `registry_snapshot.json`, `config.json` — check input shapes |
| Demo fails at dispatch step | `case.json` — must match the `routing_input` committed in the receipt |
| Unexpected API response shape | `docs/openapi.yaml` — compare response schema against actual output |
| Unclear what a file does | `INVENTORY.md` — purpose and flow step for every bundle file |

---

## Out of scope

The following are intentionally not covered by this checklist:

| Item | Reason |
|---|---|
| Multi-jurisdiction routing | Only jurisdiction `DE` is exercised in this pilot |
| Refusal flow | Not part of the canonical demo path |
| Manufacturer-side integration | Outside PostCAD scope; export packet is the handoff boundary |
| Production deployment | This bundle is for protocol verification only |
| AI or probabilistic rules | Not present; all rules are stateless and deterministic |
| Clinic or patient data | Not present; `case.json` contains routing-relevant fields only |
| Reviewer shell UI testing | Manual browser step; not automated in this checklist |
