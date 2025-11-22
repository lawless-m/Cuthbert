# Testing Guide

This document provides comprehensive testing procedures for the Network Route Visualizer across different platforms and scenarios.

## Table of Contents
- [Development Testing](#development-testing)
- [Platform-Specific Testing](#platform-specific-testing)
- [VPN Testing](#vpn-testing)
- [Multi-Node Testing](#multi-node-testing)
- [Performance Testing](#performance-testing)
- [Security Testing](#security-testing)

## Development Testing

### Running Unit Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_parse_linux_traceroute

# Run tests in release mode (faster)
cargo test --release
```

### Running Integration Tests

```bash
# Run integration tests only
cargo test --test '*'

# Run with logging
RUST_LOG=debug cargo test
```

### Code Quality Checks

```bash
# Format check
cargo fmt -- --check

# Lint with clippy
cargo clippy -- -D warnings

# Check for common mistakes
cargo clippy --all-targets --all-features

# Audit dependencies for security issues
cargo audit
```

## Platform-Specific Testing

### Linux Testing

#### Tested Distributions
- ✅ Ubuntu 22.04 LTS
- ✅ Ubuntu 24.04 LTS
- ⚠️ Debian 11/12 (should work)
- ⚠️ Fedora 38+ (should work)
- ⚠️ Arch Linux (should work)
- ⚠️ CentOS/RHEL 8+ (should work)

#### Linux Test Checklist

```bash
# 1. Build test
cargo build --release

# 2. Permission test (without sudo)
./target/release/network-route-visualizer
# Expected: Permission error or limited functionality

# 3. Permission test (with sudo)
sudo ./target/release/network-route-visualizer --no-browser
# Expected: Server starts successfully

# 4. Routing table test
curl http://localhost:8080/api/routing-table
# Expected: JSON response with routes

# 5. Traceroute test
curl -X POST http://localhost:8080/api/traceroute \
  -H "Content-Type: application/json" \
  -d '{"destination": "8.8.8.8"}'
# Expected: JSON with hops

# 6. Service test (systemd)
sudo systemctl status network-route-visualizer
# If you've set up a service file

# 7. Firewall test
sudo iptables -L | grep 8080
sudo ufw status | grep 8080
```

#### Linux-Specific Issues to Test
- [ ] Routing table with 500+ routes
- [ ] Multiple network interfaces (eth0, wlan0, tun0)
- [ ] WireGuard VPN routing
- [ ] OpenConnect VPN routing
- [ ] Docker network routes
- [ ] Kubernetes cluster routes
- [ ] IPv6 routing table
- [ ] VLAN interfaces

### macOS Testing

#### Tested Versions
- ✅ macOS 13 (Ventura)
- ✅ macOS 14 (Sonoma)
- ⚠️ macOS 12 (Monterey) (should work)

#### macOS Test Checklist

```bash
# 1. Build for Intel
cargo build --release --target x86_64-apple-darwin

# 2. Build for Apple Silicon
cargo build --release --target aarch64-apple-darwin

# 3. Universal binary (optional)
lipo -create \
  target/x86_64-apple-darwin/release/network-route-visualizer \
  target/aarch64-apple-darwin/release/network-route-visualizer \
  -output network-route-visualizer-universal

# 4. Run with sudo
sudo ./target/release/network-route-visualizer

# 5. Test browser auto-open
# Expected: Safari/default browser opens

# 6. Check Gatekeeper
./network-route-visualizer
# If blocked: System Preferences → Security & Privacy → Allow

# 7. Test with System Integrity Protection
csrutil status
# Should work even with SIP enabled
```

#### macOS-Specific Issues to Test
- [ ] Multiple VPN connections (Tunnelblick, Viscosity)
- [ ] Apple's built-in VPN
- [ ] Network locations switching
- [ ] AirDrop/Bonjour routing
- [ ] VMware Fusion network routes
- [ ] Parallels Desktop routes
- [ ] Docker for Mac routes

### Windows Testing

#### Tested Versions
- ✅ Windows 10 (21H2, 22H2)
- ✅ Windows 11 (22H2, 23H2)
- ⚠️ Windows Server 2019/2022 (should work)

#### Windows Test Checklist

```powershell
# 1. Build
cargo build --release

# 2. Run as regular user
.\target\release\network-route-visualizer.exe
# Expected: May fail without admin rights

# 3. Run as Administrator
# Right-click → Run as Administrator

# 4. Test with Windows Defender
# Check if blocked and add exception if needed

# 5. Test routing table parsing
Invoke-RestMethod http://localhost:8080/api/routing-table

# 6. Test traceroute
Invoke-RestMethod -Method Post http://localhost:8080/api/traceroute `
  -ContentType "application/json" `
  -Body '{"destination": "8.8.8.8"}'

# 7. Test Windows Firewall
netsh advfirewall show allprofiles state
Test-NetConnection localhost -Port 8080
```

#### Windows-Specific Issues to Test
- [ ] Multiple VPN clients (OpenVPN, Cisco AnyConnect)
- [ ] Windows Subsystem for Linux (WSL) routes
- [ ] Hyper-V virtual switch routes
- [ ] VirtualBox host-only network
- [ ] IPv6 routing
- [ ] VPN split tunneling
- [ ] Windows Sandbox network isolation

## VPN Testing

### OpenConnect VPN Testing

```bash
# 1. Start OpenConnect
sudo openconnect --background vpn.example.com

# 2. Verify VPN interface
ip addr show | grep tun
# or on macOS/Windows: check for new interface

# 3. Start visualizer
sudo ./network-route-visualizer

# 4. Check routes show VPN tunnel
curl http://localhost:8080/api/routing-table | jq '.routes[] | select(.interface | contains("tun"))'

# 5. Test traceroute through VPN
curl -X POST http://localhost:8080/api/traceroute \
  -H "Content-Type: application/json" \
  -d '{"destination": "internal.vpn.address"}'

# 6. Disconnect VPN and verify routes update
sudo killall openconnect
# Wait 30s for discovery update
```

### WireGuard VPN Testing

```bash
# 1. Start WireGuard
sudo wg-quick up wg0

# 2. Verify interface
sudo wg show

# 3. Start visualizer
sudo ./network-route-visualizer

# 4. Check WireGuard routes
curl http://localhost:8080/api/routing-table | jq '.routes[] | select(.interface == "wg0")'

# 5. Test bandwidth through VPN
# In UI: Click on WireGuard route → Test Bandwidth

# 6. Compare VPN vs public route
# In UI: Traceroute → Compare routes
```

### Multiple VPN Testing

Test scenario: OpenConnect + WireGuard running simultaneously

```bash
# 1. Start both VPNs
sudo openconnect --background vpn1.example.com
sudo wg-quick up wg0

# 2. Verify routing priority
ip route show | grep default

# 3. Test route selection for different destinations
# Should see different routes used based on metric/priority
```

## Multi-Node Testing

### Two-Node Setup

**Node A** (192.168.1.10):
```bash
sudo ./network-route-visualizer
```

**Node B** (192.168.1.20):
```bash
sudo ./network-route-visualizer
```

**Test Steps**:
1. Wait 60 seconds for discovery
2. Check Node A UI shows Node B
3. Check Node B UI shows Node A
4. Test latency display
5. Test bandwidth test between nodes
6. Disconnect one node, verify other shows it as offline

### Three+ Node Mesh

**Topology**:
```
     Node A (Hub)
      /  |  \
     /   |   \
  Node B  C   D
```

**Test Matrix**:
- [ ] All nodes discover each other
- [ ] Latency shown for all pairs
- [ ] Bandwidth test works for all pairs
- [ ] Routing table differences visible
- [ ] Node failure detection (unplug one node)
- [ ] Node recovery (plug back in)
- [ ] Network partition handling

### Cross-Subnet Testing

**Scenario**: Nodes on different VLANs communicating through VPN

**Setup**:
- Node A: 192.168.1.10 (VLAN 1)
- Node B: 192.168.2.20 (VLAN 2)
- VPN: Connects both VLANs

**Test**:
1. Verify routes through VPN
2. Check latency
3. Test bandwidth
4. Compare VPN vs. internet route

## Performance Testing

### Routing Table Scale Test

```bash
# Generate large routing table (Linux)
for i in {1..500}; do
  sudo ip route add 10.$((i/256)).$((i%256)).0/24 via 192.168.1.1
done

# Start visualizer
sudo ./network-route-visualizer

# Measure:
# - Startup time
# - Memory usage
# - UI responsiveness
# - Route lookup time
```

### Concurrent User Testing

```bash
# Run Apache Bench
ab -n 1000 -c 10 http://localhost:8080/api/routing-table

# Run WebSocket connection test
# Use websocket-bench or similar tool
```

### Memory Leak Testing

```bash
# Run for extended period
sudo ./network-route-visualizer &

# Monitor memory
while true; do
  ps aux | grep network-route-visualizer | grep -v grep
  sleep 60
done

# Should show stable memory usage over 24 hours
```

## Security Testing

### Privilege Escalation Test

```bash
# Should NOT allow reading arbitrary files
curl http://localhost:8080/api/../../../etc/passwd
# Expected: 404 or error

# Should NOT execute arbitrary commands
curl -X POST http://localhost:8080/api/traceroute \
  -H "Content-Type: application/json" \
  -d '{"destination": "8.8.8.8; cat /etc/passwd"}'
# Expected: Command injection prevented
```

### Input Validation Testing

```bash
# Test various invalid inputs
curl -X POST http://localhost:8080/api/trace-route \
  -H "Content-Type: application/json" \
  -d '{"destination": ""}'

curl -X POST http://localhost:8080/api/trace-route \
  -H "Content-Type: application/json" \
  -d '{"destination": "../../etc/passwd"}'

curl -X POST http://localhost:8080/api/traceroute \
  -H "Content-Type: application/json" \
  -d '{"destination": "$(whoami)"}'

# All should return appropriate error messages
```

### Port Scanning Prevention

```bash
# Tool should not be usable for port scanning
# Traceroute should only work for network layer, not port scanning
```

## Automated Testing Script

```bash
#!/bin/bash
# test-all.sh

set -e

echo "Running all tests..."

# Unit tests
echo "1. Unit tests"
cargo test

# Build
echo "2. Building release"
cargo build --release

# Lint
echo "3. Linting"
cargo clippy -- -D warnings

# Format check
echo "4. Format check"
cargo fmt -- --check

# Run server (background)
echo "5. Starting server"
sudo ./target/release/network-route-visualizer --no-browser &
SERVER_PID=$!
sleep 5

# API tests
echo "6. Testing API endpoints"
curl -f http://localhost:8080/api/routing-table || exit 1
curl -f http://localhost:8080/ || exit 1

echo "7. Testing traceroute"
curl -f -X POST http://localhost:8080/api/traceroute \
  -H "Content-Type: application/json" \
  -d '{"destination": "8.8.8.8"}' || exit 1

# Cleanup
echo "8. Cleanup"
sudo kill $SERVER_PID

echo "✓ All tests passed!"
```

## Test Report Template

When reporting test results, use this template:

```markdown
## Test Environment
- OS: Ubuntu 22.04 LTS
- Rust Version: 1.75.0
- VPN: OpenConnect 9.01
- Kernel: 5.15.0

## Test Results
- [ ] Build: ✅ PASS
- [ ] Unit Tests: ✅ PASS (45/45)
- [ ] Routing Table Parse: ✅ PASS
- [ ] Traceroute: ✅ PASS
- [ ] Multi-Node Discovery: ✅ PASS
- [ ] VPN Route Detection: ✅ PASS
- [ ] Performance: ✅ PASS (500 routes, <100ms)
- [ ] 24h Stability: ✅ PASS (no memory leak)

## Issues Found
1. [None / List any issues]

## Notes
[Any additional observations]
```

## Continuous Integration

The project uses GitHub Actions for automated testing. See `.github/workflows/ci.yml` for details.

Tests run on:
- Ubuntu Latest
- Windows Latest
- macOS Latest

Tests include:
- Compilation
- Unit tests
- Clippy linting
- Format checking
- Release builds

## Manual Test Checklist

Before each release, manually verify:

- [ ] All unit tests pass on all platforms
- [ ] UI loads in all major browsers
- [ ] VPN routes detected correctly
- [ ] Multi-node discovery works
- [ ] Bandwidth tests complete
- [ ] Traceroute works on all platforms
- [ ] Configuration files load correctly
- [ ] CLI arguments work as expected
- [ ] No memory leaks in 24h test
- [ ] Documentation is up to date
- [ ] Example config file is valid
- [ ] CHANGELOG.md updated
