// UDP multicast broadcast for node discovery

use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::UdpSocket;
use std::sync::Arc;

const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(239, 255, 42, 1);
const MULTICAST_PORT: u16 = 5678;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DiscoveryMessage {
    #[serde(rename = "announce")]
    Announce {
        node_id: String,
        hostname: String,
        addresses: Vec<IpAddr>,
        port: u16,
        timestamp: String,
        version: String,
        known_peers: Vec<String>,
    },
    #[serde(rename = "goodbye")]
    Goodbye {
        node_id: String,
        reason: String,
    },
}

pub struct DiscoveryService {
    node_id: String,
    hostname: String,
    port: u16,
}

impl DiscoveryService {
    pub fn new(node_id: String, hostname: String, port: u16) -> Self {
        DiscoveryService {
            node_id,
            hostname,
            port,
        }
    }

    pub async fn start_announcing(&self, peer_registry: Arc<super::PeerRegistry>) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.set_broadcast(true)?;

        let multicast_addr = SocketAddr::new(IpAddr::V4(MULTICAST_ADDR), MULTICAST_PORT);

        let node_id = self.node_id.clone();
        let hostname = self.hostname.clone();
        let port = self.port;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

            loop {
                interval.tick().await;

                // Get local IP addresses
                let addresses = get_local_addresses();

                // Get known peers
                let known_peers: Vec<String> = peer_registry
                    .get_all_nodes()
                    .await
                    .iter()
                    .map(|n| n.id.clone())
                    .collect();

                let announce = DiscoveryMessage::Announce {
                    node_id: node_id.clone(),
                    hostname: hostname.clone(),
                    addresses,
                    port,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    known_peers,
                };

                if let Ok(json) = serde_json::to_string(&announce) {
                    let _ = socket.send_to(json.as_bytes(), multicast_addr).await;
                    tracing::debug!("Sent discovery announcement");
                }
            }
        });

        Ok(())
    }

    pub async fn start_listening(&self, peer_registry: Arc<super::PeerRegistry>) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            MULTICAST_PORT,
        )).await?;

        // Join multicast group
        socket.join_multicast_v4(MULTICAST_ADDR, Ipv4Addr::UNSPECIFIED)?;

        let local_node_id = peer_registry.local_node_id().to_string();

        tokio::spawn(async move {
            let mut buf = vec![0u8; 4096];

            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((len, _addr)) => {
                        if let Ok(json_str) = std::str::from_utf8(&buf[..len]) {
                            if let Ok(msg) = serde_json::from_str::<DiscoveryMessage>(json_str) {
                                match msg {
                                    DiscoveryMessage::Announce {
                                        node_id,
                                        hostname,
                                        addresses,
                                        port,
                                        timestamp,
                                        ..
                                    } => {
                                        // Ignore announcements from self
                                        if node_id != local_node_id {
                                            let node = super::NodeInfo {
                                                id: node_id.clone(),
                                                hostname,
                                                addresses,
                                                port,
                                                status: super::NodeStatus::Online,
                                                last_seen: timestamp,
                                                discovered_via: "broadcast".to_string(),
                                            };
                                            peer_registry.add_node(node).await;
                                            tracing::info!("Discovered node: {}", node_id);
                                        }
                                    }
                                    DiscoveryMessage::Goodbye { node_id, .. } => {
                                        peer_registry.remove_node(&node_id).await;
                                        tracing::info!("Node left: {}", node_id);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error receiving discovery message: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn send_goodbye(&self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.set_broadcast(true)?;

        let multicast_addr = SocketAddr::new(IpAddr::V4(MULTICAST_ADDR), MULTICAST_PORT);

        let goodbye = DiscoveryMessage::Goodbye {
            node_id: self.node_id.clone(),
            reason: "shutdown".to_string(),
        };

        if let Ok(json) = serde_json::to_string(&goodbye) {
            socket.send_to(json.as_bytes(), multicast_addr).await?;
        }

        Ok(())
    }
}

fn get_local_addresses() -> Vec<IpAddr> {
    use std::net::UdpSocket as StdUdpSocket;

    // Try to get local IP by connecting to a public address
    let mut addresses = Vec::new();

    if let Ok(socket) = StdUdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                addresses.push(addr.ip());
            }
        }
    }

    // Fallback to localhost if we couldn't determine the real IP
    if addresses.is_empty() {
        addresses.push(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    }

    addresses
}
