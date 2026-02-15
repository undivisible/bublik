// Three.js scene manager for Bublik
// Uses global THREE from CDN

const orbs = new Map();
let scene, camera, renderer, particles, ambientRings, clock;
let connectionLines = [];
let cameraAngle = 0;

// World-space dimensions for orb placement
const WORLD_W = 600;
const WORLD_H = 400;

export function initScene(canvas) {
    scene = new THREE.Scene();
    scene.fog = new THREE.FogExp2(0x0a0a0a, 0.003);

    camera = new THREE.PerspectiveCamera(60, canvas.clientWidth / canvas.clientHeight, 1, 2000);
    camera.position.set(0, 80, 500);
    camera.lookAt(0, 0, 0);

    renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: false });
    renderer.setSize(canvas.clientWidth, canvas.clientHeight);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setClearColor(0x0a0a0a, 1);
    renderer.toneMapping = THREE.ACESFilmicToneMapping;
    renderer.toneMappingExposure = 1.2;

    clock = new THREE.Clock();

    // Lighting
    const ambient = new THREE.AmbientLight(0x0a1a3a, 0.6);
    scene.add(ambient);

    const keyLight = new THREE.DirectionalLight(0x00ced1, 0.8);
    keyLight.position.set(100, 200, 150);
    scene.add(keyLight);

    const rimLight = new THREE.DirectionalLight(0x9370db, 0.4);
    rimLight.position.set(-100, 100, -150);
    scene.add(rimLight);

    const pointLight = new THREE.PointLight(0x00ced1, 0.5, 800);
    pointLight.position.set(0, 150, 0);
    scene.add(pointLight);

    // Background particles
    createParticles();

    // Ambient rings on XZ plane
    createAmbientRings();
}

function createParticles() {
    const count = 1000;
    const geo = new THREE.BufferGeometry();
    const positions = new Float32Array(count * 3);
    for (let i = 0; i < count; i++) {
        positions[i * 3] = (Math.random() - 0.5) * 1200;
        positions[i * 3 + 1] = (Math.random() - 0.5) * 800;
        positions[i * 3 + 2] = (Math.random() - 0.5) * 1200;
    }
    geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));

    const mat = new THREE.PointsMaterial({
        color: 0x00ced1,
        size: 1.5,
        transparent: true,
        opacity: 0.4,
        blending: THREE.AdditiveBlending,
        depthWrite: false,
    });

    particles = new THREE.Points(geo, mat);
    scene.add(particles);
}

function createAmbientRings() {
    ambientRings = new THREE.Group();
    const ringCount = 8;
    const maxR = 350;
    for (let i = 1; i <= ringCount; i++) {
        const r = maxR * (i / ringCount);
        const geo = new THREE.RingGeometry(r - 0.3, r + 0.3, 128);
        const mat = new THREE.MeshBasicMaterial({
            color: 0x00ced1,
            transparent: true,
            opacity: 0.04,
            side: THREE.DoubleSide,
            blending: THREE.AdditiveBlending,
            depthWrite: false,
        });
        const mesh = new THREE.Mesh(geo, mat);
        mesh.rotation.x = -Math.PI / 2;
        mesh.userData.baseOpacity = 0.04;
        mesh.userData.index = i;
        ambientRings.add(mesh);
    }
    scene.add(ambientRings);
}

function normToWorld(normX, normY) {
    return {
        x: (normX - 0.5) * WORLD_W,
        y: (normY - 0.5) * WORLD_H,
        z: 0,
    };
}

export function addOrb(id, colorHex, normX, normY, radius) {
    const color = new THREE.Color(colorHex);
    const pos = normToWorld(normX, normY);

    const group = new THREE.Group();
    group.position.set(pos.x, pos.y, pos.z);

    // Core sphere — glass-like material
    const coreGeo = new THREE.SphereGeometry(radius * 0.35, 32, 32);
    const coreMat = new THREE.MeshPhysicalMaterial({
        color: color,
        emissive: color,
        emissiveIntensity: 0.6,
        metalness: 0.1,
        roughness: 0.15,
        clearcoat: 1.0,
        clearcoatRoughness: 0.05,
        transparent: true,
        opacity: 0.9,
    });
    const coreMesh = new THREE.Mesh(coreGeo, coreMat);
    coreMesh.name = 'core';
    group.add(coreMesh);

    // Glow layers (3 additive BackSide spheres)
    const glowRadii = [0.7, 1.2, 1.8];
    const glowAlphas = [0.15, 0.08, 0.04];
    for (let i = 0; i < 3; i++) {
        const glowGeo = new THREE.SphereGeometry(radius * glowRadii[i], 24, 24);
        const glowMat = new THREE.MeshBasicMaterial({
            color: color,
            transparent: true,
            opacity: glowAlphas[i],
            side: THREE.BackSide,
            blending: THREE.AdditiveBlending,
            depthWrite: false,
        });
        const glowMesh = new THREE.Mesh(glowGeo, glowMat);
        glowMesh.name = 'glow_' + i;
        group.add(glowMesh);
    }

    group.userData = {
        id,
        baseRadius: radius,
        colorHex,
        color: color.clone(),
        pulsePhase: Math.random() * Math.PI * 2,
        filterRings: [],
        binauralRings: [],
    };

    scene.add(group);
    orbs.set(id, group);
}

export function updateOrb(id, normX, normY, radius, gain, freq, binaural, filterColors) {
    const group = orbs.get(id);
    if (!group) return;

    const pos = normToWorld(normX, normY);
    group.position.set(pos.x, pos.y, pos.z);
    group.userData.baseRadius = radius;

    // Update core emissive intensity based on gain
    const core = group.getObjectByName('core');
    if (core) {
        core.material.emissiveIntensity = 0.3 + gain * 0.7;
    }

    // Update filter rings
    updateFilterRings(group, filterColors);

    // Update binaural rings
    updateBinauralRings(group, binaural);
}

function updateFilterRings(group, filterColors) {
    const ud = group.userData;

    // Remove old filter rings
    for (const ring of ud.filterRings) {
        group.remove(ring);
        ring.geometry.dispose();
        ring.material.dispose();
    }
    ud.filterRings = [];

    // Add new filter rings
    for (let i = 0; i < filterColors.length; i++) {
        const colorStr = filterColors[i];
        const color = new THREE.Color(colorStr);
        const r = ud.baseRadius * (1.5 + i * 0.5);
        const geo = new THREE.RingGeometry(r - 0.6, r + 0.6, 64);
        const mat = new THREE.MeshBasicMaterial({
            color: color,
            transparent: true,
            opacity: 0.5,
            side: THREE.DoubleSide,
            blending: THREE.AdditiveBlending,
            depthWrite: false,
        });
        const mesh = new THREE.Mesh(geo, mat);
        mesh.userData.filterIndex = i;
        mesh.name = 'filter_' + i;
        group.add(mesh);
        ud.filterRings.push(mesh);
    }
}

function updateBinauralRings(group, active) {
    const ud = group.userData;

    if (active && ud.binauralRings.length === 0) {
        const color = ud.color;
        for (let i = 0; i < 2; i++) {
            const r = ud.baseRadius * (2.0 + i * 0.5);
            const geo = new THREE.RingGeometry(r - 0.4, r + 0.4, 64);
            const mat = new THREE.MeshBasicMaterial({
                color: color,
                transparent: true,
                opacity: 0.35,
                side: THREE.DoubleSide,
                blending: THREE.AdditiveBlending,
                depthWrite: false,
            });
            const mesh = new THREE.Mesh(geo, mat);
            mesh.name = 'binaural_' + i;
            group.add(mesh);
            ud.binauralRings.push(mesh);
        }
    } else if (!active && ud.binauralRings.length > 0) {
        for (const ring of ud.binauralRings) {
            group.remove(ring);
            ring.geometry.dispose();
            ring.material.dispose();
        }
        ud.binauralRings = [];
    }
}

export function removeOrb(id) {
    const group = orbs.get(id);
    if (!group) return;

    // Dispose all children
    group.traverse((child) => {
        if (child.geometry) child.geometry.dispose();
        if (child.material) {
            if (Array.isArray(child.material)) {
                child.material.forEach(m => m.dispose());
            } else {
                child.material.dispose();
            }
        }
    });
    scene.remove(group);
    orbs.delete(id);
}

export function removeAllOrbs() {
    for (const [id] of orbs) {
        removeOrb(id);
    }
}

export function render(time) {
    if (!renderer || !scene || !camera) return;

    const t = time; // time in seconds from Rust

    // Camera orbital drift
    cameraAngle += 0.0003;
    const camDist = 500;
    const camHeight = 80 + 15 * Math.sin(t * 0.1);
    camera.position.x = Math.sin(cameraAngle) * camDist;
    camera.position.z = Math.cos(cameraAngle) * camDist;
    camera.position.y = camHeight;
    camera.lookAt(0, 0, 0);

    // Animate particles
    if (particles) {
        particles.rotation.y += 0.0001;
    }

    // Animate ambient rings
    if (ambientRings) {
        for (const ring of ambientRings.children) {
            const i = ring.userData.index;
            ring.material.opacity = ring.userData.baseOpacity + 0.015 * Math.sin(t * 0.25 + i * 0.4);
        }
    }

    // Animate orbs
    for (const [, group] of orbs) {
        const ud = group.userData;
        const pulse = 1.0 + 0.06 * Math.sin(t * 1.2 + ud.pulsePhase);
        const r = ud.baseRadius * pulse;

        // Scale core
        const core = group.getObjectByName('core');
        if (core) {
            const s = r * 0.35 / (ud.baseRadius * 0.35 || 1);
            core.scale.setScalar(s);
        }

        // Scale glow layers
        const glowRadii = [0.7, 1.2, 1.8];
        for (let i = 0; i < 3; i++) {
            const glow = group.getObjectByName('glow_' + i);
            if (glow) {
                const s = r * glowRadii[i] / (ud.baseRadius * glowRadii[i] || 1);
                glow.scale.setScalar(s);
            }
        }

        // Animate filter rings — face camera
        for (const ring of ud.filterRings) {
            ring.lookAt(camera.position);
            const fi = ring.userData.filterIndex;
            const fp = 1.0 + 0.04 * Math.sin(t * 1.8 + fi * 0.5);
            ring.scale.setScalar(fp);
        }

        // Animate binaural rings
        for (let i = 0; i < ud.binauralRings.length; i++) {
            const ring = ud.binauralRings[i];
            ring.lookAt(camera.position);
            const bp = 1.0 + 0.12 * Math.sin(t * 4.0 + i);
            ring.scale.setScalar(bp);
            ring.material.opacity = 0.35 * (0.7 + 0.3 * Math.sin(t * 4.0 + i));
        }
    }

    // Connection lines
    updateConnectionLines();

    renderer.render(scene, camera);
}

function updateConnectionLines() {
    // Remove old lines
    for (const line of connectionLines) {
        scene.remove(line);
        line.geometry.dispose();
        line.material.dispose();
    }
    connectionLines = [];

    const orbArr = Array.from(orbs.values());
    for (let i = 0; i < orbArr.length; i++) {
        for (let j = i + 1; j < orbArr.length; j++) {
            const a = orbArr[i].position;
            const b = orbArr[j].position;
            const dist = a.distanceTo(b);
            if (dist < 300) {
                const alpha = 0.15 * (1.0 - dist / 300);
                const geo = new THREE.BufferGeometry().setFromPoints([a.clone(), b.clone()]);
                const mat = new THREE.LineBasicMaterial({
                    color: 0x00ced1,
                    transparent: true,
                    opacity: alpha,
                    blending: THREE.AdditiveBlending,
                    depthWrite: false,
                });
                const line = new THREE.Line(geo, mat);
                scene.add(line);
                connectionLines.push(line);
            }
        }
    }
}

export function resize(w, h) {
    if (!renderer || !camera) return;
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
    renderer.setSize(w, h);
}

export function getOrbAtPoint(px, py, w, h) {
    if (!camera || !scene) return -1;

    const mouse = new THREE.Vector2(
        (px / w) * 2 - 1,
        -(py / h) * 2 + 1
    );

    const raycaster = new THREE.Raycaster();
    raycaster.setFromCamera(mouse, camera);

    // Collect all orb core meshes
    const targets = [];
    for (const [id, group] of orbs) {
        // Use a larger invisible sphere for easier picking
        const ud = group.userData;
        const r = ud.baseRadius * 0.7;
        const pickGeo = new THREE.SphereGeometry(r, 8, 8);
        const pickMesh = new THREE.Mesh(pickGeo, new THREE.MeshBasicMaterial({ visible: false }));
        pickMesh.position.copy(group.position);
        pickMesh.userData.orbId = id;
        targets.push(pickMesh);
        scene.add(pickMesh);
    }

    const intersects = raycaster.intersectObjects(targets);

    // Clean up pick meshes
    for (const m of targets) {
        scene.remove(m);
        m.geometry.dispose();
        m.material.dispose();
    }

    if (intersects.length > 0) {
        return intersects[0].object.userData.orbId;
    }
    return -1;
}

export function getOrbScreenPos(id, w, h) {
    const group = orbs.get(id);
    if (!group || !camera) return null;

    const pos = group.position.clone();
    pos.project(camera);

    return {
        x: (pos.x * 0.5 + 0.5) * w,
        y: (-pos.y * 0.5 + 0.5) * h,
        radius: group.userData.baseRadius,
    };
}
