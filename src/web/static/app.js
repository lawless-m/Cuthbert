// Network Route Visualizer - Enhanced Three.js Frontend (Phase 2)

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

        this.init();
        this.setupEventListeners();
        this.loadRoutingTable();
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
        this.camera.position.set(0, 15, 25);

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
        this.controls.maxDistance = 50;

        // Handle window resize
        window.addEventListener('resize', () => this.onWindowResize());

        // Start animation loop
        this.animate();
        this.updateFPS();
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
        // Clear existing nodes and edges
        this.clearScene();

        if (!this.routingTable || !this.routingTable.routes) {
            return;
        }

        // Create central node (local machine) - larger octahedron
        const localNode = this.createLocalNode(0, 0, 0);
        localNode.userData = {
            type: 'local',
            name: this.routingTable.hostname,
            info: 'Local Machine'
        };
        this.nodes.set('local', localNode);
        this.scene.add(localNode);

        // Add label for local node
        this.addLabel(localNode, this.routingTable.hostname, 'local-label');

        // Position routes around the central node with improved layout
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

            // Determine node type and color based on route
            const isDefault = route.destination === '0.0.0.0/0' || route.destination === 'default';
            const isGateway = route.gateway !== null;

            let color, nodeType;
            if (isDefault) {
                color = 0xfbbf24; // Yellow for default route
                nodeType = 'default-gateway';
            } else if (isGateway) {
                color = 0x10b981; // Green for gateway routes
                nodeType = 'gateway';
            } else {
                color = 0x6b7280; // Gray for direct routes
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

            // Add label for route
            const label = isDefault ? 'Default' : route.destination.split('/')[0];
            this.addLabel(node, label, `route-label-${index}`);

            // Create edge from local to route
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

    createLocalNode(x, y, z) {
        // Octahedron for local machine - more distinct
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

        // Add glow effect
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
            // Box for default gateway
            geometry = new THREE.BoxGeometry(1.2, 1.2, 1.2);
            size = 1.2;
        } else if (nodeType === 'gateway') {
            // Sphere for gateways
            geometry = new THREE.SphereGeometry(0.8, 16, 16);
            size = 0.8;
        } else {
            // Smaller sphere for direct routes
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
        label.position.set(0, 1.5, 0);
        object.add(label);
        this.labels.set(id, label);
    }

    highlightObject(object, isHover = false) {
        if (!object || !object.material) return;

        const originalColor = object.material.color.getHex();
        const originalEmissive = object.material.emissive.getHex();

        // Store original values
        object.userData.originalColor = originalColor;
        object.userData.originalEmissive = originalEmissive;
        object.userData.originalScale = object.scale.clone();

        // Apply highlight
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

        // Highlight the route node
        const routeNode = this.nodes.get(`route-${routeIndex}`);
        if (routeNode) {
            this.highlightObject(routeNode, false);
            this.selectedObject = routeNode;
        }

        // Highlight the edge
        const edge = this.edges.find(e => e.userData && e.userData.index === routeIndex);
        if (edge) {
            edge.material.color.setHex(0x3b82f6);
            edge.material.opacity = 1.0;
            if (edge.material.linewidth !== undefined) {
                edge.material.linewidth = 4;
            }
            this.highlightedObjects.push(edge);
        }

        // Display route details
        if (routeNode && routeNode.userData.route) {
            this.displayRouteDetails(routeNode.userData.route);
        }

        // Focus camera on selected node
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
        // Remove all nodes
        this.nodes.forEach(node => this.scene.remove(node));
        this.nodes.clear();

        // Remove all edges
        this.edges.forEach(edge => this.scene.remove(edge));
        this.edges = [];

        // Clear labels
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

            // Find and highlight the matching route
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

    onMouseMove(event) {
        const container = document.getElementById('canvas-container');
        const rect = container.getBoundingClientRect();

        this.mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
        this.mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

        this.raycaster.setFromCamera(this.mouse, this.camera);

        // Check intersections with nodes
        const nodeArray = Array.from(this.nodes.values());
        const intersects = this.raycaster.intersectObjects(nodeArray, true);

        // Clear previous hover if hovering different object
        if (this.hoveredObject && (!intersects.length || intersects[0].object !== this.hoveredObject)) {
            if (this.hoveredObject !== this.selectedObject) {
                // Remove hover highlight
                if (this.hoveredObject.userData.originalColor !== undefined) {
                    this.hoveredObject.material.emissiveIntensity = 0.2;
                    this.hoveredObject.scale.copy(this.hoveredObject.userData.originalScale);
                }
            }
            this.hoveredObject = null;
            container.style.cursor = 'default';
        }

        // Apply hover to new object
        if (intersects.length > 0) {
            const object = intersects[0].object;
            if (object !== this.hoveredObject && object !== this.selectedObject) {
                this.hoveredObject = object;
                this.highlightObject(object, true);
                container.style.cursor = 'pointer';

                // Show tooltip
                if (object.userData && object.userData.route) {
                    this.showTooltip(object.userData.route, event.clientX, event.clientY);
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
        const intersects = this.raycaster.intersectObjects(nodeArray, true);

        if (intersects.length > 0) {
            const object = intersects[0].object;
            if (object.userData && object.userData.type === 'route') {
                this.highlightRoute(object.userData.index);
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
        const intersects = this.raycaster.intersectObjects(nodeArray, true);

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

        // Mouse events for interaction
        container.addEventListener('mousemove', (e) => this.onMouseMove(e));
        container.addEventListener('click', (e) => this.onClick(e));
        container.addEventListener('dblclick', (e) => this.onDoubleClick(e));

        // UI button events
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
        });

        document.getElementById('destination').addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                document.getElementById('trace-btn').click();
            }
        });

        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            if (e.key === 'r' || e.key === 'R') {
                this.loadRoutingTable();
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
