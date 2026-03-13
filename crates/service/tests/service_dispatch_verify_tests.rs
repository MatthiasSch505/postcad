//! Dispatch verification endpoint tests.
//!
//! POST /dispatch/:receipt_hash/verify
//!
//! All tests run in-process via `tower::ServiceExt::oneshot`; no port binding.
//! Each test uses a temporary directory for all five storage layers.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use postcad_cli::route_case_from_registry_json;
use serde_json::{json, Value};
use tower::util::ServiceExt;

// ── Frozen v01 protocol-vector inputs ─────────────────────────────────────────

const V01_CASE: &str = include_str!("../../../tests/protocol_vectors/v01_basic_routing/case.json");
const V01_REGISTRY: &str =
    include_str!("../../../tests/protocol_vectors/v01_basic_routing/registry_snapshot.json");
const V01_CONFIG: &str =
    include_str!("../../../tests/protocol_vectors/v01_basic_routing/policy.json");

// ── Test helper ───────────────────────────────────────────────────────────────

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

/// Route the v01 fixture and seed receipt, policy, and dispatch stores.
/// Returns the `receipt_hash`.
fn seed_valid_fixture(tmp: &tempfile::TempDir) -> String {
    let result = route_case_from_registry_json(V01_CASE, V01_REGISTRY, V01_CONFIG)
        .expect("v01 routing must succeed");

    let receipt_hash = result.receipt.receipt_hash.clone();

    // Write receipt.
    let receipts_dir = tmp.path().join("receipts");
    std::fs::create_dir_all(&receipts_dir).unwrap();
    std::fs::write(
        receipts_dir.join(format!("{receipt_hash}.json")),
        serde_json::to_string_pretty(&result.receipt).unwrap(),
    )
    .unwrap();

    // Write derived policy.
    let policies_dir = tmp.path().join("policies");
    std::fs::create_dir_all(&policies_dir).unwrap();
    std::fs::write(
        policies_dir.join(format!("{receipt_hash}.json")),
        &result.derived_policy_json,
    )
    .unwrap();

    // Write dispatch record.
    let dispatch_dir = tmp.path().join("dispatch");
    std::fs::create_dir_all(&dispatch_dir).unwrap();
    let dispatch_record = json!({
        "receipt_hash": receipt_hash,
        "case_id": result.receipt.routing_input.case_id,
        "manufacturer": result.receipt.selected_candidate_id,
        "status": "dispatched",
        "timestamp": "2026-01-01T00:00:00+00:00",
    });
    std::fs::write(
        dispatch_dir.join(format!("{receipt_hash}.json")),
        serde_json::to_string_pretty(&dispatch_record).unwrap(),
    )
    .unwrap();

    receipt_hash
}

async fn post_verify(app: axum::Router, receipt_hash: &str) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("POST")
        .uri(format!("/dispatch/{receipt_hash}/verify"))
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

// ── Tests ─────────────────────────────────────────────────────────────────────

/// A dispatched receipt backed by a valid v01 fixture must verify as VERIFIED.
#[tokio::test]
async fn verify_dispatched_receipt_returns_verified_for_valid_fixture() {
    let tmp = tempfile::TempDir::new().unwrap();
    let receipt_hash = seed_valid_fixture(&tmp);
    let app = make_app(&tmp);

    let (status, body) = post_verify(app, &receipt_hash).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["receipt_hash"], json!(receipt_hash));
    assert_eq!(body["result"], json!("VERIFIED"));
}

/// The verification result must be written to the verification store.
#[tokio::test]
async fn verify_dispatched_receipt_persists_result() {
    let tmp = tempfile::TempDir::new().unwrap();
    let receipt_hash = seed_valid_fixture(&tmp);
    let app = make_app(&tmp);

    let (status, _) = post_verify(app, &receipt_hash).await;
    assert_eq!(status, StatusCode::OK);

    // Confirm the result file exists on disk.
    let result_path = tmp
        .path()
        .join("verification")
        .join(format!("{receipt_hash}.json"));
    assert!(result_path.exists(), "verification result file must exist");

    let stored: Value =
        serde_json::from_str(&std::fs::read_to_string(&result_path).unwrap()).unwrap();
    assert_eq!(stored["receipt_hash"], json!(receipt_hash));
    assert_eq!(stored["result"], json!("VERIFIED"));
    assert!(stored["timestamp"].is_string());
}

/// Without a dispatch record the endpoint must return HTTP 404 / dispatch_not_found.
#[tokio::test]
async fn verify_dispatched_receipt_rejects_missing_dispatch() {
    let tmp = tempfile::TempDir::new().unwrap();
    let receipt_hash = seed_valid_fixture(&tmp);

    // Remove the dispatch record so the gate fires.
    std::fs::remove_file(
        tmp.path()
            .join("dispatch")
            .join(format!("{receipt_hash}.json")),
    )
    .unwrap();

    let app = make_app(&tmp);
    let (status, body) = post_verify(app, &receipt_hash).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], json!("dispatch_not_found"));
}

/// A tampered receipt (content changed, receipt_hash field kept) must produce
/// result "INVALID" so the gate correctly detects the integrity breach.
#[tokio::test]
async fn verify_dispatched_receipt_returns_invalid_when_verification_fails() {
    let tmp = tempfile::TempDir::new().unwrap();
    let receipt_hash = seed_valid_fixture(&tmp);

    // Tamper: overwrite the stored receipt — change outcome while keeping the
    // original receipt_hash value. The canonicalization check will fail.
    let receipt_path = tmp
        .path()
        .join("receipts")
        .join(format!("{receipt_hash}.json"));
    let mut v: Value =
        serde_json::from_str(&std::fs::read_to_string(&receipt_path).unwrap()).unwrap();
    v["outcome"] = json!("tampered");
    std::fs::write(&receipt_path, serde_json::to_string_pretty(&v).unwrap()).unwrap();

    let app = make_app(&tmp);
    let (status, body) = post_verify(app, &receipt_hash).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["receipt_hash"], json!(receipt_hash));
    assert_eq!(body["result"], json!("INVALID"));
}
