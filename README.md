# Genesis

Genesis is a game built on a rules engine, with a basic map editor planned for Phase 2. Phase 1 is the rules crate, `crates/rules`. The books those mechanics are derived from are named in the Attribution section of [`NOTICE-ORC.md`](NOTICE-ORC.md).

## Status

This is Genesis v2. v1 history lives on the `v1-reference` branch.

The Cargo workspace's only member is `crates/rules`. These main specifications describe that crate, and `CHANGELOG.md` records the same work under `## [Unreleased]`:

- **Dice and checks** (`openspec/specs/dice-and-checks`). Caller-injected d20 checks, flat checks, and the untyped multiple attack penalty. The changelog records injected dice, degree of success, flat checks, the multiple attack penalty, and a character build that derives a creature from its validated base. Folding a later contribution returns an error when the list is empty, the damage die cannot be stored, or a sum overflows.
- **Creature template** (`openspec/specs/creature-template`). A character is built by composition: immutable inputs derive a read-only creature, and combat keeps its own state.
- **Modifiers** (`openspec/specs/modifiers`). Circumstance, item, status, and untyped modifiers fold onto attack rolls, Armor Class, Perception, and damage. An untyped bonus or a zero value is rejected. The changelog records stacking for circumstance, item, status, and untyped penalties, and a damage minimum of 1 before critical doubling. The specification says Phase 1 produces no item modifier.
- **Grid movement** (`openspec/specs/grid-movement`). Stride, Step, reach, and range are measured on a 5-foot grid for Small and Medium creatures, including diagonal cost, difficult terrain, and occupied squares. The changelog records 5-foot grid movement for Stride and Step, diagonal cost across a turn, and range-increment data.

Process specifications in `openspec/specs/` are `project-foundation`, `coding-constitution`, `changelog`, `spec-drift-check`, and `pr-hygiene`. The changes that added them are archived. The changelog records the pull-request hygiene job, the spec-drift CI check, and the fail-closed fixes for those checks.

`openspec/changes/amend-modifier-duplicates` is the active change. Its task checkboxes are open. The proposal amends duplicate-effect identity and the equal-sequence penalty tie, and it describes that pull request as the spec and the designer notes. `docs/designer/rules-deviations.md` lists the known penalty-tie defect and names that change.

## Roadmap

`docs/qa/test-plan.md` sets Phase 1 as the Remaster rules crate (`crates/rules`) and Phase 2 as a basic map editor. `AGENTS.md` places game and editor crates in Phase 2. The workspace member list is exactly `crates/rules`.

`docs/architect/decisions.md` (2026-10-20) records this Phase 1 order: project-foundation; dice-and-checks, including flat checks and the untyped multiple attack penalty, including agile; creature-template; grid-movement; encounter-core; circumstance-modifiers; then shields, reactive strike, positioning and flanking, seek and hidden, delay, ready, and aid. A revision the same day moves modifiers (circumstance, status, and untyped stacking) to just after dice-and-checks.

Archived changes under `openspec/changes/archive/` are:

- `add-project-foundation`
- `add-dice-and-checks`
- `add-modifiers`
- `add-creature-template`
- `add-grid-movement`
- `add-changelog`
- `add-coding-constitution`
- `add-spec-drift-check`
- `add-pr-hygiene-checks`
- `harden-pr-hygiene`

Encounter-core, shields, reactive strike, positioning and flanking, seek and hidden, delay, ready, and aid are named in that architect note and in `docs/designer/pf2e-mvp-scope.md`. They have no directory under `openspec/changes/` and no specification under `openspec/specs/`. The designer scope says it is a FINAL draft and not an OpenSpec artifact, and that it feeds proposals. Its Phase 1 goal is a spec-first, deterministic, seeded, headless core: four PCs against fixture enemies on a 5-foot grid, using baked stat blocks.

`docs/architect/decisions.md` defers attributes, proficiency, saves, skills, spells, classes, and broader conditions. The longer deferral list is the Deferred table in `docs/designer/pf2e-mvp-scope.md`.

## Build and test

`rust-toolchain.toml` pins channel `1.99.1` with the `clippy` and `rustfmt` components.

These are the proof commands from `AGENTS.md`, copied as written there:

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace
npx @fission-ai/openspec@1.14.3 validate --all --strict
```

`.github/workflows/ci.yml` runs on push and pull request to `main`. Its Cargo job runs `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, and `cargo test --workspace`. Its `spec-drift` job installs `@fission-ai/openspec@1.14.3`, runs `node --test scripts/spec-drift-check.test.mjs`, and runs `scripts/spec-drift-check.mjs`. `.github/workflows/pr-hygiene.yml` runs `node --test scripts/pr-hygiene.test.mjs` and then `scripts/pr-hygiene.mjs` on pull requests to `main` (`opened`, `synchronize`, `reopened`, and `edited`). `.github/workflows/spec-drift-edited.yml` runs the spec-drift job again on pull_request `edited` to `main`.

## Repository layout

- `openspec/specs/` holds the current specifications. `openspec/changes/` holds active changes. `openspec/changes/archive/` holds archived changes.
- `docs/` holds design docs and decisions, one folder per role. The role folders are `docs/architect/`, `docs/designer/`, `docs/visual/`, `docs/qa/`, `docs/producer/`, and `docs/audio/`. `docs/README.md` indexes them.
- `crates/rules/` is the rules crate and the only workspace member.

`AGENTS.md` is the coding constitution. `CHANGELOG.md` is the changelog. `scripts/` holds the pull-request hygiene and spec-drift checks. `.github/workflows/` holds continuous integration.

## How changes flow

Work is OpenSpec-first.

A spec is proposed as its own pull request: proposal, design, tasks, and spec deltas. Evan reviews and approves spec pull requests before implementation. He reviews specs only.

An implementation pull request follows that approved spec. The pull request body includes a line of the form `OpenSpec-Change: <id>`. The last commit of that pull request archives the change:

```
openspec archive <id> --yes
```

Archiving moves the change under `openspec/changes/archive/` and updates `openspec/specs/`. One change is one procedure family and one pull request. A deviation from the spec stops the work and goes back to Evan as a spec amendment.

## License

Code in this repository is under the MIT License. See [`LICENSE`](LICENSE). Copyright (c) 2026 Evan Burkey.

The mechanics derived from the books listed in the Attribution section of [`NOTICE-ORC.md`](NOTICE-ORC.md), and any new mechanics this product adds, are Licensed Material under the ORC License.

Contributors follow the licensing requirements in [`docs/architect/licensing.md`](docs/architect/licensing.md).
