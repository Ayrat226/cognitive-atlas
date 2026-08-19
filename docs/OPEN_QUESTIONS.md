# LITHOS — Open Questions

## Classification

| Category | Meaning |
|----------|---------|
| **ARCH** | Architecture/design decision needed |
| **SCI** | Scientific/physical model uncertainty |
| **GAME** | Gameplay/balance/UX decision |
| **PERF** | Performance/scaling unknown |
| **TECH** | Technical implementation choice |
| **POL** | Policy/governance decision |

Each question tracks:
- **Status**: Open | Researching | Prototyping | Decided (see ADR)
- **Priority**: Critical | High | Medium | Low
- **Blocker**: What depends on this answer

---

## Open Questions

### QUEST-001: Fixed-Point vs f64 for Intermediate Physics Math
**Category**: ARCH/TECH  
**Status**: Open  
**Priority**: High  
**Blocker**: Physics solver implementation (Phase 4)

**Question**: ADR-003 mandates fixed-point for positions/time. But physics solvers (implicit integration, matrix solves, eigenvalue problems) need f64 precision and standard library support. Where exactly is the boundary?

**Considerations**:
- Option A: Fixed-point everywhere (custom linear algebra, no std math) — maximum determinism, high dev cost
- Option B: Fixed-point for state, f64 for solver internals — pragmatic, but conversion boundary must be carefully defined
- Option C: f64 with deterministic compiler flags (`-C target-cpu=generic`, `-C opt-level=2`, no fast-math) — simpler, but not guaranteed across architectures

**Research Needed**: 
- Test f64 determinism across x86_64 (Intel/AMD), ARM64 (Apple M-series, AWS Graviton)
- Benchmark fixed-point linear algebra vs f64
- Prototype both for a simple implicit heat conduction solver

---

### QUEST-002: Voxel Coordinate System — Planetary vs Flat
**Category**: ARCH/SCI  
**Status**: Open  
**Priority**: High  
**Blocker**: World generation, streaming, rendering (Phase 3)

**Question**: How to handle planetary curvature in voxel coordinates?

**Options**:
- A: Local flat coordinates per region (512³ = ~1km at 2m/voxel). Stitch regions with transform. Simple rendering, but seams at region boundaries.
- B: Global spherical coordinates (ECEF or geodetic). No seams, but voxel grid distorts at poles, math complex.
- C: Cubed sphere — 6 faces, each flat grid. Good for planetary, but cube edges need stitching.
- D: Octahedral sphere — better distortion, but more complex.

**Constraints**: 
- Voxel size: 2m (Minecraft-like) to 0.5m (detail)
- Planet radius: ~6371km (Earth)
- Must support caves, overhangs (not heightmap)

**Research Needed**: Prototype cubed-sphere voxel addressing, test seam artifacts, benchmark coordinate transforms.

---

### QUEST-003: Process Rate Law Standardization
**Category**: SCI/ARCH  
**Status**: Open  
**Priority**: High  
**Blocker**: Process system implementation (Phase 4)

**Question**: What rate law formalism covers chemical, thermal, mechanical, electrical, nuclear processes uniformly?

**Options**:
- A: Generic `rate = k * Π(input_concentration^order) * f(T, P, catalyst)` with Arrhenius `k = A * exp(-Ea/RT)` — covers chemistry, some thermal
- B: Separate formalisms per domain (chemical kinetics, Fourier heat conduction, Ohm's law, Norton's theorem) — accurate but fragmented
- C: Unified "process graph" with port-based energy/mass flow, each edge has conductance/resistance — like bond graphs
- D: Data-driven: ML surrogate models trained on high-fidelity sim — flexible but opaque, non-deterministic training

**Requirements**:
- Must conserve mass/energy exactly
- Must support stiff systems (implicit solve)
- Must be data-definable (not code per process)
- Must support catalysis, inhibition, saturation

**Research Needed**: Survey bond graph literature, chemical engineering process simulators (gPROMS, Aspen), game implementations (Oxygen Not Included, Stationeers).

---

### QUEST-004: Fluid Simulation Fidelity vs Performance
**Category**: SCI/PERF  
**Status**: Open  
**Priority**: High  
**Blocker**: Logistics networks (Phase 5)

**Question**: What fluid model for pipes/tanks? 1D Navier-Stokes (pressure/flow) vs. simplified pressure-driven flow vs. particle-based?

**Options**:
- A: Full 1D compressible Navier-Stokes (Euler + friction + heat) — accurate, stiff, expensive
- B: Incompressible pressure-velocity (SIMPLE algorithm) — standard for pipe networks, assumes low Mach
- C: Lumped parameter: `flow = conductance * (P1 - P2)`, `dP/dt = (flow_in - flow_out) / compliance` — fast, scalable, less accurate for transients
- D: Particle/SPH in pipes — visual, but overkill for logistics

**Constraints**:
- 10k+ pipe segments in large factory
- Must handle: compressible gas, incompressible liquid, phase change, water hammer
- Must couple with thermal network

**Research Needed**: Benchmark A vs B vs C for 1k, 10k, 100k pipes. Test water hammer, phase change stability.

---

### QUEST-005: Aggregation State Reconstruction Fidelity
**Category**: ARCH/SCI  
**Status**: Open  
**Priority**: Critical  
**Blocker**: Aggregation LOD (Phase 3, 13)

**Question**: When disaggregating an aggregate region, how to reconstruct individual machine/batch state from aggregate throughput/temperature/energy?

**Problem**: Aggregate loses: individual machine wear, exact batch temperatures, queue states, control signals. Reconstructing "plausible" detail is ill-posed.

**Options**:
- A: Store "disaggregation seed" — deterministic RNG state to reconstruct detail pseudo-randomly. But must match original if it was previously detailed.
- B: Only aggregate regions never before detailed. Once detailed, never re-aggregate (or keep full detail forever). Simple but memory grows.
- C: Store sufficient statistics: per-machine wear distribution, batch temperature histogram, queue length distribution. Reconstruct by sampling.
- D: Hybrid: Aggregate only homogeneous groups (same machine type, same recipe). Disaggregate by cloning template + distributing aggregate totals proportionally.

**Constraints**:
- Must be deterministic (same aggregate → same detail)
- Must conserve mass/energy exactly
- Must not create exploits (duplication via aggregate→detail→aggregate)

**Research Needed**: Prototype with factory simulation. Measure round-trip error after 100 aggregate/detail cycles.

---

### QUEST-006: Electrical Network Solver for Mixed AC/DC/RF
**Category**: SCI/TECH  
**Status**: Open  
**Priority**: Medium  
**Blocker**: Power logistics (Phase 5), Electronics (Phase 9)

**Question**: How to simulate electrical networks spanning DC (batteries, solar), AC (generators, motors), RF (wireless, comms), and digital logic?

**Options**:
- A: Modified Nodal Analysis (MNA) with complex phasors for AC, time-domain for transients — standard SPICE approach, but heavy
- B: Separate solvers per domain: DC (linear), AC (phasor), RF (S-parameters), Digital (event-driven) — modular but coupling complex
- C: Behavioral models only: components define V-I curves, power in/out, efficiency curves. No field simulation.
- D: Hybrid: MNA for power distribution (low freq), behavioral for components, digital logic separate

**Constraints**:
- Must handle: transformers, rectifiers, inverters, switching supplies, transmission lines
- Real-time performance: 10k+ nodes at 60 Hz
- Deterministic

**Research Needed**: Survey game electrical sims (Factorio, Stationeers, Minecraft mods), SPICE performance, behavioral modeling literature.

---

### QUEST-007: Mechanical Power Transmission Detail Level
**Category**: SCI/GAME  
**Status**: Open  
**Priority**: Medium  
**Blocker**: Mechanical logistics (Phase 5), Manufacturing machines (Phase 6)

**Question**: How detailed should mechanical power (shafts, gears, belts, hydraulics) be?

**Options**:
- A: Torque/RPM at each connection, efficiency per component, vibration/torsional resonance — realistic, enables gear design gameplay
- B: Power (W) only, with loss factor per connection — simple, but loses gear ratio gameplay
- C: 1D wave equation for shafts (torsional vibration) — accurate for resonance, expensive
- D: Abstract "mechanical network" like electrical: effort/flow variables, bond graph style

**Gameplay Impact**: 
- Gear ratios, torque multiplication, speed reduction — core to early industrial progression
- Belt slip, gear wear, lubrication — maintenance gameplay
- Vibration → fatigue failure — engineering challenge

**Research Needed**: Define minimum viable mechanical model for gearbox design gameplay. Prototype torque/RPM propagation.

---

### QUEST-008: Item Logistics — Discrete vs Continuous Approximation
**Category**: PERF/GAME  
**Status**: Open  
**Priority**: Medium  
**Blocker**: Item logistics (Phase 5)

**Question**: Belts, tubes, robots move discrete items. At high throughput (1000s items/sec), simulate each or approximate as continuous flow?

**Options**:
- A: Always discrete — every item tracked, collision, jamming, individual routing. Correct but expensive.
- B: Continuous approximation above threshold: `flow_rate = items/sec`, density on belt, backpressure as pressure. Switch to discrete near player.
- C: Hybrid: discrete items but batched — "crates" of 64 items move as units. Reduces entity count 64×.
- D: Fluid-like: items as compressible fluid in pipes, discrete on belts. Unified solver.

**Constraints**:
- Belt throughput: up to 45 items/sec (Minecraft) to 1000s (modded)
- Must handle: jams, splits, merges, priority, filtering
- Player visibility: items on belt visible near player

**Research Needed**: Benchmark discrete vs batched vs continuous at scale. Test visual transition.

---

### QUEST-009: Heat Transfer — Conduction/Convection/Radiation Coupling
**Category**: SCI/PERF  
**Status**: Open  
**Priority**: High  
**Blocker**: Thermal logistics (Phase 5), Process thermal (Phase 4)

**Question**: How to couple conduction (solids), convection (fluids), radiation (surfaces) in unified thermal network?

**Options**:
- A: Unified thermal network: nodes = thermal masses, edges = conductances (conduction), advection (fluid flow), radiation (view factors). Single matrix solve.
- B: Separate solvers: conduction (FEM/fast approx), convection (fluid-coupled), radiation (radiosity). Iterate coupling.
- C: Lumped capacitance per object: `dT/dt = (Q_in - Q_out) / (m * c_p)`. Edges = thermal resistances. Simple, scalable.
- D: Voxel-based conduction (3D diffusion) + fluid advection + surface radiation. Most accurate, expensive.

**Constraints**:
- 100k+ thermal masses (voxels, machines, fluid cells)
- Must handle: phase change (latent heat), thermal stress, radiation at high temp
- Coupled with fluid flow (convection)

**Research Needed**: Compare C (lumped) vs A (unified network) for accuracy vs speed. Test phase change stability.

---

### QUEST-010: Research System — Knowledge Representation
**Category**: GAME/ARCH  
**Status**: Open  
**Priority**: High  
**Blocker**: Research system (Phase 16)

**Question**: How to represent "knowledge" — discovered facts, uncertainties, contradictions — in a way that's queryable, shareable, and drives technology emergence?

**Options**:
- A: Probabilistic knowledge graph: nodes = facts (e.g., "Iron melts at 1811K ± 5K"), edges = dependencies, weights = confidence. Bayesian updates from experiments.
- B: Symbolic logic: facts as predicates, rules as Horn clauses. Deduction engine. Crisp but brittle.
- C: Data-driven: player builds "models" (curves, tables) from experimental data. Technology unlocks when model fits criteria.
- D: Hybrid: qualitative facts (graph) + quantitative models (data tables). Technology requires both.

**Requirements**:
- Multiplayer: knowledge sharing, peer review, replication
- Modding: new materials/processes add to graph
- UI: visualize uncertainty, contradictions, research frontier

**Research Needed**: Survey scientific knowledge representation (Semantic Web, Bayesian networks, qualitative reasoning). Prototype minimal graph.

---

### QUEST-011: Multiplayer — Client-Side Prediction for Machine Interaction
**Category**: ARCH/TECH  
**Status**: Open  
**Priority**: High  
**Blocker**: Multiplayer (Phase 14)

**Question**: Player breaks block → immediate local feedback. But machine operation (press button, open valve) — predict locally or wait for server?

**Options**:
- A: Predict all interactions — client simulates machine tick locally, reconciles. Complex: machine state depends on network flows, other machines.
- B: Predict only movement + block break/place. Machine UI = server-authoritative (latency visible). Simpler, but feels laggy at 200ms.
- C: Predict "optimistic" machine response (button pressed → assume success), rollback on conflict. Middle ground.
- D: Server runs machine tick at 20Hz, client interpolates. No prediction for machines.

**Constraints**:
- 50+ players, shared factories
- Machine state depends on global logistics networks
- Cheat resistance: no client-authoritative production

**Research Needed**: Prototype with simple machine (furnace). Measure perceived latency at 50, 100, 200ms RTT.

---

### QUEST-012: Save Format — Chunk Compression Algorithm
**Category**: TECH/PERF  
**Status**: Open  
**Priority**: Medium  
**Blocker**: Save/load (Phase 2.6)

**Question**: Best compression for voxel chunks (32³ = 32768 blocks, palette + RLE)?

**Options**:
- A: Palette + RLE (current design) — good for homogeneous, bad for noise
- B: Palette + Zstd — better ratio, slower
- C: Bit-packed (4-8 bits/block) + LZ4 — fast, decent ratio
- D: Custom: material ID (12 bits) + state (4 bits) = 16 bits/block = 64KB raw. Compress with LZ4.

**Constraints**:
- Save size target: <500MB for heavily modified world
- Load time: <3s to playable
- Must support streaming load (priority by distance)

**Research Needed**: Benchmark on realistic chunk data (terrain, caves, player builds). Test compression ratio, compress/decompress speed.

---

### QUEST-013: Deterministic Float Math for Rendering/Physics Boundary
**Category**: TECH/ARCH  
**Status**: Open  
**Priority**: Medium  
**Blocker**: Simulation/rendering boundary (Phase 2)

**Question**: Simulation uses fixed-point. Rendering uses f32. How to convert without non-determinism or jitter?

**Options**:
- A: Fixed → f32 at chunk mesh build time. Deterministic if mesh build is deterministic.
- B: Fixed → f64 → f32. More precision, same determinism.
- C: Rendering reads fixed-point directly in shader (via 64-bit int or emulated). No conversion, but shader complexity.
- D: Simulation also outputs f32 "render positions" computed deterministically (fixed-point arithmetic rounding to f32).

**Problem**: Player at 1,000,000 km from origin — f32 precision ~0.5m. Fixed-point maintains 0.5mm. Rendering jitter if not careful.

**Research Needed**: Test rendering at planetary distances. Implement camera-relative rendering (GPU: vertex_pos - camera_pos in fixed-point → f32).

---

### QUEST-014: Modding API — Capability Granularity
**Category**: ARCH/POL  
**Status**: Open  
**Priority**: Medium  
**Blocker**: Modding (Phase 10, 21)

**Question**: How fine-grained should mod capabilities be? Per-system? Per-function? Per-data-type?

**Options**:
- A: Coarse: "can_read_simulation", "can_write_simulation", "can_register_process", "can_register_machine" — simple, but all-or-nothing
- B: Fine: capability per trait/function (e.g., `VoxelStorage::set_block`, `ProcessSystem::register_process`) — flexible, complex manifest
- C: Data-driven: mods declare what data they read/write (schemas). Runtime validates access. Like WASM component model.
- D: Trust-based: mods are code-reviewed, signed. Sandbox only for resource limits.

**Constraints**:
- Security: no sandbox escape, no simulation corruption
- Usability: modders shouldn't need capability manifest PhD
- Determinism: mod code must be deterministic

**Research Needed**: Survey WASM capability models (WASI, wasmCloud, Extism). Design minimal viable API.

---

### QUEST-015: Time Acceleration — Maximum Rate for Aggregate Simulation
**Category**: SCI/PERF  
**Status**: Open  
**Priority**: Medium  
**Blocker**: Macro simulation (Phase 13)

**Question**: What's the maximum time acceleration before aggregate simulation breaks down?

**Factors**:
- Process rates scale linearly with dt? (Arrhenius: rate ∝ exp(-Ea/RT) — not linear)
- Numerical stability: explicit Euler stable if dt < 2/λ_max. Implicit: unconditionally stable but accuracy degrades.
- Event scheduling: discrete events (failures, maintenance) at accelerated rate — Poisson process scaling?
- Wear: linear with time? Or accelerated wear models?

**Options**:
- A: Cap at 100× with sub-stepping for stiff processes
- B: Adaptive: simulate at max stable dt per process, accumulate wall time
- C: Statistical: at >1000×, switch to analytical aggregate models (queuing theory, mean-field)
- D: No cap — but document accuracy degradation

**Research Needed**: Analyze stiffness of process ODEs. Test aggregate factory at 1×, 10×, 100×, 1000×. Measure throughput error.

---

### QUEST-016: Planet Generation — Tectonic Simulation Fidelity
**Category**: SCI/PERF  
**Status**: Open  
**Priority**: Medium  
**Blocker**: World generation (Phase 3)

**Question**: How much tectonic simulation for plausible geology?

**Options**:
- A: Full plate tectonics: ~10 plates, ridge push, slab pull, transform boundaries. Simulate 1M years in steps. Expensive but plausible.
- B: Simplified: generate plate boundaries via Voronoi, assign motion vectors, compute uplift/subduction analytically. Fast, good enough.
- C: No tectonics: noise-based mountains, manual ore placement. Fast, but geologically nonsense.
- D: Pre-computed: ship with hand-crafted "geological history" textures. Not procedural.

**Constraints**:
- Generation time: <5s for region (512³ voxels)
- Must produce: mountain ranges, rift valleys, volcanic arcs, mineral deposits at plate boundaries
- Deterministic, seeded

**Research Needed**: Implement B (simplified analytic) first. Compare geological plausibility to real Earth data. Benchmark generation speed.

---

### QUEST-017: Radiation Simulation — Ionizing vs Non-Ionizing
**Category**: SCI/GAME  
**Status**: Open  
**Priority**: Low  
**Blocker**: Nuclear/survival (Phase 7, 17)

**Question**: Model ionizing radiation (alpha, beta, gamma, neutron) transport and biological effect, or abstract as "radiation level"?

**Options**:
- A: Particle transport: Monte Carlo or deterministic (SN). Isotopes, half-lives, decay chains, shielding, dose calculation (Sv). Realistic, heavy.
- B: Abstract: "radiation intensity" field, material attenuation coefficients, dose rate = intensity * time. Simpler.
- C: Hybrid: detailed for reactor core (neutron flux, criticality), abstract for environment.

**Gameplay**: 
- Reactor design: moderator, reflector, control rods, shielding
- Radiation sickness: deterministic vs stochastic effects
- Contamination: radioactive isotopes in materials, decay heat

**Research Needed**: Define minimum viable for fission reactor gameplay. Survey game implementations (Stationeers, ReactorCraft).

---

### QUEST-018: Biology — Individual vs Population Simulation
**Category**: SCI/PERF  
**Status**: Open  
**Priority**: Low  
**Blocker**: Biology/ecology (Phase 8)

**Question**: Simulate every animal/plant individually, or population-level (densities, age-structured)?

**Options**:
- A: Individual agents (boids-style) for animals, individual plants. Emergent behavior, but 100k+ entities expensive.
- B: Population density fields (reaction-diffusion PDEs). Scalable, but loses individual behavior.
- C: Hybrid: individuals near player, populations far. LOD for biology.
- D: Abstract: biomes have "carrying capacity", "biodiversity index". No simulation.

**Constraints**:
- Player can hunt, farm, domesticate — needs individuals
- Ecology: predator-prey, succession, disease spread
- Planetary scale: millions of km²

**Research Needed**: Prototype individual-based near player, population-field far. Test transition.

---

### QUEST-019: Chemistry — Reaction Network Solver
**Category**: SCI/PERF  
**Status**: Open  
**Priority**: Medium  
**Blocker**: Chemistry (Phase 9)

**Question**: Solve full reaction network (100s species, 1000s reactions) or use equilibrium/steady-state approximation?

**Options**:
- A: Full kinetic ODE system (CVODE/ARKODE). Accurate, stiff, expensive.
- B: Equilibrium solver (Gibbs free energy minimization) for fast reactions, kinetics for slow. Standard in process simulators.
- C: Reduced mechanism: lump species, quasi-steady-state approximation (QSSA). Manual or automatic reduction.
- D: Data-driven: ML surrogate for reactor output given inputs. Fast, but training, extrapolation risk.

**Constraints**:
- Industrial chemistry: Haber-Bosch, electrolysis, cracking, synthesis gas
- Player builds reactors, tunes conditions
- Must be deterministic

**Research Needed**: Survey chemical kinetics solvers (Cantera, OpenSMOKE, gPROMS). Prototype minimal network.

---

### QUEST-020: Vehicle Physics — Fidelity vs Fun
**Category**: SCI/GAME  
**Status**: Open  
**Priority**: Low  
**Blocker**: Vehicles (Phase 12)

**Question**: Rigid body + tire model (Pacejka) + suspension for ground vehicles? Or simplified arcade?

**Options**:
- A: Full rigid body + Pacejka tire model + double wishbone suspension + drivetrain. Realistic, enables engineering.
- B: Simplified: raycast suspension, friction curve, torque curve. Good feel, less params.
- C: Arcade: forces applied directly, no suspension simulation. Fun, but no engineering depth.
- D: Modular: core rigid body, pluggable tire/suspension models. Configurable fidelity.

**Constraints**:
- Ground, rail, marine, air — unified framework?
- Multiplayer: deterministic physics
- Player builds vehicles from parts

**Research Needed**: Survey game vehicle physics (BeamNG, Assetto Corsa, Unity WheelCollider, Godot VehicleBody). Define minimal for "build your own truck" gameplay.

---

## Question Index

| ID | Title | Category | Priority | Status |
|----|-------|----------|----------|--------|
| 001 | Fixed-Point vs f64 Boundary | ARCH/TECH | High | Open |
| 002 | Planetary Voxel Coordinates | ARCH/SCI | High | Open |
| 003 | Process Rate Law Formalism | SCI/ARCH | High | Open |
| 004 | Fluid Simulation Fidelity | SCI/PERF | High | Open |
| 005 | Aggregation Reconstruction | ARCH/SCI | Critical | Open |
| 006 | Electrical Network Solver | SCI/TECH | Medium | Open |
| 007 | Mechanical Power Detail | SCI/GAME | Medium | Open |
| 008 | Item Logistics Discrete/Continuous | PERF/GAME | Medium | Open |
| 009 | Heat Transfer Coupling | SCI/PERF | High | Open |
| 010 | Knowledge Representation | GAME/ARCH | High | Open |
| 011 | MP Machine Prediction | ARCH/TECH | High | Open |
| 012 | Chunk Compression Algorithm | TECH/PERF | Medium | Open |
| 013 | Fixed→Float Rendering Boundary | TECH/ARCH | Medium | Open |
| 014 | Modding Capability Granularity | ARCH/POL | Medium | Open |
| 015 | Max Time Acceleration Rate | SCI/PERF | Medium | Open |
| 016 | Tectonic Generation Fidelity | SCI/PERF | Medium | Open |
| 017 | Radiation Simulation Detail | SCI/GAME | Low | Open |
| 018 | Biology Individual/Population | SCI/PERF | Low | Open |
| 019 | Chemistry Reaction Solver | SCI/PERF | Medium | Open |
| 020 | Vehicle Physics Fidelity | SCI/GAME | Low | Open |

---

## Resolution Process

For each question:
1. **Research** — Literature survey, prior art, prototypes
2. **Prototype** — Minimal implementation of top 2-3 options
3. **Benchmark** — Performance, accuracy, determinism
4. **ADR** — Document decision with evidence
5. **Implement** — Full integration

**Rule**: No implementation of blocked system without at least "Researching" status and prototype plan.