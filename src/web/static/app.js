// Network Route Visualizer - Three.js Frontend

class RouteVisualizer {
    constructor() {
        this.scene = null;
        this.camera = null;
        this.renderer = null;
        this.controls = null;
        this.routingTable = null;
        this.nodes = new Map();
        this.edges = [];
        this.highlightedEdges = [];

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
        this.camera.position.set(0, 10, 20);

        // Create renderer
        this.renderer = new THREE.WebGLRenderer({
            canvas: document.getElementById('three-canvas'),
            antialias: true
        });
        this.renderer.setSize(container.clientWidth, container.clientHeight);
        this.renderer.setPixelRatio(window.devicePixelRatio);

        // Add lights
        const ambientLight = new THREE.AmbientLight(0xffffff, 0.6);
        this.scene.add(ambientLight);

        const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
        directionalLight.position.set(10, 10, 10);
        this.scene.add(directionalLight);

        // Add grid helper
        const gridHelper = new THREE.GridHelper(50, 50, 0x333333, 0x222222);
        this.scene.add(gridHelper);

        // Add orbit controls
        this.controls = new THREE.OrbitControls(this.camera, this.renderer.domElement);
        this.controls.enableDamping = true;
        this.controls.dampingFactor = 0.05;

        // Handle window resize
        window.addEventListener('resize', () => this.onWindowResize());

        // Start animation loop
        this.animate();
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

        // Create central node (local machine)
        const localNode = this.createNode(0, 0, 0, 0x3b82f6, 1.5);
        this.nodes.set('local', localNode);
        this.scene.add(localNode);

        // Position routes around the central node
        const routes = this.routingTable.routes;
        const radius = 10;

        routes.forEach((route, index) => {
            const angle = (index / routes.length) * Math.PI * 2;
            const x = Math.cos(angle) * radius;
            const z = Math.sin(angle) * radius;
            const y = (Math.random() - 0.5) * 2;

            // Determine color based on destination
            let color = 0x10b981; // Green for normal routes
            if (route.destination === '0.0.0.0/0' || route.destination === 'default') {
                color = 0xfbbf24; // Yellow for default route
            }

            const node = this.createNode(x, y, z, color, 0.8);
            this.nodes.set(`route-${index}`, node);
            this.scene.add(node);

            // Create edge from local to route
            const edge = this.createEdge(
                localNode.position,
                node.position,
                0x666666
            );
            this.edges.push({
                mesh: edge,
                route: route,
                index: index
            });
            this.scene.add(edge);
        });
    }

    createNode(x, y, z, color, size = 1) {
        const geometry = new THREE.SphereGeometry(size, 32, 32);
        const material = new THREE.MeshStandardMaterial({
            color: color,
            emissive: color,
            emissiveIntensity: 0.2,
            metalness: 0.3,
            roughness: 0.7
        });
        const mesh = new THREE.Mesh(geometry, material);
        mesh.position.set(x, y, z);
        return mesh;
    }

    createEdge(start, end, color) {
        const points = [start, end];
        const geometry = new THREE.BufferGeometry().setFromPoints(points);
        const material = new THREE.LineBasicMaterial({ color: color, linewidth: 1 });
        return new THREE.Line(geometry, material);
    }

    highlightRoute(routeIndex) {
        // Clear previous highlights
        this.clearHighlights();

        if (routeIndex === null) {
            return;
        }

        // Highlight the selected edge
        const edgeData = this.edges.find(e => e.index === routeIndex);
        if (edgeData) {
            edgeData.mesh.material.color.setHex(0x3b82f6);
            edgeData.mesh.material.linewidth = 3;
            this.highlightedEdges.push(edgeData.mesh);

            // Display route details
            this.displayRouteDetails(edgeData.route);
        }
    }

    clearHighlights() {
        this.highlightedEdges.forEach(edge => {
            edge.material.color.setHex(0x666666);
            edge.material.linewidth = 1;
        });
        this.highlightedEdges = [];
    }

    clearScene() {
        // Remove all nodes
        this.nodes.forEach(node => this.scene.remove(node));
        this.nodes.clear();

        // Remove all edges
        this.edges.forEach(edge => this.scene.remove(edge.mesh));
        this.edges = [];

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
                ${route.flags.length > 0 ? `<p><strong>Flags:</strong> ${route.flags.join(', ')}</p>` : ''}
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

    setupEventListeners() {
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
    }

    animate() {
        requestAnimationFrame(() => this.animate());
        this.controls.update();
        this.renderer.render(this.scene, this.camera);
    }
}

// Initialize the visualizer when the page loads
window.addEventListener('DOMContentLoaded', () => {
    new RouteVisualizer();
});
