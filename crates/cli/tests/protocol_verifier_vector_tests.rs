//! Protocol verifier vector tests.
//!
//! Each vector in `tests/protocol_verifier_vectors/` contains:
//!   - `case.json`                  — CaseInput
//!   - `registry_snapshot.json`     — Vec<ManufacturerRecord> (typed registry)
//!   - `registry_routing_config.json` — RegistryRoutingConfig (used for routing to generate receipt)
//!   - `receipt.json`               — RoutingReceipt (generated on first run, then frozen)
//!   - `policy.json`                — RoutingPolicyBundle for verification (derived or drifted; generated on first run)
//!   - `vector.json`                — metadata: expected_result, expected_error_code, generation_type
//!
//! # Seeding
//!
//! When `receipt.json` and `policy.json` are absent the test runner generates
//! them from the current routing output (applying any configured tampering or
//! drift) and passes.  On all subsequent runs both files are loaded from disk
//! and treated as the frozen protocol specification.
//!
//! # Generation types
//!
//! | `generation_type`                  | What is generated                                                   |
//! |------------------------------------|---------------------------------------------------------------------|
//! | `"valid"`                          | Correct receipt + correct policy bundle                             |
//! | `"tamper_receipt_hash"`            | receipt_hash replaced with wrong value; policy bundle correct       |
//! | `"tamper_routing_decision_hash"`   | routing_decision_hash replaced + receipt_hash recomputed; policy ok |
//! | `"drift_snapshot"`                 | Correct receipt; policy snapshot evidence references drifted        |
//! | `"drift_extra_candidate"`          | Correct receipt; policy candidates list has an extra phantom entry  |

use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use postcad_cli::{route_case_from_registry_json, verify_receipt_from_policy_json};

// ── Directory helpers ─────────────────────────────────────────────────────────

fn verifier_vectors_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/protocol_verifier_vectors")
}

fn read_vv_file(vector: &str, file: &str) -> String {
    let path = verifier_vectors_dir().join(vector).join(file);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read verifier vector {}/{}: {}", vector, file, e))
}

fn vv_path(vector: &str, file: &str) -> PathBuf {
    verifier_vectors_dir().join(vector).join(file)
}

// ── Metadata ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct VectorMeta {
    expected_result: String,
    expected_error_code: Option<String>,
    generation_type: String,
}

fn read_meta(vector: &str) -> VectorMeta {
    let raw = read_vv_file(vector, "vector.json");
    serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("cannot parse {}/vector.json: {}", vector, e))
}

// ── Receipt hash helper ───────────────────────────────────────────────────────

/// Recomputes receipt_hash for a (possibly tampered) receipt Value.
///
/// Removes the `receipt_hash` field, serialises the remainder to compact JSON
/// (serde_json alphabetises keys when serialising from a Value backed by a
/// BTreeMap, which is how RoutingReceipt deserialises), then returns the
/// lowercase SHA-256 hex digest — identical to `hash_receipt_content` in lib.rs.
fn recompute_receipt_hash(receipt_val: &Value) -> String {
    let mut obj = receipt_val.clone();
    obj.as_object_mut().unwrap().remove("receipt_hash");
    let canonical = serde_json::to_string(&obj).unwrap();
    format!("{:x}", Sha256::digest(canonical.as_bytes()))
}

// ── Generator ─────────────────────────────────────────────────────────────────

/// Generates `receipt.json` and `policy.json` for a vector if they are absent.
///
/// Called lazily on first run; subsequent runs load the frozen files from disk.
fn generate_if_missing(vector: &str, meta: &VectorMeta) {
    let receipt_path = vv_path(vector, "receipt.json");
    let policy_path = vv_path(vector, "policy.json");
    if receipt_path.exists() && policy_path.exists() {
        return; // already seeded
    }

    let case_json = read_vv_file(vector, "case.json");
    let registry_json = read_vv_file(vector, "registry_snapshot.json");
    let config_json = read_vv_file(vector, "registry_routing_config.json");

    let result = route_case_from_registry_json(&case_json, &registry_json, &config_json)
        .unwrap_or_else(|e| {
            panic!(
                "vector '{}' routing failed during generation: {}",
                vector, e
            )
        });

    // Start with the valid receipt and the derived policy bundle.
    let mut receipt_val: Value =
        serde_json::to_value(&result.receipt).expect("receipt must serialise to Value");
    let mut policy_val: Value = serde_json::from_str(&result.derived_policy_json)
        .expect("derived_policy_json must parse to Value");

    match meta.generation_type.as_str() {
        "valid" => {
            // No modifications — both values stay as derived.
        }

        "tamper_receipt_hash" => {
            // Replace receipt_hash with a wrong value.  The receipt_hash field
            // itself is changed; no recomputation is needed because the test
            // checks that step 1b (canonicalization check) fires.
            receipt_val["receipt_hash"] = Value::String("a".repeat(64));
        }

        "tamper_routing_decision_hash" => {
            // Replace routing_decision_hash with a wrong value, then
            // recompute receipt_hash so that step 1b passes and step 1e
            // (routing_decision_hash_mismatch) fires instead.
            receipt_val["routing_decision_hash"] = Value::String("b".repeat(64));
            let new_hash = recompute_receipt_hash(&receipt_val);
            receipt_val["receipt_hash"] = Value::String(new_hash);
        }

        "drift_snapshot" => {
            // Keep the receipt intact.  Change the evidence_references in the
            // first snapshot entry so the registry_snapshot_hash computed from
            // the provided policy bundle no longer matches the receipt commitment.
            if let Some(snapshots) = policy_val["snapshots"].as_array_mut() {
                if let Some(first) = snapshots.first_mut() {
                    first["evidence_references"] = serde_json::json!(["DRIFTED-EVIDENCE-REF"]);
                }
            }
        }

        "drift_extra_candidate" => {
            // Keep the receipt intact.  Add a phantom candidate to the candidates
            // list with no corresponding snapshot entry.  The snapshot hash is
            // therefore unchanged (step 4c passes) but the candidate_pool_hash
            // recomputed from the extended candidate list differs from the receipt
            // commitment (step 4d fires candidate_pool_hash_mismatch).
            let extra = serde_json::json!({
                "id": "mfr-phantom-drift",
                "manufacturer_id": "mfr-phantom-drift",
                "location": "domestic",
                "accepts_case": true,
                "eligibility": "ineligible"
            });
            if let Some(candidates) = policy_val["candidates"].as_array_mut() {
                candidates.push(extra);
            }
        }

        other => panic!("unknown generation_type {:?} in vector '{}'", other, vector),
    }

    let receipt_pretty =
        serde_json::to_string_pretty(&receipt_val).expect("receipt must serialise to pretty JSON");
    let policy_pretty =
        serde_json::to_string_pretty(&policy_val).expect("policy must serialise to pretty JSON");

    std::fs::write(&receipt_path, receipt_pretty)
        .unwrap_or_else(|e| panic!("cannot write {}: {}", receipt_path.display(), e));
    std::fs::write(&policy_path, policy_pretty)
        .unwrap_or_else(|e| panic!("cannot write {}: {}", policy_path.display(), e));

    eprintln!(
        "protocol_verifier_vector '{}': generated receipt.json + policy.json (commit these files)",
        vector
    );
}

// ── Core runner ───────────────────────────────────────────────────────────────

/// Runs one verifier vector.
///
/// 1. Seeds `receipt.json` / `policy.json` on first run.
/// 2. Loads the frozen files.
/// 3. Calls `verify_receipt_from_policy_json`.
/// 4. Asserts the result matches the expected outcome and error code.
fn run_verifier_vector(vector: &str) {
    let meta = read_meta(vector);
    generate_if_missing(vector, &meta);

    let receipt_json = read_vv_file(vector, "receipt.json");
    let case_json = read_vv_file(vector, "case.json");
    let policy_json = read_vv_file(vector, "policy.json");

    let result = verify_receipt_from_policy_json(&receipt_json, &case_json, &policy_json);

    match meta.expected_result.as_str() {
        "ok" => {
            result.unwrap_or_else(|f| {
                panic!(
                    "verifier vector '{}': expected Ok but got Err({:?}): {}",
                    vector, f.code, f.message
                )
            });
        }
        "err" => {
            let failure = result.unwrap_err_or_else(|| {
                panic!(
                    "verifier vector '{}': expected Err({:?}) but verification succeeded",
                    vector, meta.expected_error_code,
                )
            });
            if let Some(expected_code) = &meta.expected_error_code {
                assert_eq!(
                    failure.code,
                    expected_code.as_str(),
                    "verifier vector '{}': wrong error code (expected {:?}, got {:?}): {}",
                    vector,
                    expected_code,
                    failure.code,
                    failure.message,
                );
            }
        }
        other => panic!("vector '{}': unknown expected_result {:?}", vector, other),
    }
}

// ── unwrap_err_or_else helper ─────────────────────────────────────────────────

trait ResultExt<T, E> {
    fn unwrap_err_or_else(self, f: impl FnOnce() -> E) -> E;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn unwrap_err_or_else(self, f: impl FnOnce() -> E) -> E {
        match self {
            Err(e) => e,
            Ok(_) => f(),
        }
    }
}

// ── Vector tests ──────────────────────────────────────────────────────────────

/// v01 — A correctly generated receipt with its original inputs must verify
/// without error.
#[test]
fn verifier_vector_v01_valid_receipt() {
    run_verifier_vector("v01_valid_receipt");
}

/// v02 — A receipt where `routing_decision_hash` has been replaced with a
/// wrong value (and `receipt_hash` recomputed to match) must fail with
/// `routing_decision_hash_mismatch`.  The receipt_canonicalization check
/// passes; the decision hash check fires.
#[test]
fn verifier_vector_v02_tampered_routing_decision_hash() {
    run_verifier_vector("v02_tampered_routing_decision_hash");
}

/// v03 — A valid receipt verified against a policy bundle where the snapshot
/// evidence references have been changed must fail with
/// `registry_snapshot_hash_mismatch`.
#[test]
fn verifier_vector_v03_tampered_registry_snapshot_hash() {
    run_verifier_vector("v03_tampered_registry_snapshot_hash");
}

/// v04 — A valid receipt verified against a policy bundle that has an extra
/// phantom candidate (no corresponding snapshot) must fail with
/// `candidate_pool_hash_mismatch`.  The snapshot hash is unaffected; only
/// the candidate pool commitment differs.
#[test]
fn verifier_vector_v04_tampered_candidate_pool_hash() {
    run_verifier_vector("v04_tampered_candidate_pool_hash");
}

/// v05 — A receipt where `receipt_hash` has been replaced with a wrong value
/// must fail with `receipt_canonicalization_mismatch` — the very first
/// semantic check.
#[test]
fn verifier_vector_v05_tampered_receipt_hash() {
    run_verifier_vector("v05_tampered_receipt_hash");
}

// ── Cross-vector invariants ───────────────────────────────────────────────────

/// All 5 vectors must be stable: running each vector twice produces the same
/// verification outcome both times.
#[test]
fn all_verifier_vectors_are_stable() {
    let vectors = [
        "v01_valid_receipt",
        "v02_tampered_routing_decision_hash",
        "v03_tampered_registry_snapshot_hash",
        "v04_tampered_candidate_pool_hash",
        "v05_tampered_receipt_hash",
    ];

    for vector in vectors {
        let meta = read_meta(vector);
        generate_if_missing(vector, &meta);

        let receipt_json = read_vv_file(vector, "receipt.json");
        let case_json = read_vv_file(vector, "case.json");
        let policy_json = read_vv_file(vector, "policy.json");

        let r1 = verify_receipt_from_policy_json(&receipt_json, &case_json, &policy_json);
        let r2 = verify_receipt_from_policy_json(&receipt_json, &case_json, &policy_json);

        match (r1, r2) {
            (Ok(()), Ok(())) => {}
            (Err(f1), Err(f2)) => assert_eq!(
                f1.code, f2.code,
                "vector '{}': error code differs between two identical calls",
                vector
            ),
            _ => panic!(
                "vector '{}': verification result differs between two identical calls",
                vector
            ),
        }
    }
}
