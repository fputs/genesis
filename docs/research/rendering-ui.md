# Isometric rendering, IE-like UI, and input

## Visual target

Constitution I:

- Isometric (or IE-style faux-iso) world view.
- Party portraits, action bar, modal windows.
- **Must not** use mobile/MMO HUD as the primary look.

MVP: one small room, four PCs, one enemy — enough to prove camera, picking, portraits, and combat bar.

## Isometric rendering options (Rust / Bevy-centric)

### Tilemaps

[`bevy_ecs_tilemap`](https://github.com/StarArawn/bevy_ecs_tilemap) (docs.rs **0.19.0** at research time):

- Supports isometric **Diamond** and **Staggered** layouts (`TilemapType::Isometric`).
- Examples: `iso_diamond.rs`, `iso_staggered.rs`, `3d_iso` with `y_sort`.
- Entity-per-tile ECS model fits Bevy.
- Tiled integration via helpers / [`bevy_ecs_tiled`](https://github.com/adrien-bon/bevy_ecs_tiled) (isometric maps claimed in README).

**Risk:** version lag vs Bevy **0.20** — pin Bevy to the tilemap-supported version **or** vendor/patch until crates catch up.

### Non-tilemap / mesh sprites

- Place character sprites in world space with custom depth sorting (Y-sort / isometric depth).
- Fine for MVP actors even if the floor is a single background mosaic (true IE: prerendered backgrounds + animations).

### Classic IE approach (GemRB / IESDP)

- Prerendered area backgrounds (MOS/TIS/WED era) + BAM animations.
- Genesis need not adopt IE formats; the **presentation idea** (painted room + actors) is valid for MVP fixtures.

**MVP recommendation:** isometric tile floor **or** single room background texture + sorted actor sprites. Prefer tilemap if combat grid is first-class ([turn-based-combat.md](./turn-based-combat.md)).

## Camera

- Orthographic camera; pan follows active character or party centroid.
- Zoom constrained for readability of portraits/bar.
- Combat may tighten framing; avoid MMO free-fly cameras as default UX.

## IE-like UI structure

```
┌──────────────────────────────────────────────┐
│ Portraits (4 slots)                 Menus    │
│ ┌────┐┌────┐┌────┐┌────┐                     │
│ │ P1 ││ P2 ││ P3 ││ P4 │     WORLD VIEW      │
│ └────┘└────┘└────┘└────┘                     │
│                                              │
│  [ initiative strip in combat ]              │
│                                              │
│  Action bar: [Move] [Strike] …              │
└──────────────────────────────────────────────┘
         Modal layer: sheet / inspect / pause
```

### Implementation choices on Bevy

| Approach | Use for | Notes |
| --- | --- | --- |
| **Bevy UI + Feathers** | Game HUD (portraits, bars, modals) | Feathers is editor-centric but widgets expand in 0.19–0.20; retainable look with custom theming |
| **bevy_egui** | Editor tools, debug inspectors | Immediate-mode; easy tools; harder to hit classic IE art-directed HUD |
| Hybrid | egui editor / Bevy UI game | Recommended split |

Avoid defaulting the **player** HUD to floating MMO quest trackers, huge minimaps, or mobile thumb clusters.

## Input

Logical actions (recommended):

- `SelectPartyMember(0..3)`
- `Move` / `Strike` (context: explore move vs combat action)
- `Confirm` / `Cancel`
- `ToggleInspect`
- Later: camera pan/zoom

Crates:

- [`leafwing-input-manager`](https://github.com/Leafwing-Studios/leafwing-input-manager): Actionlike maps, rebinding, UI→action wiring. Verify Bevy version compatibility when pinning.
- Bevy built-in input + picking: enough for MVP if leafwing lags.

**Picking:** screen → world → tile/actor. `bevy_ecs_tilemap` has mouse-to-tile helpers/examples; isometric picking must use the same coord system as rendering.

**UI absorb:** when modals open, block world click-to-move (`bevy_egui` documents absorb-input patterns; Bevy UI needs explicit focus gating).

## Modal windows

- Stack or exclusive modal for character sheet / inspect.
- Dim or freeze world interaction while open; combat clock policy is an open design choice (pause TB timer vs disallow open during enemy turn). MVP can freeze input only.

## Performance / art pipeline (brief)

- Nearest sampling for pixel/hand-painted IE look (`ImagePlugin::default_nearest()` pattern in tilemap examples).
- Atlas sprites for characters; placeholder art acceptable for fixtures.
- No requirement for 3D PBR; Solari/path tracing in Bevy 0.20 is irrelevant to MVP CRPG presentation.

## Fyrox / Godot UI notes

- Fyrox: native UI widgets + editor; can build IE layout, less community CRPG HUD prior art in Rust blogs.
- Godot: Control nodes excel at windowed RPG UIs; theming to IE look is art work, not engine work.

## Open decisions

1. True isometric tiles vs prerendered room background for MVP.
2. Always-on grid vs snap-on-combat.
3. Game HUD toolkit: Bevy UI-only vs minimal egui for first vertical (egui is faster to iterate, weaker IE fidelity).
