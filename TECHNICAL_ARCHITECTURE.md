# Technical Architecture

## System Components

### 1. Rust Backend Service

#### Core Modules

**Routing Table Parser**
- Platform detection: Linux, Windows, macOS
- Execute OS-specific commands
- Parse output into unified data structure
- Handle errors gracefully

**Route Lookup Engine**
- Longest prefix matching algorithm
- CIDR notation support
- DNS resolution
- Interface/gateway identification

**Discovery Service**
- Broadcast announcements on all interfaces
- Listen for peer broadcasts
- Maintain peer registry
- Gossip protocol for peer sharing
- Health checking (periodic pings)

**Performance Testing**
- Ping/ICMP implementation
- Bandwidth testing (iperf-style)
- Concurrent test management
- Results caching

**Traceroute Service**
- Platform-specific traceroute execution
- Parse hop-by-hop results
- TTL-based hop discovery
- Reverse DNS lookup for IPs

**Web Server**
- Static file serving (embedded HTML/CSS/JS)
- RESTful API endpoints
- WebSocket server for real-time updates
- CORS handling for development

#### Data Structures

```rust
struct Route {
    destination: IpNetwork,  // CIDR notation
    gateway: Option<IpAddr>,
    interface: String,
    metric: u32,
    flags: RouteFlags,
}

struct Node {
    id: String,
    hostname: String,
    ip_addresses: Vec<IpAddr>,
    last_seen: DateTime<Utc>,
    status: NodeStatus,  // Online, Offline, Unreachable
}

struct Connection {
    from: String,  // node_id
    to: String,    // node_id
    latency_ms: Option<u32>,
    bandwidth_mbps: Option<(u32, u32)>,  // (download, upload)
    last_tested: DateTime<Utc>,
}

struct TracerouteHop {
    hop_number: u8,
    ip: IpAddr,
    hostname: Option<String>,
    latency_ms: Vec<f32>,  // Multiple probes
}
```

#### API Endpoints

**Local Data**
- `GET /api/routing-table` - Current machine's routing table
- `GET /api/interfaces` - Network interfaces
- `POST /api/trace-route` - Trace route to destination
  - Body: `{"destination": "8.8.8.8"}`
  - Response: Route path with interfaces

**Discovery**
- `GET /api/nodes` - List of discovered nodes
- `GET /api/nodes/{id}` - Specific node details
- `POST /api/nodes/{id}/routing-table` - Get remote routing table

**Testing**
- `POST /api/test/ping` - Ping test
  - Body: `{"target": "node_id_or_ip"}`
- `POST /api/test/bandwidth` - Bandwidth test
  - Body: `{"target": "node_id", "duration_seconds": 10}`
- `POST /api/test/traceroute` - Full traceroute
  - Body: `{"target": "ip_or_domain", "use_public": true}`

**WebSocket Events**
- `/ws` - WebSocket endpoint
  - Bidirectional JSON messages
  - Server pushes: node updates, route changes, test results
  - Client requests: test initiation, remote queries

### 2. Frontend Application

#### three.js Scene Structure

**Scene Graph**
```
Scene
├── AmbientLight
├── DirectionalLight
├── Camera (PerspectiveCamera)
├── Controls (OrbitControls)
├── MeshNodes (Group)
│   ├── LocalMachine (Mesh - larger sphere)
│   ├── RemoteMachines (Mesh - medium spheres)
│   └── InternetHops (Mesh - small spheres, translucent)
├── Edges (Group)
│   ├── VPNConnections (Line/TubeGeometry - solid)
│   └── PublicRoutes (Line - dashed)
└── Labels (CSS2DRenderer)
    ├── NodeLabels
    └── EdgeLabels (latency, bandwidth)
```

#### Component Architecture

**WebSocketManager**
- Establish connection to backend
- Handle reconnection
- Parse incoming messages
- Queue outgoing requests

**SceneManager**
- Initialize three.js scene
- Manage camera and controls
- Handle window resize
- Render loop

**NodeManager**
- Create/update/remove node meshes
- Position nodes (force-directed or manual)
- Handle node interactions (click, hover)
- Update node status (online/offline)

**EdgeManager**
- Create/update/remove edges
- Color coding based on latency
- Animate traffic flow
- Handle edge interactions

**RouteTracer**
- Highlight path for destination query
- Animate packet flow
- Compare multiple paths
- Show hop details

**UIController**
- Input handling (destination entry)
- Control panel (test buttons, toggles)
- Info panels (node details, test results)
- Settings (auto-refresh rate, test duration)

#### Visual Design

**Color Scheme**
- Local machine: Bright blue (#3b82f6)
- Remote mesh nodes: Green (#10b981)
- Internet hops: Gray (#6b7280, 50% opacity)
- Edges:
  - Excellent (<30ms): Green (#10b981)
  - Good (30-100ms): Yellow (#fbbf24)
  - Poor (>100ms): Red (#ef4444)
  - Offline: Gray dashed

**Interaction States**
- Hover: Glow effect, show tooltip
- Selected: Brighten, increase size slightly
- Active test: Pulsing animation
- Highlighted path: Animated particles flowing

**Layout Strategy**
- Local machine at origin (0, 0, 0)
- Directly connected nodes in inner sphere (radius ~5 units)
- Secondary connections in outer sphere (radius ~10 units)
- Internet hops positioned along traceroute path
- Use physics simulation for initial positioning, allow manual adjustment

### 3. Discovery Protocol

#### Broadcast Announcement

**Message Format** (JSON over UDP multicast)
```json
{
  "type": "announce",
  "node_id": "uuid-string",
  "hostname": "machine-a",
  "addresses": ["10.20.0.5", "10.50.0.10"],
  "listening_port": 8080,
  "timestamp": "2025-11-22T10:30:00Z",
  "known_peers": ["uuid-2", "uuid-3"]
}
```

**Multicast Group**: 239.255.42.1:5678
**Announcement Frequency**: Every 30 seconds
**Timeout**: Node marked offline after 90 seconds without announcement

#### Gossip Protocol

When node A connects to node B:
1. A requests B's peer list: `GET /api/nodes`
2. B returns all known nodes
3. A attempts to connect to new nodes
4. A shares its peer list with newly discovered nodes

#### Peer Health Checking
- ICMP ping every 60 seconds to each known peer
- Update latency in real-time
- Mark offline after 3 consecutive failures
- Attempt reconnection every 5 minutes

### 4. Platform-Specific Implementation

#### Linux
```rust
fn get_routing_table_linux() -> Result<Vec<Route>> {
    // Execute: ip -json route show
    // Or parse: /proc/net/route
    // Use netlink crate for efficiency
}

fn traceroute_linux(dest: &str) -> Result<Vec<TracerouteHop>> {
    // Execute: traceroute -n -w 2 {dest}
    // Parse output
}
```

#### Windows
```rust
fn get_routing_table_windows() -> Result<Vec<Route>> {
    // Execute: route print
    // Or: powershell -Command "Get-NetRoute | ConvertTo-Json"
    // Parse table format output
}

fn traceroute_windows(dest: &str) -> Result<Vec<TracerouteHop>> {
    // Execute: tracert -d -w 2000 {dest}
    // Parse output
}
```

#### macOS
```rust
fn get_routing_table_macos() -> Result<Vec<Route>> {
    // Execute: netstat -rn
    // Parse BSD format output
}
```

## Data Flow

### Startup Sequence
1. Parse command-line arguments (port, config)
2. Initialize routing table parser
3. Start discovery service (broadcast + listen)
4. Start web server
5. Open browser to localhost:8080
6. Establish WebSocket connection
7. Send initial routing table to frontend
8. Begin background ping monitoring

### Route Trace Flow
1. User enters destination in UI
2. Frontend sends WebSocket message: `{"type": "trace", "destination": "8.8.8.8"}`
3. Backend resolves DNS if needed
4. Backend performs longest prefix match
5. Backend identifies interface and gateway
6. Backend sends result: `{"type": "trace_result", "path": [...]}`
7. Frontend highlights path in 3D scene

### Bandwidth Test Flow
1. User clicks edge between nodes A and K
2. Frontend sends: `{"type": "test_bandwidth", "from": "A", "to": "K"}`
3. Backend on A contacts backend on K
4. K starts listening on test port
5. A sends test traffic to K
6. Both measure throughput
7. Results sent back: `{"type": "bandwidth_result", "download": 95, "upload": 87}`
8. Frontend updates edge label

### Public Route Discovery Flow
1. User toggles "Show Public Routes"
2. Frontend requests: `{"type": "traceroute_public", "from": "A", "to": "K"}`
3. Backend A runs traceroute to K's public IP
4. Backend K runs traceroute to A's public IP (for reverse path)
5. Both routes sent to frontend
6. Frontend renders internet hops as new nodes
7. Edges drawn showing complete path

## Security Considerations

### Authentication
- Shared secret/token for node discovery (optional)
- Challenge-response for node joining
- TLS for WebSocket connections (optional)

### Authorization
- Only authenticated nodes can query routing tables
- Rate limiting on expensive operations (bandwidth tests)
- Whitelist/blacklist for node IDs

### Data Privacy
- No sensitive data logged
- Routing tables contain network info only
- Option to exclude certain routes from sharing

## Performance Targets

- Route lookup: <10ms
- Routing table parse: <100ms
- Node discovery: <5 seconds on same subnet
- UI render: 60 FPS with 10 nodes + 100 edges
- WebSocket latency: <50ms
- Background ping overhead: <1% CPU

## Error Handling

- Graceful degradation when routing commands fail
- Retry logic for network operations
- User-friendly error messages
- Fallback to alternative parsing methods
- Connection timeout handling
- Invalid input validation

## Testing Strategy

- Unit tests for route parsing
- Unit tests for longest prefix matching
- Integration tests for API endpoints
- Mock WebSocket for frontend testing
- Cross-platform testing in VMs
- Load testing with many concurrent nodes
