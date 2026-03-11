//! Tests for POST /cases/:case_id/route — routing stored cases.
//!
//! Each test gets isolated temporary directories for both the case store and
//! the receipt store so tests are fully deterministic and do not share state.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::util::ServiceExt;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_app(tmp: &tempfile::TempDir) -> axum::Router {
    let case_store = Arc::new(postcad_service::CaseStore::new(tmp.path().join("cases")));
    let receipt_store = Arc::new(postcad_service::ReceiptStore::new(
        tmp.path().join("receipts"),
    ));
    postcad_service::app_with_stores(case_store, receipt_store)
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

// ── Fixtures ──────────────────────────────────────────────────────────────────

fn pilot_case() -> Value {
    serde_json::from_str(include_str!("../../../examples/pilot/case.json")).unwrap()
}

fn pilot_registry() -> Value {
    serde_json::from_str(include_str!(
        "../../../examples/pilot/registry_snapshot.json"
    ))
    .unwrap()
}

fn pilot_config() -> Value {
    serde_json::from_str(include_str!("../../../examples/pilot/config.json")).unwrap()
}

/// A registry with no manufacturers that serve jurisdiction DE.
fn empty_registry() -> Value {
    json!([])
}

// ── Store helper ──────────────────────────────────────────────────────────────

/// POST /cases and assert 201, returning the case_id.
async fn store_case(app: axum::Router, case: Value) -> String {
    let case_id = case["case_id"].as_str().unwrap().to_string();
    let (status, _) = post_json(app, "/cases", case).await;
    assert_eq!(status, StatusCode::CREATED);
    case_id
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// POST /cases/:id/route with a stored case and eligible registry returns 200
/// with case_id, receipt_hash, and selected_candidate_id.
#[tokio::test]
async fn route_stored_case_generates_receipt() {
    let tmp = tempfile::TempDir::new().unwrap();

    let case = pilot_case();
    let case_id = store_case(make_app(&tmp), case).await;

    let (status, body) = post_json(
        make_app(&tmp),
        &format!("/cases/{case_id}/route"),
        json!({ "registry": pilot_registry(), "config": pilot_config() }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["case_id"], case_id);
    assert!(
        body["receipt_hash"].as_str().is_some_and(|h| h.len() == 64),
        "receipt_hash must be a 64-char hex digest"
    );
    assert!(
        body["selected_candidate_id"].as_str().is_some(),
        "selected_candidate_id must be present for a routed outcome"
    );
}

/// After a successful route call the receipt file must exist on disk.
#[tokio::test]
async fn route_persists_receipt_file() {
    let tmp = tempfile::TempDir::new().unwrap();
    let receipt_dir = tmp.path().join("receipts");

    let case = pilot_case();
    let case_id = store_case(make_app(&tmp), case).await;

    let (status, body) = post_json(
        make_app(&tmp),
        &format!("/cases/{case_id}/route"),
        json!({ "registry": pilot_registry(), "config": pilot_config() }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    let receipt_hash = body["receipt_hash"].as_str().unwrap();
    let receipt_path = receipt_dir.join(format!("{receipt_hash}.json"));
    assert!(
        receipt_path.exists(),
        "receipt file must be written to data/receipts/{{hash}}.json"
    );

    // The file must contain valid JSON.
    let raw = std::fs::read_to_string(&receipt_path).unwrap();
    let parsed: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(parsed["receipt_hash"], receipt_hash);
}

/// The response contains exactly the three specified fields and no extras.
#[tokio::test]
async fn route_returns_expected_fields() {
    let tmp = tempfile::TempDir::new().unwrap();

    let case = pilot_case();
    let case_id = store_case(make_app(&tmp), case).await;

    let (status, body) = post_json(
        make_app(&tmp),
        &format!("/cases/{case_id}/route"),
        json!({ "registry": pilot_registry(), "config": pilot_config() }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    let obj = body.as_object().unwrap();
    assert_eq!(obj.len(), 3, "response must contain exactly 3 fields");
    assert!(obj.contains_key("case_id"));
    assert!(obj.contains_key("receipt_hash"));
    assert!(obj.contains_key("selected_candidate_id"));
}

/// Routing a case_id that was never stored returns 404 case_not_found.
#[tokio::test]
async fn route_rejects_missing_case() {
    let tmp = tempfile::TempDir::new().unwrap();

    let (status, body) = post_json(
        make_app(&tmp),
        "/cases/nonexistent-case-id/route",
        json!({ "registry": pilot_registry(), "config": pilot_config() }),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "case_not_found");
}

/// Routing with an empty registry returns 422 routing_refused.
#[tokio::test]
async fn route_with_empty_registry_returns_routing_refused() {
    let tmp = tempfile::TempDir::new().unwrap();

    let case = pilot_case();
    let case_id = store_case(make_app(&tmp), case).await;

    let (status, body) = post_json(
        make_app(&tmp),
        &format!("/cases/{case_id}/route"),
        json!({ "registry": empty_registry(), "config": pilot_config() }),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "routing_refused");
}

/// Routing is deterministic: the same stored case with the same registry
/// always produces the same receipt_hash and selected_candidate_id.
#[tokio::test]
async fn route_is_deterministic() {
    let tmp = tempfile::TempDir::new().unwrap();

    let case = pilot_case();
    let case_id = store_case(make_app(&tmp), case).await;

    let route_body = json!({ "registry": pilot_registry(), "config": pilot_config() });

    let (s1, b1) = post_json(
        make_app(&tmp),
        &format!("/cases/{case_id}/route"),
        route_body.clone(),
    )
    .await;
    let (s2, b2) = post_json(
        make_app(&tmp),
        &format!("/cases/{case_id}/route"),
        route_body,
    )
    .await;

    assert_eq!(s1, StatusCode::OK);
    assert_eq!(s2, StatusCode::OK);
    assert_eq!(b1["receipt_hash"], b2["receipt_hash"]);
    assert_eq!(b1["selected_candidate_id"], b2["selected_candidate_id"]);
}
