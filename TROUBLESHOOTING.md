# Troubleshooting Guide

This guide helps you resolve common issues with the Network Route Visualizer.

## Table of Contents
- [Installation Issues](#installation-issues)
- [Permission Errors](#permission-errors)
- [Discovery Problems](#discovery-problems)
- [Traceroute Failures](#traceroute-failures)
- [Performance Issues](#performance-issues)
- [Network Errors](#network-errors)

## Installation Issues

### Rust Build Fails

**Problem**: `cargo build` fails with compilation errors

**Solutions**:
1. Ensure you have Rust 1.70 or later:
   ```bash
   rustc --version
   ```

2. Update Rust toolchain:
   ```bash
   rustup update stable
   ```

3. Clean build cache and retry:
   ```bash
   cargo clean
   cargo build --release
   ```

### Missing System Dependencies

**Problem**: Build fails due to missing system libraries

**Linux Solutions**:
```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libssl-dev

# Fedora/RHEL
sudo dnf install gcc pkg-config openssl-devel

# Arch
sudo pacman -S base-devel openssl
```

**macOS Solutions**:
```bash
xcode-select --install
brew install openssl
```

## Permission Errors

### Cannot Read Routing Table

**Problem**: `Failed to parse routing table` or `Permission denied`

**Solutions**:

**Linux/macOS**:
```bash
# Run with sudo
sudo ./network-route-visualizer

# OR grant capabilities (Linux only)
sudo setcap cap_net_admin=eip network-route-visualizer
./network-route-visualizer
```

**Windows**:
- Right-click the executable
- Select "Run as Administrator"

### Port Already in Use

**Problem**: `Failed to bind to 127.0.0.1:8080: Address already in use`

**Solutions**:

1. **Use a different port**:
   ```bash
   ./network-route-visualizer --port 8081
   ```

2. **Find and stop the conflicting process**:

   Linux/macOS:
   ```bash
   lsof -i :8080
   kill -9 <PID>
   ```

   Windows:
   ```powershell
   netstat -ano | findstr :8080
   taskkill /PID <PID> /F
   ```

## Discovery Problems

### Nodes Not Discovering Each Other

**Problem**: Multiple instances running but nodes don't see each other

**Possible Causes & Solutions**:

1. **Firewall blocking multicast**:
   ```bash
   # Linux - allow multicast group
   sudo iptables -A INPUT -d 239.255.42.1 -j ACCEPT

   # macOS - check firewall settings
   sudo /usr/libexec/ApplicationFirewall/socketfilterfw --listapps

   # Windows - allow through Windows Firewall
   # Settings > Network & Internet > Windows Firewall
   ```

2. **Different network segments**:
   - Multicast doesn't cross routers by default
   - Nodes must be on the same subnet
   - Check with: `ip addr` (Linux) or `ipconfig` (Windows)

3. **VPN interfering**:
   - Some VPNs block multicast traffic
   - Try disabling VPN temporarily to test
   - Use `--multicast-interface` if your VPN supports it

4. **Discovery disabled**:
   ```bash
   # Check if discovery is enabled
   ./network-route-visualizer --help

   # Ensure you're not using --no-discovery
   ```

### Nodes Shown as Offline

**Problem**: Discovered nodes appear but show as offline

**Solutions**:

1. **Check network connectivity**:
   ```bash
   ping <node-ip>
   ```

2. **Firewall blocking HTTP/WebSocket**:
   ```bash
   # Ensure port 8080 is open
   telnet <node-ip> 8080
   ```

3. **Increase peer timeout**:
   ```bash
   ./network-route-visualizer --peer-timeout 180
   ```

## Traceroute Failures

### Traceroute Command Not Found

**Problem**: `traceroute failed: Command not found`

**Solutions**:

**Linux**:
```bash
# Ubuntu/Debian
sudo apt-get install traceroute

# Fedora/RHEL
sudo dnf install traceroute

# Arch
sudo pacman -S traceroute
```

**macOS**:
- traceroute is pre-installed
- Ensure you have admin rights

**Windows**:
- tracert is pre-installed
- Run as Administrator

### All Hops Show as Timeouts

**Problem**: Traceroute shows `* * *` for all hops

**Causes**:
1. **ICMP blocked**: Router/firewall blocks ICMP packets
2. **No internet connection**: Check basic connectivity
3. **Destination unreachable**: Target may be down or blocking

**Solutions**:
```bash
# Test basic connectivity first
ping 8.8.8.8

# Try UDP traceroute (Linux)
sudo traceroute -U 8.8.8.8

# Try TCP traceroute (Linux, requires tcptraceroute)
sudo tcptraceroute 8.8.8.8

# Windows: use pathping for more reliable results
pathping 8.8.8.8
```

## Performance Issues

### High CPU Usage

**Problem**: Application uses excessive CPU

**Solutions**:

1. **Reduce ping frequency**:
   ```bash
   ./network-route-visualizer --ping-interval 120
   ```

2. **Disable automatic ping**:
   ```bash
   ./network-route-visualizer --no-ping
   ```

3. **Limit node count**:
   - Only run on nodes that need monitoring
   - Use `--no-discovery` on passive nodes

### Slow 3D Visualization

**Problem**: Web UI is laggy or unresponsive

**Solutions**:

1. **Use a modern browser**:
   - Chrome 90+
   - Firefox 88+
   - Safari 14+
   - Edge 90+

2. **Enable hardware acceleration**:
   - Chrome: `chrome://settings` → Advanced → System → "Use hardware acceleration"
   - Firefox: `about:preferences` → General → Performance

3. **Reduce routing table size**:
   - Large routing tables (500+ routes) may cause slowness
   - Filter unnecessary routes in your OS

4. **Close other browser tabs**:
   - Three.js 3D rendering is resource-intensive

## Network Errors

### WebSocket Connection Failed

**Problem**: "WebSocket connection error" in browser console

**Solutions**:

1. **Check server is running**:
   ```bash
   curl http://localhost:8080/api/routing-table
   ```

2. **Browser blocking WebSocket**:
   - Disable browser extensions (especially ad blockers)
   - Try incognito/private mode

3. **Proxy or reverse proxy issues**:
   - If behind nginx/apache, configure WebSocket proxying:
   ```nginx
   location /ws {
       proxy_pass http://localhost:8080;
       proxy_http_version 1.1;
       proxy_set_header Upgrade $http_upgrade;
       proxy_set_header Connection "upgrade";
   }
   ```

### Cannot Access from Other Machines

**Problem**: Can access on localhost but not from other machines

**Solutions**:

1. **Bind to all interfaces**:
   ```toml
   # config.toml
   [server]
   bind_address = "0.0.0.0"
   ```

2. **Open firewall ports**:
   ```bash
   # Linux (iptables)
   sudo iptables -A INPUT -p tcp --dport 8080 -j ACCEPT

   # Linux (firewalld)
   sudo firewall-cmd --add-port=8080/tcp --permanent
   sudo firewall-cmd --reload

   # macOS
   # System Preferences → Security & Privacy → Firewall → Firewall Options

   # Windows
   netsh advfirewall firewall add rule name="Network Route Visualizer" dir=in action=allow protocol=TCP localport=8080
   ```

## Getting More Help

### Enable Debug Logging

```bash
# Set log level to debug
./network-route-visualizer --log-level debug

# Or use environment variable
RUST_LOG=debug ./network-route-visualizer

# Trace level for maximum verbosity
./network-route-visualizer --log-level trace
```

### Report Issues

If you can't resolve your issue:

1. **Check existing issues**: [GitHub Issues](https://github.com/yourusername/network-route-visualizer/issues)

2. **Create a new issue** with:
   - Operating system and version
   - Rust version (`rustc --version`)
   - Full error message
   - Debug log output
   - Steps to reproduce

3. **Include system information**:
   ```bash
   # Linux
   uname -a
   ip route show

   # macOS
   sw_vers
   netstat -rn

   # Windows
   systeminfo
   route print
   ```

## Common Configuration Mistakes

### Configuration File Not Loaded

**Problem**: Changes to `config.toml` have no effect

**Solutions**:

1. **File in wrong location**:
   ```bash
   # config.toml must be in the same directory as the binary
   ls -l config.toml
   ```

2. **Specify config file explicitly**:
   ```bash
   ./network-route-visualizer --config /path/to/config.toml
   ```

3. **TOML syntax error**:
   ```bash
   # Validate TOML syntax
   # Use online tool: https://www.toml-lint.com/
   ```

4. **CLI args override config file**:
   - Command-line arguments take precedence
   - Remove conflicting CLI flags

### Invalid Configuration Values

**Problem**: "Configuration error" on startup

**Solutions**:

1. **Check log level**:
   ```toml
   [logging]
   level = "info"  # Must be: error, warn, info, debug, or trace
   ```

2. **Validate port numbers**:
   ```toml
   [server]
   port = 8080  # Must be 1-65535
   ```

3. **Check multicast IP**:
   ```toml
   [discovery]
   multicast_group = "239.255.42.1"  # Must be valid multicast (239.0.0.0/8)
   ```

## Platform-Specific Issues

### Linux: Routing Table Empty

**Problem**: No routes shown even with sudo

**Solution**:
```bash
# Check if routes exist
ip route show

# Try different route command
sudo ./network-route-visualizer
```

### macOS: Permission Denied on Traceroute

**Problem**: Traceroute fails even with sudo

**Solution**:
```bash
# macOS traceroute requires special permissions
sudo chmod u+s /usr/sbin/traceroute
```

### Windows: Application Crashes on Startup

**Problem**: Immediate crash with no error message

**Solutions**:

1. **Run from Command Prompt** to see errors:
   ```cmd
   cd C:\path\to\binary
   network-route-visualizer.exe
   ```

2. **Install Visual C++ Redistributable**:
   - Download from Microsoft
   - Install for x64 architecture

3. **Check antivirus**:
   - Some antivirus software blocks network tools
   - Add exception for network-route-visualizer.exe
