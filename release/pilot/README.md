# PostCAD — Pilot Bundle

**Protocol:** `postcad-v1` · **Kernel:** `postcad-routing-v1`
**Canonical receipt hash:** `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb`

---

## What is this

A frozen, runnable snapshot of the PostCAD pilot. Contains the scripts,
fixtures, and specifications needed to reproduce the full routing and
dispatch flow end-to-end.

**PostCAD** routes dental manufacturing cases deterministically against a
compliance-checked manufacturer registry, commits every decision to a
tamper-evident receipt, and provides a reviewer workflow for approving and
exporting the routing decision as a canonical handoff packet.

No AI. No randomness. Same inputs always produce the same receipt hash.

---

## Bundle integrity

Before running anything, verify the bundle is intact:

```bash
./scripts/check-pilot-bundle.sh
```

Expected final line: `Bundle OK — all checks passed.`

This verifies that all required files are present, no unexpected extras exist,
and every file's SHA-256 hash matches `manifest.sha256`.

If you are an external reviewer, start with `REVIEWER_HANDOFF.md`.
If you are unsure what a file does, see `INVENTORY.md`.
For a technical explanation of what each step does, see `PROTOCOL_WALKTHROUGH.md`.

---

## Bundle contents

| File | Description |
|---|---|
| `preflight.sh` | Environment check — run before anything else |
| `demo.sh` | Canonical end-to-end demo (steps 3–7 automated) |
| `case.json` | Canonical pilot case (crown/zirconia, jurisdiction DE) |
| `registry_snapshot.json` | Manufacturer registry snapshot |
| `config.json` | Routing configuration |
| `derived_policy.json` | Full routing policy bundle (used for verification) |
| `candidates.json` | Eligible candidate list |
| `expected_routed.json` | Frozen canonical routed receipt |
| `expected_verify.json` | Frozen canonical verification result |
| `docs/openapi.yaml` | OpenAPI 3.1 spec for all pilot endpoints |
| `docs/protocol_diagram.md` | One-page protocol flow and guarantees |
| `INVENTORY.md` | Maps every bundle file to its purpose and flow step |
| `MANIFEST.txt` | File inventory with source paths |
| `manifest.sha256` | SHA-256 hashes of all bundle files (sha256sum -c compatible) |

---

## Prerequisites

```
cargo (Rust toolchain)   https://rustup.rs
curl
python3
```

---

## Canonical path

All commands run from the **repo root**.

---

### Step 1 — Preflight

Check the environment before starting:

```bash
./release/pilot/preflight.sh
```

Expected final line: `Ready to run.`

If any check fails, follow the printed instructions and re-run preflight before continuing.

---

### Step 2 — Start service

In a dedicated terminal:

```bash
cargo run -p postcad-service
```

Expected output: `postcad-service listening on 0.0.0.0:8080`

Leave this terminal running. All subsequent steps use a separate terminal.

Override address: `POSTCAD_ADDR=http://localhost:9000 cargo run -p postcad-service`

---

### Step 3 — Run pilot submission

```bash
./release/pilot/demo.sh
```

This performs the full canonical flow automatically:
- Routes the pilot case via `POST /pilot/route-normalized`
- Creates a dispatch commitment via `POST /dispatch/create`
- Approves via `POST /dispatch/{id}/approve`
- Exports via `GET /dispatch/{id}/export` → writes `release/pilot/export_packet.json`
- Verifies via `POST /verify`

Expected final output:

```
  DEMO COMPLETE
  Candidate:    pilot-de-001
  Receipt hash: 0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb
  Verification: VERIFIED
```

Steps 4–6 below describe the same flow through the interactive reviewer shell.
Skip to step 7 if you used `demo.sh` and only want to verify the frozen artifact.

---

### Step 4 — Open reviewer shell

The reviewer shell is a browser UI over the same live endpoints:

```
http://localhost:8080/reviewer
```

Submit the canonical pilot case using the form. The routing receipt will appear
with the canonical `receipt_hash` above.

---

### Step 5 — Approve dispatch

In the reviewer shell: click **Approve dispatch** on the draft dispatch shown
after routing.

Via API directly:

```bash
DISPATCH_ID=<id from step 3 or reviewer shell>

curl -s -X POST http://localhost:8080/dispatch/${DISPATCH_ID}/approve \
  -H 'Content-Type: application/json' \
  -d '{"approved_by": "reviewer"}'
```

Expected: `"status": "approved"`

---

### Step 6 — Export artifact

In the reviewer shell: click **Export dispatch**.

Via API directly:

```bash
curl -s http://localhost:8080/dispatch/${DISPATCH_ID}/export
```

Expected: `"status": "exported"`, `"receipt_hash"` matches the canonical value above.

---

### Step 7 — Verify artifact

Independent replay verification against the frozen canonical receipt:

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

This replays the routing decision from raw inputs. No stored state is trusted.
The kernel recomputes every hash and confirms all 21 receipt commitments match.

---

## What PostCAD proves

| Guarantee | Mechanism |
|---|---|
| Routing is deterministic | Same inputs → same receipt hash, always |
| Receipt cannot be silently modified | `receipt_hash` covers all 21 fields |
| Verification requires no trust | Replays from raw inputs; ignores stored state |
| Dispatch binds to exact receipt | `receipt_hash` is the chain-of-custody key |
| Approval is recorded and final | `approved_by` + `approved_at` locked on approval |

---

## API reference

Full OpenAPI 3.1 spec: `docs/openapi.yaml`

Key endpoints used in the canonical flow:

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/pilot/route-normalized` | Route the pilot case |
| `POST` | `/dispatch/create` | Create dispatch commitment |
| `POST` | `/dispatch/{id}/approve` | Approve dispatch |
| `GET` | `/dispatch/{id}/export` | Export handoff packet |
| `POST` | `/verify` | Independent replay verification |
| `GET` | `/health` | Service health check |

---

## Regenerate this bundle

```bash
./scripts/build-pilot-bundle.sh
```

Copies canonical scripts, fixtures, and docs from their source locations
into this directory. See `MANIFEST.txt` for the full file inventory.
