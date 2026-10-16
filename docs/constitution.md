# Genesis constitution

Core design principles for the Pathfinder 2e CRPG engine and editor (Rust).

## I. Classic PF2e CRPG

- Product is a classic CRPG: isometric world view, Infinity Engine look, party portraits, action bar, modal windows.
- UI MUST NOT use a mobile or MMO HUD as the primary look.
- Character, exploration, and combat use Pathfinder Second Edition (PF2e).
- MUST NOT be an action RPG.
- MUST NOT be a rules-light game only inspired by PF2e.

## II. Turn-Based Single-Player Play

- Combat MUST be turn-based. MUST NOT implement real-time-with-pause (RTwP).
- Exploration uses real-time movement; encounters switch to turn-based initiative and the PF2e action economy.
- Single-player only. MUST NOT implement multiplayer, co-op, networked sessions, or session sharing as a play mode.
- The engine is balanced for a fixed **4 player-character party**. There are always 4 party members (slots remain even if some are dead). The engine MUST NOT scale encounters or apply difficulty adjustments based on current living party size.

## III. Tabletop Fidelity First

- Follow PF2e tabletop rules and procedures. Change a tabletop rule only when it cannot operate in a video game.
- Each such change MUST record: the tabletop rule and source; why it cannot fit a video game; the substitute behavior; how play still matches tabletop intent.
- Invalid reasons to change a rule: convenience, balance preference, “video games usually do this.”

## IV. Editor and Game Are One Product

- Shipped product includes game runtime and content editor; they share the same engine and rules implementation.
- MUST NOT keep a separate editor ruleset or ship a game-only fork that cannot author content.
- Content authors MUST use the same simulation players use.
- Required authoring steps MUST NOT depend on an external editor not shipped with the product (except general-purpose tools: text editor, image editor, VCS).

## V. Engine and Tooling Scope

- Repository contains engine, editor, and authoring tooling.
- MUST NOT treat campaign narrative, adventure scripts, or setting lore as in-scope work.
- Features MUST serve the engine, editor, or shared tooling. Campaign content is not the purpose of this repository.
- MVP/test content and generated basic game data are fixtures to exercise the product — not campaign deliverables.

## VI. Agent Package Documentation

- Every package MUST have `docs/<package-path>/README.md` mirroring the package directory relative to repo root (e.g. `internal/rules` → `docs/internal/rules/README.md`).
- These files are brief agent context beyond godoc. MUST NOT duplicate the full godoc surface.
- A package without its matching docs file MUST NOT land.

## VII. Comments Describe Current Code

- Comments describe what the code is now. MUST NOT narrate history, design debates, rejected alternatives, or rationale that belongs in research/spec.
- MUST NOT cite repo history (commits, PRs, issues, prior names, “added in” / “formerly”) or specific specs/plans/tasks (e.g. paths under `specs/`).
- Applies to package comments, exported godoc, and inline comments.
- Exceptions: tabletop source citations in tests; Principle III fidelity records live in the constitution or research, not code comments.

## VIII. Tester Guide

- Repository MUST maintain root `TESTER.md` for Grok Bot (AI tester on a VM, nightly cadence).
- Guide is a specific manual walkthrough of how to use and test the software on that VM.
- Specs that change UX or add features MUST update `TESTER.md` in the same implementation and pass the walkthrough for changed/new behavior; MUST NOT land without that pass or while `TESTER.md` is missing/omitting the change.
- Changes that alter build/run/test procedures MUST update `TESTER.md` in the same change. Otherwise confirm the guide remains accurate.
