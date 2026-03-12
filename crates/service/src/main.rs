use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = postcad_service::app();
    // Railway injects PORT; fall back to POSTCAD_ADDR, then the local default.
    let addr = if let Ok(port) = std::env::var("PORT") {
        format!("0.0.0.0:{port}")
    } else {
        std::env::var("POSTCAD_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string())
    };
    let listener = TcpListener::bind(&addr).await.unwrap();
    eprintln!("postcad-service listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}
