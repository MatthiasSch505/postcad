//! Pilot bundle smoke test.
//!
//! Routes using the pilot fixture set, compares the receipt to
//! `expected_routed.json` value-for-value, then verifies and compares the
//! result to `expected_verify.json`.
//!
//! All assertions run in-process via `tower::ServiceExt::oneshot`.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::util::ServiceExt;

// ── Pilot bundle fixtures ─────────────────────────────────────────────────────

const CASE: &str = include_str!("../../../examples/pilot/case.json");
const REGISTRY: &str = include_str!("../../../examples/pilot/registry_snapshot.json");
const CONFIG: &str = include_str!("../../../examples/pilot/config.json");
const DERIVED_POLICY: &str = include_str!("../../../examples/pilot/derived_policy.json");
const EXPECTED_ROUTED: &str = include_str!("../../../examples/pilot/expected_routed.json");
const EXPECTED_VERIFY: &str = include_str!("../../../examples/pilot/expected_verify.json");

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
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    (status, json)
}

// ── Smoke tests ───────────────────────────────────────────────────────────────

/// Route using the pilot bundle and compare the receipt to expected_routed.json.
#[tokio::test]
async fn pilot_route_matches_expected_routed() {
    let body = json!({
        "case":              serde_json::from_str::<Value>(CASE).unwrap(),
        "registry_snapshot": serde_json::from_str::<Value>(REGISTRY).unwrap(),
        "routing_config":    serde_json::from_str::<Value>(CONFIG).unwrap(),
    });
    let (status, resp) = post_json("/route", body).await;
    assert_eq!(status, StatusCode::OK);

    let got = &resp["receipt"];
    let expected: Value = serde_json::from_str(EXPECTED_ROUTED).unwrap();
    assert_eq!(got, &expected, "receipt must match expected_routed.json");
}

/// Verify the expected receipt against the derived policy and compare to expected_verify.json.
#[tokio::test]
async fn pilot_verify_matches_expected_verify() {
    let receipt: Value = serde_json::from_str(EXPECTED_ROUTED).unwrap();
    let policy: Value = serde_json::from_str(DERIVED_POLICY).unwrap();
    let body = json!({
        "receipt": receipt,
        "case":    serde_json::from_str::<Value>(CASE).unwrap(),
        "policy":  policy,
    });
    let (status, resp) = post_json("/verify", body).await;
    assert_eq!(status, StatusCode::OK);

    let expected: Value = serde_json::from_str(EXPECTED_VERIFY).unwrap();
    assert_eq!(
        resp, expected,
        "verify result must match expected_verify.json"
    );
}

/// Route and then verify using the derived_policy from the same response,
/// confirming round-trip coherence through the pilot endpoints.
#[tokio::test]
async fn pilot_route_then_verify_round_trip() {
    let route_body = json!({
        "case":              serde_json::from_str::<Value>(CASE).unwrap(),
        "registry_snapshot": serde_json::from_str::<Value>(REGISTRY).unwrap(),
        "routing_config":    serde_json::from_str::<Value>(CONFIG).unwrap(),
    });
    let (route_status, route_resp) = post_json("/route", route_body).await;
    assert_eq!(route_status, StatusCode::OK);

    let verify_body = json!({
        "receipt": route_resp["receipt"],
        "case":    serde_json::from_str::<Value>(CASE).unwrap(),
        "policy":  route_resp["derived_policy"],
    });
    let (verify_status, verify_resp) = post_json("/verify", verify_body).await;
    assert_eq!(verify_status, StatusCode::OK);

    let expected: Value = serde_json::from_str(EXPECTED_VERIFY).unwrap();
    assert_eq!(verify_resp, expected);
}
