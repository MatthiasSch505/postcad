# PostCAD Pilot — Operator Handoff

---

## Purpose

This document is for a pilot operator or reviewer who needs to run the current PostCAD stack, validate the canonical output, and confirm that the acceptance criteria are met. It does not require source code knowledge. All required files are committed to the repository.

---

## Required Artifacts

| Role | Path |
|------|------|
| Pilot case | `examples/pilot/case.json` |
| Registry snapshot | `examples/pilot/registry_snapshot.json` |
| Routing config | `examples/pilot/config.json` |
| Derived policy (for verify) | `examples/pilot/derived_policy.json` |
| Expected routed output | `examples/pilot/expected_routed.json` |
| Expected verify output | `examples/pilot/expected_verify.json` |
| Local deployment doc | `docs/local_service_run.md` |
| Development bundle doc | `docs/development_bundle.md` |

All files are committed. Do not edit them before running acceptance.

---

## Acceptance Run Path

### 1. Start the service

```bash
docker compose up
```

Wait for the line:

```
postcad-service listening on 0.0.0.0:8080
```

Alternative without Docker:

```bash
cargo build --bin postcad-service
./target/debug/postcad-service
```

---

### 2. Health check

```bash
curl -s http://localhost:8080/health
```

Expected response:

```json
{"status":"ok"}
```

---

### 3. Version check

```bash
curl -s http://localhost:8080/version
```

Expected response:

```json
{"protocol_version":"postcad-v1","routing_kernel_version":"postcad-routing-v1","service":"postcad-service"}
```

---

### 4. Route with pilot fixture

```bash
curl -s -X POST http://localhost:8080/route \
  -H 'Content-Type: application/json' \
  -d "{
    \"case\":             $(cat examples/pilot/case.json),
    \"registry_snapshot\": $(cat examples/pilot/registry_snapshot.json),
    \"routing_config\":   $(cat examples/pilot/config.json)
  }" > /tmp/actual_routed.json
```

---

### 5. Compare route output against locked fixture

```bash
diff \
  <(jq -S '.receipt' /tmp/actual_routed.json) \
  <(jq -S '.' examples/pilot/expected_routed.json)
```

Expected: no diff output. Any diff means acceptance fails.

If `jq` is not available:

```bash
python3 -c "
import json, sys
got  = json.load(open('/tmp/actual_routed.json'))['receipt']
want = json.load(open('examples/pilot/expected_routed.json'))
assert got == want, f'MISMATCH\\ngot:  {json.dumps(got, sort_keys=True)[:200]}\\nwant: {json.dumps(want, sort_keys=True)[:200]}'
print('OK: route output matches expected_routed.json')
"
```

---

### 6. Verify with expected receipt and derived policy

```bash
curl -s -X POST http://localhost:8080/verify \
  -H 'Content-Type: application/json' \
  -d "{
    \"receipt\": $(cat examples/pilot/expected_routed.json),
    \"case\":    $(cat examples/pilot/case.json),
    \"policy\":  $(cat examples/pilot/derived_policy.json)
  }" > /tmp/actual_verify.json
```

---

### 7. Compare verify output against locked fixture

```bash
diff \
  <(jq -S '.' /tmp/actual_verify.json) \
  <(jq -S '.' examples/pilot/expected_verify.json)
```

Expected: no diff output.

If `jq` is not available:

```bash
python3 -c "
import json
got  = json.load(open('/tmp/actual_verify.json'))
want = json.load(open('examples/pilot/expected_verify.json'))
assert got == want, f'MISMATCH: got {got}'
print('OK: verify output matches expected_verify.json')
"
```

---

### 8. Stop the service

```bash
docker compose down
```

Or press `Ctrl-C` if running in the foreground.

---

## Acceptance Criteria

A pilot acceptance run is successful when all of the following hold:

1. The service starts locally without error.
2. `GET /health` returns exactly `{"status":"ok"}`.
3. `GET /version` returns `protocol_version: "postcad-v1"` and `routing_kernel_version: "postcad-routing-v1"`.
4. `POST /route` with the pilot fixtures returns a receipt whose fields match `expected_routed.json` value-for-value, including `receipt_hash: "0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb"`.
5. `POST /verify` with the locked receipt and derived policy returns exactly `{"result":"VERIFIED"}`.
6. No JSON files were manually edited during the run.
7. No field-by-field interpretation was required — all comparisons are automated diffs or exact equality.

---

## Failure Handling

If the acceptance run fails, check the following in order:

| Symptom | First check |
|---------|-------------|
| `Connection refused` | Service is not running or bound to wrong port — check `POSTCAD_ADDR`. |
| `404 Not Found` | Wrong endpoint path — check the URL against the list in step 4 or 6. |
| Fixture file not found | Wrong working directory — run commands from the repository root. |
| Route output does not match | Local binary is stale — run `cargo build --bin postcad-service` and restart. |
| Verify returns `FAILED` | `derived_policy.json` or `expected_routed.json` has been modified — restore with `git checkout examples/pilot/`. |
| Unexpected fields in output | The service binary does not match the committed source — rebuild from clean. |

---

## Red Lines

The following actions invalidate an acceptance run:

- **Do not edit protocol artifacts** (`expected_routed.json`, `expected_verify.json`, `derived_policy.json`) before or during acceptance.
- **Do not regenerate expected outputs** to make the run pass. If the actual output differs from the locked fixture, the run fails.
- **Do not modify service behavior** to produce the expected output. The service is a transparent wrapper; routing logic must not be changed.
- **Do not substitute alternate fixtures** without explicit written approval. The pilot acceptance run is defined against the committed fixture set only.
