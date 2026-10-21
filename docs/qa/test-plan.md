# Genesis QA Test Plan

Repo: https://github.com/fputs/genesis (OpenSpec-first). Phase 1: PF2e Remaster rules crate (`crates/rules`). Phase 2: basic map editor.

## Roles
- **Evan** reviews specs only (`openspec/changes/*`, `openspec/specs/*`). He does not review code.
- **Architect** opens implementation PRs and marks them ready for QA.
- **QA** tests behavior only, on PRs whose head CI is green: adversarial repros, edge cases, boundary values, demos, and regressions of earlier findings. Reports PASS/FAIL to **Chief of Staff** and **Architect**. **FAIL blocks merge.**
- **Architect** owns spec conformance, the constitution, the archive, CI, and fmt. QA does not re-check these (process change approved by Evan, 2026-10-20).

## Per-PR workflow
1. Identify the spec(s) the PR implements: the OpenSpec change named in the PR, `openspec/changes/<change>/specs/**/spec.md` (or `openspec/specs/**` once archived). Only specs merged to `main` count. If no approved spec exists, QA stops and flags it to Architect.
2. Check out and build (read-only; QA never pushes):
   ```
   git fetch origin pull/<N>/head:pr-<N> && git checkout pr-<N>
   git rev-parse HEAD            # record SHA
   cargo build --workspace
   cargo test --workspace
   ```
   The toolchain comes from `rust-toolchain.toml` (currently `1.99.1`). QA starts only once head CI is green; CI, fmt, and clippy are Architect's.
3. Headless demos: **TBD.** None exist yet (no `examples/`, bins, or demo scripts). When one lands, add its exact command here, e.g. `cargo run -p <crate> --example <name>`.
4. Build a behavior matrix: list every `#### Scenario` in the spec(s) to target behavior probes, and give each a check:
   - **Auto**: a named test in `cargo test`.
   - **Demo**: a headless demo run plus the expected output.
   - **Manual**: an exploratory step, with the exact command or file inspected.
   Every scenario needs at least one check. A scenario-to-test name mapping gap is reported to Architect, not counted as a QA FAIL.
5. Exploratory pass: edge cases near each scenario (boundary values, degrees of success, nat 1/20, ordering).
6. Report (template below) to Chief of Staff + Architect.

## Matrix format
| Spec | Requirement | Scenario | Check type | Check (test name / command) | Result |
|---|---|---|---|---|---|

Example (change `add-project-foundation`):
| dice-and-checks | Degrees of success | Total meets DC+10 | Manual | probe: DC 15, total 25 -> CriticalSuccess | |
| creature-template | Damage | Negative total doesn't heal | Auto | a_negative_total_does_not_raise_hit_points | |

## PASS/FAIL report
```
PR #<N> — <title> — SHA <sha> — PASS|FAIL
Specs: <paths>
Demos: <cmd → ok|fail|TBD>
Scenarios: <x>/<y> pass; failing: <list>
Bugs: <links>
Head CI: green (<link>)
Regressions: <finding> repro on <old SHA> -> gone on <new SHA> | still present | cannot verify
```
PASS means every behavior probe and scenario behaves as specified, with no reproduced bugs. Anything else is FAIL and blocks merge.

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
