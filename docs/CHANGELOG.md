# LITHOS — Changelog

## Format
Based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Categories: Added, Changed, Deprecated, Removed, Fixed, Security.

---

## [Unreleased] — Phase 0: Architecture Governance

### Added
- **PROJECT_BIBLE.md** — Complete project vision, design pillars, player journey, world, technology freedom, multiplayer target, long-term vision, scientific integrity, content governance, schema evolution, security contract, exploit audit checklist
- **ARCHITECTURE.md** — High-level structure, core principles, all module designs (Engine, Simulation, Game, Rendering, Networking, Persistence), data flow, configuration, testing strategy, external dependencies
- **ROADMAP.md** — 21 phases from Architecture Governance to Release, with detailed tasks, milestones, risk register
- **DEPENDENCY_GRAPH.md** — Module dependencies, data type dependencies, producer/consumer map, circular dependencies (documented), external dependencies, Cargo workspace structure, schema dependencies, network protocol dependencies, content definition dependencies
- **SYSTEM_CONTRACT.md** — Formal contracts for 10 core systems: Voxel Storage, Meshing, Materials, Processes, Logistics Networks, Aggregation, Rendering, Networking, Persistence, Game Logic (Player, Crafting, Research)
- **DECISIONS.md** — 15 Architecture Decision Records (ADR-001 through ADR-015) covering: Rust, Vulkan, Fixed-Point Math, Data-Oriented Design, Sparse Voxel Storage, Server-Authoritative Networking, Schema-Driven Serialization, Process-Based Simulation, Aggregation LOD, WASM Modding, Selective Time Acceleration, No GC in Simulation, Process-Based World Gen, Energy/Heat First-Class, Research System
- **TECH_DEBT.md** — 20 technical debt items classified CRITICAL/HIGH/MEDIUM/LOW with resolution plans and triggers
- **OPEN_QUESTIONS.md** — 20 open questions across ARCH, SCI, GAME, PERF, TECH, POL categories with research/prototype plans
- **CURRENT_STATE.md** — Phase 0 status, working/broken features, risks, performance baselines, test status, next actions, document checklist

### Changed
- **README.md** — Renamed project from "cognitive-atlas" to "LITHOS", updated description to match PROJECT_BIBLE vision

---

## Version History Template

### [X.Y.Z] — YYYY-MM-DD

#### Added
- Feature description

#### Changed
- Change description

#### Deprecated
- Deprecation notice

#### Removed
- Removal notice

#### Fixed
- Bug fix description

#### Security
- Security fix description

---

## Release Checklist (per version)

- [ ] All tests pass (unit, integration, simulation, stress, determinism)
- [ ] Benchmarks within regression thresholds (<5% CPU, <5% memory, <5% GPU, <10% network)
- [ ] No CRITICAL/HIGH regressions
- [ ] Documentation updated (architecture, contracts, policies)
- [ ] CHANGELOG.md updated
- [ ] CURRENT_STATE.md updated
- [ ] ROADMAP.md reflects actual implementation
- [ ] DEPENDENCY_GRAPH.md reflects actual dependencies
- [ ] TECH_DEBT.md updated
- [ ] OPEN_QUESTIONS.md updated
- [ ] Schema versions incremented if changed
- [ ] Migration tested (old save → new save → load)
- [ ] Exploit audit passed for new systems
- [ ] ADR written for architectural decisions
- [ ] CI pipeline green
- [ ] Save compatibility verified

---

## Schema Version Tracking

| Schema | Version | Last Changed | Migration Tested |
|--------|---------|--------------|------------------|
| world_version | 1 | — | — |
| save_schema_version | 1 | — | — |
| simulation_model_version | 1 | — | — |
| network_protocol_version | 1 | — | — |
| content_schema_version | 1 | — | — |
| mod_api_version | 1 | — | — |

---

## Breaking Change Log

*None yet — pre-alpha*

---

## Performance Regression Log

*None yet — baselines not established*

---

## Security Advisory Log

*None yet*