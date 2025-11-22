mod routes;
mod api;

use axum::{
    routing::get,
    Router,
};
use tower_http::services::ServeDir;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Initialize logging
    println!("Network Route Visualizer starting...");

    // Build application router
    let app = Router::new()
        .route("/", get(hello_world))
        .nest_service("/static", ServeDir::new("src/web/static"))
        .merge(api::rest::create_api_router());

    // Configure server address
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Server listening on http://{}", addr);
    println!("API endpoints:");
    println!("  GET  http://{}/api/routing-table", addr);
    println!("  POST http://{}/api/trace-route", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello_world() -> &'static str {
    "Network Route Visualizer is running!\n\
     \n\
     API Endpoints:\n\
     - GET  /api/routing-table - Get current routing table\n\
     - POST /api/trace-route   - Trace route to destination\n\
     \n\
     Web UI: /static/index.html (coming soon)"
}
