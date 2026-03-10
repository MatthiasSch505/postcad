//! Deterministic registry snapshot commitment hashing.
//!
//! Computes a canonical SHA-256 hash of a set of `ManufacturerComplianceSnapshot`
//! items. The hash is order-independent (entries are sorted by `manufacturer_id`
//! before serialization) and covers all snapshot fields that affect compliance
//! eligibility decisions.

use postcad_registry::snapshot::ManufacturerComplianceSnapshot;
use serde::Serialize;
use sha2::{Digest, Sha256};

/// Minimal stable representation of one registry snapshot entry for hashing.
///
/// Fields are in a fixed, explicit declaration order so that
/// `serde_json::to_string` emits a deterministic byte sequence without any
/// map-key sorting.
///
/// Only derive [`Serialize`] — this type is never deserialized.
#[derive(Serialize)]
struct RegistrySnapshotEntry {
    manufacturer_id: String,
    evidence_references: Vec<String>,
    attestation_statuses: Vec<String>,
    is_eligible: bool,
}

/// Computes a deterministic SHA-256 hash of a registry snapshot slice.
///
/// Steps:
/// 1. Derive a [`RegistrySnapshotEntry`] from each snapshot (all four fields).
/// 2. Sort entries by `manufacturer_id` in ascending lexicographic order so
///    the hash is order-independent with respect to the input slice.
/// 3. Serialize the sorted list to compact JSON.
/// 4. Return the lowercase hex SHA-256 digest of the JSON bytes.
///
/// An empty snapshot list produces a stable hash of `"[]"`.
pub fn hash_registry_snapshots(snapshots: &[ManufacturerComplianceSnapshot]) -> String {
    let mut entries: Vec<RegistrySnapshotEntry> = snapshots
        .iter()
        .map(|s| RegistrySnapshotEntry {
            manufacturer_id: s.manufacturer_id.clone(),
            evidence_references: s.evidence_references.clone(),
            attestation_statuses: s.attestation_statuses.clone(),
            is_eligible: s.is_eligible,
        })
        .collect();
    entries.sort_by(|a, b| a.manufacturer_id.cmp(&b.manufacturer_id));
    let json = serde_json::to_string(&entries)
        .expect("RegistrySnapshotEntry serialization must not fail");
    let digest = Sha256::digest(json.as_bytes());
    format!("{:x}", digest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcad_registry::snapshot::ManufacturerComplianceSnapshot;

    fn eligible(id: &str) -> ManufacturerComplianceSnapshot {
        ManufacturerComplianceSnapshot::new(
            id,
            vec!["REF-001".to_string()],
            vec!["verified".to_string()],
            true,
        )
    }

    fn ineligible(id: &str) -> ManufacturerComplianceSnapshot {
        ManufacturerComplianceSnapshot::new(id, vec![], vec![], false)
    }

    // ── determinism ──────────────────────────────────────────────────────────

    #[test]
    fn same_snapshot_produces_same_hash() {
        let snapshots = vec![eligible("mfr-01")];
        assert_eq!(
            hash_registry_snapshots(&snapshots),
            hash_registry_snapshots(&snapshots)
        );
    }

    #[test]
    fn empty_snapshot_produces_stable_hash() {
        let h1 = hash_registry_snapshots(&[]);
        let h2 = hash_registry_snapshots(&[]);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
        assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ── order independence ────────────────────────────────────────────────────

    #[test]
    fn different_input_order_produces_same_hash() {
        let a = eligible("mfr-01");
        let b = eligible("mfr-02");
        let h_ab = hash_registry_snapshots(&[a.clone(), b.clone()]);
        let h_ba = hash_registry_snapshots(&[b.clone(), a.clone()]);
        assert_eq!(h_ab, h_ba);
    }

    #[test]
    fn three_snapshots_in_any_order_produce_same_hash() {
        let a = eligible("mfr-a");
        let b = eligible("mfr-b");
        let c = eligible("mfr-c");
        let h1 = hash_registry_snapshots(&[a.clone(), b.clone(), c.clone()]);
        let h2 = hash_registry_snapshots(&[c.clone(), a.clone(), b.clone()]);
        let h3 = hash_registry_snapshots(&[b.clone(), c.clone(), a.clone()]);
        assert_eq!(h1, h2);
        assert_eq!(h1, h3);
    }

    // ── field mutation → different hash ──────────────────────────────────────

    #[test]
    fn changed_manufacturer_id_produces_different_hash() {
        let original = vec![eligible("mfr-01")];
        let mutated = vec![eligible("mfr-99")];
        assert_ne!(
            hash_registry_snapshots(&original),
            hash_registry_snapshots(&mutated)
        );
    }

    #[test]
    fn changed_is_eligible_produces_different_hash() {
        let original = vec![eligible("mfr-01")];
        let mutated = vec![ineligible("mfr-01")];
        assert_ne!(
            hash_registry_snapshots(&original),
            hash_registry_snapshots(&mutated)
        );
    }

    #[test]
    fn changed_evidence_references_produce_different_hash() {
        let original = vec![ManufacturerComplianceSnapshot::new(
            "mfr-01",
            vec!["ISO-A".to_string()],
            vec!["verified".to_string()],
            true,
        )];
        let mutated = vec![ManufacturerComplianceSnapshot::new(
            "mfr-01",
            vec!["ISO-B".to_string()],
            vec!["verified".to_string()],
            true,
        )];
        assert_ne!(
            hash_registry_snapshots(&original),
            hash_registry_snapshots(&mutated)
        );
    }

    #[test]
    fn changed_attestation_status_produces_different_hash() {
        let original = vec![ManufacturerComplianceSnapshot::new(
            "mfr-01",
            vec!["ISO-A".to_string()],
            vec!["verified".to_string()],
            true,
        )];
        let mutated = vec![ManufacturerComplianceSnapshot::new(
            "mfr-01",
            vec!["ISO-A".to_string()],
            vec!["rejected".to_string()],
            false,
        )];
        assert_ne!(
            hash_registry_snapshots(&original),
            hash_registry_snapshots(&mutated)
        );
    }

    // ── pool size changes ─────────────────────────────────────────────────────

    #[test]
    fn added_snapshot_produces_different_hash() {
        let one = vec![eligible("mfr-01")];
        let two = vec![eligible("mfr-01"), eligible("mfr-02")];
        assert_ne!(
            hash_registry_snapshots(&one),
            hash_registry_snapshots(&two)
        );
    }

    // ── hash format ───────────────────────────────────────────────────────────

    #[test]
    fn hash_is_64_lowercase_hex_chars() {
        let h = hash_registry_snapshots(&[eligible("mfr-01")]);
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(h, h.to_lowercase());
    }
}
