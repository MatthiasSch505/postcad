use axum::{Json, http::StatusCode, response::IntoResponse};
use postcad_cli::{build_manifest, route_case_from_policy_json, verify_receipt_from_policy_json};
use serde_json::{Value, json};

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

/// GET /protocol-manifest
///
/// Returns the static protocol manifest. No inputs required.
pub async fn protocol_manifest() -> impl IntoResponse {
    Json(serde_json::to_value(build_manifest()).unwrap())
}
