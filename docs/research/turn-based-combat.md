# Turn-based combat and explore → combat switching

## Genesis requirements

From Constitution II + MVP slice 1:

- Exploration: **real-time** movement.
- Encounter start: switch to **turn-based** initiative + PF2e action economy.
- Combat **must not** be RTwP.
- MVP actions: **Move** and **Strike** only; 4× L1 fighters vs one enemy in one room.

## Modern precedents

### Baldur’s Gate 3 (closest mode-switch model)

Official / wiki-described behavior:

- Default exploration is real-time.
- **Combat enters turn-based automatically.**
- Players can also enter **forced turn-based** outside combat (stealth, traps, hazards) via UI (hourglass) / hotkey ([Larian combat/stealth news](https://baldursgate3.game/news/a-little-about-combat-stealth_3), [bg3.wiki Turn-based mode](https://bg3.wiki/wiki/Turn-based_mode)).
- TB ends when combat/hazard conditions end, or when toggled off outside combat.

**Copy for Genesis:**

- Explicit `Explore` vs `Combat` (and later optional `TacticalExplore`) modes.
- Automatic transition on encounter trigger.
- Clear UI chrome when TB is active (initiative order, action points / actions remaining).

**Do not copy blindly:**

- BG3’s D&D 5e action economy and “team initiative bundling” differ from PF2e’s per-creature initiative and 3-action turns.
- Multiplayer pacing optimizations are irrelevant (Genesis is single-player).

### Owlcat Pathfinder (Wrath)

- Primary design was RTwP; TB added as a toggleable combat mode ([Owlcat announcement](https://wrath.owlcat.games/news/16)).
- Hotkey toggle between RTwP and TB in combat (community control guides).

**Lesson:** Building dual combat clocks is costly. Genesis constitution forbids RTwP — implement **one** combat clock (TB) from day one.

### Solasta

- TB-native; strong documentation of adapting tabletop action economy, reactions with timed windows, and AI usable in both combat and exploration ([AI diary](https://www.solasta-game.com/solasta-crown-of-the-magister/news/16-dev-diary-5-ai-basics)).

**Copy:**

- Shared decision/action evaluation for player tools and AI.
- Fidelity-first philosophy when mapping tabletop procedures to UI.

## PF2e combat loop (engine-facing sketch)

Tabletop (Remaster/Core procedures — implement from rulebooks; this is a structural sketch, not a substitute):

1. **Encounter start** → roll initiative (Perception or skill as appropriate; MVP can fix Perception).
2. Enter **rounds**; each creature’s turn: up to **3 actions** (plus reactions when triggered).
3. MVP subset: spend actions on **Stride/Move** and **Strike** only; track MAP for repeated Strikes.
4. Encounter end → leave combat mode; restore exploration movement.

Engine implications:

- Initiative order is a **resource** (sorted combatant list + current index).
- Action economy is **per turn state**, not real-time cooldowns.
- Movement in combat is discrete against remaining Speed / action costs; exploration movement is continuous (or interpolated) without action points.

## Recommended mode architecture (engine-agnostic)

```
ExploreMode
  - real-time party movement
  - encounter sensors / scripts / proximity

TransitioningToCombat
  - freeze/cancel explore movers
  - snapshot positions to combat grid/nav
  - roll initiative, build turn queue
  - swap UI chrome

CombatMode
  - only active combatant accepts orders (MVP)
  - resolve actions through rules crate
  - advance turn/round
  - detect encounter end → Exploring
```

### Bevy mapping (if Bevy chosen)

- `States` / sub-states for `GameMode::{Explore, Combat}`.
- Separate schedule sets so explore locomotion systems do not run in combat (and vice versa).
- Events: `EncounterStarted`, `TurnStarted`, `ActionDeclared`, `EncounterEnded`.

### Fyrox / Godot mapping

- Same state machine as a Rust resource/object; presentation hooks into engine scene.
- Avoid duplicating rules in scene scripts.

## Grid / positioning decisions (open)

IE classics often use continuous movement with engagement radii; Solasta uses explicit cells; PF2e tabletop uses squares.

For Genesis fidelity:

- Prefer a **combat grid** (5-foot squares) aligned with PF2e facing/reach/flanking later.
- Exploration may still feel continuous by interpolating along the same underlying grid or a finer navmesh that snaps on combat entry.

**Open decision for user:** continuous explore positions snapped on combat entry vs always-on grid. MVP one-room fight can snap on encounter start.

## Input differences by mode

| Mode | Primary input |
| --- | --- |
| Explore | click/WASD-style party move, select portrait |
| Combat | select legal action → select target/destination → confirm; hotkeys for Strike/Move |

Use an **action map** layer (logical actions) so the same `Strike` intent can come from bar click or keybind ([rendering-ui.md](./rendering-ui.md)).

## Testing hooks

- Deterministic initiative with seeded RNG.
- Scripted action sequences for the MVP encounter (4 fighters vs 1 enemy).
- Assert mode transitions: explore systems disabled while `Combat` active.
