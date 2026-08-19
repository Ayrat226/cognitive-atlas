# LITHOS — System Contracts

## Overview

Each major system must define and adhere to its contract. This document specifies the required contracts for all core systems.

---

## 1. Voxel Storage Contract

### Interface
```rust
trait VoxelStorage {
    fn get_block(&self, pos: FixedVec3) -> BlockId;
    fn set_block(&mut self, pos: FixedVec3, block: BlockId) -> BlockChange;
    fn get_chunk(&self, key: ChunkKey) -> Option<&Chunk>;
    fn load_chunk(&mut self, key: ChunkKey) -> ChunkLoadResult;
    fn unload_chunk(&mut self, key: ChunkKey);
    fn iter_dirty_chunks(&self) -> impl Iterator<Item = ChunkKey>;
    fn save_dirty_chunks(&mut self) -> SaveResult;
}
```

### Invariants
- **Consistency**: `get_block` after `set_block` returns new value
- **Persistence**: All `BlockChange` events persisted to delta log
- **Procedural base**: Unmodified chunks generated deterministically from seed
- **Sparse storage**: Only modified chunks allocated in memory
- **LOD aggregate**: Each region maintains aggregate material data

### Performance
- `get_block`: O(1) average, <50ns
- `set_block`: O(1) average, <100ns (includes dirty tracking)
- `load_chunk`: <5ms (generation + decompression)
- Memory: <1KB per modified chunk (compressed)

### Tests
- Deterministic generation: same seed → same base chunks
- Round-trip: set → get → save → load → get = same
- Concurrent access: multiple readers, single writer per chunk
- Memory bounds: 10k modified chunks < 50MB

---

## 2. Meshing Contract

### Interface
```rust
trait Meshing {
    fn rebuild_chunk(&mut self, key: ChunkKey, voxel_data: &Chunk) -> MeshResult;
    fn rebuild_borders(&mut self, key: ChunkKey, neighbors: [&Chunk; 6]);
    fn get_mesh(&self, key: ChunkKey) -> Option<&GpuMesh>;
    fn get_lod_mesh(&self, key: ChunkKey, lod: u8) -> Option<&GpuMesh>;
    fn upload_dirty(&mut self, device: &VulkanDevice) -> UploadResult;
}
```

### Invariants
- **Watertight**: No T-junctions at chunk boundaries
- **Face merging**: Greedy merging within and across chunks
- **Material IDs**: Each face has correct material_id attribute
- **LOD consistency**: LOD meshes approximate full mesh within error bound
- **GPU residency**: Only visible/nearby meshes in VRAM

### Performance
- Rebuild 32³ chunk: <2ms single-threaded
- Border stitching: <0.5ms per chunk
- GPU upload: <1ms per 100 chunks (batched)
- VRAM: <2GB for 50k visible chunks (LOD0)

### Tests
- Visual regression: screenshot comparison for known scenes
- Watertight: no cracks at chunk boundaries (raycast test)
- LOD transition: smooth cross-fade, no popping
- Memory: VRAM usage within budget at max view distance

---

## 3. Materials Contract

### Interface
```rust
trait MaterialSystem {
    fn get_definition(&self, id: MaterialId) -> &MaterialDef;
    fn create_batch(&mut self, material: MaterialId, volume: f32, temp: f32) -> BatchId;
    fn get_batch(&self, id: BatchId) -> &MaterialBatch;
    fn merge_batches(&mut self, a: BatchId, b: BatchId) -> BatchId;
    fn split_batch(&mut self, id: BatchId, volume: f32) -> (BatchId, BatchId);
    fn update_batch_state(&mut self, id: BatchId, delta: BatchStateDelta);
}
```

### MaterialDef (Data-Driven)
```rust
struct MaterialDef {
    id: MaterialId,
    name: String,
    density: f32,              // kg/m³
    specific_heat: f32,        // J/(kg·K)
    thermal_conductivity: f32, // W/(m·K)
    melting_point: f32,        // K
    boiling_point: f32,        // K
    latent_heat_fusion: f32,   // J/kg
    latent_heat_vaporization: f32, // J/kg
    tensile_strength: f32,     // Pa
    hardness: f32,             // Mohs or Vickers
    electrical_resistivity: f32, // Ω·m
    optical: OpticalProps,     // albedo, roughness, metallic, emissive_base
    tags: TagSet,              // metal, stone, organic, radioactive, etc.
    provenance: Provenance,    // Natural, Processed, Recycled, Synthesized
}
```

### Invariants
- **Conservation**: Mass conserved in all process operations
- **Energy**: Thermal energy = mass × specific_heat × (temp - reference)
- **Phase**: State determined by temperature/pressure vs phase diagram
- **Provenance**: Tracked through all transformations
- **Rendering sync**: Optical properties derived from simulation state

### Performance
- Definition lookup: O(1) <10ns
- Batch create/merge/split: <1μs
- State update: <0.5μs per batch per tick
- Memory: <200 bytes per batch

### Tests
- Conservation: total mass constant in closed system
- Phase transitions: energy matches latent heats
- Thermal equilibrium: two batches same temp after contact
- Provenance: track through 10+ process steps

---

## 4. Processes Contract

### Interface
```rust
trait ProcessSystem {
    fn register_process(&mut self, def: ProcessDef) -> ProcessId;
    fn get_process(&self, id: ProcessId) -> &ProcessDef;
    fn evaluate(&mut self, ctx: &ProcessContext) -> ProcessResult;
    fn step(&mut self, dt: f32, batches: &mut [BatchId]) -> Vec<ProcessEvent>;
}
```

### ProcessDef (Data-Driven)
```rust
struct ProcessDef {
    id: ProcessId,
    name: String,
    category: ProcessCategory, // Chemical, Thermal, Mechanical, Electrical, Nuclear
    inputs: Vec<ProcessPort>,  // material_id, min_volume, max_temp, max_pressure, rate_factor
    outputs: Vec<ProcessPort>, // material_id, volume_per_input, temp_delta, energy_per_input
    rate_law: RateLaw,         // Arrhenius, Michaelis-Menten, Linear, Custom(fn)
    energy_balance: EnergyBalance, // input_energy, output_energy, heat_generated
    constraints: ProcessConstraints, // temp_range, pressure_range, catalyst, contamination_tolerance
    failure_modes: Vec<FailureMode>, // overtemp, overpressure, contamination, catalyst_poison
}
```

### Invariants
- **Mass conservation**: Σ input mass = Σ output mass (± tolerance)
- **Energy conservation**: Σ input energy + generated = Σ output energy + heat
- **Rate limits**: Rate ≥ 0, bounded by inputs, constraints
- **Determinism**: Same inputs + dt → same outputs (fixed-point time)
- **Failure handling**: Failure modes produce events, not panics

### Performance
- Evaluate: <10μs per process per tick
- Step 1000 processes: <5ms (parallel)
- Memory: <500 bytes per active process instance

### Tests
- Conservation: mass/energy balance in 1000 random process combos
- Rate laws: match analytical solutions for simple cases
- Edge cases: zero input, max constraint, catalyst depletion
- Determinism: replay 1000 ticks → identical state

---

## 5. Logistics Networks Contract

### Interface
```rust
trait LogisticsNetwork {
    fn add_node(&mut self, node: NetworkNode) -> NodeId;
    fn add_edge(&mut self, edge: NetworkEdge) -> EdgeId;
    fn remove_node(&mut self, id: NodeId);
    fn remove_edge(&mut self, id: EdgeId);
    fn solve(&mut self, dt: f32) -> SolveResult;
    fn get_flow(&self, edge: EdgeId) -> FlowState;
    fn get_pressure(&self, node: NodeId) -> f32;
}
```

### Network Types
| Carrier | Node Types | Edge Physics | Solver |
|---------|------------|--------------|--------|
| Items | Belt, Container, Machine, Robot | Discrete, collision, jamming | Event-driven, priority queue |
| Fluids | Pipe, Tank, Pump, Valve, Machine | 1D Navier-Stokes (pressure/flow) | Implicit pressure solve |
| Gas | Duct, Tank, Compressor, Atmosphere | Compressible flow, diffusion | Semi-implicit |
| Heat | Solid, Fluid, Radiator, Machine | Conduction + advection + radiation | Implicit thermal |
| Mechanical | Shaft, Gear, Belt, Hydraulic, Machine | Torque/RPM/power, efficiency | Algebraic (steady) + dynamic |
| Power | Bus, Generator, Load, Battery, Line | V/I/P, losses, AC/DC | Modified nodal analysis |
| Data | Router, Switch, Radio, Laser, Node | Packet switching, bandwidth, latency | Discrete event |

### Invariants
- **Flow conservation**: Σ in = Σ out + accumulation (per node)
- **Pressure/voltage continuity**: Single value per node
- **Capacity limits**: Flow ≤ capacity (backpressure propagated)
- **No perpetual motion**: Energy out ≤ Energy in + generated
- **Deterministic solve**: Same topology + boundary → same flow

### Performance
- Fluid solve (10k pipes): <2ms
- Item network (1k nodes): <1ms (event-driven)
- Power solve (5k nodes): <3ms
- Memory: <100 bytes per node/edge

### Tests
- Conservation: mass/energy balance in closed networks
- Backpressure: downstream blockage propagates upstream
- Oscillation: no limit cycles in stable configurations
- Scaling: 10× nodes → <10× solve time

---

## 6. Aggregation Contract

### Interface
```rust
trait AggregationSystem {
    fn should_aggregate(&self, region: RegionKey, viewer_dist: f32) -> bool;
    fn aggregate_region(&mut self, region: RegionKey) -> AggregateState;
    fn disaggregate_region(&mut self, region: RegionKey) -> DisaggregateResult;
    fn get_aggregate(&self, region: RegionKey) -> Option<&AggregateState>;
    fn update_aggregate(&mut self, region: RegionKey, delta: AggregateDelta);
}
```

### AggregateState
```rust
struct AggregateState {
    region: RegionKey,
    material_volumes: HashMap<MaterialId, f32>, // total volume per material
    avg_temperature: f32,
    total_thermal_energy: f32,
    process_throughputs: HashMap<ProcessId, f32>, // volume/tick
    power_consumption: f32,
    heat_generation: f32,
    entity_counts: HashMap<EntityType, u32>,
    logistics_flows: LogisticsAggregateFlows,
    last_updated: Tick,
}
```

### Invariants
- **State preservation**: Disaggregate → aggregate → disaggregate = original (within tolerance)
- **Conservation**: Aggregate totals match sum of individual components
- **Transition threshold**: Hysteresis prevents thrashing (e.g., aggregate at 200m, disaggregate at 150m)
- **Event fidelity**: Significant events (explosions, failures) force disaggregation
- **Time coherence**: Aggregate stepped at same rate as individual (or with documented scaling)

### Performance
- Aggregate region (512³): <10ms
- Disaggregate region: <50ms (streaming chunks)
- Update per tick: <1ms per active aggregate
- Memory: <10KB per aggregate region

### Tests
- Round-trip: detail → aggregate → detail = original (100 random regions)
- Conservation: aggregate totals match detail sums
- Transition: no flickering at boundary distances
- Long-run: 1M ticks aggregated → no drift in conserved quantities

---

## 7. Rendering Contract

### Interface
```rust
trait Renderer {
    fn begin_frame(&mut self, view: &ViewParams) -> FrameContext;
    fn submit_mesh(&mut self, ctx: &mut FrameContext, mesh: &GpuMesh, material: MaterialView, transform: Mat4);
    fn submit_instanced(&mut self, ctx: &mut FrameContext, mesh: &GpuMesh, instances: &[InstanceData]);
    fn end_frame(&mut self, ctx: FrameContext) -> FrameResult;
    fn resize(&mut self, width: u32, height: u32);
    fn reload_shaders(&mut self) -> Result<(), ShaderError>;
}
```

### Invariants
- **Simulation independence**: Renderer reads simulation state, never writes
- **Material views**: Derived from simulation MaterialDef + BatchState (temp→emissive, damage→roughness)
- **Frame budget**: <16.6ms (60 FPS) at target resolution/quality
- **VRAM budget**: <8GB at max settings
- **Deterministic output**: Same simulation state + view → same pixels (for testing)

### Performance Targets
| Scene | 1080p | 1440p | 4K |
|-------|-------|-------|-----|
| Voxel world (50k chunks) | 8ms | 12ms | 28ms |
| Industrial (10k machines) | 6ms | 9ms | 20ms |
| Mixed | 10ms | 15ms | 35ms |

### Tests
- Visual regression: golden images for 50 reference scenes
- Performance: automated benchmark suite (CI)
- Memory: VRAM leak test (1000 frame loop)
- Correctness: validation layers clean, no sync errors

---

## 8. Networking Contract

### Interface
```rust
trait NetworkServer {
    fn start(&mut self, config: ServerConfig) -> Result<(), NetError>;
    fn tick(&mut self, dt: f32) -> ServerTickResult;
    fn handle_packet(&mut self, from: ClientId, packet: ClientPacket) -> HandleResult;
    fn get_client_state(&self, id: ClientId) -> ClientConnectionState;
}

trait NetworkClient {
    fn connect(&mut self, addr: SocketAddr) -> Result<(), NetError>;
    fn tick(&mut self, dt: f32) -> ClientTickResult;
    fn send_input(&mut self, input: PlayerInput);
    fn receive_updates(&mut self) -> Vec<ServerUpdate>;
}
```

### Invariants
- **Server authority**: Simulation state only modified by server
- **Client prediction**: Local prediction for movement, corrected by server
- **Interest management**: Clients only receive relevant state
- **Deterministic replay**: Same inputs → same server state
- **Bandwidth**: <50 KB/s avg, <200 KB/s peak per client
- **Latency tolerance**: Playable at 200ms RTT, degraded at 500ms

### Performance
- Server tick (50 players): <10ms
- Packet serialization: <0.1ms per packet
- Interest management: <1ms per tick
- Memory: <10MB per connected client

### Tests
- Reconnect: disconnect → reconnect → state sync
- Packet loss: 10% loss → no desync
- Latency: 200ms RTT → <2 correction frames
- Stress: 50 bots + 50 players → stable
- Determinism: replay recorded inputs → identical server state

---

## 9. Persistence Contract

### Interface
```rust
trait Persistence {
    fn save_world(&mut self, path: &Path, world: &World) -> SaveResult;
    fn load_world(&mut self, path: &Path) -> LoadResult<World>;
    fn save_player(&mut self, path: &Path, player: &PlayerState) -> SaveResult;
    fn load_player(&mut self, path: &Path) -> LoadResult<PlayerState>;
    fn migrate_save(&mut self, from_version: u32, to_version: u32, data: &mut [u8]) -> MigrationResult;
}
```

### Invariants
- **Schema versioning**: Every save has version, migrations tested
- **Atomic writes**: Write to temp, fsync, rename (no partial saves)
- **Backward compatibility**: Load old saves (with migration)
- **Forward compatibility**: New code loads old saves (ignore unknown fields)
- **Integrity**: Checksums on critical data, corruption detection
- **Performance**: Save <5s, Load <10s for typical world

### Performance
- Incremental save (dirty chunks only): <1s
- Full save: <5s
- Load (streaming): <3s to playable, <30s complete
- Save size: <500MB for heavily modified 1000-chunk world

### Tests
- Round-trip: save → load → save → load = identical (ignoring timestamps)
- Migration: v1 → v2 → v3 → vN all tested
- Corruption: truncated file → graceful error, backup preserved
- Concurrent: save during gameplay → no stalls >100ms
- Stress: 100 saves in a row → no leaks, no corruption

---

## 10. Game Logic Contracts

### Player Contract
- **Movement**: Deterministic, fixed-timestep, server-authoritative with client prediction
- **Inventory**: Grid-based, weight/volume limits, container hierarchy
- **Interaction**: Raycast-based, contextual, server-validated
- **Knowledge**: Discovered recipes, material properties, research progress (persisted)

### Crafting/Assembly Contract
- **No fixed recipes**: Process-based, tool/machine dependent
- **Quality**: Output varies by input quality, tool precision, operator skill
- **Automation ladder**: Manual → Tool → Machine → Automation → Software → AI
- **Reversibility**: Disassembly possible (with loss), recycling defined

### Research Contract
- **Measurement**: Instruments have precision, accuracy, calibration requirements
- **Experiments**: Controlled trials, data logging, statistical analysis
- **Knowledge graph**: Facts with confidence, dependencies, contradictions
- **Technology emergence**: Unlocked by knowledge, not predefined tree

---

## Contract Compliance Checklist

For each system implementation, verify:

- [ ] Interface matches contract
- [ ] All invariants hold (tested)
- [ ] Performance targets met (benchmarked)
- [ ] Tests pass (unit, integration, property, fuzz)
- [ ] Documentation complete
- [ ] Integration with dependent systems verified
- [ ] Impact audit completed (for new systems)
- [ ] Schema versioned (if persistent/networked)
- [ ] Migration tested (if schema change)
- [ ] Exploit audit passed