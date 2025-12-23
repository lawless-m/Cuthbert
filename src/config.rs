// Configuration management for Network Route Visualizer
// Supports CLI arguments, config file (TOML), and environment variables

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::Level;

/// Network Route Visualizer - Visualize routing tables and network topology in 3D
#[derive(Parser, Debug, Clone)]
#[command(name = "network-route-visualizer")]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Web server port
    #[arg(short, long, default_value = "8080", env = "NRV_PORT")]
    pub port: u16,

    /// Don't auto-open browser
    #[arg(long, env = "NRV_NO_BROWSER")]
    pub no_browser: bool,

    /// Logging level (error, warn, info, debug, trace)
    #[arg(short, long, default_value = "info", env = "RUST_LOG")]
    pub log_level: String,

    /// Path to configuration file
    #[arg(short, long, env = "NRV_CONFIG")]
    pub config: Option<PathBuf>,

    /// Discovery interval in seconds
    #[arg(long, default_value = "30", env = "NRV_DISCOVERY_INTERVAL")]
    pub discovery_interval: u64,

    /// Peer timeout in seconds
    #[arg(long, default_value = "90", env = "NRV_PEER_TIMEOUT")]
    pub peer_timeout: u64,

    /// Ping interval in seconds
    #[arg(long, default_value = "60", env = "NRV_PING_INTERVAL")]
    pub ping_interval: u64,

    /// Bandwidth test duration in seconds
    #[arg(long, default_value = "10", env = "NRV_BANDWIDTH_DURATION")]
    pub bandwidth_duration: u64,

    /// Bandwidth test port
    #[arg(long, default_value = "9090", env = "NRV_BANDWIDTH_PORT")]
    pub bandwidth_port: u16,

    /// Disable node discovery
    #[arg(long, env = "NRV_NO_DISCOVERY")]
    pub no_discovery: bool,

    /// Disable automatic ping
    #[arg(long, env = "NRV_NO_PING")]
    pub no_ping: bool,
}

/// Configuration file structure (TOML format)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigFile {
    /// Server settings
    #[serde(default)]
    pub server: ServerConfig,

    /// Discovery settings
    #[serde(default)]
    pub discovery: DiscoveryConfig,

    /// Testing settings
    #[serde(default)]
    pub testing: TestingConfig,

    /// Logging settings
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Web server port
    #[serde(default = "default_port")]
    pub port: u16,

    /// Auto-open browser on startup
    #[serde(default = "default_true")]
    pub auto_open_browser: bool,

    /// Bind address (default: 127.0.0.1)
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Enable node discovery
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Discovery announcement interval in seconds
    #[serde(default = "default_discovery_interval")]
    pub interval_seconds: u64,

    /// Peer timeout in seconds
    #[serde(default = "default_peer_timeout")]
    pub timeout_seconds: u64,

    /// Multicast group address
    #[serde(default = "default_multicast_group")]
    pub multicast_group: String,

    /// Multicast port
    #[serde(default = "default_multicast_port")]
    pub multicast_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestingConfig {
    /// Enable automatic ping
    #[serde(default = "default_true")]
    pub ping_enabled: bool,

    /// Ping interval in seconds
    #[serde(default = "default_ping_interval")]
    pub ping_interval_seconds: u64,

    /// Bandwidth test duration in seconds
    #[serde(default = "default_bandwidth_duration")]
    pub bandwidth_test_duration: u64,

    /// Bandwidth test port
    #[serde(default = "default_bandwidth_port")]
    pub bandwidth_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (error, warn, info, debug, trace)
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log to file
    #[serde(default)]
    pub file: Option<PathBuf>,
}

// Default value functions
fn default_port() -> u16 {
    8080
}
fn default_true() -> bool {
    true
}
fn default_bind_address() -> String {
    "127.0.0.1".to_string()
}
fn default_discovery_interval() -> u64 {
    30
}
fn default_peer_timeout() -> u64 {
    90
}
fn default_multicast_group() -> String {
    "239.255.42.1".to_string()
}
fn default_multicast_port() -> u16 {
    5678
}
fn default_ping_interval() -> u64 {
    60
}
fn default_bandwidth_duration() -> u64 {
    10
}
fn default_bandwidth_port() -> u16 {
    9090
}
fn default_log_level() -> String {
    "info".to_string()
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            port: default_port(),
            auto_open_browser: default_true(),
            bind_address: default_bind_address(),
        }
    }
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        DiscoveryConfig {
            enabled: default_true(),
            interval_seconds: default_discovery_interval(),
            timeout_seconds: default_peer_timeout(),
            multicast_group: default_multicast_group(),
            multicast_port: default_multicast_port(),
        }
    }
}

impl Default for TestingConfig {
    fn default() -> Self {
        TestingConfig {
            ping_enabled: default_true(),
            ping_interval_seconds: default_ping_interval(),
            bandwidth_test_duration: default_bandwidth_duration(),
            bandwidth_port: default_bandwidth_port(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            level: default_log_level(),
            file: None,
        }
    }
}

/// Merged configuration from all sources
#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub auto_open_browser: bool,
    pub bind_address: String,
    pub log_level: Level,
    pub discovery_enabled: bool,
    pub discovery_interval: u64,
    pub peer_timeout: u64,
    pub multicast_group: String,
    pub multicast_port: u16,
    pub ping_enabled: bool,
    pub ping_interval: u64,
    pub bandwidth_port: u16,
}

impl Config {
    /// Load configuration from all sources (CLI args, config file, defaults)
    /// Priority: CLI args > Config file > Environment variables > Defaults
    pub fn load() -> anyhow::Result<Self> {
        let cli_args = CliArgs::parse();

        // Load config file if specified
        let config_file = if let Some(config_path) = &cli_args.config {
            tracing::info!("Loading configuration from: {}", config_path.display());
            let config_content = std::fs::read_to_string(config_path)?;
            toml::from_str::<ConfigFile>(&config_content)?
        } else {
            // Try loading from default locations
            let default_paths = vec![
                PathBuf::from("config.toml"),
                PathBuf::from("network-route-visualizer.toml"),
            ];

            let mut loaded_config = None;
            for path in default_paths {
                if path.exists() {
                    tracing::info!("Loading configuration from: {}", path.display());
                    let config_content = std::fs::read_to_string(&path)?;
                    loaded_config = Some(toml::from_str::<ConfigFile>(&config_content)?);
                    break;
                }
            }

            loaded_config.unwrap_or_default()
        };

        // Merge configuration (CLI args override config file)
        let port = cli_args.port;
        let auto_open_browser = !cli_args.no_browser && config_file.server.auto_open_browser;
        let bind_address = config_file.server.bind_address;

        let log_level = parse_log_level(&cli_args.log_level)?;

        let discovery_enabled = !cli_args.no_discovery && config_file.discovery.enabled;
        let discovery_interval = cli_args.discovery_interval;
        let peer_timeout = cli_args.peer_timeout;
        let multicast_group = config_file.discovery.multicast_group;
        let multicast_port = config_file.discovery.multicast_port;

        let ping_enabled = !cli_args.no_ping && config_file.testing.ping_enabled;
        let ping_interval = cli_args.ping_interval;
        let bandwidth_port = cli_args.bandwidth_port;

        Ok(Config {
            port,
            auto_open_browser,
            bind_address,
            log_level,
            discovery_enabled,
            discovery_interval,
            peer_timeout,
            multicast_group,
            multicast_port,
            ping_enabled,
            ping_interval,
            bandwidth_port,
        })
    }
}

fn parse_log_level(level_str: &str) -> anyhow::Result<Level> {
    match level_str.to_lowercase().as_str() {
        "error" => Ok(Level::ERROR),
        "warn" => Ok(Level::WARN),
        "info" => Ok(Level::INFO),
        "debug" => Ok(Level::DEBUG),
        "trace" => Ok(Level::TRACE),
        _ => Err(anyhow::anyhow!("Invalid log level: {}", level_str)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ConfigFile::default();
        assert_eq!(config.server.port, 8080);
        assert!(config.server.auto_open_browser);
        assert!(config.discovery.enabled);
    }

    #[test]
    fn test_parse_log_level() {
        assert!(matches!(parse_log_level("info"), Ok(Level::INFO)));
        assert!(matches!(parse_log_level("debug"), Ok(Level::DEBUG)));
        assert!(parse_log_level("invalid").is_err());
    }
}
