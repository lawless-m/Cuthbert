// Comprehensive error types for the Network Route Visualizer

use thiserror::Error;

/// Main error type for the application
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to parse routing table: {0}")]
    RoutingTableParse(String),

    #[error("Failed to execute command: {0}")]
    CommandExecution(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Discovery error: {0}")]
    Discovery(String),

    #[error("Traceroute error: {0}")]
    Traceroute(String),

    #[error("Invalid destination: {0}")]
    InvalidDestination(String),

    #[error("No route to host: {0}")]
    NoRouteToHost(String),

    #[error("Bandwidth test error: {0}")]
    BandwidthTest(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("JSON serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type alias using AppError
pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    /// Convert error to user-friendly message
    pub fn user_message(&self) -> String {
        match self {
            AppError::RoutingTableParse(_) => {
                "Failed to read routing table. Please ensure you have proper permissions.".to_string()
            }
            AppError::CommandExecution(cmd) => {
                format!("Failed to execute command. Make sure {} is installed.", cmd)
            }
            AppError::Network(_) => {
                "Network error occurred. Please check your connection.".to_string()
            }
            AppError::Io(_) => {
                "File system error. Check permissions and disk space.".to_string()
            }
            AppError::Config(_) => {
                "Configuration error. Check your config file or command-line arguments.".to_string()
            }
            AppError::Discovery(_) => {
                "Node discovery failed. Check network settings and firewall rules.".to_string()
            }
            AppError::Traceroute(msg) => {
                format!("Traceroute failed: {}. Ensure traceroute/tracert is installed and you have proper permissions.", msg)
            }
            AppError::InvalidDestination(_) => {
                "Invalid destination. Please provide a valid IP address or domain name.".to_string()
            }
            AppError::NoRouteToHost(_) => {
                "No route found to destination. Check your routing table and network connectivity.".to_string()
            }
            AppError::BandwidthTest(_) => {
                "Bandwidth test failed. Ensure both nodes are reachable and ports are open.".to_string()
            }
            AppError::WebSocket(_) => {
                "WebSocket connection error. Try refreshing the page.".to_string()
            }
            AppError::Serialization(_) => {
                "Data format error. This might be a bug, please report it.".to_string()
            }
            AppError::Unknown(_) => {
                "An unexpected error occurred. Please try again.".to_string()
            }
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AppError::Network(_) | AppError::Discovery(_) | AppError::WebSocket(_)
        )
    }
}

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// Calculate delay for a given attempt number
    pub fn delay_for_attempt(&self, attempt: u32) -> u64 {
        let delay = (self.initial_delay_ms as f64)
            * self.backoff_multiplier.powi(attempt as i32);
        delay.min(self.max_delay_ms as f64) as u64
    }
}

/// Retry a fallible operation with exponential backoff
pub async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
    config: &RetryConfig,
    operation_name: &str,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    tracing::info!(
                        "{} succeeded after {} attempt(s)",
                        operation_name,
                        attempt + 1
                    );
                }
                return Ok(result);
            }
            Err(e) => {
                attempt += 1;

                if attempt >= config.max_attempts {
                    tracing::error!(
                        "{} failed after {} attempts: {}",
                        operation_name,
                        attempt,
                        e
                    );
                    return Err(e);
                }

                let delay = config.delay_for_attempt(attempt - 1);
                tracing::warn!(
                    "{} failed (attempt {}/{}): {}. Retrying in {}ms...",
                    operation_name,
                    attempt,
                    config.max_attempts,
                    e,
                    delay
                );

                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_delay() {
        let config = RetryConfig::default();
        assert_eq!(config.delay_for_attempt(0), 100);
        assert_eq!(config.delay_for_attempt(1), 200);
        assert_eq!(config.delay_for_attempt(2), 400);
    }

    #[test]
    fn test_error_retryable() {
        assert!(AppError::Network("test".to_string()).is_retryable());
        assert!(AppError::Discovery("test".to_string()).is_retryable());
        assert!(!AppError::InvalidDestination("test".to_string()).is_retryable());
    }
}
