mod routes;
mod api;
mod discovery;

use tower_http::services::ServeDir;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber;

use api::rest::AppState;
use discovery::{PeerRegistry, broadcast::DiscoveryService, gossip::GossipService, ping::PingService, bandwidth::BandwidthService};

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();
    tracing::info!("Network Route Visualizer starting...");

    // Get hostname
    let hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown".to_string());

    // Initialize peer registry
    let peer_registry = Arc::new(PeerRegistry::new());
    let local_node_id = peer_registry.local_node_id().to_string();
    tracing::info!("Local node ID: {}", local_node_id);

    // Initialize application state (without bandwidth service first)
    let state = Arc::new(AppState::new(peer_registry.clone()));

    // Configure server address
    let port = 8080;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    // Start discovery service
    let discovery = DiscoveryService::new(local_node_id.clone(), hostname.clone(), port);

    if let Err(e) = discovery.start_announcing(peer_registry.clone()).await {
        tracing::error!("Failed to start discovery announcements: {}", e);
    } else {
        tracing::info!("Discovery announcements started");
    }

    if let Err(e) = discovery.start_listening(peer_registry.clone()).await {
        tracing::error!("Failed to start discovery listener: {}", e);
    } else {
        tracing::info!("Discovery listener started on multicast");
    }

    // Start gossip service for cleanup
    let gossip = GossipService::new(peer_registry.clone());
    gossip.start_cleanup_task().await;

    // Start ping service for latency monitoring
    let ping_service = PingService::new(peer_registry.clone(), state.clone());
    ping_service.start_ping_task().await;

    // Start bandwidth test server
    let bandwidth_service = Arc::new(BandwidthService::new(state.clone()));
    bandwidth_service.start_server().await;

    // Update state with bandwidth service
    let state = Arc::new(state.as_ref().clone().with_bandwidth_service(bandwidth_service.clone()));

    // Build application router
    let app = api::rest::create_api_router(state.clone())
        .nest_service("/static", ServeDir::new("src/web/static"))
        .into_make_service();

    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("API endpoints:");
    tracing::info!("  GET  http://{}/api/routing-table", addr);
    tracing::info!("  POST http://{}/api/trace-route", addr);
    tracing::info!("  GET  http://{}/api/nodes", addr);
    tracing::info!("  WS   ws://{}/ws", addr);
    tracing::info!("Web UI: http://{}/static/index.html", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .await
        .unwrap();
}

