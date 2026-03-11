mod case_store;
mod handlers;

use std::sync::Arc;

use axum::{routing, Router};

pub use case_store::CaseStore;

/// Build the service router with the default case store (`data/cases/`).
/// Exported for in-process integration tests that do not need store control.
pub fn app() -> Router {
    app_with_store(Arc::new(CaseStore::new("data/cases")))
}

/// Build the service router with an explicit case store.
/// Use this in tests to inject a temporary directory.
pub fn app_with_store(store: Arc<CaseStore>) -> Router {
    // Case intake routes use State<Arc<CaseStore>> and are merged as a
    // separate sub-router after calling .with_state() so that the main
    // router's stateless handlers are unaffected.
    let case_routes = Router::new()
        .route("/cases", routing::post(handlers::post_case))
        .route("/cases", routing::get(handlers::list_cases))
        .route("/cases/:case_id", routing::get(handlers::get_case))
        .with_state(store);

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
}
