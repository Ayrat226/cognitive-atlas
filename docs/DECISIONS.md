# LITHOS — Architecture Decisions (ADR)

## ADR Format

Each decision records:
- **Status**: Proposed | Accepted | Superseded | Rejected
- **Context**: What problem we're solving
- **Decision**: What we chose
- **Consequences**: Trade-offs, risks, follow-up work
- **Related**: Other ADRs, docs, issues

---

## ADR-001: Rust as Primary Language
**Status**: Accepted  
**Date**: 2026-08-19

### Context
Need a systems language with memory safety, performance, concurrency, and ecosystem for game engine + simulation.

### Decision
Rust 2021 edition, stable toolchain. No nightly features unless absolutely necessary (then documented with migration plan).

### Consequences
- Steep learning curve for team members new to Rust
- Longer initial development vs C++/Unity/Godot
- **Benefits**: Memory safety eliminates entire bug classes, fearless concurrency, zero-cost abstractions, excellent WASM support for modding
- Cargo workspace manages modular architecture

### Alternatives Considered
- C++20: More familiar, but memory safety issues, no standard build system
- C# / .NET: Good productivity, but GC pauses, less control, heavier runtime
- Zig: Simpler, but smaller ecosystem, less mature tooling

---

## ADR-002: Vulkan as Rendering API
**Status**: Accepted  
**Date**: 2026-08-19

### Context
Need low-level, cross-platform GPU API for GPU-driven rendering, compute shaders, explicit control.

### Decision
Vulkan 1.2+ via `ash` crate. Minimum features: descriptor indexing, timeline semaphores, buffer device address, dynamic rendering.

### Consequences
- High initial complexity (boilerplate, synchronization)
- **Benefits**: Explicit control enables GPU-driven rendering, compute integration, fine-grained synchronization, multi-GPU, portable across Linux/Windows/Android
- Validation layers catch bugs early
- Shader language: SPIR-V (glsl → spirv via shaderc at build)

### Alternatives Considered
- wgpu: Higher-level, but abstraction overhead, less control over GPU-driven patterns
- DirectX 12: Windows only
- Metal: Apple only
- OpenGL: Legacy, no explicit control, poor compute integration

---

## ADR-003: Fixed-Point Math for Simulation Determinism
**Status**: Accepted  
**Date**: 2026-08-19

### Context
Simulation must be deterministic across platforms, builds, and time for replay, networking, and save/load consistency.

### Decision
Fixed-point (i64, Q52.11 = 1/2048 meter units) for all simulation positions, velocities, and time. f64 for intermediate physics calculations where determinism not required (rendering, audio, UI).

### Consequences
- **Benefits**: Bitwise identical results across x86_64, ARM64, WASM; no FP non-determinism from compiler optimizations, instruction ordering, or hardware differences
- Custom fixed-point types with operator overloading
- Conversion overhead at simulation/render boundary
- Range: ±2 million km (sufficient for planetary scale)
- Precision: ~0.5mm at planet scale

### Alternatives Considered
- f64 with strict compiler flags: Not guaranteed across architectures
- f32: Insufficient precision for planetary coordinates
- Integer mm: Insufficient range for planetary scale
- Decimal/bignum: Too slow for simulation core

---

## ADR-004: Data-Oriented Design / ECS-lite
**Status**: Accepted  
**Date**: 2026-08-19

### Context
Simulation involves millions of entities (voxels, batches, machines, particles). OOP with virtual dispatch and scattered memory kills performance.

### Decision
Custom data-oriented framework (not full ECS like Bevy/Flecs). Systems own contiguous arrays of plain structs. Entity references are indices (u32/u64) into system-owned arrays. No archetype migration overhead.

### Consequences
- **Benefits**: Cache-friendly, SIMD-friendly, explicit data flow, easy serialization, deterministic iteration order
- More boilerplate than ECS macros
- Cross-system queries require explicit join logic
- Entity lifetime managed by owning system

### Alternatives Considered
- Bevy ECS: Mature, but archetype migration overhead, dynamic queries, reflection overhead
- Flecs: Fast, but C API, less Rust-idiomatic
- Custom full ECS: Reinventing wheel, maintenance burden
- Pure OOP: Performance disaster at scale

---

## ADR-005: Sparse Voxel Storage with Procedural Base
**Status**: Accepted  
**Date**: 2026-08-19

### Context
Planetary scale (10¹⁵+ blocks) impossible to store fully. Must be sparse + procedural.

### Decision
- **Base terrain**: Deterministic procedural generation from seed (no storage)
- **Modifications**: Sparse hash map `RegionKey → ChunkStorage` (only changed chunks)
- **Deltas**: Append-only log of `(pos, old_block, new_block, tick, author)`
- **LOD aggregates**: Per-region material majority, average properties for distant simulation

### Consequences
- **Benefits**: Infinite world, minimal storage for unmodified areas, deterministic base generation
- Complexity: Generation must be fast enough for streaming
- Delta log grows unbounded → periodic compaction needed
- Multiplayer: Deltas replicated, base generated identically on all clients

### Alternatives Considered
- Dense octree: Memory explosion
- Chunked dense array: Still huge for planetary scale
- Pure procedural (no persistence): No player modifications
- Voxel database (SQLite/rocksdb): Overhead, not optimized for spatial access

---

## ADR-006: Server-Authoritative Networking with Client Prediction
**Status**: Accepted  
**Date**: 2026-08-19

### Context
Multiplayer with 50+ players, physics simulation, persistent world. Cheat resistance required.

### Decision
- Server runs full simulation at fixed tick rate (20-60 Hz)
- Clients predict local player movement and immediate interactions
- Server reconciles: sends authoritative state, client smoothly corrects
- Interest management: Hierarchical region subscription (planet → region → chunk)

### Consequences
- **Benefits**: Cheat-resistant, consistent simulation, scalable interest management
- Complexity: Prediction/reconciliation for movement, interaction latency
- Bandwidth: Must stay <50 KB/s avg per player
- Deterministic simulation required (ADR-003)

### Alternatives Considered
- Peer-to-peer: Cheat-prone, NAT traversal, sync complexity
- Client-authoritative: Cheat-prone, desync risk
- Lockstep: High latency sensitivity, not suitable for 50+ players
- Rollback netcode (GGPO): Complex for physics simulation, better for fighting games

---

## ADR-007: Schema-Driven Binary Serialization (Not Serde)
**Status**: Accepted  
**Date**: 2026-08-19

### Context
Save files, network packets, and mod data need versioning, migration, validation, and performance. Serde reflection is slow, schema-less, hard to migrate.

### Decision
Custom binary format with explicit schema registry:
- Compile-time schema derivation (proc macro or build.rs codegen)
- Runtime schema loading for mods
- Versioned schemas with migration transformers
- No reflection at runtime in hot paths

### Consequences
- **Benefits**: Fast (zero-copy where possible), schema validation, explicit migrations, mod-friendly, deterministic encoding
- More upfront work than `#[derive(Serialize)]`
- Schema evolution requires discipline
- Build-time codegen adds complexity

### Alternatives Considered
- Serde + bincode: Fast but no schema, migration hard, reflection overhead
- Protocol Buffers / FlatBuffers: Good but external tooling, less Rust-native
- JSON: Human-readable but slow, verbose, no schema enforcement
- msgpack: Schema-less, same migration issues

---

## ADR-008: Process-Based Simulation (Not Recipe-Based)
**Status**: Accepted  
**Date**: 2026-08-19

### Context
Core fantasy: technology emerges from constraints, not fixed recipes. "Build a steam engine from parts, not from recipe."

### Decision
Simulation models **processes** (physics/chemistry) not **recipes** (input items → output items).
- Processes define: inputs, outputs, rate laws, energy balance, constraints, failure modes
- Machines are process executors with ports, limits, wear, control signals
- Players combine machines, pipes, wires to create systems
- Quality emerges from input quality, machine precision, process control

### Consequences
- **Benefits**: Emergent technology, no hardcoded recipes, realistic engineering, modding-friendly
- Much harder to balance than recipe games
- Requires good UI/feedback for players to understand processes
- Tutorial/progression must teach principles, not recipes

### Alternatives Considered
- Recipe-based (Minecraft, Factorio): Easier to balance, but fixed technology tree
- Hybrid: Recipes for early game, processes for late game → inconsistency
- Pure physics simulation: Too detailed, computationally impossible

---

## ADR-009: Aggregation LOD for Simulation (Not Just Rendering)
**Status**: Accepted  
**Date**: 2026-08-19

### Context
Simulating every machine, pipe, voxel at full detail globally is impossible. Need LOD for simulation itself.

### Decision
Three-tier simulation LOD:
- **Individual** (player vicinity): Full per-entity, per-batch, per-voxel simulation
- **Group** (nearby): Homogeneous machines batched, fluid networks solved as groups
- **Aggregate** (distant): Region-level throughput, energy, heat, material flows

Transitions are bidirectional, state-preserving, with hysteresis.

### Consequences
- **Benefits**: Planetary scale simulation feasible, computational cost scales with player attention
- Complexity: Aggregation/disaggregation logic, state reconstruction, invariant preservation
- Must validate: aggregate → detail → aggregate = original (within tolerance)
- Events (explosions, failures) force disaggregation

### Alternatives Considered
- Rendering-only LOD: Simulation still runs everywhere → CPU bound
- Time dilation for distant: Doesn't reduce entity count, just slows it
- Pure statistical model: Loses causality, can't reconstruct detail

---

## ADR-010: Modding via Sandboxed WASM
**Status**: Accepted  
**Date**: 2026-08-19

### Context
Modding is core to longevity. Must be safe, performant, deterministic, cross-platform.

### Decision
- Mods: WebAssembly (WASM) modules, sandboxed (wasmtime/wasmer)
- Resource quotas: fuel (instructions), memory, wall time, syscalls
- Deterministic: Same inputs → same outputs (no host non-determinism)
- API: Versioned, capability-based, schema-validated
- Content packs: Data-only (RON/JSON), no code, loaded by all clients

### Consequences
- **Benefits**: Safe (no native code), portable (x86/ARM/WASI), deterministic, language-agnostic (Rust, C++, AssemblyScript, etc.)
- WASM runtime overhead (~10-20% vs native)
- API design critical: too restrictive limits mods, too open breaks sandbox
- Debugging: Need WASM-aware tooling

### Alternatives Considered
- Native DLLs: Unsafe, platform-specific, version hell
- Lua: Slower, GC pauses, harder to sandbox deterministically
- Rhai/GlueScript: Interpreted, slower, less mature tooling
- WASI Preview 2: Emerging, but not yet stable

---

## ADR-011: Time Acceleration via Selective Simulation
**Status**: Accepted  
**Date**: 2026-08-19

### Context
Player wants to speed up time (sleep, wait for crops, travel). Full simulation at 1000× impossible.

### Decision
Selective time acceleration:
- Player vicinity: Always real-time (1×)
- Active machines (player-owned, nearby): Up to 10× with sub-stepping
- Aggregate regions: Up to 10000× with scaled process rates
- Deterministic: Same result regardless of acceleration path (verified by replay)

### Consequences
- **Benefits**: Player controls time, simulation remains consistent
- Complexity: Different tick rates per region, event scheduling across rates
- Must handle: Events at boundaries, resource consumption scaling, wear scaling
- Save/load: Store current acceleration state per region

### Alternatives Considered
- Global time acceleration: Breaks player vicinity, multiplayer desync
- Skip simulation: State jumps, breaks causality, exploits possible
- Discrete event simulation: Complex for continuous processes (fluids, heat)

---

## ADR-012: No Global Garbage Collection in Simulation
**Status**: Accepted  
**Date**: 2026-08-19

### Context
Simulation runs at fixed tick rate (20-60 Hz). GC pauses cause frame spikes, non-determinism.

### Decision
- **Simulation core**: No GC. Arena/pool allocators, explicit lifetimes, `Drop` for cleanup
- **Engine/Rendering/Networking**: Standard Rust (Arc, Box, Vec) — GC not applicable
- **Scripting/WASM**: Wasmtime has its own GC, but quota-limited, deterministic
- **UI**: Can use GC (egui, etc.) — not in simulation hot path

### Consequences
- **Benefits**: Predictable frame times, no GC pauses, deterministic memory usage
- More manual memory management in simulation code
- Arena lifetimes must be carefully scoped (per-tick, per-frame, per-system)
- Leak detection at shutdown (all arenas must be empty)

### Alternatives Considered
- Global allocator with GC (Boehm, etc.): Non-deterministic, pause spikes
- Reference counting everywhere: Cache misses, atomic overhead, cycles
- Epoch-based reclamation (crossbeam-epoch): Good for lock-free, complex for general use

---

## ADR-013: Procedural Generation = Process-Based Geology
**Status**: Accepted  
**Date**: 2026-08-19

### Context
World generation should match simulation philosophy: processes, not noise layers.

### Decision
Generator simulates geological processes:
1. Tectonics → plate boundaries, uplift, subduction
2. Erosion → hydraulic, thermal, sediment transport
3. Sedimentation → deposition, compaction, lithification
4. Hydrology → watersheds, rivers, aquifers
5. Climate → precipitation, temperature, biome classification
6. Soil → parent material + climate + biota + time
7. Ecology → vegetation, fauna distribution

Each step: deterministic, seeded, versioned. Output = voxel materials + properties.

### Consequences
- **Benefits**: Geologically plausible, resources where geology puts them, cross-scale consistency, simulation can continue processes
- Slower than noise-based generation (but cached)
- Must validate: hydrology connectivity, no floating islands, biome transitions plausible
- Generator version stored in save → re-generation only on version change

### Alternatives Considered
- Noise layers (heightmap + biome noise): Fast but geologically nonsense, resources random
- Wave Function Collapse: Good for local structure, not planetary scale
- Pre-baked heightmaps: Not infinite, not modifiable

---

## ADR-014: Energy/Heat as First-Class Simulation Quantities
**Status**: Accepted  
**Date**: 2026-08-19

### Context
Industrial simulation requires energy balance and thermal management. Can't be afterthought.

### Decision
- **Energy**: Joules, conserved, tracked per batch, per machine, per network
- **Heat**: Thermal energy (Joules), temperature (Kelvin), transferred via conduction/convection/radiation
- **Power networks**: Separate logistics network (electrical, mechanical, thermal) with own solver
- **Machines**: Define energy input/output, efficiency curves, thermal limits

### Consequences
- **Benefits**: Realistic engineering constraints, waste heat management gameplay, energy automation meaningful
- More complex than "fuel → power" abstraction
- Must balance: Simulation cost vs. gameplay value
- Visual feedback critical (thermal vision, heat shimmer, glowing)

### Alternatives Considered
- Abstract "power" units: Simpler but loses thermal gameplay, efficiency trade-offs
- Only fuel consumption: No heat management, no thermal engineering
- Separate thermal sim (optional): Inconsistent, players ignore it

---

## ADR-015: Research System = Measurement + Experimentation
**Status**: Accepted  
**Date**: 2026-08-19

### Context
Technology progression should be discovery, not XP grinding.

### Decision
- **Measurement**: Instruments reveal material properties, process parameters (with precision/accuracy limits)
- **Experiments**: Player designs controlled trials, collects data, fits models
- **Knowledge graph**: Facts with confidence, dependencies, contradictions
- **Technology emergence**: Unlocked when sufficient knowledge + infrastructure exists

No tech tree. No research points. No "unlock laser → build laser."

### Consequences
- **Benefits**: Authentic science gameplay, emergent progression, rewards curiosity/rigor
- Hard to tutorialize, hard to balance "fun"
- Requires good data visualization, experiment design tools
- Multiplayer: Knowledge sharing, peer review, replication mechanics

### Alternatives Considered
- Traditional tech tree: Predictable, but contradicts core fantasy
- Research points (Factorio): Grindy, not discovery
- Blueprint finding (loot): Random, not earned through understanding

---

## Decision Index

| ADR | Title | Status |
|-----|-------|--------|
| 001 | Rust as Primary Language | Accepted |
| 002 | Vulkan as Rendering API | Accepted |
| 003 | Fixed-Point Math for Determinism | Accepted |
| 004 | Data-Oriented Design / ECS-lite | Accepted |
| 005 | Sparse Voxel + Procedural Base | Accepted |
| 006 | Server-Authoritative Networking | Accepted |
| 007 | Schema-Driven Binary Serialization | Accepted |
| 008 | Process-Based Simulation | Accepted |
| 009 | Aggregation LOD for Simulation | Accepted |
| 010 | Modding via Sandboxed WASM | Accepted |
| 011 | Selective Time Acceleration | Accepted |
| 012 | No GC in Simulation Core | Accepted |
| 013 | Process-Based World Generation | Accepted |
| 014 | Energy/Heat as First-Class | Accepted |
| 015 | Research = Measurement + Experiment | Accepted |

---

## Superseded/Rejected Decisions

*None yet — all initial decisions accepted.*