//! Protocol invariant tests for the pilot workflow.
//!
//! Each test locks one load-bearing invariant of the PostCAD protocol.
//! All inputs come from existing pilot fixtures; no new fixtures are added.
//! All tests run in-process via `tower::ServiceExt::oneshot`; no port binding.
//!
//! Invariants covered:
//!
//! A. Determinism
//!    A1. Same normalized pilot input → identical full receipt JSON on every run.
//!    A2. (Implied by A1) receipt hash is stable across runs.
//!
//! B. Tamper detection via POST /verify
//!    B1. `outcome` field altered in receipt            → verification fails.
//!    B2. `routing_decision_hash` altered in receipt    → verification fails.
//!    B3. `case_fingerprint` altered in receipt         → verification fails.
//!
//! C. Replay after export
//!    C1. Verify succeeds when fed the receipt from a completed export.
//!    C2. Two replay verifies on the same receipt produce identical results.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::util::ServiceExt;

// ── Pilot fixtures ────────────────────────────────────────────────────────────

const PILOT_CASE_JSON: &str = include_str!("../../../examples/pilot/case.json");
const REGISTRY_JSON: &str = include_str!("../../../examples/pilot/registry_snapshot.json");
const CONFIG_JSON: &str = include_str!("../../../examples/pilot/config.json");

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_app(tmp: &tempfile::TempDir) -> axum::Router {
    postcad_service::app_with_all_stores(
        Arc::new(postcad_service::CaseStore::new(tmp.path().join("cases"))),
        Arc::new(postcad_service::ReceiptStore::new(
            tmp.path().join("receipts"),
        )),
        Arc::new(postcad_service::DispatchStore::new(
            tmp.path().join("dispatch"),
        )),
        Arc::new(postcad_service::PolicyStore::new(
            tmp.path().join("policies"),
        )),
        Arc::new(postcad_service::VerificationStore::new(
            tmp.path().join("verification"),
        )),
        Arc::new(postcad_service::DispatchCommitmentStore::new(
            tmp.path().join("commitments"),
        )),
    )
}

async fn post_json(app: axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

async fn get_json(app: axum::Router, uri: &str) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

/// Canonical normalized pilot route body (uses pilot fixtures).
fn normalized_route_body() -> Value {
    json!({
        "pilot_case": {
            "case_id":          "f1000001-0000-0000-0000-000000000001",
            "restoration_type": "crown",
            "material":         "zirconia",
            "jurisdiction":     "DE"
        },
        "registry_snapshot": serde_json::from_str::<Value>(REGISTRY_JSON).unwrap(),
        "routing_config":    serde_json::from_str::<Value>(CONFIG_JSON).unwrap(),
    })
}

/// Route, assert success, return (receipt, derived_policy).
async fn route_pilot(app: axum::Router) -> (Value, Value) {
    let (status, resp) = post_json(app, "/pilot/route-normalized", normalized_route_body()).await;
    assert_eq!(status, StatusCode::OK, "pilot route must succeed: {resp}");
    assert_eq!(resp["receipt"]["outcome"], "routed");
    (resp["receipt"].clone(), resp["derived_policy"].clone())
}

// ── A: Determinism ────────────────────────────────────────────────────────────

/// A1 + A2.
/// Two successive runs with identical normalized pilot input must produce
/// bit-identical receipt JSON (not just the same hash).
#[tokio::test]
async fn normalized_route_receipt_is_fully_deterministic() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (r1, _) = route_pilot(make_app(&tmp)).await;
    let (r2, _) = route_pilot(make_app(&tmp)).await;

    assert_eq!(
        r1, r2,
        "full receipt JSON must be identical across runs for the same input"
    );
    assert_eq!(
        r1["receipt_hash"], r2["receipt_hash"],
        "receipt_hash must be stable across runs"
    );
    assert_eq!(
        r1["selected_candidate_id"], r2["selected_candidate_id"],
        "routing decision must be stable across runs"
    );
}

// ── B: Tamper detection ───────────────────────────────────────────────────────

/// Helper: route once, then verify a (possibly tampered) receipt.
/// Returns the (status, response) from POST /verify.
async fn verify_receipt(receipt: Value, policy: Value) -> (StatusCode, Value) {
    post_json(
        postcad_service::app(),
        "/verify",
        json!({
            "receipt": receipt,
            "case":    serde_json::from_str::<Value>(PILOT_CASE_JSON).unwrap(),
            "policy":  policy,
        }),
    )
    .await
}

/// B1. Altering any content field in the receipt must cause verification to fail.
/// Here we change `outcome`; the `routing_decision_hash` committed in the receipt
/// no longer matches the recomputed value.
#[tokio::test]
async fn verify_fails_if_receipt_outcome_altered() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (receipt, policy) = route_pilot(make_app(&tmp)).await;

    let mut tampered = receipt.clone();
    tampered["outcome"] = json!("refused");

    let (status, resp) = verify_receipt(tampered, policy).await;

    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "tampered receipt must be rejected: {resp}"
    );
    assert_eq!(resp["result"], "FAILED");
    assert!(
        resp["error"]["code"].is_string(),
        "error code must be present"
    );
}

/// B2. Altering `routing_decision_hash` directly in the receipt must cause
/// verification to fail. The verifier recomputes the receipt hash over all
/// fields; any field mutation is caught at `receipt_canonicalization_mismatch`
/// before the per-field checks are reached.
#[tokio::test]
async fn verify_fails_if_routing_decision_hash_altered() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (receipt, policy) = route_pilot(make_app(&tmp)).await;

    let mut tampered = receipt.clone();
    tampered["routing_decision_hash"] = json!(
        "0000000000000000000000000000000000000000000000000000000000000000"
    );

    let (status, resp) = verify_receipt(tampered, policy).await;

    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "altered routing_decision_hash must be rejected: {resp}"
    );
    assert_eq!(resp["result"], "FAILED");
    assert_eq!(resp["error"]["code"], "receipt_canonicalization_mismatch");
}

/// B3. Altering `case_fingerprint` in the receipt must cause verification to fail.
/// As with all field mutations, the outer `receipt_canonicalization_mismatch`
/// check fires first because the receipt hash covers all committed fields.
#[tokio::test]
async fn verify_fails_if_case_fingerprint_altered() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (receipt, policy) = route_pilot(make_app(&tmp)).await;

    let mut tampered = receipt.clone();
    tampered["case_fingerprint"] = json!(
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    );

    let (status, resp) = verify_receipt(tampered, policy).await;

    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "altered case_fingerprint must be rejected: {resp}"
    );
    assert_eq!(resp["result"], "FAILED");
    assert_eq!(resp["error"]["code"], "receipt_canonicalization_mismatch");
}

// ── C: Replay verification after export ──────────────────────────────────────

/// C1 + C2.
/// After the full commitment lifecycle (create → approve → export) the
/// original receipt must still verify successfully, and two successive
/// replay verifications must return identical results.
///
/// This proves that:
///   - Dispatch export does not alter receipt commitments.
///   - Replay verification is deterministic.
#[tokio::test]
async fn replay_verify_after_export_is_deterministic() {
    let tmp = tempfile::TempDir::new().unwrap();

    // Route: obtain receipt + derived_policy.
    let (receipt, derived_policy) = route_pilot(make_app(&tmp)).await;
    let receipt_hash = receipt["receipt_hash"].as_str().unwrap().to_string();

    // Create dispatch.
    let (create_status, create_resp) = post_json(
        make_app(&tmp),
        "/dispatch/create",
        json!({
            "receipt": receipt,
            "case":    serde_json::from_str::<Value>(PILOT_CASE_JSON).unwrap(),
            "policy":  derived_policy.clone(),
        }),
    )
    .await;
    assert_eq!(create_status, StatusCode::OK, "dispatch create: {create_resp}");
    let dispatch_id = create_resp["dispatch_id"].as_str().unwrap().to_string();

    // Approve.
    let (approve_status, _) = post_json(
        make_app(&tmp),
        &format!("/dispatch/{dispatch_id}/approve"),
        json!({"approved_by": "reviewer"}),
    )
    .await;
    assert_eq!(approve_status, StatusCode::OK);

    // Export.
    let (export_status, export_resp) =
        get_json(make_app(&tmp), &format!("/dispatch/{dispatch_id}/export")).await;
    assert_eq!(export_status, StatusCode::OK, "export: {export_resp}");
    assert_eq!(export_resp["status"], "exported");

    // Export must not alter the receipt commitment: receipt_hash in export
    // must equal the original receipt_hash from routing.
    assert_eq!(
        export_resp["receipt_hash"].as_str().unwrap(),
        receipt_hash,
        "export must carry the original receipt_hash without modification"
    );

    // Replay verify (run 1): feed the original receipt back into /verify.
    let (v1_status, v1_resp) = verify_receipt(receipt.clone(), derived_policy.clone()).await;
    assert_eq!(v1_status, StatusCode::OK, "replay verify 1: {v1_resp}");
    assert_eq!(
        v1_resp["result"],
        "VERIFIED",
        "replay verify must succeed after export"
    );

    // Replay verify (run 2): must return identical result.
    let (v2_status, v2_resp) = verify_receipt(receipt.clone(), derived_policy.clone()).await;
    assert_eq!(v2_status, StatusCode::OK, "replay verify 2: {v2_resp}");
    assert_eq!(
        v1_resp, v2_resp,
        "two replay verifications must produce identical results"
    );
}
