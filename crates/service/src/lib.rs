mod handlers;

use axum::{Router, routing};

/// Build the service router. Exported for in-process integration tests.
pub fn app() -> Router {
    Router::new()
        .route("/route-case", routing::post(handlers::route_case))
        .route("/route-case-from-registry", routing::post(handlers::route_case_from_registry))
        .route("/verify-receipt", routing::post(handlers::verify_receipt))
        .route("/protocol-manifest", routing::get(handlers::protocol_manifest))
}
