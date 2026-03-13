//! End-to-end smoke test for the pilot reviewer workflow.
//!
//! Exercises the full operator path in a single test:
//!
//!   1. POST /pilot/route-normalized → receipt + derived_policy
//!   2. GET  /reviewer               → reviewer shell HTML is accessible
//!   3. POST /dispatch/create        → draft dispatch bound to the receipt
//!   4. POST /dispatch/{id}/approve  → dispatch transitions to approved
//!   5. GET  /dispatch/{id}/export   → approved dispatch exported with all fields
//!
//! State persists across steps via a shared `TempDir` (all stores point to the
//! same directory on disk). Each call creates a fresh router from `make_app`
//! because `tower::ServiceExt::oneshot` consumes the router.
//!
//! No port binding. No schema changes. No new fixtures.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::util::ServiceExt;

// ── Pilot fixtures ────────────────────────────────────────────────────────────

/// Full CaseInput — used as the `case` field for dispatch/create + verify.
const PILOT_CASE_JSON: &str = include_str!("../../../examples/pilot/case.json");
const REGISTRY_JSON: &str = include_str!("../../../examples/pilot/registry_snapshot.json");
const CONFIG_JSON: &str = include_str!("../../../examples/pilot/config.json");

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
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    (status, json)
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
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    (status, json)
}

async fn get_html(app: axum::Router, uri: &str) -> (StatusCode, String) {
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
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

// ── Smoke test ────────────────────────────────────────────────────────────────

/// Full pilot reviewer workflow smoke test.
///
/// Mirrors the sequence a reviewer executes in the reviewer shell:
/// normalized route → reviewer accessible → dispatch create → approve → export.
#[tokio::test]
async fn reviewer_workflow_normalized_route_to_export() {
    let tmp = tempfile::TempDir::new().unwrap();

    // ── Step 1: submit normalized pilot input ─────────────────────────────────
    let route_body = json!({
        "pilot_case": {
            "case_id": "f1000001-0000-0000-0000-000000000001",
            "restoration_type": "crown",
            "material": "zirconia",
            "jurisdiction": "DE"
        },
        "registry_snapshot": serde_json::from_str::<Value>(REGISTRY_JSON).unwrap(),
        "routing_config":    serde_json::from_str::<Value>(CONFIG_JSON).unwrap(),
    });
    let (route_status, route_resp) =
        post_json(make_app(&tmp), "/pilot/route-normalized", route_body).await;

    assert_eq!(route_status, StatusCode::OK, "step 1 route failed: {route_resp}");
    assert_eq!(route_resp["receipt"]["outcome"], "routed");
    assert_eq!(
        route_resp["receipt"]["selected_candidate_id"],
        "pilot-de-001",
        "canonical pilot case must route to pilot-de-001"
    );
    assert!(
        route_resp["receipt"]["receipt_hash"].is_string(),
        "receipt_hash must be present"
    );
    assert!(
        route_resp["derived_policy"].is_object(),
        "derived_policy must be present for dispatch binding"
    );

    let receipt = route_resp["receipt"].clone();
    let derived_policy = route_resp["derived_policy"].clone();
    let receipt_hash = receipt["receipt_hash"].as_str().unwrap().to_string();

    // ── Step 2: reviewer shell is reachable ───────────────────────────────────
    let (reviewer_status, reviewer_html) =
        get_html(make_app(&tmp), "/reviewer").await;

    assert_eq!(reviewer_status, StatusCode::OK, "step 2 reviewer shell unreachable");
    assert!(
        reviewer_html.contains("/pilot/route-normalized"),
        "reviewer HTML must reference the normalized pilot endpoint"
    );
    assert!(
        reviewer_html.contains("/dispatch/create"),
        "reviewer HTML must reference the dispatch create endpoint"
    );

    // ── Step 3: create dispatch from the normalized receipt ───────────────────
    //
    // The reviewer shell sends: receipt (from step 1) + full CaseInput
    // (examples/pilot/case.json) + derived_policy (from step 1).
    // The normalized pilot case and the full CaseInput share the same
    // case_fingerprint — verified by the kernel before dispatch is created.
    let create_body = json!({
        "receipt": receipt,
        "case":    serde_json::from_str::<Value>(PILOT_CASE_JSON).unwrap(),
        "policy":  derived_policy,
    });
    let (create_status, create_resp) =
        post_json(make_app(&tmp), "/dispatch/create", create_body).await;

    assert_eq!(create_status, StatusCode::OK, "step 3 dispatch create failed: {create_resp}");
    assert_eq!(create_resp["status"], "draft");
    assert_eq!(create_resp["verification_passed"], true);
    assert_eq!(create_resp["receipt_hash"], receipt_hash);
    assert!(
        create_resp["dispatch_id"].is_string(),
        "dispatch_id must be present"
    );

    let dispatch_id = create_resp["dispatch_id"].as_str().unwrap().to_string();

    // ── Step 4: approve dispatch ──────────────────────────────────────────────
    let (approve_status, approve_resp) = post_json(
        make_app(&tmp),
        &format!("/dispatch/{dispatch_id}/approve"),
        json!({"approved_by": "reviewer"}),
    )
    .await;

    assert_eq!(approve_status, StatusCode::OK, "step 4 approve failed: {approve_resp}");
    assert_eq!(approve_resp["status"], "approved");
    assert_eq!(approve_resp["approved_by"], "reviewer");
    assert_eq!(approve_resp["dispatch_id"], dispatch_id);
    assert!(
        approve_resp["approved_at"].is_string(),
        "approved_at must be set after approval"
    );

    // ── Step 5: export dispatch ───────────────────────────────────────────────
    let (export_status, export_resp) =
        get_json(make_app(&tmp), &format!("/dispatch/{dispatch_id}/export")).await;

    assert_eq!(export_status, StatusCode::OK, "step 5 export failed: {export_resp}");
    assert_eq!(export_resp["status"], "exported");
    assert_eq!(export_resp["verification_passed"], true);
    assert_eq!(export_resp["dispatch_id"], dispatch_id);
    assert_eq!(export_resp["receipt_hash"], receipt_hash);
    assert_eq!(
        export_resp["selected_candidate_id"],
        "pilot-de-001",
        "export must carry the manufacturer selected by the kernel"
    );
    assert!(
        export_resp["approved_at"].is_string(),
        "approved_at must be present in export"
    );
}
