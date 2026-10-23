# Design

## Context

ORC Art. III conditions the license on four notices: the ORC notice, attribution, Reserved Material, and a statement of Licensed Material that this product offers. The research checklist (section 8) and the draft in section 1 name a repository file `LICENSES/ORC-NOTICE` and the full Player Core, GM Core, and Monster Core attribution lines. `NOTICE-ORC.md` already exists and still has the short author lines. This change does not edit that file.

The attribution lines were taken from https://2e.aonprd.com/Licenses.aspx on 2026-10-23, as the page displayed them, including punctuation and the Player Core author spelling on that page. They are not paraphrased.

## Goals / Non-Goals

**Goals:**

- One notice file, `LICENSES/ORC-NOTICE`, with four labeled parts and the player-campaign note.
- A short README section that links to that file and repeats the note.
- One Unreleased changelog bullet, because `LICENSES/` is outside the changelog spec's exempt paths.

**Non-Goals:**

- A lawyer review. That happens at release.
- Pathfinder branding, a logo, or a compatibility or endorsement claim. Book titles appear only inside the three attribution lines.
- Editing `NOTICE-ORC.md` or `docs/architect/licensing.md`.
- An in-game credits screen. Phase 1 has no game client.
- Archiving this change. The merger runs `openspec archive add-orc-notice --yes` as the last commit, right before merge.

## Decisions

### Four parts, in Art. III order

The file uses four labels, in the order the research draft uses:

1. ORC Notice. The Art. III.a sentence, including the Library of Congress identifier TX 9-307-067.
2. Attribution. The three book lines, then `Project Genesis © 2026 Evan Burkey.`
3. Reserved Material. The setting, characters, story, art, music, and the title and logo of Project Genesis.
4. Licensed Material. All new Genesis game mechanics are ORC Licensed Material.

The player-campaign note follows those four parts. It is not a fifth license part.

### Changelog

The changelog spec skips a bullet only when every changed path is under `openspec/`, `docs/`, or `.cursor/`, or is exactly `AGENTS.md`, `CHANGELOG.md`, `README.md`, `Cargo.lock`, or `rust-toolchain.toml`. `LICENSES/ORC-NOTICE` is outside that list, so the pull request adds one `### Added` bullet that links this pull request.

### What stays untouched

`NOTICE-ORC.md` stays as it is. This change adds the notice the research checklist names. It does not retitle the product or claim a relationship to Pathfinder beyond the attribution lines.

## Risks / Trade-offs

- The attribution lines match the Archives of Nethys page on the fetch date. A later change to that page does not change this file until a new change copies it again.
- Two notice files will exist until a later change decides what to do with `NOTICE-ORC.md`. This change does not make that decision.

## Migration Plan

None. The file is new. Rollback is deleting it and the README section.

## Open Questions

None for this change. A lawyer review of the notice is a release step, not a task here.
