# Classic CRPG / Infinity Engine–like precedents

Lessons for Genesis UI, content pipelines, and engine/editor boundaries. Not a license to copy assets or proprietary formats wholesale.

## Infinity Engine family (product look target)

Constitution I asks for **isometric world view, party portraits, action bar, modal windows** — the IE / early Bioware CRPG grammar (Baldur’s Gate, Icewind Dale, Planescape: Torment), **not** a mobile or MMO HUD.

Useful open research / tooling (not a stack to fork as Genesis):

| Project | Role | Why relevant |
| --- | --- | --- |
| [GemRB](https://github.com/gemrb/gemrb) | Portable open reimplementation of IE | Shows how IE games separate **resource packs**, GUI, scripts, and engine; extensibility goal for mods |
| [IESDP](https://gibberlings3.github.io/iesdp/main.htm) | Community documentation of IE formats/opcodes | Documents how IE encoded rules as **data + opcodes** (ITM/SPL/EFF/CRE/ARE/…) |
| [Near Infinity](https://github.com/NearInfinityBrowser/NearInfinity) | Java IE resource browser/editor | Classic **external** content editor pattern — powerful, but Constitution IV forbids depending on an external editor for required authoring |
| DLTCEP / WeiDU (community) | Editing / mod install | Mod ecosystem lessons; not product runtime |

### Copy

- Clear **modal depth**: inventory, character sheet, dialogue, maps as windows over the world.
- Persistent **party strip** (portraits + vitals) and a bottom **action/command bar**.
- Resource layering / override paths (GemRB sources + IESDP override notes): fixture → override → authored content later.
- Treat many mechanics as **data tables + effect opcodes**, with a small interpreter core (IESDP effect application order docs).

### Avoid

- **Shipping without an in-product editor** while requiring Near Infinity–class tools for basic content (Constitution IV).
- Replicating IE’s opaque binary formats as Genesis’s native format — prefer readable serde formats (RON/JSON) with optional binary packaging later.
- Cloning IE’s RTwP combat. Genesis **must not** implement RTwP (Constitution II). IE look ≠ IE combat model.
- Hardcoding campaign narrative into the engine repo (Constitution V).

## GemRB architecture takeaways

From GemRB’s public description and core resource setup:

- Plugin/resource-source model: game data, override, and “unhardcoded” data layers.
- Engine aims for cleaner design than original IE while remaining format-compatible for existing games.

**Genesis lesson:** Separate **engine**, **rules data**, and **fixture content** early. MVP slice 1 should load encounter fixtures the same way later editor-authored packs will load — even if the editor UI does not exist yet.

## Modern PF2e / tabletop-faithful CRPGs (behavior, not stack)

| Product | Combat model | Lesson |
| --- | --- | --- |
| Owlcat Pathfinder: Kingmaker / Wrath | Designed for RTwP; TB added (Wrath announced/shipped TB toggle) | Dual combat modes are expensive; Owlcat stressed RTwP as primary vision while adding TB ([Wrath TB announcement](https://wrath.owlcat.games/news/16)). Genesis should **not** build RTwP “just in case.” |
| Larian Baldur’s Gate 3 | Explore RT; combat TB; optional forced TB in explore | Best modern precedent for **mode switch** (see [turn-based-combat.md](./turn-based-combat.md)) |
| Solasta: Crown of the Magister | TB-native; SRD 5.1 fidelity philosophy | Explicit “adapt game to tabletop rules” stance ([Director’s Log](https://www.solasta-game.com/solasta-crown-of-the-magister/news/11-directors-log-2-adapting-the-ruleset)); reaction windows, grid fidelity, AI shared across explore/combat |

### Copy from Solasta / BG3 (design)

- Document fidelity deviations (aligns with Constitution III records).
- One action-resolution pipeline used by player and AI.
- Explicit mode transitions with clear UI affordances (hourglass / TB chrome).

### Avoid from Owlcat dual-mode path

- Designing systems assuming RTwP timing, then bolting TB on. Genesis is TB-only in combat by constitution.

## Open “classic RPG maker” engines (peripheral)

- **Eldiron** (Rust, AWGY listing): classic RPG creation with world editing — inspirational for editor+runtime product shape; not evaluated as Genesis’s rules host.
- Do **not** assume any third-party maker understands PF2e fidelity.

## UI grammar checklist (IE-like)

For rendering/UI work ([rendering-ui.md](./rendering-ui.md)):

1. World viewport is the dominant plane (isometric or faux-iso).
2. Party portraits anchored (typically left or bottom), always for 4 slots.
3. Action bar for legal actions in current mode (MVP: Move, Strike in combat).
4. Modals for sheets/inspect; pause-or-freeze world while modal open as appropriate.
5. Cursor feedback for move/attack legality (Solasta/BG3/Owlcat TB pathing cues are modern expectations).

## What Genesis should *not* inherit from IE tech debt

- Monolithic opcode soup without typed Rust APIs for tests.
- Editor-only knowledge of rules (Near Infinity understanding ≠ runtime understanding).
- Campaign scripts as the only integration tests — prefer rules unit/integration tests on fixtures.
