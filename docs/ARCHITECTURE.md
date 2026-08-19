# LITHOS — Architecture

## High-Level Structure

```
┌─────────────────────────────────────────────────────────────┐
│                        APPLICATION                            │
├─────────────┬─────────────┬─────────────┬───────────────────┤
│   ENGINE    │   GAME      │ SIMULATION  │    RENDERING      │
│  (core)     │  (logic)    │  (systems)  │    (GPU)          │
├─────────────┼─────────────┼─────────────┼───────────────────┤
│  • Memory   │  • Player   │  • Voxel    │  • Vulkan Core    │
│  • Jobs     │  • Input    │  • Materials│  • Mesh Pipeline  │
│  • Serde    │  • UI       │  • Processes│  • Material Sys   │
│  • Math     │  • Inventory│  • Logistics│  • Lighting       │
│  • Logging  │  • Crafting │  • Thermal  │  • Post-Process   │
│  • Profiler │  • Research │  • Mechanical│ • Debug Views    │
└─────────────┴─────────────┴─────────────┴───────────────────┘
         │           │           │                │
         └───────────┼───────────┼────────────────┘
                     ▼           ▼
              ┌─────────────────────────┐
              │      NETWORKING         │
              │  • Authority            │
              │  • Replication          │
              │  • Interest Mgmt        │
              │  • Prediction           │
              └─────────────────────────┘
                     │
                     ▼
              ┌─────────────────────────┐
              │      PERSISTENCE        │
              │  • World Save           │
              │  • Schema Versioning    │
              │  • Migration            │
              └─────────────────────────┘
```

---

## Core Principles

1. **Simulation ≠ Rendering** — Renderer is a pure consumer of simulation state
2. **Data-Oriented** — Systems operate on contiguous arrays, not object graphs
3. **Explicit Lifecycles** — Every system: init → update (fixed timestep) → shutdown
4. **Deterministic Core** — Same inputs → same simulation state (for replay/networking)
5. **Aggregation by Default** — Individual simulation only where necessary
6. **Schema-First** — All persistent/networked data defined in schemas, not code

---

## Module: Engine (Foundation)

### Memory
- **Arena allocators** per system/frame for temporary data
- **Pool allocators** for fixed-size frequent allocations (entities, chunks, tasks)
- **Global heap** only for long-lived heterogeneous data
- **Tracking**: allocation counts, bytes, leaks detected at shutdown

### Job System
- **Work-stealing thread pool** (rayon-style or custom)
- **Task graph** with explicit dependencies for frame work
- **Priorities**: Simulation (fixed) > Render prep > Async IO > Background
- **Profiling integration** — every task timed, dependencies visualized

### Serialization
- **Binary format**: custom, versioned, schema-driven (not serde reflection)
- **Schema registry** — compile-time verified, runtime loadable
- **Migration** — old schema → new schema transformers, tested
- **Network** — same schemas, delta compression, interest-filtered

### Math
- **Fixed-point** for simulation-deterministic positions (i64, 1/1024 meter units)
- **f32/f64** for rendering, physics solvers where determinism not required
- **SIMD** via `packed_simd` or `std::simd` for batch operations
- **Noise**: deterministic, seeded, multiple algorithms (Perlin, Simplex, Cellular, Domain Warp)

### Logging
- **Structured** — JSON lines, levels, component tags, span IDs
- **Sampling** — high-frequency events sampled, not dropped
- **Integration** — Tracy, stdout, file, in-game console

### Profiler
- **Tracy integration** — CPU, GPU, memory, locks, frame graph
- **Custom markers** — simulation phases, system timers, allocation sites
- **Continuous** — runs in release builds, minimal overhead

---

## Module: Simulation

### Voxel Storage
```
World → Regions (512³) → Chunks (32³) → Blocks
```
- **Sparse** — only modified chunks stored
- **Procedural base** — generated on-demand from seed + generator version
- **Deltas** — persistent modifications stored as (pos, old_state, new_state, tick)
- **Compression** — RLE + palette for homogeneous regions
- **LOD** — aggregate voxel data for distant regions (material majority, avg density, etc.)

### Meshing
- **Greedy meshing** per chunk, face merging across chunk boundaries
- **GPU-driven** — indirect draw, culling on GPU
- **Material ID per face** — packed into vertex attribute
- **LOD meshes** — simplified versions for distance

### Materials
- **Definition** (data): density, specific heat, thermal conductivity, melting point, tensile strength, hardness, electrical resistivity, optical properties, tags
- **Runtime state** (per batch): temperature, pressure, damage, contamination, phase
- **Batches** — homogeneous material volumes simulated together
- **Provenance** — track origin: natural, processed, recycled, synthesized

### Processes
- **Reaction** — inputs → outputs + energy/heat, rate law, catalyst, conditions
- **Phase change** — solid↔liquid↔gas↔plasma, latent heat, volume change
- **Mechanical** — stress/strain, fracture, wear, fatigue, creep
- **Thermal** — conduction, convection, radiation, generation
- **Electrical** — potential, current, resistance, capacitance, inductance
- **Chemical** — species concentrations, reaction network, equilibrium

### Logistics Networks
| Network | Carrier | Physics |
|---------|---------|---------|
| Items | Belts, tubes, robots, vehicles | Discrete, collision, jamming |
| Fluids | Pipes, tanks, pumps | Navier-Stokes (1D), pressure-driven |
| Gas | Pipes, ducts, atmosphere | Compressible flow, diffusion |
| Heat | Conduction, fluid advection, radiation | Energy conservation |
| Mechanical | Shafts, gears, belts, hydraulics | Torque, RPM, power, efficiency |
| Power | Electrical, RF, laser | Voltage, current, P=VI, losses |
| Data | Wires, fiber, radio, optical | Bits, bandwidth, latency, errors |

### Aggregation Strategy
- **Individual** — player vicinity, active machines, dynamic objects
- **Group** — same-type machines in same chunk, homogeneous fluid pipes
- **Aggregate** — distant factories, planetary atmosphere, ocean currents
- **Transitions** — bidirectional, state-preserving, validated

---

## Module: Game Logic

### Player
- **Character controller** — capsule collision, step height, slope limit, crouch/sprint/swim/fly
- **Inventory** — grid + hotbar, weight/volume limits, container access
- **Interaction** — raycast + use key, contextual (break, place, open, operate, connect)
- **Knowledge** — discovered recipes, material properties, research progress

### Crafting/Assembly
- **No fixed recipes** — instead: process definitions + available tools/machines
- **Manual** — player performs steps (hammer, saw, file, measure)
- **Tool-assisted** — jigs, guides, powered hand tools
- **Machine** — automated process with tolerances, wear, calibration
- **Quality** — output varies by input quality, machine precision, operator skill

### Research
- **Measurement** — instruments reveal material properties, process parameters
- **Experiments** — controlled trials, data collection, hypothesis testing
- **Knowledge graph** — discovered facts, relationships, uncertainties
- **Technology tree** — emergent from knowledge, not predefined

---

## Module: Rendering (Vulkan)

### Pipeline Architecture
```
Frame Graph
  ├── Shadow Pass (cascaded directional)
  ├── Depth Pre-pass
  ├── G-Buffer (PBR: albedo, normal, roughness, metallic, emissive, material_id)
  ├── Lighting (clustered forward / deferred hybrid)
  ├── Transparency (depth-peeled or stochastic)
  ├── Volumetrics (fog, clouds, gas)
  ├── Post-Process (tonemap, bloom, DOF, motion blur, AA)
  └── UI / Debug Overlay
```

### GPU-Driven Rendering
- **Indirect draws** — `vkCmdDrawIndexedIndirectCount`
- **Frustum/Occlusion culling** — compute shader, `DrawIndexedIndirectCount` with predication
- **Clustered shading** — 3D grid of light lists, frustum-aligned
- **Bindless** — descriptor indexing, all resources in arrays
- **Streaming** — texture/mesh streaming with priority, residency management

### Voxel Rendering
- **Chunk meshes** — uploaded to GPU buffers, indexed by chunk coordinate
- **LOD** — 3-4 levels, geometric error metric, smooth transitions
- **Instancing** — repeated structures (trees, rocks, machines) via instance buffers
- **Procedural detail** — shader-based micro-detail (normal maps, parallax)

### Materials (Rendering)
- **PBR metallic-roughness** — base from simulation material properties
- **Material views** — derived from simulation state (temperature→emissive, damage→roughness, wetness→specular)
- **No duplicate data** — renderer reads simulation material DB via derived views

---

## Module: Networking

### Authority
- **Server authoritative** — simulation runs on server
- **Client prediction** — movement, local interactions, UI
- **Reconciliation** — server state overrides, smooth correction

### Replication
- **State classes**:
  - `DeterministicReconstructable` — derived from seed + inputs (terrain, procedural)
  - `ReplicatedState` — authoritative, sent to relevant clients (entities, machines)
  - `ClientPredicted` — local prediction, server reconciles (player movement)
  - `ServerOnly` — never sent (AI plans, hidden state)

### Interest Management
- **Hierarchical regions** — planet → region (512³) → chunk (32³)
- **Relevance score** — distance + interaction + trajectory + importance + team
- **Dynamic subscription** — clients subscribe to region hierarchy levels

### Bandwidth Targets
- **< 50 KB/s / player** average
- **< 200 KB/s / player** peak
- **< 100 packets/s** average

---

## Module: Persistence

### Save Structure
```
save/
  ├── meta.json           # version, seed, generator_version, tick, playtime
  ├── world/
  │   ├── regions/        # sparse chunk data (compressed)
  │   ├── deltas/         # persistent modifications log
  │   └── procedural/     # generator parameters + persistent deltas
  ├── entities/           # serialized entity states
  ├── players/            # player data per UUID
  ├── simulation/         # aggregate simulation state
  └── research/           # knowledge graph, discovered facts
```

### Versioning
- `world_version` — major world format changes
- `save_schema_version` — serialization format
- `simulation_model_version` — physics/material model
- `network_protocol_version` — multiplayer compatibility
- `content_schema_version` — item/material/machine definitions
- `mod_api_version` — modding API

### Migration
- **Upgraders** — old version → new version transformers
- **Tested** — old save → migrate → load → verify key invariants
- **Rollback** — migration can fail safely, original save preserved

---

## Data Flow (Per Tick)

```
INPUT (network, player input, timers)
    ↓
SIMULATION (fixed timestep, 20-60 Hz)
    ├── Voxel updates (block changes, physics)
    ├── Process updates (machines, reactions, thermal)
    ├── Logistics updates (network flow solvers)
    ├── Entity updates (AI, vehicles, projectiles)
    └── Aggregate updates (distant regions, planetary)
    ↓
EVENTS (state changes, triggers, UI notifications)
    ↓
REPLICATION (determine dirty state, build packets)
    ↓
PERSISTENCE (async, batch dirty chunks/entities)
    ↓
RENDER PREP (extract visible state, build draw commands)
    ↓
GPU SUBMIT
```

---

## Configuration

All tunable parameters in `config/` as TOML:
- `engine.toml` — memory, jobs, logging, profiling
- `simulation.toml` — tick rate, aggregation thresholds, physics constants
- `rendering.toml` — quality settings, resolution, features
- `network.toml` — tick rate, bandwidth limits, interest params
- `worldgen.toml` — generator parameters, seeds

---

## Testing Strategy

| Level | Scope | Tools |
|-------|-------|-------|
| Unit | Individual functions, math, serialization | `cargo test` |
| Integration | Cross-system behavior, save/load, network | Custom test harness |
| Simulation | Conservation laws, invariants, long-run stability | Property-based (proptest) |
| Fuzz | Saves, packets, blueprints, mod data | `cargo fuzz` / libfuzzer |
| Stress | 10×, 100×, 1000× entity counts | Custom scenarios |
| Determinism | Replay same inputs → same state | Record/replay |
| Visual Regression | Screenshot comparison | Custom (headless Vulkan) |
| Performance | Benchmarks per system | `cargo bench` / criterion |

---

## Dependencies (External)

| Crate | Purpose | Justification |
|-------|---------|---------------|
| `winit` | Window/events | Standard, maintained |
| `ash` / `vk-sys` | Vulkan bindings | Low-level, no overhead |
| `bytemuck` | Zero-copy casting | Safe, fast |
| `bitvec` | Bit-packed voxel data | Memory efficient |
| `nalgebra` / `glam` | Math | SIMD, feature-gated |
| `parking_lot` | Fast locks | Lower overhead than std |
| `crossbeam` | Channels, epochs | Lock-free structures |
| `tracy-client` | Profiling | Industry standard |
| `serde` + `serde_json` | Config, metadata | Not for hot paths |
| `toml` | Config files | Human-readable |
| `uuid` | Entity/player IDs | Standard |
| `anyhow` / `thiserror` | Error handling | Ergonomic |
| `tracing` / `tracing-subscriber` | Structured logging | Flexible |

**Policy**: Minimize dependencies. Prefer std or small focused crates. Audit each addition.