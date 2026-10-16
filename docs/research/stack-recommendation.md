# Stack recommendation

Research snapshot: **2026-10-16**. Evidence lives in the sibling notes; this file is the decision memo.

## Primary recommendation

**Bevy (pin to a tilemap-compatible minor) + engine-agnostic PF2e rules crates + Bevy UI game chrome + deferred custom Bevy editor binary.**

| Layer | Choice | Notes |
| --- | --- | --- |
| Language | Rust (edition matching Bevy’s current) | Required by product context |
| App framework | **Bevy** (latest compatible with chosen plugins; market latest **0.20.0** as of 2026-10-08) | Largest ecosystem; ECS/schedules for modes |
| Rules / sim | **Pure Rust crates** (`rules`, `content`, maybe `sim`) — **no** Bevy dependency | Constitution III/IV; headless tests |
| Content format | **RON or JSON + serde**, schema-validated | Fixtures now; editor later; round-trip tests |
| World presentation | Orthographic 2D; **isometric tiles** via `bevy_ecs_tilemap` *or* room background + Y-sorted sprites for MVP | IE look without IE formats |
| Game UI | **Bevy UI** (Feathers widgets where useful) themed IE-like | Portraits, action bar, modals — not egui as player HUD |
| Tooling UI | `bevy_egui` / Feathers in a future `editor` binary | Separate from player chrome |
| Input | Logical action map (`leafwing-input-manager` if version-aligned, else Bevy input) | Explore vs combat maps |
| Map authoring (optional MVP) | Tiled → fixture import OK temporarily | Must not become the required long-term editor |
| Audio | Bevy audio defaults when needed | Out of scope for slice 1 loop |
| Networking | **None** | Constitution II |

### Why this wins for Genesis

1. **Constitution I–II:** Bevy 2D + custom UI can express IE chrome and Explore/Combat `States` without RTwP baggage.
2. **Constitution III:** Pure rules crates + Foundry-like RE/phased prep (see [rules-simulation.md](./rules-simulation.md)) maximize fidelity testing.
3. **Constitution IV:** Shared crates + future editor binary (Fyrox-style subprocess play) avoid a second ruleset; Bevy’s missing first-party editor is mitigated by **not** putting rules in the UI layer.
4. **Constitution V / MVP:** Engine-only fixture load is natural; no campaign tooling required.
5. Ecosystem depth for tilemaps, input, inspectors beats Fyrox/macroquad for a long rules-heavy project.

### Explicit non-choices in the primary path

- Not Fyrox-first (editor today vs ecosystem/ECS fit tradeoff — see Alternative A).
- Not Godot-hosted (splits “Rust engine” identity — see Alternative B).
- Not macroquad/ggez as foundations.
- Not waiting on official Bevy Editor shipping.

## Alternative A — Fyrox 1.0 first

**When to pick:** User prioritizes **shipping in-product scene authoring immediately** over Bevy ecosystem and ECS patterns.

| Layer | Choice |
| --- | --- |
| Engine + editor | Fyrox **1.0.x** + FyroxEd |
| Rules | Same pure Rust `rules`/`content` crates |
| UI | Fyrox UI for IE chrome |

**Tradeoffs:** Real editor and Play Mode isolation now; smaller community; scene-graph idioms; slower post-1.0 feature velocity per maintainer notes. Still acceptable if rules stay external to scripts.

## Alternative B — Godot 4 + godot-rust for shell, Rust for rules

**When to pick:** User accepts a **Godot-shaped product** to gain mature 2D UI/editor immediately, while keeping PF2e sim in Rust.

| Layer | Choice |
| --- | --- |
| Host | Godot 4.x |
| Rust | `godot` crate **0.5.x** GDExtension for rules + game systems |
| Rules | Same pure Rust crates, exposed to Godot |

**Tradeoffs:** Fastest general authoring; dual mental model; packaging/engine identity drifts from “Rust CRPG engine”; discipline required so GDScript never forks rules.

## Mapping to MVP slice 1

| MVP need | Primary stack approach |
| --- | --- |
| Load fixture room + 4 fighters + 1 enemy | `content` loader + `assets/fixtures/…` |
| Explore real-time move | Bevy explore state + simple nav |
| Switch to TB | `GameMode` state + initiative resource |
| Move + Strike only | `rules` action API; bar buttons + hotkeys |
| No editor | Game binary only; schemas stable for later editor |
| IE-ish UI | Portraits + action bar stubs in Bevy UI |

## Risks and open decisions (need user input where marked)

| Item | Risk | Recommendation / decision |
| --- | --- | --- |
| Bevy plugin version lag (`bevy_ecs_tilemap` observed on **0.19** vs Bevy **0.20**) | Build friction | **Pin Bevy to plugin-supported version** for MVP, or defer tilemap crate and use background+sprites until aligned |
| Bevy release churn | Ongoing migration cost | Budget upgrades; keep rules crates Bevy-free to reduce blast radius |
| No first-party Bevy editor | Principle IV timeline | Commit to custom editor binary post–slice 1; do not block MVP |
| PF2e data licensing | Legal | **Hand-author MVP fixtures**; decide ORC/AoN/Foundry-derived data policy before large packs (**user decision**) |
| Combat grid vs continuous explore | Design | Prefer snap-to-grid on encounter entry for fidelity (**user decision** if aesthetic prefers continuous combat) |
| Player HUD toolkit | Fidelity vs speed | Prefer Bevy UI for player; egui only for debug/editor |
| Remaster vs pre-Remaster terminology | Rules text | **User decision** — engine should name editions in fixtures/tests |
| Engine host finality | Bevy vs Fyrox vs Godot | Primary = Bevy unless editor-now or Godot-host is preferred (**user decision** if dissenting) |

## Suggested workspace layout (brief, research-grounded)

Grounded in shared-sim patterns ([editor-runtime.md](./editor-runtime.md)), not a full scaffold:

```
genesis/                 # cargo workspace
  crates/
    rules/               # PF2e procedures, checks, actions (no engine deps)
    content/             # serde schemas, validation, pack loading
    game/                # Bevy app: explore/combat, rendering, IE UI
    editor/              # later: Bevy tool app linking rules+content
  fixtures/              # MVP encounter/party/room data
  docs/…                 # Constitution VII package docs when code exists
  TESTER.md              # Constitution VIII when runnable
```

Optional later: `crates/sim` if mode/encounter orchestration should be testable without Bevy.

## Success criteria for adopting this stack

1. Headless tests resolve a scripted Strike with seeded dice.
2. Game binary loads a fixture encounter without editor.
3. Explore → combat transition freezes RT movers and shows TB chrome.
4. Same `load_encounter` API remains the editor entry point later.
5. No RTwP systems introduced “temporarily.”

## See also

- [engines-rust.md](./engines-rust.md)
- [rules-simulation.md](./rules-simulation.md)
- [editor-runtime.md](./editor-runtime.md)
- [turn-based-combat.md](./turn-based-combat.md)
- [rendering-ui.md](./rendering-ui.md)
- [sources.md](./sources.md)
