# LITHOS — Current State

## Current Phase
**Phase 0 — Architecture Governance** (In Progress)

## Milestone
**M0: Governance Documents Complete**
- Target: End of Week 1
- Status: 🟡 70% complete

---

## Working Features
*None yet — no code implemented*

---

## Broken Features
*None yet — no code implemented*

---

## Known Limitations
*None yet — no code implemented*

---

## Active Risks

| Risk | Probability | Impact | Status |
|------|-------------|--------|--------|
| Vulkan boilerplate complexity delays Phase 1 | Medium | High | Monitoring |
| Fixed-point math library scope creep | Medium | High | Scoped to Phase 1.3 |
| Determinism verification infrastructure missing | Medium | High | Planned for Phase 2.4 |
| No CI pipeline | High | Medium | Blocked on Phase 1.1 |

---

## Performance Baselines
*Not established — requires Phase 1.5 (Profiler) implementation*

| Metric | Target | Current | Notes |
|--------|--------|---------|-------|
| Frame time (simulation) | <16.6ms | — | 60 FPS target |
| Simulation tick (20 Hz) | <50ms | — | Fixed timestep |
| Memory (idle) | <2GB | — | Baseline |
| VRAM (idle) | <1GB | — | Baseline |
| World load time | <10s | — | Streaming |
| Save time (incremental) | <1s | — | Dirty chunks only |

---

## Test Status
*No tests exist yet*

| Level | Count | Passing | Coverage |
|-------|-------|---------|----------|
| Unit | 0 | 0 | 0% |
| Integration | 0 | 0 | 0% |
| Simulation | 0 | 0 | 0% |
| Property-based | 0 | 0 | 0% |
| Fuzz | 0 | 0 | 0% |
| Stress | 0 | 0 | 0% |
| Determinism | 0 | 0 | 0% |
| Persistence | 0 | 0 | 0% |
| Network | 0 | 0 | 0% |
| Visual Regression | 0 | 0 | 0% |

---

## Recently Changed
- **2026-08-19**: Created PROJECT_BIBLE.md
- **2026-08-19**: Created ARCHITECTURE.md
- **2026-08-19**: Created ROADMAP.md
- **2026-08-19**: Created DEPENDENCY_GRAPH.md
- **2026-08-19**: Created SYSTEM_CONTRACT.md
- **2026-08-19**: Created DECISIONS.md (15 ADRs)
- **2026-08-19**: Created TECH_DEBT.md (20 items)
- **2026-08-19**: Created OPEN_QUESTIONS.md (20 questions)

---

## Next Actions (Priority Order)

1. **Create Cargo workspace** — `Cargo.toml` with all crates defined
2. **Set up rust-toolchain.toml** — stable, rustfmt, clippy
3. **Configure .cargo/config.toml** — profiles, target, features
4. **Create justfile/Makefile** — common tasks (build, test, run, bench, fmt, clippy)
5. **Set up GitHub Actions CI** — fmt, clippy, test, build, bench
6. **Implement lithos-engine crates** — memory, jobs, math, logging, profiler, serialization, config
7. **Implement Vulkan core** — instance, device, swapchain, frame sync, validation
8. **Implement fixed-point math** — FixedVec3, FixedAABB, noise, geometry
9. **Implement job system** — work-stealing pool, task graph, priorities, Tracy
10. **Implement serialization** — schema derivation, registry, migration framework

---

## Document Status

| Document | Status | Last Updated |
|----------|--------|--------------|
| PROJECT_BIBLE.md | ✅ Complete | 2026-08-19 |
| ARCHITECTURE.md | ✅ Complete | 2026-08-19 |
| ROADMAP.md | ✅ Complete | 2026-08-19 |
| DEPENDENCY_GRAPH.md | ✅ Complete | 2026-08-19 |
| SYSTEM_CONTRACT.md | ✅ Complete | 2026-08-19 |
| DECISIONS.md | ✅ Complete | 2026-08-19 |
| TECH_DEBT.md | ✅ Complete | 2026-08-19 |
| OPEN_QUESTIONS.md | ✅ Complete | 2026-08-19 |
| CURRENT_STATE.md | ✅ Complete | 2026-08-19 |
| CHANGELOG.md | 🟡 Pending | — |
| DEFINITION_OF_DONE.md | 🟡 Pending | — |
| SUBAGENT_POLICY.md | 🟡 Pending | — |
| CHANGE_PROTOCOL.md | 🟡 Pending | — |
| SIMULATION_CONTRACT.md | 🟡 Pending | — |
| PERFORMANCE_CONTRACT.md | 🟡 Pending | — |
| GAMEPLAY_CONTRACT.md | 🟡 Pending | — |
| SECURITY_CONTRACT.md | 🟡 Pending | — |
| PROJECT_MEMORY_POLICY.md | 🟡 Pending | — |
| SCHEMA_EVOLUTION.md | 🟡 Pending | — |
| RESEARCH_POLICY.md | 🟡 Pending | — |
| RESEARCH_TO_IMPLEMENTATION.md | 🟡 Pending | — |
| DEPENDENCY_GRAPH_POLICY.md | 🟡 Pending | — |
| TESTING_POLICY.md | 🟡 Pending | — |
| EXPLOIT_AUDIT.md | 🟡 Pending | — |
| CONTENT_GOVERNANCE.md | 🟡 Pending | — |
| WORLD_GENERATION_POLICY.md | 🟡 Pending | — |
| DATA_MODEL_POLICY.md | 🟡 Pending | — |
| RENDERING_POLICY.md | 🟡 Pending | — |
| NETWORK_POLICY.md | 🟡 Pending | — |
| AI_OPERATIONAL_POLICY.md | 🟡 Pending | — |
| AUTOMATIC_UPDATE_PROTOCOL.md | 🟡 Pending | — |

**Note**: Policy documents (marked 🟡) are adapted from the governance pack but not yet written to this repo. They will be created as needed during Phase 0 completion.

---

## Phase 0 Acceptance Criteria Checklist

| Criterion | Status |
|-----------|--------|
| PROJECT_BIBLE.md exists | ✅ |
| ARCHITECTURE.md exists | ✅ |
| DEPENDENCY_GRAPH.md exists | ✅ |
| SYSTEM_CONTRACT.md exists | ✅ |
| DECISIONS.md exists | ✅ |
| TECH_DEBT.md exists | ✅ |
| OPEN_QUESTIONS.md exists | ✅ |
| CURRENT_STATE.md exists | ✅ |
| CHANGELOG.md exists | 🟡 Pending |
| Subagent policy defined | 🟡 Pending |
| DoD defined | 🟡 Pending |
| Change protocol defined | 🟡 Pending |
| Simulation contract defined | 🟡 Pending |
| Performance contract defined | 🟡 Pending |
| Gameplay contract defined | 🟡 Pending |
| Security contract defined | 🟡 Pending |
| Schema evolution policy defined | 🟡 Pending |
| Research policy defined | 🟡 Pending |
| Testing policy defined | 🟡 Pending |
| Exploit audit policy defined | 🟡 Pending |
| Content governance defined | 🟡 Pending |
| World generation policy defined | 🟡 Pending |
| Data model policy defined | 🟡 Pending |
| Rendering policy defined | 🟡 Pending |
| Network policy defined | 🟡 Pending |
| AI operational policy defined | 🟡 Pending |
| Automatic update protocol defined | 🟡 Pending |

**Phase 0 Complete**: When all ✅ and 🟡 items are ✅