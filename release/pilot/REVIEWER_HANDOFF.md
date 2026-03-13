# PostCAD — Reviewer Handoff

**Protocol:** `postcad-v1` · **Kernel:** `postcad-routing-v1`
**Canonical receipt hash:** `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb`

---

## What this bundle demonstrates

PostCAD routes a dental manufacturing case deterministically against a
compliance-checked manufacturer registry and produces a cryptographically
committed routing receipt. The receipt can be independently verified by any
party holding the original inputs — without trusting the platform or its stored
state.

This bundle contains the frozen canonical pilot: one case, one registry
snapshot, one routing config, and the expected outputs. Running the demo
always produces the same receipt hash. Replaying verification on the frozen
receipt always returns `VERIFIED`.

---

## Fastest review path (5 minutes)

1. Read `SEQUENCE_DIAGRAM.md` — understand the full execution order in one view.
2. Open `expected_routed.json` — inspect the 21 committed fields in the canonical receipt.
3. Open `expected_verify.json` — confirm the expected verification result is `{"result":"VERIFIED"}`.
4. Run integrity check:
   ```bash
   ./scripts/check-pilot-bundle.sh
   ```
   Expected: `Bundle OK — all checks passed.`

---

## Full review path (20–30 minutes)

1. Read `PROTOCOL_WALKTHROUGH.md` — step-by-step explanation of each stage with artifact references.
2. Read `docs/openapi.yaml` — full API contract for all 5 pilot endpoints.
3. Start the service and run the canonical demo:
   ```bash
   cargo run -p postcad-service &
   ./release/pilot/demo.sh
   ```
   Confirm the receipt hash in the output matches the canonical value above.
4. Run independent verification against the frozen receipt:
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
5. Review the reviewer shell at `http://localhost:8080/reviewer` — the UI surface for human approval.
6. Read `docs/protocol_diagram.md` — guarantees and responsibility boundary.

---

## Files that matter most

| File | Why it matters |
|---|---|
| `README.md` | Step-by-step operator guide; start here for the canonical run path |
| `SEQUENCE_DIAGRAM.md` | Full execution order across all five actors; fastest orientation |
| `PROTOCOL_WALKTHROUGH.md` | Step-by-step explanation with artifact references and verification internals |
| `docs/openapi.yaml` | Authoritative API contract for all 5 pilot endpoints |
| `demo.sh` | Canonical 6-step demo script; source of truth for the correct invocation sequence |
| `ARTIFACT_REFERENCE.md` | Maps every artifact (frozen and runtime) to its source, shape, and determinism status |
| `expected_routed.json` | Frozen canonical receipt; 21 committed fields; `receipt_hash` is the integrity anchor |
| `expected_verify.json` | Frozen expected verification response: `{"result":"VERIFIED"}` |
| `MANIFEST.txt` | Source provenance — maps every bundle file back to its repo path |
| `manifest.sha256` | Machine-verifiable SHA-256 hashes of all bundle files |

---

## What to inspect to verify determinism

Determinism means: the same inputs always produce the same `receipt_hash`.

1. The canonical receipt hash is committed in `expected_routed.json`:
   ```
   "receipt_hash": "0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb"
   ```

2. Run the demo twice (with a clean data directory between runs) and compare
   the `receipt_hash` printed in the output. It must match the value above both
   times.

3. Inspect `expected_routed.json` fields: `routing_input` contains the exact
   8-field input committed at routing time. The kernel is stateless — re-running
   on these inputs produces the same receipt every time.

4. Run the smoke test to confirm this automatically:
   ```bash
   cargo test -p postcad-service --test pilot_bundle_smoke_test
   ```

---

## What to run to verify integrity

Verify the bundle files are complete and unmodified:

```bash
./scripts/check-pilot-bundle.sh
```

This checks:
- All required files are present
- No unexpected extra files exist
- Every file's SHA-256 hash matches `manifest.sha256`

To verify a single file manually:

```bash
sha256sum release/pilot/expected_routed.json
# compare against the corresponding line in release/pilot/manifest.sha256
```

---

## What to run to verify the exported artifact

Independent replay verification against the frozen receipt. Requires the
service to be running:

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

Expected response: `{"result":"VERIFIED"}`

This replays the routing decision from raw inputs. No stored state is read. The
kernel recomputes all 21 hash fields and compares them against the submitted
receipt. Any modification to the receipt — including a consistent re-hash — is
detected because the kernel recomputes the routing decision independently and
finds it no longer matches.

---

## Out of scope / intentionally not included

| Item | Status |
|---|---|
| Multi-jurisdiction routing | Not demonstrated in this pilot (single jurisdiction: DE) |
| Refusal flow | Not in the canonical demo path; covered by protocol vector tests in the repo |
| Manufacturer-side integration | Outside PostCAD's scope; export packet is the handoff boundary |
| AI or probabilistic decision-making | Not present; all rules are stateless and deterministic |
| Production deployment configuration | Not included; this bundle is for protocol verification only |
| Clinic or patient data | Not present; `case.json` contains only routing-relevant fields |
