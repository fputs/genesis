# Editor and runtime sharing one simulation

## Constitution requirement

Principle IV: game runtime and content editor are one product; they **share the same engine and rules implementation**. No separate editor ruleset. Required authoring must not depend on an external editor (text/image/VCS OK).

MVP slice 1 is **engine-only** with fixture encounter load — but the stack must not force a future fork.

## Patterns that work

### A. Shared libraries, multiple binaries (recommended default)

```
crates/
  rules/          # pure sim
  content/        # serde schemas + validation
  runtime-app/    # player-facing game
  editor-app/     # authoring UI (later)
  fixtures/       # MVP test data
```

- Editor and game link the same `rules` + `content` crates.
- Editor may use heavier UI (egui/Feathers) and debug cameras; game uses IE chrome.
- Fixtures and later “packs” use the **same loader**.

This is the safest path under Bevy (no first-party editor yet) and still valid under Fyrox/Godot.

### B. Same process, tool mode flag

One binary with `--editor` or a main-menu “Editor” that swaps plugin sets:

- Pros: one artifact; easy to guarantee same code.
- Cons: harder isolation (editor bugs can take down play); larger binary.

Fyrox’s Play Mode prefers **separate process** for crash isolation ([editor overview](https://fyrox-book.github.io/beginning/editor_overview.html)) — worth copying even on Bevy.

### C. Engine-shipped visual editor (Fyrox / Godot)

| Host | Editor status (research date) | Shared sim tactic |
| --- | --- | --- |
| **Fyrox 1.0** | FyroxEd ships; scripts registered in `PluginConstructor::register` | Put PF2e rules in Rust libs; scripts/UI call them; do not reimplement rules in editor-only code |
| **Godot 4 + godot-rust** | Mature editor | Rust GDExtension owns rules; editor plugins call into it; avoid GDScript duplicate rules |
| **Bevy** | First-party editor **not shipped**; roadmap stages 0–5 described | Build custom editor app on Bevy + Feathers/egui; invest in BSN/RON scene IO |

### D. External editors for *optional* art only

Tiled/Aseprite/Blender for maps/sprites is fine if:

- Genesis still ships the **content compiler/import** and gameplay authoring, and
- Required encounter/creature/action authoring is in-product (later).

MVP may use Tiled → checked-in fixtures; that does not violate IV if MVP has no required authoring path yet — but long-term encounter authoring must move in-product.

## Bevy-specific editor trajectory

Evidence:

- [Bevy editor prototypes roadmap](https://bevyengine.github.io/bevy_editor_prototypes/roadmap.html) — Stage 1 viewer → Stage 2 basic edit → later project manager; BSN preferred for scenes.
- Bevy 0.19–0.20: BSN scene system + Feathers widgets; 0.20 “What’s Next” includes `.bsn` asset format for tool-driven authoring ([Bevy 0.20](https://bevy.org/news/bevy-0-20/)).
- Community: `bevy_egui`, experimental map editors (`bevy_map_editor` targeting Bevy 0.18 per its README at research time), `space_editor` (likely stale vs 0.20).

**Do not wait for Bevy Editor 1.0** to satisfy Constitution IV. Plan a **Genesis Editor** binary early in design (post–MVP slice 1), even if thin (inspect entities, edit encounter RON, play in subprocess).

## Anti-patterns (avoid)

1. **Editor simulates with simplified rules** (“close enough for placement”) — violates IV and breeds fidelity bugs.
2. **Two schemas** (editor JSON vs runtime binary) without a single source of truth and round-trip tests.
3. **Near Infinity–style external-only tooling** as the required pipeline.
4. **Blocking MVP** on a full editor — slice 1 correctly defers UI authoring; it must still load the future format.

## Minimum editor-ready contracts for MVP (even without editor UI)

Ship these with the engine-only MVP so the editor is not a rewrite:

1. Stable **content schema** for: party members, enemy, room/encounter, weapon Strike stats.
2. `load_encounter(path) -> Result<Encounter, _>` used by the game.
3. Validation errors with paths/fields (editor will display the same errors).
4. Round-trip test: load fixture → serialize → load again → equal canonical form.
5. Headless “resolve one Strike” API callable without windowing.

## Process isolation recommendation

When editor gains Play:

- Spawn game as child process with a temp pack or IPC scene handle (Fyrox pattern).
- Keep rules deterministic under a seed passed on the command line for reproduce-on-failure.

## Mapping to stack choices

| Stack | Editor path | Risk to Principle IV |
| --- | --- | --- |
| Bevy + shared crates + custom editor later | Medium effort; max control of IE UI + tools | Low if contracts above held |
| Fyrox | Faster visual editor; adapt IE HUD in Fyrox UI | Medium — ensure rules stay outside scene scripts |
| Godot + Rust sim | Fastest general editor | Medium/High — dual world (Godot scenes vs Rust) needs discipline |
