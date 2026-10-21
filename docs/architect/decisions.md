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

## 2026-10-21 — Composition: CharacterBuild -> pure derive -> Creature; runtime CombatantState separate.

Characters are built by composition. The same build inputs always produce the same character. `CharacterBuild` is immutable. Phase 1 stores the baked stat block and the choice fields (`stealth_dc`, `aid_modifier`, shield, `uses_dying_rules`). Later phases add ancestry, class, feat, attribute, and proficiency as further sources. `derive(&CharacterBuild) -> Creature` is pure: no Dice, no I/O, no global state. It folds an ordered list of contributions. Phase 1 has only the base contribution, and callers of `derive` do not change when a source is added. `Creature` is the read-only result. Runtime state (current Hit Points, temporary Hit Points, conditions, dying, wounded, shield Hit Points and raised, actions, reaction) lives on `CombatantState`, keyed by `CombatantId`, initialized from a `Creature`, and never written back into the build or the `Creature`.
