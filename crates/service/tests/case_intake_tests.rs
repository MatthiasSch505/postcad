//! Case intake endpoint tests — POST /cases, GET /cases, GET /cases/:case_id.
//!
//! Each test gets an isolated temporary directory via `tempfile::TempDir`
//! so tests are fully deterministic and do not share state.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::util::ServiceExt;

// ── Helpers ───────────────────────────────────────────────────────────────────

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

// ── Fixtures ──────────────────────────────────────────────────────────────────

fn pilot_case() -> Value {
    serde_json::from_str(include_str!("../../../examples/pilot/case.json")).unwrap()
}

fn pilot_case_b() -> Value {
    // A second distinct case with a different case_id.
    json!({
        "case_id": "a0000000-0000-0000-0000-000000000002",
        "jurisdiction": "DE",
        "routing_policy": "allow_domestic_and_cross_border",
        "patient_country": "germany",
        "manufacturer_country": "germany",
        "material": "zirconia",
        "procedure": "bridge",
        "file_type": "stl"
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// POST /cases with a valid pilot case stores it and returns {case_id, stored:true} (201).
#[tokio::test]
async fn post_case_stores_canonical_case() {
    let tmp = tempfile::TempDir::new().unwrap();
    let store = Arc::new(postcad_service::CaseStore::new(tmp.path()));
    let app = postcad_service::app_with_store(store);

    let case = pilot_case();
    let case_id = case["case_id"].as_str().unwrap().to_string();

    let (status, body) = post_json(app, "/cases", case).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["case_id"], case_id);
    assert_eq!(body["stored"], true);
}

/// Re-posting the identical case returns 200 with stored:true (idempotent).
#[tokio::test]
async fn post_case_identical_repost_is_idempotent() {
    let tmp = tempfile::TempDir::new().unwrap();
    let store = Arc::new(postcad_service::CaseStore::new(tmp.path()));

    let case = pilot_case();

    // First post: 201 Created.
    let (s1, _) = post_json(
        postcad_service::app_with_store(store.clone()),
        "/cases",
        case.clone(),
    )
    .await;
    assert_eq!(s1, StatusCode::CREATED);

    // Second identical post: 200 OK.
    let (s2, body) = post_json(postcad_service::app_with_store(store), "/cases", case).await;
    assert_eq!(s2, StatusCode::OK);
    assert_eq!(body["stored"], true);
}

/// Posting a different case body with an already-stored case_id returns 409.
#[tokio::test]
async fn post_case_conflict_returns_409() {
    let tmp = tempfile::TempDir::new().unwrap();
    let store = Arc::new(postcad_service::CaseStore::new(tmp.path()));

    let case_a = pilot_case();
    let case_id = case_a["case_id"].as_str().unwrap().to_string();

    // First post succeeds.
    let (s1, _) = post_json(
        postcad_service::app_with_store(store.clone()),
        "/cases",
        case_a,
    )
    .await;
    assert_eq!(s1, StatusCode::CREATED);

    // Different body, same case_id.
    let mut case_conflict = pilot_case_b();
    case_conflict["case_id"] = json!(case_id);
    let (s2, body) = post_json(
        postcad_service::app_with_store(store),
        "/cases",
        case_conflict,
    )
    .await;

    assert_eq!(s2, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "case_id_conflict");
}

/// POST /cases with a structurally invalid body (missing required fields) returns 422.
#[tokio::test]
async fn post_case_rejects_invalid_case() {
    let tmp = tempfile::TempDir::new().unwrap();
    let store = Arc::new(postcad_service::CaseStore::new(tmp.path()));
    let app = postcad_service::app_with_store(store);

    // CaseInput requires patient_country, manufacturer_country, material,
    // procedure, file_type. Omitting them causes deserialization to fail.
    let invalid = json!({
        "case_id": "b0000000-0000-0000-0000-000000000001"
        // all required fields omitted
    });

    let (status, body) = post_json(app, "/cases", invalid).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "parse_error");
}

/// POST /cases without case_id returns 422 parse_error.
#[tokio::test]
async fn post_case_without_case_id_returns_422() {
    let tmp = tempfile::TempDir::new().unwrap();
    let store = Arc::new(postcad_service::CaseStore::new(tmp.path()));
    let app = postcad_service::app_with_store(store);

    let no_id = json!({
        "patient_country": "germany",
        "manufacturer_country": "germany",
        "material": "zirconia",
        "procedure": "crown",
        "file_type": "stl"
    });

    let (status, body) = post_json(app, "/cases", no_id).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "parse_error");
}

/// GET /cases returns sorted list of stored case IDs.
#[tokio::test]
async fn get_cases_returns_sorted_ids() {
    let tmp = tempfile::TempDir::new().unwrap();
    let store = Arc::new(postcad_service::CaseStore::new(tmp.path()));

    // Store two cases — b before a to verify sorting.
    post_json(
        postcad_service::app_with_store(store.clone()),
        "/cases",
        pilot_case_b(),
    )
    .await;
    post_json(
        postcad_service::app_with_store(store.clone()),
        "/cases",
        pilot_case(),
    )
    .await;

    let (status, body) = get_json(postcad_service::app_with_store(store), "/cases").await;
    assert_eq!(status, StatusCode::OK);

    let ids = body["case_ids"].as_array().unwrap();
    assert_eq!(ids.len(), 2);

    // IDs must be in ascending order.
    let id0 = ids[0].as_str().unwrap();
    let id1 = ids[1].as_str().unwrap();
    assert!(
        id0 < id1,
        "case_ids must be sorted ascending: {id0} < {id1}"
    );
}

/// GET /cases on empty store returns an empty list.
#[tokio::test]
async fn get_cases_empty_store_returns_empty_list() {
    let tmp = tempfile::TempDir::new().unwrap();
    let store = Arc::new(postcad_service::CaseStore::new(tmp.path()));
    let app = postcad_service::app_with_store(store);

    let (status, body) = get_json(app, "/cases").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["case_ids"], json!([]));
}

/// GET /cases/:case_id returns the stored case JSON.
#[tokio::test]
async fn get_case_by_id_returns_stored_case() {
    let tmp = tempfile::TempDir::new().unwrap();
    let store = Arc::new(postcad_service::CaseStore::new(tmp.path()));

    let case = pilot_case();
    let case_id = case["case_id"].as_str().unwrap().to_string();

    post_json(
        postcad_service::app_with_store(store.clone()),
        "/cases",
        case.clone(),
    )
    .await;

    let (status, body) = get_json(
        postcad_service::app_with_store(store),
        &format!("/cases/{case_id}"),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    // The returned JSON must equal the stored case value-for-value.
    assert_eq!(body, case);
}

/// GET /cases/:case_id for a non-existent case returns 404.
#[tokio::test]
async fn get_case_by_id_not_found_returns_404() {
    let tmp = tempfile::TempDir::new().unwrap();
    let store = Arc::new(postcad_service::CaseStore::new(tmp.path()));
    let app = postcad_service::app_with_store(store);

    let (status, body) = get_json(app, "/cases/nonexistent-id").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "case_not_found");
}
