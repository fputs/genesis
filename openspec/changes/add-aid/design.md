# Design

## Context

`aid_modifier: Option<i32>` is copied onto the Creature. `aid_check` rolls it against DC 15 and `aid_circumstance_bonus` maps a critical success to +2. The creature-template requirement records that fixed +2 as DEV-03, because the modifier had no rank. This change supersedes DEV-03. `attack_modifiers` already takes an Aid circumstance `i32` and omits 0. It does not prepare or roll Aid. `Encounter::spend_actions` and `Encounter::spend_reaction` already exist. `spend_reaction` returns `NotConscious` when vital is not `Alive`, then `NoReaction`. `Vital` on the dying-and-victory spec is `Alive`, `Unconscious`, `Dying(1..=3)`, and `Dead`. `Side` is `Pc` or `Enemy`. See proposal.md for why this change exists.

This change names no file on `v1-reference`, so none is ported. It does not edit `docs/designer/`.

## Goals / Non-Goals

**Goals:**

- One module, `crates/rules/src/aid.rs`, for the rank, the circumstance, and the DC 15 bonus.
- `Encounter::prepare_to_aid` spends one action through `spend_actions` and records one ally.
- `Encounter::aid` rolls once and spends the reaction through `spend_reaction`.
- The returned circumstance is an input to one attack total. Nothing stores it.

**Non-Goals:**

- A skill list, a save, an attribute, or a level. `AidRank` is only the critical-success bonus, not a skill proficiency.
- Distance, cover, or a square. DEV-51 leaves proximity unchecked.
- Changing `Encounter::strike` to remember an Aid bonus. The caller passes the value on that one check.
- Reading the ally's vital. A dead ally can still be aided.
- Editing `docs/designer/rules-deviations.md` or `docs/designer/rules-rulings.md`. The log is `docs/designer/rules-rulings.md`. DEV-03's ledger row is retired in a later docs pull request.

## Decisions

### Types

```rust
pub enum AidRank {
    BelowMaster,
    Master,
    Legendary,
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
    Prepared { ally: CombatantId },
}
```

`AidCircumstance::value` returns +1, +2, +3, +4, 0, or −1. `attack_modifiers` keeps its `i32` argument. A positive value is one circumstance bonus. A negative value is one circumstance penalty. Zero adds no modifier. This change does not add a bool. `BuildParts.aid_rank` is `Option<AidRank>`. `None` stores `BelowMaster` on the Creature. `aid_modifier` stays `Option<i32>`.

```rust
pub fn aid_circumstance_bonus(degree: DegreeOfSuccess, rank: AidRank) -> AidCircumstance;
```

Success, failure, and critical failure ignore `rank`. Success is `Plus1`. Failure is `Zero`. Critical failure is `Minus1`. A critical success reads the rank: `BelowMaster` is `Plus2`, `Master` is `Plus3`, and `Legendary` is `Plus4`.

The Aid roll draws one d20 and adds the modifier with `checked_add`. It does not call `CheckModifier::new`. That constructor rejects a modifier that cannot be added to every face in `0..=255`, so a die that can show 255 still fits in `i32`. Aid rolls a d20. Its highest face is 20, so the precheck is `checked_add(modifier, 20)`. Modifier `2147483627` plus 20 is `i32::MAX` and is rolled. Modifier `2147483628` plus 20 does not fit, so the result is `AidOverflow(ArithmeticOverflow::SumDoesNotFit)` and no die is drawn. There is no second overflow type. `aid_check` returns `None` without rolling when the modifier is absent or that precheck fails. Otherwise it returns the roll and that circumstance.

### Prepare

`prepare_to_aid(aider, ally)` records `Prepared { ally }` only after the action is spent. It draws no die. Before spending, it does not mutate. The order is the `spend_actions` refusals first: no open turn, not your turn, not Alive, then fewer than 1 action. The named ally is next: `UnknownCombatant` when that id is absent, then `SelfTarget` when it is the preparer, then `NotAnAlly` when the sides differ. `NoAidModifier` is next, when `aid_modifier` is `None`. Only then does it call `spend_actions(aider, 1)` and store the named ally. A second prepare on the same turn spends another action and replaces the named ally.

`begin_encounter` sets `NotPrepared` for every participant, including one handed in as `Prepared`. `end_turn` does not clear it. `start_turn`, after the round increment has succeeded, sets the creature who is starting to `NotPrepared`. `RoundOverflow` from `end_turn` returns before any later start, so a prepared ally of the creature who would have started stays `Prepared`. An `aid` that returns `Ok` also sets that aider to `NotPrepared`. An error does not.

### Reaction

`aid(aider, ally, dice)` checks, and draws no die, in this order:

1. `UnknownCombatant` when the aider is absent.
2. `UnknownCombatant` when the ally is absent.
3. `TurnNotOpen` when no turn is open.
4. `NotConscious` when the aider's vital is not `Alive`. This wins over self, `NotThePreparedAlly`, `NotPrepared`, `NoAidModifier`, `AidOverflow`, and `NoReaction`.
5. `SelfTarget` when the ids match. This wins over `NotPrepared`.
6. `NotAnAlly` when the sides differ.
7. `NotThePreparedAlly` when preparation names a different ally.
8. `NotPrepared`.
9. `NoAidModifier` when `aid_modifier` is `None`.
10. `AidOverflow` when `checked_add(modifier, 20)` fails.
11. `NoReaction` when the reaction is not ready.

Then it rolls, then calls `spend_reaction`. `Ok` clears preparation. Actions, Hit Points, and the aider's attack count stay as they were. Aid has no attack trait, so the attack count does not change. The ally's vital is not read. The open turn does not have to be the ally, so an aider with 0 actions can still aid on someone else's turn.

Aid and Reactive Strike share this one reaction. The Aid scenario spends it with `spend_reaction` and does not call the reactive-strike API. The next `aid` returns `NoReaction`, and the die is unused. Preparation stays, because that `aid` did not return `Ok`. The reverse note is the same flag: an Aid that returned `Ok` has spent the reaction, so a later Reactive Strike is `NoReaction`. That scenario is specified on the reactive-strike side and lives in PR #55.

A second `aid` after `Ok` is step 8, because preparation was cleared, even though the reaction is also spent. Step 8 wins over step 11. The next successful `start_turn` for the aider clears a preparation the ally never triggered, and the reaction is ready again because it was not spent.

The caller passes `value()` into the attack total for the triggering check only. `attack_modifiers` still takes one Aid `i32`. A second aider, or any other circumstance modifier, is another entry in the caller list passed to `attack_check_total`. `total` keeps the highest circumstance bonus and the worst circumstance penalty. A circumstance bonus and a circumstance penalty both apply. `Encounter::strike` gains no Aid field.

Same `Side` is an ally, including `Enemy` with `Enemy`. That is DEV-17 and DEV-26.

### Proposed ledger rows

Do not edit `docs/designer/rules-deviations.md` in this change. Designer records these. DEV-03 is superseded by the ranks above and is not given a new row here. DEV-44 through DEV-49 stay reserved for the reactive-strike spec (PR #55). This change does not use them. DEV-02 and DEV-04 stay as already ledgered: the baked modifier, and DC 15 with no GM adjustment. DEV-50 and DEV-51 are the ids this change proposes.

| ID | Rule | RAW citation | Our behavior | Reason | Owning change |
|---|---|---|---|---|---|
| DEV-50 | Aid preparation lifetime | https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416). The book requires a preparation, usually an action on your turn, and does not say when that preparation ends. | `prepare_to_aid` names one ally. The preparation ends at that aider's next `start_turn`, or when `aid` returns `Ok`, whichever comes first. | Architect ruling. The table needs a typed end. | add-aid |
| DEV-51 | Aid trigger | https://2e.aonprd.com/Actions.aspx?ID=2292 (Player Core pg. 416). The trigger is an ally about to use an action that requires a skill check or attack roll. | The caller names one attack roll. A skill check is not a trigger. No distance check and no repeat DC. The one-action preparation cost is not this row. Willing is not restated here: same `Side` is DEV-17 and DEV-26. | Phase 1 has no skill check and no GM on this action. | add-aid |

### Errors and purity

`EncounterError` gains `NotPrepared`, `NoAidModifier`, `NotAnAlly`, `NotThePreparedAlly`, and `AidOverflow(ArithmeticOverflow)`. `NotConscious`, `NoReaction`, `SelfTarget`, `NotYourTurn`, `TurnNotOpen`, `InsufficientActions`, and `UnknownCombatant` stay the variants from encounter-turns. No string errors. No `unwrap`, `expect`, or `panic!`. No `unsafe`, no new dependency, and no clock. Dice are drawn only by the successful `aid` roll and by initiative. `aid_circumstance_bonus` does not roll.

## Risks / Trade-offs

- [DEV-03's ledger row still says the crit is always +2] → This spec supersedes that substitute. The docs pull request updates the ledger. This pull request does not.
- [The book does not time the preparation] → DEV-50 ends it at the next start or at `Ok`.
- [`strike` does not apply the bonus by itself] → The scenario passes the value into one `attack_check_total` and a second call with 0. A later change can pass that list into `strike`.
- [Dying vitals are not on this branch's `Vital`] → Implementation waits until `Vital::Unconscious` and `Vital::Dying` are public. This spec pull request does not add them.
- [A prepared creature who never starts again, and whose aid never returns `Ok`, stays Prepared] → `NotConscious` still blocks the reaction. `begin_encounter` is the other clear.

## Migration Plan

Spec only in this pull request. No changelog bullet and no archive. The implementation pull request adds `aid.rs`, the state field, the two encounter methods, and the error variants. It updates `aid_check_degrees_use_the_fixed_critical_bonus` so its comment says DEV-03 is superseded and a `BelowMaster` critical success is +2. It keeps `a_natural_20_uses_the_fixed_critical_bonus` and does not add `a_natural_20_below_master_grants_plus_2`. The kept scenario is BelowMaster and Plus2. It drops the scenario title `An aid modifier past i32 is not rolled`. That title is not on main's creature-template spec. The encounter tests are the d20-limit pair, and `the_aid_check_at_modifier_2147483628_rolls_nothing` covers the creature check. It does not leave a test under the old overflow name. It adds the Unreleased changelog bullet before it is done. `openspec archive add-aid --yes` is the last commit of that pull request. Rollback is reverting that pull request. There is no saved encounter to migrate.
