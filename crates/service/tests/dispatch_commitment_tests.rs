//! Dispatch Commitment Layer contract tests.
//!
//! All tests run in-process via `tower::ServiceExt::oneshot`; no port binding.
//! Each test injects a fresh temporary directory for commitment storage so
//! tests are fully isolated.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::util::ServiceExt;

// ── Canonical fixtures ────────────────────────────────────────────────────────

const CASE_JSON: &str = include_str!("../../../fixtures/case.json");
const POLICY_JSON: &str = include_str!("../../../fixtures/policy.json");
const EXPECTED_ROUTED_JSON: &str = include_str!("../../../fixtures/expected_routed.json");

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

async fn post_json_app(
    app: axum::Router,
    uri: &str,
    body: Value,
) -> (StatusCode, Value) {
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

async fn get_json_app(app: axum::Router, uri: &str) -> (StatusCode, Value) {
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

fn valid_create_body() -> Value {
    json!({
        "receipt": serde_json::from_str::<Value>(EXPECTED_ROUTED_JSON).unwrap(),
        "case":    serde_json::from_str::<Value>(CASE_JSON).unwrap(),
        "policy":  serde_json::from_str::<Value>(POLICY_JSON).unwrap(),
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// POST /dispatch/create with a valid, verified receipt must succeed.
/// Response must include dispatch_id, receipt_hash, status=draft,
/// verification_passed=true.
#[tokio::test]
async fn create_dispatch_from_verified_receipt_succeeds() {
    let tmp = tempfile::TempDir::new().unwrap();
    let app = make_app(&tmp);

    let (status, body) = post_json_app(app, "/dispatch/create", valid_create_body()).await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["status"], "draft");
    assert_eq!(body["verification_passed"], true);
    assert!(body["dispatch_id"].is_string(), "dispatch_id must be present");

    let expected_hash =
        serde_json::from_str::<Value>(EXPECTED_ROUTED_JSON).unwrap()["receipt_hash"]
            .as_str()
            .unwrap()
            .to_string();
    assert_eq!(body["receipt_hash"], expected_hash);
}

/// POST /dispatch/create with a tampered receipt must be rejected.
/// The tampered receipt fails cryptographic verification before dispatch is created.
#[tokio::test]
async fn create_dispatch_from_tampered_receipt_is_rejected() {
    let tmp = tempfile::TempDir::new().unwrap();
    let app = make_app(&tmp);

    let mut receipt: Value = serde_json::from_str(EXPECTED_ROUTED_JSON).unwrap();
    // Tamper: flip the outcome field (routing_decision_hash will mismatch)
    receipt["outcome"] = json!("refused");

    let body = json!({
        "receipt": receipt,
        "case":   serde_json::from_str::<Value>(CASE_JSON).unwrap(),
        "policy": serde_json::from_str::<Value>(POLICY_JSON).unwrap(),
    });

    let (status, resp) = post_json_app(app, "/dispatch/create", body).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {resp}");
    // Any verification failure code is acceptable; the request must not succeed.
    assert!(
        resp["error"]["code"].is_string(),
        "error.code must be present"
    );
}

/// The dispatch record must bind the exact receipt_hash from the receipt.
#[tokio::test]
async fn dispatch_binds_exact_receipt_hash() {
    let tmp = tempfile::TempDir::new().unwrap();
    let app = make_app(&tmp);

    let (_, body) = post_json_app(app, "/dispatch/create", valid_create_body()).await;

    let receipt_hash_in_receipt =
        serde_json::from_str::<Value>(EXPECTED_ROUTED_JSON).unwrap()["receipt_hash"]
            .as_str()
            .unwrap()
            .to_string();

    assert_eq!(
        body["receipt_hash"].as_str().unwrap(),
        receipt_hash_in_receipt,
        "dispatch must bind the exact receipt_hash"
    );
}

/// A second POST /dispatch/create for the same receipt must return 409.
/// One dispatch per receipt — duplicate creation is rejected.
#[tokio::test]
async fn create_dispatch_duplicate_receipt_returns_409() {
    let tmp = tempfile::TempDir::new().unwrap();

    // First call succeeds.
    let (s1, _) = post_json_app(make_app(&tmp), "/dispatch/create", valid_create_body()).await;
    assert_eq!(s1, StatusCode::OK);

    // Second call with the same receipt must be rejected.
    let (s2, body2) = post_json_app(make_app(&tmp), "/dispatch/create", valid_create_body()).await;
    assert_eq!(s2, StatusCode::CONFLICT, "body: {body2}");
    assert_eq!(body2["error"]["code"], "receipt_already_dispatched");
}

/// POST /dispatch/{dispatch_id}/approve transitions draft → approved.
/// Response must contain the full record with status=approved and approved_by set.
#[tokio::test]
async fn approve_dispatch_transitions_to_approved() {
    let tmp = tempfile::TempDir::new().unwrap();

    let (_, create_resp) =
        post_json_app(make_app(&tmp), "/dispatch/create", valid_create_body()).await;
    let dispatch_id = create_resp["dispatch_id"].as_str().unwrap().to_string();

    let approve_body = json!({"approved_by": "reviewer-1"});
    let (status, body) = post_json_app(
        make_app(&tmp),
        &format!("/dispatch/{dispatch_id}/approve"),
        approve_body,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body: {body}");
    assert_eq!(body["status"], "approved");
    assert_eq!(body["approved_by"], "reviewer-1");
    assert!(
        body["approved_at"].is_string(),
        "approved_at must be set"
    );
}

/// Approved dispatch is immutable: a second approve call must return 409.
#[tokio::test]
async fn approved_dispatch_is_immutable() {
    let tmp = tempfile::TempDir::new().unwrap();

    let (_, create_resp) =
        post_json_app(make_app(&tmp), "/dispatch/create", valid_create_body()).await;
    let dispatch_id = create_resp["dispatch_id"].as_str().unwrap().to_string();

    // First approval succeeds.
    let (s1, _) = post_json_app(
        make_app(&tmp),
        &format!("/dispatch/{dispatch_id}/approve"),
        json!({"approved_by": "reviewer-1"}),
    )
    .await;
    assert_eq!(s1, StatusCode::OK);

    // Second approval must be rejected.
    let (s2, body2) = post_json_app(
        make_app(&tmp),
        &format!("/dispatch/{dispatch_id}/approve"),
        json!({"approved_by": "reviewer-2"}),
    )
    .await;
    assert_eq!(s2, StatusCode::CONFLICT, "body: {body2}");
    assert_eq!(body2["error"]["code"], "dispatch_not_draft");
}

/// GET /dispatch/{dispatch_id}/export on a draft dispatch (not yet approved)
/// must return 422.
#[tokio::test]
async fn export_draft_dispatch_returns_422() {
    let tmp = tempfile::TempDir::new().unwrap();

    let (_, create_resp) =
        post_json_app(make_app(&tmp), "/dispatch/create", valid_create_body()).await;
    let dispatch_id = create_resp["dispatch_id"].as_str().unwrap().to_string();

    let (status, body) =
        get_json_app(make_app(&tmp), &format!("/dispatch/{dispatch_id}/export")).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
    assert_eq!(body["error"]["code"], "dispatch_not_approved");
}

/// GET /dispatch/{dispatch_id}/export on an approved dispatch must return
/// a deterministic payload containing all required fields.
#[tokio::test]
async fn export_approved_dispatch_returns_deterministic_payload() {
    let tmp = tempfile::TempDir::new().unwrap();

    // Create and approve.
    let (_, create_resp) =
        post_json_app(make_app(&tmp), "/dispatch/create", valid_create_body()).await;
    let dispatch_id = create_resp["dispatch_id"].as_str().unwrap().to_string();

    post_json_app(
        make_app(&tmp),
        &format!("/dispatch/{dispatch_id}/approve"),
        json!({"approved_by": "reviewer-1"}),
    )
    .await;

    // First export.
    let (s1, p1) =
        get_json_app(make_app(&tmp), &format!("/dispatch/{dispatch_id}/export")).await;
    assert_eq!(s1, StatusCode::OK, "body: {p1}");
    assert_eq!(p1["status"], "exported");
    assert_eq!(p1["verification_passed"], true);
    assert_eq!(p1["dispatch_id"], dispatch_id);

    // Second export — must return the same payload (deterministic + idempotent).
    let (s2, p2) =
        get_json_app(make_app(&tmp), &format!("/dispatch/{dispatch_id}/export")).await;
    assert_eq!(s2, StatusCode::OK);
    assert_eq!(p1, p2, "export payload must be deterministic");
}

/// Missing required fields in POST /dispatch/create must return 422 parse_error.
#[tokio::test]
async fn create_dispatch_missing_fields_returns_422() {
    let tmp = tempfile::TempDir::new().unwrap();
    let app = make_app(&tmp);

    let body = json!({
        "receipt": serde_json::from_str::<Value>(EXPECTED_ROUTED_JSON).unwrap(),
        // "case" and "policy" omitted
    });

    let (status, resp) = post_json_app(app, "/dispatch/create", body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {resp}");
    assert_eq!(resp["error"]["code"], "parse_error");
}
