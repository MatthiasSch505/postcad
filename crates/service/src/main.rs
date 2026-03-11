use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = postcad_service::app();
    let addr = std::env::var("POSTCAD_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let listener = TcpListener::bind(&addr).await.unwrap();
    eprintln!("postcad-service listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}
