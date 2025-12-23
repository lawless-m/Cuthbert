// Ping service for latency monitoring

use std::collections::VecDeque;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use surge_ping::{Client, Config, PingIdentifier, PingSequence};
use tokio::sync::RwLock;
use tokio::time::interval;

use super::PeerRegistry;
use crate::api::rest::AppState;
use crate::api::websocket::{Connection, ServerMessage};

const MAX_LATENCY_SAMPLES: usize = 100;
const PING_INTERVAL_SECS: u64 = 60;
const PING_TIMEOUT_SECS: u64 = 5;

#[derive(Debug, Clone, serde::Serialize)]
pub struct LatencyData {
    pub node_id: String,
    pub address: IpAddr,
    pub latency_ms: Option<f64>,
    pub timestamp: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LatencyHistory {
    pub node_id: String,
    pub samples: VecDeque<LatencyData>,
    pub avg_latency_ms: Option<f64>,
    pub min_latency_ms: Option<f64>,
    pub max_latency_ms: Option<f64>,
}

impl LatencyHistory {
    pub fn new(node_id: String) -> Self {
        LatencyHistory {
            node_id,
            samples: VecDeque::with_capacity(MAX_LATENCY_SAMPLES),
            avg_latency_ms: None,
            min_latency_ms: None,
            max_latency_ms: None,
        }
    }

    pub fn add_sample(&mut self, data: LatencyData) {
        if self.samples.len() >= MAX_LATENCY_SAMPLES {
            self.samples.pop_front();
        }
        self.samples.push_back(data);
        self.update_stats();
    }

    fn update_stats(&mut self) {
        let latencies: Vec<f64> = self.samples.iter().filter_map(|s| s.latency_ms).collect();

        if latencies.is_empty() {
            self.avg_latency_ms = None;
            self.min_latency_ms = None;
            self.max_latency_ms = None;
            return;
        }

        let sum: f64 = latencies.iter().sum();
        self.avg_latency_ms = Some(sum / latencies.len() as f64);
        self.min_latency_ms = latencies
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap());
        self.max_latency_ms = latencies
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap());
    }
}

pub struct PingService {
    peer_registry: Arc<PeerRegistry>,
    latency_histories: Arc<RwLock<std::collections::HashMap<String, LatencyHistory>>>,
    state: Arc<AppState>,
}

impl PingService {
    pub fn new(peer_registry: Arc<PeerRegistry>, state: Arc<AppState>) -> Self {
        PingService {
            peer_registry,
            latency_histories: Arc::new(RwLock::new(std::collections::HashMap::new())),
            state,
        }
    }

    pub async fn start_ping_task(&self) {
        let peer_registry = self.peer_registry.clone();
        let latency_histories = self.latency_histories.clone();
        let state = self.state.clone();

        tokio::spawn(async move {
            let mut ping_interval = interval(Duration::from_secs(PING_INTERVAL_SECS));

            loop {
                ping_interval.tick().await;

                let nodes = peer_registry.get_all_nodes().await;
                let local_id = peer_registry.local_node_id().to_string();

                for node in nodes {
                    // Don't ping ourselves
                    if node.id == local_id {
                        continue;
                    }

                    // Ping the first address
                    if let Some(&address) = node.addresses.first() {
                        let node_id = node.id.clone();
                        let latency_histories_clone = latency_histories.clone();
                        let state_clone = state.clone();

                        // Spawn a separate task for each ping to avoid blocking
                        tokio::spawn(async move {
                            let latency = Self::ping_address(address).await;
                            let timestamp = chrono::Utc::now().to_rfc3339();

                            let data = LatencyData {
                                node_id: node_id.clone(),
                                address,
                                latency_ms: latency,
                                timestamp,
                            };

                            // Update history
                            let mut histories = latency_histories_clone.write().await;
                            let history = histories
                                .entry(node_id.clone())
                                .or_insert_with(|| LatencyHistory::new(node_id.clone()));
                            history.add_sample(data.clone());

                            // Send WebSocket update
                            if let Some(latency_val) = latency {
                                let connections = vec![Connection {
                                    from: "local".to_string(),
                                    to: node_id,
                                    latency_ms: latency_val.round() as u32,
                                    timestamp: data.timestamp,
                                }];

                                state_clone
                                    .send_update(ServerMessage::LatencyUpdate { connections });
                            }

                            tracing::debug!(
                                "Pinged {} ({}): {:?} ms",
                                address,
                                latency
                                    .map(|l| format!("{:.2}", l))
                                    .unwrap_or_else(|| "timeout".to_string()),
                                latency
                            );
                        });
                    }
                }
            }
        });

        tracing::info!("Ping service started (interval: {}s)", PING_INTERVAL_SECS);
    }

    async fn ping_address(address: IpAddr) -> Option<f64> {
        let config = Config::default();

        let client = match Client::new(&config) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to create ping client: {}", e);
                return None;
            }
        };

        let payload = [0; 56];
        let mut pinger = client.pinger(address, PingIdentifier(rand::random())).await;

        // Use tokio timeout for the ping
        match tokio::time::timeout(
            Duration::from_secs(PING_TIMEOUT_SECS),
            pinger.ping(PingSequence(0), &payload),
        )
        .await
        {
            Ok(Ok((_, duration))) => Some(duration.as_secs_f64() * 1000.0),
            Ok(Err(e)) => {
                tracing::debug!("Ping failed for {}: {}", address, e);
                None
            }
            Err(_) => {
                tracing::debug!("Ping timeout for {}", address);
                None
            }
        }
    }
}
