# Design

## Context

`amend-modifier-tiebreaks` is unarchived on main. PR #30 implements it and is still open. `condition_prefers` returns at the lowest source id, and equal source ids keep the incumbent. See proposal.md for why that follows input order. The behavior to implement is `specs/modifiers/spec.md`.

## Goals / Non-Goals

**Goals:**

- After equal source ids, compare earlier type, then higher level, then the canonical duplicate-key comparison.
- Make the condition-group comparison a total order over distinct modifiers.
- Lock the repro at -5, the higher-level case, and a seeded property test.

**Non-Goals:**

- A new crate, a new dependency, or a new field on `Modifier`.
- Changing stacking, or the steps inside the duplicate-key comparison.
- Editing `docs/designer/rules-rulings.md` before the designer reviews the type order.
- Changing the frightened and sickened rewrite. The kept modifier is still a status penalty of `-rank`.

## Decisions

### 1. Steps after an equal source id

`condition_prefers` keeps its prefix: grouped value, then highest rank, a rank over none, then newest sequence, then lowest source id. A different source id still returns there.

When the source ids are equal, it continues:

1. Earlier type, in the order circumstance, item, status, untyped. This is an engineering tiebreak with no RAW basis. Designer review is pending. An untyped bonus cannot be constructed, so untyped applies to penalties.
2. Higher level. A level beats none.
3. `key_prefers`, the canonical duplicate-key comparison. It runs after type, because it does not look at type. Equal modifiers keep the incumbent.

The repro keeps circumstance -2 over status -2, then stacks the unconditioned status -3. Every order totals -5. Same type at level 1 and level 4 keeps level 4 and totals -2.

Alternative considered: keep the incumbent when the source id ties. Rejected. The two orders total -5 and -3.

Alternative considered: compare level before type. Rejected. The stated order is type, then level.

### 2. Property test

The test is in `modifiers.rs` and adds no crate. A test-local generator uses a fixed seed. Each set has at most four modifiers, and the sets include a same-source, same-value collision across type and one across level. Every ordering is totaled. The totals and the applied modifiers match, and the same seed produces the same sets. The test does not read a clock or a global generator. One `ScriptedDice` is built and left unused.

## Risks / Trade-offs

- [The designer rejects the type order] → The requirement labels it an engineering tiebreak with no RAW basis, and this change does not write the rulings log. A different order is a spec amendment before implementation.
- [A frightened type tie looks unchanged] → The winner is rewritten to status `-rank`. The repro uses condition 10, which is not rewritten.
- [`key_prefers` sees the raw value] → For frightened and sickened the grouped value is `-rank`. A raw-value difference after the other steps tie still picks one modifier, and the rewrite then sets `-rank`.
