# Proposal

## Why

The public repository needs a written licensing rule set, an ORC notice, and a MIT license on the rules crate. `crates/rules/Cargo.toml` is a code path, so this work is an OpenSpec change. The rules live in `docs/architect/licensing.md` and are not enforced by a new CI job.

## What Changes

- Add `docs/architect/licensing.md` with numbered rules LIC-1 onward, taken from the 2026-10-21 ORC research. Mark unverified items. Add no CI check.
- Keep the `NOTICE-ORC.md` author-line TODO and the generic Reserved Material list, and mark both unverified pending the books' legal pages.
- Link the licensing document from the README license section and from `docs/README.md`.
- Add one rule to `AGENTS.md`: contributors follow the licensing document, and every rules spec or doc cites its source book and page.
- Set `license = "MIT"` on `crates/rules` and add the MIT `LICENSE`.
- Add an Unreleased changelog bullet that links to the pull request.

## Capabilities

### New Capabilities

- `licensing`: The repository records MIT for source code and ORC for rules mechanics, and points contributors at `docs/architect/licensing.md`.

### Modified Capabilities

None.

## Impact

`docs/architect/licensing.md`, `docs/README.md`, `NOTICE-ORC.md`, `README.md`, `AGENTS.md`, `LICENSE`, `crates/rules/Cargo.toml`, and `CHANGELOG.md`. No new workflow and no new script. `crates/rules` gains no dependency.
