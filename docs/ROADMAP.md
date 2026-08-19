# LITHOS — Roadmap

## Phase 0 — Architecture Governance ✓
**Acceptance:**
- [x] PROJECT_BIBLE.md exists
- [x] ARCHITECTURE.md exists
- [ ] DEPENDENCY_GRAPH.md created
- [ ] SYSTEM_CONTRACT.md created
- [ ] DECISIONS.md created
- [ ] TECH_DEBT.md created
- [ ] OPEN_QUESTIONS.md created
- [ ] CURRENT_STATE.md created
- [ ] CHANGELOG.md created

---

## Phase 1 — Engine Foundation
**Goal:** Minimal working Rust application with Vulkan window, basic rendering loop, core infrastructure.

### 1.1 Project Setup
- [ ] `Cargo.toml` with workspace structure
- [ ] `rust-toolchain.toml` (stable, components: rustfmt, clippy)
- [ ] `.cargo/config.toml` (profiles, target, features)
- [ ] `justfile` or `Makefile` for common tasks
- [ ] CI: GitHub Actions (fmt, clippy, test, build)

### 1.2 Window & Vulkan Init
- [ ] `winit` window creation, event loop
- [ ] Vulkan instance, device, queues (graphics, compute, transfer)
- [ ] Swapchain, surface, presentation
- [ ] Validation layers (debug), GPU-assisted validation
- [ ] Frame synchronization (fences, semaphores, present modes)

### 1.3 Math Library
- [ ] Fixed-point types (i64, 1/1024m units) — `FixedVec3`, `FixedAABB`
- [ ] Float types (f32/f64) — `Vec3`, `Mat4`, `Quat`, `AABB`
- [ ] SIMD batch types — `Vec3x4`, `Vec3x8` for voxel ops
- [ ] Noise: Perlin, Simplex, Cellular, Domain Warp (deterministic, seeded)
- [ ] Geometry: ray/aabb, ray/triangle, frustum, intersection tests

### 1.4 Memory Management
- [ ] Arena allocator (per-frame, per-system)
- [ ] Pool allocator (fixed-size: entities, chunks, tasks, draw commands)
- [ ] Global heap tracker (leak detection, stats)
- [ ] Allocation guards (panic on OOM in debug)

### 1.5 Logging & Profiling
- [ ] `tracing` + `tracing-subscriber` (JSON, console, file)
- [ ] Tracy client integration (CPU zones, GPU zones, memory, locks)
- [ ] In-game console (command buffer, history, autocomplete)
- [ ] Structured event schema (component, level, span, fields)

### 1.6 Job System
- [ ] Work-stealing thread pool (num_cpus - 1 workers)
- [ ] Task graph with dependencies (DAG)
- [ ] Priorities: SimFixed > RenderPrep > AsyncIO > Background
- [ ] Profiling: task timers, dependency visualization
- [ ] Deterministic execution order for simulation tasks

### 1.7 Serialization
- [ ] Binary format: magic, version, schema_id, payload
- [ ] Schema registry (compile-time + runtime loadable)
- [ ] Derive macros for schema generation (or build-time codegen)
- [ ] Migration framework: version → version transformers
- [ ] Network delta compression (varint, zigzag, RLE, dictionary)

---

## Phase 2 — Voxel Sandbox Core
**Goal:** Infinite procedural voxel world, player movement, break/place, save/load.

### 2.1 Voxel Storage
- [ ] Coordinate system: World → Region(512³) → Chunk(32³) → Block
- [ ] Sparse hash map: `RegionKey → ChunkStorage`
- [ ] Chunk storage: palette + RLE compressed blocks
- [ ] Procedural base generation (seeded, versioned)
- [ ] Delta log: persistent modifications (append-only)
- [ ] LOD aggregate data: material majority, avg properties

### 2.2 Meshing
- [ ] Greedy meshing per chunk (face merging)
- [ ] Cross-chunk face merging (border stitching)
- [ ] GPU buffers: vertex (pos, normal, uv, material_id), index
- [ ] Indirect draw commands per chunk
- [ ] LOD mesh generation (3 levels: full, 1/2, 1/4)

### 2.3 Player & Camera
- [ ] Character controller (capsule, step, slope, crouch/sprint/swim/fly)
- [ ] Camera (first-person, third-person, smooth transitions)
- [ ] Input handling (movement, look, actions, UI navigation)

### 2.4 Collision & Physics (Basic)
- [ ] Voxel collision (AABB vs chunk blocks)
- [ ] Broad phase: spatial hash / sweep-and-prune
- [ ] Narrow phase: capsule vs voxel shapes
- [ ] Response: slide, step, push, stop

### 2.5 Break / Place
- [ ] Raycast (voxel traversal algorithm)
- [ ] Break: tool effectiveness, drop rules, particle FX
- [ ] Place: collision check, rotation, snap, preview ghost
- [ ] Undo/redo buffer (local, limited)

### 2.6 Save / Load
- [ ] World save: meta + regions + deltas + entities + players
- [ ] Chunk serialization (compressed palette + RLE)
- [ ] Delta log serialization
- [ ] Load: streaming, priority by distance
- [ ] Migration test: old save → new version

---

## Phase 3 — Streaming & Procedural World
**Goal:** Planetary scale, deterministic generation, seamless streaming, LOD.

### 3.1 Hierarchical Spatial Representation
- [ ] Planet → Region(512³) → Chunk(32³) → Block
- [ ] Quadtree/octree for planetary LOD
- [ ] Coordinate systems: planetary (spherical) ↔ local (flat)

### 3.2 Deterministic Generation
- [ ] Tectonics: plate boundaries, uplift, subduction
- [ ] Erosion: hydraulic, thermal, hydraulic+thermal
- [ ] Sedimentation: transport, deposition, compaction
- [ ] Hydrology: watersheds, rivers, lakes, aquifers
- [ ] Climate: latitude, elevation, precipitation, temperature
- [ ] Soil: parent material, climate, biota, time
- [ ] Ecology: biomes, vegetation, fauna distribution

### 3.3 Streaming
- [ ] Priority queue: distance, view direction, interaction
- [ ] Background generation threads
- [ ] GPU upload streaming (persistent mapped buffers)
- [ ] Unload: LRU + distance, keep modified chunks

### 3.4 LOD Transitions
- [ ] Geometry: mesh LOD + impostors for distant
- [ ] Simulation: individual → group → aggregate
- [ ] Visual: smooth cross-fade, geometric error metric

---

## Phase 4 — Material & Process Core
**Goal:** Physically-based materials, batch simulation, process framework.

### 4.1 Material System
- [ ] Material definitions (data-driven): 50+ base materials
- [ ] Properties: density, specific_heat, thermal_cond, melting, tensile, hardness, resistivity, optical
- [ ] Phase states: solid, liquid, gas, plasma (with transitions)
- [ ] Batch representation: homogeneous volumes
- [ ] Provenance tracking: natural, processed, recycled, synthesized

### 4.2 Process Framework
- [ ] Process definition: inputs, outputs, rate_law, energy, heat, conditions
- [ ] Reaction solver: implicit integration, conservation enforcement
- [ ] Phase change: latent heat, volume change, nucleation
- [ ] Mechanical: stress/strain, fracture, wear, fatigue
- [ ] Thermal: conduction, convection, radiation
- [ ] Electrical: circuit solver (modified nodal analysis)

### 4.3 Constraints & Failures
- [ ] Limits: temperature, pressure, stress, voltage, current
- [ ] Failure modes: melt, fracture, explode, short, degrade
- [ ] Wear: abrasion, corrosion, fatigue, creep, deposition
- [ ] Maintenance: inspection, repair, replacement, calibration

---

## Phase 5 — Logistics Networks
**Goal:** Unified transport for items, fluids, gas, heat, mechanical, power, data.

### 5.1 Network Abstraction
- [ ] Graph: nodes (machines, storage, junctions) + edges (pipes, belts, wires)
- [ ] Flow solver per carrier type (pressure-driven, discrete, circuit)
- [ ] Backpressure, congestion, priority, routing

### 5.2 Carrier Implementations
- [ ] Items: belt (discrete, collision), tube (pneumatic), robot (pathfinding)
- [ ] Fluids: 1D Navier-Stokes, pressure/flow, pumps, valves, tanks
- [ ] Gas: compressible flow, diffusion, compressors, atmosphere exchange
- [ ] Heat: conduction (solid), advection (fluid), radiation (surface)
- [ ] Mechanical: shafts, gears, belts, hydraulics (torque, RPM, power)
- [ ] Power: electrical (AC/DC), RF, laser (voltage, current, losses)
- [ ] Data: packets, bandwidth, latency, errors, routing

---

## Phase 6 — Manufacturing
**Goal:** Machines with tolerances, wear, quality, maintenance.

### 6.1 Machine Framework
- [ ] Machine definition: ports, processes, limits, control signals
- [ ] State machine: idle, running, fault, maintenance, calibrating
- [ ] Tolerances: input/output precision, positional, dimensional
- [ ] Wear models: per-component, usage-based, condition-based
- [ ] Quality: statistical process control, measurement, reject/rework

### 6.2 Core Machines
- [ ] Furnace (smelting, heat management, efficiency)
- [ ] Crusher/Grinder (size reduction, wear, power)
- [ ] Press/Forge (forming, force, precision, die wear)
- [ ] Lathe/Mill (machining, tool wear, tolerance, coolant)
- [ ] Assembler (pick-place, fasteners, alignment, torque)
- [ ] Extruder/Injection (plastics, metals, temperature, pressure)
- [ ] Chemical Reactor (vessels, stir, heat, pressure, catalyst)
- [ ] Centrifuge/Separator (density, RPM, balance, wear)

---

## Phase 7 — Survival Systems
**Goal:** Player needs with depth, not busywork.

### 7.1 Physiology
- [ ] Nutrition: calories, macros, micros, absorption, deficiency
- [ ] Hydration: water, electrolytes, temperature regulation
- [ ] Sleep: circadian, quality, deprivation effects
- [ ] Hygiene: disease, infection, wound care
- [ ] Health: injury, illness, poisoning, radiation, treatment
- [ ] Training: skill atrophy, practice, specialization

### 7.2 Environment
- [ ] Temperature: hypothermia, hyperthermia, clothing, shelter
- [ ] Atmosphere: O2, CO2, contaminants, pressure, filtration
- [ ] Radiation: types, shielding, dosage, sickness
- [ ] Pathogens: vectors, immunity, quarantine, vaccines

---

## Phase 8 — Biology & Ecology
**Goal:** Living systems integrated with geology/climate.

### 8.1 Plants
- [ ] Growth: photosynthesis, nutrients, water, light, CO2
- [ ] Reproduction: seeds, pollination, dispersal, genetics
- [ ] Competition: light, space, allelopathy, mycorrhizae
- [ ] Harvest: yield, quality, regrowth, sustainability

### 8.2 Animals
- [ ] AI: needs, senses, behavior trees, learning
- [ ] Physiology: same depth as player (scaled)
- [ ] Genetics: inheritance, mutation, selection, domestication
- [ ] Ecology: food webs, population dynamics, carrying capacity

### 8.3 Soil & Microbiome
- [ ] Composition: mineral, organic, water, air, biota
- [ ] Processes: decomposition, nitrification, fixation, weathering
- [ ] Management: tillage, amendment, rotation, cover crops

---

## Phase 9 — Chemistry & Electronics
**Goal:** Reaction networks, sensors, logic, control systems.

### 9.1 Chemistry
- [ ] Species: elements, compounds, ions, radicals
- [ ] Reactions: kinetics, equilibrium, catalysis, pathways
- [ ] Industrial: Haber-Bosch, electrolysis, cracking, synthesis
- [ ] Analysis: spectroscopy, chromatography, titration (player tools)

### 9.2 Electronics
- [ ] Components: R, L, C, diode, transistor, IC (behavioral models)
- [ ] Signals: analog, digital, mixed, RF, optical
- [ ] Logic: gates, flip-flops, counters, state machines
- [ ] Sensors: temp, pressure, flow, position, chemical, radiation
- [ ] Actuators: motors, valves, heaters, displays, comms

---

## Phase 10 — Computing
**Goal:** Programmable computers in-game.

### 10.1 Architecture
- [ ] ISA: custom RISC (32/64-bit), privileged modes, interrupts
- [ ] Microarchitecture: pipeline, cache, branch prediction, MMU
- [ ] Memory: RAM, ROM, flash, NVRAM, memory-mapped I/O
- [ ] Peripherals: timers, UART, SPI, I2C, GPIO, network, storage

### 10.2 Software Stack
- [ ] Bootloader → Kernel (microkernel) → Userspace
- [ ] FS: littlefs or custom (wear-leveling, atomic ops)
- [ ] Network: TCP/IP stack, custom protocols
- [ ] Languages: Forth (interactive), C (compiled), Lua/WASM (scripts)
- [ ] Apps: shell, editor, compiler, debugger, monitor, control

---

## Phase 11 — Robotics & AI
**Goal:** Autonomous agents, swarm coordination.

### 11.1 Actuators & Sensors
- [ ] Joints: revolute, prismatic, spherical (torque, position, force control)
- [ ] End effectors: gripper, tool changer, suction, magnetic
- [ ] Sensors: proprioception, vision, lidar, force/torque, tactile

### 11.2 Robot Types
- [ ] Manipulator arms (stationary, mobile base)
- [ ] Mobile: wheeled, tracked, legged, flying, swimming
- [ ] Swarm: small, simple, numerous, coordinated

### 11.3 Control & AI
- [ ] Motion planning: RRT*, trajectory optimization
- [ ] Task planning: HTN, GOAP, behavior trees
- [ ] Learning: RL (offline training, online adaptation)
- [ ] Multi-agent: consensus, auction, market-based coordination

---

## Phase 12 — Vehicles
**Goal:** Physics-based vehicles for land, rail, sea, air.

### 12.1 Ground
- [ ] Wheeled: suspension, steering, drivetrain, tire model
- [ ] Tracked: track tension, sprocket, idler, turning
- [ ] Legged: inverse kinematics, gait, balance

### 12.2 Rail
- [ ] Track: geometry, switches, signals, power
- [ ] Rolling stock: coupler, brake, bogie, adhesion

### 12.3 Marine
- [ ] Hull: displacement, hydrostatics, resistance, stability
- [ ] Propulsion: prop, jet, sail, foil
- [ ] Navigation: charts, GPS, radar, sonar, autopilot

### 12.4 Aviation
- [ ] Aerodynamics: lift, drag, moments, control surfaces
- [ ] Propulsion: prop, turbofan, rocket, electric
- [ ] Flight control: stability, autopilot, navigation
- [ ] Atmosphere: density, wind, turbulence, icing

---

## Phase 13 — Persistent Macro Simulation
**Goal:** Aggregate simulation for planetary scale, time acceleration.

### 13.1 Aggregation
- [ ] Factory → aggregate model (throughput, energy, waste, wear)
- [ ] Settlement → population, economy, infrastructure
- [ ] Region → resource flows, pollution, climate impact
- [ ] Planet → tectonics, atmosphere, oceans, biosphere

### 13.2 Time Acceleration
- [ ] 1× → 10× → 100× → 1000× → 10000×
- [ ] Selective: player vicinity real-time, distant accelerated
- [ ] Deterministic: same result regardless of acceleration path

### 13.3 Long-Run Stability
- [ ] Drift detection: conserved quantities, invariants
- [ ] Correction: periodic resync, constraint projection
- [ ] Memory: bounded growth, cleanup, compaction

---

## Phase 14 — Multiplayer
**Goal:** 50+ player dedicated server.

### 14.1 Server Architecture
- [ ] Headless simulation (no rendering)
- [ ] Tick rate: 20-60 Hz fixed
- [ ] Threading: simulation workers + network I/O

### 14.2 Authority & Replication
- [ ] Server-authoritative state
- [ ] Client prediction (movement, local interaction)
- [ ] Reconciliation (smooth correction)
- [ ] Interest management (hierarchical regions)

### 14.3 Network Protocol
- [ ] Binary, versioned, schema-driven
- [ ] Delta compression, priority, reliability tiers
- [ ] Bandwidth: <50 KB/s avg, <200 KB/s peak per player

### 14.4 Resilience
- [ ] Reconnect, resync, snapshot, delta replay
- [ ] Server restart without world loss
- [ ] Anti-cheat: server authority, input validation, rate limits

### 14.5 Stress Test
- [ ] 50 bots + 50 players
- [ ] Latency simulation (0-500ms, jitter, loss)
- [ ] Bandwidth saturation, packet flood, entity spam

---

## Phase 15 — Planetary Systems
**Goal:** Full Earth-like planet with dynamic systems.

### 15.1 Tectonics
- [ ] Plates: boundaries, motion, subduction, rifting
- [ ] Volcanism: magma, eruption, lava, ash, gas
- [ ] Seismicity: earthquakes, tsunamis, ground motion

### 15.2 Hydrology
- [ ] Oceans: currents, temperature, salinity, waves
- [ ] Groundwater: aquifers, recharge, extraction, subsidence
- [ ] Surface water: rivers, lakes, wetlands, floods

### 15.3 Climate
- [ ] Atmosphere: circulation, pressure, humidity, clouds
- [ ] Radiation: solar, terrestrial, albedo, greenhouse
- [ ] Seasons: axial tilt, orbit, Milankovitch cycles

### 15.4 Disasters
- [ ] Weather: storms, tornadoes, hurricanes, blizzards
- [ ] Geological: eruptions, quakes, landslides, tsunamis
- [ ] Ecological: fires, blooms, die-offs, invasions

---

## Phase 16 — Scientific Research
**Goal:** Player-driven discovery, measurement, experimentation.

### 16.1 Measurement
- [ ] Instruments: precision, accuracy, range, calibration, drift
- [ ] Properties: physical, chemical, thermal, electrical, nuclear
- [ ] Uncertainty: propagation, confidence intervals, significance

### 16.2 Experiments
- [ ] Design: hypothesis, variables, controls, replication
- [ ] Execution: automation, data logging, safety
- [ ] Analysis: statistics, modeling, curve fitting, hypothesis test

### 16.3 Knowledge Graph
- [ ] Facts: discovered properties, relationships, laws
- [ ] Uncertainty: confidence, conflicting evidence, unknowns
- [ ] Sharing: publications, peer review, replication, citation

### 16.4 Material Discovery
- [ ] Exploration: sampling, analysis, mapping
- [ ] Synthesis: parameter space, DOE, optimization
- [ ] Characterization: structure, properties, performance

---

## Phase 17 — Advanced Industry
**Goal:** Precision manufacturing, semiconductors, advanced energy, speculative materials.

### 17.1 Precision Manufacturing
- [ ] Metrology: nm-μm accuracy, interferometry, CMM
- [ ] Processes: lithography, etching, deposition, polishing
- [ ] Tolerance stacks, statistical control, Six Sigma

### 17.2 Semiconductor Abstraction
- [ ] Process nodes: feature size, layers, materials, yield
- [ ] Device models: MOSFET, bipolar, MEMs, photonic
- [ ] Design: schematic, layout, verification, tape-out

### 17.3 Advanced Energy
- [ ] Fission: reactor types, fuel cycle, waste, safety
- [ ] Fusion: confinement, heating, breeding, materials
- [ ] Storage: battery, supercap, flywheel, thermal, chemical, gravitational

### 17.4 Speculative Materials
- [ ] Metamaterials: negative index, cloaking, superlensing
- [ ] Exotic: metallic hydrogen, strange matter, monopoles
- [ ] Classification: SPECULATIVE, model-defined, upgrade path

---

## Phase 18 — Space
**Goal:** Orbit, spacecraft, Moon, planets.

### 18.1 Orbital Mechanics
- [ ] Two-body, n-body, perturbations, relativistic corrections
- [ ] Maneuvers: Hohmann, bi-elliptic, low-thrust, gravity assist
- [ ] Rendezvous: phasing, proximity, docking, berthing

### 18.2 Spacecraft
- [ ] Structure: loads, vibration, thermal, radiation
- [ ] Propulsion: chemical, electric, nuclear, sail, tether
- [ ] Life support: ECLSS, redundancy, failure modes
- [ ] Avionics: GNC, comms, power, thermal, data

### 18.3 Celestial Bodies
- [ ] Moon: regolith, ISRU, bases, mass driver
- [ ] Planets: Mars, Venus, gas giants, moons (procedural + real)
- [ ] Small bodies: asteroids, comets, mining, deflection

---

## Phase 19 — Procedural Universe
**Goal:** Stellar systems, unknown regions, exploration.

### 19.1 Stellar Generation
- [ ] Stars: mass, age, metallicity, type, evolution, multiples
- [ ] Systems: disks, planets, belts, resonances, migration
- [ ] Exotic: black holes, neutron stars, white dwarfs, rogue planets

### 19.2 Unknown Regions
- [ ] Procedural extrapolation beyond known physics
- [ ] Anomalies: spatial, temporal, physical law variation
- [ ] Discovery: sensors, probes, FTL (if implemented)

### 19.3 Exploration Gameplay
- [ ] Survey: orbital, atmospheric, surface, subsurface
- [ ] Claim: legal, economic, military, scientific
- [ ] Infrastructure: gates, relays, supply lines, colonies

---

## Phase 20 — Civilization
**Goal:** Emergent infrastructure, economy, population, organizations.

### 20.1 Infrastructure
- [ ] Networks: power, data, transport, logistics (planetary scale)
- [ ] Cities: zoning, growth, services, governance
- [ ] Megaprojects: space elevators, orbital rings, terraforming

### 20.2 Economy
- [ ] Resources: extraction, processing, trade, markets
- [ ] Labor: players, NPCs, robots, AI agents
- [ ] Finance: currency, credit, contracts, insurance, derivatives

### 20.3 Population
- [ ] Demographics: birth, death, migration, aging, skills
- [ ] Needs: hierarchy, satisfaction, unrest, revolution
- [ ] Culture: language, religion, ideology, technology adoption

### 20.4 Organizations
- [ ] Types: corp, coop, state, NGO, cult, militia
- [ ] Governance: hierarchy, democracy, market, algorithmic
- [ ] Conflict: competition, war, diplomacy, espionage, sabotage

---

## Phase 21 — Optimization & Release
**Goal:** Production readiness.

### 21.1 Massive Stress Testing
- [ ] 10k+ concurrent entities, 100+ players
- [ ] Planetary simulation at 1000× acceleration
- [ ] Memory: <16GB RAM, <8GB VRAM target
- [ ] Frame time: <16.6ms (60 FPS) typical, <33ms (30 FPS) worst

### 21.2 Compatibility
- [ ] Save migration: all version steps tested
- [ ] Mod API: stable, versioned, documented
- [ ] Platforms: Linux (primary), Windows (CI tested)
- [ ] GPU vendors: AMD, NVIDIA, Intel (Vulkan 1.2+)

### 21.3 Modding
- [ ] Content packs: items, materials, machines, biomes, planets
- [ ] Scripts: sandboxed WASM, resource quotas, deterministic
- [ ] Total conversions: new mechanics, UI, progression
- [ ] Workshop integration (Steam) or custom distribution

### 21.4 Polish
- [ ] Tutorial: progressive, interactive, contextual
- [ ] Accessibility: rebindable, colorblind, text size, screen reader
- [ ] Localization: framework, initial languages (EN, RU, CN, DE, ES, FR, JP)
- [ ] Performance settings: scalable quality, auto-detect

### 21.5 Release
- [ ] Beta: closed → open → stress weekends
- [ ] Launch: dedicated servers, matchmaking, friend invites
- [ ] Post-launch: weekly patches, monthly content, quarterly expansions

---

## Milestone Tracking

| Phase | Status | Target | Key Deliverable |
|-------|--------|--------|-----------------|
| 0 | 🟡 In Progress | Week 1 | All governance docs |
| 1 | ⬜ Pending | Week 2-3 | Running Vulkan app |
| 2 | ⬜ Pending | Week 4-6 | Playable voxel sandbox |
| 3 | ⬜ Pending | Week 7-10 | Planetary streaming |
| 4 | ⬜ Pending | Week 11-14 | Material/process core |
| 5 | ⬜ Pending | Week 15-18 | Logistics networks |
| 6 | ⬜ Pending | Week 19-24 | Manufacturing machines |
| 7 | ⬜ Pending | Week 25-28 | Survival systems |
| 8 | ⬜ Pending | Week 29-34 | Biology/ecology |
| 9 | ⬜ Pending | Week 35-40 | Chemistry/electronics |
| 10 | ⬜ Pending | Week 41-48 | Computing platform |
| 11 | ⬜ Pending | Week 49-56 | Robotics/AI |
| 12 | ⬜ Pending | Week 57-64 | Vehicles |
| 13 | ⬜ Pending | Week 65-72 | Macro simulation |
| 14 | ⬜ Pending | Week 73-80 | Multiplayer |
| 15 | ⬜ Pending | Week 81-90 | Planetary systems |
| 16 | ⬜ Pending | Week 91-100 | Research system |
| 17 | ⬜ Pending | Week 101-112 | Advanced industry |
| 18 | ⬜ Pending | Week 113-124 | Space |
| 19 | ⬜ Pending | Week 125-136 | Universe |
| 20 | ⬜ Pending | Week 137-148 | Civilization |
| 21 | ⬜ Pending | Week 149-156 | Release |

---

## Risk Register

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Vulkan complexity delays Phase 1 | Medium | High | Start minimal, use validation layers, prototype early |
| Voxel storage performance at planetary scale | High | High | Benchmark early, design for aggregation from start |
| Deterministic simulation across platforms | Medium | High | Fixed-point math, documented FP behavior, replay tests |
| Multiplayer desync | High | Critical | Server authority, deterministic core, reconciliation testing |
| Feature creep / scope explosion | High | High | Strict phase gates, non-goals documented, ADR for deviations |
| Burnout / velocity drop | Medium | High | Sustainable pace, clear milestones, regular retrospectives |
| Physics instability (explosions, NaN) | Medium | High | Conservation checks, bounds, implicit solvers, fuzz testing |
| Save format migration complexity | Medium | Medium | Schema versioning from day 1, automated migration tests |