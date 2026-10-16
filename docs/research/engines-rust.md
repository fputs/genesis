# Rust game engines and frameworks

Research snapshot: **2026-10-16**. Versions from [Are We Game Yet? engines](https://arewegameyet.rs/ecosystem/engines/) and official release notes unless noted.

## Decision framing for Genesis

Genesis needs:

- Long-lived systems (rules, initiative, modes, party UI) more than a jam prototype.
- 2D / isometric rendering good enough for IE-like presentation.
- A path to an **in-product editor** that shares the same simulation (Constitution IV).
- Headless or near-headless rules testing (Constitution III fidelity).

That biases toward engines with strong ECS/plugin ecosystems **or** a mature shipped editor — not the lightest 2D wrapper.

## Bevy (primary candidate)

| Field | Value (as of research date) |
| --- | --- |
| Latest on AWGY / crates.io ecosystem listing | **0.20.0** |
| License | MIT OR Apache-2.0 |
| Architecture | Data-driven ECS; systems + schedules; plugin ecosystem |
| Official release | [Bevy 0.20](https://bevy.org/news/bevy-0-20/) posted **2026-10-08** (227 contributors, 817 PRs) |

### Strengths relevant to Genesis

- **Largest Rust game ecosystem** (downloads/stars/activity on AWGY vs peers). Plugins for tilemaps, input maps, egui tooling, Tiled loaders, etc.
- **ECS fit** for many concurrent actors, conditions, auras, and mode-gated systems. `bevy_ecs` is usable as a crate; relationships landed in 0.16 ([Bevy 0.16](https://bevy.org/news/bevy-0-16/), 2025-04-24).
- **2D path is first-class**: sprites, cameras, UI; 0.20 continues Feathers (editor-centric UI widgets) and BSN scene work.
- **States / schedules** map cleanly to Explore vs Combat (see [turn-based-combat.md](./turn-based-combat.md)).
- **Editor trajectory**: first-party editor is not shipped, but BSN + Feathers + published editor roadmap exist ([editor roadmap](https://bevyengine.github.io/bevy_editor_prototypes/roadmap.html)). Custom Bevy tools can share the game World/plugins.

### Risks

- **Breaking majors roughly every quarter.** Migration guides exist (e.g. [0.19 → 0.20](https://bevy.org/learn/migration-guides/0-19-to-0-20/)); plan for churn.
- **No shipped first-party editor** as of 0.20. Roadmap Stage 2+ still aspirational; do not block MVP on Bevy Editor.
- **Ecosystem lag**: community crates (tilemap, egui, input) often pin to N−1 Bevy. Example: `bevy_ecs_tilemap` docs.rs latest observed at **0.19.0** while Bevy is **0.20.0** — verify compatibility before pinning.
- MSRV policy historically tracks recent stable Rust (engine docs claim “latest stable” style expectations).

### Fit summary

Best default for a **programmer-owned engine** where rules/sim are the product core and the editor can be built as a second binary on the same crates. Matches MVP “engine-only fixtures” and leaves a Bevy-native editor path open.

## Fyrox (strong alternative when editor-first)

| Field | Value |
| --- | --- |
| Latest on AWGY | **1.0.1** |
| Stable 1.0 blog | [Fyrox 1.0.0](https://fyrox.rs/blog/post/fyrox-game-engine-1-0-0/) (stable after ~7 years; RC notes referenced from that post) |
| License | MIT |
| Architecture | Scene graph + Rust scripts on nodes; **FyroxEd** native editor |
| Docs | [Fyrox Book — editor](https://fyrox-book.github.io/beginning/editor_overview.html), [scripts](https://fyrox-book.github.io/scripting/script.html), [graph](https://fyrox-book.github.io/scene/graph.html) |

### Strengths

- **Ships a real visual editor today** (World Viewer, Scene Preview, Inspector, Play Mode via separate process). Directly addresses Constitution IV earlier than Bevy.
- Play Mode isolation (game as child process) is a healthy editor/runtime pattern.
- 2D and 3D supported; export CLI for CI (`export-cli` in 1.0 notes).

### Risks

- **Smaller team/ecosystem** than Bevy (AWGY downloads/stars much lower). Longevity and third-party CRPG-specific crates are thinner.
- **Scene-graph + script model**, not Bevy-style ECS. Large rules simulations are still doable in Rust, but community patterns/plugins differ.
- Post-1.0 note from maintainers: major new features paused for months while bugfixing — plan for slower feature velocity.

### Fit summary

Strong if the user prioritizes **authoring UX immediately** over Bevy’s ecosystem. Still viable for MVP if rules live in shared Rust libs, but Genesis’s long-term sim complexity likely wants Bevy-scale community unless Fyrox is a deliberate commitment.

## Lighter Rust 2D frameworks (generally insufficient alone)

From AWGY (2026-10-16):

| Crate | Version listed | Role |
| --- | --- | --- |
| macroquad | 0.4.16 | Minimal immediate-mode 2D; great jams, weak for IE UI + editor product |
| ggez | 0.10.0 | Love2D-like 2D framework; slower ecosystem momentum |
| tetra | 0.9.1 | Simple 2D; smaller community |

These can prototype movement/combat loops quickly but force reinventing ECS, tooling, and editor integration. **Not recommended as the product foundation** given Constitution IV and fidelity scope.

## godot-rust (Godot 4 + Rust via GDExtension)

| Field | Value |
| --- | --- |
| Project | [godot-rust / gdext](https://godot-rust.github.io/), [github.com/godot-rust/gdext](https://github.com/godot-rust/gdext) |
| crates.io `godot` | **0.5.5** (2026-08-09); default API target Godot 4.6 per release notes |
| Model | Godot hosts scenes/editor; Rust as `cdylib` extension |

### Strengths

- **Mature editor and 2D tooling** out of the box.
- Usable for games, tools, and editor plugins per project docs.

### Risks for Genesis constraints

- Product becomes **Godot + Rust**, not a pure Rust engine. Campaign/content workflows lean on Godot scenes; dual mental model (GDScript/editor nodes vs Rust sim).
- Constitution/project context emphasize a **Rust engine + editor**. Godot can still host Rust sim crates, but the “engine” identity and packaging diverge.
- Fidelity testing and headless CI need careful separation so rules do not live only in Godot nodes.

### Fit summary

Credible **alternative** if editor velocity outweighs “Rust-native engine” preference. Document as secondary option, not primary, unless the user explicitly wants Godot.

## Other names seen (low priority)

- **Eldiron** (AWGY): classic RPG creator with world editing — interesting domain overlap, but not evaluated as a dependency host for a custom PF2e engine; treat as inspiration, not stack.
- **Renzora / space_editor**: Bevy-adjacent editor experiments; space_editor last activity ~2025-05 on AWGY — **do not assume** current Bevy 0.20 support without verification.
- **Amethyst**: archived; historical ECS influence only.

## ECS vs scene graph (short)

| Concern | Bevy ECS | Fyrox scene graph | Godot nodes |
| --- | --- | --- | --- |
| Many combatants + conditions | Natural | Manual bookkeeping or custom systems | Nodes + groups; OK but different |
| Headless rules tests | Extract `bevy_ecs` or keep rules Bevy-free | Keep rules outside Fyrox | Keep rules outside Godot |
| Shared editor/runtime | Same App plugins / World | Same scripts + Play Mode | Same GDExtension + tools |
| Community CRPG/tilemap plugins | Strongest | Weaker | Strong (Godot), Rust-specific weaker |

**Recommendation lean:** Keep **rules simulation engine-agnostic** (pure Rust crates). Put only presentation, input, and tool shells in the chosen engine. That preserves Bevy vs Fyrox vs Godot optionality longer.
