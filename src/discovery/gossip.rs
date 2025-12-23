// Gossip protocol for sharing peer information

use std::sync::Arc;
use tokio::time::{interval, Duration};

use super::PeerRegistry;

pub struct GossipService {
    peer_registry: Arc<PeerRegistry>,
}

impl GossipService {
    pub fn new(peer_registry: Arc<PeerRegistry>) -> Self {
        GossipService { peer_registry }
    }

    pub async fn start_cleanup_task(&self) {
        let peer_registry = self.peer_registry.clone();

        tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(60));

            loop {
                cleanup_interval.tick().await;

                // Clean up nodes that haven't been seen in 90 seconds
                peer_registry.cleanup_stale_nodes(90).await;
                tracing::debug!("Cleaned up stale nodes");
            }
        });
    }
}
