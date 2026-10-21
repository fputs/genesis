# coding-constitution

## Purpose

The coding constitution binds every agent that writes code in this repository. `AGENTS.md` states the rules; this capability records the ones implementation and review can check.

## ADDED Requirements

### Requirement: Agents follow AGENTS.md

Agents that write code MUST follow `AGENTS.md` at the repo root. When a rule there conflicts with a spec, the agent MUST stop and report the conflict.

#### Scenario: A conflict stops the agent

- **GIVEN** a rule in `AGENTS.md` that conflicts with a spec
- **WHEN** an agent is about to implement the conflicting change
- **THEN** the agent stops and reports the conflict

### Requirement: Proof commands pass

Before work is done, the commands in `AGENTS.md` section 2 MUST pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace`, `cargo test --workspace`, and `npx @fission-ai/openspec@1.14.3 validate --all --strict`.

#### Scenario: A failing proof command blocks done

- **GIVEN** a change whose proof commands have been run
- **WHEN** any of those commands exits non-zero
- **THEN** the change is not done

### Requirement: Lint silences use expect with a reason

A lint MUST be silenced only with `#[expect(..., reason)]` on the smallest item possible. A crate-level `#[allow]` MUST NOT be used.

#### Scenario: A silence without a reason is rejected

- **GIVEN** library code that silences a lint
- **WHEN** the attribute is reviewed
- **THEN** it is `#[expect(..., reason)]` with a real reason, and it is not a crate-level `#[allow]`

### Requirement: The rules crate stays pure and dependency-free

`crates/rules` MUST declare no dependencies, and MUST NOT use Bevy, serde, files, the network, a clock, or global randomness. Callers MUST pass dice in.

#### Scenario: The rules manifest has no dependencies

- **GIVEN** the `crates/rules` package manifest
- **WHEN** its dependencies are read
- **THEN** the package declares no dependencies

### Requirement: Tests use scripted dice and cover every scenario

Tests MUST use scripted dice so results are exact. Every spec scenario MUST have at least one test.

#### Scenario: A scenario names a test

- **GIVEN** a spec scenario
- **WHEN** the tests for that change are read
- **THEN** at least one test is named after that scenario and uses scripted dice

### Requirement: Library code does not panic

Library code MUST NOT panic on bad input. Outside tests it MUST NOT use `unwrap()`, `expect()`, `panic!`, or unchecked indexing, and MUST return `Result` with a specific error enum.

#### Scenario: Bad input does not panic

- **GIVEN** library code outside tests
- **WHEN** that code is reviewed
- **THEN** it contains no `unwrap()`, `expect()`, `panic!`, or unchecked indexing

### Requirement: New dependencies need a spec

Adding a crate MUST be named in a spec that says why. `crates/rules` MUST stay dependency-free unless a spec explicitly allows one.

#### Scenario: An unnamed dependency is refused

- **GIVEN** a proposed new crate dependency
- **WHEN** no spec names that crate and says why
- **THEN** the dependency is not added
