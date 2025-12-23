// Platform-specific routing table parsers

use super::{Route, RoutingTable};
use std::process::Command;

pub fn get_routing_table() -> Result<RoutingTable, String> {
    #[cfg(target_os = "linux")]
    return get_routing_table_linux();

    #[cfg(target_os = "windows")]
    return get_routing_table_windows();

    #[cfg(target_os = "macos")]
    return get_routing_table_macos();

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    return Err("Unsupported platform".to_string());
}

#[cfg(target_os = "linux")]
fn get_routing_table_linux() -> Result<RoutingTable, String> {
    // Try using `ip -json route show` first
    let output = Command::new("ip")
        .args(["-json", "route", "show"])
        .output()
        .map_err(|e| format!("Failed to execute ip command: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return parse_ip_json(&stdout);
    }

    // Fallback to parsing `ip route show` (non-JSON)
    let output = Command::new("ip")
        .args(["route", "show"])
        .output()
        .map_err(|e| format!("Failed to execute ip command: {}", e))?;

    if !output.status.success() {
        return Err("Failed to get routing table".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_ip_route(&stdout)
}

#[cfg(target_os = "linux")]
fn parse_ip_json(json_str: &str) -> Result<RoutingTable, String> {
    // Parse JSON output from `ip -json route show`
    let routes: Vec<serde_json::Value> = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let mut parsed_routes = Vec::new();

    for route in routes {
        let destination = route.get("dst")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
            .to_string();

        let gateway = route.get("gateway")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok());

        let interface = route.get("dev")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let metric = route.get("metric")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let flags = route.get("flags")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_else(Vec::new);

        parsed_routes.push(Route {
            destination,
            gateway,
            interface,
            metric,
            flags,
        });
    }

    let hostname = get_hostname();
    let timestamp = chrono::Utc::now().to_rfc3339();

    Ok(RoutingTable {
        hostname,
        routes: parsed_routes,
        timestamp,
    })
}

#[cfg(target_os = "linux")]
fn parse_ip_route(output: &str) -> Result<RoutingTable, String> {
    // Parse non-JSON output from `ip route show`
    let mut routes = Vec::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let destination = match parts.first() {
            Some(&"default") => "0.0.0.0/0".to_string(),
            Some(dest) => dest.to_string(),
            None => continue,
        };

        let mut gateway = None;
        let mut interface = String::new();
        let mut metric = 0;
        let flags = Vec::new();

        let mut i = 1;
        while i < parts.len() {
            match parts.get(i).copied() {
                Some("via") => {
                    if let Some(gw) = parts.get(i + 1) {
                        gateway = gw.parse().ok();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                Some("dev") => {
                    if let Some(iface) = parts.get(i + 1) {
                        interface = iface.to_string();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                Some("metric") => {
                    if let Some(m) = parts.get(i + 1) {
                        metric = m.parse().unwrap_or(0);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                _ => {
                    i += 1;
                }
            }
        }

        routes.push(Route {
            destination,
            gateway,
            interface,
            metric,
            flags,
        });
    }

    let hostname = get_hostname();
    let timestamp = chrono::Utc::now().to_rfc3339();

    Ok(RoutingTable {
        hostname,
        routes,
        timestamp,
    })
}

#[cfg(target_os = "windows")]
fn get_routing_table_windows() -> Result<RoutingTable, String> {
    Err("Windows support not yet implemented".to_string())
}

#[cfg(target_os = "macos")]
fn get_routing_table_macos() -> Result<RoutingTable, String> {
    Err("macOS support not yet implemented".to_string())
}

fn get_hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown".to_string())
}
