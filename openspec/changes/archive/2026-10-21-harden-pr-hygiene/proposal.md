# Proposal

## Why

QA found holes the current pr-hygiene and spec-drift specs allow. A pull request can go green, then drop its `OpenSpec-Change` line, skip a non-kebab id, or call an empty or specs-only commit the archive. The changelog bullet check treats a moved, edited, or fenced line as new, and it does not combine exemption categories. The specs should fail closed on those cases. DevOps implements the checks.

## What Changes

- **BREAKING** for pull requests that relied on the holes below. The gates get stricter. No new package, and no new failure line for archive-last.
- spec-drift runs again on pull_request `edited` in `.github/workflows/spec-drift-edited.yml`. The job name is `spec-drift`, permissions are `contents: read`, and concurrency is `spec-drift-edited-${{ github.event.pull_request.number }}` with cancel-in-progress. A code pull request that drops its `OpenSpec-Change` line after going green fails. The Cargo workflow keeps its existing triggers, so a body edit does not replace a failed Cargo check.
- An `OpenSpec-Change` token that is not lowercase kebab-case fails loudly as `spec-drift: invalid change id <token>` on any pull request, docs-only and openspec-only included. An empty `OpenSpec-Change:` line and a spaceless `OpenSpec-Change:Add` line are invalid and fail the same way. The token is not case-folded and not skipped. A kebab-case id that matches no change still fails as `spec-drift: unknown change <id>` on a code path, and still passes when the diff has no code path.
- A `scripts/`-only pull request is not a spec-drift code path, and it still needs a CHANGELOG bullet.
- Archive-last requires the head commit to move `openspec/changes/<id>/` to `openspec/changes/archive/<YYYY>-<MM>-<DD>-<id>/`. An empty commit, or one that only adds `openspec/specs/` paths, fails. A removal with no archive counterpart fails even when the body has no change line.
- The archive phrase is the subject, exactly `openspec archive <id> --yes`. The same words on a later line do not count. One subject names one id.
- A bullet is new only when its text is not already a bullet anywhere in the base `CHANGELOG.md` and the Unreleased section has more bullets than the base. Edited or moved bullets do not count. Fenced code blocks are ignored while finding `## [Unreleased]` and bullets.
- Exemption categories combine. A diff made only of `docs/`, `openspec/`, `.cursor/`, `AGENTS.md`, `CHANGELOG.md`, `README.md`, `Cargo.lock`, and `rust-toolchain.toml` needs no bullet.
- Every `git` subprocess runs with `LC_ALL=C`.

This pull request is the spec only. It does not change the scripts or the workflows.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `pr-hygiene`: archive-last must be a real move whose subject is the archive command; bullet identity, fences, and combined exemptions fail closed.
- `spec-drift-check`: re-run on `edited` from a separate workflow, and reject a non-kebab change id loudly on any pull request instead of skipping it.
- `changelog`: a new Unreleased bullet is text that was not already in the base file, mixed exempt paths still skip the bullet, and a `scripts/`-only pull request still adds one.

## Impact

- DevOps edits `scripts/pr-hygiene.mjs`, `scripts/pr-hygiene.test.mjs`, `scripts/spec-drift-check.mjs`, `scripts/spec-drift-check.test.mjs`, and adds `.github/workflows/spec-drift-edited.yml`. `.github/workflows/ci.yml` triggers stay as they are.
- No new dependency. `crates/rules` is untouched. `AGENTS.md` needs no amendment: the required archive subject is already `openspec archive <id> --yes`.
- The implementation pull request adds its Unreleased bullet before the archive commit, then runs `openspec archive harden-pr-hygiene --yes`.
