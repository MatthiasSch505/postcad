//! Pilot-grade registry-backed service integration tests.
//!
//! Proves the full protocol v1 flow through the HTTP service layer using
//! the registry-backed path exclusively:
//!
//!   POST /route-case-from-registry  →  receipt + derived_policy
//!   POST /verify-receipt            →  VERIFIED or error code
//!
//! Inputs are drawn from the frozen protocol conformance vectors in
//! `tests/protocol_vectors/` so the service output is anchored to the
//! same kernel artifacts verified by the vector tests.
//!
//! All tests run in-process via `tower::ServiceExt::oneshot`; no port
//! binding.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::util::ServiceExt;

// ── Frozen protocol-vector inputs (v01 — basic routing) ───────────────────────

/// Single eligible domestic manufacturer. Expected outcome: routed, mfr-de-001.
const V01_CASE: &str =
    include_str!("../../../tests/protocol_vectors/v01_basic_routing/case.json");
const V01_REGISTRY: &str =
    include_str!("../../../tests/protocol_vectors/v01_basic_routing/registry_snapshot.json");
const V01_CONFIG: &str =
    include_str!("../../../tests/protocol_vectors/v01_basic_routing/policy.json");

// ── Frozen protocol-vector inputs (v03 — jurisdiction refusal) ────────────────

/// US-only manufacturer registry; DE case must be refused.
const V03_CASE: &str =
    include_str!("../../../tests/protocol_vectors/v03_jurisdiction_refusal/case.json");
const V03_REGISTRY: &str =
    include_str!("../../../tests/protocol_vectors/v03_jurisdiction_refusal/registry_snapshot.json");
const V03_CONFIG: &str =
    include_str!("../../../tests/protocol_vectors/v03_jurisdiction_refusal/policy.json");

// ── Test helper ───────────────────────────────────────────────────────────────

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

fn registry_body(case: &str, registry: &str, config: &str) -> Value {
    json!({
        "case":     serde_json::from_str::<Value>(case).unwrap(),
        "registry": serde_json::from_str::<Value>(registry).unwrap(),
        "config":   serde_json::from_str::<Value>(config).unwrap(),
    })
}

// ── Pilot tests ───────────────────────────────────────────────────────────────

/// Successful routed flow:
/// POST /route-case-from-registry with v01 inputs must return HTTP 200,
/// outcome == "routed", and selected_candidate_id == "mfr-de-001".
/// Candidate derivation is driven entirely by the registry snapshot; no
/// hand-crafted candidate list is supplied.
#[tokio::test]
async fn pilot_registry_routed_flow() {
    let body = registry_body(V01_CASE, V01_REGISTRY, V01_CONFIG);
    let (status, resp) = post_json("/route-case-from-registry", body).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["receipt"]["outcome"], "routed",
        "expected routed outcome; got: {}", resp["receipt"]["outcome"]);
    assert_eq!(resp["receipt"]["selected_candidate_id"], "mfr-de-001",
        "expected mfr-de-001 to be selected");
    assert!(resp["receipt"]["receipt_hash"].is_string(),
        "receipt_hash must be present");
    assert!(resp["derived_policy"].is_object(),
        "derived_policy must be returned alongside the receipt");
}

/// Successful verify flow (round-trip coherence):
/// The receipt returned by POST /route-case-from-registry must be accepted
/// by POST /verify-receipt when the derived_policy from the same response is
/// used as the policy bundle. No external policy bundle is needed.
#[tokio::test]
async fn pilot_registry_verify_round_trip() {
    // Step 1: route via registry.
    let body = registry_body(V01_CASE, V01_REGISTRY, V01_CONFIG);
    let (route_status, route_resp) = post_json("/route-case-from-registry", body).await;
    assert_eq!(route_status, StatusCode::OK);

    let receipt = route_resp["receipt"].clone();
    let derived_policy = route_resp["derived_policy"].clone();

    // Step 2: verify using the receipt + derived policy from the same call.
    let verify_body = json!({
        "receipt": receipt,
        "case":    serde_json::from_str::<Value>(V01_CASE).unwrap(),
        "policy":  derived_policy,
    });
    let (verify_status, verify_resp) = post_json("/verify-receipt", verify_body).await;

    assert_eq!(verify_status, StatusCode::OK);
    assert_eq!(verify_resp["result"], "VERIFIED",
        "round-trip verify must succeed; error: {:?}", verify_resp.get("error"));
}

/// Refusal flow:
/// POST /route-case-from-registry with v03 inputs (US-only registry, DE case)
/// must return HTTP 200 with outcome == "refused" and refusal_code ==
/// "no_jurisdiction_match". Refusal is a valid domain outcome, not an
/// HTTP error.
#[tokio::test]
async fn pilot_registry_refusal_flow() {
    let body = registry_body(V03_CASE, V03_REGISTRY, V03_CONFIG);
    let (status, resp) = post_json("/route-case-from-registry", body).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["receipt"]["outcome"], "refused",
        "expected refused outcome");
    assert_eq!(resp["receipt"]["refusal_code"], "no_jurisdiction_match",
        "expected no_jurisdiction_match refusal code");
    assert!(resp["receipt"]["selected_candidate_id"].is_null(),
        "selected_candidate_id must be null for refused outcome");
}

/// Drift failure flow:
/// Take the receipt from a successful routing call, then verify it against
/// a drifted derived_policy (snapshot evidence reference changed). The
/// registry_snapshot_hash committed in the receipt no longer matches the
/// snapshot in the drifted policy, so verification must fail with
/// registry_snapshot_hash_mismatch.
#[tokio::test]
async fn pilot_registry_drift_detection_fails_verify() {
    // Step 1: route and capture receipt + derived policy.
    let body = registry_body(V01_CASE, V01_REGISTRY, V01_CONFIG);
    let (route_status, route_resp) = post_json("/route-case-from-registry", body).await;
    assert_eq!(route_status, StatusCode::OK);

    let receipt = route_resp["receipt"].clone();
    let mut drifted_policy = route_resp["derived_policy"].clone();

    // Step 2: drift the snapshot evidence in the derived policy.
    // The receipt's registry_snapshot_hash now no longer matches.
    if let Some(snapshots) = drifted_policy["snapshots"].as_array_mut() {
        if let Some(first) = snapshots.first_mut() {
            first["evidence_references"] =
                json!(["DRIFTED-EVIDENCE-REF"]);
        }
    }

    // Step 3: verify the original receipt against the drifted policy.
    let verify_body = json!({
        "receipt": receipt,
        "case":    serde_json::from_str::<Value>(V01_CASE).unwrap(),
        "policy":  drifted_policy,
    });
    let (verify_status, verify_resp) = post_json("/verify-receipt", verify_body).await;

    assert_eq!(verify_status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(verify_resp["result"], "FAILED");
    assert_eq!(verify_resp["error"]["code"], "registry_snapshot_hash_mismatch",
        "drift must be detected as registry_snapshot_hash_mismatch");
}
