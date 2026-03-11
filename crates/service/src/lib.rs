mod case_store;
mod handlers;
mod receipt_store;

use std::sync::Arc;

use axum::{routing, Router};

pub use case_store::CaseStore;
pub use receipt_store::{ReceiptStore, ReceiptStoreError};

/// Shared application state for endpoints that require both stores.
pub struct AppState {
    pub case_store: Arc<CaseStore>,
    pub receipt_store: Arc<ReceiptStore>,
}

/// Build the service router with default storage paths
/// (`data/cases/` and `data/receipts/`).
pub fn app() -> Router {
    app_with_stores(
        Arc::new(CaseStore::new("data/cases")),
        Arc::new(ReceiptStore::new("data/receipts")),
    )
}

/// Build the service router with an explicit case store and a default receipt
/// store (`data/receipts/`).
///
/// Preserved for backward compatibility with tests that only exercise the case
/// intake endpoints and do not need receipt store control.
pub fn app_with_store(store: Arc<CaseStore>) -> Router {
    app_with_stores(store, Arc::new(ReceiptStore::new("data/receipts")))
}

/// Build the service router with explicit case and receipt stores.
///
/// Use this in tests to inject temporary directories for both stores.
pub fn app_with_stores(case_store: Arc<CaseStore>, receipt_store: Arc<ReceiptStore>) -> Router {
    // Case intake routes: State<Arc<CaseStore>>.
    let case_routes = Router::new()
        .route("/cases", routing::post(handlers::post_case))
        .route("/cases", routing::get(handlers::list_cases))
        .route("/cases/:case_id", routing::get(handlers::get_case))
        .with_state(case_store.clone());

    // Case routing endpoint: State<Arc<AppState>> (needs both stores).
    let route_endpoint = Router::new()
        .route(
            "/cases/:case_id/route",
            routing::post(handlers::route_stored_case),
        )
        .with_state(Arc::new(AppState {
            case_store,
            receipt_store,
        }));

    Router::new()
        .route("/route-case", routing::post(handlers::route_case))
        .route(
            "/route-case-from-registry",
            routing::post(handlers::route_case_from_registry),
        )
        .route("/verify-receipt", routing::post(handlers::verify_receipt))
        .route(
            "/protocol-manifest",
            routing::get(handlers::protocol_manifest),
        )
        // Pilot endpoints
        .route("/health", routing::get(handlers::health))
        .route("/version", routing::get(handlers::version))
        .route("/route", routing::post(handlers::pilot_route))
        .route("/verify", routing::post(handlers::pilot_verify))
        // Case intake
        .merge(case_routes)
        // Stored-case routing
        .merge(route_endpoint)
}
