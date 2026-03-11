//! Pilot HTTP endpoint tests — /health, /version, /route, /verify.
//!
//! These tests exercise the four pilot-facing endpoints added as thin wrappers
//! around the existing registry-backed routing and verification paths.
//!
//! All tests run in-process via `tower::ServiceExt::oneshot`; no port binding.
//! Inputs are drawn from the frozen protocol conformance vectors so the output
//! is anchored to the same kernel artifacts verified elsewhere.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::util::ServiceExt;

// ── Frozen protocol-vector inputs ─────────────────────────────────────────────

/// v01 — single eligible domestic manufacturer; expected: routed, mfr-de-001.
const V01_CASE: &str = include_str!("../../../tests/protocol_vectors/v01_basic_routing/case.json");
const V01_REGISTRY: &str =
    include_str!("../../../tests/protocol_vectors/v01_basic_routing/registry_snapshot.json");
const V01_CONFIG: &str =
    include_str!("../../../tests/protocol_vectors/v01_basic_routing/policy.json");

/// v03 — US-only registry, DE case; expected: refused, no_jurisdiction_match.
const V03_CASE: &str =
    include_str!("../../../tests/protocol_vectors/v03_jurisdiction_refusal/case.json");
const V03_REGISTRY: &str =
    include_str!("../../../tests/protocol_vectors/v03_jurisdiction_refusal/registry_snapshot.json");
const V03_CONFIG: &str =
    include_str!("../../../tests/protocol_vectors/v03_jurisdiction_refusal/policy.json");

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

async fn get_json(uri: &str) -> (StatusCode, Value) {
    let app = postcad_service::app();
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
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    (status, json)
}

fn route_body(case: &str, registry: &str, config: &str) -> Value {
    json!({
        "case":             serde_json::from_str::<Value>(case).unwrap(),
        "registry_snapshot": serde_json::from_str::<Value>(registry).unwrap(),
        "routing_config":   serde_json::from_str::<Value>(config).unwrap(),
    })
}

// ── /health ───────────────────────────────────────────────────────────────────

/// GET /health must return HTTP 200 with exactly {"status":"ok"}.
#[tokio::test]
async fn health_returns_ok() {
    let (status, body) = get_json("/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({"status": "ok"}));
}

// ── /version ─────────────────────────────────────────────────────────────────

/// GET /version must return HTTP 200 with the required version fields.
/// Values must match the compile-time constants already defined in the codebase.
#[tokio::test]
async fn version_returns_known_fields() {
    let (status, body) = get_json("/version").await;
    assert_eq!(status, StatusCode::OK);

    assert_eq!(
        body["service"], "postcad-service",
        "service name must be present"
    );
    assert_eq!(
        body["protocol_version"], "postcad-v1",
        "protocol_version must match PROTOCOL_VERSION"
    );
    assert_eq!(
        body["routing_kernel_version"], "postcad-routing-v1",
        "routing_kernel_version must match ROUTING_KERNEL_VERSION"
    );
}

/// GET /version must be deterministic — two calls return identical JSON.
#[tokio::test]
async fn version_is_deterministic() {
    let (_, v1) = get_json("/version").await;
    let (_, v2) = get_json("/version").await;
    assert_eq!(v1, v2);
}

// ── /route ────────────────────────────────────────────────────────────────────

/// POST /route with v01 inputs must return HTTP 200, outcome=routed,
/// selected_candidate_id=mfr-de-001. Proves the pilot endpoint delegates to
/// the registry-backed routing path.
#[tokio::test]
async fn pilot_route_routed_outcome() {
    let body = route_body(V01_CASE, V01_REGISTRY, V01_CONFIG);
    let (status, resp) = post_json("/route", body).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        resp["receipt"]["outcome"], "routed",
        "expected routed; got: {}",
        resp["receipt"]["outcome"]
    );
    assert_eq!(resp["receipt"]["selected_candidate_id"], "mfr-de-001");
    assert!(
        resp["receipt"]["receipt_hash"].is_string(),
        "receipt_hash must be present"
    );
    assert!(
        resp["derived_policy"].is_object(),
        "derived_policy must be returned"
    );
}

/// POST /route must be deterministic — two identical calls produce the same receipt_hash.
#[tokio::test]
async fn pilot_route_is_deterministic() {
    let body = route_body(V01_CASE, V01_REGISTRY, V01_CONFIG);
    let (_, r1) = post_json("/route", body.clone()).await;
    let (_, r2) = post_json("/route", body).await;
    assert_eq!(
        r1["receipt"]["receipt_hash"], r2["receipt"]["receipt_hash"],
        "routing must be deterministic for identical inputs"
    );
}

/// POST /route with v03 inputs (US-only registry, DE case) must return HTTP 200
/// with outcome=refused and refusal_code=no_jurisdiction_match.
#[tokio::test]
async fn pilot_route_refusal_outcome() {
    let body = route_body(V03_CASE, V03_REGISTRY, V03_CONFIG);
    let (status, resp) = post_json("/route", body).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["receipt"]["outcome"], "refused");
    assert_eq!(resp["receipt"]["refusal_code"], "no_jurisdiction_match");
    assert!(
        resp["receipt"]["selected_candidate_id"].is_null(),
        "selected_candidate_id must be null for refused outcome"
    );
}

/// POST /route with missing required fields must return HTTP 422 with parse_error.
#[tokio::test]
async fn pilot_route_missing_fields_returns_422() {
    let body = json!({
        "case": serde_json::from_str::<Value>(V01_CASE).unwrap(),
        // registry_snapshot and routing_config omitted
    });
    let (status, resp) = post_json("/route", body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(resp["error"]["code"], "parse_error");
}

// ── /verify ───────────────────────────────────────────────────────────────────

/// POST /verify with a receipt obtained from /route and the derived_policy from
/// the same response must return HTTP 200 {"result":"VERIFIED"}.
#[tokio::test]
async fn pilot_verify_accepts_routed_receipt() {
    let route_body = route_body(V01_CASE, V01_REGISTRY, V01_CONFIG);
    let (route_status, route_resp) = post_json("/route", route_body).await;
    assert_eq!(route_status, StatusCode::OK);

    let verify_body = json!({
        "receipt": route_resp["receipt"],
        "case":    serde_json::from_str::<Value>(V01_CASE).unwrap(),
        "policy":  route_resp["derived_policy"],
    });
    let (status, resp) = post_json("/verify", verify_body).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        resp["result"],
        "VERIFIED",
        "round-trip verify must succeed; error: {:?}",
        resp.get("error")
    );
}

/// POST /verify with a drifted registry snapshot must fail with
/// registry_snapshot_hash_mismatch — same tamper-detection as the existing
/// verify path.
#[tokio::test]
async fn pilot_verify_drift_detection_fails() {
    let route_body = route_body(V01_CASE, V01_REGISTRY, V01_CONFIG);
    let (_, route_resp) = post_json("/route", route_body).await;

    let receipt = route_resp["receipt"].clone();
    let mut drifted_policy = route_resp["derived_policy"].clone();
    if let Some(snapshots) = drifted_policy["snapshots"].as_array_mut() {
        if let Some(first) = snapshots.first_mut() {
            first["evidence_references"] = json!(["DRIFTED-EVIDENCE-REF"]);
        }
    }

    let verify_body = json!({
        "receipt": receipt,
        "case":    serde_json::from_str::<Value>(V01_CASE).unwrap(),
        "policy":  drifted_policy,
    });
    let (status, resp) = post_json("/verify", verify_body).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(resp["result"], "FAILED");
    assert_eq!(resp["error"]["code"], "registry_snapshot_hash_mismatch");
}

/// POST /verify with missing required fields must return HTTP 422 with receipt_parse_failed.
#[tokio::test]
async fn pilot_verify_missing_fields_returns_422() {
    let body = json!({
        "receipt": {"outcome": "routed"},
        // case and policy omitted
    });
    let (status, resp) = post_json("/verify", body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(resp["error"]["code"], "receipt_parse_failed");
}
