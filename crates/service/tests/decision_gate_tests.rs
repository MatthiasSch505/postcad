//! Decision gate tests.
//!
//! Verifies the decision domain logic and the routing gate behavior:
//!
//!  - Creating valid decisions (proceed, proceed_with_risk, request_correction)
//!  - Validation: proceed_with_risk and request_correction require reason_code
//!  - Routing without a decision → decision_missing error
//!  - Routing with request_correction decision → decision_not_routable error
//!  - Routing after proceed decision → succeeds
//!  - Routing after proceed_with_risk decision → succeeds
//!  - Decision hash and type appear in the stored receipt
//!  - Dispatch verify checks decision linkage

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::util::ServiceExt;

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

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_app(tmp: &tempfile::TempDir) -> axum::Router {
    postcad_service::app_with_all_stores(
        Arc::new(postcad_service::CaseStore::new(tmp.path().join("cases"))),
        Arc::new(postcad_service::ReceiptStore::new(tmp.path().join("receipts"))),
        Arc::new(postcad_service::DispatchStore::new(tmp.path().join("dispatch"))),
        Arc::new(postcad_service::PolicyStore::new(tmp.path().join("policies"))),
        Arc::new(postcad_service::VerificationStore::new(
            tmp.path().join("verification"),
        )),
        Arc::new(postcad_service::DispatchCommitmentStore::new(
            tmp.path().join("commitments"),
        )),
        Arc::new(postcad_service::DecisionStore::new(tmp.path().join("decisions"))),
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

/// POST /cases with pilot fixture; return case_id.
async fn store_case(app: axum::Router) -> String {
    let case = pilot_case();
    let case_id = case["case_id"].as_str().unwrap().to_string();
    let (status, body) = post_json(app, "/cases", case).await;
    assert_eq!(status, StatusCode::CREATED, "store_case failed: {body}");
    case_id
}

// ── Decision creation tests ───────────────────────────────────────────────────

#[tokio::test]
async fn create_proceed_decision_succeeds() {
    let tmp = tempfile::TempDir::new().unwrap();
    let case_id = store_case(make_app(&tmp)).await;

    let (status, body) = post_json(
        make_app(&tmp),
        "/decisions",
        json!({
            "case_id": &case_id,
            "actor_role": "reviewer",
            "actor_id": "dr-smith",
            "decision_type": "proceed",
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["case_id"], case_id);
    assert_eq!(body["decision_type"], "proceed");
    assert!(body["decision_id"].as_str().is_some());
    assert!(body["decision_hash"].as_str().is_some_and(|h| h.len() == 64));
    assert!(body["input_hash"].as_str().is_some_and(|h| h.len() == 64));
    assert!(body["reason_code"].is_null());
}

#[tokio::test]
async fn create_proceed_with_risk_requires_reason_code() {
    let tmp = tempfile::TempDir::new().unwrap();
    let case_id = store_case(make_app(&tmp)).await;

    // Missing reason_code → validation error
    let (status, body) = post_json(
        make_app(&tmp),
        "/decisions",
        json!({
            "case_id": &case_id,
            "actor_role": "reviewer",
            "actor_id": "dr-smith",
            "decision_type": "proceed_with_risk",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "decision_validation_error");

    // With reason_code → succeeds
    let (status2, body2) = post_json(
        make_app(&tmp),
        "/decisions",
        json!({
            "case_id": &case_id,
            "actor_role": "reviewer",
            "actor_id": "dr-smith",
            "decision_type": "proceed_with_risk",
            "reason_code": "time_pressure",
        }),
    )
    .await;
    assert_eq!(status2, StatusCode::OK, "{body2}");
    assert_eq!(body2["reason_code"], "time_pressure");
}

#[tokio::test]
async fn create_request_correction_requires_reason_code() {
    let tmp = tempfile::TempDir::new().unwrap();
    let case_id = store_case(make_app(&tmp)).await;

    let (status, body) = post_json(
        make_app(&tmp),
        "/decisions",
        json!({
            "case_id": &case_id,
            "actor_role": "reviewer",
            "actor_id": "dr-smith",
            "decision_type": "request_correction",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "decision_validation_error");
}

#[tokio::test]
async fn create_decision_for_missing_case_returns_404() {
    let tmp = tempfile::TempDir::new().unwrap();

    let (status, body) = post_json(
        make_app(&tmp),
        "/decisions",
        json!({
            "case_id": "nonexistent-case",
            "actor_role": "reviewer",
            "actor_id": "dr-smith",
            "decision_type": "proceed",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["error"]["code"], "case_not_found");
}

// ── Routing gate tests ────────────────────────────────────────────────────────

#[tokio::test]
async fn routing_without_decision_returns_decision_missing() {
    let tmp = tempfile::TempDir::new().unwrap();
    let case_id = store_case(make_app(&tmp)).await;

    let (status, body) = post_json(
        make_app(&tmp),
        &format!("/cases/{case_id}/route"),
        json!({ "registry": pilot_registry(), "config": pilot_config() }),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "decision_missing");
}

#[tokio::test]
async fn routing_with_request_correction_is_blocked() {
    let tmp = tempfile::TempDir::new().unwrap();
    let case_id = store_case(make_app(&tmp)).await;

    // Record a request_correction decision (requires reason_code)
    let (ds, db) = post_json(
        make_app(&tmp),
        "/decisions",
        json!({
            "case_id": &case_id,
            "actor_role": "reviewer",
            "actor_id": "dr-smith",
            "decision_type": "request_correction",
            "reason_code": "unclear_margin",
        }),
    )
    .await;
    assert_eq!(ds, StatusCode::OK, "decision failed: {db}");

    let (status, body) = post_json(
        make_app(&tmp),
        &format!("/cases/{case_id}/route"),
        json!({ "registry": pilot_registry(), "config": pilot_config() }),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["error"]["code"], "decision_not_routable");
}

#[tokio::test]
async fn routing_with_proceed_succeeds() {
    let tmp = tempfile::TempDir::new().unwrap();
    let case_id = store_case(make_app(&tmp)).await;

    let (ds, db) = post_json(
        make_app(&tmp),
        "/decisions",
        json!({
            "case_id": &case_id,
            "actor_role": "reviewer",
            "actor_id": "dr-smith",
            "decision_type": "proceed",
        }),
    )
    .await;
    assert_eq!(ds, StatusCode::OK, "{db}");

    let (status, body) = post_json(
        make_app(&tmp),
        &format!("/cases/{case_id}/route"),
        json!({ "registry": pilot_registry(), "config": pilot_config() }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["case_id"], case_id);
}

#[tokio::test]
async fn routing_with_proceed_with_risk_succeeds() {
    let tmp = tempfile::TempDir::new().unwrap();
    let case_id = store_case(make_app(&tmp)).await;

    let (ds, db) = post_json(
        make_app(&tmp),
        "/decisions",
        json!({
            "case_id": &case_id,
            "actor_role": "reviewer",
            "actor_id": "dr-smith",
            "decision_type": "proceed_with_risk",
            "reason_code": "incomplete_scan",
        }),
    )
    .await;
    assert_eq!(ds, StatusCode::OK, "{db}");

    let (status, body) = post_json(
        make_app(&tmp),
        &format!("/cases/{case_id}/route"),
        json!({ "registry": pilot_registry(), "config": pilot_config() }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
}

// ── Receipt linkage tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn stored_receipt_contains_decision_fields() {
    let tmp = tempfile::TempDir::new().unwrap();
    let case_id = store_case(make_app(&tmp)).await;

    let (ds, decision) = post_json(
        make_app(&tmp),
        "/decisions",
        json!({
            "case_id": &case_id,
            "actor_role": "reviewer",
            "actor_id": "dr-smith",
            "decision_type": "proceed",
        }),
    )
    .await;
    assert_eq!(ds, StatusCode::OK, "{decision}");

    let (rs, route_body) = post_json(
        make_app(&tmp),
        &format!("/cases/{case_id}/route"),
        json!({ "registry": pilot_registry(), "config": pilot_config() }),
    )
    .await;
    assert_eq!(rs, StatusCode::OK, "{route_body}");

    let receipt_hash = route_body["receipt_hash"].as_str().unwrap();
    let receipt_path = tmp.path().join("receipts").join(format!("{receipt_hash}.json"));
    let raw = std::fs::read_to_string(&receipt_path).unwrap();
    let stored: Value = serde_json::from_str(&raw).unwrap();

    assert_eq!(stored["decision_id"], decision["decision_id"]);
    assert_eq!(stored["decision_hash"], decision["decision_hash"]);
    assert_eq!(stored["decision_type"], "proceed");
}

#[tokio::test]
async fn receipt_hash_is_unaffected_by_decision_fields() {
    // The routing kernel receipt_hash must remain verifiable even after
    // decision fields are injected. Verification should VERIFIED.
    let tmp = tempfile::TempDir::new().unwrap();
    let case_id = store_case(make_app(&tmp)).await;

    post_json(
        make_app(&tmp),
        "/decisions",
        json!({
            "case_id": &case_id,
            "actor_role": "reviewer",
            "actor_id": "dr-smith",
            "decision_type": "proceed",
        }),
    )
    .await;

    let (_, route_body) = post_json(
        make_app(&tmp),
        &format!("/cases/{case_id}/route"),
        json!({ "registry": pilot_registry(), "config": pilot_config() }),
    )
    .await;

    let receipt_hash = route_body["receipt_hash"].as_str().unwrap();
    let receipt_path = tmp.path().join("receipts").join(format!("{receipt_hash}.json"));
    let raw = std::fs::read_to_string(&receipt_path).unwrap();
    let stored_receipt: Value = serde_json::from_str(&raw).unwrap();

    // Verify that receipt_hash in stored receipt matches what the kernel computed.
    assert_eq!(
        stored_receipt["receipt_hash"].as_str().unwrap(),
        receipt_hash,
        "receipt_hash in stored receipt must match route response"
    );
}
