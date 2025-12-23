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
    #[serde(rename = "start_bandwidth_test")]
    StartBandwidthTest {
        test_id: String,
        node_id: String,
    },
}

// WebSocket message types from server to client
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "latency_update")]
    LatencyUpdate {
        connections: Vec<Connection>,
    },
    #[serde(rename = "error")]
    Error {
        request_id: Option<String>,
        error_code: String,
        message: String,
    },
    #[serde(rename = "bandwidth_test_progress")]
    BandwidthTestProgress {
        test_id: String,
        progress_percent: u8,
        phase: String,
        bytes_transferred: u64,
    },
    #[serde(rename = "bandwidth_test_result")]
    BandwidthTestResult {
        test_id: String,
        target_node_id: String,
        upload_mbps: f64,
        download_mbps: f64,
        duration_secs: u64,
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

async fn handle_client_message(msg: ClientMessage, state: &Arc<AppState>) {
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
        ClientMessage::StartBandwidthTest { test_id, node_id } => {
            tracing::info!("Bandwidth test request: {} to {}", test_id, node_id);

            if let Some(bandwidth_service) = &state.bandwidth_service {
                // Get node info to find IP address
                if let Some(node) = state.peer_registry.get_node(&node_id).await {
                    if let Some(&ip_addr) = node.addresses.first() {
                        let target_addr = std::net::SocketAddr::new(ip_addr, 8081);
                        let bandwidth_service = bandwidth_service.clone();
                        let state_clone = state.clone();
                        let test_id_clone = test_id.clone();

                        // Run bandwidth test in background
                        tokio::spawn(async move {
                            match bandwidth_service.run_bandwidth_test(test_id_clone.clone(), target_addr).await {
                                Ok(result) => {
                                    state_clone.send_update(ServerMessage::BandwidthTestResult {
                                        test_id: result.test_id,
                                        target_node_id: result.target_node_id,
                                        upload_mbps: result.upload_mbps,
                                        download_mbps: result.download_mbps,
                                        duration_secs: result.duration_secs,
                                    });
                                }
                                Err(e) => {
                                    state_clone.send_update(ServerMessage::Error {
                                        request_id: Some(test_id_clone),
                                        error_code: "BandwidthTestFailed".to_string(),
                                        message: e,
                                    });
                                }
                            }
                        });
                    }
                }
            }
        }
    }
}
