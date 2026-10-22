# Genesis QA Test Plan

Repo: https://github.com/fputs/genesis (OpenSpec-first). Phase 1: PF2e Remaster rules crate (`crates/rules`). Phase 2: basic map editor.

## Roles
- **Evan** reviews spec PRs only. He does not review implementation code.
- **Designer and QA** settle edge cases and tie rulings on the spec before coding is dispatched. The spec PR links `docs/designer/rules-rulings.md` and does not copy that log.
- The router wakes **Architect** once per head SHA after CI finishes. **Architect** and **QA** review that same head in parallel and post one combined numbered fix list. A red CI check is the send-back. Neither re-checks format, the changelog bullet, archive order, or spec drift.
- **QA** tests behavior: adversarial repros, edge cases, boundary values, demos, and regressions of earlier findings. **FAIL blocks merge.**
- **Architect** reviews spec conformance and the constitution. Architect merges when CI is green and both pass at that same head. No per-PR Chief of Staff acceptance.

## Per-PR workflow
1. Identify the spec(s) the PR implements: the OpenSpec change named in the PR, `openspec/changes/<change>/specs/**/spec.md` (or `openspec/specs/**` once archived). Only specs merged to `main` count. If no approved spec exists, QA stops and flags it to Architect.
2. Check out and build (read-only; QA never pushes):
   ```
   git fetch origin pull/<N>/head:pr-<N> && git checkout pr-<N>
   git rev-parse HEAD            # record SHA
   cargo build --workspace
   cargo test --workspace
   ```
   The toolchain comes from `rust-toolchain.toml` (currently `1.99.1`). Review starts when CI on that head has finished. A red check sends the PR back. QA does not re-check format, the changelog bullet, archive order, or spec drift.
3. Headless demos: **TBD.** None exist yet (no `examples/`, bins, or demo scripts). When one lands, add its exact command here, e.g. `cargo run -p <crate> --example <name>`.
4. Build a behavior matrix: list every `#### Scenario` in the spec(s) to target behavior probes, and give each a check:
   - **Auto**: a named test in `cargo test`.
   - **Demo**: a headless demo run plus the expected output.
   - **Manual**: an exploratory step, with the exact command or file inspected.
   Every scenario needs at least one check. A scenario-to-test name mapping gap is reported to Architect, not counted as a QA FAIL.
5. Exploratory pass: edge cases near each scenario (boundary values, degrees of success, nat 1/20, ordering).
6. Fold the report (template below) into the one combined numbered fix list on the PR.

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
