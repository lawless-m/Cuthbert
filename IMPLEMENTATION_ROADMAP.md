# Implementation Roadmap

## Development Phases

### Phase 1: Core Foundation (MVP)
**Goal**: Single machine visualization with route tracing

#### Milestone 1.1: Rust Backend Basics
- [ ] Initialize Rust project with Cargo
- [ ] Set up axum web framework
- [ ] Create project structure (modules, main.rs)
- [ ] Implement basic HTTP server serving "Hello World"
- [ ] Add static file serving capability
- [ ] Embed HTML/CSS/JS files in binary

**Dependencies**:
```toml
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tower-http = { version = "0.5", features = ["fs", "cors"] }
```

#### Milestone 1.2: Routing Table Parser
- [ ] Implement platform detection (Linux/Windows/macOS)
- [ ] Create `Route` data structure
- [ ] Linux: Parse `ip route show` output
- [ ] Windows: Parse `route print` output
- [ ] macOS: Parse `netstat -rn` output
- [ ] Handle CIDR notation
- [ ] Unit tests for each parser

**Test Cases**:
- Default route (0.0.0.0/0)
- Specific routes (/32, /24, /16)
- Link-local routes
- VPN interface routes (tun, wg)

#### Milestone 1.3: Route Lookup Engine
- [ ] Implement longest prefix match algorithm
- [ ] DNS resolution (A records)
- [ ] Create `/api/routing-table` endpoint
- [ ] Create `/api/trace-route` endpoint
- [ ] Handle invalid input gracefully
- [ ] Unit tests for prefix matching

**Algorithm**: 
- Convert all routes to binary prefix tree (trie)
- Traverse tree with destination IP
- Return longest matching route

#### Milestone 1.4: Basic Frontend
- [ ] Create HTML page with basic layout
- [ ] Initialize three.js scene
- [ ] Render single node (local machine)
- [ ] Add OrbitControls
- [ ] Fetch routing table from API
- [ ] Render routes as edges
- [ ] Add simple styling/UI

#### Milestone 1.5: Route Tracing UI
- [ ] Add input field for destination
- [ ] Call trace-route API on submit
- [ ] Highlight matching route in scene
- [ ] Display route details (interface, gateway, metric)
- [ ] Color-code edges based on route type
- [ ] Add reset button

**Deliverable**: Single machine can visualize its routing table and trace routes

---

### Phase 2: 3D Visualization Enhancement
**Goal**: Professional, interactive 3D graph

#### Milestone 2.1: Improved Visual Design
- [ ] Create distinct geometries for different node types
  - [ ] Local machine: Large glowing sphere
  - [ ] Gateway: Medium cube/pyramid
  - [ ] Destination networks: Small spheres
- [ ] Implement color scheme
- [ ] Add lighting (ambient + directional)
- [ ] Edge styling (solid vs dashed)
- [ ] Add grid or reference plane

#### Milestone 2.2: Interaction
- [ ] Raycasting for node/edge selection
- [ ] Hover effects (glow, tooltip)
- [ ] Click node to show details panel
- [ ] Click edge to show route details
- [ ] Double-click to focus camera
- [ ] Keyboard shortcuts (R for reset view, T for trace)

#### Milestone 2.3: Labels and HUD
- [ ] CSS2DRenderer for labels
- [ ] Node labels (hostname, IP)
- [ ] Edge labels (interface, metric)
- [ ] Info panel (selected node/edge details)
- [ ] Status bar (FPS, node count)
- [ ] Legend (color meanings)

#### Milestone 2.4: Layout Algorithm
- [ ] Implement force-directed layout
  - [ ] Local machine at center (fixed)
  - [ ] Routes positioned around it
  - [ ] Repulsion between nodes
  - [ ] Attraction along edges
- [ ] Optional: manual positioning mode
- [ ] Save/restore layout

**Deliverable**: Beautiful, interactive 3D visualization

---

### Phase 3: Multi-Node Discovery
**Goal**: Distributed mesh with auto-discovery

#### Milestone 3.1: WebSocket Infrastructure
- [ ] Add WebSocket support to axum
- [ ] Implement WebSocket handler
- [ ] Message protocol (JSON)
- [ ] Client-side WebSocket connection
- [ ] Reconnection logic
- [ ] Message queue for reliability

**Dependencies**:
```toml
tokio-tungstenite = "0.21"
```

#### Milestone 3.2: Discovery Protocol
- [ ] UDP multicast setup
- [ ] Broadcast announcement message
- [ ] Listen for announcements
- [ ] Parse peer announcements
- [ ] Maintain peer registry (HashMap)
- [ ] Timeout inactive peers
- [ ] Send WebSocket updates on peer changes

**Configuration**:
- Multicast group: 239.255.42.1:5678
- Announcement interval: 30s
- Timeout: 90s

#### Milestone 3.3: Gossip Protocol
- [ ] Implement `/api/nodes` endpoint (list peers)
- [ ] Request peer list from discovered nodes
- [ ] Merge peer lists
- [ ] Avoid duplicate connections
- [ ] Handle network partitions

#### Milestone 3.4: Remote Routing Tables
- [ ] Implement `/api/nodes/{id}/routing-table` endpoint
- [ ] HTTP client to query remote nodes
- [ ] Cache remote routing tables
- [ ] Update cache on periodic refresh
- [ ] Display multiple nodes in 3D scene
- [ ] Toggle between machine views

#### Milestone 3.5: Connectivity Diagnosis
- [ ] Detect indirect knowledge (gossip only)
- [ ] Attempt direct connection
- [ ] Classify failures:
  - [ ] No route to host
  - [ ] Connection refused (firewall)
  - [ ] Timeout
- [ ] Generate suggested fix commands
- [ ] Display in UI with copy button

**Deliverable**: Multi-node mesh with auto-discovery

---

### Phase 4: Performance Testing
**Goal**: Latency monitoring and bandwidth testing

#### Milestone 4.1: Background Ping
- [ ] Implement ICMP ping (or TCP ping fallback)
- [ ] Ping all discovered nodes every 60s
- [ ] Store latency history (last 100 samples)
- [ ] Send WebSocket updates with latency
- [ ] Update edge colors in real-time
- [ ] Configurable ping interval

**Dependencies**:
```toml
surge-ping = "0.8"  # ICMP ping
```

#### Milestone 4.2: On-Demand Bandwidth Test
- [ ] Implement TCP bandwidth test protocol
  - [ ] Server mode: listen and receive
  - [ ] Client mode: send data
  - [ ] Measure throughput both directions
- [ ] `/api/test/bandwidth` endpoint
- [ ] Coordinate test between two nodes
- [ ] Progress updates via WebSocket
- [ ] Display results on edge label
- [ ] Configurable test duration

#### Milestone 4.3: UI for Testing
- [ ] "Test Bandwidth" button on edge click
- [ ] Progress bar during test
- [ ] Results display (Mbps up/down)
- [ ] Historical test results
- [ ] Cancel test button

**Deliverable**: Real-time latency + on-demand bandwidth testing

---

### Phase 5: Public Internet Routes
**Goal**: Compare VPN vs public paths with traceroute

#### Milestone 5.1: Traceroute Implementation
- [ ] Linux: Execute `traceroute -n`
- [ ] Windows: Execute `tracert -d`
- [ ] macOS: Execute `traceroute -n`
- [ ] Parse hop output
- [ ] Reverse DNS lookup for IPs (optional)
- [ ] `/api/traceroute` endpoint

#### Milestone 5.2: Public Route Visualization
- [ ] Toggle "Show Public Routes" in UI
- [ ] Request traceroute for node pairs
- [ ] Render internet hops as new nodes
- [ ] Position hops spatially (geographic if possible)
- [ ] Draw edges through hops
- [ ] Differentiate public edges (dashed, different color)

#### Milestone 5.3: Route Comparison
- [ ] Side-by-side comparison panel
- [ ] VPN path vs Public path
- [ ] Hop count comparison
- [ ] Latency comparison
- [ ] Highlight differences
- [ ] Recommend better path

**Deliverable**: Full public internet route visualization

---

### Phase 6: Polish and Deployment
**Goal**: Production-ready tool

#### Milestone 6.1: Error Handling
- [ ] Comprehensive error types
- [ ] User-friendly error messages
- [ ] Logging framework (tracing/log)
- [ ] Graceful degradation
- [ ] Retry logic for transient failures

**Dependencies**:
```toml
tracing = "0.1"
tracing-subscriber = "0.3"
```

#### Milestone 6.2: Configuration
- [ ] Command-line arguments (clap)
  - [ ] Port number
  - [ ] Auto-open browser
  - [ ] Log level
  - [ ] Discovery settings
- [ ] Optional config file (TOML)
- [ ] Environment variables

**Dependencies**:
```toml
clap = { version = "4", features = ["derive"] }
```

#### Milestone 6.3: Documentation
- [ ] README.md with installation/usage
- [ ] API documentation
- [ ] Architecture diagrams
- [ ] Troubleshooting guide
- [ ] Video demo/screenshots

#### Milestone 6.4: Cross-Platform Testing
- [ ] Test on Ubuntu 22.04/24.04
- [ ] Test on Windows 10/11
- [ ] Test on macOS (Intel and ARM)
- [ ] Test with OpenConnect VPN
- [ ] Test with WireGuard
- [ ] Test with multiple VPNs simultaneously

#### Milestone 6.5: Build and Release
- [ ] GitHub Actions CI/CD
- [ ] Cross-compilation setup
- [ ] Release binaries for all platforms
- [ ] Checksums and signatures
- [ ] Homebrew formula (optional)
- [ ] Snap/Flatpak package (optional)

**Deliverable**: Production-ready, cross-platform releases

---

## Implementation Order

### Recommended Sequence
1. **Phase 1** (Core MVP) - 2-3 weeks
   - Get basic functionality working on one platform (Linux)
   - Validate core concepts
   
2. **Phase 2** (Visualization) - 1-2 weeks
   - Make it look good
   - Ensure UX is intuitive
   
3. **Phase 3** (Multi-Node) - 2-3 weeks
   - Most complex networking code
   - Requires testing with multiple machines
   
4. **Phase 4** (Performance) - 1 week
   - Adds value but builds on existing infrastructure
   
5. **Phase 5** (Public Routes) - 1 week
   - Nice-to-have feature
   
6. **Phase 6** (Polish) - 1-2 weeks
   - Continuous throughout, but final push at end

**Total Estimated Time**: 8-12 weeks for full implementation

---

## Testing Strategy

### Unit Tests
- Route parsing for each platform
- Longest prefix matching algorithm
- JSON serialization/deserialization
- Discovery message parsing

### Integration Tests
- Full API endpoint tests
- WebSocket message flow
- Multi-node discovery scenario
- Route tracing end-to-end

### Manual Testing
- Visual inspection of 3D scene
- Interaction testing (clicks, hovers)
- Cross-platform routing table parsing
- Real VPN connections (OpenConnect, WireGuard)

### Performance Testing
- Large routing tables (500+ routes)
- Many nodes (20+, though target is 10)
- Rapid discovery/disappearance
- WebSocket message throughput

---

## Risk Mitigation

### Technical Risks

**Risk**: Platform-specific routing table formats vary widely
- **Mitigation**: Implement robust parsers with extensive test cases, graceful fallbacks

**Risk**: UDP multicast may not work across all VPN types
- **Mitigation**: Fallback to manual peer configuration, TCP-based discovery

**Risk**: Three.js performance degrades with many nodes
- **Mitigation**: Level-of-detail rendering, culling, efficient geometries

**Risk**: Elevated permissions required
- **Mitigation**: Clear documentation, platform-specific installers that request permissions

**Risk**: Firewalls block discovery or testing traffic
- **Mitigation**: Configurable ports, detection and clear error messages

### Project Risks

**Risk**: Scope creep
- **Mitigation**: Stick to phased approach, mark nice-to-haves clearly

**Risk**: Cross-platform testing delays
- **Mitigation**: Set up VMs early, automate testing where possible

**Risk**: Complex Rust async code
- **Mitigation**: Keep async boundaries clear, extensive logging

---

## Success Metrics

- [ ] Correctly parses routing tables on all three platforms
- [ ] Discovers nodes within 10 seconds on same subnet
- [ ] Route trace returns results in <100ms
- [ ] 3D visualization renders at 60 FPS
- [ ] Works with OpenConnect and WireGuard simultaneously
- [ ] Generates correct fix commands for connectivity issues
- [ ] Single binary deployment (no external dependencies)
- [ ] Zero-config auto-discovery works for 10 nodes
- [ ] User can complete full workflow (discover, trace, test) in <5 minutes

---

## Future Enhancements (Post-Launch)

### Additional Features
- Historical route changes over time
- Traffic volume estimation
- Automatic route optimization
- BGP information display
- Integration with network monitoring tools
- Mobile companion app
- Cloud service for remote management
- Docker container deployment
- Kubernetes network policy visualization

### Technical Improvements
- More efficient binary format for messages
- Delta updates instead of full snapshots
- Compressed WebSocket messages
- GPU acceleration for large graphs
- Machine learning for anomaly detection
- Integration with prometheus/grafana
