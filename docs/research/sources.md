# Sources

Bibliography of materials consulted for this corpus. Prefer these over secondary SEO posts when re-checking facts. Research snapshot: **2026-10-16**.

## Project context (internal)

| Source | Why relevant |
| --- | --- |
| [docs/constitution.md](../constitution.md) | Binding product principles |
| [docs/mvp-slice-1.md](../mvp-slice-1.md) | First vertical scope |
| [docs/project-context.md](../project-context.md) | Repo identity and hosting notes |

## Rust engines and ecosystem

| Source | Why relevant |
| --- | --- |
| https://arewegameyet.rs/ecosystem/engines/ | Version/activity snapshot for Bevy, Fyrox, macroquad, ggez, godot, etc. |
| https://bevy.org/news/bevy-0-20/ | Bevy 0.20 official notes (2026-10-08): Feathers, BSN, roadmap hints |
| https://bevy.org/learn/migration-guides/0-19-to-0-20/ | Churn/breaking-change evidence |
| https://bevy.org/news/bevy-0-16/ | ECS relationships; earlier milestone context |
| https://bevy.org/news/bevy-0-19/ | BSN landing; Feathers/BSN direction |
| https://docs.rs/bevy/0.16/bevy/ecs/index.html | Bevy ECS design statement (usable standalone) |
| https://fyrox.rs/blog/post/fyrox-game-engine-1-0-0/ | Fyrox 1.0 stable announcement + export CLI / loading |
| https://fyrox-book.github.io/beginning/editor_overview.html | FyroxEd capabilities, Play Mode process isolation |
| https://fyrox-book.github.io/scripting/script.html | Script registration / shared runtime model |
| https://fyrox-book.github.io/scene/graph.html | Scene graph model |
| https://godot-rust.github.io/ | godot-rust project overview |
| https://github.com/godot-rust/gdext | GDExtension Rust bindings status |
| https://crates.io/crates/godot | godot 0.5.5 release metadata |
| https://bevyengine.github.io/bevy_editor_prototypes/roadmap.html | First-party Bevy editor stages (not shipped) |
| https://github.com/bevyengine/bevy/issues/23637 | BSN editor infrastructure gaps (write-back, etc.) |

### Secondary engine comparisons (used cautiously)

| Source | Why relevant |
| --- | --- |
| https://indietooling.com/posts/fyrox-vs-bevy-rust-engines | Bevy vs Fyrox narrative (verify versions against primary) |
| https://www.pistack.xyz/posts/2026-09-11-rust-game-engines-bevy-fyrox-macroquad/ | 2026 comparison table (secondary) |

## CRPG / IE precedents

| Source | Why relevant |
| --- | --- |
| https://github.com/gemrb/gemrb | Open IE reimplementation |
| https://gemrb.github.io/ | GemRB project goals / release notes |
| https://gibberlings3.github.io/iesdp/main.htm | IE formats, effects, scripting docs |
| https://gibberlings3.github.io/iesdp/file_formats/index.htm | Format index (ARE/CRE/ITM/SPL/…) |
| https://github.com/NearInfinityBrowser/NearInfinity | External IE editor pattern (what not to require) |

## Turn-based / mode switch

| Source | Why relevant |
| --- | --- |
| https://baldursgate3.game/news/a-little-about-combat-stealth_3 | Larian on combat + forced TB in explore |
| https://bg3.wiki/wiki/Turn-based_mode | BG3 mode behavior summary |
| https://wrath.owlcat.games/news/16 | Owlcat TB announcement; RTwP-primary caution |
| https://www.solasta-game.com/solasta-crown-of-the-magister/news/11-directors-log-2-adapting-the-ruleset | Fidelity-first adaptation philosophy |
| https://www.solasta-game.com/solasta-crown-of-the-magister/news/7-dev-diary-1-reaction-system | Reactions as VG adaptation example |
| https://www.solasta-game.com/solasta-crown-of-the-magister/news/16-dev-diary-5-ai-basics | Shared AI across explore/combat |

## Rules simulation / PF2e digital systems

| Source | Why relevant |
| --- | --- |
| https://github.com/foundryvtt/pf2e | Reference implementation of digital PF2e |
| https://www.mintlify.com/foundryvtt/pf2e/rule-elements/overview | Rule Element architecture |
| https://www.mintlify.com/foundryvtt/pf2e/rule-elements/common-rule-elements | Common RE shapes |
| https://github.com/foundryvtt/pf2e/wiki/Quickstart-guide-for-rule-elements | Preparation phases / pitfalls |
| https://github.com/foundryvtt/pf2e/blob/master/src/module/rules/index.ts | RE registry / deserialization |
| https://2e.aonprd.com/ | Archives of Nethys reference site |
| https://docs.rs/loom_ruleset/latest/loom_ruleset/ | Deterministic rules AST prior art (not PF2e) |

## Rendering / UI / input crates

| Source | Why relevant |
| --- | --- |
| https://github.com/StarArawn/bevy_ecs_tilemap | Isometric tilemaps for Bevy |
| https://docs.rs/crate/bevy_ecs_tilemap/latest | Crate features + version (0.19.0 observed) |
| https://github.com/StarArawn/bevy_ecs_tilemap/blob/main/examples/iso_diamond.rs | Diamond iso example |
| https://github.com/adrien-bon/bevy_ecs_tiled | Tiled → Bevy loader (iso support claimed) |
| https://docs.rs/bevy_egui/latest/bevy_egui/ | Editor/debug UI integration |
| https://github.com/Leafwing-Studios/leafwing-input-manager | Action-map input |
| https://github.com/jbuehler23/bevy_map_editor | Community Bevy map editor (check Bevy pin; 0.18 noted in README) |

## Notes on source quality

- Official Bevy/Fyrox/godot-rust docs and repos outrank blog comparisons.
- Foundry pf2e is architecture reference, not a licensed content pack for Genesis.
- Owlcat/Larian/Solasta materials are design precedents; stacks (Unity/Divinity engine/custom) are not proposed for Genesis.
- Where community crate versions disagree with engine latest, **re-check crates.io on implementation day**.
