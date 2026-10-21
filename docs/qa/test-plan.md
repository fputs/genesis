# Genesis QA Test Plan

Repo: https://github.com/fputs/genesis (OpenSpec-first). Phase 1: PF2e Remaster rules crate (`crates/rules`). Phase 2: basic map editor.

## Roles
- **Evan** reviews specs only (`openspec/changes/*`, `openspec/specs/*`). He does not review code.
- **Architect** opens implementation PRs and marks them ready for QA.
- **QA** verifies each ready PR against its merged spec(s) and reports PASS/FAIL to **Chief of Staff** and **Architect**. **FAIL blocks merge.**

## Per-PR workflow
1. Identify the spec(s) the PR implements: the OpenSpec change named in the PR, `openspec/changes/<change>/specs/**/spec.md` (or `openspec/specs/**` once archived). Only specs merged to `main` count. If no approved spec exists, the PR fails.
2. Check out and build (read-only; QA never pushes):
   ```
   git fetch origin pull/<N>/head:pr-<N> && git checkout pr-<N>
   git rev-parse HEAD            # record SHA
   cargo fmt --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo build --workspace
   cargo test --workspace
   ```
   The toolchain comes from `rust-toolchain.toml` (currently `1.99.1`, with clippy and rustfmt). These are the same four steps as `.github/workflows/ci.yml`. CI must also be green on the PR.
3. Headless demos: **TBD.** None exist yet (no `examples/`, bins, or demo scripts). When one lands, add its exact command here, e.g. `cargo run -p <crate> --example <name>`.
4. Build a scenario matrix: list every `#### Scenario` in the spec(s) and give each a check:
   - **Auto**: a named test in `cargo test` (config rule: tests use scripted dice and cite the Player Core section).
   - **Demo**: a headless demo run plus the expected output.
   - **Manual**: an exploratory step, with the exact command or file inspected.
   Every scenario needs at least one check. A scenario with no check counts as a FAIL (spec coverage gap).
5. Exploratory pass: edge cases near each scenario (boundary values, degrees of success, nat 1/20, ordering). Also check the Phase 1 constraints in `openspec/config.yaml`: `crates/rules` uses no Bevy, serde, or file I/O; Dice are injected; ids are `CombatantId`; no game/editor crates before Phase 2.
6. Report (template below) to Chief of Staff + Architect.

## Matrix format
| Spec | Requirement | Scenario | Check type | Check (test name / command) | Result |
|---|---|---|---|---|---|

Example (change `add-project-foundation`):
| project-foundation | Cargo workspace | Workspace has one member | Manual | `cargo metadata --no-deps --format-version 1` lists only `crates/rules` | |
| project-foundation | CI checks and Cargo cache | Required cargo steps | Manual | inspect `ci.yml` steps + green PR run | |
| project-foundation | Role documentation tree | Role folders are exactly the six roles | Manual | `ls docs/` = architect designer visual qa producer audio; no writer | |

## PASS/FAIL report
```
PR #<N> — <title> — SHA <sha> — PASS|FAIL
Specs: <paths>
Build/fmt/clippy/test: <ok|fail> (<n> passed, <n> failed)
Demos: <cmd → ok|fail|TBD>
Scenarios: <x>/<y> pass; failing: <list>
Bugs: <links>
```
PASS means everything is green and every scenario passes. Anything else is FAIL and blocks merge.

## Bug report template
```
Title:
Build / commit SHA:
Spec + scenario: <spec path> › <Requirement> › <Scenario>
Steps to reproduce:
Expected (per spec):
Actual:
Evidence / logs: <command output, test name, CI link>
Severity: Blocker | Major | Minor | Trivial
```

## Verification rule
Never mark a fix verified until you have (1) reproduced the original bug on the old build (record its SHA) and (2) confirmed the bug is gone on the new build (record its SHA), using the same steps. If the bug can't be reproduced on the old build, report it as "cannot verify". Do not mark it verified.
