# Design

## Context

ORC Art. III conditions the license on four notices: the ORC notice, attribution, Reserved Material, and Expressly Designated Licensed Material. The research checklist (section 8) and the draft in section 1 name a repository file `LICENSES/ORC-NOTICE` and the full Player Core, GM Core, and Monster Core attribution lines. Designer review of pull request 85 makes `LICENSES/ORC-NOTICE` the only notice and removes `NOTICE-ORC.md`.

The attribution lines were taken from https://2e.aonprd.com/Licenses.aspx on 2026-10-23, as the page displayed them, including punctuation and the Player Core author spelling on that page. They are not paraphrased.

## Goals / Non-Goals

**Goals:**

- One notice file, `LICENSES/ORC-NOTICE`, with the Art. III parts, the Licensed Material statement, and the player-campaign note.
- A short README section that links to that file and repeats the note.
- One Unreleased Added bullet, because `LICENSES/` is outside the changelog spec's exempt paths, and one Removed bullet for `NOTICE-ORC.md`.

**Non-Goals:**

- A lawyer review. That happens at release.
- Pathfinder branding, a logo, or a compatibility or endorsement claim. Book titles appear only inside the three attribution lines.
- A lawyer's rewrite of `docs/architect/licensing.md` beyond pointing its notice references at `LICENSES/ORC-NOTICE`.
- An in-game credits screen. Phase 1 has no game client.
- Archiving this change. The merger runs `openspec archive add-orc-notice --yes` as the last commit, right before merge.

## Decisions

### Notice parts, in Art. III order

The file uses these labels:

1. ORC Notice. The Art. III.a sentence, including the Library of Congress identifier TX 9-307-067.
2. Attribution. The three book lines, the credit request `Genesis Licensed Material`, then `Project Genesis © 2026 Evan Burkey.`
3. Reserved Material. The Designer review sentence, including that Licensed Material from Paizo Inc. is not Reserved Material.
4. Expressly Designated Licensed Material. Immediately after Reserved Material: `None. No Reserved Material in this product is designated as Licensed Material.`
5. Licensed Material. All new Genesis game mechanics are ORC Licensed Material.

The player-campaign note follows those parts. It is not another license part. Designer review of pull request 85 set the Reserved Material sentence, the Expressly Designated Licensed Material section, the credit request, and the removal of `NOTICE-ORC.md`.

### Changelog

The changelog spec skips a bullet only when every changed path is under `openspec/`, `docs/`, or `.cursor/`, or is exactly `AGENTS.md`, `CHANGELOG.md`, `README.md`, `Cargo.lock`, or `rust-toolchain.toml`. `LICENSES/ORC-NOTICE` is outside that list, so the pull request adds one `### Added` bullet that links this pull request.

### One notice file

`NOTICE-ORC.md` is removed. `README.md` and `docs/architect/licensing.md` name `LICENSES/ORC-NOTICE`. The three book lines stay byte-identical to the Archives of Nethys page fetched on 2026-10-23, including `Miko Kallio`. The notice does not claim a relationship to Pathfinder beyond those attribution lines.

## Risks / Trade-offs

- The attribution lines match the Archives of Nethys page on the fetch date. A later change to that page does not change this file until a new change copies it again.
- `docs/architect/licensing.md` still marks the physical-book check of the author lines unverified. This change copies the Archives of Nethys lines and does not add a lawyer review.

## Migration Plan

None. Rollback restores `NOTICE-ORC.md` and removes `LICENSES/ORC-NOTICE`.

## Open Questions

None for this change. A lawyer review of the notice is a release step, not a task here.
