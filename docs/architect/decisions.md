# Architecture decisions

## 2026-10-20 — v2 restart is OpenSpec-first

Genesis v2 restarts OpenSpec-first. v1 stays on branch `v1-reference` as a parts bin. Port a file only when a change names it.

## 2026-10-20 — Rules crate is engine-free

The rules crate has no Bevy, serde, or file I/O. Callers inject Dice. Ids are CombatantId.

## 2026-10-20 — Phase 1 change order

1. project-foundation
2. dice-and-checks, including flat checks and untyped MAP, including agile
3. creature-template: baked stats, optional stealth_dc, aid modifier, no fixed party size
4. grid-movement
5. encounter-core
6. circumstance-modifiers
7. then shields, reactive strike, positioning/flanking, seek/hidden, delay/ready/aid

## 2026-10-20 — Deferred

Deferred: attributes, proficiency, saves, skills, spells, classes, and broader conditions.

## 2026-10-20 — Phase 1 order revised

2026-10-20: Phase 1 order revised: modifiers (circumstance, status, untyped stacking; minimal) moves to change 3, right after dice-and-checks, because encounter-core needs unconscious -4 status. creature-template gains uses_dying_rules (default false; without it a creature dies at 0 HP). RAW fixes vs v1 tracked in docs/designer/pf2e-citations.md.

## 2026-10-20 — Workflow (Evan)

2026-10-20: Workflow (Evan): Evan reviews only specs. Per change: (1) spec-only PR (proposal, design, tasks, spec deltas); Evan approves and merges it. (2) After merge, Architect launches implementation on its own PR, no further approval. (3) Implementation merges when CI is green, all tasks checked, v1-ported acceptance tests pass, Architect review against spec passes, and Chief of Staff accepts; Architect then merges and archives the change. Any deviation from spec stops work and goes back to Evan as a spec amendment. No force-push; only merged PR branches are deleted.

## 2026-10-20 — Changelog interim rules (Chief of Staff ruling)

2026-10-20: Changelog interim rules (Chief of Staff ruling): (a) a CHANGELOG.md bullet is an Architect merge criterion for implementation PRs; (b) PRs touching only .cursor/**, README.md, Cargo.lock or rust-toolchain.toml are treated as docs-only and need no bullet.

## Backlog: end-of-Phase-1 changelog spec amendment

1. CI enforcement of the changelog bullet.
2. Add a Deprecated section.
3. The spec-drift-check bullet in CHANGELOG.md lacks a PR link because add-changelog task 2.1 mandated its wording, which contradicts the spec's link rule.
4. Release cutting (Unreleased to a version) is owned by DevOps.
5. Define categories for mixed spec+docs PRs and the toolchain/README/.cursor cases formally.

## 2026-10-21 — Composition: CharacterBuild -> pure derive -> Creature; runtime CombatantState separate.

Characters are built by composition. The same build inputs always produce the same character. `CharacterBuild` is immutable. Phase 1 stores the baked stat block and the choice fields (`stealth_dc`, `aid_modifier`, shield, `uses_dying_rules`). Later phases add ancestry, class, feat, attribute, and proficiency as further sources. `derive(&CharacterBuild) -> Creature` is pure: no Dice, no I/O, no global state. It folds an ordered list of contributions. Phase 1 has only the base contribution, and callers of `derive` do not change when a source is added. `Creature` is the read-only result. Runtime state (current Hit Points, temporary Hit Points, conditions, dying, wounded, shield Hit Points and raised, actions, reaction) lives on `CombatantState`, keyed by `CombatantId`, initialized from a `Creature`, and never written back into the build or the `Creature`.

## 2026-10-21 — spec-drift-check follows add-dice-and-checks

`spec-drift-check` slots immediately after `add-dice-and-checks`. The relative order of the rules changes after `add-dice-and-checks` stays as recorded on 2026-10-20.

## 2026-10-21 — Implementation archives in its last commit

An implementation pull request's last commit is `openspec archive <id> --yes`. There is no separate archive pull request. This gate leaves `main` green.
