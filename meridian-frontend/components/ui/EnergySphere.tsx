'use client';

import React, { useRef, useMemo } from 'react';
import { Canvas, useFrame } from '@react-three/fiber';
import { Sphere, Trail } from '@react-three/drei';
import * as THREE from 'three';

// --------------------------------------------------------
// Shaders for the "Tesla Coil" Plasma Sphere
// --------------------------------------------------------

const plasmaVertexShader = `
  varying vec2 vUv;
  varying vec3 vNormal;
  varying vec3 vPosition;
  uniform float uTime;

  void main() {
    vUv = uv;
    vNormal = normalize(normalMatrix * normal);
    vPosition = position;
    gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
  }
`;

const plasmaFragmentShader = `
  varying vec3 vNormal;
  varying vec3 vPosition;
  uniform float uTime;
  uniform vec3 uColorCore;
  uniform vec3 uColorArc;

  // Simplex 3D Noise 
  vec3 mod289(vec3 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
  vec4 mod289(vec4 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
  vec4 permute(vec4 x) { return mod289(((x*34.0)+1.0)*x); }
  vec4 taylorInvSqrt(vec4 r) { return 1.79284291400159 - 0.85373472095314 * r; }
  float snoise(vec3 v) {
    const vec2  C = vec2(1.0/6.0, 1.0/3.0) ;
    const vec4  D = vec4(0.0, 0.5, 1.0, 2.0);
    vec3 i  = floor(v + dot(v, C.yyy) );
    vec3 x0 = v - i + dot(i, C.xxx) ;
    vec3 g = step(x0.yzx, x0.xyz);
    vec3 l = 1.0 - g;
    vec3 i1 = min( g.xyz, l.zxy );
    vec3 i2 = max( g.xyz, l.zxy );
    vec3 x1 = x0 - i1 + C.xxx;
    vec3 x2 = x0 - i2 + C.yyy;
    vec3 x3 = x0 - D.yyy;
    i = mod289(i);
    vec4 p = permute( permute( permute(
              i.z + vec4(0.0, i1.z, i2.z, 1.0 ))
            + i.y + vec4(0.0, i1.y, i2.y, 1.0 ))
            + i.x + vec4(0.0, i1.x, i2.x, 1.0 ));
    float n_ = 0.142857142857;
    vec3  ns = n_ * D.wyz - D.xzx;
    vec4 j = p - 49.0 * floor(p * ns.z * ns.z);
    vec4 x_ = floor(j * ns.z);
    vec4 y_ = floor(j - 7.0 * x_ );
    vec4 x = x_ *ns.x + ns.yyyy;
    vec4 y = y_ *ns.x + ns.yyyy;
    vec4 h = 1.0 - abs(x) - abs(y);
    vec4 b0 = vec4( x.xy, y.xy );
    vec4 b1 = vec4( x.zw, y.zw );
    vec4 s0 = floor(b0)*2.0 + 1.0;
    vec4 s1 = floor(b1)*2.0 + 1.0;
    vec4 sh = -step(h, vec4(0.0));
    vec4 a0 = b0.xzyw + s0.xzyw*sh.xxyy ;
    vec4 a1 = b1.xzyw + s1.xzyw*sh.zzww ;
    vec3 p0 = vec3(a0.xy,h.x);
    vec3 p1 = vec3(a0.zw,h.y);
    vec3 p2 = vec3(a1.xy,h.z);
    vec3 p3 = vec3(a1.zw,h.w);
    vec4 norm = taylorInvSqrt(vec4(dot(p0,p0), dot(p1,p1), dot(p2, p2), dot(p3,p3)));
    p0 *= norm.x;
    p1 *= norm.y;
    p2 *= norm.z;
    p3 *= norm.w;
    vec4 m = max(0.6 - vec4(dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3)), 0.0);
    m = m * m;
    return 42.0 * dot( m*m, vec4( dot(p0,x0), dot(p1,x1),
                                  dot(p2,x2), dot(p3,x3) ) );
  }

  void main() {
    // Basic Fresnel
    vec3 viewDir = normalize(cameraPosition - vPosition);
    float fresnel = dot(viewDir, vNormal);
    fresnel = pow(1.0 - abs(fresnel), 4.0);

    // Dynamic Electric Arcs
    // High frequency noise
    float noise1 = snoise(vPosition * 3.0 + vec3(uTime * 1.5));
    float noise2 = snoise(vPosition * 8.0 - vec3(uTime * 2.0));
    
    // Create sharp lines (electricity look)
    float electricity = abs(noise1 + noise2);
    electricity = 0.05 / electricity; // Inverse to make thin bright lines
    electricity = pow(electricity, 1.5);
    electricity = clamp(electricity, 0.0, 5.0);

    // Pulsing Core
    float pulse = sin(uTime * 3.0) * 0.2 + 0.8;

    vec3 finalColor = mix(uColorCore, uColorArc, electricity * 0.5);
    
    // Add rim glow + electric intensity
    finalColor += uColorArc * fresnel * 2.0;
    finalColor += uColorArc * electricity * 0.8;

    gl_FragColor = vec4(finalColor, min(1.0, (electricity + fresnel) * 0.8));
  }
`;

// --------------------------------------------------------
// Components
// --------------------------------------------------------

function ElectricSphere() {
    const mesh = useRef<THREE.Mesh>(null!);
    const uniforms = useMemo(
        () => ({
            uTime: { value: 0 },
            uColorCore: { value: new THREE.Color('#064e3b') }, // Deep Emerald
            uColorArc: { value: new THREE.Color('#6ee7b7') },  // Bright Electric Green/Teal
        }),
        []
    );

    useFrame((state) => {
        const { clock } = state;
        if (mesh.current) {
            // Slow down rotation
            mesh.current.rotation.y = clock.getElapsedTime() * 0.05;
            mesh.current.rotation.z = clock.getElapsedTime() * 0.02;
            // Slow down the plasma pulse effect by passing a slower time value
            (mesh.current.material as THREE.ShaderMaterial).uniforms.uTime.value = clock.getElapsedTime() * 0.4;
        }
    });

    return (
        <mesh ref={mesh} scale={2.2}>
            <sphereGeometry args={[1, 128, 128]} />
            <shaderMaterial
                vertexShader={plasmaVertexShader}
                fragmentShader={plasmaFragmentShader}
                uniforms={uniforms}
                transparent={true}
                blending={THREE.AdditiveBlending}
                depthWrite={false}
                side={THREE.DoubleSide}
            />
        </mesh>
    );
}

function Sparks({ count = 300 }) {
    const points = useRef<THREE.Points>(null!);

    // Create random start positions and velocities
    const [positions, randomness] = useMemo(() => {
        const pos = new Float32Array(count * 3);
        const rand = new Float32Array(count * 3);
        for (let i = 0; i < count; i++) {
            // Start on surface
            const theta = Math.random() * Math.PI * 2;
            const phi = Math.acos((Math.random() * 2) - 1);
            const r = 2.5;

            pos[i * 3] = r * Math.sin(phi) * Math.cos(theta);
            pos[i * 3 + 1] = r * Math.sin(phi) * Math.sin(theta);
            pos[i * 3 + 2] = r * Math.cos(phi);

            // Random orbital params
            rand[i * 3] = (Math.random() - 0.5) * 2; // Speed factor
            rand[i * 3 + 1] = Math.random() * Math.PI * 2; // Phase
            rand[i * 3 + 2] = Math.random() * 0.5 + 0.5; // Radius jitter
        }
        return [pos, rand];
    }, [count]);

    useFrame((state) => {
        const t = state.clock.getElapsedTime() * 0.3; // Slow global time for particles
        if (!points.current) return;

        const posArray = points.current.geometry.attributes.position.array as Float32Array;

        for (let i = 0; i < count; i++) {
            // Much slower individual speed
            const speed = 0.5 + randomness[i * 3] * 0.5;
            const phase = randomness[i * 3 + 1];
            const radius = 3.0 + Math.sin(t * speed + phase) * 0.5; // Pulsing orbit

            // Orbital motion
            const x = Math.cos(t * speed * 0.5 + phase) * radius;
            const y = Math.sin(t * speed * 0.3 + phase) * radius;
            const z = Math.sin(t * speed + phase) * radius * 0.5; // Elliptic

            // Update particle positions
            posArray[i * 3] = x;
            posArray[i * 3 + 1] = y;
            posArray[i * 3 + 2] = z;
        }
        points.current.geometry.attributes.position.needsUpdate = true;
    });

    return (
        <points ref={points}>
            <bufferGeometry>
                <bufferAttribute attach="attributes-position" count={count} array={positions} itemSize={3} />
            </bufferGeometry>
            <pointsMaterial
                size={0.05}
                color="#a7f3d0"
                transparent
                opacity={0.8}
                blending={THREE.AdditiveBlending}
            />
        </points>
    );
}

function MagneticRings() {
    return (
        <group>
            {/* Ring 1 - Fast Vertical */}
            <mesh rotation={[0, 0, Math.PI / 4]}>
                <torusGeometry args={[3.2, 0.01, 16, 100]} />
                <meshBasicMaterial color="#10b981" transparent opacity={0.3} blending={THREE.AdditiveBlending} />
            </mesh>
            {/* Ring 2 - Slow Horizontal */}
            <mesh rotation={[Math.PI / 2, 0, 0]}>
                <torusGeometry args={[3.8, 0.015, 16, 100]} />
                <meshBasicMaterial color="#34d399" transparent opacity={0.2} blending={THREE.AdditiveBlending} />
            </mesh>
        </group>
    )
}

export function EnergySphere() {
    return (
        <div className="w-full h-full relative">
            <Canvas
                camera={{ position: [0, 0, 7], fov: 45 }}
                gl={{ antialias: true, alpha: true, outputColorSpace: THREE.SRGBColorSpace }}
                dpr={[1, 2]}
            >
                <group scale={0.9}>
                    <ElectricSphere />
                    <Sparks count={400} />
                    <MagneticRings />
                </group>
            </Canvas>
        </div>
    );
}
