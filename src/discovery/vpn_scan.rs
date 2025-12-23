// VPN subnet discovery via ping sweep
// For VPNs like OpenConnect where broadcast/multicast doesn't work
// and we can't query the VPN tool for peer info (unlike WireGuard)

use std::net::{IpAddr, Ipv4Addr};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use surge_ping::{Client, Config, PingIdentifier, PingSequence};
use tokio::sync::Semaphore;

/// Information about a VPN/tun interface
#[derive(Debug, Clone)]
pub struct VpnInterface {
    pub name: String,
    pub address: Ipv4Addr,
    pub prefix_len: u8,
}

impl VpnInterface {
    /// Calculate the number of hosts in this subnet (excluding network and broadcast)
    #[allow(dead_code)] // Used in tests
    pub fn host_count(&self) -> u32 {
        if self.prefix_len >= 31 {
            return if self.prefix_len == 31 { 2 } else { 1 };
        }
        (1u32 << (32 - self.prefix_len)) - 2
    }

    /// Generate all host IPs in the subnet (excluding network address, broadcast, and self)
    pub fn host_ips(&self) -> Vec<Ipv4Addr> {
        let addr_u32 = u32::from(self.address);
        let mask = if self.prefix_len == 0 {
            0
        } else {
            !((1u32 << (32 - self.prefix_len)) - 1)
        };
        let network = addr_u32 & mask;

        let mut hosts = Vec::new();

        if self.prefix_len >= 31 {
            // Point-to-point link (/31 or /32)
            if self.prefix_len == 31 {
                // /31 has 2 usable addresses
                for i in 0..2 {
                    let ip = Ipv4Addr::from(network + i);
                    if ip != self.address {
                        hosts.push(ip);
                    }
                }
            }
            // /32 has no other hosts
        } else {
            // Normal subnet - skip network address (first) and broadcast (last)
            let first_host = network + 1;
            let last_host = network + (1u32 << (32 - self.prefix_len)) - 2;

            for host in first_host..=last_host {
                let ip = Ipv4Addr::from(host);
                if ip != self.address {
                    hosts.push(ip);
                }
            }
        }

        hosts
    }
}

/// Detect tun/tap interfaces that might be VPN tunnels
pub fn get_vpn_interfaces() -> Vec<VpnInterface> {
    let mut interfaces = Vec::new();

    // Get list of tun interfaces
    let tun_names = get_tun_interface_names();

    for name in tun_names {
        if let Some(iface) = get_interface_address(&name) {
            interfaces.push(iface);
        }
    }

    interfaces
}

/// Get names of tun/tap interfaces from /sys/class/net
fn get_tun_interface_names() -> Vec<String> {
    let mut names = Vec::new();

    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_string();

                // Check if it's a tun/tap device
                if name.starts_with("tun") || name.starts_with("tap") {
                    names.push(name);
                    continue;
                }

                // Also check the device type for virtual interfaces
                // tun devices have type 65534 (ARPHRD_NONE) or similar
                let type_path = entry.path().join("type");
                if let Ok(type_str) = std::fs::read_to_string(&type_path) {
                    if let Ok(dev_type) = type_str.trim().parse::<u32>() {
                        // ARPHRD_NONE (65534) is commonly used by tun devices
                        if dev_type == 65534 {
                            names.push(name);
                        }
                    }
                }
            }
        }
    }

    names
}

/// Get IP address and prefix length for an interface using `ip addr show`
fn get_interface_address(interface_name: &str) -> Option<VpnInterface> {
    let output = Command::new("ip")
        .args(["addr", "show", interface_name])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse output looking for "inet X.X.X.X/Y" line
    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("inet ") {
            // Format: "inet 10.0.0.5/24 ..." or "inet 10.0.0.5/24 brd ... scope ..."
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(addr_cidr) = parts.get(1) {
                if let Some((addr_str, prefix_str)) = addr_cidr.split_once('/') {
                    if let (Ok(addr), Ok(prefix)) =
                        (addr_str.parse::<Ipv4Addr>(), prefix_str.parse::<u8>())
                    {
                        return Some(VpnInterface {
                            name: interface_name.to_string(),
                            address: addr,
                            prefix_len: prefix,
                        });
                    }
                }
            }
        }
    }

    None
}

/// Perform a ping sweep of VPN subnets and return responding IPs
/// This is an async function that pings all hosts concurrently with rate limiting
pub async fn scan_vpn_subnets() -> Vec<IpAddr> {
    let interfaces = get_vpn_interfaces();

    if interfaces.is_empty() {
        tracing::debug!("No VPN interfaces found for subnet scanning");
        return Vec::new();
    }

    let mut all_hosts: Vec<Ipv4Addr> = Vec::new();

    for iface in &interfaces {
        let hosts = iface.host_ips();
        let host_count = hosts.len();

        // Skip very large subnets to avoid overwhelming the network
        if host_count > 1024 {
            tracing::warn!(
                "VPN interface {} has {} hosts (/{}) - skipping (too large)",
                iface.name,
                host_count,
                iface.prefix_len
            );
            continue;
        }

        tracing::info!(
            "Will scan VPN interface {} ({}/{}) - {} hosts",
            iface.name,
            iface.address,
            iface.prefix_len,
            host_count
        );

        all_hosts.extend(hosts);
    }

    if all_hosts.is_empty() {
        return Vec::new();
    }

    // Ping all hosts concurrently with a semaphore to limit parallelism
    ping_sweep(&all_hosts).await
}

/// Ping sweep a list of IPs and return those that respond
async fn ping_sweep(hosts: &[Ipv4Addr]) -> Vec<IpAddr> {
    const MAX_CONCURRENT_PINGS: usize = 64;
    const PING_TIMEOUT_MS: u64 = 1000; // 1 second timeout for sweep

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_PINGS));
    let mut handles = Vec::with_capacity(hosts.len());

    for &host in hosts {
        let permit = semaphore.clone().acquire_owned().await;
        if permit.is_err() {
            continue;
        }
        let permit = permit.unwrap();

        handles.push(tokio::spawn(async move {
            let result = ping_host(IpAddr::V4(host), PING_TIMEOUT_MS).await;
            drop(permit);
            if result {
                Some(IpAddr::V4(host))
            } else {
                None
            }
        }));
    }

    let mut responding = Vec::new();
    for handle in handles {
        if let Ok(Some(ip)) = handle.await {
            responding.push(ip);
        }
    }

    tracing::info!(
        "VPN subnet scan complete: {}/{} hosts responded",
        responding.len(),
        hosts.len()
    );

    responding
}

/// Ping a single host and return true if it responds
async fn ping_host(address: IpAddr, timeout_ms: u64) -> bool {
    let config = Config::default();

    let client = match Client::new(&config) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let payload = [0; 56];
    let mut pinger = client.pinger(address, PingIdentifier(rand::random())).await;

    match tokio::time::timeout(
        Duration::from_millis(timeout_ms),
        pinger.ping(PingSequence(0), &payload),
    )
    .await
    {
        Ok(Ok(_)) => true,
        _ => false,
    }
}

/// Get VPN peer IPs by performing a subnet scan
/// This is the main entry point, similar to wireguard::get_wireguard_peer_ips()
/// Note: This performs network I/O so should be called sparingly
pub async fn get_vpn_peer_ips() -> Vec<IpAddr> {
    scan_vpn_subnets().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vpn_interface_host_count() {
        let iface = VpnInterface {
            name: "tun0".to_string(),
            address: Ipv4Addr::new(10, 0, 0, 5),
            prefix_len: 24,
        };
        assert_eq!(iface.host_count(), 254); // /24 = 256 - 2

        let iface = VpnInterface {
            name: "tun0".to_string(),
            address: Ipv4Addr::new(10, 0, 0, 1),
            prefix_len: 30,
        };
        assert_eq!(iface.host_count(), 2); // /30 = 4 - 2

        let iface = VpnInterface {
            name: "tun0".to_string(),
            address: Ipv4Addr::new(10, 0, 0, 1),
            prefix_len: 31,
        };
        assert_eq!(iface.host_count(), 2); // /31 point-to-point

        let iface = VpnInterface {
            name: "tun0".to_string(),
            address: Ipv4Addr::new(10, 0, 0, 1),
            prefix_len: 32,
        };
        assert_eq!(iface.host_count(), 1); // /32 single host
    }

    #[test]
    fn test_vpn_interface_host_ips() {
        let iface = VpnInterface {
            name: "tun0".to_string(),
            address: Ipv4Addr::new(10, 0, 0, 5),
            prefix_len: 30,
        };
        let hosts = iface.host_ips();
        // /30 network: 10.0.0.4, hosts: 10.0.0.5, 10.0.0.6, broadcast: 10.0.0.7
        // Self is 10.0.0.5, so we should get 10.0.0.6
        assert_eq!(hosts.len(), 1);
        assert!(hosts.contains(&Ipv4Addr::new(10, 0, 0, 6)));
        assert!(!hosts.contains(&Ipv4Addr::new(10, 0, 0, 5))); // Excludes self
    }

    #[test]
    fn test_vpn_interface_host_ips_31() {
        let iface = VpnInterface {
            name: "tun0".to_string(),
            address: Ipv4Addr::new(10, 0, 0, 0),
            prefix_len: 31,
        };
        let hosts = iface.host_ips();
        // /31 point-to-point: 10.0.0.0 and 10.0.0.1
        // Self is 10.0.0.0, so we should get 10.0.0.1
        assert_eq!(hosts.len(), 1);
        assert!(hosts.contains(&Ipv4Addr::new(10, 0, 0, 1)));
    }
}
