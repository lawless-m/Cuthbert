// Bandwidth testing service

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

use crate::api::rest::AppState;
use crate::api::websocket::ServerMessage;

const BANDWIDTH_TEST_PORT: u16 = 8081;
const BANDWIDTH_TEST_DURATION_SECS: u64 = 10;
const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks

#[derive(Debug, Clone, serde::Serialize)]
pub struct BandwidthTestResult {
    pub test_id: String,
    pub target_node_id: String,
    pub upload_mbps: f64,
    pub download_mbps: f64,
    pub duration_secs: u64,
    pub timestamp: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BandwidthTestProgress {
    pub test_id: String,
    pub progress_percent: u8,
    pub current_phase: String, // "upload", "download", "complete"
    pub bytes_transferred: u64,
}

pub struct BandwidthService {
    state: Arc<AppState>,
    active_tests: Arc<RwLock<std::collections::HashMap<String, BandwidthTestProgress>>>,
}

impl BandwidthService {
    pub fn new(state: Arc<AppState>) -> Self {
        BandwidthService {
            state,
            active_tests: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub async fn start_server(&self) {
        let state = self.state.clone();

        tokio::spawn(async move {
            let addr = SocketAddr::from(([0, 0, 0, 0], BANDWIDTH_TEST_PORT));
            let listener = match TcpListener::bind(addr).await {
                Ok(l) => l,
                Err(e) => {
                    tracing::error!("Failed to bind bandwidth test server: {}", e);
                    return;
                }
            };

            tracing::info!("Bandwidth test server listening on port {}", BANDWIDTH_TEST_PORT);

            loop {
                match listener.accept().await {
                    Ok((socket, addr)) => {
                        tracing::info!("Bandwidth test connection from {}", addr);
                        tokio::spawn(Self::handle_test_connection(socket, state.clone()));
                    }
                    Err(e) => {
                        tracing::error!("Failed to accept bandwidth test connection: {}", e);
                    }
                }
            }
        });
    }

    async fn handle_test_connection(mut socket: TcpStream, _state: Arc<AppState>) {
        // Read test header (1 byte: 0 = receive mode, 1 = send mode)
        let mut mode_byte = [0u8; 1];
        if socket.read_exact(&mut mode_byte).await.is_err() {
            return;
        }

        match mode_byte[0] {
            0 => {
                // Receive mode: client will send data to us
                let mut buffer = vec![0u8; CHUNK_SIZE];
                let mut total_bytes = 0u64;
                let start = Instant::now();

                while start.elapsed().as_secs() < BANDWIDTH_TEST_DURATION_SECS {
                    match socket.read(&mut buffer).await {
                        Ok(0) => break, // Connection closed
                        Ok(n) => total_bytes += n as u64,
                        Err(_) => break,
                    }
                }

                tracing::debug!("Received {} bytes in bandwidth test", total_bytes);
            }
            1 => {
                // Send mode: we send data to client
                let buffer = vec![0u8; CHUNK_SIZE];
                let start = Instant::now();

                while start.elapsed().as_secs() < BANDWIDTH_TEST_DURATION_SECS {
                    if socket.write_all(&buffer).await.is_err() {
                        break;
                    }
                }

                let _ = socket.shutdown().await;
            }
            _ => {
                tracing::warn!("Invalid bandwidth test mode: {}", mode_byte[0]);
            }
        }
    }

    pub async fn run_bandwidth_test(
        &self,
        test_id: String,
        target_addr: SocketAddr,
    ) -> Result<BandwidthTestResult, String> {
        let target_node_id = format!("{}", target_addr.ip());

        // Update progress: starting
        self.update_progress(test_id.clone(), 0, "initializing".to_string(), 0)
            .await;

        // Test upload speed (we send data)
        let upload_mbps = match self.test_upload(test_id.clone(), target_addr).await {
            Ok(mbps) => mbps,
            Err(e) => {
                self.remove_test(&test_id).await;
                return Err(format!("Upload test failed: {}", e));
            }
        };

        // Test download speed (we receive data)
        let download_mbps = match self.test_download(test_id.clone(), target_addr).await {
            Ok(mbps) => mbps,
            Err(e) => {
                self.remove_test(&test_id).await;
                return Err(format!("Download test failed: {}", e));
            }
        };

        // Update progress: complete
        self.update_progress(test_id.clone(), 100, "complete".to_string(), 0)
            .await;
        self.remove_test(&test_id).await;

        Ok(BandwidthTestResult {
            test_id,
            target_node_id,
            upload_mbps,
            download_mbps,
            duration_secs: BANDWIDTH_TEST_DURATION_SECS * 2,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn test_upload(
        &self,
        test_id: String,
        target_addr: SocketAddr,
    ) -> Result<f64, String> {
        self.update_progress(test_id.clone(), 10, "upload".to_string(), 0)
            .await;

        let mut socket = TcpStream::connect(target_addr)
            .await
            .map_err(|e| format!("Failed to connect: {}", e))?;

        // Send mode header (1 = we send, server receives)
        socket
            .write_all(&[0u8])
            .await
            .map_err(|e| format!("Failed to send mode: {}", e))?;

        let buffer = vec![0u8; CHUNK_SIZE];
        let mut total_bytes = 0u64;
        let start = Instant::now();

        while start.elapsed().as_secs() < BANDWIDTH_TEST_DURATION_SECS {
            if socket.write_all(&buffer).await.is_err() {
                break;
            }
            total_bytes += CHUNK_SIZE as u64;

            // Update progress
            let progress = 10 + ((start.elapsed().as_secs() * 40) / BANDWIDTH_TEST_DURATION_SECS) as u8;
            self.update_progress(test_id.clone(), progress, "upload".to_string(), total_bytes)
                .await;
        }

        let _ = socket.shutdown().await;
        let elapsed = start.elapsed().as_secs_f64();
        let mbps = (total_bytes as f64 * 8.0) / (elapsed * 1_000_000.0);

        Ok(mbps)
    }

    async fn test_download(
        &self,
        test_id: String,
        target_addr: SocketAddr,
    ) -> Result<f64, String> {
        self.update_progress(test_id.clone(), 50, "download".to_string(), 0)
            .await;

        let mut socket = TcpStream::connect(target_addr)
            .await
            .map_err(|e| format!("Failed to connect: {}", e))?;

        // Receive mode header (1 = server sends, we receive)
        socket
            .write_all(&[1u8])
            .await
            .map_err(|e| format!("Failed to send mode: {}", e))?;

        let mut buffer = vec![0u8; CHUNK_SIZE];
        let mut total_bytes = 0u64;
        let start = Instant::now();

        while start.elapsed().as_secs() < BANDWIDTH_TEST_DURATION_SECS {
            match socket.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => total_bytes += n as u64,
                Err(_) => break,
            }

            // Update progress
            let progress = 50 + ((start.elapsed().as_secs() * 40) / BANDWIDTH_TEST_DURATION_SECS) as u8;
            self.update_progress(test_id.clone(), progress, "download".to_string(), total_bytes)
                .await;
        }

        let elapsed = start.elapsed().as_secs_f64();
        let mbps = (total_bytes as f64 * 8.0) / (elapsed * 1_000_000.0);

        Ok(mbps)
    }

    async fn update_progress(&self, test_id: String, progress: u8, phase: String, bytes: u64) {
        let progress_data = BandwidthTestProgress {
            test_id: test_id.clone(),
            progress_percent: progress,
            current_phase: phase.clone(),
            bytes_transferred: bytes,
        };

        let mut tests = self.active_tests.write().await;
        tests.insert(test_id.clone(), progress_data.clone());

        // Send WebSocket update
        self.state.send_update(ServerMessage::BandwidthTestProgress {
            test_id,
            progress_percent: progress,
            phase,
            bytes_transferred: bytes,
        });
    }

    async fn remove_test(&self, test_id: &str) {
        let mut tests = self.active_tests.write().await;
        tests.remove(test_id);
    }
}
