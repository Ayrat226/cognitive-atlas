# LITHOS — Project Bible

## Identity

**Working Title:** LITHOS  
**Genre:** Voxel Planetary Simulation / Emergent Technology Sandbox  
**Target Platform:** PC (Linux/Windows), dedicated server for multiplayer  
**Rendering:** Vulkan, GPU-driven, PBR, hierarchical geometry  
**Simulation Language:** Rust (core), optional Lua/WASM for user scripts  
**Network Model:** Server-authoritative, interest-managed replication

---

## Core Fantasy

A Minecraft-like voxel world where technology, ecology, manufacturing, and civilization emerge from a coherent simulation rather than fixed recipes. You don't craft a "steam engine" from a recipe — you build a pressure vessel, a piston, a crankshaft, connect them with piping, feed it fuel and water, and it works because the physics says it works. Or it explodes because you miscalculated.

The world is an Earth-like procedural planet: tectonics, hydrology, climate, ecology, geology. No human civilization by default — you bring it. Every modification persists. The simulation runs at multiple levels of detail: detailed where you watch, aggregated where you don't.

---

## Visual Identity

- **16×16 texture discipline** — crisp, readable, stylized
- **Voxel readability** — clear silhouettes, distinct materials
- **Stylized human/avatar proportions** — ~2m tall, recognizable
- **Dense industrial language** — pipes, cables, gears, belts, machinery
- **Modern lighting/material response** — PBR, directional shadows, GI approximations
- **Readable silhouettes** — every machine/block identifiable at distance

---

## Design Pillars

| Pillar | Meaning |
|--------|---------|
| **Causality** | Every effect traces to a cause. No magic. Conservation laws hold. |
| **Freedom** | Players discover solutions. Constraints enable creativity. |
| **Engineering** | Real trade-offs: efficiency vs. complexity, precision vs. speed, safety vs. output. |
| **Discovery** | Unknown materials, processes, technologies reward experimentation and measurement. |
| **Automation** | Ladder: Manual → Tool-assisted → Machine → Automation → Software → AI → Civilization. |
| **Exploration** | Planetary scale. Procedural geology, climate, ecology. Resources where geology puts them. |
| **Multiplayer Emergence** | 50+ players. Shared persistent world. Economy, organizations, conflict, cooperation emerge. |
| **Long-term Persistence** | World modifications survive. Server restarts don't reset progress. |

---

## Player Journey

```
Primitive Survival
    ↓
Materials (mining, smelting, refining, alloys)
    ↓
Manufacturing (machining, forming, assembly, tolerances, wear)
    ↓
Industry (logistics, power, heat, automation, quality control)
    ↓
Electronics (sensors, logic, signal processing, control systems)
    ↓
Computing (CPU, ISA, memory, OS, software, networking)
    ↓
Automation (robotics, remote control, AI, swarm coordination)
    ↓
Aerospace (flight, orbit, spacecraft, life support)
    ↓
Space (Moon, planets, stations, interplanetary logistics)
    ↓
Advanced Research (material discovery, physics experiments, speculative tech)
```

Each stage unlocks new measurement precision, new materials, new processes, new energy densities, new computing power, new mobility.

---

## World

**Earth-like procedural planet** — tectonics → uplift → erosion → sedimentation → hydrology → climate → soil → ecology.  
**Persistent modifications** — Store deltas/events, not re-saving immutable procedural base.  
**Human civilization absent by default** — Ruins, artifacts, and anomalies possible.  
**Cross-scale consistency** — Local voxel simulation matches regional aggregate matches planetary model.

---

## Technology Freedom

Systems expose **constraints and capabilities**, not just recipe IDs.  
A machine defines: input ports, output ports, energy requirements, thermal limits, mechanical stress limits, wear rates, tolerances, control signals.  
Players combine, modify, optimize. The simulation resolves the result.

---

## Multiplayer Target

**50+ concurrent players** as early architectural target.  
Server-authoritative simulation. Clients send intent/input.  
Interest management: distance, line of sight, interaction, future trajectory, simulation importance, team relevance.  
Hierarchical region relevance for planetary scale.

---

## Long-term Vision

A **stable simulation platform** on which large quantities of emergent technology can be built without repeated rewrites of engine fundamentals.  
Modding as first-class: validated schemas, resource budgets, sandboxed scripts, deterministic VM.

---

## Scientific Integrity Policy

Every researched claim classified: **FACT / CONSENSUS / APPROXIMATION / ASSUMPTION / UNKNOWN / SPECULATIVE**.  
If exact model too expensive: preserve intended behavior, document approximation, define upgrade path, never call approximation exact simulation.  
All physical quantities have unambiguous units. No magic-unit fields.

---

## Content Governance

**Data-driven** — Content in data definitions, not code branches.  
Every item: stable ID, tags, dependencies, provenance, lifecycle, localization-ready metadata, icon/model refs, tests.  
**Generated content** — Deterministic, collision-resistant IDs.  
**Deprecation** — Never remove silently. Mark deprecated, migrate, then remove when safe.

---

## Schema Evolution

All persistent/network/data schemas versioned.  
Never silently change meaning of existing field.  
Add migration for incompatible changes.  
Test old save → new save → load → rollback.  
Network protocol changes require compatibility strategy.

---

## Security Contract

- Never execute unrestricted native user code on server
- Sandbox: instruction quota, CPU budget, memory quota, IO restrictions, filesystem sandbox, network sandbox, deterministic VM/JIT
- Clients send intent, not authoritative state
- Mods validated: schemas, dependencies, resource budgets, file paths, scripts, network access
- Test against: infinite loops, excessive entity creation, pathological networks, packet floods, giant recursive data, simulation denial, malformed saves/mods
- No server credentials, filesystem paths, internal keys, or privileged APIs exposed to gameplay scripts

---

## Exploit Audit Checklist

After each major system:
- Can matter be duplicated?
- Can energy be created?
- Can processing loop produce net-positive resources without input?
- Can fluids bypass conservation?
- Can aggregate→detailed conversion duplicate state?
- Can player create infinite entities?
- Can user code monopolize CPU?
- Can network state be desynchronized?
- Can low-cost input produce high-value output without intended constraints?
- Can a technology eliminate all meaningful trade-offs?

Classify: CRITICAL / HIGH / MEDIUM / LOW.  
Never silently patch emergent behavior before classifying it.