// REST API endpoints

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::routes::{parser, lookup::RouteEngine, RoutingTable};
use crate::discovery::{NodeInfo, PeerRegistry};
use super::{TraceRouteRequest, TraceRouteResponse, ErrorResponse};
use super::websocket::ServerMessage;

#[derive(Clone)]
pub struct AppState {
    pub peer_registry: Arc<PeerRegistry>,
    pub broadcast_tx: broadcast::Sender<ServerMessage>,
}

impl AppState {
    pub fn new(peer_registry: Arc<PeerRegistry>) -> Self {
        let (tx, _) = broadcast::channel(100);
        AppState {
            peer_registry,
            broadcast_tx: tx,
        }
    }

    pub fn subscribe_to_updates(&self) -> broadcast::Receiver<ServerMessage> {
        self.broadcast_tx.subscribe()
    }

    pub fn send_update(&self, msg: ServerMessage) {
        let _ = self.broadcast_tx.send(msg);
    }
}

pub fn create_api_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(hello_world))
        .route("/ws", get(super::websocket::websocket_handler))
        .route("/api/routing-table", get(get_routing_table))
        .route("/api/trace-route", post(trace_route))
        .route("/api/nodes", get(get_nodes))
        .route("/api/nodes/:node_id", get(get_node))
        .route("/api/nodes/:node_id/routing-table", get(get_remote_routing_table))
        .with_state(state)
}

async fn hello_world(State(_state): State<Arc<AppState>>) -> &'static str {
    "Network Route Visualizer is running!\n\
     \n\
     API Endpoints:\n\
     - GET  /api/routing-table      - Get current routing table\n\
     - POST /api/trace-route        - Trace route to destination\n\
     - GET  /api/nodes              - List discovered nodes\n\
     - GET  /api/nodes/{id}         - Get node details\n\
     - WS   /ws                     - WebSocket for real-time updates\n\
     \n\
     Web UI: /static/index.html"
}

async fn get_routing_table(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<RoutingTable>, (StatusCode, Json<ErrorResponse>)> {
    match parser::get_routing_table() {
        Ok(table) => Ok(Json(table)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "FailedToGetRoutingTable".to_string(),
                message: e,
            }),
        )),
    }
}

async fn trace_route(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<TraceRouteRequest>,
) -> Result<Json<TraceRouteResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Resolve destination to IP
    let ip = match request.destination.parse() {
        Ok(ip) => ip,
        Err(_) => {
            // Try DNS resolution
            match tokio::net::lookup_host(format!("{}:0", request.destination))
                .await
                .ok()
                .and_then(|mut addrs| addrs.next())
            {
                Some(addr) => addr.ip(),
                None => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "InvalidDestination".to_string(),
                            message: format!("Could not resolve destination: {}", request.destination),
                        }),
                    ));
                }
            }
        }
    };

    // Get routing table
    let routing_table = match parser::get_routing_table() {
        Ok(table) => table,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "FailedToGetRoutingTable".to_string(),
                    message: e,
                }),
            ));
        }
    };

    // Perform route lookup
    let engine = RouteEngine::new(&routing_table);
    let matched_route = engine.lookup(ip).cloned();

    if matched_route.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "NoRouteToHost".to_string(),
                message: format!("No route found to {}", ip),
            }),
        ));
    }

    Ok(Json(TraceRouteResponse {
        destination: request.destination,
        resolved_ip: ip.to_string(),
        matched_route,
    }))
}

#[derive(serde::Serialize)]
struct NodesResponse {
    nodes: Vec<NodeInfo>,
    local_node_id: String,
}

async fn get_nodes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<NodesResponse>, (StatusCode, Json<ErrorResponse>)> {
    let nodes = state.peer_registry.get_all_nodes().await;
    let local_node_id = state.peer_registry.local_node_id().to_string();

    Ok(Json(NodesResponse {
        nodes,
        local_node_id,
    }))
}

async fn get_node(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(node_id): axum::extract::Path<String>,
) -> Result<Json<NodeInfo>, (StatusCode, Json<ErrorResponse>)> {
    match state.peer_registry.get_node(&node_id).await {
        Some(node) => Ok(Json(node)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "NodeNotFound".to_string(),
                message: format!("Node {} not found", node_id),
            }),
        )),
    }
}

async fn get_remote_routing_table(
    State(_state): State<Arc<AppState>>,
    axum::extract::Path(_node_id): axum::extract::Path<String>,
) -> Result<Json<RoutingTable>, (StatusCode, Json<ErrorResponse>)> {
    // In a real implementation, this would fetch the routing table from the remote node
    // For now, return an error
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ErrorResponse {
            error: "NotImplemented".to_string(),
            message: "Remote routing table fetching not yet implemented".to_string(),
        }),
    ))
}
