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

## 2026-10-22 — Changelog backlog moved to the spec

The end-of-Phase-1 changelog backlog moved to the changelog spec in `amend-changelog-policy`. The moved items are the Deprecated section, the pull-request link rule for the unlinked spec-drift bullet, DevOps-owned release cuts, and the category for a mixed pull request, including cursor, README, and toolchain files. This section does not restate those rules.

## 2026-10-21 — Composition: CharacterBuild -> pure derive -> Creature; runtime CombatantState separate.

Characters are built by composition. The same build inputs always produce the same character. `CharacterBuild` is immutable. Phase 1 stores the baked stat block and the choice fields (`stealth_dc`, `aid_modifier`, shield, `uses_dying_rules`). Later phases add ancestry, class, feat, attribute, and proficiency as further sources. `derive(&CharacterBuild) -> Creature` is pure: no Dice, no I/O, no global state. It folds an ordered list of contributions. Phase 1 has only the base contribution, and callers of `derive` do not change when a source is added. `Creature` is the read-only result. Runtime state (current Hit Points, temporary Hit Points, conditions, dying, wounded, shield Hit Points and raised, actions, reaction) lives on `CombatantState`, keyed by `CombatantId`, initialized from a `Creature`, and never written back into the build or the `Creature`.

## 2026-10-21 — spec-drift-check follows add-dice-and-checks

`spec-drift-check` slots immediately after `add-dice-and-checks`. The relative order of the rules changes after `add-dice-and-checks` stays as recorded on 2026-10-20.

## 2026-10-21 — Workflow v2 (Evan)

Superseded on 2026-10-22 by the single draft pull request below. These standing rules now apply on that one pull request.

1. The router wakes Architect once per head SHA, after CI finishes.
2. Revised 2026-10-23: archive once, as the last commit, only after Architect and QA have both passed the current head (`openspec archive <id> --yes`). Never un-archive. If a fix is needed after the archive, Architect escalates to Chief of Staff. No separate archive pull request. That commit leaves `main` green.
3. Architect and QA review in parallel at that same head and post one combined numbered fix list. A red CI check is the send-back. Reviewers do not re-check CI-gated items: format, the changelog bullet, archive order, spec drift.
4. No per-PR Chief of Staff acceptance. Architect merges that pull request when CI is green after that archive commit. Architect and QA have both passed the head that was archived. Chief of Staff spot-checks merged PRs daily.
5. Coding agents apply the whole fix list in one push. After two send-back rounds the PR escalates to Chief of Staff.
6. A small spec clarification is fixed in that pull request, cites Designer, and needs a new `spec-approved: <sha>` review. A behavior or scope change goes to Chief of Staff for Evan.
7. Designer and QA settle edge cases and tie rulings on the spec before coding is dispatched. The pull request links `docs/designer/rules-rulings.md` and does not copy the log.

No force-push. Only merged PR branches are deleted.

## 2026-10-22 — One draft pull request (Evan)

One change is one draft pull request. The body includes `OpenSpec-Change: <id>`. The first commit touches only `openspec/changes/<id>/`. Designer rules on that pull request before tests or code. Evan approves the spec only by a pull request review whose body line is `spec-approved: <sha>`. An issue comment does not approve. Tests, then the code they cover, land on the same branch. Architect and QA review that one head in parallel. The merger archives once, right before merge, after both have passed that head. Architect merges when CI is green. There is no second pull request.

A refactor with no spec edit uses one line `OpenSpec-Refactor: <kebab-item>` and no `OpenSpec-Change` line. The item is one kebab token and is not looked up. Every changed path starts with `crates/` or is exactly `CHANGELOG.md`. A `crates/` path containing `/tests/` is not modified. The Unreleased bullet is still required. Architect does not merge until Architect and QA confirm, in the combined review, that behavior is unchanged.

spec-drift does not require OpenSpec-Change for script and approval paths; spec-approved enforces it.

Revised 2026-10-23: required code-owner review through Chief of Staff, and the required `spec-approved` check. CODEOWNERS for `/openspec/`, `/AGENTS.md`, `/.github/`, and `/scripts/` takes effect only after both. The workflow file runs from the pull request merge ref, so a pull request can rewrite it. An open pull request whose base sha predates the merge keeps bootstrap until an event refreshes that base. This repository change does not edit branch protection.

## 2026-10-22 — PR #70 un-archived during a fix round (recorded)

PR #70 (`add-delay-and-ready`) was un-archived in `2f2f8aa` and re-archived in `fe6bc6d` during a fix round. That is against AGENTS.md section 1. The archived folder is dated 2026-10-23, and its archive task 7.3 is unticked. Evan accepted it as recorded. The cause was an Architect instruction.
