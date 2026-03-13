//! Operator UI and receipts REST endpoint tests.
//!
//! Verifies that the embedded UI is served correctly and that the receipts
//! endpoints behave as documented. All tests run in-process via
//! `tower::ServiceExt::oneshot`; no port binding.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tower::util::ServiceExt;

// Frozen v01 receipt for seeding tests.
const V01_RECEIPT: &str =
    include_str!("../../../tests/protocol_vectors/v01_basic_routing/expected_receipt.json");
const V01_RECEIPT_HASH: &str = "cbc0e380572bd572229a9d96e7a452e7213b059717aafcebf1ba888797b4b8c0";

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

fn seed_receipt(tmp: &tempfile::TempDir) {
    let dir = tmp.path().join("receipts");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(format!("{V01_RECEIPT_HASH}.json")), V01_RECEIPT).unwrap();
}

async fn get_raw(app: axum::Router, uri: &str) -> (StatusCode, Vec<u8>) {
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec();
    (status, bytes)
}

async fn get_json(app: axum::Router, uri: &str) -> (StatusCode, Value) {
    let (status, bytes) = get_raw(app, uri).await;
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    (status, json)
}

// ── Operator UI tests ─────────────────────────────────────────────────────────

/// GET / must return HTTP 200 with Content-Type text/html.
#[tokio::test]
async fn operator_ui_index_serves_html() {
    let tmp = tempfile::TempDir::new().unwrap();
    let app = make_app(&tmp);
    let req = Request::builder()
        .method("GET")
        .uri("/")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        ct.contains("text/html"),
        "content-type must be text/html, got: {ct}"
    );
}

/// The index page body must contain the five section identifiers expected by
/// the operator workflow (section IDs are stable across UI revisions).
#[tokio::test]
async fn operator_ui_contains_section_anchors() {
    let tmp = tempfile::TempDir::new().unwrap();
    let app = make_app(&tmp);
    let (status, bytes) = get_raw(app, "/").await;
    assert_eq!(status, StatusCode::OK);
    let html = std::str::from_utf8(&bytes).unwrap();
    for anchor in [
        "section-intake",
        "section-cases",
        "section-receipts",
        "section-history",
        "section-status",
    ] {
        assert!(html.contains(anchor), "HTML must contain id={anchor}");
    }
}

/// The index page must reference all required endpoint paths so the JS can
/// actually call them.
#[tokio::test]
async fn operator_ui_references_all_endpoints() {
    let tmp = tempfile::TempDir::new().unwrap();
    let app = make_app(&tmp);
    let (_, bytes) = get_raw(app, "/").await;
    let html = std::str::from_utf8(&bytes).unwrap();
    let required = [
        "/cases",
        "/cases/",
        "/receipts",
        "/dispatch/",
        "/routes",
        "/health",
        "/version",
    ];
    for path in required {
        assert!(
            html.contains(path),
            "HTML must reference endpoint path '{path}'"
        );
    }
}

/// The polished UI must use operator-friendly button labels (not raw endpoint
/// paths) so the workflow is clear without curl knowledge.
#[tokio::test]
async fn operator_ui_has_operator_friendly_labels() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, bytes) = get_raw(make_app(&tmp), "/").await;
    let html = std::str::from_utf8(&bytes).unwrap();
    for label in [
        "Store Case",
        "Route This Case",
        "View JSON",
        "Dispatch",
        "Verify Integrity",
        "Refresh",
    ] {
        assert!(
            html.contains(label),
            "HTML must contain operator label '{label}'"
        );
    }
}

/// The polished UI must expose stable viewer element IDs used by all result
/// display paths (fixes the pre-class bug and verifies the stable #case-detail
/// element is not recreated inside dynamic table HTML).
#[tokio::test]
async fn operator_ui_has_stable_viewer_elements() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, bytes) = get_raw(make_app(&tmp), "/").await;
    let html = std::str::from_utf8(&bytes).unwrap();
    for id in [
        "case-detail",
        "receipt-detail",
        "intake-result",
        "route-modal-result",
        "status-detail",
    ] {
        assert!(
            html.contains(id),
            "HTML must contain stable element id='{id}'"
        );
    }
}

/// The UI must include a global Refresh All button for fast full-page reload.
#[tokio::test]
async fn operator_ui_has_refresh_all_button() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, bytes) = get_raw(make_app(&tmp), "/").await;
    let html = std::str::from_utf8(&bytes).unwrap();
    assert!(
        html.contains("refreshAll"),
        "HTML must contain refreshAll function"
    );
    assert!(
        html.contains("Refresh all"),
        "HTML must contain 'Refresh all' button text"
    );
}

// ── GET /receipts tests ───────────────────────────────────────────────────────

/// GET /receipts with an empty store must return HTTP 200 and an empty list.
#[tokio::test]
async fn list_receipts_empty_store() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, body) = get_json(make_app(&tmp), "/receipts").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["receipts"], serde_json::json!([]));
}

/// GET /receipts with a seeded receipt must include the receipt hash.
#[tokio::test]
async fn list_receipts_returns_stored_hashes() {
    let tmp = tempfile::TempDir::new().unwrap();
    seed_receipt(&tmp);
    let (status, body) = get_json(make_app(&tmp), "/receipts").await;
    assert_eq!(status, StatusCode::OK);
    let hashes: Vec<&str> = body["receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(
        hashes.contains(&V01_RECEIPT_HASH),
        "receipts list must contain v01 hash"
    );
}

// ── GET /receipts/:hash tests ─────────────────────────────────────────────────

/// GET /receipts/:hash for a seeded receipt must return the receipt JSON.
#[tokio::test]
async fn get_receipt_returns_stored_json() {
    let tmp = tempfile::TempDir::new().unwrap();
    seed_receipt(&tmp);
    let (status, body) = get_json(make_app(&tmp), &format!("/receipts/{V01_RECEIPT_HASH}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["receipt_hash"].as_str().unwrap(), V01_RECEIPT_HASH);
}

/// GET /receipts/:hash for an unknown hash must return HTTP 404.
#[tokio::test]
async fn get_receipt_not_found() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, body) = get_json(make_app(&tmp), "/receipts/deadbeef").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(
        body["error"]["code"],
        serde_json::json!("receipt_not_found")
    );
}
