# Network Route Visualizer

A cross-platform network topology visualization tool that displays routing tables as an interactive 3D graph. Visualize routes, trace paths, discover connected nodes, and diagnose connectivity issues—all in a beautiful 3D interface.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey)
![Status](https://img.shields.io/badge/status-in%20development-yellow)

## Overview

Managing complex routing scenarios with multiple VPN connections can be challenging. This tool helps you answer questions like:
- Which VPN does traffic to destination X use?
- Can machine A reach machine K?
- What's the latency between nodes?
- Is the VPN path faster than the public internet?
- Why can't two machines communicate?

## Features

### 🗺️ Interactive 3D Visualization
- Beautiful 3D graph powered by three.js
- Your machine at the center with routes radiating outward
- Color-coded connections based on performance
- Interactive: click nodes and edges for detailed information

### 🔍 Route Tracing
- Enter any destination (IP or domain name)
- See the exact path your traffic will take
- Highlights which VPN tunnel or interface will be used
- Perform longest prefix matching on your routing table

### 🌐 Multi-Node Discovery
- Zero-configuration auto-discovery of other nodes
- View routing tables from remote machines
- See the entire network topology
- Identify connectivity gaps with actionable suggestions

### ⚡ Performance Testing
- Automatic background latency monitoring
- On-demand bandwidth testing between nodes
- Visual indicators show connection quality
- Compare VPN routes vs. public internet paths

### 🔧 Connectivity Diagnosis
- Detect when nodes can't communicate
- Get platform-specific fix commands (ready to copy-paste)
- See alternative routing paths
- Identify firewall and configuration issues

## Installation

### Prerequisites
- Rust 1.70 or later (for building from source)
- Elevated privileges (root on Linux/macOS, Administrator on Windows)

### Building from Source

```bash
git clone https://github.com/yourusername/network-route-visualizer.git
cd network-route-visualizer
cargo build --release
```

The compiled binary will be in `target/release/network-route-visualizer`.

### Pre-built Binaries

Download the latest release for your platform:
- [Linux (x86_64)](releases)
- [Windows (x86_64)](releases)
- [macOS (Intel & Apple Silicon)](releases)

## Quick Start

### Single Machine Mode

```bash
# Linux/macOS (requires root)
sudo ./network-route-visualizer

# Windows (run as Administrator)
.\network-route-visualizer.exe
```

The tool will:
1. Start a web server on `http://localhost:8080`
2. Auto-open your browser
3. Display your routing table as a 3D graph

Enter a destination in the search box to trace the route!

### With Custom Configuration

```bash
# Use a specific port
sudo ./network-route-visualizer --port 3000

# Disable browser auto-open
sudo ./network-route-visualizer --no-browser

# Use a config file
sudo ./network-route-visualizer --config my-config.toml

# Set log level
sudo ./network-route-visualizer --log-level debug

# Disable discovery
sudo ./network-route-visualizer --no-discovery
```

### Multi-Node Mesh

Run the same binary on multiple machines connected via VPN. They'll automatically discover each other and share routing information.

```bash
# On machine A
sudo ./network-route-visualizer

# On machine B (on the same VPN network)
sudo ./network-route-visualizer

# They'll discover each other automatically!
```

## Usage

### Command-Line Options

```bash
network-route-visualizer [OPTIONS]

OPTIONS:
    -p, --port <PORT>                   Web server port (default: 8080)
        --no-browser                    Don't auto-open browser
    -l, --log-level <LEVEL>            Logging level: error, warn, info, debug, trace
    -c, --config <PATH>                Path to configuration file
        --discovery-interval <SECONDS>  Discovery interval (default: 30)
        --peer-timeout <SECONDS>        Peer timeout (default: 90)
        --ping-interval <SECONDS>       Ping interval (default: 60)
        --bandwidth-duration <SECONDS>  Bandwidth test duration (default: 10)
        --bandwidth-port <PORT>         Bandwidth test port (default: 9090)
        --no-discovery                  Disable node discovery
        --no-ping                       Disable automatic ping
    -h, --help                         Print help information
    -V, --version                      Print version information
```

### Configuration File

Create a `config.toml` file for persistent settings:

```toml
[server]
port = 8080
auto_open_browser = true
bind_address = "127.0.0.1"  # Use "0.0.0.0" for external access

[discovery]
enabled = true
interval_seconds = 30
timeout_seconds = 90
multicast_group = "239.255.42.1"
multicast_port = 5678

[testing]
ping_enabled = true
ping_interval_seconds = 60
bandwidth_test_duration = 10
bandwidth_port = 9090

[logging]
level = "info"
# file = "/var/log/network-route-visualizer.log"  # Optional
```

**Configuration Priority**: CLI arguments > Config file > Environment variables > Defaults

**Environment Variables**:
```bash
export NRV_PORT=3000
export NRV_LOG_LEVEL=debug
export NRV_NO_BROWSER=1
export NRV_CONFIG=/path/to/config.toml
```

See `config.example.toml` for a complete example.

### Platform-Specific Notes

**Linux:**
- Requires root or `CAP_NET_ADMIN` capability
- Uses `ip route show` for routing table
- Best tested with OpenConnect and WireGuard VPNs

**Windows:**
- Must run as Administrator
- Uses `route print` or PowerShell `Get-NetRoute`

**macOS:**
- Requires root
- Uses `netstat -rn` for routing table

## API Documentation

The tool exposes REST and WebSocket APIs for programmatic access. See [API_SPECIFICATION.md](API_SPECIFICATION.md) for complete documentation.

Quick examples:

```bash
# Get routing table
curl http://localhost:8080/api/routing-table

# Trace route to destination
curl -X POST http://localhost:8080/api/trace-route \
  -H "Content-Type: application/json" \
  -d '{"destination": "8.8.8.8"}'

# List discovered nodes
curl http://localhost:8080/api/nodes
```

## Documentation

- **[PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)** - Project goals, use cases, and features
- **[TECHNICAL_ARCHITECTURE.md](TECHNICAL_ARCHITECTURE.md)** - System design and architecture
- **[IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md)** - Development phases and milestones
- **[API_SPECIFICATION.md](API_SPECIFICATION.md)** - Complete API reference
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Common issues and solutions
- **[TESTING.md](TESTING.md)** - Testing guide and procedures

## Development

### Building

```bash
# Development build
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Project Structure

```
network-route-visualizer/
├── src/
│   ├── main.rs              # Entry point
│   ├── routes/              # Routing table parsing
│   ├── discovery/           # Node discovery protocol
│   ├── testing/             # Ping and bandwidth testing
│   ├── api/                 # REST and WebSocket APIs
│   └── web/static/          # Frontend (HTML, JS, CSS)
├── tests/                   # Integration tests
└── docs/                    # Documentation
```

See [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md) for the development plan.

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Use Cases

### VPN Troubleshooting
You have multiple VPN connections (corporate, personal, region-specific). Quickly see which tunnel your traffic uses and why.

### Network Diagnostics
Two machines on different VPNs can't communicate. The tool shows the missing route and provides the exact command to fix it.

### Performance Optimization
Compare latency between VPN path and public internet. Decide if the VPN overhead is worth it for specific destinations.

### Network Visualization
Understand your complex network topology at a glance. Perfect for presentations or documentation.

## Technology Stack

- **Backend**: Rust with axum web framework
- **Frontend**: three.js for 3D visualization
- **Communication**: REST API + WebSocket for real-time updates
- **Discovery**: UDP multicast with gossip protocol
- **Platform Support**: Linux, Windows, macOS

## Roadmap

- [x] Core routing table visualization
- [x] Route tracing with longest prefix match
- [ ] Multi-node auto-discovery (in progress)
- [ ] Performance testing (ping, bandwidth)
- [ ] Public internet route comparison
- [ ] Cross-platform releases

See [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md) for detailed milestones.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- 3D visualization powered by [three.js](https://threejs.org/)
- Inspired by the need to understand complex VPN routing scenarios

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/network-route-visualizer/issues)
- **Documentation**: See the `docs/` directory
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/network-route-visualizer/discussions)

---

**Note**: This tool requires elevated privileges to read routing tables. Always review the code before running with sudo/admin rights.
