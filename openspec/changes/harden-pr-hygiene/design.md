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
- No new npm dependency, no new job, and no new archive-last failure line.
- No opt-out label, and no check that a bullet uses a given subsection or links the pull request. The changelog spec still requires both of authors.
- No change to Cargo steps, and no push trigger for `pr-hygiene`.
- Do not treat `scripts/` as a code path.

## Decisions

### 1. spec-drift listens for `edited`; Cargo does not

`.github/workflows/ci.yml` lists pull_request types `opened`, `synchronize`, `reopened`, and `edited`. The Cargo job gets `if: github.event_name != 'pull_request' || github.event.action != 'edited'`. `spec-drift` has no such `if`. An `edited` payload still carries the pull request body, which is the input the reference check already uses.

Alternative considered: a second workflow that only runs spec-drift on `edited`. That duplicates the job. One workflow with a job `if` keeps a single spec-drift definition.

### 2. A bad token is a failed reference

`OpenSpec-Change:` plus one token is always a reference on a code-path pull request. The id grammar stays `^[a-z0-9]+(?:-[a-z0-9]+)*$`. Anything else logs `spec-drift: invalid change id <token>` and does not also log `spec-drift: code change missing OpenSpec-Change`. Do not lowercase the token and look it up. Extra words stay "not a reference", as they are today.

Alternative considered: case-insensitive match against `add-foo` for `Add-Foo`. Rejected. The directory name is the id, and a case-folded hit would accept a line that is not that name.

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

A path is exempt when it is under `docs/`, `openspec/`, or `.cursor/`, or is exactly `AGENTS.md`, `CHANGELOG.md`, `README.md`, `Cargo.lock`, or `rust-toolchain.toml`. The bullet check passes with no new bullet when every changed path is in that set. `Cargo.lock` plus `docs/` is exempt. `Cargo.lock` plus `crates/` is not.

### 6. Git children get `LC_ALL=C`

Pass `LC_ALL=C` on the environment of every `git` spawn in both scripts. Keep matching `does not exist` and `exists on disk, but not in`. The fixture uses a stand-in `git` that prints a non-English missing line unless that variable is `C`.

## Risks / Trade-offs

- [GitHub fires `edited` for title and base edits too] → spec-drift re-runs; the Cargo job stays skipped, so the extra run is the Node script only.
- [A cleanup that deletes an active directory whose archive is already on main must use the archive subject] → one subject rule for every removal. The pre-existing-archive scenario passes when the head commit is that deletion.
- [Two changes cannot be archived by one subject] → the pull request fails closed. Split them.
- [A wording change that also adds a bullet still passes, because the count went up and the new text is absent] → an edit alone does not.
- [The scenario "A removal without a reference skips archive-last" keeps its old name] → a modified requirement cannot drop a scenario. Its outcome is now failure, same as "A removal without a reference fails".

## Migration Plan

Genesis DevOps agent implements the two scripts, their fixtures, and the `ci.yml` trigger. No `AGENTS.md` edit. spec-drift is already the required check; listing `edited` makes that same check run on body edits. Rollback is a revert of the implementation commits before the archive commit.

## Open Questions

None.
