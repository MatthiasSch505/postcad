use postcad_audit::{route_case_with_compliance_audit, RoutingServiceResult};
use postcad_core::{
    Case, CaseId, Country, DentalCase, FileType, ManufacturerEligibility, ManufacturingLocation,
    Material, ProcedureType, RefusalReason, RoutingCandidate, RoutingCandidateId, RoutingDecision,
    RoutingPolicy, fingerprint_case,
};
use postcad_registry::snapshot::ManufacturerComplianceSnapshot;
use serde::{Deserialize, Serialize};
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

/// One entry from the `--candidates` JSON array.
#[derive(Debug, Deserialize)]
pub struct CandidateInput {
    pub id: String,
    pub manufacturer_id: String,
    /// `"domestic"` | `"cross_border"` | `"unknown"`
    pub location: String,
    pub accepts_case: bool,
    /// `"eligible"` | `"ineligible"` | `"unknown"`
    pub eligibility: String,
}

/// One entry from the `--snapshot` JSON array.
#[derive(Debug, Deserialize)]
pub struct SnapshotInput {
    pub manufacturer_id: String,
    pub evidence_references: Vec<String>,
    pub attestation_statuses: Vec<String>,
    pub is_eligible: bool,
}

// ── Output DTOs ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, PartialEq)]
pub struct RouteCaseOutput {
    pub outcome: String,
    pub selected_candidate_id: Option<String>,
    pub routing_proof_hash: String,
    pub policy_fingerprint: String,
    pub case_fingerprint: String,
    pub refusal: Option<RefusalOutput>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct RefusalOutput {
    pub code: String,
    pub reasons: Vec<String>,
}

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum CliError {
    #[error("JSON parse error: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("invalid field value: {0}")]
    InvalidField(String),
}

// ── Public entrypoint ─────────────────────────────────────────────────────────

/// Runs the compliance-aware routing pipeline from raw JSON strings.
///
/// This is the testable pure core of the `route-case` command. No file I/O.
pub fn route_case_from_json(
    case_json: &str,
    candidates_json: &str,
    snapshots_json: &str,
) -> Result<RouteCaseOutput, CliError> {
    let case_input: CaseInput = serde_json::from_str(case_json)?;
    let candidates_input: Vec<CandidateInput> = serde_json::from_str(candidates_json)?;
    let snapshots_input: Vec<SnapshotInput> = serde_json::from_str(snapshots_json)?;

    let case = build_case(&case_input)?;
    let candidates = build_candidates(&candidates_input)?;
    let snapshots = build_snapshots(&snapshots_input);
    let policy = parse_routing_policy(case_input.routing_policy.as_deref())?;
    let jurisdiction = case_input.jurisdiction.as_deref().unwrap_or("global");

    let case_fp = fingerprint_case(&case);

    let result = route_case_with_compliance_audit(
        &case,
        jurisdiction,
        policy,
        &candidates,
        &snapshots,
        None,
    );

    Ok(build_output(result, case_fp))
}

// ── Conversion helpers ────────────────────────────────────────────────────────

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

fn build_candidates(inputs: &[CandidateInput]) -> Result<Vec<RoutingCandidate>, CliError> {
    inputs
        .iter()
        .map(|c| {
            Ok(RoutingCandidate::new(
                RoutingCandidateId::new(c.id.as_str()),
                c.manufacturer_id.as_str(),
                parse_location(&c.location)?,
                c.accepts_case,
                parse_eligibility(&c.eligibility)?,
            ))
        })
        .collect()
}

fn build_snapshots(inputs: &[SnapshotInput]) -> Vec<ManufacturerComplianceSnapshot> {
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

fn build_output(result: RoutingServiceResult, case_fp: String) -> RouteCaseOutput {
    let proof_hash = result.proof.hash_hex.clone();
    let policy_fp = result.policy_fingerprint.clone();

    match result.outcome.decision {
        RoutingDecision::Selected(id) => RouteCaseOutput {
            outcome: "routed".to_string(),
            selected_candidate_id: Some(id.0),
            routing_proof_hash: proof_hash,
            policy_fingerprint: policy_fp,
            case_fingerprint: case_fp,
            refusal: None,
        },
        RoutingDecision::Refused(r) => RouteCaseOutput {
            outcome: "refused".to_string(),
            selected_candidate_id: None,
            routing_proof_hash: proof_hash,
            policy_fingerprint: policy_fp,
            case_fingerprint: case_fp,
            refusal: Some(RefusalOutput {
                code: r
                    .reasons
                    .first()
                    .map(reason_str)
                    .unwrap_or_else(|| "Unknown".to_string()),
                reasons: r.reasons.iter().map(reason_str).collect(),
            }),
        },
        RoutingDecision::NoEligibleCandidate => RouteCaseOutput {
            outcome: "refused".to_string(),
            selected_candidate_id: None,
            routing_proof_hash: proof_hash,
            policy_fingerprint: policy_fp,
            case_fingerprint: case_fp,
            refusal: Some(RefusalOutput {
                code: "NoEligibleCandidate".to_string(),
                reasons: vec!["NoEligibleCandidate".to_string()],
            }),
        },
    }
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

fn reason_str(r: &RefusalReason) -> String {
    format!("{:?}", r)
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
        let output = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON)
            .expect("routing should succeed");

        assert_eq!(output.outcome, "routed");
        assert_eq!(output.selected_candidate_id, Some("rc-1".to_string()));
        assert!(output.refusal.is_none());
    }

    #[test]
    fn routed_case_proof_hash_is_64_hex_chars() {
        let output = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();

        assert_eq!(output.routing_proof_hash.len(), 64);
        assert!(output.routing_proof_hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn routed_case_policy_fingerprint_is_64_hex_chars() {
        let output = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();

        assert_eq!(output.policy_fingerprint.len(), 64);
    }

    #[test]
    fn routed_case_case_fingerprint_is_64_hex_chars() {
        let output = route_case_from_json(CASE_JSON, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();

        assert_eq!(output.case_fingerprint.len(), 64);
    }

    #[test]
    fn routed_case_with_fixed_case_id_is_deterministic() {
        // Same case_id -> same case_fingerprint across calls.
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
        let output = route_case_from_json(CASE_JSON, CANDIDATES_JSON, INELIGIBLE_SNAPSHOTS_JSON)
            .expect("parse should succeed");

        assert_eq!(output.outcome, "refused");
        assert!(output.selected_candidate_id.is_none());
        let refusal = output.refusal.expect("refusal should be present");
        assert_eq!(refusal.code, "NoEligibleCandidate");
    }

    #[test]
    fn refused_case_no_snapshots_returns_refused_outcome() {
        let output = route_case_from_json(CASE_JSON, CANDIDATES_JSON, EMPTY_SNAPSHOTS_JSON)
            .expect("parse should succeed");

        assert_eq!(output.outcome, "refused");
        assert!(output.selected_candidate_id.is_none());
        assert!(output.refusal.is_some());
    }

    #[test]
    fn refused_case_proof_hash_is_present() {
        let output = route_case_from_json(CASE_JSON, CANDIDATES_JSON, INELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();

        assert_eq!(output.routing_proof_hash.len(), 64);
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
        let output = route_case_from_json(case_json, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();

        assert_eq!(output.outcome, "routed");
        assert_eq!(output.selected_candidate_id, Some("rc-1".to_string()));
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
        let output = route_case_from_json(case_no_id, CANDIDATES_JSON, ELIGIBLE_SNAPSHOTS_JSON)
            .unwrap();

        assert_eq!(output.outcome, "routed");
        assert_eq!(output.case_fingerprint.len(), 64);
    }

    #[test]
    fn multi_candidate_eligible_selects_first_in_order() {
        let candidates = r#"[
            {"id":"rc-1","manufacturer_id":"mfr-01","location":"domestic","accepts_case":true,"eligibility":"eligible"},
            {"id":"rc-2","manufacturer_id":"mfr-02","location":"domestic","accepts_case":true,"eligibility":"eligible"}
        ]"#;
        let snapshots = r#"[
            {"manufacturer_id":"mfr-01","evidence_references":[],"attestation_statuses":[],"is_eligible":true},
            {"manufacturer_id":"mfr-02","evidence_references":[],"attestation_statuses":[],"is_eligible":true}
        ]"#;
        let output = route_case_from_json(CASE_JSON, candidates, snapshots).unwrap();

        // rc-1 is first eligible in original slice order
        assert_eq!(output.selected_candidate_id, Some("rc-1".to_string()));
    }

    #[test]
    fn first_candidate_ineligible_selects_second() {
        let candidates = r#"[
            {"id":"rc-1","manufacturer_id":"mfr-01","location":"domestic","accepts_case":true,"eligibility":"eligible"},
            {"id":"rc-2","manufacturer_id":"mfr-02","location":"domestic","accepts_case":true,"eligibility":"eligible"}
        ]"#;
        let snapshots = r#"[
            {"manufacturer_id":"mfr-01","evidence_references":[],"attestation_statuses":[],"is_eligible":false},
            {"manufacturer_id":"mfr-02","evidence_references":[],"attestation_statuses":[],"is_eligible":true}
        ]"#;
        let output = route_case_from_json(CASE_JSON, candidates, snapshots).unwrap();

        assert_eq!(output.selected_candidate_id, Some("rc-2".to_string()));
    }
}
