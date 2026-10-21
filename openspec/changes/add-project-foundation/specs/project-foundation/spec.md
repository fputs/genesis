# project-foundation

## Purpose

The project-foundation capability is the Genesis v2 repository skeleton: the Rust workspace, continuous integration, changelog, agent guide, and role documentation tree.

## ADDED Requirements

### Requirement: Cargo workspace

The repository SHALL contain a Cargo workspace at the repo root whose only member is `crates/rules`.

#### Scenario: Workspace has one member

- **GIVEN** a checkout of the repository
- **WHEN** Cargo reads the root workspace manifest
- **THEN** the workspace members are exactly `crates/rules`

### Requirement: Empty rules library

The `crates/rules` package SHALL be a library crate with no dependencies, and its library source SHALL be empty except for a doc comment.

#### Scenario: Rules crate declares no dependencies

- **GIVEN** the `crates/rules` package manifest
- **WHEN** its dependencies are read
- **THEN** the package declares no dependencies

#### Scenario: Library source is a doc comment only

- **GIVEN** the `crates/rules` library source
- **WHEN** the source is read
- **THEN** it contains a doc comment and no items

### Requirement: Stable toolchain pin

The repository SHALL pin the Rust toolchain in `rust-toolchain.toml` to stable channel `1.99.1`.

#### Scenario: Toolchain file names the current stable

- **GIVEN** `rust-toolchain.toml` at the repo root
- **WHEN** the toolchain channel is read
- **THEN** the channel is `1.99.1`, the current stable release

### Requirement: Continuous integration workflow

The repository SHALL run `.github/workflows/ci.yml` on push and pull_request events targeting `main`.

#### Scenario: Workflow triggers

- **GIVEN** `.github/workflows/ci.yml`
- **WHEN** the workflow triggers are read
- **THEN** it runs on push and pull_request to `main`

### Requirement: CI checks and Cargo cache

The CI workflow SHALL run `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings`, `cargo build --workspace`, and `cargo test --workspace`, and SHALL cache Cargo.

#### Scenario: Required cargo steps

- **GIVEN** a CI run on a pull request to `main`
- **WHEN** the workflow executes
- **THEN** it runs fmt check, clippy with warnings denied, workspace build, and workspace test
- **AND** it restores and saves a Cargo cache

### Requirement: Changelog

The repository SHALL contain a root `CHANGELOG.md` in Keep a Changelog format with an Unreleased section that notes the v2 restart and that v1 is preserved on branch `v1-reference`.

#### Scenario: Unreleased notes the restart

- **GIVEN** `CHANGELOG.md`
- **WHEN** the Unreleased section is read
- **THEN** it records the v2 restart and that v1 is preserved on branch `v1-reference`

### Requirement: Agent guide

The repository SHALL contain a short root `AGENTS.md` that points agents at the OpenSpec workflow and the rules in `openspec/config.yaml`.

#### Scenario: Guide points at OpenSpec

- **GIVEN** `AGENTS.md` at the repo root
- **WHEN** an agent reads it
- **THEN** it directs the agent to the OpenSpec workflow and to the rules in `openspec/config.yaml`

### Requirement: Role documentation tree

The repository SHALL contain `docs/` whose role subfolders are exactly `architect`, `designer`, `visual`, `qa`, `producer`, and `audio`, and each of those subfolders SHALL contain a `.gitkeep` or a short document.

#### Scenario: Role folders are exactly the six roles

- **GIVEN** the `docs/` tree
- **WHEN** the role subfolders are listed
- **THEN** they are exactly `architect`, `designer`, `visual`, `qa`, `producer`, and `audio`
- **AND** each contains a `.gitkeep` or a short document
- **AND** there is no `writer` folder

### Requirement: Docs index

`docs/README.md` SHALL index the role folders `architect`, `designer`, `visual`, `qa`, `producer`, and `audio`, and SHALL include the sentence "Design docs and decisions, one folder per role. OpenSpec specs live in openspec/."

#### Scenario: Index lists the six roles

- **GIVEN** `docs/README.md`
- **WHEN** the index is read
- **THEN** it lists `architect`, `designer`, `visual`, `qa`, `producer`, and `audio`
- **AND** it includes the sentence "Design docs and decisions, one folder per role. OpenSpec specs live in openspec/."
- **AND** it does not list `writer`
