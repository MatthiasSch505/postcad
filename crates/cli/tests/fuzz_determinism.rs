//! Property-based fuzz tests for routing determinism and verification soundness.
//!
//! Uses `proptest` to enforce four invariants across randomized structurally
//! valid inputs — without subprocess spawning (library level, fast):
//!
//! 1. **Determinism** — `route_case_from_policy_json(c, p) ==
//!    route_case_from_policy_json(c, p)` for any inputs, including
//!    multi-candidate policies.
//!
//! 2. **Verifiability** — `verify_receipt_from_policy_json` returns `Ok` for
//!    every receipt freshly produced by `route_case_from_policy_json`.
//!
//! 3. **Tamper detection** — zeroing any single committed hash field (while
//!    recomputing `receipt_hash` to bypass artifact-integrity) deterministically
//!    produces the expected stable failure code.
//!
//! 4. **No panic** — no combination of arbitrary string inputs may cause a
//!    panic; `route_case_from_policy_json` and `verify_receipt_from_policy_json`
//!    must always return `Ok` or `Err`, never unwind.
//!
//! ## Generator design
//!
//! - `arb_case_json` / `arb_policy_json` — fully randomised domain fields;
//!   routing may succeed (routed or refused) or return a parse/validation error.
//! - `arb_policy_json_multi` — 1–3 candidates with varied eligibility and
//!   location; stresses the ordering invariant.
//! - `arb_routable_pair` — always produces a *routed* receipt; uses a single
//!   domestic eligible candidate with `is_eligible: true`.  `ComplianceGate`
//!   gates solely on `is_eligible`, so jurisdiction is irrelevant here.

use postcad_cli::{route_case_from_policy_json, verify_receipt_from_policy_json};
use proptest::prelude::*;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

// ── Helper ────────────────────────────────────────────────────────────────────

/// Recomputes `receipt_hash` in-place after other fields have been tampered.
///
/// Removes `receipt_hash` from the object, canonicalises with `serde_json`
/// (which sorts keys alphabetically via BTreeMap), then returns the SHA-256
/// hex digest — matching the production implementation exactly.
fn recompute_receipt_hash(v: &Value) -> String {
    let mut obj = v.clone();
    obj.as_object_mut().unwrap().remove("receipt_hash");
    let canonical = serde_json::to_string(&obj).unwrap();
    format!("{:x}", Sha256::digest(canonical.as_bytes()))
}

// ── Leaf strategies ───────────────────────────────────────────────────────────

fn arb_jurisdiction() -> impl Strategy<Value = &'static str> {
    prop_oneof![Just("DE"), Just("US"), Just("FR"), Just("JP"), Just("GB")]
}

fn arb_routing_policy() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("allow_domestic_and_cross_border"),
        Just("allow_domestic_only"),
    ]
}

fn arb_country() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("germany"),
        Just("united_states"),
        Just("france"),
        Just("japan"),
        Just("united_kingdom"),
    ]
}

fn arb_material() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("zirconia"),
        Just("pmma"),
        Just("emax"),
        Just("cobalt_chrome"),
        Just("titanium"),
    ]
}

fn arb_procedure() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("crown"),
        Just("bridge"),
        Just("veneer"),
        Just("implant"),
        Just("denture"),
    ]
}

fn arb_file_type() -> impl Strategy<Value = &'static str> {
    // "3mf" is intentionally omitted; its serde rename is implementation-defined
    // and the other three variants are sufficient for full coverage.
    prop_oneof![Just("stl"), Just("obj"), Just("ply")]
}

fn arb_location() -> impl Strategy<Value = &'static str> {
    prop_oneof![Just("domestic"), Just("cross_border")]
}

fn arb_policy_version() -> impl Strategy<Value = Option<&'static str>> {
    prop_oneof![Just(None), Just(Some("v1")), Just(Some("2024-01"))]
}

// ── Composite strategies ──────────────────────────────────────────────────────

prop_compose! {
    /// Structurally valid case JSON with all domain fields randomised.
    ///
    /// `jurisdiction` and `routing_policy` in the case are ignored by
    /// `route_case_from_policy_json` (both come from the policy bundle);
    /// they are included for structural completeness only.
    fn arb_case_json()(
        jur  in arb_jurisdiction(),
        rp   in arb_routing_policy(),
        pc   in arb_country(),
        mc   in arb_country(),
        mat  in arb_material(),
        proc in arb_procedure(),
        ft   in arb_file_type(),
    ) -> String {
        json!({
            "case_id":              "f0000000-0000-0000-0000-000000000001",
            "jurisdiction":         jur,
            "routing_policy":       rp,
            "patient_country":      pc,
            "manufacturer_country": mc,
            "material":             mat,
            "procedure":            proc,
            "file_type":            ft,
        }).to_string()
    }
}

prop_compose! {
    /// Policy bundle with a single candidate whose eligibility is randomised.
    ///
    /// Routing may produce `routed`, `refused`, or a snapshot-validation error
    /// depending on the combination of fields.
    fn arb_policy_json()(
        jur     in arb_jurisdiction(),
        rp      in arb_routing_policy(),
        ver     in arb_policy_version(),
        loc     in arb_location(),
        is_elig in proptest::bool::ANY,
        accepts in proptest::bool::ANY,
    ) -> String {
        let eligibility = if is_elig { "eligible" } else { "ineligible" };
        let attests: &[&str] = if is_elig { &["verified"] } else { &[] };
        json!({
            "jurisdiction":   jur,
            "routing_policy": rp,
            "policy_version": ver,
            "candidates": [{
                "id":              "rc-fuzz-01",
                "manufacturer_id": "mfr-fuzz-01",
                "location":        loc,
                "accepts_case":    accepts,
                "eligibility":     eligibility,
            }],
            "snapshots": [{
                "manufacturer_id":      "mfr-fuzz-01",
                "evidence_references":  ["ISO-9001-2024"],
                "attestation_statuses": attests,
                "is_eligible":          is_elig,
            }],
        }).to_string()
    }
}

prop_compose! {
    /// Policy bundle with 1–3 candidates with varied eligibility and location.
    ///
    /// Each candidate has a unique `manufacturer_id`; the corresponding snapshot
    /// mirrors its eligibility.  Stresses the candidate-ordering invariant and
    /// the multi-candidate routing path.
    fn arb_policy_json_multi()(
        jur  in arb_jurisdiction(),
        rp   in arb_routing_policy(),
        ver  in arb_policy_version(),
        cands in proptest::collection::vec(
            (arb_location(), proptest::bool::ANY, proptest::bool::ANY),
            1..=3,
        ),
    ) -> String {
        let candidates: Vec<Value> = cands.iter().enumerate().map(|(i, (loc, is_elig, accepts))| {
            json!({
                "id":              format!("rc-fuzz-{:02}", i + 1),
                "manufacturer_id": format!("mfr-fuzz-{:02}", i + 1),
                "location":        loc,
                "accepts_case":    accepts,
                "eligibility":     if *is_elig { "eligible" } else { "ineligible" },
            })
        }).collect();

        let snapshots: Vec<Value> = cands.iter().enumerate().map(|(i, (_, is_elig, _))| {
            let attests: &[&str] = if *is_elig { &["verified"] } else { &[] };
            json!({
                "manufacturer_id":      format!("mfr-fuzz-{:02}", i + 1),
                "evidence_references":  ["ISO-9001-2024"],
                "attestation_statuses": attests,
                "is_eligible":          is_elig,
            })
        }).collect();

        json!({
            "jurisdiction":   jur,
            "routing_policy": rp,
            "policy_version": ver,
            "candidates": candidates,
            "snapshots":  snapshots,
        }).to_string()
    }
}

prop_compose! {
    /// (case, policy) pair guaranteed to produce a *routed* receipt.
    ///
    /// Routing is guaranteed because:
    ///   - `eligibility: "eligible"` + `accepts_case: true` → candidate accepted
    ///   - `is_eligible: true` → passes `ComplianceGate` (the only compliance check)
    ///   - `location: "domestic"` → allowed by both routing policies
    ///
    /// The policy's `routing_policy` is randomised to cover both variants;
    /// jurisdiction is randomised because compliance is `is_eligible`-only.
    /// Case domain fields (patient/manufacturer country, material, procedure,
    /// file type) are fully randomised; they do not affect routing outcomes in
    /// the policy-bundle path.
    fn arb_routable_pair()(
        jur  in arb_jurisdiction(),
        rp   in arb_routing_policy(),
        ver  in arb_policy_version(),
        pc   in arb_country(),
        mc   in arb_country(),
        mat  in arb_material(),
        proc in arb_procedure(),
        ft   in arb_file_type(),
    ) -> (String, String) {
        let case = json!({
            "case_id":              "f0000000-0000-0000-0000-000000000001",
            "jurisdiction":         jur,
            "routing_policy":       "allow_domestic_and_cross_border",
            "patient_country":      pc,
            "manufacturer_country": mc,
            "material":             mat,
            "procedure":            proc,
            "file_type":            ft,
        }).to_string();

        let policy = json!({
            "jurisdiction":   jur,
            "routing_policy": rp,
            "policy_version": ver,
            "candidates": [{
                "id":              "rc-fuzz-01",
                "manufacturer_id": "mfr-fuzz-01",
                "location":        "domestic",
                "accepts_case":    true,
                "eligibility":     "eligible",
            }],
            "snapshots": [{
                "manufacturer_id":      "mfr-fuzz-01",
                "evidence_references":  ["ISO-9001-2024"],
                "attestation_statuses": ["verified"],
                "is_eligible":          true,
            }],
        }).to_string();

        (case, policy)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    // ── Invariant 1: Routing determinism ─────────────────────────────────────

    /// Two independent calls to `route_case_from_policy_json` with the same
    /// arguments must produce byte-for-byte identical receipts.
    ///
    /// This covers both the `Ok` path (identical receipt JSON values) and the
    /// `Err` path (identical error codes).  A flip between `Ok` and `Err` on
    /// repeated identical calls is an unconditional failure.
    #[test]
    fn prop_route_case_is_deterministic(
        case   in arb_case_json(),
        policy in arb_policy_json(),
    ) {
        let r1 = route_case_from_policy_json(&case, &policy);
        let r2 = route_case_from_policy_json(&case, &policy);

        match (r1, r2) {
            (Ok(r1), Ok(r2)) => {
                prop_assert_eq!(
                    serde_json::to_value(&r1).unwrap(),
                    serde_json::to_value(&r2).unwrap(),
                    "route-case must produce identical receipts on repeated calls",
                );
            }
            (Err(e1), Err(e2)) => {
                prop_assert_eq!(
                    e1.code(), e2.code(),
                    "routing errors must be identical on repeated calls",
                );
            }
            (Ok(_), Err(e)) => {
                prop_assert!(false, "first call Ok, second call Err({})", e.code());
            }
            (Err(e), Ok(_)) => {
                prop_assert!(false, "first call Err({}), second call Ok", e.code());
            }
        }
    }

    /// Determinism holds with multi-candidate policies (varied eligibility and
    /// location), stressing the deterministic-hash selector and sort invariant.
    #[test]
    fn prop_route_case_is_deterministic_multi_candidate(
        case   in arb_case_json(),
        policy in arb_policy_json_multi(),
    ) {
        let r1 = route_case_from_policy_json(&case, &policy);
        let r2 = route_case_from_policy_json(&case, &policy);

        match (r1, r2) {
            (Ok(r1), Ok(r2)) => {
                prop_assert_eq!(
                    serde_json::to_value(&r1).unwrap(),
                    serde_json::to_value(&r2).unwrap(),
                    "route-case must be deterministic with multiple candidates",
                );
            }
            (Err(e1), Err(e2)) => {
                prop_assert_eq!(e1.code(), e2.code());
            }
            _ => {
                prop_assert!(false, "inconsistent routing across two identical calls");
            }
        }
    }

    // ── Invariant 2: Verifiability ────────────────────────────────────────────

    /// Every receipt freshly routed via `arb_routable_pair` must be accepted by
    /// `verify_receipt_from_policy_json` with the same inputs.
    ///
    /// The routable-pair generator guarantees a `routed` outcome, so `.expect`
    /// is used to surface generator bugs rather than silently skipping cases.
    #[test]
    fn prop_valid_receipt_always_verifies(
        (case, policy) in arb_routable_pair(),
    ) {
        let receipt = route_case_from_policy_json(&case, &policy)
            .expect("arb_routable_pair must always produce a valid routing");
        let receipt_json = serde_json::to_string(&receipt).unwrap();

        let result = verify_receipt_from_policy_json(&receipt_json, &case, &policy);
        prop_assert!(
            result.is_ok(),
            "freshly-routed receipt must always verify; error: {:?}", result.err(),
        );
    }

    /// `verify_receipt_from_policy_json` must succeed for any receipt produced
    /// by `route_case_from_policy_json`, regardless of outcome (routed or
    /// refused) and regardless of policy randomisation.
    ///
    /// Skips test cases where routing itself fails (parse/validation error)
    /// since there is no receipt to verify.
    #[test]
    fn prop_any_fresh_receipt_always_verifies(
        case   in arb_case_json(),
        policy in arb_policy_json(),
    ) {
        if let Ok(receipt) = route_case_from_policy_json(&case, &policy) {
            let receipt_json = serde_json::to_string(&receipt).unwrap();
            let result = verify_receipt_from_policy_json(&receipt_json, &case, &policy);
            prop_assert!(
                result.is_ok(),
                "freshly-routed receipt must always verify; error: {:?}", result.err(),
            );
        }
        // Routing error (parse / invalid_snapshot): no receipt to verify — skip.
    }

    // ── Invariant 3: Tamper detection ─────────────────────────────────────────

    /// Zeroing `case_fingerprint` (then recomputing `receipt_hash` to bypass
    /// artifact-integrity) must always produce `case_fingerprint_mismatch`.
    ///
    /// `receipt_hash` recomputation is required so the verifier passes step 1b
    /// and reaches the fingerprint check at step 2 where the zeroed value is
    /// compared against the recomputed fingerprint.
    #[test]
    fn prop_tampered_case_fingerprint_always_fails_verification(
        (case, policy) in arb_routable_pair(),
    ) {
        let receipt = route_case_from_policy_json(&case, &policy)
            .expect("arb_routable_pair must always produce a valid routing");
        let mut rv = serde_json::to_value(&receipt).unwrap();

        rv["case_fingerprint"] =
            json!("0000000000000000000000000000000000000000000000000000000000000000");
        rv["receipt_hash"] = json!(recompute_receipt_hash(&rv));

        let tampered = serde_json::to_string(&rv).unwrap();
        let err = verify_receipt_from_policy_json(&tampered, &case, &policy)
            .expect_err("tampered case_fingerprint must fail verification");

        prop_assert_eq!(
            err.code, "case_fingerprint_mismatch",
            "expected case_fingerprint_mismatch, got {:?}", err,
        );
    }

    /// Zeroing `registry_snapshot_hash` (then recomputing `receipt_hash`) must
    /// always produce `registry_snapshot_hash_mismatch`.
    #[test]
    fn prop_tampered_registry_snapshot_hash_always_fails_verification(
        (case, policy) in arb_routable_pair(),
    ) {
        let receipt = route_case_from_policy_json(&case, &policy)
            .expect("arb_routable_pair must always produce a valid routing");
        let mut rv = serde_json::to_value(&receipt).unwrap();

        rv["registry_snapshot_hash"] =
            json!("0000000000000000000000000000000000000000000000000000000000000000");
        rv["receipt_hash"] = json!(recompute_receipt_hash(&rv));

        let tampered = serde_json::to_string(&rv).unwrap();
        let err = verify_receipt_from_policy_json(&tampered, &case, &policy)
            .expect_err("tampered registry_snapshot_hash must fail verification");

        prop_assert_eq!(
            err.code, "registry_snapshot_hash_mismatch",
            "expected registry_snapshot_hash_mismatch, got {:?}", err,
        );
    }

    /// Zeroing `routing_decision_hash` (then recomputing `receipt_hash`) must
    /// always produce `routing_decision_hash_mismatch`.
    ///
    /// The verifier recomputes the decision hash from the receipt's own
    /// `(outcome, policy_version, refusal_code, routing_kernel_version,
    /// selected_candidate_id)` at step 1e (self-contained check) and compares
    /// it to the stored value.  A zeroed hash triggers a mismatch.
    #[test]
    fn prop_tampered_routing_decision_hash_always_fails_verification(
        (case, policy) in arb_routable_pair(),
    ) {
        let receipt = route_case_from_policy_json(&case, &policy)
            .expect("arb_routable_pair must always produce a valid routing");
        let mut rv = serde_json::to_value(&receipt).unwrap();

        rv["routing_decision_hash"] =
            json!("0000000000000000000000000000000000000000000000000000000000000000");
        rv["receipt_hash"] = json!(recompute_receipt_hash(&rv));

        let tampered = serde_json::to_string(&rv).unwrap();
        let err = verify_receipt_from_policy_json(&tampered, &case, &policy)
            .expect_err("tampered routing_decision_hash must fail verification");

        prop_assert_eq!(
            err.code, "routing_decision_hash_mismatch",
            "expected routing_decision_hash_mismatch, got {:?}", err,
        );
    }

    // ── Invariant 4: No panic ─────────────────────────────────────────────────

    /// `route_case_from_policy_json` must not panic on arbitrary case JSON.
    ///
    /// Proptest generates random Unicode strings; the function must return
    /// `Ok` or `Err` and must not unwind.  A canonical valid policy is used so
    /// that any parse failure is attributable to the case input.
    #[test]
    fn prop_no_panic_on_arbitrary_case_json(case in any::<String>()) {
        const VALID_POLICY: &str = concat!(
            r#"{"jurisdiction":"DE","routing_policy":"allow_domestic_and_cross_border","#,
            r#""candidates":[{"id":"rc-de-01","manufacturer_id":"mfr-de-01","#,
            r#""location":"domestic","accepts_case":true,"eligibility":"eligible"}],"#,
            r#""snapshots":[{"manufacturer_id":"mfr-de-01","#,
            r#""evidence_references":["ISO-9001-2024"],"#,
            r#""attestation_statuses":["verified"],"is_eligible":true}]}"#,
        );
        let _ = route_case_from_policy_json(&case, VALID_POLICY);
    }

    /// `route_case_from_policy_json` must not panic on arbitrary policy JSON.
    ///
    /// A canonical valid case is used so that any error is attributable to the
    /// policy input.
    #[test]
    fn prop_no_panic_on_arbitrary_policy_json(policy in any::<String>()) {
        const VALID_CASE: &str = concat!(
            r#"{"case_id":"a1b2c3d4-0000-0000-0000-000000000001","#,
            r#""jurisdiction":"DE","routing_policy":"allow_domestic_and_cross_border","#,
            r#""patient_country":"germany","manufacturer_country":"germany","#,
            r#""material":"zirconia","procedure":"crown","file_type":"stl"}"#,
        );
        let _ = route_case_from_policy_json(VALID_CASE, &policy);
    }

    /// `verify_receipt_from_policy_json` must not panic on any combination of
    /// arbitrary string inputs.
    #[test]
    fn prop_no_panic_on_arbitrary_verify_inputs(
        receipt in any::<String>(),
        case    in any::<String>(),
        policy  in any::<String>(),
    ) {
        let _ = verify_receipt_from_policy_json(&receipt, &case, &policy);
    }
}
