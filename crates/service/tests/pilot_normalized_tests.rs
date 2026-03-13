//! Contract tests for POST /pilot/route-normalized.
//!
//! Proves:
//!  1. Normalized input routes successfully — outcome, candidate, receipt_hash present.
//!  2. Identical inputs produce deterministic output (same receipt_hash both calls).
//!  3. Same semantic case via normalized path and direct /route path → same receipt_hash.
//!  4. Invalid pilot_case field returns 422 with stable error code.
//!  5. Missing top-level fields return 422 with parse_error.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::util::ServiceExt;

// ── Test helpers ──────────────────────────────────────────────────────────────

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

// ── Shared fixtures ───────────────────────────────────────────────────────────

fn pilot_case() -> Value {
    json!({
        "case_id": "f1000001-0000-0000-0000-000000000001",
        "restoration_type": "crown",
        "material": "zirconia",
        "jurisdiction": "DE"
    })
}

fn registry_snapshot() -> Value {
    json!([{
        "attestation_statuses": ["verified"],
        "capabilities": ["crown"],
        "country": "germany",
        "display_name": "Pilot GmbH",
        "is_active": true,
        "jurisdictions_served": ["germany"],
        "manufacturer_id": "pilot-de-001",
        "materials_supported": ["zirconia"],
        "sla_days": 5
    }])
}

fn routing_config() -> Value {
    json!({"jurisdiction": "DE", "routing_policy": "allow_domestic_and_cross_border"})
}

fn normalized_route_body() -> Value {
    json!({
        "pilot_case": pilot_case(),
        "registry_snapshot": registry_snapshot(),
        "routing_config": routing_config()
    })
}

// ── Test 1: normalized input routes successfully ──────────────────────────────

#[tokio::test]
async fn normalized_input_routes_successfully() {
    let (status, body) = post_json("/pilot/route-normalized", normalized_route_body()).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["receipt"]["outcome"], "routed");
    assert_eq!(body["receipt"]["selected_candidate_id"], "pilot-de-001");
    assert!(
        body["receipt"]["receipt_hash"].is_string(),
        "receipt_hash must be present"
    );
    assert!(
        body["derived_policy"].is_object(),
        "derived_policy must be present"
    );
}

// ── Test 2: deterministic output ─────────────────────────────────────────────

#[tokio::test]
async fn normalized_input_produces_deterministic_receipt_hash() {
    let (_, r1) = post_json("/pilot/route-normalized", normalized_route_body()).await;
    let (_, r2) = post_json("/pilot/route-normalized", normalized_route_body()).await;

    assert_eq!(
        r1["receipt"]["receipt_hash"], r2["receipt"]["receipt_hash"],
        "identical inputs must produce identical receipt hashes"
    );
}

// ── Test 3: kernel semantics unchanged ───────────────────────────────────────

#[tokio::test]
async fn normalized_path_and_direct_path_produce_same_receipt_hash() {
    // Route via the normalized pilot endpoint.
    let (status_norm, norm) =
        post_json("/pilot/route-normalized", normalized_route_body()).await;
    assert_eq!(status_norm, StatusCode::OK);

    // Route via the existing /route endpoint with the equivalent full CaseInput.
    let direct_body = json!({
        "case": {
            "case_id": "f1000001-0000-0000-0000-000000000001",
            "jurisdiction": "DE",
            "routing_policy": "allow_domestic_and_cross_border",
            "patient_country": "germany",
            "manufacturer_country": "germany",
            "material": "zirconia",
            "procedure": "crown",
            "file_type": "stl"
        },
        "registry_snapshot": registry_snapshot(),
        "routing_config": routing_config()
    });
    let (status_direct, direct) = post_json("/route", direct_body).await;
    assert_eq!(status_direct, StatusCode::OK);

    assert_eq!(
        norm["receipt"]["receipt_hash"], direct["receipt"]["receipt_hash"],
        "normalized path must produce the same receipt hash as the direct CaseInput path"
    );
}

// ── Test 4: invalid jurisdiction returns 422 with stable error code ───────────

#[tokio::test]
async fn unknown_jurisdiction_returns_422_with_parse_error_code() {
    let body = json!({
        "pilot_case": {
            "restoration_type": "crown",
            "material": "zirconia",
            "jurisdiction": "XX"
        },
        "registry_snapshot": registry_snapshot(),
        "routing_config": routing_config()
    });
    let (status, resp) = post_json("/pilot/route-normalized", body).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(resp["error"]["code"], "parse_error");
    assert!(
        resp["error"]["message"]
            .as_str()
            .unwrap_or("")
            .contains("XX"),
        "error message must identify the bad value"
    );
}

// ── Test 5: missing required pilot_case field returns 422 ─────────────────────

#[tokio::test]
async fn missing_restoration_type_returns_422() {
    let body = json!({
        "pilot_case": {"material": "zirconia", "jurisdiction": "DE"},
        "registry_snapshot": registry_snapshot(),
        "routing_config": routing_config()
    });
    let (status, resp) = post_json("/pilot/route-normalized", body).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(resp["error"]["code"], "parse_error");
}

// ── Test 6: missing top-level field returns 422 ───────────────────────────────

#[tokio::test]
async fn missing_registry_snapshot_returns_422() {
    let body = json!({
        "pilot_case": pilot_case(),
        "routing_config": routing_config()
        // registry_snapshot deliberately omitted
    });
    let (status, resp) = post_json("/pilot/route-normalized", body).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(resp["error"]["code"], "parse_error");
}
