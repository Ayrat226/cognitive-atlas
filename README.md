# LITHOS

A voxel planetary simulation game where technology, ecology, manufacturing, and civilization emerge from a coherent causal simulation — not fixed recipes.

## Core Vision

Minecraft-like readability and usability at the presentation layer.  
A data-oriented, multi-layered causal simulation at the core.  
New materials, processes, machines, technologies, computers, and robots require no pre-existing recipes — they emerge from constraints and capabilities.

Scalable from a small voxel sandbox to a full planet, multiplayer, industrial civilization, and space.

## Design Pillars

1. **Causality** — Every effect has a traceable cause
2. **Freedom** — Players discover solutions, not follow recipes
3. **Engineering** — Real constraints create meaningful decisions
4. **Discovery** — Unknown systems reward experimentation
5. **Automation** — Manual → Tool → Machine → AI → Civilization
6. **Exploration** — Planetary scale with procedural depth
7. **Multiplayer Emergence** — 50+ players, shared persistent world
8. **Long-term Persistence** — Modifications survive across sessions

## Non-Goals

- Simulating every atom globally
- Forcing every action into tedious manual procedures
- Making all realism equally detailed
- Hardcoding every possible machine
- Requiring a single optimal technology path

## Quick Start

```bash
# Build and run (once implemented)
cargo run --release
```

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for system design.  
See [docs/ROADMAP.md](docs/ROADMAP.md) for phased implementation plan.  
See [docs/PROJECT_BIBLE.md](docs/PROJECT_BIBLE.md) for complete project vision.

## Development Principles

- Correctness > Architecture integrity > Consistency > Scalability > Performance > Maintainability > Gameplay quality > Visual polish
- No significant change without impact audit
- Every system: model, state, interfaces, lifecycle, persistence, networking, performance strategy, tests, documentation
- Research → Prototype → Benchmark → Adversarial Review → Implement → Test → Document

## License

Proprietary — All rights reserved.