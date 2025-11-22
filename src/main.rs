mod routes;
mod api;
mod discovery;
mod config;
mod error;

use tower_http::services::ServeDir;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use api::rest::AppState;
use discovery::{PeerRegistry, broadcast::DiscoveryService, gossip::GossipService, ping::PingService, bandwidth::BandwidthService};
use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from all sources
    let config = Config::load()?;

    // Initialize logging with configured level
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_file(true)
                .with_line_number(true)
        )
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    tracing_subscriber::EnvFilter::new(config.log_level.as_str())
                })
        )
        .init();

    tracing::info!("Network Route Visualizer v{} starting...", env!("CARGO_PKG_VERSION"));
    tracing::info!("Log level: {}", config.log_level);

    // Get hostname
    let hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| {
            tracing::warn!("Could not determine hostname, using 'unknown'");
            "unknown".to_string()
        });
    tracing::info!("Hostname: {}", hostname);

    // Initialize peer registry
    let peer_registry = Arc::new(PeerRegistry::new());
    let local_node_id = peer_registry.local_node_id().to_string();
    tracing::info!("Local node ID: {}", local_node_id);

    // Initialize application state (without bandwidth service first)
    let state = Arc::new(AppState::new(peer_registry.clone()));

    // Configure server address
    let addr = SocketAddr::from((
        config.bind_address.parse::<std::net::IpAddr>()
            .unwrap_or_else(|_| {
                tracing::warn!("Invalid bind address, using 127.0.0.1");
                std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))
            }),
        config.port
    ));
    tracing::info!("Server will bind to: {}", addr);

    // Start discovery service (if enabled)
    if config.discovery_enabled {
        tracing::info!("Starting node discovery service...");
        let discovery = DiscoveryService::new(local_node_id.clone(), hostname.clone(), config.port);

        if let Err(e) = discovery.start_announcing(peer_registry.clone()).await {
            tracing::error!("Failed to start discovery announcements: {}", e);
            tracing::warn!("Continuing without discovery announcements");
        } else {
            tracing::info!("Discovery announcements started (interval: {}s)", config.discovery_interval);
        }

        if let Err(e) = discovery.start_listening(peer_registry.clone()).await {
            tracing::error!("Failed to start discovery listener: {}", e);
            tracing::warn!("Continuing without discovery listener");
        } else {
            tracing::info!("Discovery listener started on multicast {}:{}",
                config.multicast_group, config.multicast_port);
        }

        // Start gossip service for cleanup
        let gossip = GossipService::new(peer_registry.clone());
        gossip.start_cleanup_task().await;
        tracing::info!("Gossip service started (peer timeout: {}s)", config.peer_timeout);
    } else {
        tracing::info!("Node discovery disabled by configuration");
    }

    // Start ping service for latency monitoring (if enabled)
    if config.ping_enabled {
        tracing::info!("Starting ping service...");
        let ping_service = PingService::new(peer_registry.clone(), state.clone());
        ping_service.start_ping_task().await;
        tracing::info!("Ping service started (interval: {}s)", config.ping_interval);
    } else {
        tracing::info!("Ping service disabled by configuration");
    }

    // Start bandwidth test server
    tracing::info!("Starting bandwidth test server on port {}...", config.bandwidth_port);
    let bandwidth_service = Arc::new(BandwidthService::new(state.clone()));
    bandwidth_service.start_server().await;
    tracing::info!("Bandwidth test server started");

    // Update state with bandwidth service
    let state = Arc::new(state.as_ref().clone().with_bandwidth_service(bandwidth_service.clone()));

    // Build application router
    let app = api::rest::create_api_router(state.clone())
        .nest_service("/static", ServeDir::new("src/web/static"))
        .into_make_service();

    tracing::info!("═══════════════════════════════════════════════════════════");
    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("═══════════════════════════════════════════════════════════");
    tracing::info!("API endpoints:");
    tracing::info!("  GET  http://{}/api/routing-table", addr);
    tracing::info!("  POST http://{}/api/trace-route", addr);
    tracing::info!("  POST http://{}/api/traceroute", addr);
    tracing::info!("  GET  http://{}/api/nodes", addr);
    tracing::info!("  WS   ws://{}/ws", addr);
    tracing::info!("═══════════════════════════════════════════════════════════");
    tracing::info!("Web UI: http://{}/static/index.html", addr);
    tracing::info!("═══════════════════════════════════════════════════════════");

    // Auto-open browser if enabled
    if config.auto_open_browser {
        let url = format!("http://{}/static/index.html", addr);
        tracing::info!("Auto-opening browser to {}", url);

        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();

        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(&url).spawn();

        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("cmd").args(&["/C", "start", &url]).spawn();
    }

    // Start server with proper error handling
    tracing::info!("Starting HTTP server...");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| {
            tracing::error!("Failed to bind to address {}: {}", addr, e);
            anyhow::anyhow!("Failed to bind to {}: {}. Is another instance running?", addr, e)
        })?;

    tracing::info!("✓ Server started successfully!");
    tracing::info!("Press Ctrl+C to stop");

    axum::serve(listener, app)
        .await
        .map_err(|e| {
            tracing::error!("Server error: {}", e);
            anyhow::anyhow!("Server error: {}", e)
        })?;

    Ok(())
}

