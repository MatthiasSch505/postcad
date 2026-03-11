# PostCAD Protocol v1 — Pilot Workflow

Runnable end-to-end demonstration of PostCAD Protocol v1: registry-backed routing, deterministic selection, and replay verification.

---

## Pilot Overview

The pilot exercises the full protocol loop:

```
case.json + registry_snapshot.json + config.json
          ↓
  route-case-from-registry
          ↓
  receipt.json  (committed, self-verified)
          ↓
  Verification: OK
```

Routing is deterministic: the same inputs always produce the same receipt hash.

Self-verification runs automatically inside the routing step. If the receipt fails to verify, the command exits non-zero and no receipt is written.

---

## Inputs

| File | Description |
|------|-------------|
| `case.json` | Dental case — procedure, material, jurisdiction, routing policy. |
| `registry_snapshot.json` | Three manufacturing candidates with capabilities and attestation statuses. |
| `config.json` | Routing configuration — jurisdiction `DE`, policy `allow_domestic_and_cross_border`. |

### case.json

```json
{
  "case_id": "f1000001-0000-0000-0000-000000000001",
  "jurisdiction": "DE",
  "routing_policy": "allow_domestic_and_cross_border",
  "patient_country": "germany",
  "manufacturer_country": "germany",
  "material": "zirconia",
  "procedure": "crown",
  "file_type": "stl"
}
```

### registry_snapshot.json

Three active manufacturers, all serving Germany, all capable of zirconia crowns:

- `pilot-de-001` — Alpha Dental GmbH (5-day SLA)
- `pilot-de-002` — Beta Zahntechnik GmbH (3-day SLA)
- `pilot-de-003` — Gamma Dental GmbH (7-day SLA)

All three pass the compliance gate (`attestation_statuses: ["verified"]`). The routing kernel selects `pilot-de-001` deterministically via `HighestPriority` strategy.

---

## Running the Pilot

Prerequisites: Rust toolchain installed (`cargo`).

### Full routing + verification

```bash
cd examples/pilot
./run_pilot.sh
```

Expected output:

```
PostCAD Protocol v1 — Pilot Workflow
======================================

Routing case...

Result:               routed
Selected candidate:   pilot-de-001
Receipt hash:         <64-char hex digest>
Kernel version:       postcad-routing-v1

Receipt written to:   examples/pilot/receipt.json

Verification: OK

  (Self-verification ran inside the routing step.
   The receipt would not have been emitted if it failed to verify.)
```

### Standalone end-to-end demo

```bash
cd examples/pilot
./verify.sh
```

This runs the `demo-run` command using frozen protocol-vector v01 fixtures, which routes a case and immediately verifies the receipt. Output includes the VERIFIED result and protocol version.

```bash
./verify.sh --json    # JSON output
```

---

## Expected Output

After `./run_pilot.sh`:

- `receipt.json` is written to `examples/pilot/`.
- The receipt is a complete PostCAD Protocol v1 routing receipt with all 21 committed fields.
- The `receipt_hash` is a SHA-256 digest of the canonical receipt content.
- The receipt hash is stable: running the script again with the same inputs produces the same hash.

Determinism check:

```bash
HASH1=$(./run_pilot.sh 2>/dev/null | grep "Receipt hash" | awk '{print $NF}')
HASH2=$(./run_pilot.sh 2>/dev/null | grep "Receipt hash" | awk '{print $NF}')
[ "$HASH1" = "$HASH2" ] && echo "Determinism: OK"
```

---

## Protocol Reference

- Specification: `docs/postcad_protocol_v1.md`
- Protocol version: `postcad-v1` (semver `1.0`)
- Routing kernel: `postcad-routing-v1` (semver `1.0`)
- Full manifest: `postcad-cli protocol-manifest`
- Compact info:  `postcad-cli protocol-info`
