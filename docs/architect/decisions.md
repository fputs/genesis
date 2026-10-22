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

## 2026-10-20 — Changelog interim rules (Chief of Staff ruling)

2026-10-20: Changelog interim rules (Chief of Staff ruling). (a) Revised 2026-10-21: the Unreleased bullet on an implementation PR is enforced by the `pr-hygiene` job. Reviewers do not re-check it. (b) PRs touching only .cursor/**, README.md, Cargo.lock, or rust-toolchain.toml are treated as docs-only and need no bullet.

## Backlog: end-of-Phase-1 changelog spec amendment

1. Add a Deprecated section.
2. The spec-drift-check bullet in CHANGELOG.md lacks a PR link because add-changelog task 2.1 mandated its wording, which contradicts the spec's link rule.
3. Release cutting (Unreleased to a version) is owned by DevOps.
4. Define categories for mixed spec+docs PRs and the toolchain/README/.cursor cases formally.

## 2026-10-21 — Composition: CharacterBuild -> pure derive -> Creature; runtime CombatantState separate.

Characters are built by composition. The same build inputs always produce the same character. `CharacterBuild` is immutable. Phase 1 stores the baked stat block and the choice fields (`stealth_dc`, `aid_modifier`, shield, `uses_dying_rules`). Later phases add ancestry, class, feat, attribute, and proficiency as further sources. `derive(&CharacterBuild) -> Creature` is pure: no Dice, no I/O, no global state. It folds an ordered list of contributions. Phase 1 has only the base contribution, and callers of `derive` do not change when a source is added. `Creature` is the read-only result. Runtime state (current Hit Points, temporary Hit Points, conditions, dying, wounded, shield Hit Points and raised, actions, reaction) lives on `CombatantState`, keyed by `CombatantId`, initialized from a `Creature`, and never written back into the build or the `Creature`.

## 2026-10-21 — spec-drift-check follows add-dice-and-checks

`spec-drift-check` slots immediately after `add-dice-and-checks`. The relative order of the rules changes after `add-dice-and-checks` stays as recorded on 2026-10-20.

## 2026-10-21 — Workflow v2 (Evan)

Evan reviews and merges spec PRs (proposal, design, tasks, spec deltas). After that merge, Architect launches implementation on its own PR.

1. The router wakes Architect once per head SHA, after CI finishes.
2. OpenSpec archive happens once, as the last commit right before merge (`openspec archive <id> --yes`). No un-archive and re-archive per fix. No separate archive pull request. That commit leaves `main` green.
3. Architect and QA review in parallel at that same head and post one combined numbered fix list. A red CI check is the send-back. Reviewers do not re-check CI-gated items: format, the changelog bullet, archive order, spec drift.
4. No per-PR Chief of Staff acceptance. Architect merges an implementation PR when CI is green and Architect and QA both pass at that same head. Chief of Staff spot-checks merged PRs daily.
5. Coding agents apply the whole fix list in one push. After two send-back rounds the PR escalates to Chief of Staff.
6. A small spec clarification is fixed in the implementation PR with a Designer citation. A behavior or scope change goes to Chief of Staff for Evan.
7. Designer and QA settle edge cases and tie rulings on the spec before coding is dispatched. A spec PR links `docs/designer/rules-rulings.md` and does not copy the log.

No force-push. Only merged PR branches are deleted.
