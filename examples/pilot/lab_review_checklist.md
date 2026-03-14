# Lab Review Checklist

External recipient workflow for reviewing a PostCAD pilot bundle.

---

## Step 1 — Receive pilot bundle

Obtain the `pilot_bundle/` directory from the sending operator.

The bundle must contain:

| File | Contents |
|---|---|
| `route.json` | Routing result — outcome and selected manufacturer |
| `receipt.json` | Receipt with cryptographic hash commitment |
| `verification.json` | Verification result (`VERIFIED` or `FAILED`) |
| `export_packet.json` | Approved dispatch packet |
| `reproducibility.json` | Reproducibility check result |
| `bundle_manifest.json` | Artifact index with creation timestamp |

---

## Step 2 — Validate bundle

Run bundle validation to confirm all files are present, non-empty, and valid JSON:

```bash
./examples/pilot/validate_bundle.sh pilot_bundle
```

Expected output: `PASSED — bundle is valid`. If validation fails, request a corrected bundle from the sender.

---

## Step 3 — Inspect artifacts

Review the individual artifact files:

```bash
cat pilot_bundle/receipt.json           # routing decision and receipt hash
cat pilot_bundle/verification.json      # VERIFIED / FAILED
cat pilot_bundle/export_packet.json     # dispatch packet bound to the receipt
cat pilot_bundle/reproducibility.json   # reproducibility status
```

Replay the run summary for a human-readable overview:

```bash
./examples/pilot/replay_run.sh pilot_bundle
```

---

## Step 4 — Generate intake decision

Run the intake script to receive the bundle and produce an intake decision:

```bash
./examples/pilot/intake_bundle.sh pilot_bundle
```

Or run the decision script directly on a pre-validated bundle:

```bash
./examples/pilot/intake_decision.sh pilot_bundle
```

Possible verdicts:

| Verdict | Meaning |
|---|---|
| **accepted for review** | All critical artifacts present and valid — bundle is suitable for external review |
| **requires clarification** | Bundle present but one or more artifacts degraded — operator should clarify |
| **rejected** | One or more critical artifacts missing — bundle cannot be accepted |

---

## Intake criteria

A bundle is **accepted for review** when:

- [ ] Route artifact present with a valid outcome
- [ ] Receipt present
- [ ] Verification result is `VERIFIED`
- [ ] Dispatch packet present and exported
- [ ] Reproducibility check passed (recommended)

A bundle **requires clarification** when verification or reproducibility is missing or degraded but routing and dispatch artifacts are present.

A bundle is **rejected** when any of route, receipt, verification, or dispatch packet are missing.

---

## Notes

- The receipt hash in `receipt.json` is the cryptographic commitment to the routing decision. It must match across all artifact files.
- A `VERIFIED` verification result confirms that the receipt hash was independently reproduced from the original inputs.
- Reproducibility check confirms that the routing decision is deterministic across multiple executions.
- Do not accept a bundle where `verification.json` shows `FAILED` without explicit clarification from the sender.
