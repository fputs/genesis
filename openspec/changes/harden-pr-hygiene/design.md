# Design

## Context

`scripts/pr-hygiene.mjs` and `scripts/spec-drift-check.mjs` already implement the specs on main. See proposal.md for why those specs leave the QA holes open. Normative behavior is the delta specs. AGENTS.md was pre-flighted: the archive subject this change requires is already `openspec archive <id> --yes`, and no constitution line has to change.

## Goals / Non-Goals

**Goals:**

- Fail closed on the QA repros named in the delta scenarios.
- Keep each check a straight comparison: path set, subject string, bullet text, fence toggle, or `LC_ALL`.

**Non-Goals:**

- No general commit-message parser. Do not scan commit bodies, trailers, footers, or conventional-commit prefixes. The archive check compares the subject to `openspec archive <id> --yes`, or to those commands joined by ` && ` in ascending id order.
- No case-folding of an `OpenSpec-Change` token onto a differently cased directory.
- No new npm dependency, and no new archive-last failure line. The `edited` re-run is a separate spec-drift-only workflow.
- No opt-out label, and no check that a bullet uses a given subsection or links the pull request. The changelog spec still requires both of authors.
- No change to Cargo steps, no change to `.github/workflows/ci.yml` triggers, and no push trigger for `pr-hygiene`.
- Do not add `scripts/` to the spec-drift code-path list, and do not exempt it from the changelog bullet.

## Decisions

### 1. `edited` is a separate spec-drift workflow

A skipped required check counts as passing. An `if` that skips the Cargo `ci` job on `edited` would turn a red `ci` check green when someone edits the body. That is why the earlier rejection of a second workflow is reversed: one workflow with a job `if` does not keep the failed Cargo result.

`.github/workflows/ci.yml` keeps its current triggers (push and pull_request to `main`, no `edited` type) and no skip `if`. `.github/workflows/spec-drift-edited.yml` runs on pull_request `edited` targeting `main`. It sets `permissions: contents: read`. Its concurrency group is `spec-drift-edited-${{ github.event.pull_request.number }}` with `cancel-in-progress: true`, so an older passing run cannot become the latest result. Its only job is named exactly `spec-drift` and runs only `scripts/spec-drift-check.mjs`.

Reusing the name `spec-drift` is safe. The Cargo `ci` job is never skipped, and the edit run re-runs the same script on the same commit. The required check named `spec-drift` sees that edit run, and a failed Cargo check stays failed.

Alternative considered: skip Cargo with `if` inside `ci.yml`. Rejected. GitHub treats a skipped check as success, so a body edit would replace a failed Cargo check.

### 2. A bad token fails on any pull request

`OpenSpec-Change:` plus one token is a reference on every pull request, including docs-only and openspec-only. The id grammar stays `^[a-z0-9]+(?:-[a-z0-9]+)*$`. An empty `OpenSpec-Change:` line and a spaceless `OpenSpec-Change:Add` line are invalid and fail the same way. Anything else logs `spec-drift: invalid change id <token>` and does not also log `spec-drift: code change missing OpenSpec-Change`. Do not lowercase the token and look it up. Extra words stay "not a reference". A missing line, or a valid kebab-case id that names no change, still passes when the diff has no code path. Push still skips the reference check.

Alternative considered: case-insensitive match against `add-foo` for `Add-Foo`, and ignoring a bad token on docs-only diffs. Rejected. Fail closed: the directory name is the id, and an invalid token is a mistake on every pull request.

### 3. Archive-last is the head commit's move

Ignore the pull request body. For each direct child of `openspec/changes/` other than `archive` that is a tree on the merge-base and gone on the head:

- The head tree has `openspec/changes/archive/<YYYY>-<MM>-<DD>-<id>/`, where the remainder after the date equals `<id>`. A directory already on the merge-base counts. A missing one fails.
- The head commit, compared with its first parent, deletes `openspec/changes/<id>/`.
- When that dated directory is not on the merge-base, the same head commit adds it. An earlier commit in the pull request does not.
- `git log -1 --format=%s`, trimmed, is `openspec archive <id> --yes` for every removed id, joined by ` && ` in ascending id order. Do not read the commit body. One id needs no join.
- Every other path in that commit is under `openspec/changes/archive/`, under `openspec/specs/`, or under the removed directory.

An empty commit and a specs-only head commit fail because they do not do that delete and add. One failure line: `archive commit is not last`. Several removed ids pass only when the subject is their exact commands joined by ` && ` in ascending id order. A command on a later line does not count. Splitting the subject on that literal is the whole parser.

### 4. Bullet identity is text plus count, outside fences

Bullet text is the trimmed list item with the marker removed. It is new only when that text is not a bullet in any unfenced section of the base `CHANGELOG.md` and the head Unreleased section has more unfenced bullets than the base. A fence is a trimmed line of three or more backticks and toggles. Tilde fences are out of scope.

### 5. Exempt paths are one set

A path is exempt when it is under `docs/`, `openspec/`, or `.cursor/`, or is exactly `AGENTS.md`, `CHANGELOG.md`, `README.md`, `Cargo.lock`, or `rust-toolchain.toml`. The bullet check passes with no new bullet when every changed path is in that set. `Cargo.lock` plus `docs/` is exempt. `Cargo.lock` plus `crates/` is not. `scripts/` is not in this set.

### 6. `scripts/` needs a bullet and is not a code path

A pull request that only changes paths under `scripts/` is not a spec-drift code path, so a missing `OpenSpec-Change` line does not fail spec-drift. The same pull request changes how checks behave, so pr-hygiene fails it unless a new Unreleased bullet was added. Do not add `scripts/` to the code-path list.

### 7. Git children get `LC_ALL=C`

Pass `LC_ALL=C` on the environment of every `git` spawn in both scripts. Keep matching `does not exist` and `exists on disk, but not in`. The fixture uses a stand-in `git` that prints a non-English missing line unless that variable is `C`.

## Risks / Trade-offs

- [GitHub fires `edited` for title and base edits too] → the spec-drift-only workflow re-runs. `ci.yml` does not run, so a failed Cargo check is not skipped and stays failed.
- [Swapping a stale Unreleased bullet for a new one leaves the count unchanged] → the bullet check fails. Fail-closed, accepted.
- [GitHub "Update branch" after the archive commit puts a merge commit at head] → archive-last fails, because that merge is not the archive move. The remedy is to re-archive so the archive commit is last again.
- [A cleanup that deletes an active directory whose archive is already on main must use the archive subject] → one subject rule for every removal. The pre-existing-archive scenario passes when the head commit is that deletion.
- [A wording change that also adds a bullet still passes, because the count went up and the new text is absent] → an edit alone does not.

## Migration Plan

Genesis DevOps agent implements the two scripts, their fixtures, and `.github/workflows/spec-drift-edited.yml`. `.github/workflows/ci.yml` triggers stay unchanged. No `AGENTS.md` edit. Rollback is a revert of the implementation commits before the archive commit. If "Update branch" adds a merge after the archive commit, re-archive.

## Open Questions

None.
