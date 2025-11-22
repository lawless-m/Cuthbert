# Network Route Visualizer - Development Guide

## Quick Start for Claude Code

This project implements a cross-platform network topology visualization tool in Rust with a three.js frontend. You have four planning documents to guide implementation:

### 📋 Planning Documents

1. **PROJECT_OVERVIEW.md** - Start here to understand what we're building and why
2. **TECHNICAL_ARCHITECTURE.md** - System design, data structures, and component details
3. **IMPLEMENTATION_ROADMAP.md** - Phased development plan with milestones
4. **API_SPECIFICATION.md** - Complete API documentation with examples

### 🎯 Implementation Strategy

**Follow the phased approach in IMPLEMENTATION_ROADMAP.md:**

#### Phase 1: Core Foundation (Start Here)
Build the MVP - single machine visualization with route tracing:
- Set up Rust project with axum web framework
- Implement routing table parser for Linux (then Windows/macOS)
- Create route lookup engine (longest prefix matching)
- Build basic three.js frontend
- Implement route tracing UI

**Goal**: Get something working quickly that you can iterate on.

#### Phase 2-6: Build Out Features
Once Phase 1 works, add:
- Enhanced 3D visualization
- Multi-node discovery mesh
- Performance testing (ping, bandwidth)
- Public internet routes with traceroute
- Polish and cross-platform support

### 🏗️ Project Structure

Create this structure:
```
network-route-visualizer/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── routes/
│   │   ├── mod.rs
│   │   ├── parser.rs      # Platform-specific route parsing
│   │   └── lookup.rs      # Longest prefix matching
│   ├── discovery/
│   │   ├── mod.rs
│   │   ├── broadcast.rs   # UDP multicast
│   │   └── gossip.rs      # Peer sharing
│   ├── testing/
│   │   ├── mod.rs
│   │   ├── ping.rs
│   │   └── bandwidth.rs
│   ├── api/
│   │   ├── mod.rs
│   │   ├── rest.rs        # REST endpoints
│   │   └── websocket.rs   # WebSocket handler
│   └── web/
│       └── static/
│           ├── index.html
│           ├── app.js     # three.js application
│           └── styles.css
└── tests/
    ├── integration_tests.rs
    └── fixtures/
```

### 🔧 Key Technologies

Refer to TECHNICAL_ARCHITECTURE.md for detailed crate choices, but here are the essentials:

**Rust Backend:**
- `axum` - Web framework
- `tokio` - Async runtime
- `serde` / `serde_json` - Serialization
- `tower-http` - Static file serving

**Frontend:**
- `three.js` - 3D visualization
- WebSocket API - Real-time updates
- Vanilla JavaScript (no framework needed)

### 📝 Development Notes

**Platform Detection:**
Use conditional compilation:
```rust
#[cfg(target_os = "linux")]
fn parse_routes() -> Result<Vec<Route>> { ... }

#[cfg(target_os = "windows")]  
fn parse_routes() -> Result<Vec<Route>> { ... }
```

**Route Parsing:**
- Linux: `ip -json route show` (preferred) or parse `/proc/net/route`
- Windows: `route print` or PowerShell `Get-NetRoute | ConvertTo-Json`
- macOS: `netstat -rn`

See TECHNICAL_ARCHITECTURE.md for detailed parsing logic.

**Longest Prefix Matching:**
Implement a binary prefix tree (trie) for efficient route lookups. Critical for performance.

**WebSocket Communication:**
Use JSON messages for all WebSocket communication. See API_SPECIFICATION.md for complete message formats.

### 🧪 Testing Strategy

1. **Unit Tests First**: Test route parsing with real routing table examples
2. **Integration Tests**: Test API endpoints
3. **Manual Testing**: Visual verification of 3D scene
4. **Cross-Platform**: Test on Linux/Windows/macOS VMs

### ⚠️ Important Considerations

**Permissions:**
- Reading routing tables requires elevated privileges (root/admin)
- Document this requirement clearly
- Consider using capabilities on Linux to avoid full root

**Error Handling:**
- Graceful degradation when commands fail
- User-friendly error messages
- Comprehensive logging with `tracing` crate

**Cross-Platform:**
- Test early and often on all platforms
- Don't assume routing table formats
- VPN interface names vary wildly

### 🎨 UI/UX Guidelines

**3D Visualization:**
- Local machine: Large, prominent, centered
- VPN connections: Clearly labeled edges
- Color coding: Green (good) → Yellow (ok) → Red (poor)
- Interactive: Click nodes/edges for details
- Smooth camera transitions

**Route Tracing:**
- Highlight path with animation
- Show interface names and gateway IPs
- Make it obvious which VPN is being used

**Connectivity Issues:**
- Show clear error states (grayed nodes)
- Provide copy-pasteable fix commands
- Suggest alternative routes

### 🚀 Deployment

**Single Binary:**
- Embed all static files using `include_str!` or similar
- Cross-compile for Linux, Windows, macOS
- Auto-open browser on startup (optional flag)

**Command-Line Options:**
```
--port <PORT>           Web server port (default: 8080)
--no-browser           Don't auto-open browser
--log-level <LEVEL>    Logging verbosity
--config <PATH>        Config file path
```

### 📚 Reference the Docs

**When implementing:**
- Route parsing → TECHNICAL_ARCHITECTURE.md "Platform-Specific Implementation"
- API endpoints → API_SPECIFICATION.md "REST API Endpoints"
- Discovery protocol → TECHNICAL_ARCHITECTURE.md "Discovery Protocol"
- Data structures → TECHNICAL_ARCHITECTURE.md "Data Structures"
- Milestones → IMPLEMENTATION_ROADMAP.md for detailed checklists

### ✅ Definition of Done

Phase 1 is complete when:
- [ ] Can parse routing table on at least Linux
- [ ] Can trace route to any destination
- [ ] 3D visualization displays routes
- [ ] Path highlights when destination entered
- [ ] Basic error handling works
- [ ] Runs as single binary

Then move to Phase 2!

### 💡 Tips for Success

1. **Start simple**: Get Phase 1 working before adding complexity
2. **Test incrementally**: Don't write too much before testing
3. **Use logging**: Add tracing early for debugging
4. **Refer to specs**: Check API_SPECIFICATION.md for exact message formats
5. **Follow the roadmap**: It's designed to build working software at each phase

### 🆘 Common Pitfalls

- **Route parsing**: Formats vary wildly, handle edge cases
- **UDP multicast**: May not cross all VPN types, have fallback
- **Three.js performance**: Use efficient geometries, avoid creating too many objects
- **Async Rust**: Keep boundaries clear, use structured concurrency
- **CIDR matching**: Implement carefully, test thoroughly

---

## Getting Started Command

```bash
# Initialize the project
cargo new network-route-visualizer
cd network-route-visualizer

# Add initial dependencies to Cargo.toml (see TECHNICAL_ARCHITECTURE.md)

# Start with Phase 1, Milestone 1.1 from IMPLEMENTATION_ROADMAP.md
```

Good luck! Build Phase 1 first, get it working, then iterate. The planning docs have everything you need.
