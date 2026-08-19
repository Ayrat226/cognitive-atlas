# LITHOS — Technical Debt Register

## Debt Classification

| Priority | Criteria |
|----------|----------|
| **CRITICAL** | Blocks correctness, security, or architectural integrity. Must fix before next phase. |
| **HIGH** | Significant performance, maintainability, or scalability impact. Fix in current phase. |
| **MEDIUM** | Known limitation with workaround. Fix in next 2 phases. |
| **LOW** | Cosmetic, minor inconvenience, or future improvement. Fix when convenient. |

---

## Current Technical Debt

### DEBT-001: No Actual Code Yet (Foundation)
**Priority**: CRITICAL  
**Status**: Open  
**Phase**: 1  
**Description**: Repository contains only documentation. No Rust project, no Vulkan init, no engine foundation.  
**Impact**: Cannot proceed to Phase 2 without Phase 1 complete.  
**Resolution**: Implement Phase 1 per ROADMAP.md — Cargo workspace, Vulkan window, math, memory, jobs, logging, profiler, serialization.  
**Trigger**: Start of Phase 1 implementation.

---

### DEBT-002: Fixed-Point Math Library Not Implemented
**Priority**: CRITICAL  
**Status**: Open  
**Phase**: 1  
**Description**: ADR-003 mandates fixed-point (Q52.11) for simulation determinism. No implementation exists.  
**Impact**: All simulation code depends on this. Blocking.  
**Resolution**: Implement `lithos-math` crate with `FixedVec3`, `FixedAABB`, `FixedMat4`, noise functions, geometry tests. Property-test against f64 reference.  
**Trigger**: Phase 1.3 (Math Library).

---

### DEBT-003: Vulkan Boilerplate Not Started
**Priority**: CRITICAL  
**Status**: Open  
**Phase**: 1  
**Description**: Vulkan initialization, swapchain, synchronization, validation layers — all missing.  
**Impact**: Rendering cannot begin. Blocking Phase 1.2.  
**Resolution**: Implement minimal Vulkan core: instance → device → queues → swapchain → frame sync → validation. Use `ash` crate.  
**Trigger**: Phase 1.2 (Window & Vulkan Init).

---

### DEBT-004: Job System Not Implemented
**Priority**: HIGH  
**Status**: Open  
**Phase**: 1  
**Description**: Work-stealing thread pool with task graph, priorities, profiling integration — missing.  
**Impact**: Parallel simulation, meshing, worldgen all need this.  
**Resolution**: Implement `lithos-jobs` crate. Consider `rayon` as base or custom for deterministic ordering. Integrate Tracy zones.  
**Trigger**: Phase 1.6 (Job System).

---

### DEBT-005: Serialization Framework Not Implemented
**Priority**: HIGH  
**Status**: Open  
**Phase**: 1  
**Description**: Schema-driven binary serialization with migration (ADR-007) not built.  
**Impact**: Save/load, networking, modding all depend on this.  
**Resolution**: Implement `lithos-serialization` with proc-macro schema derivation, registry, migration framework. Benchmark vs bincode/serde.  
**Trigger**: Phase 1.7 (Serialization).

---

### DEBT-006: Voxel Storage Design Not Validated
**Priority**: HIGH  
**Status**: Open  
**Phase**: 2  
**Description**: Sparse hash map + procedural base + delta log design (ADR-005) untested at scale.  
**Impact**: Core data structure. If performance inadequate, major redesign needed.  
**Resolution**: Prototype in Phase 2.1 with benchmarks: 10k chunks, 1M blocks, generation + access + save/load. Target: <50ns get, <100ns set, <5ms chunk gen.  
**Trigger**: Phase 2.1 (Voxel Storage).

---

### DEBT-007: Meshing Algorithm Unproven at Scale
**Priority**: HIGH  
**Status**: Open  
**Phase**: 2  
**Description**: Greedy meshing + cross-chunk stitching + GPU-driven indirect draws designed but not implemented.  
**Impact**: Rendering performance, visual quality.  
**Resolution**: Implement in Phase 2.2. Test: 50k chunks, LOD transitions, border stitching correctness, VRAM usage.  
**Trigger**: Phase 2.2 (Meshing).

---

### DEBT-008: Deterministic Simulation Across Platforms Unverified
**Priority**: HIGH  
**Status**: Open  
**Phase**: 2-3  
**Description**: Fixed-point math (ADR-003) should guarantee determinism, but compiler optimizations, SIMD, and hardware differences may introduce divergence.  
**Impact**: Networking desync, replay divergence, save/load corruption.  
**Resolution**: 
- CI test: run simulation on x86_64 and ARM64 (qemu or GitHub Actions ARM runners)
- Replay test: record inputs → replay on different arch → compare state hash
- Document any required compiler flags (`-C target-cpu=native` avoided in sim)
**Trigger**: Phase 2.4 (Collision & Physics) — first deterministic simulation test.

---

### DEBT-009: Process System Solver Stability Unknown
**Priority**: HIGH  
**Status**: Open  
**Phase**: 4  
**Description**: Implicit integration for stiff reaction networks, phase changes, thermal conduction — numerical stability unproven.  
**Impact**: Explosions, NaN, energy non-conservation, simulation blow-up.  
**Resolution**: 
- Implement with conservative time stepping, clamping, fallback to smaller dt
- Property tests: conservation invariants, bounded state, no NaN
- Fuzz test: random process networks, long runs
**Trigger**: Phase 4.2 (Process Framework).

---

### DEBT-010: Logistics Network Solver Scalability
**Priority**: MEDIUM  
**Status**: Open  
**Phase**: 5  
**Description**: Fluid (1D Navier-Stokes), power (nodal analysis), item (event-driven) solvers designed but scaling untested.  
**Impact**: Large factories may lag.  
**Resolution**: Benchmark at 10×, 100×, 1000× representative loads. Profile solve time vs network size. Optimize sparse matrix solves, event queue.  
**Trigger**: Phase 5.2 (Carrier Implementations).

---

### DEBT-011: Aggregation/Disaggregation State Fidelity
**Priority**: HIGH  
**Status**: Open  
**Phase**: 3, 13  
**Description**: LOD transitions (ADR-009) must preserve state. Round-trip error accumulation unmeasured.  
**Impact**: Long-run drift, exploits (duplication), visual popping.  
**Resolution**: 
- Define error tolerances per quantity (mass: 0.01%, energy: 0.1%, position: 1mm)
- Automated test: detail → aggregate → detail → aggregate... 1000 cycles → measure drift
- Hysteresis tuning (aggregate at 200m, disaggregate at 150m)
**Trigger**: Phase 3.4 (LOD Transitions) and Phase 13.1 (Aggregation).

---

### DEBT-012: Network Protocol Not Designed
**Priority**: HIGH  
**Status**: Open  
**Phase**: 14  
**Description**: Binary protocol, delta compression, interest management, reconnection — all TBD.  
**Impact**: Multiplayer cannot work without this.  
**Resolution**: Design in Phase 14.1-14.3. Prototype with 50 bots. Measure bandwidth, latency tolerance, desync rate.  
**Trigger**: Phase 14 (Multiplayer).

---

### DEBT-013: Save Migration Framework Untested
**Priority**: MEDIUM  
**Status**: Open  
**Phase**: 2, ongoing  
**Description**: Schema versioning and migration (ADR-007) designed but no actual migrations exist yet.  
**Impact**: Future save breaking changes will be painful.  
**Resolution**: 
- Add migration test to CI: generate v1 save → migrate to v2 → load → verify
- Test every schema change with migration
- Maintain old save fixtures in repo
**Trigger**: First schema change after initial save format (Phase 2.6).

---

### DEBT-014: WASM Modding Sandbox Not Prototyped
**Priority**: MEDIUM  
**Status**: Open  
**Phase**: 10, 21  
**Description**: ADR-010 chooses WASM but runtime (wasmtime vs wasmer), API design, quota enforcement unprototyped.  
**Impact**: Modding is core to longevity. Late discovery of limitations costly.  
**Resolution**: Prototype in Phase 10 (Computing) — simple WASM module calling simulation API. Measure overhead, test sandbox escape attempts.  
**Trigger**: Phase 10.2 (Software Stack).

---

### DEBT-015: Time Acceleration Determinism
**Priority**: MEDIUM  
**Status**: Open  
**Phase**: 13  
**Description**: Selective acceleration (ADR-011) must produce identical results regardless of acceleration path.  
**Impact**: Save/load at different acceleration → different world. Multiplayer desync.  
**Resolution**: 
- Deterministic event scheduling across rates
- Replay test: run at 1× → save → load at 100× → run → compare to 1× run
- Document any non-deterministic approximations
**Trigger**: Phase 13.2 (Time Acceleration).

---

### DEBT-016: No CI Pipeline
**Priority**: HIGH  
**Status**: Open  
**Phase**: 1  
**Description**: No GitHub Actions for fmt, clippy, test, build, benchmarks.  
**Impact**: Quality gate missing. Regressions undetected.  
**Resolution**: Set up CI in Phase 1.1. Jobs: fmt, clippy, test (unit), build (release), bench (baseline).  
**Trigger**: Phase 1.1 (Project Setup).

---

### DEBT-017: No Profiling Baseline
**Priority**: MEDIUM  
**Status**: Open  
**Phase**: 1  
**Description**: Performance contracts (PERFORMANCE_CONTRACT.md) require baselines. None exist.  
**Impact**: Cannot measure regressions.  
**Resolution**: Establish baselines in Phase 1.5 (Profiler) for: frame time, sim tick time, memory, VRAM. Store in `benchmarks/baselines/`.  
**Trigger**: Phase 1.5 (Logging & Profiling).

---

### DEBT-018: Asset Pipeline Undefined
**Priority**: LOW  
**Status**: Open  
**Phase**: 2, ongoing  
**Description**: Texture format (16×16 discipline), model format, shader compilation, audio — not specified.  
**Impact**: Inconsistent assets, build complexity.  
**Resolution**: Define in Phase 2: 
- Textures: PNG → ASTC/BC7 at build, 16×16 base, mipmaps, array textures
- Models: glTF 2.0 → custom binary, meshlets for GPU culling
- Shaders: GLSL → SPIR-V at build, hot-reload in dev
- Audio: OGG Vorbis → streaming
**Trigger**: Phase 2.5 (Break/Place — first assets needed).

---

### DEBT-019: Input/Accessibility Framework Missing
**Priority**: LOW  
**Status**: Open  
**Phase**: 21  
**Description**: Rebindable keys, colorblind modes, text scaling, screen reader support — not designed.  
**Impact**: Accessibility compliance, player experience.  
**Resolution**: Design in Phase 21.4 (Polish). Use `accesskit` for screen readers.  
**Trigger**: Phase 21 (Optimization & Release).

---

### DEBT-020: Localization Framework Missing
**Priority**: LOW  
**Status**: Open  
**Phase**: 21  
**Description**: No i18n infrastructure. All text hardcoded in English.  
**Impact**: Late localization costly.  
**Resolution**: Add `fluent` or `gettext` integration in Phase 2. Extract all UI strings to `.ftl` files.  
**Trigger**: Phase 2.3 (Player & Camera — first UI).

---

## Debt Summary

| Priority | Count |
|----------|-------|
| CRITICAL | 3 |
| HIGH | 8 |
| MEDIUM | 6 |
| LOW | 3 |
| **Total** | **20** |

---

## Debt Paydown Strategy

1. **Phase 1**: Resolve all CRITICAL + HIGH foundation debt (DEBT-001 through DEBT-005, DEBT-016, DEBT-017)
2. **Phase 2**: Validate core data structures (DEBT-006, DEBT-007) and establish determinism (DEBT-008)
3. **Phase 3-4**: Address simulation correctness debt (DEBT-009, DEBT-011)
4. **Phase 5-13**: Tackle scalability debt as systems come online (DEBT-010, DEBT-011, DEBT-015)
5. **Phase 14**: Network debt (DEBT-012)
6. **Phase 10, 21**: Modding and polish debt (DEBT-014, DEBT-018, DEBT-019, DEBT-020)
7. **Ongoing**: Migration debt (DEBT-013) with every schema change

**Rule**: No phase transition with unresolved CRITICAL debt. HIGH debt must have mitigation plan.