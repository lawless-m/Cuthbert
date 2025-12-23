// Discovery module - node discovery and peer management

pub mod bandwidth;
pub mod broadcast;
pub mod gossip;
pub mod ping;
pub mod traceroute;
pub mod vpn_scan;
pub mod wireguard;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub hostname: String,
    pub addresses: Vec<IpAddr>,
    pub port: u16,
    pub status: NodeStatus,
    pub last_seen: String,
    pub discovered_via: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Online,
    Offline,
    Unreachable,
}

#[derive(Debug, Clone)]
pub struct PeerRegistry {
    nodes: Arc<RwLock<HashMap<String, NodeInfo>>>,
    local_node_id: String,
}

impl PeerRegistry {
    pub fn new() -> Self {
        let local_node_id = Uuid::new_v4().to_string();
        PeerRegistry {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            local_node_id,
        }
    }

    pub fn local_node_id(&self) -> &str {
        &self.local_node_id
    }

    pub async fn add_node(&self, node: NodeInfo) {
        let mut nodes = self.nodes.write().await;
        nodes.insert(node.id.clone(), node);
    }

    pub async fn remove_node(&self, node_id: &str) {
        let mut nodes = self.nodes.write().await;
        nodes.remove(node_id);
    }

    pub async fn get_node(&self, node_id: &str) -> Option<NodeInfo> {
        let nodes = self.nodes.read().await;
        nodes.get(node_id).cloned()
    }

    pub async fn get_all_nodes(&self) -> Vec<NodeInfo> {
        let nodes = self.nodes.read().await;
        nodes.values().cloned().collect()
    }

    pub async fn cleanup_stale_nodes(&self, timeout_seconds: u64) {
        let mut nodes = self.nodes.write().await;
        let now = chrono::Utc::now();

        nodes.retain(|_, node| {
            if let Ok(last_seen) = chrono::DateTime::parse_from_rfc3339(&node.last_seen) {
                let elapsed = now.signed_duration_since(last_seen.with_timezone(&chrono::Utc));
                elapsed.num_seconds() < timeout_seconds as i64
            } else {
                false
            }
        });
    }
}
