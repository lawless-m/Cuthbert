// WebSocket handler for real-time updates

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::routes::{parser, lookup::RouteEngine};
use super::rest::AppState;

// WebSocket message types from client to server
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "trace_route")]
    TraceRoute {
        request_id: String,
        destination: String,
    },
    #[serde(rename = "subscribe")]
    Subscribe {
        topics: Vec<String>,
    },
    #[serde(rename = "get_remote_routing_table")]
    GetRemoteRoutingTable {
        request_id: String,
        node_id: String,
    },
}

// WebSocket message types from server to client
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "node_discovered")]
    NodeDiscovered {
        node: crate::discovery::NodeInfo,
    },
    #[serde(rename = "node_status_changed")]
    NodeStatusChanged {
        node_id: String,
        status: String,
        reason: Option<String>,
    },
    #[serde(rename = "latency_update")]
    LatencyUpdate {
        connections: Vec<Connection>,
    },
    #[serde(rename = "routing_table_changed")]
    RoutingTableChanged {
        node_id: String,
        routes_added: Vec<crate::routes::Route>,
        routes_removed: Vec<String>,
    },
    #[serde(rename = "trace_route_result")]
    TraceRouteResult {
        request_id: String,
        destination: String,
        resolved_ip: String,
        matched_route: Option<crate::routes::Route>,
    },
    #[serde(rename = "error")]
    Error {
        request_id: Option<String>,
        error_code: String,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct Connection {
    pub from: String,
    pub to: String,
    pub latency_ms: u32,
    pub timestamp: String,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcast channel for node updates
    let mut rx = state.subscribe_to_updates();

    // Spawn task to forward broadcast messages to WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap_or_default();
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    handle_client_message(client_msg, &state).await;
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
}

async fn handle_client_message(msg: ClientMessage, _state: &Arc<AppState>) {
    match msg {
        ClientMessage::TraceRoute { request_id, destination } => {
            // Handle trace route request
            tracing::info!("Trace route request: {} to {}", request_id, destination);
            // Implementation will send result back via broadcast
        }
        ClientMessage::Subscribe { topics } => {
            tracing::info!("Client subscribed to topics: {:?}", topics);
        }
        ClientMessage::GetRemoteRoutingTable { request_id, node_id } => {
            tracing::info!("Get remote routing table: {} for {}", request_id, node_id);
        }
    }
}
