// WireGuard peer discovery
// Parses `wg show` output to discover WireGuard peers for unicast discovery
// (since multicast doesn't work over WireGuard tunnels)

use std::net::{IpAddr, SocketAddr};
use std::process::Command;
use std::str::FromStr;

/// Information about a WireGuard peer
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields kept for debugging and future use
pub struct WireGuardPeer {
    /// The WireGuard interface this peer belongs to
    pub interface: String,
    /// The peer's public key (truncated for logging)
    pub public_key: String,
    /// The peer's endpoint (IP:port) if known
    pub endpoint: Option<SocketAddr>,
    /// Allowed IPs for this peer
    pub allowed_ips: Vec<String>,
}

/// Information about a WireGuard interface
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields kept for debugging and future use
pub struct WireGuardInterface {
    pub name: String,
    pub peers: Vec<WireGuardPeer>,
}

/// Get all WireGuard interfaces and their peers
pub fn get_wireguard_interfaces() -> Vec<WireGuardInterface> {
    let mut interfaces = Vec::new();

    // First, get list of WireGuard interface names
    let interface_names = get_wireguard_interface_names();

    for iface_name in interface_names {
        if let Some(iface) = parse_wireguard_interface(&iface_name) {
            interfaces.push(iface);
        }
    }

    interfaces
}

/// Get all known WireGuard peer endpoints for unicast discovery
pub fn get_wireguard_peer_endpoints() -> Vec<SocketAddr> {
    let interfaces = get_wireguard_interfaces();
    let mut endpoints = Vec::new();

    for iface in interfaces {
        for peer in iface.peers {
            if let Some(endpoint) = peer.endpoint {
                // Use the endpoint IP but with our discovery port
                // The actual port will be set by the caller
                endpoints.push(endpoint);
            }
        }
    }

    endpoints
}

/// Get WireGuard peer IPs (without port) for discovery announcements
pub fn get_wireguard_peer_ips() -> Vec<IpAddr> {
    get_wireguard_peer_endpoints()
        .into_iter()
        .map(|addr| addr.ip())
        .collect()
}

/// Get list of WireGuard interface names on the system
fn get_wireguard_interface_names() -> Vec<String> {
    // Try `wg show interfaces` first
    if let Ok(output) = Command::new("wg").args(["show", "interfaces"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.split_whitespace().map(String::from).collect();
        }
    }

    // Fallback: look for wg* interfaces in /sys/class/net (Linux)
    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
            return entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.starts_with("wg") {
                        Some(name)
                    } else {
                        None
                    }
                })
                .collect();
        }
    }

    Vec::new()
}

/// Parse WireGuard interface details using `wg show <interface>`
fn parse_wireguard_interface(interface_name: &str) -> Option<WireGuardInterface> {
    let output = Command::new("wg")
        .args(["show", interface_name])
        .output()
        .ok()?;

    if !output.status.success() {
        tracing::debug!("Failed to get WireGuard info for {}", interface_name);
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut peers = Vec::new();
    let mut current_peer: Option<WireGuardPeer> = None;

    for line in stdout.lines() {
        let line = line.trim();

        if line.starts_with("peer:") {
            // Save previous peer if exists
            if let Some(peer) = current_peer.take() {
                peers.push(peer);
            }

            // Start new peer
            let public_key = line
                .strip_prefix("peer:")
                .map(|s| s.trim().to_string())
                .unwrap_or_default();

            current_peer = Some(WireGuardPeer {
                interface: interface_name.to_string(),
                public_key,
                endpoint: None,
                allowed_ips: Vec::new(),
            });
        } else if let Some(ref mut peer) = current_peer {
            if line.starts_with("endpoint:") {
                if let Some(endpoint_str) = line.strip_prefix("endpoint:") {
                    let endpoint_str = endpoint_str.trim();
                    // Parse endpoint (format: IP:port or [IPv6]:port)
                    peer.endpoint = parse_endpoint(endpoint_str);
                }
            } else if line.starts_with("allowed ips:") {
                if let Some(ips_str) = line.strip_prefix("allowed ips:") {
                    peer.allowed_ips = ips_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
        }
    }

    // Don't forget the last peer
    if let Some(peer) = current_peer {
        peers.push(peer);
    }

    Some(WireGuardInterface {
        name: interface_name.to_string(),
        peers,
    })
}

/// Parse a WireGuard endpoint string (IP:port or [IPv6]:port)
fn parse_endpoint(endpoint_str: &str) -> Option<SocketAddr> {
    // Try direct parsing first
    if let Ok(addr) = SocketAddr::from_str(endpoint_str) {
        return Some(addr);
    }

    // Handle IPv6 format [::1]:port
    if endpoint_str.starts_with('[') {
        if let Some(bracket_end) = endpoint_str.find(']') {
            let ip_str = &endpoint_str[1..bracket_end];
            let port_str = endpoint_str.get(bracket_end + 2..)?;

            let ip: IpAddr = ip_str.parse().ok()?;
            let port: u16 = port_str.parse().ok()?;

            return Some(SocketAddr::new(ip, port));
        }
    }

    // Handle IPv4 format 1.2.3.4:port
    if let Some(colon_pos) = endpoint_str.rfind(':') {
        let ip_str = &endpoint_str[..colon_pos];
        let port_str = &endpoint_str[colon_pos + 1..];

        let ip: IpAddr = ip_str.parse().ok()?;
        let port: u16 = port_str.parse().ok()?;

        return Some(SocketAddr::new(ip, port));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_endpoint_ipv4() {
        let result = parse_endpoint("192.168.1.100:51820");
        assert!(result.is_some());
        let addr = result.unwrap();
        assert_eq!(addr.port(), 51820);
    }

    #[test]
    fn test_parse_endpoint_ipv6() {
        let result = parse_endpoint("[::1]:51820");
        assert!(result.is_some());
        let addr = result.unwrap();
        assert_eq!(addr.port(), 51820);
    }

    #[test]
    fn test_parse_endpoint_invalid() {
        assert!(parse_endpoint("invalid").is_none());
        assert!(parse_endpoint("").is_none());
    }
}
