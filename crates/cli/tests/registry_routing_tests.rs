//! Registry-backed routing contract tests.
//!
//! These tests prove that the registry derivation path produces the same
//! deterministic routing behavior as the existing kernel, and that every
//! exclusion rule fires correctly.

use postcad_cli::{route_case_from_registry_json, verify_receipt_from_policy_json};

// ── Canonical fixture JSON ────────────────────────────────────────────────────

/// One eligible German manufacturer: active, Crown/Zirconia, all-verified.
const CASE_JSON: &str = r#"{
  "case_id": "a1b2c3d4-0000-0000-0000-000000000001",
  "jurisdiction": "DE",
  "routing_policy": "allow_domestic_and_cross_border",
  "patient_country": "germany",
  "manufacturer_country": "germany",
  "material": "zirconia",
  "procedure": "crown",
  "file_type": "stl"
}"#;

const SINGLE_ELIGIBLE_REGISTRY: &str = r#"[
  {
    "attestation_statuses": ["verified"],
    "capabilities": ["crown"],
    "country": "germany",
    "display_name": "Test GmbH",
    "is_active": true,
    "jurisdictions_served": ["germany"],
    "manufacturer_id": "mfr-de-01",
    "materials_supported": ["zirconia"],
    "sla_days": 5
  }
]"#;

const DE_CONFIG: &str =
    r#"{"jurisdiction": "DE", "routing_policy": "allow_domestic_and_cross_border"}"#;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn route(registry: &str) -> postcad_cli::RegistryRoutingResult {
    route_case_from_registry_json(CASE_JSON, registry, DE_CONFIG).expect("routing must succeed")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// Same registry + case must always produce the same receipt hash —
/// proving end-to-end determinism of the registry derivation path.
#[test]
fn registry_derived_routing_is_deterministic() {
    let r1 = route(SINGLE_ELIGIBLE_REGISTRY);
    let r2 = route(SINGLE_ELIGIBLE_REGISTRY);
    assert_eq!(
        r1.receipt.receipt_hash, r2.receipt.receipt_hash,
        "identical inputs must produce identical receipt hashes"
    );
}

/// A routed receipt must contain the correct selected candidate id
/// (== manufacturer_id in the registry path).
#[test]
fn registry_routed_receipt_selects_eligible_manufacturer() {
    let result = route(SINGLE_ELIGIBLE_REGISTRY);
    assert_eq!(result.receipt.outcome, "routed");
    assert_eq!(
        result.receipt.selected_candidate_id.as_deref(),
        Some("mfr-de-01"),
        "selected_candidate_id must equal manufacturer_id from registry"
    );
}

/// A receipt produced via the registry path must be verifiable with the
/// derived policy bundle — proving round-trip coherence.
#[test]
fn registry_routed_receipt_is_verifiable() {
    let result = route(SINGLE_ELIGIBLE_REGISTRY);
    let receipt_json = serde_json::to_string(&result.receipt).unwrap();
    verify_receipt_from_policy_json(&receipt_json, CASE_JSON, &result.derived_policy_json)
        .expect("receipt produced by registry path must verify successfully");
}

/// An empty registry (no manufacturers) must produce a refusal with the
/// registry-level reason code `no_eligible_manufacturer`.
#[test]
fn registry_empty_produces_refusal() {
    let result = route("[]");
    assert_eq!(result.receipt.outcome, "refused");
    assert_eq!(
        result.receipt.refusal_code.as_deref(),
        Some("no_eligible_manufacturer")
    );
}

/// An inactive manufacturer must be routed around — yields refusal when it is
/// the only candidate.
#[test]
fn registry_inactive_manufacturer_excluded() {
    let inactive = r#"[
      {
        "attestation_statuses": ["verified"],
        "capabilities": ["crown"],
        "country": "germany",
        "display_name": "Inactive GmbH",
        "is_active": false,
        "jurisdictions_served": ["germany"],
        "manufacturer_id": "mfr-de-inactive",
        "materials_supported": ["zirconia"],
        "sla_days": 3
      }
    ]"#;
    let result = route(inactive);
    assert_eq!(
        result.receipt.outcome, "refused",
        "inactive manufacturer must not be routed"
    );
}

/// A manufacturer that does not serve the requested jurisdiction must not
/// appear in the candidate pool.
#[test]
fn registry_wrong_jurisdiction_excluded() {
    let france_only = r#"[
      {
        "attestation_statuses": ["verified"],
        "capabilities": ["crown"],
        "country": "france",
        "display_name": "French Lab",
        "is_active": true,
        "jurisdictions_served": ["france"],
        "manufacturer_id": "mfr-fr-01",
        "materials_supported": ["zirconia"],
        "sla_days": 7
      }
    ]"#;
    // Case requests DE jurisdiction; this manufacturer only serves France.
    let result = route(france_only);
    assert_eq!(
        result.receipt.outcome, "refused",
        "manufacturer not serving DE must not be routed for a DE case"
    );
}

/// A manufacturer with a non-Verified attestation must be marked ineligible
/// and excluded from routing.
#[test]
fn registry_expired_attestation_excluded() {
    let expired = r#"[
      {
        "attestation_statuses": ["expired"],
        "capabilities": ["crown"],
        "country": "germany",
        "display_name": "Lapsed GmbH",
        "is_active": true,
        "jurisdictions_served": ["germany"],
        "manufacturer_id": "mfr-de-lapsed",
        "materials_supported": ["zirconia"],
        "sla_days": 5
      }
    ]"#;
    let result = route(expired);
    assert_eq!(
        result.receipt.outcome, "refused",
        "expired attestation must exclude manufacturer"
    );
}

/// A manufacturer that lacks the required capability (Crown) must be
/// marked as not accepting the case and excluded by the kernel.
#[test]
fn registry_missing_capability_excluded() {
    let no_crown = r#"[
      {
        "attestation_statuses": ["verified"],
        "capabilities": ["implant"],
        "country": "germany",
        "display_name": "Implant-Only GmbH",
        "is_active": true,
        "jurisdictions_served": ["germany"],
        "manufacturer_id": "mfr-de-implant",
        "materials_supported": ["zirconia"],
        "sla_days": 5
      }
    ]"#;
    let result = route(no_crown);
    // The manufacturer is compliance-eligible (Verified) but doesn't support
    // Crown.  accepts_case = false so it won't be selected.
    let outcome = &result.receipt.outcome;
    // Either refused OR routed to a different manufacturer — with a single
    // incapable candidate the only valid outcome is refusal.
    assert_eq!(
        outcome, "refused",
        "manufacturer lacking Crown capability must not be selected"
    );
}

/// A manufacturer that lacks the required material must not be selected.
#[test]
fn registry_unsupported_material_excluded() {
    let wrong_material = r#"[
      {
        "attestation_statuses": ["verified"],
        "capabilities": ["crown"],
        "country": "germany",
        "display_name": "PMMA GmbH",
        "is_active": true,
        "jurisdictions_served": ["germany"],
        "manufacturer_id": "mfr-de-pmma",
        "materials_supported": ["pmma"],
        "sla_days": 4
      }
    ]"#;
    let result = route(wrong_material);
    assert_eq!(
        result.receipt.outcome, "refused",
        "manufacturer not supporting Zirconia must not be selected"
    );
}

/// When two eligible manufacturers are present, only one is selected and the
/// result is still deterministic (same receipt hash on repeated calls).
#[test]
fn registry_multi_candidate_selection_is_deterministic() {
    let two_eligible = r#"[
      {
        "attestation_statuses": ["verified"],
        "capabilities": ["crown"],
        "country": "germany",
        "display_name": "Alpha GmbH",
        "is_active": true,
        "jurisdictions_served": ["germany"],
        "manufacturer_id": "mfr-de-alpha",
        "materials_supported": ["zirconia"],
        "sla_days": 5
      },
      {
        "attestation_statuses": ["verified"],
        "capabilities": ["crown"],
        "country": "germany",
        "display_name": "Beta GmbH",
        "is_active": true,
        "jurisdictions_served": ["germany"],
        "manufacturer_id": "mfr-de-beta",
        "materials_supported": ["zirconia"],
        "sla_days": 3
      }
    ]"#;
    let r1 = route(two_eligible);
    let r2 = route(two_eligible);
    assert_eq!(r1.receipt.outcome, "routed");
    assert_eq!(
        r1.receipt.receipt_hash, r2.receipt.receipt_hash,
        "multi-candidate routing must be deterministic"
    );
    // Both calls must select the same manufacturer.
    assert_eq!(
        r1.receipt.selected_candidate_id,
        r2.receipt.selected_candidate_id
    );
}

/// A registry with one eligible and one ineligible (no attestations) manufacturer
/// must route to the eligible one.
#[test]
fn registry_routes_eligible_skips_ineligible_in_mixed_pool() {
    let mixed = r#"[
      {
        "attestation_statuses": [],
        "capabilities": ["crown"],
        "country": "germany",
        "display_name": "Unattested GmbH",
        "is_active": true,
        "jurisdictions_served": ["germany"],
        "manufacturer_id": "mfr-de-unattested",
        "materials_supported": ["zirconia"],
        "sla_days": 5
      },
      {
        "attestation_statuses": ["verified"],
        "capabilities": ["crown"],
        "country": "germany",
        "display_name": "Certified GmbH",
        "is_active": true,
        "jurisdictions_served": ["germany"],
        "manufacturer_id": "mfr-de-certified",
        "materials_supported": ["zirconia"],
        "sla_days": 5
      }
    ]"#;
    let result = route(mixed);
    assert_eq!(result.receipt.outcome, "routed");
    assert_eq!(
        result.receipt.selected_candidate_id.as_deref(),
        Some("mfr-de-certified"),
        "only the certified manufacturer should be selected"
    );
}

// ── Deterministic refusal reason code tests ───────────────────────────────────

/// An empty registry must yield `no_eligible_manufacturer` (not the generic
/// `no_eligible_candidates` from the kernel) because the registry layer can
/// inspect the empty set before any filtering.
///
/// Note: the current implementation falls through all stepwise checks when the
/// registry is empty and returns the `no_eligible_manufacturer` sentinel.
#[test]
fn refusal_reason_empty_registry_is_no_eligible_manufacturer() {
    let result = route("[]");
    assert_eq!(result.receipt.outcome, "refused");
    assert_eq!(
        result.receipt.refusal_code.as_deref(),
        Some("no_eligible_manufacturer"),
        "empty registry must produce no_eligible_manufacturer"
    );
}

/// A registry where all manufacturers are inactive must produce `no_active_manufacturer`.
#[test]
fn refusal_reason_all_inactive_is_no_active_manufacturer() {
    let all_inactive = r#"[
      {
        "attestation_statuses": ["verified"],
        "capabilities": ["crown"],
        "country": "germany",
        "display_name": "Inactive A",
        "is_active": false,
        "jurisdictions_served": ["germany"],
        "manufacturer_id": "mfr-de-inactive-a",
        "materials_supported": ["zirconia"],
        "sla_days": 5
      },
      {
        "attestation_statuses": ["verified"],
        "capabilities": ["crown"],
        "country": "germany",
        "display_name": "Inactive B",
        "is_active": false,
        "jurisdictions_served": ["germany"],
        "manufacturer_id": "mfr-de-inactive-b",
        "materials_supported": ["zirconia"],
        "sla_days": 5
      }
    ]"#;
    let result = route(all_inactive);
    assert_eq!(result.receipt.outcome, "refused");
    assert_eq!(
        result.receipt.refusal_code.as_deref(),
        Some("no_active_manufacturer"),
        "all-inactive registry must produce no_active_manufacturer"
    );
}

/// When all active manufacturers fail the jurisdiction filter, the code must
/// be `no_jurisdiction_match`.
#[test]
fn refusal_reason_no_jurisdiction_match() {
    let us_only = r#"[
      {
        "attestation_statuses": ["verified"],
        "capabilities": ["crown"],
        "country": "united_states",
        "display_name": "US Lab",
        "is_active": true,
        "jurisdictions_served": ["united_states"],
        "manufacturer_id": "mfr-us-01",
        "materials_supported": ["zirconia"],
        "sla_days": 5
      }
    ]"#;
    // CASE_JSON requests jurisdiction DE; this manufacturer only serves US.
    let result = route(us_only);
    assert_eq!(result.receipt.outcome, "refused");
    assert_eq!(
        result.receipt.refusal_code.as_deref(),
        Some("no_jurisdiction_match"),
        "no active manufacturer serving the jurisdiction must produce no_jurisdiction_match"
    );
}

/// When manufacturers serve the jurisdiction but lack the required capability,
/// the code must be `no_capability_match`.
#[test]
fn refusal_reason_no_capability_match() {
    let implant_only = r#"[
      {
        "attestation_statuses": ["verified"],
        "capabilities": ["implant"],
        "country": "germany",
        "display_name": "Implant Lab",
        "is_active": true,
        "jurisdictions_served": ["germany"],
        "manufacturer_id": "mfr-de-implant",
        "materials_supported": ["zirconia"],
        "sla_days": 5
      }
    ]"#;
    // CASE_JSON requests procedure=crown; this manufacturer only does implant.
    let result = route(implant_only);
    assert_eq!(result.receipt.outcome, "refused");
    assert_eq!(
        result.receipt.refusal_code.as_deref(),
        Some("no_capability_match"),
        "no manufacturer with the required capability must produce no_capability_match"
    );
}

/// When manufacturers serve the jurisdiction and have the capability but lack
/// the required material, the code must be `no_material_match`.
#[test]
fn refusal_reason_no_material_match() {
    let pmma_only = r#"[
      {
        "attestation_statuses": ["verified"],
        "capabilities": ["crown"],
        "country": "germany",
        "display_name": "PMMA Lab",
        "is_active": true,
        "jurisdictions_served": ["germany"],
        "manufacturer_id": "mfr-de-pmma",
        "materials_supported": ["pmma"],
        "sla_days": 5
      }
    ]"#;
    // CASE_JSON requests material=zirconia; this manufacturer only supports pmma.
    let result = route(pmma_only);
    assert_eq!(result.receipt.outcome, "refused");
    assert_eq!(
        result.receipt.refusal_code.as_deref(),
        Some("no_material_match"),
        "no manufacturer supporting the required material must produce no_material_match"
    );
}

/// When manufacturers pass all structural filters but have expired attestations,
/// the code must be `attestation_failed`.
#[test]
fn refusal_reason_attestation_failed() {
    let expired = r#"[
      {
        "attestation_statuses": ["expired"],
        "capabilities": ["crown"],
        "country": "germany",
        "display_name": "Lapsed GmbH",
        "is_active": true,
        "jurisdictions_served": ["germany"],
        "manufacturer_id": "mfr-de-lapsed",
        "materials_supported": ["zirconia"],
        "sla_days": 5
      },
      {
        "attestation_statuses": ["revoked"],
        "capabilities": ["crown"],
        "country": "germany",
        "display_name": "Revoked GmbH",
        "is_active": true,
        "jurisdictions_served": ["germany"],
        "manufacturer_id": "mfr-de-revoked",
        "materials_supported": ["zirconia"],
        "sla_days": 5
      }
    ]"#;
    let result = route(expired);
    assert_eq!(result.receipt.outcome, "refused");
    assert_eq!(
        result.receipt.refusal_code.as_deref(),
        Some("attestation_failed"),
        "all-expired attestations must produce attestation_failed"
    );
}

/// The refusal reason code must be consistent across repeated calls with the
/// same inputs (determinism invariant).
#[test]
fn refusal_reason_codes_are_deterministic() {
    let us_only = r#"[
      {
        "attestation_statuses": ["verified"],
        "capabilities": ["crown"],
        "country": "united_states",
        "display_name": "US Lab",
        "is_active": true,
        "jurisdictions_served": ["united_states"],
        "manufacturer_id": "mfr-us-01",
        "materials_supported": ["zirconia"],
        "sla_days": 5
      }
    ]"#;
    let r1 = route(us_only);
    let r2 = route(us_only);
    assert_eq!(r1.receipt.refusal_code, r2.receipt.refusal_code);
    assert_eq!(r1.receipt.receipt_hash, r2.receipt.receipt_hash);
}

/// A receipt with a specific refusal reason code must verify successfully
/// using the derived policy bundle — round-trip coherence.
#[test]
fn refusal_reason_receipt_is_verifiable() {
    let us_only = r#"[
      {
        "attestation_statuses": ["verified"],
        "capabilities": ["crown"],
        "country": "united_states",
        "display_name": "US Lab",
        "is_active": true,
        "jurisdictions_served": ["united_states"],
        "manufacturer_id": "mfr-us-01",
        "materials_supported": ["zirconia"],
        "sla_days": 5
      }
    ]"#;
    let result = route(us_only);
    assert_eq!(
        result.receipt.refusal_code.as_deref(),
        Some("no_jurisdiction_match")
    );
    let receipt_json = serde_json::to_string(&result.receipt).unwrap();
    verify_receipt_from_policy_json(&receipt_json, CASE_JSON, &result.derived_policy_json)
        .expect("specific refusal code receipt must verify successfully");
}
