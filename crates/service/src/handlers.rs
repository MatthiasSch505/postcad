use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use postcad_cli::{
    build_manifest, route_case_from_policy_json, route_case_from_registry_json,
    verify_receipt_from_policy_json, CaseInput, PROTOCOL_VERSION,
};
use serde_json::{json, Value};

use crate::case_store::{CaseStore, CaseStoreError, StoreOutcome};
use crate::receipt_store::{ReceiptStore, ReceiptStoreError};
use crate::AppState;

/// POST /route-case
///
/// Request: `{"case": {...}, "policy": {...}}`
/// Success (200): `{"receipt": {...}}`
/// Error (422): `{"error": {"code": "...", "message": "..."}}`
///
/// Delegates directly to `route_case_from_policy_json`; no routing logic here.
pub async fn route_case(Json(body): Json<Value>) -> impl IntoResponse {
    let case = &body["case"];
    let policy = &body["policy"];

    if case.is_null() || policy.is_null() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": {"code": "parse_error", "message": "request must contain 'case' and 'policy' fields"}})),
        )
            .into_response();
    }

    let case_json = serde_json::to_string(case).unwrap();
    let policy_json = serde_json::to_string(policy).unwrap();

    match route_case_from_policy_json(&case_json, &policy_json) {
        Ok(receipt) => (
            StatusCode::OK,
            Json(json!({"receipt": serde_json::to_value(&receipt).unwrap()})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": {"code": e.code(), "message": e.to_string()}})),
        )
            .into_response(),
    }
}

/// POST /verify-receipt
///
/// Request: `{"receipt": {...}, "case": {...}, "policy": {...}}`
/// Success (200): `{"result": "VERIFIED"}`
/// Failure (422): `{"result": "FAILED", "error": {"code": "...", "message": "..."}}`
///
/// Delegates directly to `verify_receipt_from_policy_json`; no verification logic here.
pub async fn verify_receipt(Json(body): Json<Value>) -> impl IntoResponse {
    let receipt = &body["receipt"];
    let case = &body["case"];
    let policy = &body["policy"];

    if receipt.is_null() || case.is_null() || policy.is_null() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"result": "FAILED", "error": {"code": "receipt_parse_failed", "message": "request must contain 'receipt', 'case', and 'policy' fields"}})),
        )
            .into_response();
    }

    let receipt_json = serde_json::to_string(receipt).unwrap();
    let case_json = serde_json::to_string(case).unwrap();
    let policy_json = serde_json::to_string(policy).unwrap();

    match verify_receipt_from_policy_json(&receipt_json, &case_json, &policy_json) {
        Ok(()) => (StatusCode::OK, Json(json!({"result": "VERIFIED"}))).into_response(),
        Err(f) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"result": "FAILED", "error": {"code": f.code, "message": f.message}})),
        )
            .into_response(),
    }
}

/// POST /route-case-from-registry
///
/// Request: `{"case": {...}, "registry": [...], "config": {...}}`
/// Success (200): `{"receipt": {...}, "derived_policy": {...}}`
/// Error (422): `{"error": {"code": "...", "message": "..."}}`
///
/// Derives a RoutingPolicyBundle from typed ManufacturerRecord data and routes
/// via the kernel. Returns both the receipt and the derived policy so callers
/// can verify the receipt without re-deriving.
pub async fn route_case_from_registry(Json(body): Json<Value>) -> impl IntoResponse {
    let case = &body["case"];
    let registry = &body["registry"];
    let config = &body["config"];

    if case.is_null() || registry.is_null() || config.is_null() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": {"code": "parse_error", "message": "request must contain 'case', 'registry', and 'config' fields"}})),
        )
            .into_response();
    }

    let case_json = serde_json::to_string(case).unwrap();
    let registry_json = serde_json::to_string(registry).unwrap();
    let config_json = serde_json::to_string(config).unwrap();

    match route_case_from_registry_json(&case_json, &registry_json, &config_json) {
        Ok(result) => {
            let derived_policy: Value =
                serde_json::from_str(&result.derived_policy_json).unwrap_or(Value::Null);
            (
                StatusCode::OK,
                Json(json!({
                    "receipt": serde_json::to_value(&result.receipt).unwrap(),
                    "derived_policy": derived_policy,
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": {"code": e.code(), "message": e.to_string()}})),
        )
            .into_response(),
    }
}

/// GET /protocol-manifest
///
/// Returns the static protocol manifest. No inputs required.
pub async fn protocol_manifest() -> impl IntoResponse {
    Json(serde_json::to_value(build_manifest()).unwrap())
}

// ── Pilot endpoints ───────────────────────────────────────────────────────────

/// GET /health
///
/// Liveness check. Always returns HTTP 200 `{"status":"ok"}`.
pub async fn health() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}

/// GET /version
///
/// Returns service identity and kernel version strings that are already
/// defined as compile-time constants in the codebase. No new version
/// semantics are introduced.
pub async fn version() -> impl IntoResponse {
    let manifest = build_manifest();
    Json(json!({
        "service": "postcad-service",
        "protocol_version": PROTOCOL_VERSION,
        "routing_kernel_version": manifest.routing_kernel_version,
    }))
}

/// POST /route
///
/// Pilot routing endpoint.
/// Request: `{"case": {...}, "registry_snapshot": [...], "routing_config": {...}}`
/// Success (200): `{"receipt": {...}, "derived_policy": {...}}`
/// Error (422): `{"error": {"code": "...", "message": "..."}}`
///
/// Delegates directly to `route_case_from_registry_json`; no routing logic here.
pub async fn pilot_route(Json(body): Json<Value>) -> impl IntoResponse {
    let case = &body["case"];
    let registry_snapshot = &body["registry_snapshot"];
    let routing_config = &body["routing_config"];

    if case.is_null() || registry_snapshot.is_null() || routing_config.is_null() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": {"code": "parse_error", "message": "request must contain 'case', 'registry_snapshot', and 'routing_config' fields"}})),
        )
            .into_response();
    }

    let case_json = serde_json::to_string(case).unwrap();
    let registry_json = serde_json::to_string(registry_snapshot).unwrap();
    let config_json = serde_json::to_string(routing_config).unwrap();

    match route_case_from_registry_json(&case_json, &registry_json, &config_json) {
        Ok(result) => {
            let derived_policy: Value =
                serde_json::from_str(&result.derived_policy_json).unwrap_or(Value::Null);
            (
                StatusCode::OK,
                Json(json!({
                    "receipt": serde_json::to_value(&result.receipt).unwrap(),
                    "derived_policy": derived_policy,
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": {"code": e.code(), "message": e.to_string()}})),
        )
            .into_response(),
    }
}

/// POST /verify
///
/// Pilot verification endpoint.
/// Request: `{"receipt": {...}, "case": {...}, "policy": {...}}`
/// Success (200): `{"result": "VERIFIED"}`
/// Failure (422): `{"result": "FAILED", "error": {"code": "...", "message": "..."}}`
///
/// Delegates directly to `verify_receipt_from_policy_json`; no verification logic here.
pub async fn pilot_verify(Json(body): Json<Value>) -> impl IntoResponse {
    let receipt = &body["receipt"];
    let case = &body["case"];
    let policy = &body["policy"];

    if receipt.is_null() || case.is_null() || policy.is_null() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"result": "FAILED", "error": {"code": "receipt_parse_failed", "message": "request must contain 'receipt', 'case', and 'policy' fields"}})),
        )
            .into_response();
    }

    let receipt_json = serde_json::to_string(receipt).unwrap();
    let case_json = serde_json::to_string(case).unwrap();
    let policy_json = serde_json::to_string(policy).unwrap();

    match verify_receipt_from_policy_json(&receipt_json, &case_json, &policy_json) {
        Ok(()) => (StatusCode::OK, Json(json!({"result": "VERIFIED"}))).into_response(),
        Err(f) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"result": "FAILED", "error": {"code": f.code, "message": f.message}})),
        )
            .into_response(),
    }
}

// ── Case intake endpoints ─────────────────────────────────────────────────────

/// POST /cases
///
/// Request body: case JSON matching the existing routing case shape.
/// `case_id` must be present.
///
/// Success (201): `{"case_id": "...", "stored": true}`
/// Identical re-post (200): `{"case_id": "...", "stored": true}`
/// Conflict (409): `{"error": {"code": "case_id_conflict", "message": "..."}}`
/// Invalid input (422): `{"error": {"code": "parse_error", "message": "..."}}`
pub async fn post_case(
    State(store): State<Arc<CaseStore>>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    // Validate using the existing CaseInput parser.
    let input: CaseInput = match serde_json::from_value(body.clone()) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"error": {"code": "parse_error", "message": e.to_string()}})),
            )
                .into_response();
        }
    };

    let case_id = match input.case_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"error": {"code": "parse_error", "message": "case_id is required"}})),
            )
                .into_response();
        }
    };

    let canonical = serde_json::to_string_pretty(&body).unwrap();

    match store.store(&case_id, &canonical) {
        Ok(StoreOutcome::Created) => (
            StatusCode::CREATED,
            Json(json!({"case_id": case_id, "stored": true})),
        )
            .into_response(),
        Ok(StoreOutcome::Identical) => (
            StatusCode::OK,
            Json(json!({"case_id": case_id, "stored": true})),
        )
            .into_response(),
        Err(CaseStoreError::Conflict) => (
            StatusCode::CONFLICT,
            Json(json!({"error": {"code": "case_id_conflict",
                "message": format!("case_id '{case_id}' already stored with different content")}})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "storage_error", "message": e.to_string()}})),
        )
            .into_response(),
    }
}

/// GET /cases
///
/// Returns the sorted list of stored case IDs.
/// Success (200): `{"case_ids": ["...", ...]}`
pub async fn list_cases(State(store): State<Arc<CaseStore>>) -> impl IntoResponse {
    match store.list() {
        Ok(ids) => (StatusCode::OK, Json(json!({"case_ids": ids}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "storage_error", "message": e.to_string()}})),
        )
            .into_response(),
    }
}

/// GET /cases/:case_id
///
/// Returns the stored case JSON.
/// Success (200): the original case JSON object
/// Not found (404): `{"error": {"code": "case_not_found", "message": "..."}}`
pub async fn get_case(
    State(store): State<Arc<CaseStore>>,
    Path(case_id): Path<String>,
) -> impl IntoResponse {
    match store.get(&case_id) {
        Ok(Some(value)) => (StatusCode::OK, Json(value)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(
                json!({"error": {"code": "case_not_found", "message": format!("case '{case_id}' not found")}}),
            ),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "storage_error", "message": e.to_string()}})),
        )
            .into_response(),
    }
}

// ── Stored-case routing endpoint ──────────────────────────────────────────────

/// POST /cases/:case_id/route
///
/// Request body: `{"registry": [...], "config": {...}}`
///
/// Reads the stored case, runs the routing kernel, persists the receipt to
/// `data/receipts/{receipt_hash}.json`, and returns:
///
/// Success (200):   `{"case_id": "...", "receipt_hash": "...", "selected_candidate_id": "..."}`
/// Refused (422):   `{"error": {"code": "routing_refused", "message": "..."}}`
/// Not found (404): `{"error": {"code": "case_not_found", "message": "..."}}`
/// Internal (500):  `{"error": {"code": "internal_error", "message": "..."}}`
pub async fn route_stored_case(
    State(state): State<Arc<AppState>>,
    Path(case_id): Path<String>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    // 1. Load the stored case.
    let case_value = match state.case_store.get(&case_id) {
        Ok(Some(v)) => v,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": {"code": "case_not_found",
                    "message": format!("case '{case_id}' not found")}})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": {"code": "internal_error", "message": e.to_string()}})),
            )
                .into_response();
        }
    };

    // 2. Extract registry and config from the request body.
    let registry = &body["registry"];
    let config = &body["config"];
    if registry.is_null() || config.is_null() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": {"code": "parse_error",
                "message": "request must contain 'registry' and 'config' fields"}})),
        )
            .into_response();
    }

    let case_json = serde_json::to_string(&case_value).unwrap();
    let registry_json = serde_json::to_string(registry).unwrap();
    let config_json = serde_json::to_string(config).unwrap();

    // 3. Run the routing kernel (same path as /route-case-from-registry).
    let result = match route_case_from_registry_json(&case_json, &registry_json, &config_json) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"error": {"code": e.code(), "message": e.to_string()}})),
            )
                .into_response();
        }
    };

    let receipt = &result.receipt;
    let receipt_hash = &receipt.receipt_hash;

    // 4. Persist the receipt (routed or refused — both are valid audit artifacts).
    let receipt_json = serde_json::to_string_pretty(receipt).unwrap();
    if let Err(e) = state.receipt_store.store(receipt_hash, &receipt_json) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "internal_error", "message": e.to_string()}})),
        )
            .into_response();
    }

    // 5. Return outcome-appropriate response.
    if receipt.outcome == "refused" {
        let code = receipt
            .refusal_code
            .as_deref()
            .unwrap_or("no_eligible_candidates");
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": {"code": "routing_refused",
                "message": format!("routing refused: {code}")}})),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(json!({
            "case_id": case_id,
            "receipt_hash": receipt_hash,
            "selected_candidate_id": receipt.selected_candidate_id,
        })),
    )
        .into_response()
}

// ── Route history endpoint ────────────────────────────────────────────────────

/// GET /routes
///
/// Returns all stored routing receipts as a route history list, sorted by
/// timestamp descending (receipt_hash ascending as tiebreaker).
///
/// Success (200): `{"routes": [{case_id, receipt_hash, selected_candidate_id, timestamp}, ...]}`
/// Internal (500): `{"error": {"code": "internal_error"|"receipt_parse_error", "message": "..."}}`
pub async fn list_routes(State(store): State<Arc<ReceiptStore>>) -> impl IntoResponse {
    match store.list_routes() {
        Ok(entries) => {
            let routes: Vec<Value> = entries
                .into_iter()
                .map(|e| {
                    json!({
                        "case_id": e.case_id,
                        "receipt_hash": e.receipt_hash,
                        "selected_candidate_id": e.selected_candidate_id,
                        "timestamp": e.timestamp,
                    })
                })
                .collect();
            (StatusCode::OK, Json(json!({"routes": routes}))).into_response()
        }
        Err(ReceiptStoreError::ParseError(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "receipt_parse_error", "message": msg}})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "internal_error", "message": e.to_string()}})),
        )
            .into_response(),
    }
}
