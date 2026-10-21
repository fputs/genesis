# Design

## Context

`.github/workflows/ci.yml` is one Cargo job on push and pull_request to `main`. Active changes live in `openspec/changes/<id>/`; archived changes live in `openspec/changes/archive/`, either as `<id>` or `YYYY-MM-DD-<id>` (see `2026-10-21-add-project-foundation`). See proposal.md for why this gate exists. Normative behavior is `specs/spec-drift-check/spec.md`.

Phase 1 order in `docs/architect/decisions.md` places `dice-and-checks` immediately after the archived foundation change, then the rules changes (modifiers next among those). This gate is inserted immediately after `add-dice-and-checks` and does not reorder those rules changes. Apply records that insertion; this design does not edit the decision log.

The 2026-10-20 decision log describes a separate archive pull request after implementation. This gate replaces that. The implementation pull request's last commit is `openspec archive <id> --yes`, so the merged tree has no fully checked active change and `main` stays green. Push to `main` repeats the check only as a backstop. Genesis DevOps agent implements this change.

## Goals / Non-Goals

**Goals:**

- One local script, one new job, three checks, deterministic results from the checkout.
- Fail a pull request whose result still has a fully checked active change, so that tree never merges.
- Keep `main` green. Push to `main` repeats the completed-change check only as a backstop.
- The implementation pull request passes after its last commit, `openspec archive <id> --yes`.
- One reference mechanism for code-path pull requests.

**Non-Goals:**

- Editing the Cargo job, its toolchain, or its cache.
- Calling the GitHub API or any other network service from the check.
- Parsing branch names, commit messages, or review comments.
- Treating `scripts/`, `docs/`, `.cursor/`, or `openspec/` as code paths.
- Judging whether checked boxes match the work. The check reads checkbox syntax only.
- Running the job for branches other than `main`.

## Decisions

### 1. Reference mechanism is a pull request body line

Chosen line, after trim: `OpenSpec-Change: <change-id>`.

`<change-id>` is lowercase letters and digits in hyphen-separated groups (`add-dice-and-checks`). One id per line. Several ids are several lines. Duplicates count once. A line with extra words is not a reference. Fenced lines are ordinary lines.

The id exists when `openspec/changes/<id>/` is active, or a direct child of `openspec/changes/archive/` is named `<id>` or `YYYY-MM-DD-<id>` with the entire remainder equal to the id. `foundation` does not match `2026-10-21-add-project-foundation`.

Alternative considered: require the branch name `openspec/<change-id>`. Archive pull requests already use other prefixes (`chore/archive-foundation-and-designer-docs`), and one branch name cannot carry two ids. Body lines work for every pull request the workflow already uses.

### 2. Code paths are a closed list

A changed path is a code path only when it is root `Cargo.toml`, root `Cargo.lock`, root `rust-toolchain.toml`, or starts with `crates/` or `.github/workflows/`. Diff inputs are additions, modifications, deletions, and both paths of a rename, between the pull request base commit and the checked-out commit.

`crates/rules/Cargo.toml` is a code path because it is under `crates/`. `docs/Cargo.toml` is not. `crates-extra/foo.rs` is not.

When the diff has no code path, the reference check passes and does not read the body. A docs-only pull request and an openspec-only pull request are exempt, including when the body names an unknown id. A diff that also touches a code path is not exempt.

Alternative considered: require a reference on every pull request. Spec-only pull requests would then need a trailer for no engine change. An implementation pull request that archives in its last commit still needs the trailer when it edits a code path. The exemption matches the paths named in the request.

### 3. The reference check runs only on pull_request

Push to `main` has no pull request body in the local event. Looking up the merged pull request would call the GitHub API. The reference check is skipped on push and counts as passed. Pull request bodies come from `github.event.pull_request.body` in `GITHUB_EVENT_PATH` (null body is empty).

### 4. Completed-change check is the same tree predicate

Fully checked and active are defined in the spec. Tracked boxes are hyphen task items. A box is checked when its brackets hold only `x` or `X`, with any spacing, which is how apply treats `- [ x]`. `archive` is not an active change.

The predicate does not look at the base commit. It fails when the tree under test has any fully checked active change.

- **pull_request to `main`:** the tree under test is the checked-out result after the pull request's changes (the default merge commit). Earlier commits on the branch may have every box checked. The head must not. The implementation pull request's last commit is `openspec archive <id> --yes`, which moves the change under `openspec/changes/archive/`. There is no separate archive pull request.
- **push to `main`:** the tree under test is the pushed commit. The check is the same predicate, run only as a backstop. The designed merge already passed on the pull request, so this push stays green and `main` does not go red.

`pull_request.base.sha` is still required for the code-path diff. It is not an input to the completed-change check.

### 5. One Node script, pinned CLI, no package

Add `scripts/spec-drift-check.mjs` with no npm dependencies. The job checks out with `fetch-depth: 0`, sets up Node 22, installs `@fission-ai/openspec@1.14.3` so `openspec` is on `PATH`, runs `node --test scripts/spec-drift-check.test.mjs`, then runs `node scripts/spec-drift-check.mjs`.

The script spawns `openspec validate --all --strict` in the repo root. It always runs validation, the completed-change check, and the reference check. It prints every failure line and exits 1 if any check failed:

- `spec-drift: validate failed`
- `spec-drift: completed unarchived change <id>`
- `spec-drift: code change missing OpenSpec-Change`
- `spec-drift: unknown change <id>`

Validation output is forwarded to the log. Tests may set `SPEC_DRIFT_EVENT` (`push` or `pull_request`), `SPEC_DRIFT_ROOT`, `SPEC_DRIFT_BASE_SHA`, and `SPEC_DRIFT_BODY`. When those are unset, the script uses `GITHUB_EVENT_NAME`, the repo root, and `GITHUB_EVENT_PATH`.

Alternative considered: a bash script. The job already installs Node for the CLI, and `node:test` plus `JSON.parse` on the event file avoids a second parser. Bash remains a viable shape; the behavior in the spec does not depend on the language. Node is the choice because OpenSpec is a Node CLI and the test runner is built in.

### 6. Tests are a table of fixture repositories

`scripts/spec-drift-check.test.mjs` uses `node:test`. Each table row builds a temporary git repository: base commit, head commit, optional pull request body, and a stub `openspec` executable prepended to `PATH` that exits 0 or 1 and prints a marker. The test runs the same `node scripts/spec-drift-check.mjs` entry point as CI and asserts the exit status and failure lines.

The table includes every pass and fail scenario of the validation, completed-change, and reference checks in the spec, including docs-only, openspec-only, several ids, a pull request whose result is still fully checked, the implementation pull request whose last commit is `openspec archive <id> --yes`, and the push of that green result. A push of a fully checked active change is the backstop row. One row reads `.github/workflows/ci.yml` and asserts the `@fission-ai/openspec@1.14.3` pin and Node 22.

### 7. Phase 1 note lands at apply time

When this change is applied, append dated decisions to `docs/architect/decisions.md`. One says `spec-drift-check` slots immediately after `add-dice-and-checks`. Another says an implementation pull request's last commit is `openspec archive <id> --yes`, there is no separate archive pull request, and this gate leaves `main` green. Leave the relative order of the rules changes after `add-dice-and-checks` as it stands. Do not rewrite the 2026-10-20 lists in place. Genesis DevOps agent does this apply.

## Risks / Trade-offs

- [Shallow checkout cannot see the base commit] → `fetch-depth: 0` on this job only. The Cargo job stays as it is.
- [A direct push could land a fully checked active change] → Push to `main` runs the same completed-change check as a backstop. The designed path never merges that tree, because the pull request fails first. `main` stays green.
- [Unchecking a box makes a completed change look open] → Accepted. The check is the checkbox state on the tree under test.
- [A pull request that only edits `scripts/` needs no change id] → Accepted. The code-path list is closed. This change's implementation pull request also edits `.github/workflows/ci.yml`, so it still needs `OpenSpec-Change: add-spec-drift-check`.
- [Fenced examples in the body count as references] → Accepted. Matching is line-based so it stays deterministic.
- [CLI pin drifts from `@latest`] → The pin is `1.14.3`, the version resolved while this change was specified. Bumping it is a later change.

## Migration Plan

1. Merge this spec pull request. It touches only `openspec/changes/add-spec-drift-check/`, so once the job exists it would be openspec-only and exempt.
2. After `add-dice-and-checks`, Genesis DevOps agent applies this change on its own pull request. That pull request edits `.github/workflows/**`, so its body includes `OpenSpec-Change: add-spec-drift-check`. Its last commit is `openspec archive add-spec-drift-check --yes`. The checked-out result has the change under `openspec/changes/archive/`, so the completed-change check passes.
3. The push of that merge to `main` runs the same check and passes. `main` stays green. There is no follow-up archive pull request.
4. Rollback is removing the `spec-drift` job and the script. No data migration.
