use axum::{http::StatusCode, response::IntoResponse, Json};
use postcad_cli::{
    build_manifest, route_case_from_policy_json, route_case_from_registry_json,
    verify_receipt_from_policy_json, PROTOCOL_VERSION,
};
use serde_json::{json, Value};

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
