//! Service contract tests — prove the HTTP wrapper returns the same deterministic
//! artifacts as the CLI/kernel path for identical inputs.
//!
//! All tests run in-process via `tower::ServiceExt::oneshot`; no port binding.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::util::ServiceExt;

// ── Canonical fixture corpus ──────────────────────────────────────────────────

const CASE_JSON: &str = include_str!("../../../fixtures/case.json");
const POLICY_JSON: &str = include_str!("../../../fixtures/policy.json");
const EXPECTED_ROUTED_JSON: &str = include_str!("../../../fixtures/expected_routed.json");
const EXPECTED_MANIFEST_JSON: &str = include_str!("../../../fixtures/expected_manifest.json");

// ── Test helpers ──────────────────────────────────────────────────────────────

async fn post_json(uri: &str, body: Value) -> (StatusCode, Value) {
    let app = postcad_service::app();
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    (status, json)
}

async fn get_json(uri: &str) -> (StatusCode, Value) {
    let app = postcad_service::app();
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    (status, json)
}

fn canonical_request_body() -> Value {
    json!({
        "case": serde_json::from_str::<Value>(CASE_JSON).unwrap(),
        "policy": serde_json::from_str::<Value>(POLICY_JSON).unwrap(),
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// The receipt returned by POST /route-case must equal the frozen CLI artifact
/// value-for-value, proving the service is a transparent kernel wrapper.
#[tokio::test]
async fn route_case_receipt_matches_kernel_artifact() {
    let (status, body) = post_json("/route-case", canonical_request_body()).await;
    assert_eq!(status, StatusCode::OK);

    let got = &body["receipt"];
    let expected: Value = serde_json::from_str(EXPECTED_ROUTED_JSON).unwrap();
    assert_eq!(got, &expected, "receipt must match frozen CLI artifact");
}

/// A policy with no eligible candidates must return HTTP 200 with outcome=refused.
/// Refusal is a valid domain outcome, not a service error.
#[tokio::test]
async fn route_case_refusal_is_200_with_refused_outcome() {
    let refusal_policy = json!({
        "jurisdiction": "DE",
        "routing_policy": "allow_domestic_and_cross_border",
        "candidates": [{"id": "rc-de-01", "manufacturer_id": "mfr-de-01", "location": "domestic", "accepts_case": true, "eligibility": "eligible"}],
        "snapshots": [{"manufacturer_id": "mfr-de-01", "evidence_references": [], "attestation_statuses": [], "is_eligible": false}]
    });
    let body = json!({
        "case": serde_json::from_str::<Value>(CASE_JSON).unwrap(),
        "policy": refusal_policy,
    });

    let (status, resp) = post_json("/route-case", body).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["receipt"]["outcome"], "refused");
    assert_eq!(resp["receipt"]["refusal_code"], "no_eligible_candidates");
}

/// POST /verify-receipt with the frozen routed receipt + original inputs must
/// return HTTP 200 {"result":"VERIFIED"} — same contract as the CLI path.
#[tokio::test]
async fn verify_receipt_frozen_fixture_accepted() {
    let frozen: Value = serde_json::from_str(EXPECTED_ROUTED_JSON).unwrap();
    let body = json!({
        "receipt": frozen,
        "case": serde_json::from_str::<Value>(CASE_JSON).unwrap(),
        "policy": serde_json::from_str::<Value>(POLICY_JSON).unwrap(),
    });

    let (status, resp) = post_json("/verify-receipt", body).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["result"], "VERIFIED");
}

/// POST /route-case → extract receipt → POST /verify-receipt with same inputs
/// must return VERIFIED, proving round-trip coherence through the service layer.
#[tokio::test]
async fn route_case_to_verify_receipt_round_trip() {
    let (_, route_resp) = post_json("/route-case", canonical_request_body()).await;
    let receipt = route_resp["receipt"].clone();

    let verify_body = json!({
        "receipt": receipt,
        "case": serde_json::from_str::<Value>(CASE_JSON).unwrap(),
        "policy": serde_json::from_str::<Value>(POLICY_JSON).unwrap(),
    });
    let (status, verify_resp) = post_json("/verify-receipt", verify_body).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(verify_resp["result"], "VERIFIED");
}

/// Passing the frozen receipt with a drifted snapshot (changed evidence) must
/// fail with registry_snapshot_hash_mismatch — same as the CLI drift detection
/// test. The receipt itself is unmodified so receipt_hash still validates.
#[tokio::test]
async fn verify_receipt_drifted_snapshot_returns_registry_hash_mismatch() {
    let frozen: Value = serde_json::from_str(EXPECTED_ROUTED_JSON).unwrap();

    // Same jurisdiction + routing_policy + candidates so policy_fingerprint is
    // unchanged. Only snapshot evidence differs — changes registry_snapshot_hash.
    let drifted_policy = json!({
        "jurisdiction": "DE",
        "routing_policy": "allow_domestic_and_cross_border",
        "candidates": [{"id": "rc-de-01", "manufacturer_id": "mfr-de-01", "location": "domestic", "accepts_case": true, "eligibility": "eligible"}],
        "snapshots": [{"manufacturer_id": "mfr-de-01", "evidence_references": ["DRIFTED-REF"], "attestation_statuses": ["verified"], "is_eligible": true}]
    });

    let body = json!({
        "receipt": frozen,
        "case": serde_json::from_str::<Value>(CASE_JSON).unwrap(),
        "policy": drifted_policy,
    });

    let (status, resp) = post_json("/verify-receipt", body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(resp["result"], "FAILED");
    assert_eq!(resp["error"]["code"], "registry_snapshot_hash_mismatch");
}

/// GET /protocol-manifest must return the frozen manifest fixture value-for-value.
#[tokio::test]
async fn protocol_manifest_matches_frozen_fixture() {
    let (status, body) = get_json("/protocol-manifest").await;
    assert_eq!(status, StatusCode::OK);

    let expected: Value = serde_json::from_str(EXPECTED_MANIFEST_JSON).unwrap();
    assert_eq!(body, expected, "manifest must match frozen fixture");
}

/// A malformed case field must return HTTP 422 with a parse_error code.
#[tokio::test]
async fn route_case_malformed_case_returns_422() {
    let body = json!({
        "case": {"patient_country": "germany", "manufacturer_country": "germany",
                 "material": "INVALID_MATERIAL", "procedure": "crown",
                 "file_type": "stl"},
        "policy": serde_json::from_str::<Value>(POLICY_JSON).unwrap(),
    });

    let (status, resp) = post_json("/route-case", body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(resp["error"]["code"], "parse_error");
}
