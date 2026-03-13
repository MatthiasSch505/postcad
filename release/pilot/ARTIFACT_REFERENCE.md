# PostCAD — Pilot Artifact Reference

---

## Purpose

Maps every artifact produced or consumed in the canonical pilot flow to its
source, shape, and role. Use this document to understand what each file and
response object represents, where it comes from, and how it relates to the
frozen bundle assets.

---

## Frozen bundle artifacts vs generated runtime artifacts

**Frozen bundle artifacts** ship in this bundle. They are committed, hashed in
`manifest.sha256`, and must not change for the pilot to reproduce correctly.

**Generated runtime artifacts** are produced when `demo.sh` runs. They are
expected to be deterministic for identical inputs (same receipt hash every
time) but are not committed — they are produced fresh each run.

| Artifact | Type | File |
|---|---|---|
| Case input | Frozen | `case.json` |
| Registry snapshot | Frozen | `registry_snapshot.json` |
| Routing configuration | Frozen | `config.json` |
| Derived routing policy | Frozen | `derived_policy.json` |
| Eligible candidate list | Frozen | `candidates.json` |
| Canonical routing receipt | Frozen | `expected_routed.json` |
| Canonical verification result | Frozen | `expected_verify.json` |
| Export packet | Generated at runtime | `export_packet.json` |

---

## Canonical generated artifacts

### 1 — Normalized pilot submission (request payload)

Constructed by `demo.sh` and sent to `POST /pilot/route-normalized`. Not
written to disk — assembled inline from the frozen bundle files.

**Source:** `demo.sh` step 1, from `case.json`, `registry_snapshot.json`, `config.json`
**Endpoint:** `POST /pilot/route-normalized`

Shape:
```json
{
  "pilot_case": {
    "case_id":          "f1000001-0000-0000-0000-000000000001",
    "restoration_type": "crown",
    "material":         "zirconia",
    "jurisdiction":     "DE"
  },
  "registry_snapshot": { ... },
  "routing_config":    { "jurisdiction": "DE", "routing_policy": "allow_domestic_and_cross_border" }
}
```

---

### 2 — Routing result (in-memory response)

Returned by `POST /pilot/route-normalized`. Held in shell variable by
`demo.sh` — not written to disk at this step.

**Source:** routing kernel response
**Endpoint:** `POST /pilot/route-normalized`

Top-level keys: `receipt`, `derived_policy`

The `receipt` object contains 21 committed fields (see next section).
The `derived_policy` object is passed unchanged to `POST /dispatch/create`
and `POST /verify`.

---

### 3 — Routing receipt

The canonical output of the routing kernel. Committed to `expected_routed.json`
in the frozen bundle. Any run on the same inputs must produce a receipt with
the same `receipt_hash`.

**Source:** routing kernel
**Canonical file:** `expected_routed.json`
**Canonical receipt hash:** `0db54077cff0fbc45d22eff7323f5d49497fcac1a74d2d3955c00f0a9044bcfb`

Fields (21 total):

| Field | Role |
|---|---|
| `schema_version` | Receipt schema version |
| `routing_kernel_version` | Kernel that produced this receipt (`postcad-routing-v1`) |
| `outcome` | `routed` or `refused` |
| `selected_candidate_id` | Winning manufacturer (`pilot-de-001`) |
| `refusal_code` | Populated only when `outcome` is `refused` |
| `routing_input` | Exact 8-field input committed at routing time |
| `routing_input_hash` | SHA-256 of canonical routing input |
| `case_fingerprint` | SHA-256 of case fields |
| `policy_fingerprint` | SHA-256 of policy bundle |
| `policy_version` | Policy version string (null in this pilot) |
| `registry_snapshot_hash` | SHA-256 of registry snapshot |
| `candidate_pool_hash` | SHA-256 of full candidate pool before filtering |
| `eligible_candidate_ids_hash` | SHA-256 of post-compliance candidate list |
| `selection_input_candidate_ids_hash` | SHA-256 of list fed to the selector |
| `candidate_order_hash` | SHA-256 confirming stable sort before selection |
| `routing_decision_hash` | SHA-256 of the selection outcome |
| `routing_proof_hash` | SHA-256 of the full proof object |
| `audit_seq` | Sequence number in the audit chain |
| `audit_entry_hash` | SHA-256 of this audit entry |
| `audit_previous_hash` | SHA-256 of the previous audit entry (genesis = 64 zeros) |
| `receipt_hash` | SHA-256 of all preceding fields — top-level integrity seal |

---

### 4 — Dispatch record (server-side, transitions through three states)

Created by `POST /dispatch/create`. Stored server-side in `data/`. The
dispatch record transitions through three states during the pilot flow:

| State | Trigger | Key fields added |
|---|---|---|
| `draft` | `POST /dispatch/create` | `dispatch_id`, `receipt_hash`, `case_id`, `selected_candidate_id`, `verification_passed`, `created_at` |
| `approved` | `POST /dispatch/{id}/approve` | `approved_by`, `approved_at` |
| `exported` | `GET /dispatch/{id}/export` | `status: exported` |

The dispatch record is keyed by `receipt_hash`. Only one dispatch per
`receipt_hash` is allowed — a duplicate submission returns HTTP 409.

---

### 5 — Export packet

The canonical handoff artifact. Written to `export_packet.json` by `demo.sh`
step 4. This is the only runtime-generated artifact persisted to disk.

**Source:** `GET /dispatch/{id}/export`
**File:** `export_packet.json`

Fields:

| Field | Value in canonical pilot run |
|---|---|
| `dispatch_id` | UUID assigned at dispatch creation |
| `receipt_hash` | `0db54077...` (must match `expected_routed.json`) |
| `selected_candidate_id` | `pilot-de-001` |
| `case_id` | `f1000001-0000-0000-0000-000000000001` |
| `verification_passed` | `true` |
| `approved_by` | `reviewer` |
| `approved_at` | Timestamp — varies per run |
| `created_at` | Timestamp — varies per run |
| `status` | `exported` |
| `manufacturer_payload_json` | `null` in this pilot |

---

### 6 — Verification result

Returned by `POST /verify`. The frozen canonical version is `expected_verify.json`.

**Source:** `POST /verify` (independent replay — no stored state read)
**Canonical file:** `expected_verify.json`
**Expected response:** `{"result":"VERIFIED"}`

---

## Which artifact is verified

`POST /verify` accepts:
- `receipt` — the routing receipt (from `expected_routed.json` or from the routing result)
- `case` — the original case input (`case.json`)
- `policy` — the derived policy (`derived_policy.json`)

It replays the routing decision from raw inputs, recomputes all 21 hash fields,
and compares them to the submitted receipt. The receipt is the artifact being
verified — not the export packet and not the dispatch record.

---

## Which artifact is reviewed by the reviewer shell

The reviewer shell (`http://localhost:8080/reviewer`) displays the **draft
dispatch record** created by `POST /dispatch/create`. The reviewer sees the
`receipt_hash`, `selected_candidate_id`, and `verification_passed` fields and
approves the dispatch — which locks `approved_by` and `approved_at`.

The reviewer approves the routing decision (as committed in the receipt), not
the raw input or the export packet.

---

## Which artifacts are expected to remain deterministic

For identical inputs (`case.json`, `registry_snapshot.json`, `config.json`):

| Artifact | Deterministic? | Notes |
|---|---|---|
| `receipt_hash` | Yes — always `0db54077...` | Kernel is stateless |
| `selected_candidate_id` | Yes — always `pilot-de-001` | Hash-based selector, stable sort |
| `outcome` | Yes — always `routed` | All three candidates pass compliance |
| `dispatch_id` | No — UUID generated per run | Keying is by `receipt_hash`, not `dispatch_id` |
| `created_at` / `approved_at` | No — timestamps vary per run | Not covered by `receipt_hash` |
| `export_packet.json` | Partially — `receipt_hash` and selection are stable; timestamps vary | Compare `receipt_hash` field only |
| Verification result | Yes — always `{"result":"VERIFIED"}` | Pure replay; no state dependency |

---

## Where each artifact comes from

| Artifact | Script step | Endpoint | File |
|---|---|---|---|
| Normalized submission | `demo.sh` step 1 | `POST /pilot/route-normalized` | Assembled from `case.json`, `registry_snapshot.json`, `config.json` |
| Routing receipt | `demo.sh` step 1 (response) | `POST /pilot/route-normalized` | Canonical frozen copy: `expected_routed.json` |
| Draft dispatch | `demo.sh` step 2 | `POST /dispatch/create` | Server-side in `data/` |
| Approved dispatch | `demo.sh` step 3 | `POST /dispatch/{id}/approve` | Server-side in `data/` |
| Export packet | `demo.sh` step 4 | `GET /dispatch/{id}/export` | Written to `export_packet.json` |
| Verification result | `demo.sh` step 5 | `POST /verify` | Canonical frozen copy: `expected_verify.json` |

---

## Failure triage

### Missing `export_packet.json`

`demo.sh` did not complete step 4 (export). Check whether:
- The service was running at `http://localhost:8080`
- The dispatch was successfully approved in step 3
- `demo.sh` exited early (run with `bash -x ./release/pilot/demo.sh` to trace)

### Integrity mismatch (`check-pilot-bundle.sh` fails)

One or more frozen bundle files have been modified. Run:
```bash
(cd release/pilot && sha256sum -c manifest.sha256)
```
Lines not ending in `OK` identify the modified files. Regenerate the bundle
from the repo with `./scripts/build-pilot-bundle.sh`.

### Verification returns FAILED

The receipt submitted to `POST /verify` does not match the inputs. Likely causes:
- `derived_policy.json` does not match the policy used at routing time (`policy_fingerprint_mismatch`)
- `case.json` fields differ from those committed in the receipt (`case_fingerprint_mismatch`)
- The receipt itself was modified (`receipt_hash` or a committed field changed)

The error response includes a `code` field identifying which check failed.

### Reviewer shell shows no pending dispatch

No draft dispatch exists in `data/`. Either:
- `POST /dispatch/create` has not been called yet (run `demo.sh`, or call the endpoint manually)
- The service was restarted and `data/` was cleared
- A dispatch for this `receipt_hash` already exists and was already approved (HTTP 409 on create)
