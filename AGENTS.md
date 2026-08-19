# LITHOS — Agent Instructions

## Quick Start

```bash
# Build everything
just build-release

# Run checks (fmt + clippy + test)
just ci

# Run client
just run

# Run server
just run-server

# Check single crate
cargo check -p lithos-engine-math
```

## Project Structure

```
lithos/                          # Cargo workspace root
├── crates/
│   ├── engine/*                 # Core: memory, jobs, math, logging, profiler, serialization, config, window
│   ├── simulation/*             # Voxel, materials, processes, logistics, aggregation, worldgen
│   ├── game/*                   # Player, inventory, crafting, research, ui
│   ├── rendering/*              # Vulkan, mesh, materials, lighting, post
│   ├── networking/*             # Authority, replication, interest, prediction
│   ├── persistence/*            # World/entity/player save, migration
│   ├── app/                     # Client binary (lithos)
│   └── server/                  # Dedicated server binary (lithos-server)
├── docs/                        # Architecture, roadmap, contracts, decisions
├── config.toml                  # Client config
├── server.toml                  # Server config
└── justfile                     # Task runner
```

## Key Commands

| Task | Command |
|------|---------|
| Full build (release) | `just build-release` |
| Check workspace | `cargo check --workspace` |
| Test all | `cargo test --workspace` |
| Format check | `cargo fmt --all -- --check` |
| Clippy (strict) | `cargo clippy --workspace --all-targets -- -D warnings` |
| Benchmarks | `cargo bench --workspace` |
| CI pipeline | `just ci` |

## Critical Gotchas

### Ash 0.37 API
- **DescriptorIndexing** extension removed → use khr or remove from required extensions
- **p_next chain** requires `*mut c_void` casts, not `*const _`
- **wait_for_fences** second arg is `bool` (not u32)
- **create_device** takes 3 args: `(physical_device, &create_info, None)`
- Device layer fields (`enabled_layer_count`, `pp_enabled_layer_names`) are deprecated

### Static Mut Refs (Rust 2024 UB)
Several engine crates use `static mut` + `as_mut()` — **undefined behavior**. Fix pattern:
```rust
use std::sync::OnceLock;
static GLOBAL: OnceLock<Type> = OnceLock::new();
fn init() { GLOBAL.set(Type::new()).ok(); }
fn get() -> &'static Type { GLOBAL.get().unwrap() }
```
Affected: `engine/jobs`, `engine/logging`, `engine/profiler`

### Fixed-Point Math
- Simulation uses **Q52.11** (i64, 1/2048m ≈ 0.5mm precision)
- All simulation positions/velocities/time use `Fixed` type
- Rendering uses f32 — convert at boundary: `fixed.to_f32()`

### Determinism Requirements
- Simulation must be bitwise identical across x86_64/ARM64
- CI runs determinism test: executes same simulation twice, compares state hashes
- No f64 in simulation hot path; use `Fixed` everywhere

## Architecture Notes

### Module Dependencies (simplified)
```
engine/memory → engine/jobs → simulation/*
engine/math   → engine/* + simulation/* + rendering/*
rendering/vulkan → rendering/mesh, materials, lighting
simulation/voxel → simulation/materials, processes, logistics
game/* → simulation/* + engine/*
```

### Data Flow (per tick)
```
Input → Simulation (fixed timestep) → Events → Replication → Persistence → Render prep → GPU submit
```

### Entry Points
- **Client**: `crates/app/src/main.rs` → `run_app()` → winit event loop
- **Server**: `crates/server/src/main.rs` → tokio main → simulation loop

## Configuration

| File | Purpose |
|------|---------|
| `config.toml` | Client: window, rendering, simulation, network, persistence |
| `server.toml` | Server: headless, more threads, higher chunk limits |
| `rust-toolchain.toml` | Stable + rustfmt, clippy, rust-src |
| `.cargo/config.toml` | LLD linker, native CPU, optimized profiles |

## Testing

```bash
# All tests
cargo test --workspace

# Unit only
cargo test --workspace --lib

# Determinism test (runs simulation twice, compares hashes)
cargo test --package lithos-engine-math determinism
```

CI runs on x86_64 and aarch64 (via qemu) to verify cross-platform determinism.

## Useful justfile Targets

```bash
just build-release    # Release build
just build-dev        # Debug build
just run              # Run client
just run-server       # Run server
just check            # cargo check --workspace --all-targets
just clippy           # Strict clippy
just fmt              # Format all
just bench            # Benchmarks
just clean            # cargo clean
just watch            # cargo watch on check/test
```

## Docs to Read First

- `docs/PROJECT_BIBLE.md` — Vision, pillars, player journey
- `docs/ARCHITECTURE.md` — System design, module contracts
- `docs/ROADMAP.md` — 21 phases, current: Phase 1 (Engine Foundation)
- `docs/DECISIONS.md` — 15 ADRs (why Rust, Vulkan, fixed-point, etc.)

## Current Phase

**Phase 1: Engine Foundation** — Vulkan renderer, math validation, job system benchmarks.
Next: Phase 2 — Voxel storage, meshing, player controller, break/place, save/load.

## Environment

- Rust stable (via rust-toolchain.toml)
- Vulkan SDK required (`libvulkan-dev` on Linux)
- Build deps: `clang`, `lld`, `pkg-config`
- CI: GitHub Actions (ubuntu-latest)