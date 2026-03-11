//! Dispatch endpoint tests.
//!
//! POST /dispatch/:receipt_hash
//!
//! All tests run in-process via `tower::ServiceExt::oneshot`; no port binding.
//! Each test uses a temporary directory to isolate storage.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::util::ServiceExt;

// Frozen v01 protocol vector receipt — provides a real receipt_hash and fields.
const V01_RECEIPT: &str =
    include_str!("../../../tests/protocol_vectors/v01_basic_routing/expected_receipt.json");

const V01_RECEIPT_HASH: &str = "cbc0e380572bd572229a9d96e7a452e7213b059717aafcebf1ba888797b4b8c0";

// ── Test helper ───────────────────────────────────────────────────────────────

/// Build an app with isolated temp stores and pre-seed the receipt store with
/// the v01 receipt if `seed_receipt` is true.
fn make_app(tmp: &tempfile::TempDir, seed_receipt: bool) -> axum::Router {
    let receipts_dir = tmp.path().join("receipts");
    let dispatch_dir = tmp.path().join("dispatch");

    if seed_receipt {
        std::fs::create_dir_all(&receipts_dir).unwrap();
        std::fs::write(
            receipts_dir.join(format!("{V01_RECEIPT_HASH}.json")),
            V01_RECEIPT,
        )
        .unwrap();
    }

    postcad_service::app_with_all_stores(
        Arc::new(postcad_service::CaseStore::new(tmp.path().join("cases"))),
        Arc::new(postcad_service::ReceiptStore::new(receipts_dir)),
        Arc::new(postcad_service::DispatchStore::new(dispatch_dir)),
        Arc::new(postcad_service::PolicyStore::new(
            tmp.path().join("policies"),
        )),
        Arc::new(postcad_service::VerificationStore::new(
            tmp.path().join("verification"),
        )),
    )
}

async fn post_dispatch(app: axum::Router, receipt_hash: &str) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("POST")
        .uri(format!("/dispatch/{receipt_hash}"))
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

/// POST /dispatch/:receipt_hash with a stored receipt must return HTTP 200
/// and `{"receipt_hash": "...", "dispatched": true}`.
#[tokio::test]
async fn dispatch_creates_dispatch_record() {
    let tmp = tempfile::TempDir::new().unwrap();
    let app = make_app(&tmp, true);

    let (status, body) = post_dispatch(app, V01_RECEIPT_HASH).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["receipt_hash"], json!(V01_RECEIPT_HASH));
    assert_eq!(body["dispatched"], json!(true));

    // Dispatch record must be persisted on disk.
    let record_path = tmp
        .path()
        .join("dispatch")
        .join(format!("{V01_RECEIPT_HASH}.json"));
    assert!(record_path.exists(), "dispatch record file must exist");

    let record: Value =
        serde_json::from_str(&std::fs::read_to_string(&record_path).unwrap()).unwrap();
    assert_eq!(record["receipt_hash"], json!(V01_RECEIPT_HASH));
    assert_eq!(
        record["case_id"],
        json!("00000001-0000-0000-0000-000000000001")
    );
    assert_eq!(record["manufacturer"], json!("mfr-de-001"));
    assert_eq!(record["status"], json!("dispatched"));
    assert!(record["timestamp"].is_string());
}

/// POST /dispatch/:receipt_hash when no receipt exists must return HTTP 404
/// with `code == "receipt_not_found"`.
#[tokio::test]
async fn dispatch_rejects_missing_receipt() {
    let tmp = tempfile::TempDir::new().unwrap();
    let app = make_app(&tmp, false); // receipt store is empty

    let (status, body) = post_dispatch(app, V01_RECEIPT_HASH).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], json!("receipt_not_found"));
}

/// A second POST /dispatch/:receipt_hash for the same receipt must return
/// HTTP 409 with `code == "dispatch_already_exists"`.
#[tokio::test]
async fn dispatch_prevents_duplicate_dispatch() {
    let tmp = tempfile::TempDir::new().unwrap();

    // First dispatch — must succeed.
    let app1 = make_app(&tmp, true);
    let (status1, _) = post_dispatch(app1, V01_RECEIPT_HASH).await;
    assert_eq!(status1, StatusCode::OK);

    // Second dispatch against the same temp dir — must be rejected.
    let app2 = make_app(&tmp, false); // receipt already seeded in step above
    let (status2, body2) = post_dispatch(app2, V01_RECEIPT_HASH).await;
    assert_eq!(status2, StatusCode::CONFLICT);
    assert_eq!(body2["error"]["code"], json!("dispatch_already_exists"));
}

/// The success response must contain exactly the two expected keys.
#[tokio::test]
async fn dispatch_returns_expected_fields() {
    let tmp = tempfile::TempDir::new().unwrap();
    let app = make_app(&tmp, true);

    let (status, body) = post_dispatch(app, V01_RECEIPT_HASH).await;

    assert_eq!(status, StatusCode::OK);

    let obj = body.as_object().expect("response must be a JSON object");
    let keys: Vec<&str> = obj.keys().map(String::as_str).collect();
    assert_eq!(keys.len(), 2, "response must have exactly 2 keys: {keys:?}");
    assert!(obj.contains_key("receipt_hash"));
    assert!(obj.contains_key("dispatched"));
}
