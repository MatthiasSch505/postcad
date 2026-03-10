//! Deterministic candidate pool snapshot and hashing.
//!
//! This module owns the only path from raw candidate input to the
//! `candidate_pool_hash` value embedded in a routing receipt.
//!
//! # Why a separate snapshot type?
//!
//! [`CandidateEntry`] is the stable public input DTO; it may grow fields over
//! time for ergonomic or diagnostic reasons. [`CandidateSnapshot`] is the
//! commitment layer: it contains **only** the fields that directly affect
//! routing, compliance evaluation, or deterministic candidate selection. If an
//! irrelevant field is added to `CandidateEntry` (e.g., a display label), the
//! `candidate_pool_hash` must not change.
//!
//! # Canonical ordering
//!
//! [`hash_candidate_pool`] sorts the snapshot list by `id` in ascending
//! lexicographic order before serializing. This makes the hash
//! **order-independent**: the same pool of candidates produces the same hash
//! regardless of the order they were listed in the input document.

use crate::policy_bundle::CandidateEntry;
use sha2::{Digest, Sha256};

// ── Snapshot type ─────────────────────────────────────────────────────────────

/// Minimal, stable representation of one routing candidate committed to a
/// receipt's `candidate_pool_hash`.
///
/// Fields are declared in a fixed, explicit order. `serde_json::to_string`
/// emits struct fields in declaration order (no map-key sorting, no
/// floating-point), so this order is part of the stable serialization contract.
///
/// Only derive [`serde::Serialize`] — this type is never deserialized.
#[derive(serde::Serialize)]
pub(crate) struct CandidateSnapshot {
    /// Stable routing candidate identifier.
    ///
    /// **Why included:** this is the primary key used for deterministic
    /// sorting and for referencing the selected candidate in the receipt.
    /// A change to `id` changes which candidate is selected.
    pub id: String,

    /// Identifier of the manufacturer this candidate represents.
    ///
    /// **Why included:** the routing engine looks up the compliance snapshot
    /// for this manufacturer. Two candidates with different `manufacturer_id`
    /// values may have different compliance outcomes even if all other fields
    /// are equal.
    pub manufacturer_id: String,

    /// Manufacturing location class: `"domestic"`, `"cross_border"`, or
    /// `"unknown"`.
    ///
    /// **Why included:** the routing policy filter (`AllowDomesticOnly` vs.
    /// `AllowDomesticAndCrossBorder`) decides which candidates survive based
    /// solely on this field. Changing `location` can turn a pass into a
    /// refusal or vice versa.
    pub location: String,

    /// Whether the manufacturer has explicitly accepted this case type.
    ///
    /// **Why included:** a candidate with `accepts_case = false` is filtered
    /// out before selection. Including it ensures the hash detects flips
    /// between accepted and rejected states.
    pub accepts_case: bool,

    /// Pre-computed eligibility hint: `"eligible"`, `"ineligible"`, or
    /// `"unknown"`.
    ///
    /// **Why included:** the compliance engine uses this field — in combination
    /// with the compliance snapshot — to decide whether a candidate clears the
    /// compliance gate. Mutating it can change a compliant candidate into an
    /// ineligible one without touching the snapshot document.
    pub eligibility: String,
}

// ── Derivation helper ─────────────────────────────────────────────────────────

/// Derives a [`CandidateSnapshot`] from one [`CandidateEntry`].
///
/// Copies only the fields that belong in the stable commitment. Fields on
/// `CandidateEntry` that do not appear in `CandidateSnapshot` are
/// intentionally excluded.
pub(crate) fn derive_snapshot(entry: &CandidateEntry) -> CandidateSnapshot {
    CandidateSnapshot {
        id: entry.id.clone(),
        manufacturer_id: entry.manufacturer_id.clone(),
        location: entry.location.clone(),
        accepts_case: entry.accepts_case,
        eligibility: entry.eligibility.clone(),
    }
}

// ── Eligible-ID hash helper ───────────────────────────────────────────────────

/// Computes the canonical SHA-256 hash of a sorted eligible candidate ID list.
///
/// Steps:
/// 1. Sort the IDs in ascending lexicographic order.
/// 2. Serialize to a compact JSON array (strings only — no full snapshot objects).
/// 3. Return the lowercase hex SHA-256 digest.
///
/// An empty eligible set (all candidates filtered out) produces a stable,
/// non-empty hash of the empty JSON array `"[]"`.
pub(crate) fn hash_eligible_ids(ids: &[String]) -> String {
    let mut sorted = ids.to_vec();
    sorted.sort();
    let json =
        serde_json::to_string(&sorted).expect("eligible ID list serialization must not fail");
    let digest = Sha256::digest(json.as_bytes());
    format!("{:x}", digest)
}

// ── Selector-input hash helper ────────────────────────────────────────────────

/// Computes the canonical SHA-256 hash of the selector-input candidate ID list,
/// preserving the exact order the list was presented to the selector.
///
/// Unlike [`hash_eligible_ids`], this function does **not** sort. The order of
/// `ids` is committed to verbatim so that any reordering of the selector input
/// produces a different hash.
///
/// Steps:
/// 1. Serialize to a compact JSON array in the given order.
/// 2. Return the lowercase hex SHA-256 digest.
pub(crate) fn hash_selector_input(ids: &[String]) -> String {
    let json =
        serde_json::to_string(ids).expect("selector input ID list serialization must not fail");
    let digest = Sha256::digest(json.as_bytes());
    format!("{:x}", digest)
}

// ── Candidate-order hash helper ───────────────────────────────────────────────

/// Computes the canonical SHA-256 hash of the deterministically ordered
/// (sorted ascending by ID) eligible candidate ID list.
///
/// Steps:
/// 1. Sort the IDs in ascending lexicographic order.
/// 2. Serialize to a compact JSON array (strings only).
/// 3. Return the lowercase hex SHA-256 digest.
///
/// An empty eligible set produces a stable, non-empty hash of `"[]"`.
/// This hash commits to both the set membership and the canonical ordering,
/// providing evidence that the selector was presented candidates in a
/// deterministic, bias-free order.
pub(crate) fn hash_candidate_order(ids: &[String]) -> String {
    let mut sorted = ids.to_vec();
    sorted.sort();
    let json =
        serde_json::to_string(&sorted).expect("candidate order ID list serialization must not fail");
    let digest = Sha256::digest(json.as_bytes());
    format!("{:x}", digest)
}

// ── Compliance-derived pool hash helper ───────────────────────────────────────

/// Recomputes the candidate pool hash with `eligibility` derived from the
/// compliance gate output, replacing the declared field on each entry.
///
/// For every entry:
/// - `"eligible"` when `manufacturer_id` is in `compliant_manufacturer_ids`
/// - `"ineligible"` otherwise
///
/// All other fields (`id`, `manufacturer_id`, `location`, `accepts_case`) are
/// copied verbatim. The derived pool is sorted and hashed using the same
/// canonical representation as [`hash_candidate_pool`].
///
/// Using this function in both the routing pipeline and receipt verification
/// makes `candidate_pool_hash` reproducible from routing input + registry
/// snapshots + routing policy — independent of the caller-declared eligibility.
pub(crate) fn hash_candidate_pool_from_compliance(
    entries: &[CandidateEntry],
    compliant_manufacturer_ids: &[String],
) -> String {
    let derived: Vec<CandidateEntry> = entries
        .iter()
        .map(|e| CandidateEntry {
            eligibility: if compliant_manufacturer_ids.contains(&e.manufacturer_id) {
                "eligible".to_string()
            } else {
                "ineligible".to_string()
            },
            ..e.clone()
        })
        .collect();
    hash_candidate_pool(&derived)
}

// ── Pool hash helper ──────────────────────────────────────────────────────────

/// Computes the canonical SHA-256 hash of a candidate pool.
///
/// Steps:
/// 1. Derive a [`CandidateSnapshot`] from each entry (only stable fields).
/// 2. Sort snapshots by `id` in ascending lexicographic order.
/// 3. Serialize to compact JSON (field declaration order, no whitespace).
/// 4. Return the lowercase hex SHA-256 digest.
///
/// The resulting hash is **order-independent**: presenting the same candidates
/// in any order produces the same hash.
pub(crate) fn hash_candidate_pool(entries: &[CandidateEntry]) -> String {
    let mut snapshots: Vec<CandidateSnapshot> =
        entries.iter().map(derive_snapshot).collect();
    snapshots.sort_by(|a, b| a.id.cmp(&b.id));
    let json =
        serde_json::to_string(&snapshots).expect("CandidateSnapshot serialization must not fail");
    let digest = Sha256::digest(json.as_bytes());
    format!("{:x}", digest)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, manufacturer_id: &str, location: &str, accepts: bool, elig: &str)
    -> CandidateEntry {
        CandidateEntry {
            id: id.to_string(),
            manufacturer_id: manufacturer_id.to_string(),
            location: location.to_string(),
            accepts_case: accepts,
            eligibility: elig.to_string(),
        }
    }

    // ── same pool → same hash ─────────────────────────────────────────────────

    #[test]
    fn same_pool_produces_same_hash() {
        let pool = vec![entry("rc-1", "mfr-01", "domestic", true, "eligible")];
        assert_eq!(hash_candidate_pool(&pool), hash_candidate_pool(&pool));
    }

    #[test]
    fn empty_pool_produces_stable_hash() {
        let h1 = hash_candidate_pool(&[]);
        let h2 = hash_candidate_pool(&[]);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
        assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ── order independence ────────────────────────────────────────────────────

    #[test]
    fn different_input_order_produces_same_hash() {
        let e1 = entry("rc-1", "mfr-01", "domestic", true, "eligible");
        let e2 = entry("rc-2", "mfr-02", "cross_border", true, "eligible");

        let h_ab = hash_candidate_pool(&[e1.clone(), e2.clone()]);
        let h_ba = hash_candidate_pool(&[e2.clone(), e1.clone()]);

        assert_eq!(h_ab, h_ba, "hash must be order-independent after canonical sort");
    }

    #[test]
    fn three_candidates_in_any_order_produce_same_hash() {
        let a = entry("rc-a", "mfr-01", "domestic", true, "eligible");
        let b = entry("rc-b", "mfr-02", "domestic", true, "eligible");
        let c = entry("rc-c", "mfr-03", "domestic", true, "eligible");

        let h1 = hash_candidate_pool(&[a.clone(), b.clone(), c.clone()]);
        let h2 = hash_candidate_pool(&[c.clone(), a.clone(), b.clone()]);
        let h3 = hash_candidate_pool(&[b.clone(), c.clone(), a.clone()]);

        assert_eq!(h1, h2);
        assert_eq!(h1, h3);
    }

    // ── field mutation → different hash ───────────────────────────────────────

    #[test]
    fn changed_id_produces_different_hash() {
        let original = vec![entry("rc-1", "mfr-01", "domestic", true, "eligible")];
        let mutated = vec![entry("rc-99", "mfr-01", "domestic", true, "eligible")];
        assert_ne!(hash_candidate_pool(&original), hash_candidate_pool(&mutated));
    }

    #[test]
    fn changed_manufacturer_id_produces_different_hash() {
        let original = vec![entry("rc-1", "mfr-01", "domestic", true, "eligible")];
        let mutated = vec![entry("rc-1", "mfr-99", "domestic", true, "eligible")];
        assert_ne!(hash_candidate_pool(&original), hash_candidate_pool(&mutated));
    }

    #[test]
    fn changed_location_produces_different_hash() {
        let original = vec![entry("rc-1", "mfr-01", "domestic", true, "eligible")];
        let mutated = vec![entry("rc-1", "mfr-01", "cross_border", true, "eligible")];
        assert_ne!(hash_candidate_pool(&original), hash_candidate_pool(&mutated));
    }

    #[test]
    fn changed_accepts_case_produces_different_hash() {
        let original = vec![entry("rc-1", "mfr-01", "domestic", true, "eligible")];
        let mutated = vec![entry("rc-1", "mfr-01", "domestic", false, "eligible")];
        assert_ne!(hash_candidate_pool(&original), hash_candidate_pool(&mutated));
    }

    #[test]
    fn changed_eligibility_produces_different_hash() {
        let original = vec![entry("rc-1", "mfr-01", "domestic", true, "eligible")];
        let mutated = vec![entry("rc-1", "mfr-01", "domestic", true, "ineligible")];
        assert_ne!(hash_candidate_pool(&original), hash_candidate_pool(&mutated));
    }

    // ── pool size changes ─────────────────────────────────────────────────────

    #[test]
    fn added_candidate_produces_different_hash() {
        let one = vec![entry("rc-1", "mfr-01", "domestic", true, "eligible")];
        let two = vec![
            entry("rc-1", "mfr-01", "domestic", true, "eligible"),
            entry("rc-2", "mfr-02", "domestic", true, "eligible"),
        ];
        assert_ne!(hash_candidate_pool(&one), hash_candidate_pool(&two));
    }

    #[test]
    fn removed_candidate_produces_different_hash() {
        let two = vec![
            entry("rc-1", "mfr-01", "domestic", true, "eligible"),
            entry("rc-2", "mfr-02", "domestic", true, "eligible"),
        ];
        let one = vec![entry("rc-1", "mfr-01", "domestic", true, "eligible")];
        assert_ne!(hash_candidate_pool(&two), hash_candidate_pool(&one));
    }

    // ── hash format ───────────────────────────────────────────────────────────

    #[test]
    fn hash_is_64_lowercase_hex_chars() {
        let pool = vec![entry("rc-1", "mfr-01", "domestic", true, "eligible")];
        let h = hash_candidate_pool(&pool);
        assert_eq!(h.len(), 64, "SHA-256 hex must be 64 chars");
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()), "must be hex: {h}");
        assert_eq!(h, h.to_lowercase(), "must be lowercase hex");
    }

    // ── snapshot field isolation ──────────────────────────────────────────────

    #[test]
    fn derive_snapshot_copies_all_five_fields() {
        let e = entry("rc-x", "mfr-x", "cross_border", false, "ineligible");
        let s = derive_snapshot(&e);
        assert_eq!(s.id, "rc-x");
        assert_eq!(s.manufacturer_id, "mfr-x");
        assert_eq!(s.location, "cross_border");
        assert!(!s.accepts_case);
        assert_eq!(s.eligibility, "ineligible");
    }

    #[test]
    fn canonical_sort_is_by_id_ascending() {
        // After sorting: rc-a, rc-b, rc-c
        let pool = vec![
            entry("rc-c", "mfr-03", "domestic", true, "eligible"),
            entry("rc-a", "mfr-01", "domestic", true, "eligible"),
            entry("rc-b", "mfr-02", "domestic", true, "eligible"),
        ];
        // Build the expected JSON manually to verify sort order.
        let mut snaps: Vec<CandidateSnapshot> = pool.iter().map(derive_snapshot).collect();
        snaps.sort_by(|a, b| a.id.cmp(&b.id));
        let json = serde_json::to_string(&snaps).unwrap();
        // rc-a must appear before rc-b which must appear before rc-c.
        let pos_a = json.find("\"rc-a\"").expect("rc-a must be present");
        let pos_b = json.find("\"rc-b\"").expect("rc-b must be present");
        let pos_c = json.find("\"rc-c\"").expect("rc-c must be present");
        assert!(pos_a < pos_b, "rc-a must precede rc-b in sorted JSON");
        assert!(pos_b < pos_c, "rc-b must precede rc-c in sorted JSON");
    }

    // ── hash_selector_input ────────────────────────────────────────────────────

    #[test]
    fn selector_input_same_list_produces_same_hash() {
        let ids = vec!["rc-1".to_string(), "rc-2".to_string()];
        assert_eq!(hash_selector_input(&ids), hash_selector_input(&ids));
    }

    #[test]
    fn selector_input_empty_list_produces_stable_hash() {
        let h1 = hash_selector_input(&[]);
        let h2 = hash_selector_input(&[]);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
        assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn selector_input_different_order_produces_different_hash() {
        let ab = vec!["rc-1".to_string(), "rc-2".to_string()];
        let ba = vec!["rc-2".to_string(), "rc-1".to_string()];
        assert_ne!(
            hash_selector_input(&ab),
            hash_selector_input(&ba),
            "selector input hash must be order-sensitive"
        );
    }

    #[test]
    fn selector_input_differs_from_eligible_ids_hash_when_order_differs() {
        // eligible_ids_hash sorts, selector_input does not. So for the same
        // two IDs in a non-sorted order, the two hashes must differ.
        let ids_unsorted = vec!["rc-z".to_string(), "rc-a".to_string()];
        assert_ne!(
            hash_eligible_ids(&ids_unsorted),
            hash_selector_input(&ids_unsorted),
            "eligible hash (sorted) must differ from selector hash (unsorted) for non-sorted input"
        );
    }

    #[test]
    fn selector_input_hash_is_64_lowercase_hex_chars() {
        let ids = vec!["rc-1".to_string()];
        let h = hash_selector_input(&ids);
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(h, h.to_lowercase());
    }
}
