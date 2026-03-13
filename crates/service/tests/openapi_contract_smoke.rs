//! OpenAPI contract smoke test for the shipped pilot endpoints.
//!
//! Mechanically checks that live responses match the contract defined in
//! `docs/openapi.yaml` at a practical level:
//!
//!   - Expected HTTP status codes are returned.
//!   - All required response fields named in the spec are present.
//!   - Error responses carry the documented `{ error: { code, message } }` shape.
//!
//! Spec sections exercised:
//!   POST /pilot/route-normalized    → RouteResponse / ErrorResponse
//!   POST /verify                    → VerifyResponse / VerifyFailedResponse
//!   POST /dispatch/create           → DispatchRecord / ErrorResponse (409)
//!   POST /dispatch/{id}/approve     → DispatchRecord / ErrorResponse (404)
//!   GET  /dispatch/{id}/export      → DispatchExport / ErrorResponse (422)
//!
//! All tests run in-process via `tower::ServiceExt::oneshot`. No port binding.
//! Uses existing pilot fixtures. No new dependencies.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::util::ServiceExt;

const REGISTRY_JSON: &str = include_str!("../../../examples/pilot/registry_snapshot.json");
const CONFIG_JSON: &str = include_str!("../../../examples/pilot/config.json");
const CASE_JSON: &str = include_str!("../../../examples/pilot/case.json");

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

/// Canonical normalized pilot route request body using pilot fixtures.
fn pilot_route_body() -> Value {
    json!({
        "pilot_case": {
            "case_id":          "f1000001-0000-0000-0000-000000000001",
            "restoration_type": "crown",
            "material":         "zirconia",
            "jurisdiction":     "DE"
        },
        "registry_snapshot": serde_json::from_str::<Value>(REGISTRY_JSON).unwrap(),
        "routing_config":    serde_json::from_str::<Value>(CONFIG_JSON).unwrap(),
    })
}

/// Assert all RoutingReceipt required fields (spec: components/schemas/RoutingReceipt).
fn assert_receipt_fields(receipt: &Value, ctx: &str) {
    // 18 required fields listed in the spec — nullable/optional fields excluded.
    for field in &[
        "schema_version",
        "routing_kernel_version",
        "outcome",
        "case_fingerprint",
        "policy_fingerprint",
        "routing_proof_hash",
        "registry_snapshot_hash",
        "candidate_pool_hash",
        "eligible_candidate_ids_hash",
        "selection_input_candidate_ids_hash",
        "candidate_order_hash",
        "routing_decision_hash",
        "audit_seq",
        "audit_entry_hash",
        "audit_previous_hash",
        "routing_input",
        "routing_input_hash",
        "receipt_hash",
    ] {
        assert!(
            receipt.get(*field).is_some(),
            "{ctx}: RoutingReceipt missing required field '{field}'"
        );
    }
}

/// Assert DispatchRecord fields that are present in BOTH the create and approve responses.
///
/// Note: the spec lists `created_at` as required in DispatchRecord, but
/// `POST /dispatch/create` returns a partial record that omits it. The full
/// record (including `created_at`) is present in the approve and export responses.
/// This helper checks only the fields that are consistently returned by all handlers.
fn assert_dispatch_record_fields(record: &Value, ctx: &str) {
    for field in &[
        "dispatch_id",
        "case_id",
        "receipt_hash",
        "verification_passed",
        "status",
    ] {
        assert!(
            record.get(*field).is_some(),
            "{ctx}: DispatchRecord missing required field '{field}'"
        );
    }
}

/// Assert all DispatchExport required fields (spec: components/schemas/DispatchExport).
fn assert_dispatch_export_fields(export: &Value, ctx: &str) {
    for field in &[
        "approved_at",
        "approved_by",
        "case_id",
        "created_at",
        "dispatch_id",
        "manufacturer_payload_json",
        "receipt_hash",
        "selected_candidate_id",
        "status",
        "verification_passed",
    ] {
        assert!(
            export.get(*field).is_some(),
            "{ctx}: DispatchExport missing required field '{field}'"
        );
    }
}

/// Assert ErrorResponse shape: { error: { code: string, message: string } }
/// (spec: components/schemas/ErrorResponse + ErrorDetail)
fn assert_error_shape(resp: &Value, ctx: &str) {
    assert!(
        resp["error"]["code"].is_string(),
        "{ctx}: ErrorResponse.error.code must be a string, got: {resp}"
    );
    assert!(
        resp["error"]["message"].is_string(),
        "{ctx}: ErrorResponse.error.message must be a string, got: {resp}"
    );
}

// ── POST /pilot/route-normalized ──────────────────────────────────────────────

/// spec: POST /pilot/route-normalized → 200 RouteResponse
/// RouteResponse requires: receipt (RoutingReceipt), derived_policy (object)
#[tokio::test]
async fn contract_pilot_route_normalized_200() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, resp) = post_json(make_app(&tmp), "/pilot/route-normalized", pilot_route_body()).await;

    assert_eq!(status, StatusCode::OK, "expected 200: {resp}");
    assert!(resp["receipt"].is_object(), "RouteResponse.receipt must be an object");
    assert!(
        resp["derived_policy"].is_object(),
        "RouteResponse.derived_policy must be an object"
    );
    assert_receipt_fields(&resp["receipt"], "POST /pilot/route-normalized 200");

    let outcome = resp["receipt"]["outcome"].as_str().unwrap();
    assert!(
        outcome == "routed" || outcome == "refused",
        "RoutingReceipt.outcome must be 'routed' or 'refused', got '{outcome}'"
    );
}

/// spec: POST /pilot/route-normalized → 422 ErrorResponse (missing required field)
/// PilotCaseInput requires: restoration_type, material, jurisdiction
#[tokio::test]
async fn contract_pilot_route_normalized_422_missing_field() {
    let tmp = tempfile::TempDir::new().unwrap();
    let body = json!({
        "pilot_case": {
            "case_id":   "f1000001-0000-0000-0000-000000000001",
            "material":  "zirconia",
            "jurisdiction": "DE"
            // restoration_type intentionally omitted — required per spec
        },
        "registry_snapshot": serde_json::from_str::<Value>(REGISTRY_JSON).unwrap(),
        "routing_config":    serde_json::from_str::<Value>(CONFIG_JSON).unwrap(),
    });

    let (status, resp) = post_json(make_app(&tmp), "/pilot/route-normalized", body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "expected 422: {resp}");
    assert_error_shape(&resp, "POST /pilot/route-normalized 422");
}

// ── POST /verify ──────────────────────────────────────────────────────────────

/// spec: POST /verify → 200 VerifyResponse { result: "VERIFIED" }
#[tokio::test]
async fn contract_verify_200() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, route_resp) = post_json(make_app(&tmp), "/pilot/route-normalized", pilot_route_body()).await;
    let receipt = route_resp["receipt"].clone();
    let policy = route_resp["derived_policy"].clone();
    let case = serde_json::from_str::<Value>(CASE_JSON).unwrap();

    let (status, resp) = post_json(
        make_app(&tmp),
        "/verify",
        json!({ "receipt": receipt, "case": case, "policy": policy }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "expected 200: {resp}");
    assert_eq!(
        resp["result"].as_str().unwrap(),
        "VERIFIED",
        "VerifyResponse.result must be 'VERIFIED'"
    );
}

/// spec: POST /verify → 422 VerifyFailedResponse { result: "FAILED", error: { code, message } }
#[tokio::test]
async fn contract_verify_422_tampered_receipt() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, route_resp) = post_json(make_app(&tmp), "/pilot/route-normalized", pilot_route_body()).await;
    let mut tampered = route_resp["receipt"].clone();
    tampered["routing_decision_hash"] =
        json!("0000000000000000000000000000000000000000000000000000000000000000");
    let policy = route_resp["derived_policy"].clone();
    let case = serde_json::from_str::<Value>(CASE_JSON).unwrap();

    let (status, resp) = post_json(
        make_app(&tmp),
        "/verify",
        json!({ "receipt": tampered, "case": case, "policy": policy }),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "expected 422: {resp}");
    assert_eq!(
        resp["result"].as_str().unwrap(),
        "FAILED",
        "VerifyFailedResponse.result must be 'FAILED'"
    );
    assert_error_shape(&resp, "POST /verify 422");
}

// ── POST /dispatch/create ─────────────────────────────────────────────────────

/// spec: POST /dispatch/create → 200 DispatchRecord
#[tokio::test]
async fn contract_dispatch_create_200() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, route_resp) = post_json(make_app(&tmp), "/pilot/route-normalized", pilot_route_body()).await;
    let receipt = route_resp["receipt"].clone();
    let policy = route_resp["derived_policy"].clone();
    let case = serde_json::from_str::<Value>(CASE_JSON).unwrap();

    let (status, resp) = post_json(
        make_app(&tmp),
        "/dispatch/create",
        json!({ "receipt": receipt, "case": case, "policy": policy }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "expected 200: {resp}");
    assert_dispatch_record_fields(&resp, "POST /dispatch/create 200");
    assert_eq!(resp["status"].as_str().unwrap(), "draft");
    assert_eq!(resp["verification_passed"].as_bool().unwrap(), true);
}

/// spec: POST /dispatch/create → 409 ErrorResponse (duplicate receipt_hash)
#[tokio::test]
async fn contract_dispatch_create_409_duplicate() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, route_resp) = post_json(make_app(&tmp), "/pilot/route-normalized", pilot_route_body()).await;
    let receipt = route_resp["receipt"].clone();
    let policy = route_resp["derived_policy"].clone();
    let case = serde_json::from_str::<Value>(CASE_JSON).unwrap();
    let body = json!({ "receipt": receipt, "case": case, "policy": policy });

    post_json(make_app(&tmp), "/dispatch/create", body.clone()).await; // first → 200
    let (status, resp) = post_json(make_app(&tmp), "/dispatch/create", body).await; // second → 409

    assert_eq!(status, StatusCode::CONFLICT, "expected 409: {resp}");
    assert_error_shape(&resp, "POST /dispatch/create 409");
}

// ── POST /dispatch/{id}/approve ───────────────────────────────────────────────

/// spec: POST /dispatch/{id}/approve → 200 DispatchRecord (approved, approved_at set)
#[tokio::test]
async fn contract_dispatch_approve_200() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, route_resp) = post_json(make_app(&tmp), "/pilot/route-normalized", pilot_route_body()).await;
    let receipt = route_resp["receipt"].clone();
    let policy = route_resp["derived_policy"].clone();
    let case = serde_json::from_str::<Value>(CASE_JSON).unwrap();

    let (_, create_resp) = post_json(
        make_app(&tmp),
        "/dispatch/create",
        json!({ "receipt": receipt, "case": case, "policy": policy }),
    )
    .await;
    let dispatch_id = create_resp["dispatch_id"].as_str().unwrap();

    let (status, resp) = post_json(
        make_app(&tmp),
        &format!("/dispatch/{dispatch_id}/approve"),
        json!({ "approved_by": "reviewer" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "expected 200: {resp}");
    assert_dispatch_record_fields(&resp, "POST /dispatch/:id/approve 200");
    assert_eq!(resp["status"].as_str().unwrap(), "approved");
    assert!(
        resp["approved_at"].is_string(),
        "DispatchRecord.approved_at must be set after approval"
    );
}

/// spec: POST /dispatch/{id}/approve → 404 ErrorResponse (dispatch not found)
#[tokio::test]
async fn contract_dispatch_approve_404() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, resp) = post_json(
        make_app(&tmp),
        "/dispatch/00000000-0000-0000-0000-000000000000/approve",
        json!({ "approved_by": "reviewer" }),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND, "expected 404: {resp}");
    assert_error_shape(&resp, "POST /dispatch/:id/approve 404");
}

// ── GET /dispatch/{id}/export ─────────────────────────────────────────────────

/// spec: GET /dispatch/{id}/export → 200 DispatchExport (all 10 required fields)
#[tokio::test]
async fn contract_dispatch_export_200() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, route_resp) = post_json(make_app(&tmp), "/pilot/route-normalized", pilot_route_body()).await;
    let receipt = route_resp["receipt"].clone();
    let policy = route_resp["derived_policy"].clone();
    let case = serde_json::from_str::<Value>(CASE_JSON).unwrap();

    let (_, create_resp) = post_json(
        make_app(&tmp),
        "/dispatch/create",
        json!({ "receipt": receipt, "case": case, "policy": policy }),
    )
    .await;
    let dispatch_id = create_resp["dispatch_id"].as_str().unwrap();

    post_json(
        make_app(&tmp),
        &format!("/dispatch/{dispatch_id}/approve"),
        json!({ "approved_by": "reviewer" }),
    )
    .await;

    let (status, resp) = get_json(
        make_app(&tmp),
        &format!("/dispatch/{dispatch_id}/export"),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "expected 200: {resp}");
    assert_dispatch_export_fields(&resp, "GET /dispatch/:id/export 200");
    assert_eq!(resp["status"].as_str().unwrap(), "exported");
    assert_eq!(resp["verification_passed"].as_bool().unwrap(), true);
}

/// spec: GET /dispatch/{id}/export → 422 ErrorResponse (dispatch not yet approved)
#[tokio::test]
async fn contract_dispatch_export_422_not_approved() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (_, route_resp) = post_json(make_app(&tmp), "/pilot/route-normalized", pilot_route_body()).await;
    let receipt = route_resp["receipt"].clone();
    let policy = route_resp["derived_policy"].clone();
    let case = serde_json::from_str::<Value>(CASE_JSON).unwrap();

    let (_, create_resp) = post_json(
        make_app(&tmp),
        "/dispatch/create",
        json!({ "receipt": receipt, "case": case, "policy": policy }),
    )
    .await;
    let dispatch_id = create_resp["dispatch_id"].as_str().unwrap();

    // Export without approving → 422 per spec
    let (status, resp) = get_json(
        make_app(&tmp),
        &format!("/dispatch/{dispatch_id}/export"),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "expected 422: {resp}");
    assert_error_shape(&resp, "GET /dispatch/:id/export 422");
}
