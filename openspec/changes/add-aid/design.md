# Design

## Context

`aid_modifier: Option<i32>` is copied onto the Creature. `aid_check` rolls it against DC 15 and `aid_circumstance_bonus` maps a critical success to +2. The creature-template requirement records that fixed +2 because the modifier has no rank. `attack_modifiers` already takes an Aid circumstance `i32` and omits 0. It does not prepare or roll Aid. `Encounter::spend_actions` and `Encounter::spend_reaction` already exist. `spend_reaction` returns `NotConscious` when vital is not `Alive`, then `NoReaction`. `Vital` on the dying-and-victory spec is `Alive`, `Unconscious`, `Dying(1..=3)`, and `Dead`. `Side` is `Pc` or `Enemy`. See proposal.md for why this change exists.

This change names no file on `v1-reference`, so none is ported. It does not edit `docs/designer/`.

## Goals / Non-Goals

**Goals:**

- One module, `crates/rules/src/aid.rs`, for the rank, the circumstance, and the DC 15 bonus.
- `Encounter::prepare_to_aid` spends one action through `spend_actions`.
- `Encounter::aid` rolls once and spends the reaction through `spend_reaction`.
- The returned circumstance is an input to one `attack_modifiers` call. Nothing stores it.

**Non-Goals:**

- A skill list, a save, an attribute, a level, or a legendary rank. `AidRank` is only the critical-success bonus.
- Distance, cover, or a square. DEV-51 leaves proximity unchecked.
- Changing `Encounter::strike` to remember an Aid bonus. The caller passes the value on that one check.
- Reading the ally's vital. The caller decides that the ally is about to roll.
- Editing `docs/designer/rules-deviations.md` or `docs/designer/rules-rulings.md`.

## Decisions

### Types

```rust
pub enum AidRank {
    Untrained,
    Trained,
    Expert,
    Master,
}

pub enum AidCircumstance {
    Plus1,
    Plus2,
    Plus3,
    Plus4,
    Zero,
    Minus1,
}

pub enum AidPreparation {
    NotPrepared,
    Prepared,
}
```

`AidCircumstance::value` returns +1, +2, +3, +4, 0, or -1. `attack_modifiers` keeps its `i32` argument. This change does not add a bool. `BuildParts.aid_rank` is `Option<AidRank>`. `None` stores `Trained` on the Creature. `aid_modifier` stays `Option<i32>`.

```rust
pub fn aid_circumstance_bonus(degree: DegreeOfSuccess, rank: AidRank) -> AidCircumstance;
```

Success, failure, and critical failure ignore `rank`. A critical success reads it: Untrained Plus1, Trained Plus2, Expert Plus3, Master Plus4. `aid_check` still returns `None` without rolling when the modifier is absent or `CheckModifier::new` fails. Otherwise it returns the roll and that circumstance.

### Prepare

`prepare_to_aid` calls `spend_actions(id, 1)` and sets `Prepared` only after that returns `Ok`. The order is the order `spend_actions` already uses: no open turn, not your turn, not Alive, then fewer than 1 action. A second prepare on the same turn spends another action and stays `Prepared`.

`begin_encounter` sets `NotPrepared` for every participant, including one handed in as `Prepared`. `end_turn` does not clear it. `start_turn`, after the round increment has succeeded, sets the creature who is starting to `NotPrepared`. `RoundOverflow` from `end_turn` returns before any later start, so a prepared ally of the creature who would have started stays `Prepared`.

### Reaction

`aid(aider, ally, dice)` checks, and draws no die, in this order:

1. `UnknownCombatant` when the aider is absent.
2. `UnknownCombatant` when the ally is absent.
3. `NotConscious` when the aider's vital is not `Alive`. This wins over self, `NotPrepared`, `NoAidModifier`, `AidOverflow`, and `NoReaction`.
4. `SelfTarget` when the ids match.
5. `NotAnAlly` when the sides differ.
6. `NotPrepared`.
7. `NoAidModifier` when `aid_modifier` is `None`.
8. `AidOverflow` when the modifier cannot be added to every face in `0..=255`. Modifier `2147483647` is this error.
9. `NoReaction` when the reaction is not ready.

Then it rolls through `aid_check`, then calls `spend_reaction`. Preparation stays `Prepared`. Actions, Hit Points, and the ally's attack bonus stay as they were. The ally's vital is not read. The open turn does not have to be the ally, so an aider with 0 actions can still aid on the ally's turn.

A second aid in the same round is step 9, because preparation was not cleared. The next successful `start_turn` for the aider clears preparation and grants the reaction again. An aid before a new prepare is step 6.

The caller passes `value()` to `attack_modifiers` for the triggering check only. A later call that passes 0 does not include the bonus. `Encounter::strike` gains no Aid field.

Same `Side` is an ally, including `Enemy` with `Enemy`. That is the willing requirement in this change.

### Proposed ledger rows

Do not edit `docs/designer/rules-deviations.md` in this change. Designer records these. The ids are proposals. DEV-44 through DEV-49 are reserved for reactive-strike. DEV-02 and DEV-04 stay as already ledgered: the baked modifier, and DC 15 with no GM adjustment. The fixed-+2 substitute on the current creature-template requirement is replaced by DEV-50.

| ID | Rule | RAW citation | Our behavior | Reason | Owning change |
|---|---|---|---|---|---|
| DEV-50 | Aid critical-success bonus | https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416) | `AidRank::{Untrained, Trained, Expert, Master}` grants +1, +2, +3, +4. No legendary rank. Success +1, failure 0, and critical failure -1 do not read the rank. | Phase 1 has no skill proficiency. The four bonuses are the ranks this change stores. | 13 |
| DEV-51 | GM adjudication of Aid | Same page | Preparation is exactly 1 action on your turn. Willing means the same `Side`. No distance check, no repeat DC, and the ally's vital is not read. The caller names one attack check. | No GM, no skills, and no consent or distance model on this action. | 13 |

(DEV-50, proposed). (DEV-51, proposed).

### Errors and purity

`EncounterError` gains `NotPrepared`, `NoAidModifier`, `NotAnAlly`, and `AidOverflow(ArithmeticOverflow)`. `NotConscious`, `NoReaction`, `SelfTarget`, `NotYourTurn`, `InsufficientActions`, and `UnknownCombatant` stay the variants from encounter-turns. No string errors. No `unwrap`, `expect`, or `panic!`. No `unsafe`, no new dependency, and no clock. Dice are drawn only by the successful `aid` roll and by initiative. `aid_circumstance_bonus` does not roll.

## Risks / Trade-offs

- [The critical-success numbers differ from the book] → DEV-50 states the four ranks. Success, failure, and critical failure match the book.
- [`strike` does not apply the bonus by itself] → The scenario passes the value into one `attack_check_total` and a second call with 0. A later change can pass that list into `strike`.
- [Dying vitals are not on this branch's `Vital`] → Implementation waits until `Vital::Unconscious` and `Vital::Dying` are public. This spec pull request does not add them.
- [A prepared creature who never starts again stays Prepared] → `NotConscious` still blocks the reaction. `begin_encounter` and that creature's next start are the clears.

## Migration Plan

Spec only in this pull request. No changelog bullet and no archive. The implementation pull request adds `aid.rs`, the state field, the two encounter methods, and the error variants. It updates `aid_check_degrees_use_the_fixed_critical_bonus` so its comment cites DEV-50 and its Trained critical success stays +2. It keeps `a_natural_20_uses_the_fixed_critical_bonus` as the Trained natural 20, which stays Plus2, and adds the Expert natural 20 beside it. It does not delete `an_aid_modifier_past_i32_is_not_rolled`. It adds the Unreleased changelog bullet before it is done. `openspec archive add-aid --yes` is the last commit of that pull request. Rollback is reverting that pull request. There is no saved encounter to migrate.
