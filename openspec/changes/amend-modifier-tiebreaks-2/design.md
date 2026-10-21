# Design

## Context

`amend-modifier-tiebreaks` is unarchived on main. PR #30 implements it and is still open. `condition_prefers` returns at the lowest source id, and equal source ids keep the incumbent. See proposal.md for why that follows input order. The behavior to implement is `specs/modifiers/spec.md`.

## Goals / Non-Goals

**Goals:**

- Group on (condition, statistic, bonus or penalty, type) after the frightened and sickened rewrite to status.
- After equal source ids, the higher level wins. That step is an engineering tiebreak with no rules effect, and Rules 2266 (Player Core p.398) is context only. Then the canonical duplicate-key comparison.
- Make that comparison a total order over distinct modifiers.

**Non-Goals:**

- A new crate, a new dependency, or a new field on `Modifier`.
- Changing stacking, or the steps inside the duplicate-key comparison.
- Editing `docs/designer/rules-rulings.md` further. This change copies amendment 2. The implementation leaves that file as copied.
- Changing the frightened and sickened rewrite. The kept modifier is still a status penalty of `-rank`.

## Decisions

### 1. Group key, then the tie

Rewrite frightened and sickened penalties to status before grouping. Rank 0 still contributes nothing. The group key is then (condition, statistic, bonus or penalty, type). Different types do not share a group, so they stack (Player Core p.400). There is no type tiebreak.

`condition_prefers` keeps grouped value, then highest rank, a rank over none, then newest sequence, then lowest source id. When the source ids are equal it continues with the higher level, a level over none, then `key_prefers`. The level step is an engineering tiebreak with no rules effect. Rules 2266 (Player Core p.398) is context only. Equal modifiers keep the incumbent.

Circumstance -2 and status -2 are different groups. Circumstance -2 applies alone. Status -3 beats status -2. Every order applies those two and totals -5. Item -2 and circumstance -3 are different types, so both apply, and circumstance -2 loses to circumstance -3. Every order totals -5. Same type at level 1 and level 4 keeps level 4. Two frightened status penalties at rank 2 keep level 3, and the rewrite sets the value to -2.

Alternative considered: break a cross-type group with a type order. Rejected. Different types stack. The order dependence was the shared group, not a missing tiebreak.

### 2. Property test

The test is in `modifiers.rs` and adds no crate. A test-local generator uses a fixed seed and does not read a clock or a global generator. It emits random sets, one unconditioned untyped exact duplicate, and one set with both levels absent, so the comparison falls through to the canonical key. Each set has at most four modifiers, and every permutation of such a set is totaled. Every ordering keeps the same applied modifiers and the same total. The oracle, computed independently of `total`, is the best bonus per type plus the worst penalty per type plus every untyped penalty, after condition grouping and the R-01 duplicate-key collapse (one entry per source id, statistic, type, bonus or penalty, condition id). Frightened and sickened use `-rank`, and the status rewrite happens before the group key. One `ScriptedDice` is built and left unused.

## Risks / Trade-offs

- [Grouping sees the pre-rewrite type] → Frightened and sickened become status before the key is formed, so a frightened circumstance penalty joins the status group.
- [The implementation edits the rulings log] → This change copies amendment 2. The implementation leaves `docs/designer/rules-rulings.md` as copied.
- [`key_prefers` sees the raw value] → For frightened and sickened the grouped value is `-rank`. A raw-value difference after the other steps tie still picks one modifier, and the rewrite then sets `-rank`.
