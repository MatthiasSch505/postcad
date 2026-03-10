//! CLI library for the `route-case` pipeline.
//!
//! # Module layout
//!
//! - [`receipt`] — public receipt contract (no internal type deps; stable API)
//! - [`policy_bundle`] — stable public artifact schema for the policy bundle
//! - This file — case input DTO, error types, pipeline entry points, mapping
//!   layer, field parsers, and tests
//!
//! # Mapping layer
//!
//! [`map_result_to_receipt`] is the single translation point between internal
//! routing/audit domain types and the public [`RoutingReceipt`] contract. All
//! knowledge of `RoutingServiceResult`, `RoutingDecision`, `AuditLog`, etc.
//! lives in that function and the helpers it calls. The [`RoutingReceipt`] type
//! itself has no dependency on internal crates.

pub mod receipt;
pub use receipt::{RECEIPT_SCHEMA_VERSION, ReceiptVerificationResult, RefusalDetail, RoutingReceipt};

pub mod policy_bundle;
pub use policy_bundle::{CandidateEntry, RoutingPolicyBundle, SnapshotEntry};

mod candidate_snapshot;
use candidate_snapshot::{hash_candidate_pool, hash_eligible_ids, hash_selector_input};

pub mod verifier;
pub use verifier::VerificationFailure;

// ── Internal imports (used only by the mapping layer and pipeline helpers) ───

use postcad_audit::{AuditEvent, AuditLog, RoutingServiceResult, route_case_with_compliance_audit};
use postcad_core::{
    Case, CaseId, Country, DentalCase, FileType, ManufacturerEligibility, ManufacturingLocation,
    Material, ProcedureType, RoutingCandidate, RoutingCandidateId, RoutingDecision, RoutingPolicy,
    RoutingPolicyConfig, fingerprint_case, fingerprint_policy,
};
use postcad_registry::snapshot::ManufacturerComplianceSnapshot;
use postcad_registry::validate_snapshots;

// ── External imports (shared by DTOs, errors, and tests) ─────────────────────

use serde::Deserialize;
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

// ── Input DTOs ────────────────────────────────────────────────────────────────

/// JSON representation of a dental case read from `--case`.
///
/// `case_id` is optional; a fresh UUID is generated when absent.
/// `jurisdiction` defaults to `"global"`.
/// `routing_policy` defaults to `"allow_domestic_and_cross_border"`.
#[derive(Debug, Deserialize)]
pub struct CaseInput {
    pub case_id: Option<String>,
    pub jurisdiction: Option<String>,
    pub routing_policy: Option<String>,
    pub patient_country: String,
    pub manufacturer_country: String,
    pub material: String,
    pub procedure: String,
    pub file_type: String,
}

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum CliError {
    #[error("JSON parse error: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("invalid field value: {0}")]
    InvalidField(String),
    #[error("snapshot validation failed: {0}")]
    SnapshotValidation(String),
}

impl CliError {
    /// Stable machine-readable code for this error, used in the JSON envelope.
    pub fn code(&self) -> &'static str {
        match self {
            CliError::ParseError(_) => "parse_error",
            CliError::InvalidField(_) => "parse_error",
            CliError::SnapshotValidation(_) => "invalid_snapshot",
        }
    }
}

// ── Public pipeline entry point ───────────────────────────────────────────────

/// Runs the compliance-aware routing pipeline from raw JSON strings and returns
/// the public [`RoutingReceipt`].
///
/// This is the testable pure core of the `route-case` command. No file I/O,
/// no timestamps, no randomness. Internal domain types are consumed here and
/// translated into the stable receipt contract via [`map_result_to_receipt`].
pub fn route_case_from_json(
    case_json: &str,
    candidates_json: &str,
    snapshots_json: &str,
) -> Result<RoutingReceipt, CliError> {
    let case_input: CaseInput = serde_json::from_str(case_json)?;
    let candidates_input: Vec<CandidateEntry> = serde_json::from_str(candidates_json)?;
    let snapshots_input: Vec<SnapshotEntry> = serde_json::from_str(snapshots_json)?;

    let case = build_case(&case_input)?;
    let candidates = build_candidates(&candidates_input)?;
    let snapshots = build_snapshots(&snapshots_input);
    let policy = parse_routing_policy(case_input.routing_policy.as_deref())?;
    let jurisdiction = case_input.jurisdiction.as_deref().unwrap_or("global");

    validate_snapshots(&snapshots)
        .map_err(|e| CliError::SnapshotValidation(e.to_string()))?;

    // Bind the candidate snapshot before compliance filtering so the hash
    // covers exactly the pool presented at routing time.
    let candidate_pool_hash = hash_candidate_pool(&candidates_input);

    // Capture input candidate IDs before compliance filtering so that refusal
    // detail can report the full evaluated set.
    let all_input_candidate_ids: Vec<String> =
        candidates.iter().map(|c| c.id.0.clone()).collect();

    let case_fingerprint = fingerprint_case(&case);

    let result = route_case_with_compliance_audit(
        &case,
        jurisdiction,
        policy,
        &candidates,
        &snapshots,
        None, // no policy_version in the three-file path
    );

    Ok(map_result_to_receipt(result, case_fingerprint, all_input_candidate_ids, candidate_pool_hash))
}

/// Runs the compliance-aware routing pipeline from a case JSON and a policy
/// bundle JSON, returning the public [`RoutingReceipt`].
///
/// This is the policy-based complement to [`route_case_from_json`]. Routing
/// configuration (jurisdiction, routing policy, candidates, snapshots) all come
/// from the policy bundle; only the case data comes from `case_json`.
///
/// Symmetric with [`verify_receipt_from_policy_json`]: both functions accept the
/// same `(case.json, policy.json)` pair, making them the natural entry points
/// for the public contract matrix.
pub fn route_case_from_policy_json(
    case_json: &str,
    policy_json: &str,
) -> Result<RoutingReceipt, CliError> {
    let case_input: CaseInput = serde_json::from_str(case_json)?;
    let policy_input: RoutingPolicyBundle = serde_json::from_str(policy_json)?;

    let case = build_case(&case_input)?;
    let candidates = build_candidates(&policy_input.candidates)?;
    let snapshots = build_snapshots(&policy_input.snapshots);
    let policy = parse_routing_policy(policy_input.routing_policy.as_deref())?;
    let jurisdiction = policy_input.jurisdiction.as_deref().unwrap_or("global");

    validate_snapshots(&snapshots)
        .map_err(|e| CliError::SnapshotValidation(e.to_string()))?;

    let candidate_pool_hash = hash_candidate_pool(&policy_input.candidates);

    let all_input_candidate_ids: Vec<String> =
        candidates.iter().map(|c| c.id.0.clone()).collect();

    let case_fingerprint = fingerprint_case(&case);

    let result = route_case_with_compliance_audit(
        &case,
        jurisdiction,
        policy,
        &candidates,
        &snapshots,
        policy_input.policy_version.clone(),
    );

    Ok(map_result_to_receipt(result, case_fingerprint, all_input_candidate_ids, candidate_pool_hash))
}

/// Verifies a routing receipt against the original inputs that produced it.
///
/// Re-runs the pipeline with the same inputs, then compares the recomputed
/// receipt against the provided one field-by-field. Returns `"valid"` when all
/// fields match; `"mismatch"` with a list of differing field names otherwise.
///
/// The receipt JSON must be the output of a previous `route-case --json` run.
/// Any parse or pipeline error is returned as a [`CliError`].
pub fn verify_receipt_from_json(
    receipt_json: &str,
    case_json: &str,
    candidates_json: &str,
    snapshots_json: &str,
) -> Result<ReceiptVerificationResult, CliError> {
    let provided: RoutingReceipt = serde_json::from_str(receipt_json)?;
    let recomputed = route_case_from_json(case_json, candidates_json, snapshots_json)?;

    let mut mismatched: Vec<String> = Vec::new();

    macro_rules! check {
        ($field:ident) => {
            if provided.$field != recomputed.$field {
                mismatched.push(stringify!($field).to_string());
            }
        };
    }

    check!(outcome);
    check!(case_fingerprint);
    check!(policy_fingerprint);
    check!(policy_version);
    check!(routing_proof_hash);
    check!(registry_snapshot_hash);
    check!(candidate_pool_hash);
    check!(eligible_candidate_ids_hash);
    check!(selection_input_candidate_ids_hash);
    check!(selected_candidate_id);
    check!(refusal_code);
    check!(audit_seq);
    check!(audit_entry_hash);
    check!(audit_previous_hash);
    check!(refusal);
    check!(receipt_hash);

    if mismatched.is_empty() {
        Ok(ReceiptVerificationResult { result: "valid".to_string(), mismatched_fields: None })
    } else {
        Ok(ReceiptVerificationResult {
            result: "mismatch".to_string(),
            mismatched_fields: Some(mismatched),
        })
    }
}

/// Computes the canonical artifact-integrity hash for a receipt.
///
/// Serializes the full receipt to a `serde_json::Value` (which produces an
/// alphabetically ordered JSON object via `serde_json`'s BTreeMap), removes the
/// `receipt_hash` field to avoid circular hashing, serializes to a compact JSON
/// string, and returns `SHA-256(bytes)` as a 64-character lowercase hex string.
///
/// This path is used identically during generation (in [`finalize_receipt`]) and
/// during verification, guaranteeing the same canonical form in both directions.
fn hash_receipt_content(receipt: &RoutingReceipt) -> String {
    let mut obj =
        serde_json::to_value(receipt).expect("RoutingReceipt serialization must not fail");
    obj.as_object_mut()
        .expect("RoutingReceipt must serialize as a JSON object")
        .remove("receipt_hash");
    let canonical =
        serde_json::to_string(&obj).expect("canonical receipt serialization must not fail");
    let digest = Sha256::digest(canonical.as_bytes());
    format!("{:x}", digest)
}

/// Computes `receipt_hash` and stamps it onto the receipt before returning it.
///
/// Must be the last operation in `map_result_to_receipt` so that the hash covers
/// all other fields in their final state.
fn finalize_receipt(mut receipt: RoutingReceipt) -> RoutingReceipt {
    receipt.receipt_hash = hash_receipt_content(&receipt);
    receipt
}

/// Validates the `schema_version` field in a raw receipt JSON value.
///
/// Must be called before deserializing into [`RoutingReceipt`] so that missing
/// or unsupported versions produce their specific stable failure codes rather
/// than a generic parse error.
fn check_receipt_schema_version(raw: &serde_json::Value) -> Result<(), VerificationFailure> {
    match raw.get("schema_version") {
        None => Err(VerificationFailure::missing_receipt_schema_version()),
        Some(v) if v.is_null() => Err(VerificationFailure::missing_receipt_schema_version()),
        Some(v) => match v.as_str() {
            None => Err(VerificationFailure::invalid_receipt_schema_version()),
            Some(s) if s == RECEIPT_SCHEMA_VERSION => Ok(()),
            Some(s) => Err(VerificationFailure::unsupported_receipt_schema_version(s)),
        },
    }
}

/// Verifies a routing receipt step-by-step against a case and policy document.
///
/// Returns `Ok(())` when all checks pass (`VERIFIED`).
/// Returns `Err(reason)` at the first failing check (`VERIFICATION FAILED`).
///
/// Verification order:
/// 1. Receipt JSON parses into the public schema.
/// 2. `case_fingerprint` matches the provided case input.
/// 3. `policy_fingerprint` matches the provided policy input.
/// 4–6. Recomputed routing proof hash matches `routing_proof_hash`.
/// 7. Audit entry hash and previous hash match the chain linkage fields.
pub fn verify_receipt_from_policy_json(
    receipt_json: &str,
    case_json: &str,
    policy_json: &str,
) -> Result<(), VerificationFailure> {
    // Step 0: parse raw JSON, then validate schema_version before full deserialization.
    let raw: serde_json::Value = serde_json::from_str(receipt_json)
        .map_err(|e| VerificationFailure::receipt_parse_failed(e.to_string()))?;
    check_receipt_schema_version(&raw)?;

    // Step 1: deserialize as v1 receipt.
    let receipt: RoutingReceipt = serde_json::from_value(raw)
        .map_err(|e| VerificationFailure::receipt_parse_failed(e.to_string()))?;

    // Step 1b: validate full artifact receipt_hash before any semantic check.
    let computed_receipt_hash = hash_receipt_content(&receipt);
    if computed_receipt_hash != receipt.receipt_hash {
        return Err(VerificationFailure::receipt_hash_mismatch(
            &receipt.receipt_hash,
            &computed_receipt_hash,
        ));
    }

    // Step 2: build case, compute case_fingerprint.
    let case_input: CaseInput = serde_json::from_str(case_json)
        .map_err(|e| VerificationFailure::case_parse_failed(e.to_string()))?;
    let case = build_case(&case_input)
        .map_err(|e| VerificationFailure::case_parse_failed(e.to_string()))?;
    let computed_case_fp = fingerprint_case(&case);
    if computed_case_fp != receipt.case_fingerprint {
        return Err(VerificationFailure::case_fingerprint_mismatch(
            &receipt.case_fingerprint,
            &computed_case_fp,
        ));
    }

    // Step 3: build policy config, compute policy_fingerprint.
    let policy_input: RoutingPolicyBundle = serde_json::from_str(policy_json)
        .map_err(|e| VerificationFailure::policy_bundle_parse_failed(e.to_string()))?;
    let routing_policy = parse_routing_policy(policy_input.routing_policy.as_deref())
        .map_err(|e| VerificationFailure::policy_bundle_parse_failed(e.to_string()))?;
    let policy_config = match &policy_input.compliance_profile {
        Some(p) => RoutingPolicyConfig::new(routing_policy.clone()).with_compliance_profile(p),
        None => RoutingPolicyConfig::new(routing_policy.clone()),
    };
    let computed_policy_fp = fingerprint_policy(&policy_config);
    if computed_policy_fp != receipt.policy_fingerprint {
        return Err(VerificationFailure::policy_fingerprint_mismatch(
            &receipt.policy_fingerprint,
            &computed_policy_fp,
        ));
    }

    // Step 3c: verify policy_version matches the receipt commitment.
    if policy_input.policy_version != receipt.policy_version {
        return Err(VerificationFailure::policy_version_mismatch(
            receipt.policy_version.as_deref().unwrap_or("(none)"),
            policy_input.policy_version.as_deref().unwrap_or("(none)"),
        ));
    }

    // Step 3b: verify candidate_pool_hash from policy's embedded candidates.
    let computed_candidate_hash = hash_candidate_pool(&policy_input.candidates);
    if computed_candidate_hash != receipt.candidate_pool_hash {
        return Err(VerificationFailure::candidate_pool_hash_mismatch(
            &receipt.candidate_pool_hash,
            &computed_candidate_hash,
        ));
    }

    // Steps 4–6: re-run routing kernel, verify proof hash.
    let jurisdiction = policy_input.jurisdiction.as_deref().unwrap_or("global");
    let candidates = build_candidates(&policy_input.candidates)
        .map_err(|e| VerificationFailure::policy_bundle_parse_failed(e.to_string()))?;
    let snapshots = build_snapshots(&policy_input.snapshots);
    validate_snapshots(&snapshots)
        .map_err(|e| VerificationFailure::policy_bundle_parse_failed(e.to_string()))?;

    let result = route_case_with_compliance_audit(
        &case,
        jurisdiction,
        routing_policy,
        &candidates,
        &snapshots,
        policy_input.policy_version.clone(),
    );

    if result.proof.hash_hex != receipt.routing_proof_hash {
        return Err(VerificationFailure::routing_proof_hash_mismatch(
            &receipt.routing_proof_hash,
            &result.proof.hash_hex,
        ));
    }

    // Step 4b: verify eligible_candidate_ids_hash.
    let computed_eligible_hash =
        hash_eligible_ids(&result.decision_trace.eligible_candidate_ids);
    if computed_eligible_hash != receipt.eligible_candidate_ids_hash {
        return Err(VerificationFailure::eligible_candidate_ids_hash_mismatch(
            &receipt.eligible_candidate_ids_hash,
            &computed_eligible_hash,
        ));
    }

    // Step 4c: verify selection_input_candidate_ids_hash.
    let computed_selector_hash =
        hash_selector_input(&result.decision_trace.eligible_candidate_ids);
    if computed_selector_hash != receipt.selection_input_candidate_ids_hash {
        return Err(VerificationFailure::selection_input_candidate_ids_hash_mismatch(
            &receipt.selection_input_candidate_ids_hash,
            &computed_selector_hash,
        ));
    }

    // Step 7: verify audit chain linkage by reconstructing the AuditEvent.
    let case_id_str = case.id.0.to_string();
    let audit_event = match &result.outcome.decision {
        RoutingDecision::Selected(selected_id) => AuditEvent::CaseRouted {
            case_id: case_id_str,
            proof_hash: result.proof.hash_hex.clone(),
            selected_candidate_id: selected_id.0.clone(),
        },
        RoutingDecision::Refused(_) | RoutingDecision::NoEligibleCandidate => {
            let refusal_code = receipt.refusal_code.clone().unwrap_or_else(|| "unknown".to_string());
            AuditEvent::CaseRefused {
                case_id: case_id_str,
                proof_hash: result.proof.hash_hex.clone(),
                refusal_code,
            }
        }
    };
    let mut log = AuditLog::new();
    let entry = log.append(audit_event);

    if entry.hash != receipt.audit_entry_hash {
        return Err(VerificationFailure::audit_entry_hash_mismatch(
            &receipt.audit_entry_hash,
            &entry.hash,
        ));
    }
    if entry.previous_hash != receipt.audit_previous_hash {
        return Err(VerificationFailure::audit_previous_hash_mismatch(
            &receipt.audit_previous_hash,
            &entry.previous_hash,
        ));
    }

    Ok(())
}

/// Verifies a routing receipt against four separate input artifacts.
///
/// This is the four-artifact verification path: `receipt.json`, `case.json`,
/// `policy.json` (jurisdiction + routing policy + snapshots, **no candidates**),
/// and `candidates.json` (the frozen candidate pool).
///
/// Verification order:
/// 1. Receipt JSON parses into the public schema.
/// 2. `case_fingerprint` matches the provided case input.
/// 3. `candidate_pool_hash` matches SHA-256(canonical(candidates)).
/// 4. `policy_fingerprint` matches the provided policy input.
/// 5–6. Recomputed routing proof hash matches `routing_proof_hash`.
/// 7. Audit entry hash and previous hash match the chain linkage fields.
pub fn verify_receipt_from_inputs(
    receipt_json: &str,
    case_json: &str,
    policy_json: &str,
    candidates_json: &str,
) -> Result<(), VerificationFailure> {
    // Step 0: parse raw JSON, then validate schema_version before full deserialization.
    let raw: serde_json::Value = serde_json::from_str(receipt_json)
        .map_err(|e| VerificationFailure::receipt_parse_failed(e.to_string()))?;
    check_receipt_schema_version(&raw)?;

    // Step 1: deserialize as v1 receipt.
    let receipt: RoutingReceipt = serde_json::from_value(raw)
        .map_err(|e| VerificationFailure::receipt_parse_failed(e.to_string()))?;

    // Step 1b: validate full artifact receipt_hash before any semantic check.
    let computed_receipt_hash = hash_receipt_content(&receipt);
    if computed_receipt_hash != receipt.receipt_hash {
        return Err(VerificationFailure::receipt_hash_mismatch(
            &receipt.receipt_hash,
            &computed_receipt_hash,
        ));
    }

    // Step 2: build case, compute case_fingerprint.
    let case_input: CaseInput = serde_json::from_str(case_json)
        .map_err(|e| VerificationFailure::case_parse_failed(e.to_string()))?;
    let case = build_case(&case_input)
        .map_err(|e| VerificationFailure::case_parse_failed(e.to_string()))?;
    let computed_case_fp = fingerprint_case(&case);
    if computed_case_fp != receipt.case_fingerprint {
        return Err(VerificationFailure::case_fingerprint_mismatch(
            &receipt.case_fingerprint,
            &computed_case_fp,
        ));
    }

    // Step 3: parse candidates, verify candidate_pool_hash.
    let candidates_input: Vec<CandidateEntry> = serde_json::from_str(candidates_json)
        .map_err(|e| VerificationFailure::policy_bundle_parse_failed(e.to_string()))?;
    let computed_candidate_hash = hash_candidate_pool(&candidates_input);
    if computed_candidate_hash != receipt.candidate_pool_hash {
        return Err(VerificationFailure::candidate_pool_hash_mismatch(
            &receipt.candidate_pool_hash,
            &computed_candidate_hash,
        ));
    }

    // Step 4: build policy config, compute policy_fingerprint.
    let policy_input: RoutingPolicyBundle = serde_json::from_str(policy_json)
        .map_err(|e| VerificationFailure::policy_bundle_parse_failed(e.to_string()))?;
    let routing_policy = parse_routing_policy(policy_input.routing_policy.as_deref())
        .map_err(|e| VerificationFailure::policy_bundle_parse_failed(e.to_string()))?;
    let policy_config = match &policy_input.compliance_profile {
        Some(p) => RoutingPolicyConfig::new(routing_policy.clone()).with_compliance_profile(p),
        None => RoutingPolicyConfig::new(routing_policy.clone()),
    };
    let computed_policy_fp = fingerprint_policy(&policy_config);
    if computed_policy_fp != receipt.policy_fingerprint {
        return Err(VerificationFailure::policy_fingerprint_mismatch(
            &receipt.policy_fingerprint,
            &computed_policy_fp,
        ));
    }

    // Step 4b: verify policy_version matches the receipt commitment.
    if policy_input.policy_version != receipt.policy_version {
        return Err(VerificationFailure::policy_version_mismatch(
            receipt.policy_version.as_deref().unwrap_or("(none)"),
            policy_input.policy_version.as_deref().unwrap_or("(none)"),
        ));
    }

    // Steps 5–6: re-run routing with candidates from candidates.json + snapshots
    // from policy.json, verify proof hash.
    let jurisdiction = policy_input.jurisdiction.as_deref().unwrap_or("global");
    let candidates = build_candidates(&candidates_input)
        .map_err(|e| VerificationFailure::policy_bundle_parse_failed(e.to_string()))?;
    let snapshots = build_snapshots(&policy_input.snapshots);
    validate_snapshots(&snapshots)
        .map_err(|e| VerificationFailure::policy_bundle_parse_failed(e.to_string()))?;

    let result = route_case_with_compliance_audit(
        &case,
        jurisdiction,
        routing_policy,
        &candidates,
        &snapshots,
        policy_input.policy_version.clone(),
    );

    if result.proof.hash_hex != receipt.routing_proof_hash {
        return Err(VerificationFailure::routing_proof_hash_mismatch(
            &receipt.routing_proof_hash,
            &result.proof.hash_hex,
        ));
    }

    // Step 5b: verify eligible_candidate_ids_hash.
    let computed_eligible_hash =
        hash_eligible_ids(&result.decision_trace.eligible_candidate_ids);
    if computed_eligible_hash != receipt.eligible_candidate_ids_hash {
        return Err(VerificationFailure::eligible_candidate_ids_hash_mismatch(
            &receipt.eligible_candidate_ids_hash,
            &computed_eligible_hash,
        ));
    }

    // Step 5c: verify selection_input_candidate_ids_hash.
    let computed_selector_hash =
        hash_selector_input(&result.decision_trace.eligible_candidate_ids);
    if computed_selector_hash != receipt.selection_input_candidate_ids_hash {
        return Err(VerificationFailure::selection_input_candidate_ids_hash_mismatch(
            &receipt.selection_input_candidate_ids_hash,
            &computed_selector_hash,
        ));
    }

    // Step 7: verify audit chain linkage.
    let case_id_str = case.id.0.to_string();
    let audit_event = match &result.outcome.decision {
        RoutingDecision::Selected(selected_id) => AuditEvent::CaseRouted {
            case_id: case_id_str,
            proof_hash: result.proof.hash_hex.clone(),
            selected_candidate_id: selected_id.0.clone(),
        },
        RoutingDecision::Refused(_) | RoutingDecision::NoEligibleCandidate => {
            let refusal_code = receipt.refusal_code.clone().unwrap_or_else(|| "unknown".to_string());
            AuditEvent::CaseRefused {
                case_id: case_id_str,
                proof_hash: result.proof.hash_hex.clone(),
                refusal_code,
            }
        }
    };
    let mut log = AuditLog::new();
    let entry = log.append(audit_event);

    if entry.hash != receipt.audit_entry_hash {
        return Err(VerificationFailure::audit_entry_hash_mismatch(
            &receipt.audit_entry_hash,
            &entry.hash,
        ));
    }
    if entry.previous_hash != receipt.audit_previous_hash {
        return Err(VerificationFailure::audit_previous_hash_mismatch(
            &receipt.audit_previous_hash,
            &entry.previous_hash,
        ));
    }

    Ok(())
}

// ── Mapping layer ─────────────────────────────────────────────────────────────

/// Translates an internal [`RoutingServiceResult`] into the public
/// [`RoutingReceipt`] contract.
///
/// This function is the single point of coupling between the internal routing
/// and audit domain types and the external CLI output schema. All field
/// extraction from internal types happens here; nothing internal leaks into
/// the returned value.
///
/// An `AuditLog` entry is appended to generate the chain linkage fields. The
/// log is local to this call (seq always starts at 0), producing a
/// self-contained, independently verifiable audit anchor per invocation.
fn map_result_to_receipt(
    result: RoutingServiceResult,
    case_fingerprint: String,
    all_input_candidate_ids: Vec<String>,
    candidate_pool_hash: String,
) -> RoutingReceipt {
    // Extract primitive values from internal types before the match so they
    // are not accidentally referenced after `result` is partially moved.
    let routing_proof_hash: String = result.proof.hash_hex.clone();
    let policy_fingerprint: String = result.policy_fingerprint.clone();
    let policy_version: Option<String> = result.audit_receipt.policy_version.clone();
    let case_id_str: String = result.audit_receipt.case_id.clone();
    let compliant_candidate_count: usize =
        result.audit_receipt.candidate_ids_considered.len();
    let registry_snapshot_hash: String = result
        .audit_receipt
        .registry_snapshot_hash
        .clone()
        .unwrap_or_default();
    let eligible_candidate_ids_hash: String =
        hash_eligible_ids(&result.decision_trace.eligible_candidate_ids);
    let selection_input_candidate_ids_hash: String =
        hash_selector_input(&result.decision_trace.eligible_candidate_ids);

    match result.outcome.decision {
        RoutingDecision::Selected(selected_id) => {
            let selected_candidate_id = selected_id.0.clone();

            let audit_event = AuditEvent::CaseRouted {
                case_id: case_id_str,
                proof_hash: routing_proof_hash.clone(),
                selected_candidate_id: selected_candidate_id.clone(),
            };
            let (audit_seq, audit_entry_hash, audit_previous_hash) =
                append_audit_entry(audit_event);

            finalize_receipt(RoutingReceipt {
                schema_version: RECEIPT_SCHEMA_VERSION.to_string(),
                outcome: "routed".to_string(),
                case_fingerprint,
                policy_fingerprint,
                policy_version,
                routing_proof_hash,
                registry_snapshot_hash,
                candidate_pool_hash,
                eligible_candidate_ids_hash,
                selection_input_candidate_ids_hash,
                selected_candidate_id: Some(selected_candidate_id),
                refusal_code: None,
                audit_seq,
                audit_entry_hash,
                audit_previous_hash,
                refusal: None,
                receipt_hash: String::new(),
            })
        }

        RoutingDecision::Refused(case_refusal) => {
            let (refusal_code, message) = extract_refusal_fields(&case_refusal);
            let failed_constraint = refusal_code_to_constraint(&refusal_code);

            let audit_event = AuditEvent::CaseRefused {
                case_id: case_id_str,
                proof_hash: routing_proof_hash.clone(),
                refusal_code: refusal_code.clone(),
            };
            let (audit_seq, audit_entry_hash, audit_previous_hash) =
                append_audit_entry(audit_event);

            finalize_receipt(RoutingReceipt {
                schema_version: RECEIPT_SCHEMA_VERSION.to_string(),
                outcome: "refused".to_string(),
                case_fingerprint,
                policy_fingerprint,
                policy_version,
                routing_proof_hash,
                registry_snapshot_hash,
                candidate_pool_hash,
                eligible_candidate_ids_hash,
                selection_input_candidate_ids_hash: selection_input_candidate_ids_hash.clone(),
                selected_candidate_id: None,
                refusal_code: Some(refusal_code),
                audit_seq,
                audit_entry_hash,
                audit_previous_hash,
                refusal: Some(RefusalDetail {
                    message,
                    evaluated_candidate_ids: all_input_candidate_ids,
                    failed_constraint,
                }),
                receipt_hash: String::new(),
            })
        }

        RoutingDecision::NoEligibleCandidate => {
            let refusal_code = "no_eligible_candidates".to_string();
            let failed_constraint = no_candidate_constraint(
                all_input_candidate_ids.len(),
                compliant_candidate_count,
            );

            let audit_event = AuditEvent::CaseRefused {
                case_id: case_id_str,
                proof_hash: routing_proof_hash.clone(),
                refusal_code: refusal_code.clone(),
            };
            let (audit_seq, audit_entry_hash, audit_previous_hash) =
                append_audit_entry(audit_event);

            finalize_receipt(RoutingReceipt {
                schema_version: RECEIPT_SCHEMA_VERSION.to_string(),
                outcome: "refused".to_string(),
                case_fingerprint,
                policy_fingerprint,
                policy_version,
                routing_proof_hash,
                registry_snapshot_hash,
                candidate_pool_hash,
                eligible_candidate_ids_hash,
                selection_input_candidate_ids_hash,
                selected_candidate_id: None,
                refusal_code: Some(refusal_code),
                audit_seq,
                audit_entry_hash,
                audit_previous_hash,
                refusal: Some(RefusalDetail {
                    message: "No eligible candidate found".to_string(),
                    evaluated_candidate_ids: all_input_candidate_ids,
                    failed_constraint,
                }),
                receipt_hash: String::new(),
            })
        }
    }
}

/// Appends one event to a fresh `AuditLog` and returns the resulting
/// `(seq, entry_hash, previous_hash)` triple as owned `String`/`u64` values.
///
/// The log is local to this call; callers receive only the primitive fields
/// they need for the receipt, not the `AuditEntry` or `AuditLog` types.
fn append_audit_entry(event: AuditEvent) -> (u64, String, String) {
    let mut log = AuditLog::new();
    let entry = log.append(event);
    (entry.seq, entry.hash.clone(), entry.previous_hash.clone())
}

/// Extracts `(code, message)` strings from a `CaseRefusal`.
///
/// Falls back to stable sentinel strings when no reason is recorded so the
/// receipt always carries non-empty fields.
fn extract_refusal_fields(
    refusal: &postcad_core::CaseRefusal,
) -> (String, String) {
    refusal
        .reasons
        .first()
        .map(|r| (r.code().to_string(), r.message().to_string()))
        .unwrap_or_else(|| {
            ("unknown".to_string(), "Unknown refusal reason".to_string())
        })
}

/// Maps a stable refusal code string to the `failed_constraint` label used in
/// [`RefusalDetail`].
fn refusal_code_to_constraint(code: &str) -> String {
    match code {
        "invalid_input" | "unsupported_case" => "case_validation",
        "compliance_failed" => "compliance_gate",
        _ => "unknown",
    }
    .to_string()
}

/// Derives the `failed_constraint` label for a `NoEligibleCandidate` outcome
/// based on how many candidates were present before and after compliance
/// filtering.
fn no_candidate_constraint(input_count: usize, compliant_count: usize) -> String {
    if input_count == 0 {
        "no_input_candidates"
    } else if compliant_count == 0 {
        "compliance_gate"
    } else {
        "routing_policy"
    }
    .to_string()
}

// ── Input construction helpers ────────────────────────────────────────────────

fn build_case(input: &CaseInput) -> Result<Case, CliError> {
    let dental_case = DentalCase {
        patient_country: parse_country(&input.patient_country)?,
        manufacturer_country: parse_country(&input.manufacturer_country)?,
        material: parse_material(&input.material)?,
        procedure: parse_procedure(&input.procedure)?,
        file_type: parse_file_type(&input.file_type)?,
    };

    let case_id = match &input.case_id {
        Some(s) => CaseId(
            Uuid::parse_str(s)
                .map_err(|e| CliError::InvalidField(format!("invalid case_id UUID: {}", e)))?,
        ),
        None => CaseId::new(),
    };

    Ok(Case {
        id: case_id,
        dental_case,
        created_at: chrono::Utc::now(),
    })
}

fn build_candidates(inputs: &[CandidateEntry]) -> Result<Vec<RoutingCandidate>, CliError> {
    let mut seen_ids = std::collections::HashSet::new();
    let mut result = Vec::with_capacity(inputs.len());

    for c in inputs {
        if c.id.is_empty() {
            return Err(CliError::InvalidField(
                "candidate id must not be empty".to_string(),
            ));
        }
        if c.manufacturer_id.is_empty() {
            return Err(CliError::InvalidField(
                "candidate manufacturer_id must not be empty".to_string(),
            ));
        }
        if !seen_ids.insert(c.id.as_str()) {
            return Err(CliError::InvalidField(format!(
                "duplicate candidate id: {}",
                c.id
            )));
        }
        result.push(RoutingCandidate::new(
            RoutingCandidateId::new(c.id.as_str()),
            c.manufacturer_id.as_str(),
            parse_location(&c.location)?,
            c.accepts_case,
            parse_eligibility(&c.eligibility)?,
        ));
    }

    Ok(result)
}

fn build_snapshots(inputs: &[SnapshotEntry]) -> Vec<ManufacturerComplianceSnapshot> {
    inputs
        .iter()
        .map(|s| {
            ManufacturerComplianceSnapshot::new(
                s.manufacturer_id.as_str(),
                s.evidence_references.clone(),
                s.attestation_statuses.clone(),
                s.is_eligible,
            )
        })
        .collect()
}

// ── Field parsers ─────────────────────────────────────────────────────────────

fn parse_country(s: &str) -> Result<Country, CliError> {
    match s {
        "united_states" => Ok(Country::UnitedStates),
        "germany" => Ok(Country::Germany),
        "france" => Ok(Country::France),
        "japan" => Ok(Country::Japan),
        "united_kingdom" => Ok(Country::UnitedKingdom),
        other if other.starts_with("other:") => Ok(Country::Other(other[6..].to_string())),
        _ => Err(CliError::InvalidField(format!("unknown country: {}", s))),
    }
}

fn parse_material(s: &str) -> Result<Material, CliError> {
    match s {
        "zirconia" => Ok(Material::Zirconia),
        "pmma" => Ok(Material::Pmma),
        "emax" => Ok(Material::Emax),
        "cobalt_chrome" => Ok(Material::CobaltChrome),
        "titanium" => Ok(Material::Titanium),
        other if other.starts_with("other:") => Ok(Material::Other(other[6..].to_string())),
        _ => Err(CliError::InvalidField(format!("unknown material: {}", s))),
    }
}

fn parse_procedure(s: &str) -> Result<ProcedureType, CliError> {
    match s {
        "crown" => Ok(ProcedureType::Crown),
        "bridge" => Ok(ProcedureType::Bridge),
        "veneer" => Ok(ProcedureType::Veneer),
        "implant" => Ok(ProcedureType::Implant),
        "denture" => Ok(ProcedureType::Denture),
        other if other.starts_with("other:") => Ok(ProcedureType::Other(other[6..].to_string())),
        _ => Err(CliError::InvalidField(format!("unknown procedure: {}", s))),
    }
}

fn parse_file_type(s: &str) -> Result<FileType, CliError> {
    match s {
        "stl" => Ok(FileType::Stl),
        "obj" => Ok(FileType::Obj),
        "ply" => Ok(FileType::Ply),
        "three_mf" => Ok(FileType::ThreeMf),
        other if other.starts_with("other:") => Ok(FileType::Other(other[6..].to_string())),
        _ => Err(CliError::InvalidField(format!("unknown file_type: {}", s))),
    }
}

fn parse_location(s: &str) -> Result<ManufacturingLocation, CliError> {
    match s {
        "domestic" => Ok(ManufacturingLocation::Domestic),
        "cross_border" => Ok(ManufacturingLocation::CrossBorder),
        "unknown" => Ok(ManufacturingLocation::Unknown),
        _ => Err(CliError::InvalidField(format!("unknown location: {}", s))),
    }
}

fn parse_eligibility(s: &str) -> Result<ManufacturerEligibility, CliError> {
    match s {
        "eligible" => Ok(ManufacturerEligibility::Eligible),
        "ineligible" => Ok(ManufacturerEligibility::Ineligible),
        "unknown" => Ok(ManufacturerEligibility::Unknown),
        _ => Err(CliError::InvalidField(format!("unknown eligibility: {}", s))),
    }
}

fn parse_routing_policy(s: Option<&str>) -> Result<RoutingPolicy, CliError> {
    match s.unwrap_or("allow_domestic_and_cross_border") {
        "allow_domestic_only" => Ok(RoutingPolicy::AllowDomesticOnly),
        "allow_domestic_and_cross_border" => Ok(RoutingPolicy::AllowDomesticAndCrossBorder),
        other => Err(CliError::InvalidField(format!("unknown routing_policy: {}", other))),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Fixtures ──────────────────────────────────────────────────────────────

    const CASE_JSON: &str = r#"{
        "case_id": "00000000-0000-0000-0000-000000000001",
        "jurisdiction": "DE",
        "patient_country": "united_states",
        "manufacturer_country": "germany",
        "material": "zirconia",
        "procedure": "crown",
        "file_type": "stl"
    }"#;

    const CANDIDATES_JSON: &str = r#"[
        {
            "id": "rc-1",
            "manufacturer_id": "mfr-01",
            "location": "domestic",
            "accepts_case": true,
            "eligibility": "eligible"
        }
    ]"#;

    const ELIGIBLE_SNAPSHOTS_JSON: &str = r#"[
        {
            "manufacturer_id": "mfr-01",
            "evidence_references": ["REF-001"],
            "attestation_statuses": ["verified"],
            "is_eligible": true
        }
    ]"#;

    const INELIGIBLE_SNAPSHOTS_JSON: &str = r#"[
        {
            "manufacturer_id": "mfr-01",
            "evidence_references": ["REF-001"],
            "attestation_statuses": ["rejected"],
            "is_eligible": false
        }
    ]"#;

    const EMPTY_SNAPSHOTS_JSON: &str = r#"[]"#;

    // ── Test 1: successful route ───────────────────────────────────────────────

    #[test]
    fn routed_case_returns_selected_outcome() {
        let receipt = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON)
            .expect("routing should succeed");

        assert_eq!(receipt.outcome, "routed");
        assert_eq!(receipt.selected_candidate_id, Some("rc-1".to_string()));
        assert!(receipt.refusal_code.is_none());
        assert!(receipt.refusal.is_none());
    }

    #[test]
    fn routed_case_proof_hash_is_64_hex_chars() {
        let receipt = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();

        assert_eq!(receipt.routing_proof_hash.len(), 64);
        assert!(receipt.routing_proof_hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn routed_case_policy_fingerprint_is_64_hex_chars() {
        let receipt = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();

        assert_eq!(receipt.policy_fingerprint.len(), 64);
    }

    #[test]
    fn routed_case_case_fingerprint_is_64_hex_chars() {
        let receipt = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();

        assert_eq!(receipt.case_fingerprint.len(), 64);
    }

    #[test]
    fn routed_case_with_fixed_case_id_is_deterministic() {
        let a = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let b = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();

        assert_eq!(a.case_fingerprint, b.case_fingerprint);
        assert_eq!(a.policy_fingerprint, b.policy_fingerprint);
        assert_eq!(a.routing_proof_hash, b.routing_proof_hash);
        assert_eq!(a.selected_candidate_id, b.selected_candidate_id);
    }

    // ── Test 2: refusal — ineligible snapshot ─────────────────────────────────

    #[test]
    fn refused_case_ineligible_snapshot_returns_refused_outcome() {
        let receipt = route_case_from_json(CASE_JSON, CANDIDATES_JSON, INELIGIBLE_SNAPSHOTS_JSON)
            .expect("parse should succeed");

        assert_eq!(receipt.outcome, "refused");
        assert!(receipt.selected_candidate_id.is_none());
        assert_eq!(receipt.refusal_code.as_deref(), Some("no_eligible_candidates"));
    }

    #[test]
    fn refused_case_no_snapshots_returns_refused_outcome() {
        let receipt = route_case_from_json(CASE_JSON, CANDIDATES_JSON, EMPTY_SNAPSHOTS_JSON)
            .expect("parse should succeed");

        assert_eq!(receipt.outcome, "refused");
        assert!(receipt.selected_candidate_id.is_none());
        assert!(receipt.refusal_code.is_some());
    }

    #[test]
    fn refused_case_proof_hash_is_present() {
        let receipt = route_case_from_json(CASE_JSON, CANDIDATES_JSON, INELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();

        assert_eq!(receipt.routing_proof_hash.len(), 64);
    }

    #[test]
    fn refused_and_routed_produce_different_proof_hashes() {
        let routed =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let refused =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, INELIGIBLE_SNAPSHOTS_JSON).unwrap();

        assert_ne!(routed.routing_proof_hash, refused.routing_proof_hash);
    }

    // ── Test 3: parse failures ────────────────────────────────────────────────

    #[test]
    fn invalid_case_json_returns_parse_error() {
        let result = route_case_from_json(r#"{"not": "valid json"#, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON);
        assert!(matches!(result, Err(CliError::ParseError(_))));
    }

    #[test]
    fn invalid_candidates_json_returns_parse_error() {
        let result = route_case_from_json(CASE_JSON, r#"[{"broken"#, ELIGIBLE_SNAPSHOTS_JSON);
        assert!(matches!(result, Err(CliError::ParseError(_))));
    }

    #[test]
    fn invalid_snapshots_json_returns_parse_error() {
        let result = route_case_from_json(CASE_JSON, CANDIDATES_JSON, r#"not json at all"#);
        assert!(matches!(result, Err(CliError::ParseError(_))));
    }

    #[test]
    fn unknown_country_returns_invalid_field_error() {
        let bad_case = r#"{
            "case_id": "00000000-0000-0000-0000-000000000002",
            "patient_country": "atlantis",
            "manufacturer_country": "germany",
            "material": "zirconia",
            "procedure": "crown",
            "file_type": "stl"
        }"#;
        let result = route_case_from_json(bad_case, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON);
        assert!(matches!(result, Err(CliError::InvalidField(_))));
    }

    #[test]
    fn unknown_material_returns_invalid_field_error() {
        let bad_case = r#"{
            "case_id": "00000000-0000-0000-0000-000000000003",
            "patient_country": "united_states",
            "manufacturer_country": "germany",
            "material": "unobtainium",
            "procedure": "crown",
            "file_type": "stl"
        }"#;
        let result = route_case_from_json(bad_case, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON);
        assert!(matches!(result, Err(CliError::InvalidField(_))));
    }

    #[test]
    fn unknown_location_returns_invalid_field_error() {
        let bad_candidates = r#"[{
            "id": "rc-1",
            "manufacturer_id": "mfr-01",
            "location": "orbit",
            "accepts_case": true,
            "eligibility": "eligible"
        }]"#;
        let result = route_case_from_json(CASE_JSON, bad_candidates, ELIGIBLE_SNAPSHOTS_JSON);
        assert!(matches!(result, Err(CliError::InvalidField(_))));
    }

    // ── CandidateEntry boundary validation ────────────────────────────────────

    #[test]
    fn candidate_with_unknown_eligibility_returns_invalid_field_error() {
        let bad_candidates = r#"[{
            "id": "rc-1",
            "manufacturer_id": "mfr-01",
            "location": "domestic",
            "accepts_case": true,
            "eligibility": "maybe"
        }]"#;
        let result = route_case_from_json(CASE_JSON, bad_candidates, ELIGIBLE_SNAPSHOTS_JSON);
        assert!(matches!(result, Err(CliError::InvalidField(_))));
    }

    #[test]
    fn candidate_with_empty_id_returns_invalid_field_error() {
        let bad_candidates = r#"[{
            "id": "",
            "manufacturer_id": "mfr-01",
            "location": "domestic",
            "accepts_case": true,
            "eligibility": "eligible"
        }]"#;
        let result = route_case_from_json(CASE_JSON, bad_candidates, ELIGIBLE_SNAPSHOTS_JSON);
        assert!(matches!(result, Err(CliError::InvalidField(_))));
    }

    #[test]
    fn candidate_with_empty_manufacturer_id_returns_invalid_field_error() {
        let bad_candidates = r#"[{
            "id": "rc-1",
            "manufacturer_id": "",
            "location": "domestic",
            "accepts_case": true,
            "eligibility": "eligible"
        }]"#;
        let result = route_case_from_json(CASE_JSON, bad_candidates, ELIGIBLE_SNAPSHOTS_JSON);
        assert!(matches!(result, Err(CliError::InvalidField(_))));
    }

    #[test]
    fn candidate_with_duplicate_id_returns_invalid_field_error() {
        let bad_candidates = r#"[
            {"id":"rc-1","manufacturer_id":"mfr-01","location":"domestic","accepts_case":true,"eligibility":"eligible"},
            {"id":"rc-1","manufacturer_id":"mfr-02","location":"domestic","accepts_case":true,"eligibility":"eligible"}
        ]"#;
        let result = route_case_from_json(CASE_JSON, bad_candidates, ELIGIBLE_SNAPSHOTS_JSON);
        assert!(matches!(result, Err(CliError::InvalidField(_))));
    }

    #[test]
    fn candidates_with_same_manufacturer_id_but_different_ids_are_accepted() {
        // Multiple candidates for the same manufacturer (e.g. domestic + cross-border)
        // is valid; only duplicate candidate *ids* are rejected.
        let candidates = r#"[
            {"id":"rc-1","manufacturer_id":"mfr-01","location":"domestic","accepts_case":true,"eligibility":"eligible"},
            {"id":"rc-2","manufacturer_id":"mfr-01","location":"cross_border","accepts_case":true,"eligibility":"eligible"}
        ]"#;
        let snapshots = r#"[
            {"manufacturer_id":"mfr-01","evidence_references":["REF-A"],"attestation_statuses":["verified"],"is_eligible":true}
        ]"#;
        let result = route_case_from_json(CASE_JSON, candidates, snapshots);
        assert!(result.is_ok());
    }

    // ── SnapshotEntry attestation-status boundary behavior ────────────────────

    #[test]
    fn snapshot_with_unknown_attestation_status_and_ineligible_flag_is_accepted() {
        // Unknown status strings are not parsed by the validator; as long as
        // is_eligible=false the snapshot passes boundary checks.
        let snapshots = r#"[
            {"manufacturer_id":"mfr-01","evidence_references":["REF-001"],"attestation_statuses":["pending"],"is_eligible":false}
        ]"#;
        let result = route_case_from_json(CASE_JSON, CANDIDATES_JSON, snapshots);
        // Snapshot itself is structurally valid; routing outcome is a refusal
        // because the snapshot is ineligible.
        assert!(result.is_ok());
        let receipt = result.unwrap();
        assert_eq!(receipt.outcome, "refused");
    }

    #[test]
    fn snapshot_eligible_with_unknown_attestation_status_only_is_rejected() {
        // An unknown status is not "verified", so an eligible snapshot that has
        // only unknown statuses fails the EligibleWithNoVerifiedAttestation check.
        let snapshots = r#"[
            {"manufacturer_id":"mfr-01","evidence_references":["REF-001"],"attestation_statuses":["pending"],"is_eligible":true}
        ]"#;
        let result = route_case_from_json(CASE_JSON, CANDIDATES_JSON, snapshots);
        assert!(matches!(result, Err(CliError::SnapshotValidation(_))));
    }

    // ── Test 4: field parsing ─────────────────────────────────────────────────

    #[test]
    fn other_prefix_country_is_accepted() {
        let case_json = r#"{
            "case_id": "00000000-0000-0000-0000-000000000004",
            "patient_country": "other:Brazil",
            "manufacturer_country": "germany",
            "material": "zirconia",
            "procedure": "crown",
            "file_type": "stl"
        }"#;
        let result = route_case_from_json(case_json, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON);
        assert!(result.is_ok());
    }

    #[test]
    fn allow_domestic_only_policy_routes_domestic_candidate() {
        let case_json = r#"{
            "case_id": "00000000-0000-0000-0000-000000000005",
            "routing_policy": "allow_domestic_only",
            "patient_country": "united_states",
            "manufacturer_country": "germany",
            "material": "zirconia",
            "procedure": "crown",
            "file_type": "stl"
        }"#;
        let receipt = route_case_from_json(case_json, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();

        assert_eq!(receipt.outcome, "routed");
        assert_eq!(receipt.selected_candidate_id, Some("rc-1".to_string()));
    }

    #[test]
    fn missing_case_id_generates_new_uuid_and_routes() {
        let case_no_id = r#"{
            "patient_country": "united_states",
            "manufacturer_country": "germany",
            "material": "zirconia",
            "procedure": "crown",
            "file_type": "stl"
        }"#;
        let receipt = route_case_from_json(case_no_id, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();

        assert_eq!(receipt.outcome, "routed");
        assert_eq!(receipt.case_fingerprint.len(), 64);
    }

    #[test]
    fn multi_candidate_eligible_selects_first_in_order() {
        let candidates = r#"[
            {"id":"rc-1","manufacturer_id":"mfr-01","location":"domestic","accepts_case":true,"eligibility":"eligible"},
            {"id":"rc-2","manufacturer_id":"mfr-02","location":"domestic","accepts_case":true,"eligibility":"eligible"}
        ]"#;
        let snapshots = r#"[
            {"manufacturer_id":"mfr-01","evidence_references":["REF-A"],"attestation_statuses":["verified"],"is_eligible":true},
            {"manufacturer_id":"mfr-02","evidence_references":["REF-B"],"attestation_statuses":["verified"],"is_eligible":true}
        ]"#;
        let receipt = route_case_from_json(CASE_JSON, candidates, snapshots).unwrap();

        assert_eq!(receipt.selected_candidate_id, Some("rc-1".to_string()));
    }

    // ── Test 5: snapshot validation ───────────────────────────────────────────

    #[test]
    fn snapshot_with_duplicate_manufacturer_id_returns_error() {
        let snapshots = r#"[
            {"manufacturer_id":"mfr-01","evidence_references":["REF-001"],"attestation_statuses":["verified"],"is_eligible":true},
            {"manufacturer_id":"mfr-01","evidence_references":["REF-002"],"attestation_statuses":["verified"],"is_eligible":true}
        ]"#;
        let result = route_case_from_json(CASE_JSON, CANDIDATES_JSON, snapshots);
        assert!(matches!(result, Err(CliError::SnapshotValidation(_))));
    }

    #[test]
    fn snapshot_eligible_with_no_evidence_returns_error() {
        let snapshots = r#"[
            {"manufacturer_id":"mfr-01","evidence_references":[],"attestation_statuses":[],"is_eligible":true}
        ]"#;
        let result = route_case_from_json(CASE_JSON, CANDIDATES_JSON, snapshots);
        assert!(matches!(result, Err(CliError::SnapshotValidation(_))));
    }

    #[test]
    fn snapshot_eligible_with_only_rejected_attestation_returns_error() {
        let snapshots = r#"[
            {"manufacturer_id":"mfr-01","evidence_references":["REF-001"],"attestation_statuses":["rejected"],"is_eligible":true}
        ]"#;
        let result = route_case_from_json(CASE_JSON, CANDIDATES_JSON, snapshots);
        assert!(matches!(result, Err(CliError::SnapshotValidation(_))));
    }

    #[test]
    fn snapshot_with_duplicate_evidence_reference_returns_error() {
        let snapshots = r#"[
            {"manufacturer_id":"mfr-01","evidence_references":["REF-001","REF-001"],"attestation_statuses":["verified"],"is_eligible":true}
        ]"#;
        let result = route_case_from_json(CASE_JSON, CANDIDATES_JSON, snapshots);
        assert!(matches!(result, Err(CliError::SnapshotValidation(_))));
    }

    #[test]
    fn snapshot_with_more_attestations_than_evidence_returns_error() {
        let snapshots = r#"[
            {"manufacturer_id":"mfr-01","evidence_references":["REF-001"],"attestation_statuses":["verified","verified"],"is_eligible":true}
        ]"#;
        let result = route_case_from_json(CASE_JSON, CANDIDATES_JSON, snapshots);
        assert!(matches!(result, Err(CliError::SnapshotValidation(_))));
    }

    #[test]
    fn snapshot_with_empty_manufacturer_id_returns_error() {
        let snapshots = r#"[
            {"manufacturer_id":"","evidence_references":[],"attestation_statuses":[],"is_eligible":false}
        ]"#;
        let result = route_case_from_json(CASE_JSON, CANDIDATES_JSON, snapshots);
        assert!(matches!(result, Err(CliError::SnapshotValidation(_))));
    }

    #[test]
    fn valid_snapshot_does_not_return_validation_error() {
        let result =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON);
        assert!(result.is_ok());
    }

    #[test]
    fn first_candidate_ineligible_selects_second() {
        let candidates = r#"[
            {"id":"rc-1","manufacturer_id":"mfr-01","location":"domestic","accepts_case":true,"eligibility":"eligible"},
            {"id":"rc-2","manufacturer_id":"mfr-02","location":"domestic","accepts_case":true,"eligibility":"eligible"}
        ]"#;
        let snapshots = r#"[
            {"manufacturer_id":"mfr-01","evidence_references":["REF-A"],"attestation_statuses":["rejected"],"is_eligible":false},
            {"manufacturer_id":"mfr-02","evidence_references":["REF-B"],"attestation_statuses":["verified"],"is_eligible":true}
        ]"#;
        let receipt = route_case_from_json(CASE_JSON, candidates, snapshots).unwrap();

        assert_eq!(receipt.selected_candidate_id, Some("rc-2".to_string()));
    }

    // ── Test 6: refusal detail ────────────────────────────────────────────────

    #[test]
    fn refused_output_includes_refusal_detail() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, INELIGIBLE_SNAPSHOTS_JSON).unwrap();

        assert_eq!(receipt.refusal_code.as_deref(), Some("no_eligible_candidates"));
        let detail = receipt.refusal.expect("refusal detail must be present");
        assert!(!detail.evaluated_candidate_ids.is_empty());
        assert!(!detail.failed_constraint.is_empty());
    }

    #[test]
    fn refused_detail_evaluated_candidate_ids_are_deterministic() {
        let a = route_case_from_json(CASE_JSON, CANDIDATES_JSON, INELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();
        let b = route_case_from_json(CASE_JSON, CANDIDATES_JSON, INELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();
        assert_eq!(
            a.refusal.unwrap().evaluated_candidate_ids,
            b.refusal.unwrap().evaluated_candidate_ids,
        );
    }

    #[test]
    fn refused_detail_evaluated_candidate_ids_match_input_candidates() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, INELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let detail = receipt.refusal.unwrap();
        assert_eq!(detail.evaluated_candidate_ids, vec!["rc-1"]);
    }

    #[test]
    fn refused_detail_failed_constraint_is_compliance_gate_when_snapshot_rejects() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, INELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let detail = receipt.refusal.unwrap();
        assert_eq!(detail.failed_constraint, "compliance_gate");
    }

    #[test]
    fn refused_detail_failed_constraint_is_compliance_gate_when_no_snapshot() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, EMPTY_SNAPSHOTS_JSON).unwrap();
        let detail = receipt.refusal.unwrap();
        assert_eq!(detail.failed_constraint, "compliance_gate");
    }

    #[test]
    fn refused_detail_failed_constraint_is_no_input_candidates_when_empty_list() {
        let empty_candidates = r#"[]"#;
        let receipt =
            route_case_from_json(CASE_JSON, empty_candidates, EMPTY_SNAPSHOTS_JSON).unwrap();
        let detail = receipt.refusal.unwrap();
        assert_eq!(detail.evaluated_candidate_ids, Vec::<String>::new());
        assert_eq!(detail.failed_constraint, "no_input_candidates");
    }

    #[test]
    fn routed_output_has_no_refusal_code_and_no_refusal_detail() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        assert!(receipt.refusal_code.is_none());
        assert!(receipt.refusal.is_none());
    }

    #[test]
    fn multi_candidate_refused_detail_lists_all_input_candidates() {
        let candidates = r#"[
            {"id":"rc-1","manufacturer_id":"mfr-01","location":"domestic","accepts_case":true,"eligibility":"eligible"},
            {"id":"rc-2","manufacturer_id":"mfr-02","location":"domestic","accepts_case":true,"eligibility":"eligible"}
        ]"#;
        let snapshots = r#"[
            {"manufacturer_id":"mfr-01","evidence_references":["REF-A"],"attestation_statuses":["rejected"],"is_eligible":false},
            {"manufacturer_id":"mfr-02","evidence_references":["REF-B"],"attestation_statuses":["rejected"],"is_eligible":false}
        ]"#;
        let receipt = route_case_from_json(CASE_JSON, candidates, snapshots).unwrap();
        let detail = receipt.refusal.unwrap();
        assert_eq!(detail.evaluated_candidate_ids, vec!["rc-1", "rc-2"]);
        assert_eq!(detail.failed_constraint, "compliance_gate");
    }

    // ── Test 7: audit chain linkage ───────────────────────────────────────────

    #[test]
    fn routed_receipt_audit_entry_hash_is_64_hex_chars() {
        let receipt = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();

        assert_eq!(receipt.audit_entry_hash.len(), 64);
        assert!(receipt.audit_entry_hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn refused_receipt_audit_entry_hash_is_64_hex_chars() {
        let receipt = route_case_from_json(CASE_JSON, CANDIDATES_JSON, INELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();

        assert_eq!(receipt.audit_entry_hash.len(), 64);
        assert!(receipt.audit_entry_hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn receipt_audit_seq_is_zero() {
        let routed = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();
        let refused = route_case_from_json(CASE_JSON, CANDIDATES_JSON, INELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();

        assert_eq!(routed.audit_seq, 0);
        assert_eq!(refused.audit_seq, 0);
    }

    #[test]
    fn receipt_audit_previous_hash_is_genesis() {
        let receipt = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();

        assert_eq!(
            receipt.audit_previous_hash,
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn routed_receipt_audit_entry_hash_is_deterministic_for_fixed_case_id() {
        let a = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let b = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();

        assert_eq!(a.audit_entry_hash, b.audit_entry_hash);
        assert_eq!(a.audit_previous_hash, b.audit_previous_hash);
        assert_eq!(a.audit_seq, b.audit_seq);
    }

    #[test]
    fn refused_receipt_audit_entry_hash_is_deterministic_for_fixed_case_id() {
        let a = route_case_from_json(CASE_JSON, CANDIDATES_JSON, INELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let b = route_case_from_json(CASE_JSON, CANDIDATES_JSON, INELIGIBLE_SNAPSHOTS_JSON).unwrap();

        assert_eq!(a.audit_entry_hash, b.audit_entry_hash);
    }

    #[test]
    fn routed_and_refused_produce_different_audit_entry_hashes() {
        let routed =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let refused =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, INELIGIBLE_SNAPSHOTS_JSON).unwrap();

        assert_ne!(routed.audit_entry_hash, refused.audit_entry_hash);
    }

    // ── Test 8: verify_receipt_from_policy_json ──────────────────────────────

    // CASE_JSON has no routing_policy → defaults to allow_domestic_and_cross_border.
    // POLICY_JSON must use the same policy so policy_fingerprints match.
    const POLICY_JSON: &str = r#"{
        "jurisdiction": "DE",
        "routing_policy": "allow_domestic_and_cross_border",
        "candidates": [{
            "id": "rc-1",
            "manufacturer_id": "mfr-01",
            "location": "domestic",
            "accepts_case": true,
            "eligibility": "eligible"
        }],
        "snapshots": [{
            "manufacturer_id": "mfr-01",
            "evidence_references": ["REF-001"],
            "attestation_statuses": ["verified"],
            "is_eligible": true
        }]
    }"#;

    const POLICY_INELIGIBLE_JSON: &str = r#"{
        "jurisdiction": "DE",
        "routing_policy": "allow_domestic_and_cross_border",
        "candidates": [{
            "id": "rc-1",
            "manufacturer_id": "mfr-01",
            "location": "domestic",
            "accepts_case": true,
            "eligibility": "eligible"
        }],
        "snapshots": [{
            "manufacturer_id": "mfr-01",
            "evidence_references": ["REF-001"],
            "attestation_statuses": ["rejected"],
            "is_eligible": false
        }]
    }"#;

    #[test]
    fn verify_receipt_from_policy_json_valid_for_routed_case() {
        let receipt = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();

        verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_JSON)
            .expect("should be VERIFIED");
    }

    #[test]
    fn verify_receipt_from_policy_json_valid_for_refused_case() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, INELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();

        verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_INELIGIBLE_JSON)
            .expect("should be VERIFIED");
    }

    /// Recomputes `receipt_hash` so that the artifact-integrity check passes and
    /// the specific field-level check is exercised. Used by tamper tests that
    /// want to reach past step 1b and into the semantic verification steps.
    fn tamper_field_recompute_hash(receipt: RoutingReceipt) -> String {
        let mut r = receipt;
        r.receipt_hash = hash_receipt_content(&r);
        serde_json::to_string(&r).unwrap()
    }

    #[test]
    fn verify_receipt_from_policy_json_fails_on_tampered_case_fingerprint() {
        let mut receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        receipt.case_fingerprint =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        let receipt_json = tamper_field_recompute_hash(receipt);

        let err = verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_JSON)
            .unwrap_err();
        assert_eq!(err.code, "case_fingerprint_mismatch");
        assert!(err.message.contains("case_fingerprint mismatch"), "got: {}", err.message);
    }

    #[test]
    fn verify_receipt_from_policy_json_fails_on_tampered_policy_fingerprint() {
        let mut receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        receipt.policy_fingerprint =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        let receipt_json = tamper_field_recompute_hash(receipt);

        let err = verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_JSON)
            .unwrap_err();
        assert_eq!(err.code, "policy_fingerprint_mismatch");
        assert!(err.message.contains("policy_fingerprint mismatch"), "got: {}", err.message);
    }

    #[test]
    fn verify_receipt_from_policy_json_fails_on_tampered_proof_hash() {
        let mut receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        receipt.routing_proof_hash =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        let receipt_json = tamper_field_recompute_hash(receipt);

        let err = verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_JSON)
            .unwrap_err();
        assert_eq!(err.code, "routing_proof_hash_mismatch");
        assert!(err.message.contains("routing_proof_hash mismatch"), "got: {}", err.message);
    }

    #[test]
    fn verify_receipt_from_policy_json_fails_on_tampered_audit_entry_hash() {
        let mut receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        receipt.audit_entry_hash =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        let receipt_json = tamper_field_recompute_hash(receipt);

        let err = verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_JSON)
            .unwrap_err();
        assert_eq!(err.code, "audit_entry_hash_mismatch");
        assert!(err.message.contains("audit_entry_hash mismatch"), "got: {}", err.message);
    }

    #[test]
    fn verify_receipt_from_policy_json_fails_on_tampered_audit_previous_hash() {
        let mut receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        receipt.audit_previous_hash =
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string();
        let receipt_json = tamper_field_recompute_hash(receipt);

        let err = verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_JSON)
            .unwrap_err();
        assert_eq!(err.code, "audit_previous_hash_mismatch");
        assert!(err.message.contains("audit_previous_hash mismatch"), "got: {}", err.message);
    }

    #[test]
    fn verify_receipt_from_policy_json_fails_on_invalid_receipt_json() {
        let err = verify_receipt_from_policy_json("{not json", CASE_JSON, POLICY_JSON)
            .unwrap_err();
        assert_eq!(err.code, "receipt_parse_failed");
    }

    #[test]
    fn verify_receipt_from_policy_json_fails_on_invalid_case_json() {
        let receipt = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();

        let err = verify_receipt_from_policy_json(&receipt_json, "{not json", POLICY_JSON)
            .unwrap_err();
        assert_eq!(err.code, "case_parse_failed");
    }

    #[test]
    fn verify_receipt_from_policy_json_fails_on_invalid_policy_json() {
        let receipt = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();

        let err = verify_receipt_from_policy_json(&receipt_json, CASE_JSON, "{not json")
            .unwrap_err();
        assert_eq!(err.code, "policy_bundle_parse_failed");
    }

    #[test]
    fn verify_receipt_from_policy_json_fails_on_wrong_inputs() {
        // Receipt produced with eligible snapshot; verify with ineligible policy.
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();

        let err = verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_INELIGIBLE_JSON)
            .unwrap_err();
        assert!(!err.code.is_empty());
        assert!(!err.message.is_empty());
    }

    // ── Test 9: verify_receipt_from_json (legacy) ─────────────────────────────

    fn receipt_json_for(case: &str, candidates: &str, snapshots: &str) -> String {
        let receipt = route_case_from_json(case, candidates, snapshots).unwrap();
        serde_json::to_string(&receipt).unwrap()
    }

    #[test]
    fn verify_receipt_valid_for_routed_case() {
        let receipt_json = receipt_json_for(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON);
        let result = verify_receipt_from_json(
            &receipt_json,
            CASE_JSON,
            CANDIDATES_JSON,
            ELIGIBLE_SNAPSHOTS_JSON,
        )
        .unwrap();

        assert_eq!(result.result, "valid");
        assert!(result.mismatched_fields.is_none());
    }

    #[test]
    fn verify_receipt_valid_for_refused_case() {
        let receipt_json =
            receipt_json_for(CASE_JSON, CANDIDATES_JSON, INELIGIBLE_SNAPSHOTS_JSON);
        let result = verify_receipt_from_json(
            &receipt_json,
            CASE_JSON,
            CANDIDATES_JSON,
            INELIGIBLE_SNAPSHOTS_JSON,
        )
        .unwrap();

        assert_eq!(result.result, "valid");
        assert!(result.mismatched_fields.is_none());
    }

    #[test]
    fn verify_receipt_mismatch_when_outcome_tampered() {
        let receipt = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();
        // Tamper outcome field in the serialized JSON.
        let tampered = serde_json::to_string(&receipt)
            .unwrap()
            .replace("\"routed\"", "\"refused\"");

        let result = verify_receipt_from_json(
            &tampered,
            CASE_JSON,
            CANDIDATES_JSON,
            ELIGIBLE_SNAPSHOTS_JSON,
        )
        .unwrap();

        assert_eq!(result.result, "mismatch");
        let fields = result.mismatched_fields.unwrap();
        assert!(fields.contains(&"outcome".to_string()));
    }

    #[test]
    fn verify_receipt_mismatch_when_routing_proof_hash_tampered() {
        let mut receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        receipt.routing_proof_hash =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        let tampered = serde_json::to_string(&receipt).unwrap();

        let result = verify_receipt_from_json(
            &tampered,
            CASE_JSON,
            CANDIDATES_JSON,
            ELIGIBLE_SNAPSHOTS_JSON,
        )
        .unwrap();

        assert_eq!(result.result, "mismatch");
        let fields = result.mismatched_fields.unwrap();
        assert!(fields.contains(&"routing_proof_hash".to_string()));
    }

    #[test]
    fn verify_receipt_mismatch_when_wrong_inputs_provided() {
        // Receipt was produced for routed case; verify against refused inputs.
        let receipt_json = receipt_json_for(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON);
        let result = verify_receipt_from_json(
            &receipt_json,
            CASE_JSON,
            CANDIDATES_JSON,
            INELIGIBLE_SNAPSHOTS_JSON,
        )
        .unwrap();

        assert_eq!(result.result, "mismatch");
        assert!(result.mismatched_fields.is_some());
    }

    #[test]
    fn verify_receipt_parse_error_on_invalid_receipt_json() {
        let result =
            verify_receipt_from_json(r#"not json"#, CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON);
        assert!(matches!(result, Err(CliError::ParseError(_))));
    }

    // ── Test 10: candidate_pool_hash ──────────────────────────────────────

    #[test]
    fn receipt_includes_candidate_pool_hash_as_64_hex_chars() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();

        assert_eq!(receipt.candidate_pool_hash.len(), 64);
        assert!(receipt.candidate_pool_hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn candidate_pool_hash_is_deterministic_for_same_input() {
        let a = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let b = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();

        assert_eq!(a.candidate_pool_hash, b.candidate_pool_hash);
    }

    #[test]
    fn different_candidates_produce_different_candidate_pool_hash() {
        let other_candidates = r#"[
            {"id":"rc-99","manufacturer_id":"mfr-99","location":"domestic","accepts_case":true,"eligibility":"eligible"}
        ]"#;
        let other_snapshots = r#"[
            {"manufacturer_id":"mfr-99","evidence_references":["REF-X"],"attestation_statuses":["verified"],"is_eligible":true}
        ]"#;
        let a = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let b = route_case_from_json(CASE_JSON, other_candidates, other_snapshots).unwrap();

        assert_ne!(a.candidate_pool_hash, b.candidate_pool_hash);
    }

    #[test]
    fn empty_candidates_produce_a_stable_hash() {
        let empty_candidates = r#"[]"#;
        let a = route_case_from_json(CASE_JSON, empty_candidates, EMPTY_SNAPSHOTS_JSON).unwrap();
        let b = route_case_from_json(CASE_JSON, empty_candidates, EMPTY_SNAPSHOTS_JSON).unwrap();

        assert_eq!(a.candidate_pool_hash, b.candidate_pool_hash);
        assert_eq!(a.candidate_pool_hash.len(), 64);
    }

    // ── Test 11: verify_receipt_from_inputs ───────────────────────────────────

    #[test]
    fn verify_receipt_from_inputs_passes_for_routed_case() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();

        // policy.json for verify_receipt_from_inputs has NO candidates field.
        let policy_no_candidates = r#"{
            "jurisdiction": "DE",
            "routing_policy": "allow_domestic_and_cross_border",
            "snapshots": [{
                "manufacturer_id": "mfr-01",
                "evidence_references": ["REF-001"],
                "attestation_statuses": ["verified"],
                "is_eligible": true
            }]
        }"#;

        verify_receipt_from_inputs(&receipt_json, CASE_JSON, policy_no_candidates, CANDIDATES_JSON)
            .expect("should be VERIFIED");
    }

    #[test]
    fn verify_receipt_from_inputs_passes_for_refused_case() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, INELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();

        let policy_no_candidates = r#"{
            "jurisdiction": "DE",
            "routing_policy": "allow_domestic_and_cross_border",
            "snapshots": [{
                "manufacturer_id": "mfr-01",
                "evidence_references": ["REF-001"],
                "attestation_statuses": ["rejected"],
                "is_eligible": false
            }]
        }"#;

        verify_receipt_from_inputs(&receipt_json, CASE_JSON, policy_no_candidates, CANDIDATES_JSON)
            .expect("should be VERIFIED");
    }

    #[test]
    fn verify_receipt_from_inputs_fails_with_candidate_pool_hash_mismatch_when_candidates_tampered() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();

        let policy_no_candidates = r#"{
            "jurisdiction": "DE",
            "routing_policy": "allow_domestic_and_cross_border",
            "snapshots": [{
                "manufacturer_id": "mfr-tampered",
                "evidence_references": ["REF-X"],
                "attestation_statuses": ["verified"],
                "is_eligible": true
            }]
        }"#;

        // Provide different candidates than those used when routing.
        let tampered_candidates = r#"[
            {"id":"rc-tampered","manufacturer_id":"mfr-tampered","location":"domestic","accepts_case":true,"eligibility":"eligible"}
        ]"#;

        let err = verify_receipt_from_inputs(
            &receipt_json,
            CASE_JSON,
            policy_no_candidates,
            tampered_candidates,
        )
        .unwrap_err();

        assert_eq!(err.code, "candidate_pool_hash_mismatch");
        assert!(err.message.contains("candidate_pool_hash mismatch"));
    }

    #[test]
    fn verify_receipt_from_policy_json_fails_with_candidate_pool_hash_mismatch_when_policy_candidates_differ() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();

        // Policy with different candidates than those used during routing.
        let policy_different_candidates = r#"{
            "jurisdiction": "DE",
            "routing_policy": "allow_domestic_and_cross_border",
            "candidates": [{
                "id": "rc-different",
                "manufacturer_id": "mfr-02",
                "location": "domestic",
                "accepts_case": true,
                "eligibility": "eligible"
            }],
            "snapshots": [{
                "manufacturer_id": "mfr-02",
                "evidence_references": ["REF-002"],
                "attestation_statuses": ["verified"],
                "is_eligible": true
            }]
        }"#;

        let err =
            verify_receipt_from_policy_json(&receipt_json, CASE_JSON, policy_different_candidates)
                .unwrap_err();

        assert_eq!(err.code, "candidate_pool_hash_mismatch");
    }

    #[test]
    fn verify_receipt_from_inputs_fails_on_invalid_candidates_json() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();

        let err = verify_receipt_from_inputs(
            &receipt_json,
            CASE_JSON,
            POLICY_JSON,
            r#"not json"#,
        )
        .unwrap_err();

        assert_eq!(err.code, "policy_bundle_parse_failed");
    }

    // ── Test 12: candidate_pool_hash verification enforcement ─────────────────
    //
    // These tests verify that independent receipt verification enforces
    // candidate_pool_hash correctly via both verification paths.

    // ── field mutation ────────────────────────────────────────────────────────

    #[test]
    fn verify_from_policy_json_fails_when_candidate_field_mutated() {
        // Receipt was produced with location="domestic". The verifier receives
        // the same candidate id but with location="cross_border". The
        // candidate_pool_hash must detect this mutation before routing reruns.
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();

        let policy_mutated_location = r#"{
            "jurisdiction": "DE",
            "routing_policy": "allow_domestic_and_cross_border",
            "candidates": [{
                "id": "rc-1",
                "manufacturer_id": "mfr-01",
                "location": "cross_border",
                "accepts_case": true,
                "eligibility": "eligible"
            }],
            "snapshots": [{
                "manufacturer_id": "mfr-01",
                "evidence_references": ["REF-001"],
                "attestation_statuses": ["verified"],
                "is_eligible": true
            }]
        }"#;

        let err =
            verify_receipt_from_policy_json(&receipt_json, CASE_JSON, policy_mutated_location)
                .unwrap_err();

        assert_eq!(err.code, "candidate_pool_hash_mismatch");
        assert!(
            err.message.contains("candidate_pool_hash mismatch"),
            "unexpected message: {}",
            err.message
        );
    }

    // ── candidate added ───────────────────────────────────────────────────────

    #[test]
    fn verify_from_inputs_fails_when_candidate_added() {
        // Receipt produced with one candidate; verifier receives that candidate
        // plus an extra one. The expanded pool changes the hash.
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();

        let policy_no_candidates = r#"{
            "jurisdiction": "DE",
            "routing_policy": "allow_domestic_and_cross_border",
            "snapshots": [{
                "manufacturer_id": "mfr-01",
                "evidence_references": ["REF-001"],
                "attestation_statuses": ["verified"],
                "is_eligible": true
            }]
        }"#;
        let candidates_with_extra = r#"[
            {"id":"rc-1","manufacturer_id":"mfr-01","location":"domestic","accepts_case":true,"eligibility":"eligible"},
            {"id":"rc-extra","manufacturer_id":"mfr-extra","location":"domestic","accepts_case":true,"eligibility":"eligible"}
        ]"#;

        let err = verify_receipt_from_inputs(
            &receipt_json,
            CASE_JSON,
            policy_no_candidates,
            candidates_with_extra,
        )
        .unwrap_err();

        assert_eq!(err.code, "candidate_pool_hash_mismatch");
        assert!(
            err.message.contains("candidate_pool_hash mismatch"),
            "unexpected message: {}",
            err.message
        );
    }

    // ── candidate removed ─────────────────────────────────────────────────────

    #[test]
    fn verify_from_policy_json_fails_when_candidate_removed() {
        // Route with two candidates; verify with only one. The shrunken pool
        // changes the hash.
        let two_candidates = r#"[
            {"id":"rc-1","manufacturer_id":"mfr-01","location":"domestic","accepts_case":true,"eligibility":"eligible"},
            {"id":"rc-2","manufacturer_id":"mfr-02","location":"domestic","accepts_case":true,"eligibility":"eligible"}
        ]"#;
        // mfr-02 is ineligible so routing deterministically selects rc-1.
        let two_snapshots = r#"[
            {"manufacturer_id":"mfr-01","evidence_references":["REF-A"],"attestation_statuses":["verified"],"is_eligible":true},
            {"manufacturer_id":"mfr-02","evidence_references":["REF-B"],"attestation_statuses":["rejected"],"is_eligible":false}
        ]"#;

        let receipt = route_case_from_json(CASE_JSON, two_candidates, two_snapshots).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();

        // Policy bundle presents only rc-1, omitting rc-2.
        let policy_one_candidate = r#"{
            "jurisdiction": "DE",
            "routing_policy": "allow_domestic_and_cross_border",
            "candidates": [{
                "id": "rc-1",
                "manufacturer_id": "mfr-01",
                "location": "domestic",
                "accepts_case": true,
                "eligibility": "eligible"
            }],
            "snapshots": [{
                "manufacturer_id": "mfr-01",
                "evidence_references": ["REF-A"],
                "attestation_statuses": ["verified"],
                "is_eligible": true
            }]
        }"#;

        let err =
            verify_receipt_from_policy_json(&receipt_json, CASE_JSON, policy_one_candidate)
                .unwrap_err();

        assert_eq!(err.code, "candidate_pool_hash_mismatch");
        assert!(
            err.message.contains("candidate_pool_hash mismatch"),
            "unexpected message: {}",
            err.message
        );
    }

    // ── order independence ────────────────────────────────────────────────────

    #[test]
    fn verify_from_inputs_succeeds_when_candidate_order_differs() {
        // Route with two candidates, only one eligible (so the routing result —
        // and therefore routing_proof_hash — is the same regardless of input
        // order). Verify with the candidates in reversed order. Because
        // hash_candidate_pool sorts by id before hashing, candidate_pool_hash
        // matches and the full verification still passes.
        let two_candidates = r#"[
            {"id":"rc-1","manufacturer_id":"mfr-01","location":"domestic","accepts_case":true,"eligibility":"eligible"},
            {"id":"rc-2","manufacturer_id":"mfr-02","location":"domestic","accepts_case":true,"eligibility":"eligible"}
        ]"#;
        let snapshots = r#"[
            {"manufacturer_id":"mfr-01","evidence_references":["REF-A"],"attestation_statuses":["verified"],"is_eligible":true},
            {"manufacturer_id":"mfr-02","evidence_references":["REF-B"],"attestation_statuses":["rejected"],"is_eligible":false}
        ]"#;

        let receipt = route_case_from_json(CASE_JSON, two_candidates, snapshots).unwrap();
        assert_eq!(receipt.selected_candidate_id.as_deref(), Some("rc-1"),
            "routing must select rc-1 (only eligible candidate)");
        let receipt_json = serde_json::to_string(&receipt).unwrap();

        // Policy: same two snapshots, NO candidates field (supplied separately).
        let policy_no_candidates = r#"{
            "jurisdiction": "DE",
            "routing_policy": "allow_domestic_and_cross_border",
            "snapshots": [
                {"manufacturer_id":"mfr-01","evidence_references":["REF-A"],"attestation_statuses":["verified"],"is_eligible":true},
                {"manufacturer_id":"mfr-02","evidence_references":["REF-B"],"attestation_statuses":["rejected"],"is_eligible":false}
            ]
        }"#;

        // Candidates presented in reversed order to the verifier.
        let candidates_reversed = r#"[
            {"id":"rc-2","manufacturer_id":"mfr-02","location":"domestic","accepts_case":true,"eligibility":"eligible"},
            {"id":"rc-1","manufacturer_id":"mfr-01","location":"domestic","accepts_case":true,"eligibility":"eligible"}
        ]"#;

        verify_receipt_from_inputs(
            &receipt_json,
            CASE_JSON,
            policy_no_candidates,
            candidates_reversed,
        )
        .expect("verification must succeed: same pool, different order, same hash after canonical sort");
    }

    // ── Test 13: eligible_candidate_ids_hash verification enforcement ──────────

    fn routed_policy_json() -> String {
        r#"{
            "jurisdiction": "DE",
            "routing_policy": "allow_domestic_and_cross_border",
            "candidates": [{"id":"rc-1","manufacturer_id":"mfr-01","location":"domestic","accepts_case":true,"eligibility":"eligible"}],
            "snapshots": [{"manufacturer_id":"mfr-01","evidence_references":["REF-A"],"attestation_statuses":["verified"],"is_eligible":true}]
        }"#.to_string()
    }

    fn refused_policy_json() -> String {
        r#"{
            "jurisdiction": "DE",
            "routing_policy": "allow_domestic_and_cross_border",
            "candidates": [{"id":"rc-1","manufacturer_id":"mfr-01","location":"domestic","accepts_case":true,"eligibility":"eligible"}],
            "snapshots": [{"manufacturer_id":"mfr-01","evidence_references":["REF-A"],"attestation_statuses":["rejected"],"is_eligible":false}]
        }"#.to_string()
    }

    #[test]
    fn eligible_hash_verification_passes_for_routed_case() {
        let policy = routed_policy_json();
        let receipt = route_case_from_policy_json(CASE_JSON, &policy).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();
        verify_receipt_from_policy_json(&receipt_json, CASE_JSON, &policy)
            .expect("verification must pass for routed case with correct eligible_candidate_ids_hash");
    }

    #[test]
    fn eligible_hash_verification_passes_for_refused_case() {
        let policy = refused_policy_json();
        let receipt = route_case_from_policy_json(CASE_JSON, &policy).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();
        // Refused: eligible set is empty; hash must still verify.
        assert!(receipt.eligible_candidate_ids_hash.len() == 64);
        verify_receipt_from_policy_json(&receipt_json, CASE_JSON, &policy)
            .expect("verification must pass for refused case with empty eligible set");
    }

    #[test]
    fn eligible_hash_verification_fails_when_receipt_hash_tampered() {
        let policy = routed_policy_json();
        let receipt = route_case_from_policy_json(CASE_JSON, &policy).unwrap();
        // Tamper the field via Value to avoid accidental multi-replace; then
        // recompute receipt_hash so the artifact check passes and the field
        // check fires.
        let mut val: serde_json::Value = serde_json::to_value(&receipt).unwrap();
        val["eligible_candidate_ids_hash"] =
            serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
        let tampered_r: RoutingReceipt = serde_json::from_value(val.clone()).unwrap();
        val["receipt_hash"] = serde_json::json!(hash_receipt_content(&tampered_r));
        let tampered = serde_json::to_string(&val).unwrap();

        let err = verify_receipt_from_policy_json(&tampered, CASE_JSON, &policy)
            .expect_err("verification must fail when eligible_candidate_ids_hash is tampered");
        assert_eq!(err.code, "eligible_candidate_ids_hash_mismatch");
    }

    #[test]
    fn eligible_hash_is_order_independent() {
        // Route with two candidates; both survive compliance so eligible set is {rc-1, rc-2}.
        // The hash must be the same regardless of the order they appear in DecisionTrace.
        let two_candidates = r#"[
            {"id":"rc-1","manufacturer_id":"mfr-01","location":"domestic","accepts_case":true,"eligibility":"eligible"},
            {"id":"rc-2","manufacturer_id":"mfr-02","location":"domestic","accepts_case":true,"eligibility":"eligible"}
        ]"#;
        let snapshots = r#"[
            {"manufacturer_id":"mfr-01","evidence_references":["REF-A"],"attestation_statuses":["verified"],"is_eligible":true},
            {"manufacturer_id":"mfr-02","evidence_references":["REF-B"],"attestation_statuses":["verified"],"is_eligible":true}
        ]"#;
        let receipt_ab = route_case_from_json(CASE_JSON, two_candidates, snapshots).unwrap();

        let two_candidates_rev = r#"[
            {"id":"rc-2","manufacturer_id":"mfr-02","location":"domestic","accepts_case":true,"eligibility":"eligible"},
            {"id":"rc-1","manufacturer_id":"mfr-01","location":"domestic","accepts_case":true,"eligibility":"eligible"}
        ]"#;
        let receipt_ba = route_case_from_json(CASE_JSON, two_candidates_rev, snapshots).unwrap();

        assert_eq!(
            receipt_ab.eligible_candidate_ids_hash,
            receipt_ba.eligible_candidate_ids_hash,
            "eligible_candidate_ids_hash must be order-independent"
        );
    }

    #[test]
    fn eligible_hash_changes_when_eligibility_flips() {
        // routed case: one candidate survives → hash of ["rc-1"]
        let policy_eligible = routed_policy_json();
        let receipt_eligible = route_case_from_policy_json(CASE_JSON, &policy_eligible).unwrap();

        // refused case: same candidate fails compliance → hash of []
        let policy_refused = refused_policy_json();
        let receipt_refused = route_case_from_policy_json(CASE_JSON, &policy_refused).unwrap();

        assert_ne!(
            receipt_eligible.eligible_candidate_ids_hash,
            receipt_refused.eligible_candidate_ids_hash,
            "eligible_candidate_ids_hash must differ when the eligible set changes"
        );
    }

    // ── Test 14: selection_input_candidate_ids_hash verification enforcement ───

    #[test]
    fn selection_input_hash_verification_passes_for_routed_case() {
        let policy = routed_policy_json();
        let receipt = route_case_from_policy_json(CASE_JSON, &policy).unwrap();
        assert_eq!(receipt.selection_input_candidate_ids_hash.len(), 64);
        let receipt_json = serde_json::to_string(&receipt).unwrap();
        verify_receipt_from_policy_json(&receipt_json, CASE_JSON, &policy)
            .expect("verification must pass for routed case with correct selection_input_candidate_ids_hash");
    }

    #[test]
    fn selection_input_hash_verification_passes_for_refused_case() {
        // Refused case: selector input is empty; hash still verifies.
        let policy = refused_policy_json();
        let receipt = route_case_from_policy_json(CASE_JSON, &policy).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();
        verify_receipt_from_policy_json(&receipt_json, CASE_JSON, &policy)
            .expect("verification must pass for refused case with empty selector input");
    }

    #[test]
    fn selection_input_hash_verification_fails_when_receipt_hash_tampered() {
        let policy = routed_policy_json();
        let receipt = route_case_from_policy_json(CASE_JSON, &policy).unwrap();
        // Tamper the field, then recompute receipt_hash so the artifact check
        // passes and the specific field check fires.
        let mut val: serde_json::Value = serde_json::to_value(&receipt).unwrap();
        val["selection_input_candidate_ids_hash"] =
            serde_json::json!("0000000000000000000000000000000000000000000000000000000000000001");
        let tampered_r: RoutingReceipt = serde_json::from_value(val.clone()).unwrap();
        val["receipt_hash"] = serde_json::json!(hash_receipt_content(&tampered_r));
        let tampered = serde_json::to_string(&val).unwrap();

        let err = verify_receipt_from_policy_json(&tampered, CASE_JSON, &policy)
            .expect_err("verification must fail when selection_input_candidate_ids_hash is tampered");
        assert_eq!(err.code, "selection_input_candidate_ids_hash_mismatch");
    }

    #[test]
    fn selection_input_hash_is_order_sensitive() {
        // Route with two eligible candidates in order [rc-1, rc-2] and then
        // [rc-2, rc-1]. Both selections differ (selector picks first), but
        // we directly compare the selection_input_candidate_ids_hash values to
        // confirm order-sensitivity.
        use candidate_snapshot::hash_selector_input;
        let ids_ab = vec!["rc-1".to_string(), "rc-2".to_string()];
        let ids_ba = vec!["rc-2".to_string(), "rc-1".to_string()];
        assert_ne!(
            hash_selector_input(&ids_ab),
            hash_selector_input(&ids_ba),
            "selection_input hash must differ for different candidate orderings"
        );
    }

    #[test]
    fn selection_input_hash_preserved_when_single_eligible_candidate_routed() {
        // A pool with multiple input candidates but only one compliant (and thus
        // eligible) candidate. The selector sees exactly one candidate; the
        // selection_input_candidate_ids_hash encodes that singleton list.
        let two_candidates = r#"[
            {"id":"rc-1","manufacturer_id":"mfr-01","location":"domestic","accepts_case":true,"eligibility":"eligible"},
            {"id":"rc-2","manufacturer_id":"mfr-02","location":"domestic","accepts_case":true,"eligibility":"eligible"}
        ]"#;
        let snapshots_one_eligible = r#"[
            {"manufacturer_id":"mfr-01","evidence_references":["REF-A"],"attestation_statuses":["verified"],"is_eligible":true},
            {"manufacturer_id":"mfr-02","evidence_references":["REF-B"],"attestation_statuses":["rejected"],"is_eligible":false}
        ]"#;

        let receipt = route_case_from_json(CASE_JSON, two_candidates, snapshots_one_eligible).unwrap();
        assert_eq!(receipt.selected_candidate_id.as_deref(), Some("rc-1"));
        // candidate_pool_hash covers both candidates; selection_input covers only rc-1.
        assert_ne!(
            receipt.candidate_pool_hash,
            receipt.selection_input_candidate_ids_hash,
            "pool hash and selector hash must differ when only a subset is eligible"
        );

        // Verify round-trip.
        let policy_no_candidates = r#"{
            "jurisdiction": "DE",
            "routing_policy": "allow_domestic_and_cross_border",
            "snapshots": [
                {"manufacturer_id":"mfr-01","evidence_references":["REF-A"],"attestation_statuses":["verified"],"is_eligible":true},
                {"manufacturer_id":"mfr-02","evidence_references":["REF-B"],"attestation_statuses":["rejected"],"is_eligible":false}
            ]
        }"#;
        let receipt_json = serde_json::to_string(&receipt).unwrap();
        verify_receipt_from_inputs(&receipt_json, CASE_JSON, policy_no_candidates, two_candidates)
            .expect("verification must pass for receipt with single eligible out of multi-candidate pool");
    }

    // ── Test 15: receipt schema versioning ────────────────────────────────────

    #[test]
    fn route_case_receipt_includes_schema_version_1() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        assert_eq!(receipt.schema_version, RECEIPT_SCHEMA_VERSION);
        assert_eq!(receipt.schema_version, "1");
    }

    #[test]
    fn verify_receipt_accepts_valid_v1_schema_version() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();
        verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_JSON)
            .expect("valid v1 receipt must verify");
    }

    #[test]
    fn verify_receipt_rejects_missing_schema_version() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let mut raw: serde_json::Value = serde_json::to_value(&receipt).unwrap();
        raw.as_object_mut().unwrap().remove("schema_version");
        let receipt_json = serde_json::to_string(&raw).unwrap();

        let err = verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_JSON)
            .unwrap_err();
        assert_eq!(err.code, "missing_receipt_schema_version");
    }

    #[test]
    fn verify_receipt_rejects_null_schema_version() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let mut raw: serde_json::Value = serde_json::to_value(&receipt).unwrap();
        raw["schema_version"] = serde_json::Value::Null;
        let receipt_json = serde_json::to_string(&raw).unwrap();

        let err = verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_JSON)
            .unwrap_err();
        assert_eq!(err.code, "missing_receipt_schema_version");
    }

    #[test]
    fn verify_receipt_rejects_invalid_schema_version_type() {
        // schema_version is a number, not a string — malformed.
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let mut raw: serde_json::Value = serde_json::to_value(&receipt).unwrap();
        raw["schema_version"] = serde_json::json!(1);
        let receipt_json = serde_json::to_string(&raw).unwrap();

        let err = verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_JSON)
            .unwrap_err();
        assert_eq!(err.code, "invalid_receipt_schema_version");
    }

    #[test]
    fn verify_receipt_rejects_unsupported_schema_version() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let mut raw: serde_json::Value = serde_json::to_value(&receipt).unwrap();
        raw["schema_version"] = serde_json::json!("2");
        let receipt_json = serde_json::to_string(&raw).unwrap();

        let err = verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_JSON)
            .unwrap_err();
        assert_eq!(err.code, "unsupported_receipt_schema_version");
        assert!(err.message.contains("\"2\""), "message must include the found version");
    }

    #[test]
    fn verify_receipt_from_inputs_also_rejects_missing_schema_version() {
        // Confirm the same check applies to the four-artifact path used by the CLI.
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let mut raw: serde_json::Value = serde_json::to_value(&receipt).unwrap();
        raw.as_object_mut().unwrap().remove("schema_version");
        let receipt_json = serde_json::to_string(&raw).unwrap();

        let err = verify_receipt_from_inputs(&receipt_json, CASE_JSON, POLICY_JSON, CANDIDATES_JSON)
            .unwrap_err();
        assert_eq!(err.code, "missing_receipt_schema_version");
    }

    // ── Test 16: receipt_hash ─────────────────────────────────────────────────

    #[test]
    fn route_case_receipt_includes_receipt_hash_as_64_hex_chars() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        assert_eq!(receipt.receipt_hash.len(), 64);
        assert!(receipt.receipt_hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn receipt_hash_is_deterministic_for_same_inputs() {
        let a = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let b = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        assert_eq!(a.receipt_hash, b.receipt_hash);
    }

    #[test]
    fn receipt_hash_differs_for_routed_vs_refused() {
        let routed =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let refused =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, INELIGIBLE_SNAPSHOTS_JSON).unwrap();
        assert_ne!(routed.receipt_hash, refused.receipt_hash);
    }

    #[test]
    fn verify_receipt_accepts_correct_receipt_hash() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();
        verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_JSON)
            .expect("receipt with correct hash must verify");
    }

    #[test]
    fn verify_receipt_rejects_tampered_receipt_hash_field() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let mut raw: serde_json::Value = serde_json::to_value(&receipt).unwrap();
        raw["receipt_hash"] =
            serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
        let receipt_json = serde_json::to_string(&raw).unwrap();

        let err = verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_JSON)
            .unwrap_err();
        assert_eq!(err.code, "receipt_hash_mismatch");
        assert!(err.message.contains("receipt_hash mismatch"));
    }

    #[test]
    fn verify_receipt_rejects_tampered_outcome_via_receipt_hash() {
        // Tamper a semantic field; the receipt_hash catches it before semantic checks.
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let mut raw: serde_json::Value = serde_json::to_value(&receipt).unwrap();
        raw["outcome"] = serde_json::json!("refused"); // lie about the outcome
        // Leave receipt_hash as-is (it now covers the tampered content).
        let receipt_json = serde_json::to_string(&raw).unwrap();

        let err = verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_JSON)
            .unwrap_err();
        assert_eq!(err.code, "receipt_hash_mismatch");
    }

    #[test]
    fn verify_receipt_from_inputs_also_validates_receipt_hash() {
        let receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        let mut raw: serde_json::Value = serde_json::to_value(&receipt).unwrap();
        raw["case_fingerprint"] =
            serde_json::json!("0000000000000000000000000000000000000000000000000000000000000000");
        let receipt_json = serde_json::to_string(&raw).unwrap();

        let err =
            verify_receipt_from_inputs(&receipt_json, CASE_JSON, POLICY_JSON, CANDIDATES_JSON)
                .unwrap_err();
        assert_eq!(err.code, "receipt_hash_mismatch");
    }

    // ── Test 17: policy_version commitment ────────────────────────────────────

    const POLICY_WITH_VERSION_JSON: &str = r#"{
        "jurisdiction": "DE",
        "routing_policy": "allow_domestic_and_cross_border",
        "policy_version": "v1",
        "candidates": [{
            "id": "rc-1",
            "manufacturer_id": "mfr-01",
            "location": "domestic",
            "accepts_case": true,
            "eligibility": "eligible"
        }],
        "snapshots": [{
            "manufacturer_id": "mfr-01",
            "evidence_references": ["REF-001"],
            "attestation_statuses": ["verified"],
            "is_eligible": true
        }]
    }"#;

    #[test]
    fn policy_version_is_committed_in_receipt() {
        let receipt = route_case_from_policy_json(CASE_JSON, POLICY_WITH_VERSION_JSON).unwrap();
        assert_eq!(receipt.policy_version, Some("v1".to_string()));
    }

    #[test]
    fn receipt_without_policy_version_has_null_version() {
        let receipt = route_case_from_policy_json(CASE_JSON, POLICY_JSON).unwrap();
        assert!(receipt.policy_version.is_none());
    }

    #[test]
    fn policy_version_changes_routing_proof_hash() {
        let receipt_no_version = route_case_from_policy_json(CASE_JSON, POLICY_JSON).unwrap();
        let receipt_with_version =
            route_case_from_policy_json(CASE_JSON, POLICY_WITH_VERSION_JSON).unwrap();
        // policy_version feeds into RoutingDecisionFingerprint → proof hash.
        assert_ne!(
            receipt_no_version.routing_proof_hash,
            receipt_with_version.routing_proof_hash
        );
    }

    #[test]
    fn verify_receipt_from_policy_json_passes_with_correct_policy_version() {
        let receipt = route_case_from_policy_json(CASE_JSON, POLICY_WITH_VERSION_JSON).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();

        verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_WITH_VERSION_JSON)
            .expect("should be VERIFIED");
    }

    #[test]
    fn verify_receipt_from_policy_json_fails_when_policy_version_tampered_in_receipt() {
        let receipt = route_case_from_policy_json(CASE_JSON, POLICY_WITH_VERSION_JSON).unwrap();
        // Recompute receipt_hash after tampering so it passes the artifact hash check,
        // then check that the explicit policy_version comparison fires.
        let mut raw: serde_json::Value = serde_json::to_value(&receipt).unwrap();
        raw["policy_version"] = serde_json::json!("v-tampered");
        // Recompute receipt_hash over the tampered object.
        raw.as_object_mut().unwrap().remove("receipt_hash");
        let without_hash = serde_json::to_string(&raw).unwrap();
        use sha2::{Digest, Sha256};
        let new_hash = format!("{:x}", Sha256::digest(without_hash.as_bytes()));
        raw["receipt_hash"] = serde_json::json!(new_hash);
        let receipt_json = serde_json::to_string(&raw).unwrap();

        let err = verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_WITH_VERSION_JSON)
            .unwrap_err();
        assert_eq!(err.code, "policy_version_mismatch");
    }

    #[test]
    fn verify_receipt_from_policy_json_fails_when_bundle_version_differs_from_receipt() {
        // Receipt was issued with "v1"; verifier supplies a bundle with "v2".
        let receipt = route_case_from_policy_json(CASE_JSON, POLICY_WITH_VERSION_JSON).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();

        let policy_v2 = POLICY_WITH_VERSION_JSON.replace("\"v1\"", "\"v2\"");
        let err = verify_receipt_from_policy_json(&receipt_json, CASE_JSON, &policy_v2)
            .unwrap_err();
        assert_eq!(err.code, "policy_version_mismatch");
    }

    #[test]
    fn verify_receipt_from_policy_json_fails_when_bundle_has_no_version_but_receipt_does() {
        // Receipt was issued with "v1"; verifier supplies a bundle with no version.
        let receipt = route_case_from_policy_json(CASE_JSON, POLICY_WITH_VERSION_JSON).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();

        let err = verify_receipt_from_policy_json(&receipt_json, CASE_JSON, POLICY_JSON)
            .unwrap_err();
        assert_eq!(err.code, "policy_version_mismatch");
    }

    #[test]
    fn verify_receipt_from_inputs_passes_with_correct_policy_version() {
        let receipt = route_case_from_policy_json(CASE_JSON, POLICY_WITH_VERSION_JSON).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();
        // verify_receipt_from_inputs uses policy.json for snapshots/version,
        // candidates.json separately. Build a policy-only bundle (no candidates).
        let policy_only = r#"{
            "jurisdiction": "DE",
            "routing_policy": "allow_domestic_and_cross_border",
            "policy_version": "v1",
            "snapshots": [{
                "manufacturer_id": "mfr-01",
                "evidence_references": ["REF-001"],
                "attestation_statuses": ["verified"],
                "is_eligible": true
            }]
        }"#;

        verify_receipt_from_inputs(&receipt_json, CASE_JSON, policy_only, CANDIDATES_JSON)
            .expect("should be VERIFIED");
    }

    #[test]
    fn verify_receipt_from_inputs_fails_when_policy_version_differs() {
        let receipt = route_case_from_policy_json(CASE_JSON, POLICY_WITH_VERSION_JSON).unwrap();
        let receipt_json = serde_json::to_string(&receipt).unwrap();
        let policy_wrong_version = r#"{
            "jurisdiction": "DE",
            "routing_policy": "allow_domestic_and_cross_border",
            "policy_version": "v2",
            "snapshots": [{
                "manufacturer_id": "mfr-01",
                "evidence_references": ["REF-001"],
                "attestation_statuses": ["verified"],
                "is_eligible": true
            }]
        }"#;

        let err =
            verify_receipt_from_inputs(&receipt_json, CASE_JSON, policy_wrong_version, CANDIDATES_JSON)
                .unwrap_err();
        assert_eq!(err.code, "policy_version_mismatch");
    }
}
