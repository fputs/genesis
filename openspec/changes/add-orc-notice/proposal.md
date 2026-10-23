# Proposal

## Why

Genesis needs an ORC License Art. III notice that a reader can find in the repository. The notice has to name the license, state that new Genesis game mechanics are Licensed Material, list this product's Reserved Material, and carry the Player Core, GM Core, and Monster Core attribution lines in full. The README has to point at that notice and say what happens with player-made campaigns.

## What Changes

- Add the `orc-notice` capability. Its scenarios are the acceptance check for `LICENSES/ORC-NOTICE`, the README section, and the changelog bullet.
- `LICENSES/ORC-NOTICE` carries the four Art. III parts from the ORC research (license reference, attribution, Reserved Material, and Licensed Material), the line `Project Genesis © 2026 Evan Burkey.`, and the player-campaign note.
- Attribution copies the three book lines verbatim from the Archives of Nethys licenses page fetched on 2026-10-23: https://2e.aonprd.com/Licenses.aspx
- `README.md` gains a short section that links to `LICENSES/ORC-NOTICE` and repeats the player-campaign note.
- `CHANGELOG.md` gains one `### Added` bullet under `## [Unreleased]`. `LICENSES/ORC-NOTICE` is not an exempt path under the changelog spec, so this pull request is not docs-only.
- This change does not edit `NOTICE-ORC.md`, `docs/architect/licensing.md`, `crates/`, or `scripts/`. It does not add a lawyer review. It does not archive. The merger archives right before merge.

## Capabilities

### New Capabilities

- `orc-notice`: the repository ORC notice, the README pointer, and the changelog bullet for that notice.

### Modified Capabilities

None.

## Impact

- Later commits on this branch add `LICENSES/ORC-NOTICE`, a section in `README.md`, and one Unreleased bullet in `CHANGELOG.md`.
- No Rust code, no new crate, and no new dependency.
- The notice names the three books only as attribution. It does not add Pathfinder branding, a logo, or a compatibility claim.
- This pull request is one draft. The body includes `OpenSpec-Change: add-orc-notice`.

This change does not add a ruling. The log is [docs/designer/rules-rulings.md](../../../docs/designer/rules-rulings.md).
