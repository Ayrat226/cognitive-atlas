# LITHOS — Dependency Graph

## Module Dependencies

```
engine (core)
├── memory
├── jobs
├── math
├── logging
├── profiler
├── serialization
└── config

simulation
├── engine
│   ├── voxel_storage → engine.memory, engine.jobs, engine.serialization, engine.math
│   ├── meshing → engine.memory, engine.jobs, engine.math, rendering.gpu_buffers
│   ├── materials → engine.memory, engine.serialization, engine.math
│   ├── processes → engine.memory, engine.jobs, engine.math, simulation.materials
│   ├── logistics → engine.memory, engine.jobs, engine.math, simulation.materials, simulation.processes
│   ├── aggregation → engine.memory, engine.jobs, engine.math, simulation.*
│   └── worldgen → engine.memory, engine.jobs, engine.math, engine.noise

game
├── engine
├── simulation
│   ├── player → engine.input, simulation.voxel_storage, simulation.meshing
│   ├── inventory → engine.serialization, simulation.materials
│   ├── crafting → simulation.materials, simulation.processes, simulation.logistics
│   ├── research → simulation.materials, simulation.processes, game.knowledge
│   └── ui → engine.logging, rendering

rendering
├── engine
├── simulation (read-only)
│   ├── vulkan_core → engine.memory, engine.logging, engine.profiler
│   ├── mesh_pipeline → rendering.vulkan_core, simulation.meshing
│   ├── material_system → rendering.vulkan_core, simulation.materials
│   ├── lighting → rendering.vulkan_core
│   ├── post_process → rendering.vulkan_core
│   └── debug_views → rendering.vulkan_core

networking
├── engine
├── simulation (read-only)
├── game (read-only)
│   ├── authority → engine.jobs, engine.serialization
│   ├── replication → engine.serialization, engine.jobs, networking.authority
│   ├── interest_mgmt → simulation.voxel_storage, simulation.aggregation
│   └── prediction → game.player, networking.replication

persistence
├── engine
├── simulation
├── game
│   ├── world_save → engine.serialization, simulation.voxel_storage, simulation.materials
│   ├── entity_save → engine.serialization, game.*
│   ├── player_save → engine.serialization, game.player
│   └── migration → engine.serialization, persistence.*
```

---

## Data Type Dependencies

### Core Types (engine.math)
```
FixedVec3, FixedAABB → used by: voxel_storage, meshing, player, physics, worldgen
Vec3, Mat4, Quat → used by: rendering, meshing, player, camera, vehicles
Noise → used by: worldgen, meshing (procedural detail), materials (procedural textures)
```

### Simulation Types (simulation.*)
```
BlockId → voxel_storage, meshing, player, crafting, world_save
MaterialId → materials, processes, logistics, crafting, rendering.material_system
ProcessId → processes, logistics, machines, crafting
EntityId → player, machines, vehicles, robotics, entities, networking
ChunkKey → voxel_storage, meshing, worldgen, streaming, networking.interest_mgmt
RegionKey → voxel_storage, worldgen, streaming, networking.interest_mgmt, aggregation
BatchId → materials, processes, logistics, aggregation
```

### Network Types (networking.*)
```
ReplicationState → authority, replication, prediction
InterestRegion → interest_mgmt, replication, streaming
ClientInput → prediction, authority, replication
ServerSnapshot → authority, replication, reconnect
```

### Persistence Types (persistence.*)
```
WorldMeta → world_save, migration, networking.reconnect
ChunkData → world_save, voxel_storage, streaming
DeltaEntry → world_save, voxel_storage, aggregation
EntityState → entity_save, networking.replication
PlayerState → player_save, networking.replication
```

---

## System Producer/Consumer Map

| System | Produces | Consumes |
|--------|----------|----------|
| voxel_storage | BlockId, ChunkData, DeltaEntry | worldgen, meshing, player, processes, logistics, world_save |
| meshing | GPU buffers, DrawCommands | voxel_storage, rendering.mesh_pipeline |
| materials | MaterialId, BatchState, Properties | processes, logistics, crafting, rendering.material_system |
| processes | ProcessOutput, Heat, Waste | materials, logistics, machines, aggregation |
| logistics | FlowRates, Pressures, Routing | processes, machines, power, heat, data |
| aggregation | AggregateState, Throughput | voxel_storage, processes, logistics, networking.interest_mgmt |
| worldgen | ChunkData (procedural), BiomeMap | voxel_storage, meshing, materials, ecology |
| player | PlayerInput, PlayerState | voxel_storage, meshing, inventory, crafting, networking.prediction |
| crafting | Recipes (emergent), Products | materials, processes, player, machines |
| research | KnowledgeGraph, DiscoveredFacts | materials, processes, player, crafting |
| vulkan_core | Device, Queues, Swapchain | mesh_pipeline, material_system, lighting, post_process |
| mesh_pipeline | DrawCommands, IndirectBuffers | meshing, vulkan_core, material_system |
| material_system | MaterialViews, PBRParams | simulation.materials, mesh_pipeline, lighting |
| authority | ServerState, Events | replication, prediction, persistence |
| replication | ClientPackets, ServerPackets | authority, interest_mgmt, prediction |
| interest_mgmt | SubscriptionSet | voxel_storage, aggregation, replication |
| prediction | PredictedState, Corrections | player, replication, authority |
| world_save | SaveFile | voxel_storage, materials, entities, players, migration |
| migration | UpgradedSave | world_save, serialization, all versioned types |

---

## Circular Dependencies (Intentional & Documented)

| Cycle | Reason | Resolution |
|-------|--------|------------|
| materials ↔ processes | Materials define process properties; processes transform materials | Interface segregation: MaterialProvider trait, ProcessConsumer trait |
| logistics ↔ processes | Processes need transport; logistics need process rates | Event-driven: processes publish demand/supply; logistics subscribe |
| aggregation ↔ voxel_storage | Aggregation summarizes voxels; voxel storage provides detail | Explicit LOD transitions: aggregate → detail on demand, detail → aggregate on unload |
| player ↔ voxel_storage | Player breaks/places; voxel storage provides collision | Command pattern: player issues BlockChangeCommand; voxel_storage applies |
| networking.authority ↔ networking.replication | Authority produces state; replication consumes | Unidirectional data flow: authority → replication via channels |

---

## External Dependencies

| Crate | Used By | Version Policy |
|-------|---------|----------------|
| winit | engine.window, game.input | Lock to minor, test on update |
| ash / vk-sys | rendering.vulkan_core | Lock to patch, vendor-specific testing |
| bytemuck | engine.serialization, rendering.gpu_buffers | Conservative, audit unsafe |
| bitvec | simulation.voxel_storage | Stable API, minimal |
| nalgebra / glam | engine.math, rendering | One only, feature-gated SIMD |
| parking_lot | engine.jobs, engine.memory, networking | Stable, low churn |
| crossbeam | engine.jobs, networking | Stable |
| tracy-client | engine.profiler, rendering | Optional feature, no-op when disabled |
| serde / serde_json | engine.config, persistence.metadata | Not in hot paths |
| toml | engine.config | Stable |
| uuid | game.player, persistence.entity_save | Stable |
| anyhow / thiserror | All (error handling) | Stable |
| tracing / tracing-subscriber | engine.logging, all | Stable |

---

## Build-Time Dependency Graph (Cargo Workspace)

```
lithos (workspace)
├── lithos-engine (lib)
│   ├── lithos-memory
│   ├── lithos-jobs
│   ├── lithos-math
│   ├── lithos-logging
│   ├── lithos-profiler
│   ├── lithos-serialization
│   └── lithos-config
├── lithos-simulation (lib)
│   ├── lithos-voxel
│   ├── lithos-materials
│   ├── lithos-processes
│   ├── lithos-logistics
│   ├── lithos-aggregation
│   └── lithos-worldgen
├── lithos-game (lib)
│   ├── lithos-player
│   ├── lithos-inventory
│   ├── lithos-crafting
│   ├── lithos-research
│   └── lithos-ui
├── lithos-rendering (lib)
│   ├── lithos-vulkan
│   ├── lithos-mesh
│   ├── lithos-materials-render
│   ├── lithos-lighting
│   └── lithos-post
├── lithos-networking (lib)
│   ├── lithos-authority
│   ├── lithos-replication
│   ├── lithos-interest
│   └── lithos-prediction
├── lithos-persistence (lib)
│   ├── lithos-world-save
│   ├── lithos-entity-save
│   ├── lithos-player-save
│   └── lithos-migration
├── lithos-app (bin) → links all libs
├── lithos-server (bin) → links engine, simulation, networking, persistence
└── lithos-tests (lib) → test utilities, property tests, benchmarks
```

---

## Schema Dependencies (Versioned)

```
save_schema_v1
  ├── world_meta_v1
  ├── chunk_data_v1 (palette + RLE)
  ├── delta_entry_v1
  ├── entity_state_v1
  ├── player_state_v1
  └── simulation_state_v1

save_schema_v2 (example future)
  ├── world_meta_v2 (+ generator_version)
  ├── chunk_data_v2 (+ LOD aggregate data)
  ├── delta_entry_v2 (+ tick, author)
  ├── entity_state_v2 (+ component_bitset)
  ├── player_state_v2 (+ knowledge_graph_ref)
  └── simulation_state_v2 (+ aggregate_version)

Migration: v1 → v2 transformers for each type, tested in persistence.migration
```

---

## Network Protocol Dependencies

```
network_protocol_v1
  ├── handshake_v1 (versions, capabilities)
  ├── client_input_v1 (move, look, action, sequence)
  ├── server_snapshot_v1 (entity states, chunk updates, events)
  ├── replication_update_v1 (dirty state, interest changes)
  ├── rpc_v1 (admin, chat, trade, blueprint)
  └── disconnect_v1 (reason, reconnect_token)

Compatibility: client v1 ↔ server v1 only. v2 requires migration handshake.
```

---

## Content Definition Dependencies

```
materials.ron → materials, processes, rendering.material_system, crafting
processes.ron → processes, materials, machines, logistics
machines.ron → machines, processes, logistics, crafting, research
items.ron → items, materials, crafting, inventory, rendering
biomes.ron → worldgen, ecology, materials, rendering
technology.ron → research, crafting, machines, processes
```

All content files: stable IDs, versioned, validated on load, migration supported.