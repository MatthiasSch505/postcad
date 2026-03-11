//! Tests for GET /routes — route history endpoint.
//!
//! Each test uses isolated temporary directories so tests are fully
//! deterministic and do not share state.

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

// ── Pilot fixtures ────────────────────────────────────────────────────────────

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

// ── Store-and-route helper ────────────────────────────────────────────────────

/// Store a case then route it. Returns the receipt_hash from the response.
async fn store_and_route(tmp: &tempfile::TempDir, case: Value) -> String {
    let case_id = case["case_id"].as_str().unwrap().to_string();

    let (s1, _) = post_json(make_app(tmp), "/cases", case).await;
    assert_eq!(s1, StatusCode::CREATED);

    let (s2, body) = post_json(
        make_app(tmp),
        &format!("/cases/{case_id}/route"),
        json!({ "registry": pilot_registry(), "config": pilot_config() }),
    )
    .await;
    assert_eq!(s2, StatusCode::OK, "route failed: {body}");
    body["receipt_hash"].as_str().unwrap().to_string()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// GET /routes on an empty receipts directory returns an empty list.
#[tokio::test]
async fn get_routes_returns_empty_when_no_receipts() {
    let tmp = tempfile::TempDir::new().unwrap();
    let (status, body) = get_json(make_app(&tmp), "/routes").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["routes"], json!([]));
}

/// GET /routes after routing a case returns one history entry.
#[tokio::test]
async fn get_routes_returns_history_entries() {
    let tmp = tempfile::TempDir::new().unwrap();
    let receipt_hash = store_and_route(&tmp, pilot_case()).await;

    let (status, body) = get_json(make_app(&tmp), "/routes").await;
    assert_eq!(status, StatusCode::OK);

    let routes = body["routes"].as_array().unwrap();
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0]["receipt_hash"], receipt_hash);
}

/// Each history entry contains exactly the four specified fields.
#[tokio::test]
async fn routes_extract_expected_fields() {
    let tmp = tempfile::TempDir::new().unwrap();
    store_and_route(&tmp, pilot_case()).await;

    let (status, body) = get_json(make_app(&tmp), "/routes").await;
    assert_eq!(status, StatusCode::OK);

    let entry = &body["routes"][0];
    let obj = entry.as_object().unwrap();
    assert_eq!(obj.len(), 4, "each entry must have exactly 4 fields");
    assert!(obj.contains_key("case_id"), "missing case_id");
    assert!(obj.contains_key("receipt_hash"), "missing receipt_hash");
    assert!(
        obj.contains_key("selected_candidate_id"),
        "missing selected_candidate_id"
    );
    assert!(obj.contains_key("timestamp"), "missing timestamp");

    // case_id must round-trip to the original case.
    let pilot = pilot_case();
    assert_eq!(entry["case_id"], pilot["case_id"]);

    // receipt_hash must be a 64-char hex digest.
    let h = entry["receipt_hash"].as_str().unwrap();
    assert_eq!(h.len(), 64, "receipt_hash must be 64 hex chars");

    // timestamp must look like RFC 3339 (at minimum non-empty).
    let ts = entry["timestamp"].as_str().unwrap();
    assert!(!ts.is_empty(), "timestamp must not be empty");
    assert!(ts.contains('T'), "timestamp must contain 'T' separator");
    assert!(ts.ends_with('Z'), "timestamp must end with 'Z'");
}

/// With two receipts, GET /routes returns both sorted by (timestamp DESC,
/// receipt_hash ASC). When both files share the same modification timestamp
/// (as is typical in tests) the tiebreaker produces a deterministic order.
#[tokio::test]
async fn routes_sorted_by_timestamp() {
    let tmp = tempfile::TempDir::new().unwrap();

    // Two distinct cases with distinct IDs.
    let case_a = pilot_case(); // case_id = f1000001-...
    let case_b = json!({
        "case_id": "f2000002-0000-0000-0000-000000000002",
        "jurisdiction": "DE",
        "routing_policy": "allow_domestic_and_cross_border",
        "patient_country": "germany",
        "manufacturer_country": "germany",
        "material": "zirconia",
        "procedure": "crown",
        "file_type": "stl"
    });

    let hash_a = store_and_route(&tmp, case_a).await;
    let hash_b = store_and_route(&tmp, case_b).await;

    let (status, body) = get_json(make_app(&tmp), "/routes").await;
    assert_eq!(status, StatusCode::OK);

    let routes = body["routes"].as_array().unwrap();
    assert_eq!(routes.len(), 2);

    // Both hashes must be present.
    let hashes: Vec<&str> = routes
        .iter()
        .map(|r| r["receipt_hash"].as_str().unwrap())
        .collect();
    assert!(hashes.contains(&hash_a.as_str()));
    assert!(hashes.contains(&hash_b.as_str()));

    // If timestamps are equal (common in tests), order must be deterministic:
    // receipt_hash ascending.
    let ts0 = routes[0]["timestamp"].as_str().unwrap();
    let ts1 = routes[1]["timestamp"].as_str().unwrap();
    if ts0 == ts1 {
        assert!(
            hashes[0] <= hashes[1],
            "when timestamps are equal, routes must be sorted by receipt_hash ascending"
        );
    }
}
