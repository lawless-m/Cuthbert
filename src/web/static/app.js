// Network Route Visualizer - Enhanced Three.js Frontend (Phase 3)

class RouteVisualizer {
    constructor() {
        this.scene = null;
        this.camera = null;
        this.renderer = null;
        this.labelRenderer = null;
        this.controls = null;
        this.routingTable = null;
        this.nodes = new Map();
        this.edges = [];
        this.labels = new Map();
        this.highlightedObjects = [];
        this.raycaster = new THREE.Raycaster();
        this.mouse = new THREE.Vector2();
        this.hoveredObject = null;
        this.selectedObject = null;
        this.fps = 0;
        this.frameCount = 0;
        this.lastTime = performance.now();

        // WebSocket and discovery
        this.ws = null;
        this.wsReconnectAttempts = 0;
        this.wsMaxReconnectAttempts = 5;
        this.discoveredNodes = new Map();
        this.localNodeId = null;
        this.latencyData = new Map(); // nodeId -> latency_ms
        this.bandwidthTests = new Map(); // testId -> test data
        this.bandwidthResults = new Map(); // nodeId -> result
        this.tracerouteData = null; // Current traceroute result
        this.tracerouteHops = new Map(); // hop IP -> mesh object
        this.tracerouteEdges = []; // Edges for traceroute path
        this.showPublicRoutes = false; // Toggle for public route visualization

        this.init();
        this.setupEventListeners();
        this.connectWebSocket();
        this.loadRoutingTable();
        this.loadDiscoveredNodes();
    }

    init() {
        const container = document.getElementById('canvas-container');

        // Create scene
        this.scene = new THREE.Scene();
        this.scene.background = new THREE.Color(0x0a0a0a);
        this.scene.fog = new THREE.Fog(0x0a0a0a, 50, 100);

        // Create camera
        this.camera = new THREE.PerspectiveCamera(
            75,
            container.clientWidth / container.clientHeight,
            0.1,
            1000
        );
        this.camera.position.set(0, 20, 35);

        // Create WebGL renderer
        this.renderer = new THREE.WebGLRenderer({
            canvas: document.getElementById('three-canvas'),
            antialias: true
        });
        this.renderer.setSize(container.clientWidth, container.clientHeight);
        this.renderer.setPixelRatio(window.devicePixelRatio);

        // Create CSS2D renderer for labels
        this.labelRenderer = new THREE.CSS2DRenderer();
        this.labelRenderer.setSize(container.clientWidth, container.clientHeight);
        this.labelRenderer.domElement.style.position = 'absolute';
        this.labelRenderer.domElement.style.top = '0';
        this.labelRenderer.domElement.style.pointerEvents = 'none';
        container.appendChild(this.labelRenderer.domElement);

        // Enhanced lighting
        const ambientLight = new THREE.AmbientLight(0xffffff, 0.5);
        this.scene.add(ambientLight);

        const directionalLight1 = new THREE.DirectionalLight(0xffffff, 0.8);
        directionalLight1.position.set(10, 10, 10);
        this.scene.add(directionalLight1);

        const directionalLight2 = new THREE.DirectionalLight(0x4488ff, 0.3);
        directionalLight2.position.set(-10, 5, -10);
        this.scene.add(directionalLight2);

        // Add point light at center
        const pointLight = new THREE.PointLight(0x3b82f6, 1, 30);
        pointLight.position.set(0, 0, 0);
        this.scene.add(pointLight);

        // Add grid helper
        const gridHelper = new THREE.GridHelper(50, 50, 0x333333, 0x1a1a1a);
        this.scene.add(gridHelper);

        // Add orbit controls
        this.controls = new THREE.OrbitControls(this.camera, this.renderer.domElement);
        this.controls.enableDamping = true;
        this.controls.dampingFactor = 0.05;
        this.controls.minDistance = 5;
        this.controls.maxDistance = 80;

        // Handle window resize
        window.addEventListener('resize', () => this.onWindowResize());

        // Start animation loop
        this.animate();
        this.updateFPS();
    }

    connectWebSocket() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws`;

        try {
            this.ws = new WebSocket(wsUrl);

            this.ws.onopen = () => {
                console.log('WebSocket connected');
                this.wsReconnectAttempts = 0;
                this.showSuccess('Connected to server');

                // Subscribe to all updates
                this.wsSend({
                    type: 'subscribe',
                    topics: ['nodes', 'latency', 'routes']
                });
            };

            this.ws.onmessage = (event) => {
                try {
                    const message = JSON.parse(event.data);
                    this.handleWebSocketMessage(message);
                } catch (e) {
                    console.error('Failed to parse WebSocket message:', e);
                }
            };

            this.ws.onerror = (error) => {
                console.error('WebSocket error:', error);
            };

            this.ws.onclose = () => {
                console.log('WebSocket disconnected');
                this.showError('Disconnected from server');

                // Attempt to reconnect
                if (this.wsReconnectAttempts < this.wsMaxReconnectAttempts) {
                    this.wsReconnectAttempts++;
                    const delay = Math.min(1000 * Math.pow(2, this.wsReconnectAttempts), 30000);
                    console.log(`Reconnecting in ${delay}ms...`);
                    setTimeout(() => this.connectWebSocket(), delay);
                }
            };
        } catch (e) {
            console.error('Failed to create WebSocket:', e);
        }
    }

    wsSend(message) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
        }
    }

    handleWebSocketMessage(message) {
        console.log('WebSocket message:', message);

        switch (message.type) {
            case 'node_discovered':
                this.handleNodeDiscovered(message.node);
                break;
            case 'node_status_changed':
                this.handleNodeStatusChanged(message.node_id, message.status);
                break;
            case 'latency_update':
                this.handleLatencyUpdate(message.connections);
                break;
            case 'routing_table_changed':
                console.log('Routing table changed for node:', message.node_id);
                break;
            case 'trace_route_result':
                this.handleTraceRouteResult(message);
                break;
            case 'error':
                this.showError(message.message);
                break;
            case 'bandwidth_test_progress':
                this.handleBandwidthTestProgress(message);
                break;
            case 'bandwidth_test_result':
                this.handleBandwidthTestResult(message);
                break;
        }
    }

    handleBandwidthTestProgress(message) {
        console.log('Bandwidth test progress:', message);
        this.bandwidthTests.set(message.test_id, message);
        this.updateDiscoveredNodesList();
    }

    handleBandwidthTestResult(message) {
        console.log('Bandwidth test result:', message);
        this.bandwidthResults.set(message.target_node_id, message);
        this.bandwidthTests.delete(message.test_id);
        this.updateDiscoveredNodesList();
        this.showSuccess(`Bandwidth test complete: ↑${message.upload_mbps.toFixed(2)} Mbps ↓${message.download_mbps.toFixed(2)} Mbps`);
    }

    handleNodeDiscovered(node) {
        console.log('Node discovered:', node);
        this.discoveredNodes.set(node.id, node);
        this.showSuccess(`New node discovered: ${node.hostname}`);
        this.updateDiscoveredNodesList();
        this.visualizeDiscoveredNodes();
    }

    handleNodeStatusChanged(nodeId, status) {
        const node = this.discoveredNodes.get(nodeId);
        if (node) {
            node.status = status;
            console.log(`Node ${nodeId} status changed to ${status}`);
            this.visualizeDiscoveredNodes();
        }
    }

    handleLatencyUpdate(connections) {
        console.log('Latency update:', connections);

        connections.forEach(conn => {
            // Store latency data
            this.latencyData.set(conn.to, conn.latency_ms);

            // Update the edge for this connection
            const edgeIndex = this.edges.findIndex(e =>
                e.userData.type === 'mesh-edge' && e.userData.nodeId === conn.to
            );

            if (edgeIndex !== -1) {
                const edge = this.edges[edgeIndex];
                const oldEdge = edge;

                // Get positions from current edge
                const positions = edge.geometry.attributes.position.array;
                const start = new THREE.Vector3(positions[0], positions[1], positions[2]);
                const end = new THREE.Vector3(positions[3], positions[4], positions[5]);

                // Remove old edge
                this.scene.remove(edge);

                // Create new edge with latency-based color
                const color = this.getLatencyColor(conn.latency_ms);
                const newEdge = this.createEdge(start, end, color, false);
                newEdge.userData = oldEdge.userData;

                // Add latency label
                this.addEdgeLabel(newEdge, `${conn.latency_ms}ms`, `latency-${conn.to}`);

                this.edges[edgeIndex] = newEdge;
                this.scene.add(newEdge);
            }
        });
    }

    getLatencyColor(latencyMs) {
        // Color code based on latency thresholds
        // < 20ms: green, 20-50ms: yellow-green, 50-100ms: yellow, 100-200ms: orange, > 200ms: red
        if (latencyMs < 20) return 0x10b981; // green
        if (latencyMs < 50) return 0x84cc16; // yellow-green
        if (latencyMs < 100) return 0xfbbf24; // yellow
        if (latencyMs < 200) return 0xf97316; // orange
        return 0xef4444; // red
    }

    addEdgeLabel(edge, text, labelId) {
        // Get midpoint of edge
        const positions = edge.geometry.attributes.position.array;
        const midX = (positions[0] + positions[3]) / 2;
        const midY = (positions[1] + positions[4]) / 2;
        const midZ = (positions[2] + positions[5]) / 2;

        const labelDiv = document.createElement('div');
        labelDiv.className = 'node-label';
        labelDiv.textContent = text;
        labelDiv.style.color = '#fbbf24';
        labelDiv.style.fontSize = '10px';
        labelDiv.style.fontFamily = 'monospace';
        labelDiv.style.background = 'rgba(0, 0, 0, 0.7)';
        labelDiv.style.padding = '2px 4px';
        labelDiv.style.borderRadius = '3px';

        const label = new THREE.CSS2DObject(labelDiv);
        label.position.set(midX, midY, midZ);
        edge.add(label);

        this.labels.set(labelId, label);
    }

    handleTraceRouteResult(result) {
        if (result.matched_route && this.routingTable) {
            const routeIndex = this.routingTable.routes.findIndex(r =>
                r.destination === result.matched_route.destination &&
                r.interface === result.matched_route.interface
            );

            if (routeIndex !== -1) {
                this.highlightRoute(routeIndex);
                this.showSuccess(`Route to ${result.destination} (${result.resolved_ip}) found!`);
            }
        }
    }

    async loadDiscoveredNodes() {
        try {
            const response = await fetch('/api/nodes');
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            const data = await response.json();
            this.localNodeId = data.local_node_id;

            data.nodes.forEach(node => {
                this.discoveredNodes.set(node.id, node);
            });

            console.log(`Loaded ${data.nodes.length} discovered nodes`);
            this.updateDiscoveredNodesList();
            this.visualizeDiscoveredNodes();
        } catch (error) {
            console.error('Failed to load discovered nodes:', error);
        }
    }

    updateDiscoveredNodesList() {
        const container = document.getElementById('discovered-nodes-list');
        if (!container) return;

        if (this.discoveredNodes.size === 0) {
            container.innerHTML = '<p style="color: #6b7280;">No nodes discovered yet</p>';
            return;
        }

        const nodesList = Array.from(this.discoveredNodes.values())
            .map(node => {
                const statusColor = node.status === 'online' ? '#10b981' : '#6b7280';
                const latency = this.latencyData.get(node.id);
                const result = this.bandwidthResults.get(node.id);

                // Check if there's an active test for this node
                const activeTest = Array.from(this.bandwidthTests.values())
                    .find(test => test.test_id.includes(node.id));

                let bandwidthInfo = '';
                if (activeTest) {
                    bandwidthInfo = `<div style="font-size: 10px; color: #fbbf24; margin-top: 4px;">Testing: ${activeTest.phase} (${activeTest.progress_percent}%)</div>`;
                } else if (result) {
                    bandwidthInfo = `<div style="font-size: 10px; color: #10b981; margin-top: 4px;">↑${result.upload_mbps.toFixed(1)} Mbps ↓${result.download_mbps.toFixed(1)} Mbps</div>`;
                }

                return `
                    <div class="node-item ${node.status}" style="position: relative;">
                        <div class="node-hostname">${node.hostname}</div>
                        <div class="node-id">${node.id.substring(0, 8)}...</div>
                        <div style="font-size: 11px; color: #a0a0a0;">
                            ${node.addresses.join(', ')}<br>
                            ${latency !== undefined ? `Latency: ${latency}ms<br>` : ''}
                        </div>
                        <div class="node-status ${node.status}">${node.status}</div>
                        ${bandwidthInfo}
                        ${!activeTest && node.status === 'online' ? `<button class="bandwidth-test-btn" data-node-id="${node.id}" style="margin-top: 6px; padding: 4px 8px; background: #3b82f6; border: none; color: white; border-radius: 3px; cursor: pointer; font-size: 11px;">Test Bandwidth</button>` : ''}
                    </div>
                `;
            }).join('');

        container.innerHTML = nodesList;

        // Add event listeners to bandwidth test buttons
        document.querySelectorAll('.bandwidth-test-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const nodeId = e.target.getAttribute('data-node-id');
                this.startBandwidthTest(nodeId);
            });
        });
    }

    startBandwidthTest(nodeId) {
        const testId = `test-${nodeId}-${Date.now()}`;
        this.wsSend({
            type: 'start_bandwidth_test',
            test_id: testId,
            node_id: nodeId
        });
        this.showInfo(`Starting bandwidth test to node ${nodeId.substring(0, 8)}...`);
    }

    async loadRoutingTable() {
        try {
            const response = await fetch('/api/routing-table');
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            this.routingTable = await response.json();
            this.visualizeRoutes();
            this.showInfo(`Loaded ${this.routingTable.routes.length} routes from ${this.routingTable.hostname}`);
            this.updateStats();
        } catch (error) {
            this.showError(`Failed to load routing table: ${error.message}`);
        }
    }

    visualizeRoutes() {
        // Clear existing route nodes and edges (but not discovered nodes)
        const routeKeys = Array.from(this.nodes.keys()).filter(key => key.startsWith('route-'));
        routeKeys.forEach(key => {
            const node = this.nodes.get(key);
            if (node) {
                this.scene.remove(node);
                this.nodes.delete(key);
            }
        });

        this.edges.forEach(edge => this.scene.remove(edge));
        this.edges = [];

        if (!this.routingTable || !this.routingTable.routes) {
            return;
        }

        // Create/update central node (local machine)
        let localNode = this.nodes.get('local');
        if (!localNode) {
            localNode = this.createLocalNode(0, 0, 0);
            localNode.userData = {
                type: 'local',
                name: this.routingTable.hostname,
                info: 'Local Machine'
            };
            this.nodes.set('local', localNode);
            this.scene.add(localNode);
            this.addLabel(localNode, this.routingTable.hostname, 'local-label');
        }

        // Position routes around the central node
        const routes = this.routingTable.routes;
        const radius = 12;
        const layers = Math.ceil(routes.length / 12);

        routes.forEach((route, index) => {
            const layer = Math.floor(index / 12);
            const indexInLayer = index % 12;
            const itemsInLayer = Math.min(12, routes.length - layer * 12);

            const angle = (indexInLayer / itemsInLayer) * Math.PI * 2;
            const currentRadius = radius + layer * 4;
            const x = Math.cos(angle) * currentRadius;
            const z = Math.sin(angle) * currentRadius;
            const y = (layer - layers / 2) * 3;

            const isDefault = route.destination === '0.0.0.0/0' || route.destination === 'default';
            const isGateway = route.gateway !== null;

            let color, nodeType;
            if (isDefault) {
                color = 0xfbbf24;
                nodeType = 'default-gateway';
            } else if (isGateway) {
                color = 0x10b981;
                nodeType = 'gateway';
            } else {
                color = 0x6b7280;
                nodeType = 'direct';
            }

            const node = this.createRouteNode(x, y, z, color, nodeType);
            node.userData = {
                type: 'route',
                nodeType: nodeType,
                route: route,
                index: index
            };
            this.nodes.set(`route-${index}`, node);
            this.scene.add(node);

            const label = isDefault ? 'Default' : route.destination.split('/')[0];
            this.addLabel(node, label, `route-label-${index}`);

            const edge = this.createEdge(
                localNode.position,
                node.position,
                color,
                isDefault
            );
            edge.userData = {
                type: 'edge',
                route: route,
                index: index
            };
            this.edges.push(edge);
            this.scene.add(edge);
        });

        this.updateStats();
    }

    visualizeDiscoveredNodes() {
        // Remove existing discovered node meshes
        const discoveredKeys = Array.from(this.nodes.keys()).filter(key => key.startsWith('discovered-'));
        discoveredKeys.forEach(key => {
            const node = this.nodes.get(key);
            if (node) {
                this.scene.remove(node);
                this.nodes.delete(key);
            }
        });

        // Position discovered nodes in a separate layer
        const nodeArray = Array.from(this.discoveredNodes.values());
        const meshRadius = 25;

        nodeArray.forEach((node, index) => {
            const angle = (index / nodeArray.length) * Math.PI * 2;
            const x = Math.cos(angle) * meshRadius;
            const z = Math.sin(angle) * meshRadius;
            const y = 8;

            const color = node.status === 'online' ? 0x10b981 : 0x6b7280;
            const mesh = this.createDiscoveredNodeMesh(x, y, z, color);

            mesh.userData = {
                type: 'discovered-node',
                node: node,
                nodeId: node.id
            };

            this.nodes.set(`discovered-${node.id}`, mesh);
            this.scene.add(mesh);
            this.addLabel(mesh, node.hostname, `discovered-label-${node.id}`);

            // Create edge from local node to discovered node
            const localNode = this.nodes.get('local');
            if (localNode) {
                // Use latency-based color if available
                const latency = this.latencyData.get(node.id);
                const edgeColor = latency !== undefined ? this.getLatencyColor(latency) : color;

                const edge = this.createEdge(
                    localNode.position,
                    mesh.position,
                    edgeColor,
                    false
                );
                edge.userData = {
                    type: 'mesh-edge',
                    nodeId: node.id
                };

                // Add latency label if we have latency data
                if (latency !== undefined) {
                    this.addEdgeLabel(edge, `${latency}ms`, `latency-${node.id}`);
                }

                this.edges.push(edge);
                this.scene.add(edge);
            }
        });

        this.updateStats();
    }

    createLocalNode(x, y, z) {
        const geometry = new THREE.OctahedronGeometry(2, 0);
        const material = new THREE.MeshStandardMaterial({
            color: 0x3b82f6,
            emissive: 0x3b82f6,
            emissiveIntensity: 0.4,
            metalness: 0.5,
            roughness: 0.3
        });
        const mesh = new THREE.Mesh(geometry, material);
        mesh.position.set(x, y, z);

        const glowGeometry = new THREE.OctahedronGeometry(2.3, 0);
        const glowMaterial = new THREE.MeshBasicMaterial({
            color: 0x3b82f6,
            transparent: true,
            opacity: 0.2
        });
        const glow = new THREE.Mesh(glowGeometry, glowMaterial);
        mesh.add(glow);

        return mesh;
    }

    createRouteNode(x, y, z, color, nodeType) {
        let geometry, size;

        if (nodeType === 'default-gateway') {
            geometry = new THREE.BoxGeometry(1.2, 1.2, 1.2);
            size = 1.2;
        } else if (nodeType === 'gateway') {
            geometry = new THREE.SphereGeometry(0.8, 16, 16);
            size = 0.8;
        } else {
            geometry = new THREE.SphereGeometry(0.6, 12, 12);
            size = 0.6;
        }

        const material = new THREE.MeshStandardMaterial({
            color: color,
            emissive: color,
            emissiveIntensity: 0.2,
            metalness: 0.4,
            roughness: 0.6
        });
        const mesh = new THREE.Mesh(geometry, material);
        mesh.position.set(x, y, z);
        return mesh;
    }

    createDiscoveredNodeMesh(x, y, z, color) {
        // Use dodecahedron for discovered nodes to distinguish them
        const geometry = new THREE.DodecahedronGeometry(1.5, 0);
        const material = new THREE.MeshStandardMaterial({
            color: color,
            emissive: color,
            emissiveIntensity: 0.3,
            metalness: 0.5,
            roughness: 0.4
        });
        const mesh = new THREE.Mesh(geometry, material);
        mesh.position.set(x, y, z);
        return mesh;
    }

    createEdge(start, end, color, isDashed = false) {
        const points = [start.clone(), end.clone()];
        const geometry = new THREE.BufferGeometry().setFromPoints(points);

        const material = isDashed
            ? new THREE.LineDashedMaterial({
                color: color,
                linewidth: 2,
                dashSize: 0.5,
                gapSize: 0.3
            })
            : new THREE.LineBasicMaterial({
                color: color,
                linewidth: 2,
                opacity: 0.6,
                transparent: true
            });

        const line = new THREE.Line(geometry, material);
        if (isDashed) {
            line.computeLineDistances();
        }
        return line;
    }

    addLabel(object, text, id) {
        const labelDiv = document.createElement('div');
        labelDiv.className = 'node-label';
        labelDiv.textContent = text;
        labelDiv.style.color = '#e0e0e0';
        labelDiv.style.fontSize = '12px';
        labelDiv.style.fontFamily = 'monospace';
        labelDiv.style.padding = '2px 6px';
        labelDiv.style.background = 'rgba(0, 0, 0, 0.7)';
        labelDiv.style.borderRadius = '3px';

        const label = new THREE.CSS2DObject(labelDiv);
        label.position.set(0, 2, 0);
        object.add(label);
        this.labels.set(id, label);
    }

    highlightObject(object, isHover = false) {
        if (!object || !object.material) return;

        const originalColor = object.material.color.getHex();
        const originalEmissive = object.material.emissive.getHex();

        object.userData.originalColor = originalColor;
        object.userData.originalEmissive = originalEmissive;
        object.userData.originalScale = object.scale.clone();

        if (isHover) {
            object.material.emissiveIntensity = 0.6;
            object.scale.multiplyScalar(1.1);
        } else {
            object.material.emissive.setHex(0xffffff);
            object.material.emissiveIntensity = 0.8;
            object.scale.multiplyScalar(1.2);
        }

        this.highlightedObjects.push(object);
    }

    clearHighlights() {
        this.highlightedObjects.forEach(obj => {
            if (obj.userData.originalColor !== undefined) {
                obj.material.color.setHex(obj.userData.originalColor);
                obj.material.emissive.setHex(obj.userData.originalEmissive);
                obj.material.emissiveIntensity = 0.2;
            }
            if (obj.userData.originalScale) {
                obj.scale.copy(obj.userData.originalScale);
            }
        });
        this.highlightedObjects = [];
    }

    highlightRoute(routeIndex) {
        this.clearHighlights();
        this.selectedObject = null;

        if (routeIndex === null) {
            return;
        }

        const routeNode = this.nodes.get(`route-${routeIndex}`);
        if (routeNode) {
            this.highlightObject(routeNode, false);
            this.selectedObject = routeNode;
        }

        const edge = this.edges.find(e => e.userData && e.userData.index === routeIndex);
        if (edge) {
            edge.material.color.setHex(0x3b82f6);
            edge.material.opacity = 1.0;
            if (edge.material.linewidth !== undefined) {
                edge.material.linewidth = 4;
            }
            this.highlightedObjects.push(edge);
        }

        if (routeNode && routeNode.userData.route) {
            this.displayRouteDetails(routeNode.userData.route);
        }

        this.focusOnObject(routeNode);
    }

    focusOnObject(object) {
        if (!object) return;

        const targetPosition = object.position.clone();
        const distance = 15;
        const direction = this.camera.position.clone().sub(this.controls.target).normalize();

        this.controls.target.copy(targetPosition);
        this.camera.position.copy(targetPosition.clone().add(direction.multiplyScalar(distance)));
    }

    clearScene() {
        this.nodes.forEach(node => this.scene.remove(node));
        this.nodes.clear();

        this.edges.forEach(edge => this.scene.remove(edge));
        this.edges = [];

        this.labels.clear();
        this.clearHighlights();
    }

    displayRouteDetails(route) {
        const detailsDiv = document.getElementById('route-details');
        detailsDiv.innerHTML = `
            <div class="route-item">
                <p><strong>Destination:</strong> ${route.destination}</p>
                <p><strong>Gateway:</strong> ${route.gateway || 'None (direct)'}</p>
                <p><strong>Interface:</strong> ${route.interface}</p>
                <p><strong>Metric:</strong> ${route.metric}</p>
                ${route.flags && route.flags.length > 0 ? `<p><strong>Flags:</strong> ${route.flags.join(', ')}</p>` : ''}
            </div>
        `;
    }

    async traceRoute(destination) {
        try {
            const response = await fetch('/api/trace-route', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ destination: destination })
            });

            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.message || 'Failed to trace route');
            }

            const result = await response.json();

            if (result.matched_route) {
                const routeIndex = this.routingTable.routes.findIndex(r =>
                    r.destination === result.matched_route.destination &&
                    r.interface === result.matched_route.interface
                );

                if (routeIndex !== -1) {
                    this.highlightRoute(routeIndex);
                    this.showSuccess(`Route to ${destination} (${result.resolved_ip}) found!`);
                }
            } else {
                this.showError('No route found to destination');
            }
        } catch (error) {
            this.showError(`Error tracing route: ${error.message}`);
        }
    }

    async performTraceroute(destination) {
        try {
            this.showSuccess(`Performing traceroute to ${destination}...`);

            const response = await fetch('/api/traceroute', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ destination: destination })
            });

            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.message || 'Failed to perform traceroute');
            }

            const result = await response.json();
            this.tracerouteData = result;

            if (result.completed && result.hops.length > 0) {
                this.showSuccess(`Traceroute completed: ${result.hops.length} hops to ${destination}`);

                // Auto-enable public routes view
                document.getElementById('show-public-routes').checked = true;
                this.showPublicRoutes = true;

                this.visualizeTraceroute(result);

                // Perform route comparison with VPN route
                this.compareRoutes(destination, result);
            } else if (result.error) {
                this.showError(`Traceroute failed: ${result.error}`);
            } else {
                this.showError('Traceroute did not complete successfully');
            }
        } catch (error) {
            this.showError(`Error performing traceroute: ${error.message}`);
        }
    }

    visualizeTraceroute(tracerouteData) {
        // Clear existing traceroute visualization
        this.clearTracerouteVisualization();

        if (!tracerouteData || !tracerouteData.hops || tracerouteData.hops.length === 0) {
            return;
        }

        console.log('Visualizing traceroute with', tracerouteData.hops.length, 'hops');

        // Create nodes for each hop
        const hopNodes = [];
        const radius = 15; // Distance from center
        const angleStep = (Math.PI * 2) / Math.max(tracerouteData.hops.length, 8);

        tracerouteData.hops.forEach((hop, index) => {
            if (!hop.ip || hop.timed_out) {
                // Skip timed out hops
                hopNodes.push(null);
                return;
            }

            // Position hops in a circle around the center
            const angle = index * angleStep;
            const x = Math.cos(angle) * radius;
            const y = 2 + (index * 0.5); // Slight vertical offset
            const z = Math.sin(angle) * radius;

            const hopMesh = this.createHopNode(x, y, z, hop.ip);
            hopMesh.userData = {
                type: 'traceroute-hop',
                hop: hop,
                index: index
            };

            this.scene.add(hopMesh);
            this.tracerouteHops.set(hop.ip, hopMesh);
            hopNodes.push(hopMesh);

            // Add label
            const avgRtt = hop.rtt_ms.filter(r => r !== null).reduce((a, b) => a + b, 0) /
                          hop.rtt_ms.filter(r => r !== null).length || 0;
            const label = avgRtt > 0 ? `${hop.ip}\n${avgRtt.toFixed(1)}ms` : hop.ip;
            this.addLabel(hopMesh, label, `hop-${hop.ip}`);
        });

        // Create edges connecting the hops
        const localNode = this.nodes.get('local');
        if (localNode) {
            // Connect local node to first hop
            const firstHop = hopNodes.find(h => h !== null);
            if (firstHop) {
                const edge = this.createEdge(
                    localNode.position,
                    firstHop.position,
                    0xef4444, // Red color for public routes
                    true      // Dashed line
                );
                edge.userData = { type: 'traceroute-edge' };
                this.scene.add(edge);
                this.tracerouteEdges.push(edge);
            }
        }

        // Connect consecutive hops
        for (let i = 0; i < hopNodes.length - 1; i++) {
            const currentHop = hopNodes[i];
            const nextHop = hopNodes[i + 1];

            if (currentHop && nextHop) {
                const edge = this.createEdge(
                    currentHop.position,
                    nextHop.position,
                    0xef4444, // Red color for public routes
                    true      // Dashed line
                );
                edge.userData = { type: 'traceroute-edge' };
                this.scene.add(edge);
                this.tracerouteEdges.push(edge);
            }
        }

        this.updateTracerouteVisualization();
    }

    createHopNode(x, y, z, ip) {
        // Use octahedron for public internet hops
        const geometry = new THREE.OctahedronGeometry(1, 0);
        const material = new THREE.MeshStandardMaterial({
            color: 0xef4444, // Red color for public routes
            emissive: 0xef4444,
            emissiveIntensity: 0.3,
            metalness: 0.5,
            roughness: 0.4
        });
        const mesh = new THREE.Mesh(geometry, material);
        mesh.position.set(x, y, z);
        return mesh;
    }

    updateTracerouteVisualization() {
        // Show or hide traceroute visualization based on toggle
        const visible = this.showPublicRoutes;

        this.tracerouteHops.forEach((hopMesh) => {
            hopMesh.visible = visible;
        });

        this.tracerouteEdges.forEach((edge) => {
            edge.visible = visible;
        });

        // Update labels visibility
        this.tracerouteHops.forEach((hopMesh, ip) => {
            const label = this.labels.get(`hop-${ip}`);
            if (label) {
                label.element.style.display = visible ? 'block' : 'none';
            }
        });
    }

    clearTracerouteVisualization() {
        // Remove all traceroute hops
        this.tracerouteHops.forEach((hopMesh, ip) => {
            this.scene.remove(hopMesh);
            const label = this.labels.get(`hop-${ip}`);
            if (label) {
                this.labels.delete(`hop-${ip}`);
            }
        });
        this.tracerouteHops.clear();

        // Remove all traceroute edges
        this.tracerouteEdges.forEach((edge) => {
            this.scene.remove(edge);
        });
        this.tracerouteEdges = [];
    }

    async compareRoutes(destination, tracerouteData) {
        // Get VPN route information
        let vpnRoute = null;
        let vpnLatency = null;

        try {
            const traceResponse = await fetch('/api/trace-route', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ destination: destination })
            });

            if (traceResponse.ok) {
                const traceResult = await traceResponse.json();
                vpnRoute = traceResult.matched_route;
            }
        } catch (error) {
            console.error('Error getting VPN route:', error);
        }

        // Calculate public route metrics
        const publicHops = tracerouteData.hops.filter(h => !h.timed_out && h.ip);
        const publicHopCount = publicHops.length;

        // Calculate average latency for public route
        let publicLatency = 0;
        if (publicHops.length > 0) {
            const lastHop = publicHops[publicHops.length - 1];
            const rttValues = lastHop.rtt_ms.filter(r => r !== null);
            if (rttValues.length > 0) {
                publicLatency = rttValues.reduce((a, b) => a + b, 0) / rttValues.length;
            }
        }

        // Try to get VPN latency from latency data if available
        if (vpnRoute && vpnRoute.gateway) {
            // Look for latency to nodes that might be the destination
            // This is a simplified approach - in a real implementation,
            // we might need to ping the destination through the VPN
        }

        // Build comparison panel
        const comparisonDiv = document.getElementById('route-comparison');
        const contentDiv = document.getElementById('comparison-content');

        let comparisonHTML = '<div class="comparison-table">';
        comparisonHTML += '<table style="width: 100%; border-collapse: collapse;">';
        comparisonHTML += '<tr><th style="text-align: left; padding: 8px; border-bottom: 1px solid #333;"></th>';
        comparisonHTML += '<th style="text-align: left; padding: 8px; border-bottom: 1px solid #333;">VPN Route</th>';
        comparisonHTML += '<th style="text-align: left; padding: 8px; border-bottom: 1px solid #333;">Public Route</th></tr>';

        // Hop count comparison
        comparisonHTML += '<tr>';
        comparisonHTML += '<td style="padding: 8px; border-bottom: 1px solid #222;"><strong>Hop Count</strong></td>';
        comparisonHTML += `<td style="padding: 8px; border-bottom: 1px solid #222;">${vpnRoute ? '1 (via ' + vpnRoute.interface + ')' : 'N/A'}</td>`;
        comparisonHTML += `<td style="padding: 8px; border-bottom: 1px solid #222;">${publicHopCount}</td>`;
        comparisonHTML += '</tr>';

        // Latency comparison
        comparisonHTML += '<tr>';
        comparisonHTML += '<td style="padding: 8px; border-bottom: 1px solid #222;"><strong>Latency</strong></td>';
        comparisonHTML += `<td style="padding: 8px; border-bottom: 1px solid #222;">${vpnLatency !== null ? vpnLatency.toFixed(1) + ' ms' : 'Unknown'}</td>`;
        comparisonHTML += `<td style="padding: 8px; border-bottom: 1px solid #222;">${publicLatency > 0 ? publicLatency.toFixed(1) + ' ms' : 'Unknown'}</td>`;
        comparisonHTML += '</tr>';

        // Gateway/Interface
        comparisonHTML += '<tr>';
        comparisonHTML += '<td style="padding: 8px; border-bottom: 1px solid #222;"><strong>Gateway</strong></td>';
        comparisonHTML += `<td style="padding: 8px; border-bottom: 1px solid #222;">${vpnRoute ? (vpnRoute.gateway || 'Direct') : 'N/A'}</td>`;
        comparisonHTML += `<td style="padding: 8px; border-bottom: 1px solid #222;">${publicHops.length > 0 ? publicHops[0].ip : 'N/A'}</td>`;
        comparisonHTML += '</tr>';

        // Route type
        comparisonHTML += '<tr>';
        comparisonHTML += '<td style="padding: 8px;"><strong>Route Type</strong></td>';
        comparisonHTML += `<td style="padding: 8px;">${vpnRoute ? (vpnRoute.interface.includes('tun') || vpnRoute.interface.includes('wg') ? 'VPN Tunnel' : 'Direct') : 'N/A'}</td>`;
        comparisonHTML += '<td style="padding: 8px;">Public Internet</td>';
        comparisonHTML += '</tr>';

        comparisonHTML += '</table>';

        // Recommendation
        comparisonHTML += '<div style="margin-top: 16px; padding: 12px; background: rgba(59, 130, 246, 0.1); border-left: 3px solid #3b82f6; border-radius: 4px;">';
        comparisonHTML += '<strong>Recommendation:</strong><br>';

        if (vpnRoute) {
            if (vpnRoute.interface.includes('tun') || vpnRoute.interface.includes('wg')) {
                comparisonHTML += 'Traffic is routed through VPN tunnel (' + vpnRoute.interface + '). ';
                if (vpnLatency !== null && publicLatency > 0) {
                    if (vpnLatency < publicLatency) {
                        comparisonHTML += 'VPN route is faster! 🚀';
                    } else {
                        comparisonHTML += 'Public route may be faster, but VPN provides security. 🔒';
                    }
                } else {
                    comparisonHTML += 'VPN provides encryption and privacy. 🔒';
                }
            } else {
                comparisonHTML += 'Using direct route (not through VPN). Consider using VPN for security. 🔓';
            }
        } else {
            comparisonHTML += 'No VPN route found. Traffic goes through public internet. 🌐';
        }

        comparisonHTML += '</div>';
        comparisonHTML += '</div>';

        contentDiv.innerHTML = comparisonHTML;
        comparisonDiv.style.display = 'block';
    }

    onMouseMove(event) {
        const container = document.getElementById('canvas-container');
        const rect = container.getBoundingClientRect();

        this.mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
        this.mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

        this.raycaster.setFromCamera(this.mouse, this.camera);

        const nodeArray = Array.from(this.nodes.values());
        const hopArray = Array.from(this.tracerouteHops.values());
        const intersects = this.raycaster.intersectObjects([...nodeArray, ...hopArray], true);

        if (this.hoveredObject && (!intersects.length || intersects[0].object !== this.hoveredObject)) {
            if (this.hoveredObject !== this.selectedObject) {
                if (this.hoveredObject.userData.originalColor !== undefined) {
                    this.hoveredObject.material.emissiveIntensity = 0.2;
                    this.hoveredObject.scale.copy(this.hoveredObject.userData.originalScale);
                }
            }
            this.hoveredObject = null;
            container.style.cursor = 'default';
        }

        if (intersects.length > 0) {
            const object = intersects[0].object;
            if (object !== this.hoveredObject && object !== this.selectedObject) {
                this.hoveredObject = object;
                this.highlightObject(object, true);
                container.style.cursor = 'pointer';

                if (object.userData && object.userData.route) {
                    this.showTooltip(object.userData.route, event.clientX, event.clientY);
                } else if (object.userData && object.userData.node) {
                    this.showTooltip({
                        info: `${object.userData.node.hostname} (${object.userData.node.status})`
                    }, event.clientX, event.clientY);
                } else if (object.userData && object.userData.hop) {
                    const hop = object.userData.hop;
                    const avgRtt = hop.rtt_ms.filter(r => r !== null).reduce((a, b) => a + b, 0) /
                                  hop.rtt_ms.filter(r => r !== null).length || 0;
                    this.showTooltip({
                        info: `Hop ${hop.hop_number}: ${hop.ip}\nRTT: ${avgRtt.toFixed(1)}ms`
                    }, event.clientX, event.clientY);
                } else if (object.userData && object.userData.name) {
                    this.showTooltip({ info: object.userData.info }, event.clientX, event.clientY);
                }
            }
        } else {
            this.hideTooltip();
        }
    }

    onClick(event) {
        const container = document.getElementById('canvas-container');
        const rect = container.getBoundingClientRect();

        this.mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
        this.mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

        this.raycaster.setFromCamera(this.mouse, this.camera);

        const nodeArray = Array.from(this.nodes.values());
        const hopArray = Array.from(this.tracerouteHops.values());
        const intersects = this.raycaster.intersectObjects([...nodeArray, ...hopArray], true);

        if (intersects.length > 0) {
            const object = intersects[0].object;
            if (object.userData && object.userData.type === 'route') {
                this.highlightRoute(object.userData.index);
            } else if (object.userData && object.userData.type === 'discovered-node') {
                console.log('Clicked discovered node:', object.userData.node);
                // TODO: Show node details or switch to that node's view
            }
        }
    }

    onDoubleClick(event) {
        const container = document.getElementById('canvas-container');
        const rect = container.getBoundingClientRect();

        this.mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
        this.mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

        this.raycaster.setFromCamera(this.mouse, this.camera);

        const nodeArray = Array.from(this.nodes.values());
        const hopArray = Array.from(this.tracerouteHops.values());
        const intersects = this.raycaster.intersectObjects([...nodeArray, ...hopArray], true);

        if (intersects.length > 0) {
            this.focusOnObject(intersects[0].object);
        }
    }

    showTooltip(data, x, y) {
        let tooltip = document.getElementById('tooltip');
        if (!tooltip) {
            tooltip = document.createElement('div');
            tooltip.id = 'tooltip';
            tooltip.className = 'tooltip';
            document.body.appendChild(tooltip);
        }

        if (data.destination) {
            tooltip.innerHTML = `
                <strong>${data.destination}</strong><br>
                via ${data.gateway || 'direct'}<br>
                ${data.interface}
            `;
        } else {
            tooltip.innerHTML = data.info || '';
        }

        tooltip.style.left = (x + 10) + 'px';
        tooltip.style.top = (y + 10) + 'px';
        tooltip.style.display = 'block';
    }

    hideTooltip() {
        const tooltip = document.getElementById('tooltip');
        if (tooltip) {
            tooltip.style.display = 'none';
        }
    }

    setupEventListeners() {
        const container = document.getElementById('canvas-container');

        container.addEventListener('mousemove', (e) => this.onMouseMove(e));
        container.addEventListener('click', (e) => this.onClick(e));
        container.addEventListener('dblclick', (e) => this.onDoubleClick(e));

        document.getElementById('trace-btn').addEventListener('click', () => {
            const destination = document.getElementById('destination').value.trim();
            if (destination) {
                this.traceRoute(destination);
            } else {
                this.showError('Please enter a destination');
            }
        });

        document.getElementById('refresh-btn').addEventListener('click', () => {
            this.loadRoutingTable();
            this.loadDiscoveredNodes();
        });

        document.getElementById('traceroute-btn').addEventListener('click', () => {
            const destination = document.getElementById('destination').value.trim();
            if (destination) {
                this.performTraceroute(destination);
            } else {
                this.showError('Please enter a destination for traceroute');
            }
        });

        document.getElementById('show-public-routes').addEventListener('change', (e) => {
            this.showPublicRoutes = e.target.checked;
            this.updateTracerouteVisualization();
        });

        document.getElementById('destination').addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                document.getElementById('trace-btn').click();
            }
        });

        document.addEventListener('keydown', (e) => {
            if (e.key === 'r' || e.key === 'R') {
                this.loadRoutingTable();
                this.loadDiscoveredNodes();
            } else if (e.key === 't' || e.key === 'T') {
                document.getElementById('destination').focus();
            } else if (e.key === 'Escape') {
                this.clearHighlights();
                this.selectedObject = null;
                document.getElementById('route-details').innerHTML = '';
            }
        });
    }

    updateStats() {
        const statsDiv = document.getElementById('stats');
        if (statsDiv && this.routingTable) {
            statsDiv.innerHTML = `
                Nodes: ${this.nodes.size} |
                Routes: ${this.routingTable.routes.length} |
                Discovered: ${this.discoveredNodes.size} |
                FPS: ${this.fps}
            `;
        }
    }

    updateFPS() {
        setInterval(() => {
            const now = performance.now();
            const delta = now - this.lastTime;
            this.fps = Math.round((this.frameCount * 1000) / delta);
            this.frameCount = 0;
            this.lastTime = now;
            this.updateStats();
        }, 1000);
    }

    showError(message) {
        const errorDiv = document.getElementById('error');
        errorDiv.textContent = message;
        errorDiv.className = 'error';
        errorDiv.style.display = 'block';
        setTimeout(() => {
            errorDiv.style.display = 'none';
        }, 5000);
    }

    showSuccess(message) {
        const errorDiv = document.getElementById('error');
        errorDiv.textContent = message;
        errorDiv.className = 'success';
        errorDiv.style.display = 'block';
        setTimeout(() => {
            errorDiv.style.display = 'none';
        }, 5000);
    }

    showInfo(message) {
        const detailsDiv = document.getElementById('route-details');
        detailsDiv.innerHTML = `<p>${message}</p>`;
    }

    onWindowResize() {
        const container = document.getElementById('canvas-container');
        this.camera.aspect = container.clientWidth / container.clientHeight;
        this.camera.updateProjectionMatrix();
        this.renderer.setSize(container.clientWidth, container.clientHeight);
        this.labelRenderer.setSize(container.clientWidth, container.clientHeight);
    }

    animate() {
        requestAnimationFrame(() => this.animate());
        this.frameCount++;
        this.controls.update();
        this.renderer.render(this.scene, this.camera);
        this.labelRenderer.render(this.scene, this.camera);
    }
}

// Initialize the visualizer when the page loads
window.addEventListener('DOMContentLoaded', () => {
    new RouteVisualizer();
});
