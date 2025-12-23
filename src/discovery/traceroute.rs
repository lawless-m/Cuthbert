// Traceroute implementation for public internet route visualization
//
// This module provides cross-platform traceroute functionality to discover
// the actual path packets take through the public internet, allowing comparison
// with VPN routes.

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::process::Command;
use tokio::task;

/// Represents a single hop in a traceroute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracerouteHop {
    /// Hop number (1-based)
    pub hop_number: u32,
    /// IP address of the hop (if available)
    pub ip: Option<String>,
    /// Hostname (if reverse DNS lookup succeeded)
    pub hostname: Option<String>,
    /// Round-trip times in milliseconds
    pub rtt_ms: Vec<Option<f64>>,
    /// Whether this hop timed out
    pub timed_out: bool,
}

/// Complete traceroute result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracerouteResult {
    /// Destination that was traced
    pub destination: String,
    /// Resolved IP address
    pub destination_ip: String,
    /// List of hops from source to destination
    pub hops: Vec<TracerouteHop>,
    /// Whether the trace completed successfully
    pub completed: bool,
    /// Error message if trace failed
    pub error: Option<String>,
}

/// Platform-specific traceroute executor
pub struct TracerouteExecutor;

impl TracerouteExecutor {
    /// Execute traceroute to the given destination
    ///
    /// Returns the traceroute result with all hops discovered.
    /// This is a blocking operation that spawns a subprocess.
    pub async fn traceroute(destination: IpAddr) -> Result<TracerouteResult, String> {
        // Execute in blocking task to avoid blocking async runtime
        task::spawn_blocking(move || Self::execute_traceroute_blocking(destination))
            .await
            .map_err(|e| format!("Task join error: {}", e))?
    }

    fn execute_traceroute_blocking(destination: IpAddr) -> Result<TracerouteResult, String> {
        let destination_str = destination.to_string();

        #[cfg(target_os = "linux")]
        let output = Self::execute_linux_traceroute(&destination_str)?;

        #[cfg(target_os = "windows")]
        let output = Self::execute_windows_traceroute(&destination_str)?;

        #[cfg(target_os = "macos")]
        let output = Self::execute_macos_traceroute(&destination_str)?;

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        return Err("Traceroute not supported on this platform".to_string());

        Ok(output)
    }

    #[cfg(target_os = "linux")]
    fn execute_linux_traceroute(destination: &str) -> Result<TracerouteResult, String> {
        tracing::info!("Executing Linux traceroute to {}", destination);

        // Try traceroute -n (no DNS resolution) first
        let output = Command::new("traceroute")
            .arg("-n") // No DNS resolution (faster)
            .arg("-q") // 3 queries per hop (default)
            .arg("3")
            .arg("-w") // Wait time
            .arg("2") // 2 seconds
            .arg("-m") // Max hops
            .arg("30")
            .arg(destination)
            .output()
            .map_err(|e| format!("Failed to execute traceroute: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("traceroute failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::debug!("traceroute output:\n{}", stdout);

        Self::parse_linux_traceroute(&stdout, destination)
    }

    #[cfg(target_os = "windows")]
    fn execute_windows_traceroute(destination: &str) -> Result<TracerouteResult, String> {
        tracing::info!("Executing Windows tracert to {}", destination);

        // Windows uses tracert with -d flag for no DNS resolution
        let output = Command::new("tracert")
            .arg("-d") // No DNS resolution
            .arg("-h") // Max hops
            .arg("30")
            .arg("-w") // Timeout per reply (milliseconds)
            .arg("2000")
            .arg(destination)
            .output()
            .map_err(|e| format!("Failed to execute tracert: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("tracert failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::debug!("tracert output:\n{}", stdout);

        Self::parse_windows_traceroute(&stdout, destination)
    }

    #[cfg(target_os = "macos")]
    fn execute_macos_traceroute(destination: &str) -> Result<TracerouteResult, String> {
        tracing::info!("Executing macOS traceroute to {}", destination);

        // macOS traceroute with -n for no DNS resolution
        let output = Command::new("traceroute")
            .arg("-n") // No DNS resolution
            .arg("-q") // Queries per hop
            .arg("3")
            .arg("-w") // Wait time
            .arg("2")
            .arg("-m") // Max hops
            .arg("30")
            .arg(destination)
            .output()
            .map_err(|e| format!("Failed to execute traceroute: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("traceroute failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::debug!("traceroute output:\n{}", stdout);

        Self::parse_macos_traceroute(&stdout, destination)
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn parse_linux_traceroute(output: &str, destination: &str) -> Result<TracerouteResult, String> {
        let mut hops = Vec::new();

        // Skip first line (header: "traceroute to...")
        for line in output.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Parse line format: " 1  192.168.1.1  1.234 ms  1.456 ms  1.678 ms"
            // or: " 1  * * *" (timeout)
            let parts: Vec<&str> = line.split_whitespace().collect();

            let hop_number: u32 = match parts.first().and_then(|s| s.parse().ok()) {
                Some(n) => n,
                None => continue, // Skip malformed lines
            };

            // Check for timeout (* * *)
            if parts.get(1).copied() == Some("*") {
                hops.push(TracerouteHop {
                    hop_number,
                    ip: None,
                    hostname: None,
                    rtt_ms: vec![None, None, None],
                    timed_out: true,
                });
                continue;
            }

            // Extract IP address (second field)
            let ip = parts.get(1).map(|s| s.to_string());

            // Extract RTT values (every other field after IP, before "ms")
            let mut rtt_values = Vec::new();
            let mut i = 2;
            while i < parts.len() {
                if let Some(part) = parts.get(i) {
                    if let Ok(rtt) = part.parse::<f64>() {
                        rtt_values.push(Some(rtt));
                        i += 2; // Skip "ms"
                    } else if *part == "*" {
                        rtt_values.push(None);
                        i += 1;
                    } else {
                        i += 1;
                    }
                } else {
                    break;
                }
            }

            hops.push(TracerouteHop {
                hop_number,
                ip,
                hostname: None, // DNS resolution disabled
                rtt_ms: rtt_values,
                timed_out: false,
            });
        }

        Ok(TracerouteResult {
            destination: destination.to_string(),
            destination_ip: destination.to_string(),
            hops,
            completed: true,
            error: None,
        })
    }

    #[cfg(target_os = "windows")]
    fn parse_windows_traceroute(
        output: &str,
        destination: &str,
    ) -> Result<TracerouteResult, String> {
        let mut hops = Vec::new();
        let mut in_trace_section = false;

        // Windows tracert format:
        // Tracing route to 8.8.8.8 over a maximum of 30 hops
        //
        //   1     1 ms     1 ms     1 ms  192.168.1.1
        //   2     *        *        *     Request timed out.

        for line in output.lines() {
            let line = line.trim();

            // Skip until we reach the trace section
            if line.starts_with("Tracing route") {
                in_trace_section = true;
                continue;
            }

            if !in_trace_section || line.is_empty() {
                continue;
            }

            // Parse hop line
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let hop_number: u32 = match parts[0].parse() {
                Ok(n) => n,
                Err(_) => continue,
            };

            // Check for timeout
            if line.contains("Request timed out") || parts.iter().any(|&p| p == "*") {
                hops.push(TracerouteHop {
                    hop_number,
                    ip: None,
                    hostname: None,
                    rtt_ms: vec![None, None, None],
                    timed_out: true,
                });
                continue;
            }

            // Extract RTT values and IP
            let mut rtt_values = Vec::new();
            let mut ip = None;

            for i in 1..parts.len() {
                let part = parts[i];

                // Try to parse as RTT (number followed by "ms")
                if let Ok(rtt) = part.parse::<f64>() {
                    rtt_values.push(Some(rtt));
                } else if part == "<1" {
                    rtt_values.push(Some(0.5)); // Less than 1ms
                } else if part.contains('.') || part.contains(':') {
                    // Looks like an IP address
                    ip = Some(part.to_string());
                }
            }

            hops.push(TracerouteHop {
                hop_number,
                ip,
                hostname: None,
                rtt_ms: rtt_values,
                timed_out: false,
            });
        }

        Ok(TracerouteResult {
            destination: destination.to_string(),
            destination_ip: destination.to_string(),
            hops,
            completed: true,
            error: None,
        })
    }

    #[cfg(target_os = "macos")]
    fn parse_macos_traceroute(output: &str, destination: &str) -> Result<TracerouteResult, String> {
        // macOS traceroute output is similar to Linux
        // Format: "traceroute to 8.8.8.8 (8.8.8.8), 30 hops max, 60 byte packets"
        //  1  192.168.1.1  1.234 ms  1.456 ms  1.678 ms
        Self::parse_linux_traceroute(output, destination)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn test_parse_linux_traceroute() {
        let output = r#"traceroute to 8.8.8.8 (8.8.8.8), 30 hops max, 60 byte packets
 1  192.168.1.1  1.234 ms  1.456 ms  1.678 ms
 2  10.0.0.1  5.123 ms  5.234 ms  5.345 ms
 3  * * *
 4  8.8.8.8  15.678 ms  15.789 ms  15.890 ms"#;

        let result = TracerouteExecutor::parse_linux_traceroute(output, "8.8.8.8").unwrap();

        assert_eq!(result.hops.len(), 4);
        assert_eq!(result.hops[0].hop_number, 1);
        assert_eq!(result.hops[0].ip, Some("192.168.1.1".to_string()));
        assert_eq!(result.hops[0].rtt_ms.len(), 3);
        assert!(!result.hops[0].timed_out);

        assert_eq!(result.hops[2].hop_number, 3);
        assert!(result.hops[2].timed_out);
        assert_eq!(result.hops[2].ip, None);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_parse_windows_traceroute() {
        let output = r#"
Tracing route to 8.8.8.8 over a maximum of 30 hops

  1     1 ms     1 ms     1 ms  192.168.1.1
  2     5 ms     5 ms     5 ms  10.0.0.1
  3     *        *        *     Request timed out.
  4    15 ms    15 ms    16 ms  8.8.8.8

Trace complete."#;

        let result = TracerouteExecutor::parse_windows_traceroute(output, "8.8.8.8").unwrap();

        assert_eq!(result.hops.len(), 4);
        assert_eq!(result.hops[0].hop_number, 1);
        assert_eq!(result.hops[0].ip, Some("192.168.1.1".to_string()));

        assert_eq!(result.hops[2].hop_number, 3);
        assert!(result.hops[2].timed_out);
    }
}
