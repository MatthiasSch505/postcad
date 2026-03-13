use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use postcad_cli::{
    build_manifest, normalize_pilot_case_json, route_case_from_policy_json,
    route_case_from_registry_json, verify_receipt_from_policy_json, CaseInput, PROTOCOL_VERSION,
};
use serde_json::{json, Value};

use crate::case_store::{CaseStore, CaseStoreError, StoreOutcome};
use crate::demo::DEMO_HTML;
use crate::dispatch_commitment::{DispatchCommitmentError, DispatchCommitmentStore, DispatchRecord};
use crate::receipt_store::{ReceiptStore, ReceiptStoreError};
use crate::reviewer::REVIEWER_HTML;
use crate::ui::OPERATOR_UI_HTML;
use crate::{AppState, DispatchState, DispatchVerifyState};

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

/// POST /pilot/route-normalized
///
/// Accepts a minimal normalized pilot case input and routes it through the
/// existing registry-backed routing kernel.
///
/// Request:
/// ```json
/// {
///   "pilot_case":     {"case_id"?: "...", "restoration_type": "crown",
///                      "material": "zirconia", "jurisdiction": "DE"},
///   "registry_snapshot": [...],
///   "routing_config": {"jurisdiction": "DE", "routing_policy": "..."}
/// }
/// ```
/// Success (200): `{"receipt": {...}, "derived_policy": {...}}`
/// Error (422): `{"error": {"code": "...", "message": "..."}}`
///
/// The `pilot_case` is normalized via [`normalize_pilot_case_json`] before
/// being forwarded to [`route_case_from_registry_json`]. All routing kernel
/// semantics, receipt commitments, and determinism guarantees are unchanged.
pub async fn pilot_route_normalized(Json(body): Json<Value>) -> impl IntoResponse {
    let pilot_case = &body["pilot_case"];
    let registry_snapshot = &body["registry_snapshot"];
    let routing_config = &body["routing_config"];

    if pilot_case.is_null() || registry_snapshot.is_null() || routing_config.is_null() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": {"code": "parse_error", "message": "request must contain 'pilot_case', 'registry_snapshot', and 'routing_config' fields"}})),
        )
            .into_response();
    }

    let pilot_case_json = serde_json::to_string(pilot_case).unwrap();
    let registry_json = serde_json::to_string(registry_snapshot).unwrap();
    let config_json = serde_json::to_string(routing_config).unwrap();

    let case_json = match normalize_pilot_case_json(&pilot_case_json) {
        Ok(j) => j,
        Err(e) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"error": {"code": e.code(), "message": e.to_string()}})),
            )
                .into_response();
        }
    };

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

// ── Reviewer endpoints ────────────────────────────────────────────────────────

/// GET /reviewer
///
/// Serves the minimal reviewer shell — one page, two actions (Route + Verify),
/// auto-loads pilot fixtures, calls real kernel endpoints.
pub async fn reviewer_page() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        REVIEWER_HTML,
    )
}

/// GET /pilot-fixtures
///
/// Reads `examples/pilot/{case.json, registry_snapshot.json, config.json}` and
/// returns them as a single JSON object ready to POST to `POST /route`.
///
/// Returns 503 if any file is missing (service must be started from repo root).
pub async fn pilot_fixtures() -> impl IntoResponse {
    let paths = [
        ("case", "examples/pilot/case.json"),
        ("registry_snapshot", "examples/pilot/registry_snapshot.json"),
        ("routing_config", "examples/pilot/config.json"),
    ];

    let mut out = serde_json::Map::new();
    for (key, path) in &paths {
        match std::fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str::<Value>(&content) {
                Ok(v) => { out.insert((*key).to_string(), v); }
                Err(e) => return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": {"code": "fixture_parse_error",
                        "message": format!("{path}: {e}")}})),
                ).into_response(),
            },
            Err(e) => return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"error": {"code": "fixture_not_found",
                    "message": format!("Could not read {path}: {e}. Start the service from the repo root.")}})),
            ).into_response(),
        }
    }
    (StatusCode::OK, Json(Value::Object(out))).into_response()
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

    // 4b. Persist the derived policy so the dispatch verification gate can
    //     replay the verification without requiring the caller to re-supply it.
    if let Err(e) = state
        .policy_store
        .store(receipt_hash, &result.derived_policy_json)
    {
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

// ── Dispatch endpoint ─────────────────────────────────────────────────────────

/// POST /dispatch/:receipt_hash
///
/// Reads an existing stored receipt, creates a dispatch record, and writes it
/// to `data/dispatch/{receipt_hash}.json`.
///
/// Success (200):       `{"receipt_hash": "...", "dispatched": true}`
/// Not found (404):     `{"error": {"code": "receipt_not_found", "message": "..."}}`
/// Conflict (409):      `{"error": {"code": "dispatch_already_exists", "message": "..."}}`
/// Internal (500):      `{"error": {"code": "internal_error", "message": "..."}}`
pub async fn dispatch_receipt(
    State(state): State<Arc<DispatchState>>,
    Path(receipt_hash): Path<String>,
) -> impl IntoResponse {
    // 1. Read the stored receipt.
    let receipt = match state.receipt_store.read(&receipt_hash) {
        Ok(Some(v)) => v,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": {"code": "receipt_not_found",
                    "message": format!("receipt '{receipt_hash}' not found")}})),
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

    // 2. Guard against duplicate dispatch.
    if state.dispatch_store.exists(&receipt_hash) {
        return (
            StatusCode::CONFLICT,
            Json(json!({"error": {"code": "dispatch_already_exists",
                "message": format!("dispatch already exists for receipt '{receipt_hash}'")}})),
        )
            .into_response();
    }

    // 3. Extract required fields.
    let case_id = match receipt["routing_input"]["case_id"].as_str() {
        Some(v) => v.to_string(),
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": {"code": "internal_error",
                    "message": "receipt missing routing_input.case_id"}})),
            )
                .into_response();
        }
    };
    let manufacturer = receipt["selected_candidate_id"]
        .as_str()
        .map(|s| s.to_string());

    // 4. Build the dispatch record.
    let timestamp = Utc::now().to_rfc3339();
    let record = json!({
        "receipt_hash": receipt_hash,
        "case_id": case_id,
        "manufacturer": manufacturer,
        "status": "dispatched",
        "timestamp": timestamp,
    });
    let record_json = serde_json::to_string_pretty(&record).unwrap();

    // 5. Persist the dispatch record.
    match state.dispatch_store.store(&receipt_hash, &record_json) {
        Ok(()) => {}
        Err(crate::dispatch_store::DispatchStoreError::AlreadyExists) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({"error": {"code": "dispatch_already_exists",
                    "message": format!("dispatch already exists for receipt '{receipt_hash}'")}})),
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
    }

    // 6. Return deterministic response.
    (
        StatusCode::OK,
        Json(json!({"receipt_hash": receipt_hash, "dispatched": true})),
    )
        .into_response()
}

// ── Dispatch verification endpoint ───────────────────────────────────────────

/// POST /dispatch/:receipt_hash/verify
///
/// Verifies a dispatched routing artifact by:
/// 1. Confirming a dispatch record exists.
/// 2. Loading the stored receipt.
/// 3. Loading the stored derived policy bundle (written at routing time).
/// 4. Calling the existing verification path with the receipt, the case
///    extracted from the receipt's `routing_input`, and the stored policy.
/// 5. Persisting and returning the deterministic result.
///
/// Success (200):     `{"receipt_hash": "...", "result": "VERIFIED"}`
///                    or `{"receipt_hash": "...", "result": "INVALID"}`
/// Not found (404):   `{"error": {"code": "dispatch_not_found", ...}}`
///                    `{"error": {"code": "receipt_not_found", ...}}`
/// Input missing (422): `{"error": {"code": "verification_input_missing", ...}}`
/// Internal (500):    `{"error": {"code": "internal_error", ...}}`
pub async fn dispatch_verify(
    State(state): State<Arc<DispatchVerifyState>>,
    Path(receipt_hash): Path<String>,
) -> impl IntoResponse {
    // 1. Confirm a dispatch record exists.
    if !state.dispatch_store.exists(&receipt_hash) {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": {"code": "dispatch_not_found",
                "message": format!("no dispatch record for receipt '{receipt_hash}'")}})),
        )
            .into_response();
    }

    // 2. Load the stored receipt.
    let receipt_value = match state.receipt_store.read(&receipt_hash) {
        Ok(Some(v)) => v,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": {"code": "receipt_not_found",
                    "message": format!("receipt '{receipt_hash}' not found")}})),
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

    // 3. Load the stored derived policy bundle.
    let policy_json = match state.policy_store.read(&receipt_hash) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"error": {"code": "verification_input_missing",
                    "message": format!("no policy bundle stored for receipt '{receipt_hash}'")}})),
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

    // 4. Extract the case from receipt.routing_input (same fields as CaseInput).
    let case_input = &receipt_value["routing_input"];
    if case_input.is_null() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "internal_error",
                "message": "receipt missing routing_input"}})),
        )
            .into_response();
    }
    let case_json = serde_json::to_string(case_input).unwrap();
    let receipt_json = serde_json::to_string(&receipt_value).unwrap();

    // 5. Reuse the existing verification path.
    let result_str =
        match postcad_cli::verify_receipt_from_policy_json(&receipt_json, &case_json, &policy_json)
        {
            Ok(()) => "VERIFIED",
            Err(_) => "INVALID",
        };

    // 6. Persist the verification result.
    let timestamp = Utc::now().to_rfc3339();
    let result_record = json!({
        "receipt_hash": receipt_hash,
        "result": result_str,
        "timestamp": timestamp,
    });
    let result_json = serde_json::to_string_pretty(&result_record).unwrap();
    if let Err(e) = state.verification_store.store(&receipt_hash, &result_json) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "internal_error", "message": e.to_string()}})),
        )
            .into_response();
    }

    // 7. Return deterministic response.
    (
        StatusCode::OK,
        Json(json!({"receipt_hash": receipt_hash, "result": result_str})),
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

// ── Receipts REST endpoints ───────────────────────────────────────────────────

/// GET /receipts
///
/// Returns the sorted list of all stored receipt hashes.
/// Success (200): `{"receipts": ["...", ...]}`
pub async fn list_receipts(State(store): State<Arc<ReceiptStore>>) -> impl IntoResponse {
    match store.list_hashes() {
        Ok(hashes) => (StatusCode::OK, Json(json!({"receipts": hashes}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "internal_error", "message": e.to_string()}})),
        )
            .into_response(),
    }
}

/// GET /receipts/:receipt_hash
///
/// Returns the stored receipt JSON.
/// Success (200): the receipt JSON object
/// Not found (404): `{"error": {"code": "receipt_not_found", "message": "..."}}`
pub async fn get_receipt(
    State(store): State<Arc<ReceiptStore>>,
    Path(receipt_hash): Path<String>,
) -> impl IntoResponse {
    match store.read(&receipt_hash) {
        Ok(Some(v)) => (StatusCode::OK, Json(v)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": {"code": "receipt_not_found",
                "message": format!("receipt '{receipt_hash}' not found")}})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "internal_error", "message": e.to_string()}})),
        )
            .into_response(),
    }
}

// ── Operator UI ───────────────────────────────────────────────────────────────

/// GET /demo
///
/// Serves the embedded reviewer demo page.
pub async fn demo_page() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        DEMO_HTML,
    )
}

/// GET /
///
/// Serves the embedded single-page operator UI.
pub async fn operator_ui() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        OPERATOR_UI_HTML,
    )
}

// ── Dispatch Commitment Layer ─────────────────────────────────────────────────

/// POST /dispatch/create
///
/// Creates a dispatch commitment from a **verification-passed** routing receipt.
/// The receipt is verified inline; if verification fails the dispatch is rejected.
///
/// Request: `{"receipt": {...}, "case": {...}, "policy": {...}}`
/// Success (200):   `{"dispatch_id": "...", "receipt_hash": "...", "case_id": "...",
///                    "selected_candidate_id": "..." | null, "verification_passed": true, "status": "draft"}`
/// Verify fail (422): `{"error": {"code": "...", "message": "..."}}`
/// Already exists (409): `{"error": {"code": "receipt_already_dispatched", ...}}`
pub async fn create_dispatch_commitment(
    State(store): State<std::sync::Arc<DispatchCommitmentStore>>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let receipt = &body["receipt"];
    let case = &body["case"];
    let policy = &body["policy"];

    if receipt.is_null() || case.is_null() || policy.is_null() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": {"code": "parse_error",
                "message": "request must contain 'receipt', 'case', and 'policy' fields"}})),
        )
            .into_response();
    }

    let receipt_json = serde_json::to_string(receipt).unwrap();
    let case_json = serde_json::to_string(case).unwrap();
    let policy_json = serde_json::to_string(policy).unwrap();

    // Gate: verify the receipt before allowing dispatch creation.
    if let Err(f) =
        postcad_cli::verify_receipt_from_policy_json(&receipt_json, &case_json, &policy_json)
    {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": {"code": f.code, "message": f.message}})),
        )
            .into_response();
    }

    // Extract receipt fields needed for the commitment record.
    let receipt_hash = match receipt["receipt_hash"].as_str() {
        Some(h) => h.to_string(),
        None => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({"error": {"code": "parse_error",
                    "message": "receipt missing receipt_hash"}})),
            )
                .into_response();
        }
    };
    let case_id = receipt["routing_input"]["case_id"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let selected_candidate_id = receipt["selected_candidate_id"]
        .as_str()
        .map(|s| s.to_string());

    let dispatch_id = uuid::Uuid::new_v4().to_string();
    let record = DispatchRecord {
        dispatch_id: dispatch_id.clone(),
        case_id: case_id.clone(),
        selected_candidate_id: selected_candidate_id.clone(),
        receipt_hash: receipt_hash.clone(),
        verification_passed: true,
        status: "draft".to_string(),
        approved_by: None,
        approved_at: None,
        created_at: Utc::now().to_rfc3339(),
        manufacturer_payload_json: None,
    };

    match store.create(&record) {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({
                "dispatch_id": dispatch_id,
                "receipt_hash": receipt_hash,
                "case_id": case_id,
                "selected_candidate_id": selected_candidate_id,
                "verification_passed": true,
                "status": "draft",
            })),
        )
            .into_response(),
        Err(DispatchCommitmentError::ReceiptAlreadyDispatched) => (
            StatusCode::CONFLICT,
            Json(json!({"error": {"code": "receipt_already_dispatched",
                "message": format!("a dispatch commitment already exists for receipt '{receipt_hash}'")}})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "internal_error", "message": e.to_string()}})),
        )
            .into_response(),
    }
}

/// POST /dispatch/:dispatch_id/approve
///
/// Transitions a draft dispatch commitment to `approved`.
/// Approved dispatches are immutable — a second approve call returns 409.
///
/// Request: `{"approved_by": "..."}` (optional; defaults to `"system"`)
/// Success (200): full dispatch record JSON
/// Not draft (409): `{"error": {"code": "dispatch_not_draft", ...}}`
/// Not found (404): `{"error": {"code": "dispatch_not_found", ...}}`
pub async fn approve_dispatch_commitment(
    State(store): State<std::sync::Arc<DispatchCommitmentStore>>,
    Path(dispatch_id): Path<String>,
    body: Option<Json<Value>>,
) -> impl IntoResponse {
    let approved_by = body
        .as_ref()
        .and_then(|b| b["approved_by"].as_str())
        .unwrap_or("system")
        .to_string();

    match store.approve(&dispatch_id, &approved_by) {
        Ok(record) => (StatusCode::OK, Json(serde_json::to_value(&record).unwrap())).into_response(),
        Err(DispatchCommitmentError::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": {"code": "dispatch_not_found",
                "message": format!("dispatch '{dispatch_id}' not found")}})),
        )
            .into_response(),
        Err(DispatchCommitmentError::NotDraft) => (
            StatusCode::CONFLICT,
            Json(json!({"error": {"code": "dispatch_not_draft",
                "message": format!("dispatch '{dispatch_id}' is not in draft state")}})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "internal_error", "message": e.to_string()}})),
        )
            .into_response(),
    }
}

/// GET /dispatch/:dispatch_id/export
///
/// Returns a deterministic, exportable dispatch payload.
/// Transitions `approved` → `exported`; idempotent if already `exported`.
/// Rejected for `draft` dispatches (must be approved first).
///
/// Success (200): `{"dispatch_id": "...", "receipt_hash": "...", ...}`
/// Not approved (422): `{"error": {"code": "dispatch_not_approved", ...}}`
/// Not found (404): `{"error": {"code": "dispatch_not_found", ...}}`
pub async fn export_dispatch_commitment(
    State(store): State<std::sync::Arc<DispatchCommitmentStore>>,
    Path(dispatch_id): Path<String>,
) -> impl IntoResponse {
    match store.mark_exported(&dispatch_id) {
        Ok(record) => {
            // Build deterministic export payload: canonical field order, all fields present.
            let payload = json!({
                "approved_at": record.approved_at,
                "approved_by": record.approved_by,
                "case_id": record.case_id,
                "created_at": record.created_at,
                "dispatch_id": record.dispatch_id,
                "manufacturer_payload_json": record.manufacturer_payload_json,
                "receipt_hash": record.receipt_hash,
                "selected_candidate_id": record.selected_candidate_id,
                "status": record.status,
                "verification_passed": record.verification_passed,
            });
            (StatusCode::OK, Json(payload)).into_response()
        }
        Err(DispatchCommitmentError::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": {"code": "dispatch_not_found",
                "message": format!("dispatch '{dispatch_id}' not found")}})),
        )
            .into_response(),
        Err(DispatchCommitmentError::NotApproved) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": {"code": "dispatch_not_approved",
                "message": format!("dispatch '{dispatch_id}' must be approved before export")}})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "internal_error", "message": e.to_string()}})),
        )
            .into_response(),
    }
}
