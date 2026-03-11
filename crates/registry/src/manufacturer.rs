//! Typed manufacturer registry model.
//!
//! [`ManufacturerRecord`] is the canonical, compliance-first source of truth for a
//! single manufacturer in the PostCAD registry. It is separate from the lightweight
//! [`super::ManufacturerComplianceSnapshot`] used at routing time: the snapshot is
//! a pre-computed eligibility verdict; the record is the full typed fact base from
//! which eligibility can be derived.
//!
//! # Determinism guarantee
//!
//! [`canonical_hash`] produces a stable SHA-256 over the record's fields. Vec fields
//! are sorted before hashing so the hash is independent of insertion order. Struct
//! fields are declared alphabetically so `serde_json` serialization produces
//! alphabetically ordered JSON keys without any post-processing.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// ── Typed enums ───────────────────────────────────────────────────────────────

/// Country / jurisdiction in the manufacturer registry.
///
/// Variants are serialized as snake_case strings (e.g. `"united_states"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManufacturerCountry {
    France,
    Germany,
    Japan,
    UnitedKingdom,
    UnitedStates,
}

/// Dental material supported by the manufacturer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManufacturerMaterial {
    CobaltChrome,
    Emax,
    Pmma,
    Titanium,
    Zirconia,
}

/// Dental procedure (capability) supported by the manufacturer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManufacturerCapability {
    Bridge,
    Crown,
    Denture,
    Implant,
    Veneer,
}

/// Attestation / certification status for a single evidence item.
///
/// Only [`AttestationStatus::Verified`] passes compliance filtering.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttestationStatus {
    Expired,
    Pending,
    Revoked,
    Verified,
}

impl AttestationStatus {
    /// Returns `true` only for the `Verified` state.
    pub fn is_compliant(&self) -> bool {
        matches!(self, AttestationStatus::Verified)
    }
}

// ── Typed record ──────────────────────────────────────────────────────────────

/// Full typed manufacturer record.
///
/// Fields are declared in alphabetical order so `serde_json` serialization
/// produces alphabetically ordered JSON keys — a requirement for canonical hashing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManufacturerRecord {
    /// All current attestation / certification statuses.
    pub attestation_statuses: Vec<AttestationStatus>,
    /// Dental procedures this manufacturer can produce.
    pub capabilities: Vec<ManufacturerCapability>,
    /// Physical country where the manufacturer operates.
    pub country: ManufacturerCountry,
    /// Human-readable name (not used for routing decisions).
    pub display_name: String,
    /// Whether the manufacturer is currently accepting cases.
    pub is_active: bool,
    /// Countries / jurisdictions this manufacturer can legally serve.
    pub jurisdictions_served: Vec<ManufacturerCountry>,
    /// Stable, opaque identifier — must be unique across the registry.
    pub manufacturer_id: String,
    /// Materials this manufacturer can work with.
    pub materials_supported: Vec<ManufacturerMaterial>,
    /// Typical turnaround in calendar days (informational; not used for routing).
    pub sla_days: u32,
}

// ── Deterministic filtering ───────────────────────────────────────────────────

/// Keeps only manufacturers where [`ManufacturerRecord::is_active`] is `true`.
pub fn filter_active(records: &[ManufacturerRecord]) -> Vec<&ManufacturerRecord> {
    records.iter().filter(|r| r.is_active).collect()
}

/// Keeps only manufacturers that serve the requested jurisdiction.
pub fn filter_by_jurisdiction<'a>(
    records: &'a [ManufacturerRecord],
    target: &ManufacturerCountry,
) -> Vec<&'a ManufacturerRecord> {
    records
        .iter()
        .filter(|r| r.jurisdictions_served.contains(target))
        .collect()
}

/// Keeps only manufacturers that have the required capability.
pub fn filter_by_capability<'a>(
    records: &'a [ManufacturerRecord],
    required: &ManufacturerCapability,
) -> Vec<&'a ManufacturerRecord> {
    records
        .iter()
        .filter(|r| r.capabilities.contains(required))
        .collect()
}

/// Keeps only manufacturers that support the required material.
pub fn filter_by_material<'a>(
    records: &'a [ManufacturerRecord],
    required: &ManufacturerMaterial,
) -> Vec<&'a ManufacturerRecord> {
    records
        .iter()
        .filter(|r| r.materials_supported.contains(required))
        .collect()
}

/// Keeps only manufacturers that have at least one attestation and all
/// attestations are [`AttestationStatus::Verified`].
///
/// An empty `attestation_statuses` list is treated as non-compliant: a
/// manufacturer cannot prove eligibility without evidence.
pub fn filter_by_attestation(records: &[ManufacturerRecord]) -> Vec<&ManufacturerRecord> {
    records
        .iter()
        .filter(|r| {
            !r.attestation_statuses.is_empty()
                && r.attestation_statuses.iter().all(|s| s.is_compliant())
        })
        .collect()
}

/// Composite eligibility filter applying all four criteria in sequence:
/// active → jurisdiction → capability → material → attestation.
///
/// Returns only manufacturers that pass every check.
pub fn eligible_records<'a>(
    records: &'a [ManufacturerRecord],
    jurisdiction: &ManufacturerCountry,
    capability: &ManufacturerCapability,
    material: &ManufacturerMaterial,
) -> Vec<&'a ManufacturerRecord> {
    let active = filter_active(records);
    let by_jurisdiction: Vec<&ManufacturerRecord> = active
        .into_iter()
        .filter(|r| r.jurisdictions_served.contains(jurisdiction))
        .collect();
    let by_capability: Vec<&ManufacturerRecord> = by_jurisdiction
        .into_iter()
        .filter(|r| r.capabilities.contains(capability))
        .collect();
    let by_material: Vec<&ManufacturerRecord> = by_capability
        .into_iter()
        .filter(|r| r.materials_supported.contains(material))
        .collect();
    by_material
        .into_iter()
        .filter(|r| {
            !r.attestation_statuses.is_empty()
                && r.attestation_statuses.iter().all(|s| s.is_compliant())
        })
        .collect()
}

// ── Canonical hash ────────────────────────────────────────────────────────────

/// Returns the SHA-256 canonical hash of a manufacturer record as a 64-char
/// lowercase hex string.
///
/// Canonical form: compact JSON with alphabetically ordered keys (guaranteed by
/// struct field declaration order) and lexicographically sorted array elements
/// (applied by [`sort_json_arrays`]). This makes the hash independent of the
/// insertion order of Vec fields.
pub fn canonical_hash(record: &ManufacturerRecord) -> String {
    let value =
        serde_json::to_value(record).expect("ManufacturerRecord is always JSON-serializable");
    let sorted = sort_json_arrays(value);
    let json = serde_json::to_string(&sorted).expect("canonical serialization never fails");
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    hex::encode(hasher.finalize())
}

/// Recursively sorts all JSON arrays by their compact string representation.
/// Objects are passed through with their existing key order preserved.
fn sort_json_arrays(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(arr) => {
            let mut items: Vec<serde_json::Value> = arr.into_iter().map(sort_json_arrays).collect();
            items.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
            serde_json::Value::Array(items)
        }
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.into_iter()
                .map(|(k, v)| (k, sort_json_arrays(v)))
                .collect(),
        ),
        other => other,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn active_de_crown_zirconia() -> ManufacturerRecord {
        ManufacturerRecord {
            attestation_statuses: vec![AttestationStatus::Verified],
            capabilities: vec![ManufacturerCapability::Crown],
            country: ManufacturerCountry::Germany,
            display_name: "Test GmbH".to_string(),
            is_active: true,
            jurisdictions_served: vec![ManufacturerCountry::Germany],
            manufacturer_id: "mfr-de-01".to_string(),
            materials_supported: vec![ManufacturerMaterial::Zirconia],
            sla_days: 5,
        }
    }

    // ── filter_active ─────────────────────────────────────────────────────────

    #[test]
    fn filter_active_keeps_active_manufacturer() {
        let records = [active_de_crown_zirconia()];
        let result = filter_active(&records);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_active_excludes_inactive_manufacturer() {
        let mut record = active_de_crown_zirconia();
        record.is_active = false;
        let records = [record];
        let result = filter_active(&records);
        assert!(result.is_empty(), "inactive manufacturer must be excluded");
    }

    // ── filter_by_jurisdiction ────────────────────────────────────────────────

    #[test]
    fn filter_by_jurisdiction_keeps_matching_jurisdiction() {
        let records = [active_de_crown_zirconia()];
        let result = filter_by_jurisdiction(&records, &ManufacturerCountry::Germany);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_by_jurisdiction_excludes_unsupported_jurisdiction() {
        let records = [active_de_crown_zirconia()]; // only serves Germany
        let result = filter_by_jurisdiction(&records, &ManufacturerCountry::France);
        assert!(
            result.is_empty(),
            "manufacturer not serving France must be excluded"
        );
    }

    // ── filter_by_capability ──────────────────────────────────────────────────

    #[test]
    fn filter_by_capability_keeps_supported_capability() {
        let records = [active_de_crown_zirconia()];
        let result = filter_by_capability(&records, &ManufacturerCapability::Crown);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_by_capability_excludes_missing_capability() {
        let records = [active_de_crown_zirconia()]; // only Crown
        let result = filter_by_capability(&records, &ManufacturerCapability::Implant);
        assert!(
            result.is_empty(),
            "manufacturer lacking Implant capability must be excluded"
        );
    }

    // ── filter_by_material ────────────────────────────────────────────────────

    #[test]
    fn filter_by_material_keeps_supported_material() {
        let records = [active_de_crown_zirconia()];
        let result = filter_by_material(&records, &ManufacturerMaterial::Zirconia);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_by_material_excludes_unsupported_material() {
        let records = [active_de_crown_zirconia()]; // only Zirconia
        let result = filter_by_material(&records, &ManufacturerMaterial::Titanium);
        assert!(
            result.is_empty(),
            "manufacturer not supporting Titanium must be excluded"
        );
    }

    // ── filter_by_attestation ─────────────────────────────────────────────────

    #[test]
    fn filter_by_attestation_keeps_all_verified() {
        let records = [active_de_crown_zirconia()]; // Verified
        let result = filter_by_attestation(&records);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn filter_by_attestation_excludes_expired_status() {
        let mut record = active_de_crown_zirconia();
        record.attestation_statuses = vec![AttestationStatus::Expired];
        let records = [record];
        let result = filter_by_attestation(&records);
        assert!(
            result.is_empty(),
            "manufacturer with Expired attestation must be excluded"
        );
    }

    #[test]
    fn filter_by_attestation_excludes_pending_status() {
        let mut record = active_de_crown_zirconia();
        record.attestation_statuses = vec![AttestationStatus::Pending];
        let records = [record];
        let result = filter_by_attestation(&records);
        assert!(
            result.is_empty(),
            "manufacturer with Pending attestation must be excluded"
        );
    }

    #[test]
    fn filter_by_attestation_excludes_revoked_status() {
        let mut record = active_de_crown_zirconia();
        record.attestation_statuses = vec![AttestationStatus::Revoked];
        let records = [record];
        let result = filter_by_attestation(&records);
        assert!(
            result.is_empty(),
            "manufacturer with Revoked attestation must be excluded"
        );
    }

    #[test]
    fn filter_by_attestation_excludes_mixed_verified_and_non_verified() {
        let mut record = active_de_crown_zirconia();
        record.attestation_statuses = vec![AttestationStatus::Verified, AttestationStatus::Expired];
        let records = [record];
        let result = filter_by_attestation(&records);
        assert!(
            result.is_empty(),
            "any non-Verified attestation must disqualify the manufacturer"
        );
    }

    #[test]
    fn filter_by_attestation_excludes_empty_attestation_list() {
        let mut record = active_de_crown_zirconia();
        record.attestation_statuses = vec![];
        let records = [record];
        let result = filter_by_attestation(&records);
        assert!(
            result.is_empty(),
            "manufacturer with no attestations cannot prove compliance"
        );
    }

    // ── eligible_records composite filter ─────────────────────────────────────

    #[test]
    fn eligible_records_keeps_fully_eligible_manufacturer() {
        let records = [active_de_crown_zirconia()];
        let result = eligible_records(
            &records,
            &ManufacturerCountry::Germany,
            &ManufacturerCapability::Crown,
            &ManufacturerMaterial::Zirconia,
        );
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn eligible_records_excludes_inactive() {
        let mut record = active_de_crown_zirconia();
        record.is_active = false;
        let records = [record];
        let result = eligible_records(
            &records,
            &ManufacturerCountry::Germany,
            &ManufacturerCapability::Crown,
            &ManufacturerMaterial::Zirconia,
        );
        assert!(result.is_empty());
    }

    #[test]
    fn eligible_records_excludes_wrong_jurisdiction() {
        let records = [active_de_crown_zirconia()];
        let result = eligible_records(
            &records,
            &ManufacturerCountry::Japan,
            &ManufacturerCapability::Crown,
            &ManufacturerMaterial::Zirconia,
        );
        assert!(result.is_empty());
    }

    #[test]
    fn eligible_records_excludes_missing_capability() {
        let records = [active_de_crown_zirconia()];
        let result = eligible_records(
            &records,
            &ManufacturerCountry::Germany,
            &ManufacturerCapability::Bridge,
            &ManufacturerMaterial::Zirconia,
        );
        assert!(result.is_empty());
    }

    #[test]
    fn eligible_records_excludes_unsupported_material() {
        let records = [active_de_crown_zirconia()];
        let result = eligible_records(
            &records,
            &ManufacturerCountry::Germany,
            &ManufacturerCapability::Crown,
            &ManufacturerMaterial::Pmma,
        );
        assert!(result.is_empty());
    }

    #[test]
    fn eligible_records_excludes_failing_attestation() {
        let mut record = active_de_crown_zirconia();
        record.attestation_statuses = vec![AttestationStatus::Revoked];
        let records = [record];
        let result = eligible_records(
            &records,
            &ManufacturerCountry::Germany,
            &ManufacturerCapability::Crown,
            &ManufacturerMaterial::Zirconia,
        );
        assert!(result.is_empty());
    }

    #[test]
    fn eligible_records_selects_only_eligible_from_mixed_pool() {
        let eligible = active_de_crown_zirconia();
        let mut inactive = active_de_crown_zirconia();
        inactive.is_active = false;
        inactive.manufacturer_id = "mfr-de-02".to_string();
        let records = [eligible, inactive];

        let result = eligible_records(
            &records,
            &ManufacturerCountry::Germany,
            &ManufacturerCapability::Crown,
            &ManufacturerMaterial::Zirconia,
        );
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].manufacturer_id, "mfr-de-01");
    }

    // ── canonical_hash ────────────────────────────────────────────────────────

    #[test]
    fn canonical_hash_is_deterministic() {
        let record = active_de_crown_zirconia();
        assert_eq!(canonical_hash(&record), canonical_hash(&record));
    }

    #[test]
    fn canonical_hash_differs_for_different_records() {
        let mut a = active_de_crown_zirconia();
        let mut b = active_de_crown_zirconia();
        b.manufacturer_id = "mfr-de-99".to_string();
        assert_ne!(canonical_hash(&a), canonical_hash(&b));

        a.sla_days = 10;
        assert_ne!(canonical_hash(&a), canonical_hash(&b));
    }

    #[test]
    fn canonical_hash_independent_of_vec_element_insertion_order() {
        let mut a = active_de_crown_zirconia();
        a.jurisdictions_served = vec![ManufacturerCountry::Germany, ManufacturerCountry::France];
        a.materials_supported = vec![ManufacturerMaterial::Zirconia, ManufacturerMaterial::Pmma];

        let mut b = active_de_crown_zirconia();
        b.jurisdictions_served = vec![ManufacturerCountry::France, ManufacturerCountry::Germany];
        b.materials_supported = vec![ManufacturerMaterial::Pmma, ManufacturerMaterial::Zirconia];

        assert_eq!(
            canonical_hash(&a),
            canonical_hash(&b),
            "canonical hash must be independent of Vec insertion order"
        );
    }

    #[test]
    fn canonical_hash_is_64_hex_chars() {
        let hash = canonical_hash(&active_de_crown_zirconia());
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
