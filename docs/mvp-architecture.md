# MVP architecture — Genesis slice 1

Architecture for the first vertical: load a fixture encounter, explore a small room in real time, enter turn-based PF2e combat against one enemy with four level-1 fighters (Move + Strike only).

This document is the implementation contract for slice 1. Product principles live in [constitution.md](./constitution.md). Scope lives in [mvp-slice-1.md](./mvp-slice-1.md). Research evidence lives in [research/](./research/README.md).

## Locked decisions

| Decision | Choice |
| --- | --- |
| Engine host | **Bevy** (pin to a minor compatible with chosen plugins, e.g. tilemap) |
| Rules / content | Pure Rust crates — **no Bevy dependency** in rules |
| MVP content | **Hand-authored fixtures** only (no ORC/AoN/Foundry packs) |
| Rules terminology | **Pathfinder Remaster** naming in fixtures, APIs, and tests |
| Combat positioning | **Snap to grid on encounter start**; exploration may feel continuous |
| Editor in slice 1 | **None** — game binary loads fixtures; crate boundaries must still allow a later editor binary |
| Networking | None |
| Combat clock | Turn-based only — no RTwP systems, even temporary |

## Goals and non-goals

### Goals

1. Prove Explore → Combat mode switch with PF2e initiative and 3-action turns.
2. Resolve Move (Stride) and Strike through a shared rules API with seeded, testable outcomes.
3. Present an IE-like chrome stub: four party portraits, action bar (Move / Strike), modal-capable layout — not a mobile/MMO HUD.
4. Keep rules headless-testable so fidelity work does not require the renderer.
5. Leave a clean seam for a future `editor` binary that links the same `rules` + `content` crates.

### Non-goals (slice 1)

- In-product content editor
- Full PF2e action/feat/spell/condition surface
- Character creation, inventory UI, dialogue, stealth, reactions beyond what Strike/Move need
- Campaign narrative or adventure scripting
- Difficulty scaling by living party size (party is always 4 slots)
- IE file formats (BAM/TIS/etc.) — presentation may *look* IE-like without adopting those formats

## Workspace layout

Cargo workspace at repository root:

```
genesis/
  Cargo.toml                 # workspace
  crates/
    rules/                   # PF2e procedures (Remaster naming); no engine deps
    content/                 # serde schemas, validation, pack/fixture loaders
    game/                    # Bevy app: modes, render, IE UI, input
  fixtures/                  # hand-authored MVP encounter/party/room data
  docs/
    crates/rules/README.md   # Constitution VI — required with each package
    crates/content/README.md
    crates/game/README.md
  TESTER.md                  # Constitution VIII — land with first runnable binary
```

Optional later (not required for first green loop): `crates/sim` for mode/encounter orchestration testable without Bevy.

**Hard rule:** `crates/rules` and `crates/content` must not depend on Bevy. `crates/game` depends on both.

## Crate responsibilities

### `rules`

Owns tabletop procedures for the MVP subset:

- Creature/combatant runtime state derived from prepared stats (HP, AC, attack bonus, damage, Speed, traits needed for Strike/Move)
- Check resolution (d20 + modifiers vs DC), degree of success
- Action economy: 3 actions per turn; MAP on repeated Strikes
- Legal action generation for the active combatant (Move, Strike) given position, Speed, reach, and remaining actions
- Initiative order advancement and encounter end detection (all enemies defeated / party wiped — define precisely in tests)
- Seeded RNG interface for rolls (injectable; tests supply a scripted sequence)

Architecture style (from research): **phased preparation + synthetics accumulator** inspired by Foundry PF2e Rule Elements — not IE opcodes. For slice 1, most fighter numbers can come straight from fixtures with a thin prepare pass; leave the phased hook points so feats/conditions can attach later without rewriting Strike.

Public surface (conceptual):

- `prepare_combatant(defn) -> Combatant`
- `start_encounter(party, enemies, positions, rng) -> Encounter`
- `legal_actions(encounter, id) -> Vec<Action>`
- `declare_action(encounter, id, action, rng) -> Result<Events, Illegal>`
- `advance_if_turn_done(encounter) -> …`

No rendering types. No Bevy `Entity` IDs inside rules — use stable `CombatantId` / `CreatureId` from content.

### `content`

Owns data shapes and I/O:

- Serde types for fixtures (RON or JSON; pick one primary format for MVP and stick to it)
- Schema validation (fail closed in tests/CI)
- `load_encounter(path) -> EncounterFixture` used by the game binary (and later by the editor)
- Remaster-oriented field names in schemas (`armor_class`, `strike`, etc. — consistent glossary in package docs)

Fixtures are **test content**, not campaign packs. Hand-authored only for MVP.

### `game`

Owns Bevy presentation and mode orchestration:

- App bootstrap, asset loading of fixtures via `content`
- `GameMode` states: `Explore` | `TransitioningToCombat` | `Combat`
- Explore locomotion (real-time); disable in combat schedules
- Snap-to-grid transition; combat movement on 5-foot squares
- Isometric (or IE-style) room view + Y-sorted actors
- IE-like UI: 4 portrait slots (always), action bar, basic modals/feedback
- Input action maps that emit the same intents rules understand (`MoveTo`, `Strike`)
- Bind Bevy entities ↔ `CombatantId` in a thin adapter layer — rules never see entities

## Runtime mode machine

```
Explore
  real-time party movement in the room
  encounter trigger (proximity / volume / fixture flag)
       │
       ▼
TransitioningToCombat
  cancel/freeze explore movers
  snap each combatant to combat grid
  roll initiative (MVP: Perception; seeded in tests)
  build turn queue; swap UI chrome
       │
       ▼
Combat
  active combatant only accepts orders (MVP)
  spend up to 3 actions: Move and/or Strike
  resolve through rules; apply events to view
  advance turns/rounds
  on encounter end → Explore (restore RT movement)
```

**Bevy mapping:** `States` / schedule sets so explore locomotion does not run in `Combat`. Events such as `EncounterStarted`, `TurnStarted`, `ActionResolved`, `EncounterEnded` bridge rules outcomes to presentation.

**Forbidden:** a parallel real-time combat clock, pause-as-combat, or “temporary RTwP.”

## Positioning and grid

| Phase | Behavior |
| --- | --- |
| Explore | Continuous (or fine-step) movement for feel |
| Encounter start | **Snap** all combatants to the nearest legal 5-foot square; reject overlaps per fixture spawn rules |
| Combat | Discrete grid movement; Move spends actions per PF2e Stride/Speed rules as implemented in `rules` |

Facing, flanking, and difficult terrain are out of scope unless required to make Move/Strike legal; do not invent video-game substitutes — omit or implement per Remaster procedures.

## Party model

- Exactly **4** PC slots in party state and UI, always.
- Dead PCs remain slots (portrait state shows dead); no encounter scaling when living count &lt; 4.
- MVP fixtures: four level-1 fighters with Move + Strike only.

## MVP rules surface (Remaster)

Implement only what the vertical needs, with tabletop citations in tests:

| Procedure | MVP expectation |
| --- | --- |
| Initiative | Roll once at encounter start; fixed skill (Perception) for all unless fixture overrides |
| Turn | Up to 3 actions |
| Move | Stride-equivalent: spend 1 action, move up to Speed on grid |
| Strike | Attack roll vs AC; damage on success; MAP −5 / −10 on 2nd / 3rd Strike that turn |
| HP / dying | Enough to end the fight (enemy to 0 HP; optional simple PC downed — document if simplified under Principle III) |
| Reactions / AoO | Out of scope unless a fidelity hole blocks Move/Strike; default omit |

Any intentional deviation from Remaster procedures requires a Principle III fidelity record (not a code comment).

## Content / fixture shape (illustrative)

Hand-authored files under `fixtures/` (exact schema owned by `content`):

- `encounters/room_one.json` (or `.ron`) — room id, trigger, enemy spawn, PC spawns
- `creatures/fighter_l1.json` — Remaster-oriented stat block + Strike definition
- `creatures/enemy_mvp.json` — single enemy
- `rooms/small_room.json` — walkable grid / collision for explore + combat

`content::load_encounter` is the **only** supported way to start the MVP scenario. The future editor must call the same loader/API.

## Presentation architecture

### World

- Orthographic camera; pan follows party centroid or active combatant
- Room: isometric tilemap **or** single background + collision grid — choose based on Bevy/plugin pin; combat grid must be first-class either way
- Actors: sprites with isometric depth sort; no MMO nameplate stack as primary UI

### UI (IE-like)

```
Portraits ×4 (always)     |          World view
                          |
Action bar: Move | Strike |     (combat: initiative / actions left)
```

- Player HUD uses **Bevy UI** (not egui). egui is reserved for later editor/debug tools.
- Modals for confirmations/errors; no mobile joystick / MMO unit-frame layout as the default look.

### Input

Logical action map → rules intents. Explore: move party / select portrait. Combat: choose Move or Strike → destination/target → confirm. Hotkeys bind to the same intents as the action bar.

## Testing strategy

| Layer | What |
| --- | --- |
| `rules` unit/integration | Seeded Strike/Move sequences; MAP; initiative order; tabletop source citations in tests |
| `content` | Fixture schema validation; reject invalid packs in CI |
| `game` (smoke) | Load fixture; assert Explore→Combat transition; one scripted player turn |
| `TESTER.md` | Manual walkthrough for Grok Bot: launch, move into trigger, take Move/Strike, finish fight |

Headless `cargo test -p rules -p content` must pass without a GPU. Do not land packages without matching `docs/crates/<name>/README.md`.

## Fidelity and deviation ledger

- Default: implement Remaster procedures for the MVP subset.
- If a rule cannot operate in a video game, record in project research/constitution materials (Principle III): rule + source, why it cannot fit, substitute, how intent is preserved.
- Snap-to-grid on encounter entry is an accepted presentation/adapter choice for digital play; combat procedures on the grid still follow tabletop intent. If snap itself ever conflicts with a specific Remaster procedure, file a Principle III record rather than silently drifting.

## Editor seam (deferred, but binding)

Slice 1 ships no editor binary. Binding constraints so Principle IV stays achievable:

1. Same `load_encounter` / content schemas for tools later.
2. No rules logic in Bevy systems beyond calling `rules` and applying results.
3. Future `crates/editor` is a separate binary linking `rules` + `content` (prefer subprocess play for isolation).
4. Tiled or other external map tools may generate fixtures optionally; they must not become a required authoring step for gameplay content long-term.

## Implementation sequence (suggested)

1. Workspace + `rules` + `content` skeletons, package docs, empty `TESTER.md` stub noting “not runnable yet” only until binary exists — replace with real walkthrough when runnable.
2. Hand fixtures for room, 4 fighters, 1 enemy.
3. Headless: prepare → initiative → scripted Move/Strike → enemy defeat.
4. `game`: load fixture, explore move, trigger, snap, combat UI, wire actions.
5. Pin Bevy/plugin versions; document in `crates/game` README.
6. Tester guide pass for the vertical.

## Success criteria

1. `cargo test` proves a seeded Strike and Move sequence against fixture stats.
2. Game binary loads the fixture encounter with no editor.
3. Player can walk into the room in Explore; encounter starts; movers freeze; grid snap occurs; initiative UI appears.
4. Each of the four PCs can take turns using Move and Strike only; enemy can be reduced to 0 HP.
5. No RTwP path exists in code or UI.
6. `TESTER.md` walkthrough covers the loop and passes on a clean VM build.

## References

- [constitution.md](./constitution.md)
- [mvp-slice-1.md](./mvp-slice-1.md)
- [research/stack-recommendation.md](./research/stack-recommendation.md)
- [research/rules-simulation.md](./research/rules-simulation.md)
- [research/turn-based-combat.md](./research/turn-based-combat.md)
- [research/editor-runtime.md](./research/editor-runtime.md)
- [research/rendering-ui.md](./research/rendering-ui.md)
