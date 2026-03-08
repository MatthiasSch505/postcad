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
pub use receipt::{ReceiptVerificationResult, RefusalDetail, RoutingReceipt};

pub mod policy_bundle;
pub use policy_bundle::{CandidateEntry, RoutingPolicyBundle, SnapshotEntry};

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
        None,
    );

    Ok(map_result_to_receipt(result, case_fingerprint, all_input_candidate_ids))
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

    let all_input_candidate_ids: Vec<String> =
        candidates.iter().map(|c| c.id.0.clone()).collect();

    let case_fingerprint = fingerprint_case(&case);

    let result = route_case_with_compliance_audit(
        &case,
        jurisdiction,
        policy,
        &candidates,
        &snapshots,
        None,
    );

    Ok(map_result_to_receipt(result, case_fingerprint, all_input_candidate_ids))
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
    check!(routing_proof_hash);
    check!(selected_candidate_id);
    check!(refusal_code);
    check!(audit_seq);
    check!(audit_entry_hash);
    check!(audit_previous_hash);
    check!(refusal);

    if mismatched.is_empty() {
        Ok(ReceiptVerificationResult { result: "valid".to_string(), mismatched_fields: None })
    } else {
        Ok(ReceiptVerificationResult {
            result: "mismatch".to_string(),
            mismatched_fields: Some(mismatched),
        })
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
    // Step 1: parse receipt.
    let receipt: RoutingReceipt = serde_json::from_str(receipt_json)
        .map_err(|e| VerificationFailure::receipt_parse_failed(e.to_string()))?;

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
        None,
    );

    if result.proof.hash_hex != receipt.routing_proof_hash {
        return Err(VerificationFailure::routing_proof_hash_mismatch(
            &receipt.routing_proof_hash,
            &result.proof.hash_hex,
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
) -> RoutingReceipt {
    // Extract primitive values from internal types before the match so they
    // are not accidentally referenced after `result` is partially moved.
    let routing_proof_hash: String = result.proof.hash_hex.clone();
    let policy_fingerprint: String = result.policy_fingerprint.clone();
    let case_id_str: String = result.audit_receipt.case_id.clone();
    let compliant_candidate_count: usize =
        result.audit_receipt.candidate_ids_considered.len();

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

            RoutingReceipt {
                outcome: "routed".to_string(),
                case_fingerprint,
                policy_fingerprint,
                routing_proof_hash,
                selected_candidate_id: Some(selected_candidate_id),
                refusal_code: None,
                audit_seq,
                audit_entry_hash,
                audit_previous_hash,
                refusal: None,
            }
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

            RoutingReceipt {
                outcome: "refused".to_string(),
                case_fingerprint,
                policy_fingerprint,
                routing_proof_hash,
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
            }
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

            RoutingReceipt {
                outcome: "refused".to_string(),
                case_fingerprint,
                policy_fingerprint,
                routing_proof_hash,
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
            }
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

    #[test]
    fn verify_receipt_from_policy_json_fails_on_tampered_case_fingerprint() {
        let mut receipt =
            route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON).unwrap();
        receipt.case_fingerprint =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        let receipt_json = serde_json::to_string(&receipt).unwrap();

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
        let receipt_json = serde_json::to_string(&receipt).unwrap();

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
        let receipt_json = serde_json::to_string(&receipt).unwrap();

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
        let receipt_json = serde_json::to_string(&receipt).unwrap();

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
        let receipt_json = serde_json::to_string(&receipt).unwrap();

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
}
