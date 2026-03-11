mod case_store;
mod dispatch_store;
mod handlers;
mod policy_store;
mod receipt_store;
pub mod ui;
mod verification_store;

use std::sync::Arc;

use axum::{routing, Router};

pub use case_store::CaseStore;
pub use dispatch_store::DispatchStore;
pub use policy_store::PolicyStore;
pub use receipt_store::{ReceiptStore, ReceiptStoreError};
pub use verification_store::VerificationStore;

/// Shared application state for the stored-case routing endpoint.
///
/// Requires the case store, receipt store, and policy store so that routing
/// can persist both the receipt and the derived policy bundle together.
pub struct AppState {
    pub case_store: Arc<CaseStore>,
    pub receipt_store: Arc<ReceiptStore>,
    pub policy_store: Arc<PolicyStore>,
}

/// Shared application state for the dispatch endpoint.
pub struct DispatchState {
    pub receipt_store: Arc<ReceiptStore>,
    pub dispatch_store: Arc<DispatchStore>,
}

/// Shared application state for the dispatch verification endpoint.
pub struct DispatchVerifyState {
    pub dispatch_store: Arc<DispatchStore>,
    pub receipt_store: Arc<ReceiptStore>,
    pub policy_store: Arc<PolicyStore>,
    pub verification_store: Arc<VerificationStore>,
}

/// Build the service router with default storage paths.
pub fn app() -> Router {
    app_with_all_stores(
        Arc::new(CaseStore::new("data/cases")),
        Arc::new(ReceiptStore::new("data/receipts")),
        Arc::new(DispatchStore::new("data/dispatch")),
        Arc::new(PolicyStore::new("data/policies")),
        Arc::new(VerificationStore::new("data/verification")),
    )
}

/// Build the service router with an explicit case store and default stores for
/// everything else.
///
/// Preserved for backward compatibility with tests that only exercise the case
/// intake endpoints.
pub fn app_with_store(store: Arc<CaseStore>) -> Router {
    app_with_all_stores(
        store,
        Arc::new(ReceiptStore::new("data/receipts")),
        Arc::new(DispatchStore::new("data/dispatch")),
        Arc::new(PolicyStore::new("data/policies")),
        Arc::new(VerificationStore::new("data/verification")),
    )
}

/// Build the service router with explicit case and receipt stores.
///
/// All other stores default to their canonical `data/` paths.
/// Use [`app_with_all_stores`] in tests that exercise dispatch or verification.
pub fn app_with_stores(case_store: Arc<CaseStore>, receipt_store: Arc<ReceiptStore>) -> Router {
    app_with_all_stores(
        case_store,
        receipt_store,
        Arc::new(DispatchStore::new("data/dispatch")),
        Arc::new(PolicyStore::new("data/policies")),
        Arc::new(VerificationStore::new("data/verification")),
    )
}

/// Build the service router with explicit control over all five storage layers.
///
/// Use this in tests that need to inject temporary directories for dispatch,
/// policy, or verification storage.
pub fn app_with_all_stores(
    case_store: Arc<CaseStore>,
    receipt_store: Arc<ReceiptStore>,
    dispatch_store: Arc<DispatchStore>,
    policy_store: Arc<PolicyStore>,
    verification_store: Arc<VerificationStore>,
) -> Router {
    // Case intake routes: State<Arc<CaseStore>>.
    let case_routes = Router::new()
        .route("/cases", routing::post(handlers::post_case))
        .route("/cases", routing::get(handlers::list_cases))
        .route("/cases/:case_id", routing::get(handlers::get_case))
        .with_state(case_store.clone());

    // Receipt REST + route history endpoints: State<Arc<ReceiptStore>>.
    let receipt_endpoints = Router::new()
        .route("/receipts", routing::get(handlers::list_receipts))
        .route(
            "/receipts/:receipt_hash",
            routing::get(handlers::get_receipt),
        )
        .route("/routes", routing::get(handlers::list_routes))
        .with_state(receipt_store.clone());

    // Case routing endpoint: State<Arc<AppState>> (needs case, receipt, and policy stores).
    let route_endpoint = Router::new()
        .route(
            "/cases/:case_id/route",
            routing::post(handlers::route_stored_case),
        )
        .with_state(Arc::new(AppState {
            case_store,
            receipt_store: receipt_store.clone(),
            policy_store: policy_store.clone(),
        }));

    // Dispatch endpoint: State<Arc<DispatchState>>.
    let dispatch_endpoint = Router::new()
        .route(
            "/dispatch/:receipt_hash",
            routing::post(handlers::dispatch_receipt),
        )
        .with_state(Arc::new(DispatchState {
            receipt_store: receipt_store.clone(),
            dispatch_store: dispatch_store.clone(),
        }));

    // Dispatch verification endpoint: State<Arc<DispatchVerifyState>>.
    let dispatch_verify_endpoint = Router::new()
        .route(
            "/dispatch/:receipt_hash/verify",
            routing::post(handlers::dispatch_verify),
        )
        .with_state(Arc::new(DispatchVerifyState {
            dispatch_store,
            receipt_store,
            policy_store,
            verification_store,
        }));

    Router::new()
        .route("/", routing::get(handlers::operator_ui))
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
        // Receipts + route history
        .merge(receipt_endpoints)
        // Stored-case routing
        .merge(route_endpoint)
        // Dispatch
        .merge(dispatch_endpoint)
        // Dispatch verification
        .merge(dispatch_verify_endpoint)
}
