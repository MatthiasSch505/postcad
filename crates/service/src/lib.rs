mod handlers;

use axum::{routing, Router};

/// Build the service router. Exported for in-process integration tests.
pub fn app() -> Router {
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
}
