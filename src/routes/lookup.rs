// Route lookup engine with longest prefix matching

use super::{Route, RoutingTable};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub struct RouteEngine {
    routes: Vec<Route>,
}

impl RouteEngine {
    pub fn new(routing_table: &RoutingTable) -> Self {
        RouteEngine {
            routes: routing_table.routes.clone(),
        }
    }

    /// Find the best matching route for a destination IP using longest prefix matching
    pub fn lookup(&self, dest: IpAddr) -> Option<&Route> {
        let mut best_match: Option<(&Route, u8)> = None;

        for route in &self.routes {
            if let Some(prefix_len) = matches_cidr(&route.destination, dest) {
                match best_match {
                    None => best_match = Some((route, prefix_len)),
                    Some((_, current_len)) if prefix_len > current_len => {
                        best_match = Some((route, prefix_len));
                    }
                    _ => {}
                }
            }
        }

        best_match.map(|(route, _)| route)
    }
}

/// Check if an IP matches a CIDR and return the prefix length if it does
fn matches_cidr(cidr: &str, ip: IpAddr) -> Option<u8> {
    // Handle special cases
    if cidr == "default" || cidr == "0.0.0.0/0" || cidr == "::/0" {
        return Some(0);
    }

    // Parse CIDR notation
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.is_empty() {
        return None;
    }

    let network_ip: IpAddr = parts[0].parse().ok()?;
    let prefix_len: u8 = if parts.len() > 1 {
        parts[1].parse().ok()?
    } else {
        // No prefix length specified, assume /32 for IPv4 or /128 for IPv6
        match network_ip {
            IpAddr::V4(_) => 32,
            IpAddr::V6(_) => 128,
        }
    };

    // Check if both IPs are the same version
    match (network_ip, ip) {
        (IpAddr::V4(net), IpAddr::V4(addr)) => {
            if ip_v4_matches(net, addr, prefix_len) {
                Some(prefix_len)
            } else {
                None
            }
        }
        (IpAddr::V6(net), IpAddr::V6(addr)) => {
            if ip_v6_matches(net, addr, prefix_len) {
                Some(prefix_len)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn ip_v4_matches(network: Ipv4Addr, addr: Ipv4Addr, prefix_len: u8) -> bool {
    if prefix_len == 0 {
        return true;
    }
    if prefix_len > 32 {
        return false;
    }

    let network_bits = u32::from(network);
    let addr_bits = u32::from(addr);

    let mask = if prefix_len == 32 {
        0xFFFFFFFF
    } else {
        0xFFFFFFFF << (32 - prefix_len)
    };

    (network_bits & mask) == (addr_bits & mask)
}

fn ip_v6_matches(network: Ipv6Addr, addr: Ipv6Addr, prefix_len: u8) -> bool {
    if prefix_len == 0 {
        return true;
    }
    if prefix_len > 128 {
        return false;
    }

    let network_bits = u128::from(network);
    let addr_bits = u128::from(addr);

    let mask = if prefix_len == 128 {
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
    } else {
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF << (128 - prefix_len)
    };

    (network_bits & mask) == (addr_bits & mask)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_matches() {
        let network: Ipv4Addr = "192.168.1.0".parse().unwrap();
        let addr1: Ipv4Addr = "192.168.1.100".parse().unwrap();
        let addr2: Ipv4Addr = "192.168.2.100".parse().unwrap();

        assert!(ip_v4_matches(network, addr1, 24));
        assert!(!ip_v4_matches(network, addr2, 24));
    }

    #[test]
    fn test_matches_cidr() {
        let ip: IpAddr = "192.168.1.100".parse().unwrap();

        assert_eq!(matches_cidr("192.168.1.0/24", ip), Some(24));
        assert_eq!(matches_cidr("192.168.0.0/16", ip), Some(16));
        assert_eq!(matches_cidr("0.0.0.0/0", ip), Some(0));
        assert_eq!(matches_cidr("192.168.2.0/24", ip), None);
    }
}
