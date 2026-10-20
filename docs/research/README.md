# Genesis research corpus

Research notes informing engine/stack choices for **Genesis**: a Rust PF2e classic CRPG engine + in-product editor (isometric / Infinity Engine–style, turn-based combat, tabletop fidelity).

This folder is **research only**. It does not implement product code.

## How to read

1. Start with [stack-recommendation.md](./stack-recommendation.md) for the decision and tradeoffs.
2. Use the topic notes below when you need evidence, alternatives, or open questions.
3. [sources.md](./sources.md) is the bibliography (prefer primary docs/repos over SEO blogs).

Constitution and MVP framing (outside this folder):

- [../README.md](../README.md) — docs index
- [../constitution.md](../constitution.md)
- [../mvp-slice-1.md](../mvp-slice-1.md)
- [../project-context.md](../project-context.md)
- [../rules-corpus/README.md](../rules-corpus/README.md) — classes, archetypes, spells, substrate roadmap (implementation-shaped; not a copy of this research tree)

## Corpus index

| Note | Topic |
| --- | --- |
| [engines-rust.md](./engines-rust.md) | Rust engines/frameworks: Bevy, Fyrox, lighter 2D libs, godot-rust |
| [crpg-precedents.md](./crpg-precedents.md) | IE-like open engines, editors, and product lessons |
| [turn-based-combat.md](./turn-based-combat.md) | Explore → combat mode switch; TB action economy patterns |
| [rules-simulation.md](./rules-simulation.md) | Encoding PF2e-like rules: data vs logic, fidelity testing |
| [editor-runtime.md](./editor-runtime.md) | Shipping editor with the same sim/runtime |
| [rendering-ui.md](./rendering-ui.md) | Isometric rendering, IE-like UI, input |
| [sources.md](./sources.md) | URLs/docs/repos consulted |
| [stack-recommendation.md](./stack-recommendation.md) | Primary stack + alternatives mapped to principles/MVP |

## Method notes

- Prefer official release notes, docs, and repositories.
- Version-sensitive claims include dates/versions where known (research snapshot: **2026-10-16**).
- Unknowns are marked; capabilities are not invented.
- Ecosystem crates often lag Bevy majors by one release — pin and verify before coding.

## Related product constraints (summary)

- Rust; isometric IE look (not mobile/MMO HUD).
- Turn-based combat only (no RTwP); explore real-time, then switch to TB.
- Single-player; fixed 4-PC party; tabletop PF2e fidelity.
- Engine + editor share one sim (editor deferred for MVP slice 1, but stack must not paint into a corner).
- Repo is engine/tooling, not campaign content; MVP loads fixture encounter data.
