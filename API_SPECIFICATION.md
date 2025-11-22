# API Specification

## Overview

The Network Route Visualizer exposes both REST and WebSocket APIs for communication between the frontend and backend, as well as between distributed nodes.

**Base URL**: `http://localhost:8080` (configurable)

---

## REST API Endpoints

### Local Machine Information

#### GET /api/routing-table
Get the current machine's routing table.

**Response**: 200 OK
```json
{
  "hostname": "machine-a",
  "routes": [
    {
      "destination": "0.0.0.0/0",
      "gateway": "192.168.1.1",
      "interface": "eth0",
      "metric": 100,
      "flags": ["U", "G"]
    },
    {
      "destination": "10.20.0.0/24",
      "gateway": null,
      "interface": "tun0",
      "metric": 50,
      "flags": ["U"]
    }
  ],
  "timestamp": "2025-11-22T10:30:00Z"
}
```

#### GET /api/interfaces
Get network interfaces.

**Response**: 200 OK
```json
{
  "interfaces": [
    {
      "name": "eth0",
      "addresses": ["192.168.1.100"],
      "mac": "00:11:22:33:44:55",
      "status": "up",
      "type": "ethernet"
    },
    {
      "name": "tun0",
      "addresses": ["10.20.0.5"],
      "mac": null,
      "status": "up",
      "type": "vpn"
    }
  ]
}
```

#### POST /api/trace-route
Trace the route to a destination.

**Request Body**:
```json
{
  "destination": "8.8.8.8"
}
```

**Response**: 200 OK
```json
{
  "destination": "8.8.8.8",
  "resolved_ip": "8.8.8.8",
  "matched_route": {
    "destination": "0.0.0.0/0",
    "gateway": "192.168.1.1",
    "interface": "eth0",
    "metric": 100
  },
  "path": [
    {
      "step": 1,
      "type": "local_interface",
      "interface": "eth0",
      "ip": "192.168.1.100"
    },
    {
      "step": 2,
      "type": "gateway",
      "ip": "192.168.1.1"
    },
    {
      "step": 3,
      "type": "destination",
      "ip": "8.8.8.8"
    }
  ]
}
```

**Error Response**: 404 Not Found
```json
{
  "error": "NoRouteToHost",
  "message": "No route found to 10.99.99.99",
  "suggestions": [
    "Check if the destination network is reachable",
    "Verify VPN connections are active"
  ]
}
```

---

### Node Discovery

#### GET /api/nodes
List all discovered nodes.

**Response**: 200 OK
```json
{
  "nodes": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "hostname": "machine-b",
      "addresses": ["10.20.0.10", "192.168.1.50"],
      "port": 8080,
      "status": "online",
      "latency_ms": 12,
      "last_seen": "2025-11-22T10:29:00Z",
      "discovered_via": "broadcast"
    }
  ],
  "local_node_id": "450e8400-e29b-41d4-a716-446655440000"
}
```

#### GET /api/nodes/{node_id}
Get details for a specific node.

**Response**: 200 OK
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "hostname": "machine-b",
  "addresses": ["10.20.0.10"],
  "port": 8080,
  "status": "online",
  "latency_ms": 12,
  "last_seen": "2025-11-22T10:29:00Z",
  "capabilities": ["routing-table", "bandwidth-test", "traceroute"],
  "uptime_seconds": 3600,
  "version": "0.1.0"
}
```

#### GET /api/nodes/{node_id}/routing-table
Get routing table from a remote node.

**Response**: 200 OK (same format as GET /api/routing-table)

**Error Response**: 503 Service Unavailable
```json
{
  "error": "NodeUnreachable",
  "message": "Cannot connect to node machine-b",
  "node_status": "offline"
}
```

---

### Performance Testing

#### POST /api/test/ping
Ping a target (node or IP).

**Request Body**:
```json
{
  "target": "550e8400-e29b-41d4-a716-446655440000",
  "count": 10
}
```

**Response**: 200 OK
```json
{
  "target": "machine-b",
  "target_ip": "10.20.0.10",
  "packets_sent": 10,
  "packets_received": 10,
  "packet_loss_percent": 0,
  "latency_ms": {
    "min": 10.2,
    "max": 15.8,
    "avg": 12.5,
    "stddev": 1.8
  }
}
```

#### POST /api/test/bandwidth
Test bandwidth to a target node.

**Request Body**:
```json
{
  "target": "550e8400-e29b-41d4-a716-446655440000",
  "duration_seconds": 10,
  "direction": "both"
}
```

**Note**: This initiates a test. Progress updates are sent via WebSocket.

**Response**: 202 Accepted
```json
{
  "test_id": "test-12345",
  "status": "initiated",
  "estimated_duration_seconds": 10
}
```

**WebSocket Updates** (during test):
```json
{
  "type": "bandwidth_test_progress",
  "test_id": "test-12345",
  "progress_percent": 50,
  "current_throughput_mbps": 92
}
```

**WebSocket Final Result**:
```json
{
  "type": "bandwidth_test_complete",
  "test_id": "test-12345",
  "from_node": "machine-a",
  "to_node": "machine-b",
  "download_mbps": 95.2,
  "upload_mbps": 87.5,
  "test_duration_seconds": 10,
  "timestamp": "2025-11-22T10:35:00Z"
}
```

#### POST /api/test/traceroute
Run traceroute to a destination.

**Request Body**:
```json
{
  "destination": "8.8.8.8",
  "use_public_internet": true,
  "max_hops": 30,
  "timeout_seconds": 2
}
```

**Response**: 200 OK
```json
{
  "destination": "8.8.8.8",
  "hops": [
    {
      "hop_number": 1,
      "ip": "192.168.1.1",
      "hostname": "gateway.local",
      "latency_ms": [1.2, 1.1, 1.3],
      "avg_latency_ms": 1.2
    },
    {
      "hop_number": 2,
      "ip": "10.50.1.1",
      "hostname": null,
      "latency_ms": [5.4, 5.6, 5.5],
      "avg_latency_ms": 5.5
    },
    {
      "hop_number": 3,
      "ip": "*",
      "hostname": null,
      "latency_ms": [],
      "avg_latency_ms": null
    }
  ],
  "total_hops": 15,
  "completed": true
}
```

---

### Connectivity Diagnosis

#### POST /api/diagnose
Diagnose connectivity to a target.

**Request Body**:
```json
{
  "target": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Response**: 200 OK
```json
{
  "target_node": "machine-k",
  "reachable": false,
  "known_via_gossip": true,
  "diagnosis": {
    "issue_type": "NoRoute",
    "description": "No route exists to 10.20.0.15",
    "suggested_fixes": [
      {
        "platform": "linux",
        "description": "Add route via gateway",
        "command": "sudo ip route add 10.20.0.0/24 via 10.50.0.1 dev tun0"
      },
      {
        "platform": "windows",
        "description": "Add route via gateway",
        "command": "route add 10.20.0.0 mask 255.255.0.0 10.50.0.1"
      }
    ],
    "alternative_paths": [
      {
        "description": "Route through machine-b",
        "hops": ["machine-a", "machine-b", "machine-k"],
        "total_latency_ms": 45
      }
    ]
  }
}
```

---

## WebSocket API

### Connection
**Endpoint**: `ws://localhost:8080/ws`

**Protocol**: JSON messages in both directions

---

### Client → Server Messages

#### Request Trace Route
```json
{
  "type": "trace_route",
  "request_id": "req-123",
  "destination": "8.8.8.8"
}
```

#### Request Bandwidth Test
```json
{
  "type": "test_bandwidth",
  "request_id": "req-124",
  "target_node": "550e8400-e29b-41d4-a716-446655440000",
  "duration_seconds": 10
}
```

#### Subscribe to Updates
```json
{
  "type": "subscribe",
  "topics": ["nodes", "latency", "routes"]
}
```

#### Request Remote Routing Table
```json
{
  "type": "get_remote_routing_table",
  "request_id": "req-125",
  "node_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

---

### Server → Client Messages

#### Node Discovery
```json
{
  "type": "node_discovered",
  "node": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "hostname": "machine-c",
    "addresses": ["10.20.0.15"],
    "port": 8080,
    "status": "online"
  }
}
```

#### Node Status Update
```json
{
  "type": "node_status_changed",
  "node_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "offline",
  "reason": "timeout"
}
```

#### Latency Update
```json
{
  "type": "latency_update",
  "connections": [
    {
      "from": "450e8400-e29b-41d4-a716-446655440000",
      "to": "550e8400-e29b-41d4-a716-446655440000",
      "latency_ms": 12,
      "timestamp": "2025-11-22T10:30:00Z"
    }
  ]
}
```

#### Route Table Changed
```json
{
  "type": "routing_table_changed",
  "node_id": "450e8400-e29b-41d4-a716-446655440000",
  "routes_added": [
    {
      "destination": "10.30.0.0/24",
      "gateway": "10.20.0.1",
      "interface": "wg0",
      "metric": 0
    }
  ],
  "routes_removed": []
}
```

#### Trace Route Result
```json
{
  "type": "trace_route_result",
  "request_id": "req-123",
  "destination": "8.8.8.8",
  "path": [
    {"step": 1, "type": "local_interface", "interface": "eth0"},
    {"step": 2, "type": "gateway", "ip": "192.168.1.1"},
    {"step": 3, "type": "destination", "ip": "8.8.8.8"}
  ]
}
```

#### Bandwidth Test Progress
```json
{
  "type": "bandwidth_test_progress",
  "request_id": "req-124",
  "progress_percent": 50,
  "current_throughput_mbps": 92
}
```

#### Bandwidth Test Complete
```json
{
  "type": "bandwidth_test_complete",
  "request_id": "req-124",
  "from_node": "machine-a",
  "to_node": "machine-b",
  "download_mbps": 95.2,
  "upload_mbps": 87.5
}
```

#### Error Message
```json
{
  "type": "error",
  "request_id": "req-123",
  "error_code": "NodeUnreachable",
  "message": "Cannot connect to node machine-k"
}
```

---

## Node-to-Node Communication

Nodes communicate with each other using the same REST API, but with authentication.

### Authentication
**Header**: `X-Node-Auth: <shared-secret>`

Or use TLS client certificates (optional).

---

### Discovery Messages (UDP Multicast)

#### Announcement
**Multicast Address**: 239.255.42.1:5678
**Frequency**: Every 30 seconds

```json
{
  "type": "announce",
  "node_id": "450e8400-e29b-41d4-a716-446655440000",
  "hostname": "machine-a",
  "addresses": ["10.20.0.5", "192.168.1.100"],
  "port": 8080,
  "timestamp": "2025-11-22T10:30:00Z",
  "version": "0.1.0",
  "known_peers": [
    "550e8400-e29b-41d4-a716-446655440000",
    "650e8400-e29b-41d4-a716-446655440000"
  ]
}
```

#### Goodbye (Graceful Shutdown)
```json
{
  "type": "goodbye",
  "node_id": "450e8400-e29b-41d4-a716-446655440000",
  "reason": "shutdown"
}
```

---

## Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `InvalidDestination` | 400 | Destination IP or hostname is invalid |
| `NoRouteToHost` | 404 | No route found to destination |
| `NodeNotFound` | 404 | Specified node ID does not exist |
| `NodeUnreachable` | 503 | Cannot connect to remote node |
| `TestInProgress` | 409 | Another test is already running |
| `PermissionDenied` | 403 | Insufficient privileges to execute command |
| `PlatformNotSupported` | 501 | Feature not available on this platform |
| `InternalError` | 500 | Unexpected server error |

---

## Rate Limiting

To prevent abuse and resource exhaustion:

- **Bandwidth tests**: Maximum 1 concurrent test per node pair
- **Traceroute**: Maximum 5 per minute per client
- **Routing table queries**: Maximum 10 per minute per remote node
- **WebSocket messages**: Maximum 100 per second per connection

Rate limit responses:
```json
{
  "error": "RateLimitExceeded",
  "message": "Too many requests",
  "retry_after_seconds": 30
}
```

---

## Versioning

API version is included in responses:
```json
{
  "api_version": "1.0",
  "data": { ... }
}
```

Breaking changes will increment the major version and may require a new endpoint path: `/api/v2/...`

---

## Examples

### Complete Workflow: Route Tracing

1. **Connect WebSocket**:
   ```javascript
   const ws = new WebSocket('ws://localhost:8080/ws');
   ```

2. **Subscribe to updates**:
   ```javascript
   ws.send(JSON.stringify({
     type: 'subscribe',
     topics: ['routes', 'latency']
   }));
   ```

3. **Request trace route**:
   ```javascript
   ws.send(JSON.stringify({
     type: 'trace_route',
     request_id: 'req-123',
     destination: 'google.com'
   }));
   ```

4. **Receive result**:
   ```javascript
   ws.onmessage = (event) => {
     const msg = JSON.parse(event.data);
     if (msg.type === 'trace_route_result' && msg.request_id === 'req-123') {
       // Highlight path in 3D scene
       highlightPath(msg.path);
     }
   };
   ```

### Complete Workflow: Bandwidth Test

1. **Click edge in UI** between machine-a and machine-k

2. **Send test request**:
   ```javascript
   ws.send(JSON.stringify({
     type: 'test_bandwidth',
     request_id: 'req-124',
     target_node: 'machine-k-id',
     duration_seconds: 10
   }));
   ```

3. **Receive progress updates**:
   ```javascript
   ws.onmessage = (event) => {
     const msg = JSON.parse(event.data);
     if (msg.type === 'bandwidth_test_progress') {
       updateProgressBar(msg.progress_percent);
     } else if (msg.type === 'bandwidth_test_complete') {
       displayResults(msg.download_mbps, msg.upload_mbps);
       updateEdgeLabel(msg);
     }
   };
   ```

---

## Security Considerations

### Authentication
- Optional shared secret for node-to-node communication
- WebSocket connections from localhost only (by default)
- Option to enable remote WebSocket with authentication

### Data Validation
- All inputs sanitized to prevent command injection
- IP addresses validated
- Node IDs validated (UUID format)
- Rate limiting enforced

### Privilege Separation
- Read-only operations don't require elevated privileges
- Routing table modifications not supported (security)
- Test operations validated before execution
