# Network Route Visualizer - Project Overview

## Project Description

A cross-platform network topology visualization tool that displays routing tables as an interactive 3D graph. The tool shows machines at the center with routes, gateways, and destinations as nodes and edges, with the ability to trace routes and test connections.

## Core Concept

- **Central machine view**: Each machine displays its routing table as a radial graph
- **Distributed mesh**: Multiple machines can discover each other and share routing information
- **Route tracing**: Users can enter a destination (IP or domain) and see the highlighted path
- **Performance testing**: On-demand latency monitoring and bandwidth testing between nodes
- **Public internet comparison**: Visualize alternative routes via public internet with full traceroute

## Target Platform

Cross-platform support for:
- Linux (primary focus - multiple VPN connections via OpenConnect and WireGuard)
- Windows
- macOS

## Use Case

Managing complex routing scenarios with multiple VPN connections where different traffic uses different tunnels. The tool helps answer:
- Which VPN does traffic to X use?
- Can machine A reach machine K?
- What's the latency/performance between nodes?
- Is the VPN path faster than public internet?
- Why can't two machines communicate?

## Key Features

### 1. Routing Table Visualization
- Parse routing tables from OS-specific commands
- Display as 3D node-edge graph using three.js
- Machine at center, routes radiating outward
- Interface/VPN labeling (tun0, wg0, etc.)

### 2. Route Tracing
- User enters destination (domain or IP)
- Perform longest prefix match lookup
- Highlight the path through the graph
- Show which interface/VPN would be used

### 3. Auto-Discovery Mesh
- Zero-config discovery of other nodes on VPN networks
- Broadcast on all interfaces to find peers
- Gossip protocol for peer sharing
- View routing tables from remote machines
- Identify connectivity gaps with remediation suggestions

### 4. Performance Testing
- **Automatic ping monitoring**: Background latency checks (every 30-60s)
- **On-demand bandwidth testing**: Click any edge to run throughput test
- Visual indicators: color-coded edges based on latency/performance
- Historical data display

### 5. Public Internet Routes
- Compare VPN path vs public internet path
- Full traceroute display showing all hops
- Internet hops rendered as additional nodes in 3D space
- Toggle between views

### 6. Diagnostic Features
- Detect when nodes know about each other but can't communicate
- Suggest routing commands to fix connectivity
- Display missing routes, firewall issues, or VPN problems
- Copy-paste ready commands for both Linux and Windows

## Technology Stack

### Backend
- **Language**: Rust
- **Web Framework**: axum or actix-web
- **Async Runtime**: tokio
- **Serialization**: serde (JSON)
- **Communication**: WebSocket for real-time updates
- **Discovery**: Custom broadcast + gossip protocol

### Frontend
- **Visualization**: three.js for 3D graphics
- **Layout**: Force-directed or custom positioning
- **Interaction**: OrbitControls, raycasting for selection
- **Communication**: WebSocket client

### Platform-Specific
- **Linux**: `ip route show`, netlink crate
- **Windows**: `route print` or PowerShell `Get-NetRoute`
- **macOS**: `netstat -rn`

## Architecture

### Single Binary Deployment
- Rust backend serves both API and static files
- Embedded HTML/CSS/JS in the binary
- Auto-opens browser on startup (optional)
- Runs on localhost (e.g., http://localhost:8080)

### Multi-Node Communication
- Each machine runs the same binary
- Nodes discover each other via broadcast
- RESTful/WebSocket API between nodes
- Endpoints: `/routing-table`, `/trace-route/{dest}`, `/test-bandwidth`

### Privilege Requirements
- Requires elevated permissions (root/admin) to read routing tables
- Document this requirement clearly

## User Workflow

### Single Machine Mode
1. User starts the binary
2. Browser opens to visualization
3. Routing table displayed as 3D graph
4. User enters destination to trace route
5. Path highlights through the graph

### Multi-Machine Mode
1. Drop binary on multiple machines
2. Binaries auto-discover each other via VPN networks
3. Web UI shows all discovered nodes
4. User can view any machine's routing table
5. User can test connections between any two nodes
6. Identify and fix connectivity issues

### Route Comparison
1. Select two nodes (or enter destination)
2. View VPN path
3. Toggle to see public internet path
4. Compare latency and hop count
5. Run on-demand bandwidth test if needed

## Success Criteria

- Clean, intuitive 3D visualization that doesn't become cluttered
- Sub-second response for route lookups
- Accurate longest prefix matching
- Reliable auto-discovery (>95% success rate on same network)
- Cross-platform binary compilation
- Single binary deployment with no dependencies
- Helpful diagnostic messages with actionable commands

## Future Enhancements (Out of Scope for Initial Version)

- Historical route changes over time
- Traffic volume visualization
- Automatic route optimization suggestions
- Export topology as diagram
- Mobile app version
- Packet capture integration
- BGP route information
