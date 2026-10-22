# Design

## Context

`Encounter::spend_reaction` already clears one ready reaction for an Alive combatant and returns `NotConscious` without writing when the vital is not Alive. `Encounter::strike` spends an action, reads the multiple attack penalty from the open turn, and increments the attack count. `in_reach` already answers reach. The reactor's melee reach is `ReachFeet` on the creature's strike. Squares are not stored on the encounter. `Vital::Dying` and `Vital::Unconscious` are named by `add-dying-and-victory` and are not in the crate yet. See proposal.md for why this change exists.

This change names no file on `v1-reference`, so none is ported. It does not edit `docs/designer/`.

## Goals / Non-Goals

**Goals:**

- One module, `crates/rules/src/reactive.rs`, for the trigger, the reach test, and the disruption result.
- `Encounter::reactive_strike` spends the reaction only through `spend_reaction`.
- The Strike uses penalty 0 and does not change the stored attack count.
- A critical hit on a manipulate action is `Disruption::Disrupted`.

**Non-Goals:**

- Reading a path, a `SquareEvent`, or any square other than the actor's square at the moment of the action (proposed DEV-45). Grid-movement keeps emitting leave and enter events and still resolves no reaction.
- Storing a square or a reach on `CombatantState` or `Encounter`.
- A feat or a class (proposed DEV-44).
- Calling `Encounter::strike`. That function spends an action and applies the multiple attack penalty.
- Shield Block. `add-shields` owns that argument. This Strike does not take it.
- Resolving Point Out, Stride, Drop Prone, Stand, Crawl, or Tumble Through. The caller names the activity.
- A second `ActionTrait`, a second damage function, or a second dying formula.
- Editing `docs/designer/rules-deviations.md` or `docs/designer/rules-rulings.md`.

## Decisions

### One call, caller squares

```rust
pub enum Activity {
    Manipulate,
    Move,
    Step,
    RangedAttack,
    Other,
}

pub enum ReactiveStrike {
    Available,
    Unavailable,
}

pub enum Disruption {
    Disrupted,
    NotDisrupted,
}

pub struct ReactiveOutcome {
    pub strike: StrikeOutcome,
    pub disruption: Disruption,
}
```

`Encounter::reactive_strike` takes the reactor id, the actor id, `Activity`, `ReactiveStrike`, both squares, both `Size` values, a `StrikeProfile`, the caller modifiers, and the dice. Reach is `creature.strike.reach_feet` for the reactor. There is no reach argument and no `bool`.

`Activity::Move` is Stride, Stand, Drop Prone, and Crawl. `Activity::Step` is Step, which has the move trait and does not trigger. `Activity::Other` is a melee Strike. `Activity::RangedAttack` is the ranged-attack trigger and is not a manipulate action. The caller passes Point Out as `Activity::Manipulate`. This module does not import positioning's `ActionTrait`.

Alternative considered: a `triggers: bool` on Move. Rejected. Step and Stride would share a variant and hide the rule in a flag.

Alternative considered: walk `MoveOutcome.events`. Rejected. That is the path case in proposed DEV-45. The encounter does not keep those events.

### Reach

Check size, then the reach value, then `in_reach`. Either creature not Small or Medium is `UnsupportedSize` (DEV-19). A reach that is not a multiple of 5 is `InvalidReach`. Reach 0 is a multiple of 5; `in_reach` then returns false and the error is `OutOfReach`. The same square is `OutOfReach`. False from `in_reach` is `OutOfReach`. `GridError::Overflow` is `Overflow`. No other grid error is produced for two Small or Medium squares.

`in_reach` is the only distance function. A reach of 5, 10, or greater than 10 keeps that function's diagonal rule.

### Errors

`ReactiveError` is `UnknownCombatant`, `SelfTarget`, `NotConscious`, `Unavailable`, `NotATrigger`, `UnsupportedSize`, `InvalidReach`, `Overflow`, `OutOfReach`, `TargetDead`, `NoReaction`, `AttackOverflow(ModifierError)`, `DamageOverflow(ModifierError)`, and `HitPoints(HpError)`. No string errors. The encounter method returns `ReactiveError`. It does not add those variants to `EncounterError`.

The order is the spec's order. `NotConscious` is `vital != Vital::Alive`, the same test as `spend_reaction`. `Unavailable` is `ReactiveStrike::Unavailable`. `NotATrigger` is `Step` or `Other`. `TargetDead` is the actor's vital `Dead`. `NoReaction` is a reactor whose reaction is not ready.

An error before the d20 draws no attack die. `AttackOverflow` is `AttackTotalTerm` rejecting the attack bonus, before the d20. `DamageOverflow` is the damage sum failing after the d20 and the damage die. Both leave the reaction ready and leave Hit Points unchanged.

### The Strike

Do not call `Encounter::strike`. Build the attack total with `attack_check_total`, penalty term 0, and the caller modifiers on the attack statistic only. Do not call `attack_trait_penalty`. Agile does not change the penalty. The degree is `degree_of_success` against the actor's baked Armor Class (DEV-36). A miss deals 0 and draws no damage die. A hit rolls damage once and calls `damage_dealt`, which doubles a critical success. Apply that amount with the same function `Encounter::strike` uses. Do not copy that function and do not call `note_hit_points` from this module.

The stored attack count is not incremented. Actions are not changed. The turn is not ended.

Resolve the Hit Point write on a copy of the actor's state. Call `spend_reaction` only after that copy succeeds. Then copy Hit Points, temporary Hit Points, and vital back. `spend_reaction` is what clears `reaction_ready`. Copying the whole state back would restore the reaction. On `spend_reaction` `Err`, leave the actor's Hit Points as they were.

`Disruption::Disrupted` is only `DegreeOfSuccess::CriticalSuccess` together with `Activity::Manipulate`. Every other `Ok` is `NotDisrupted`, including a critical hit on Move or RangedAttack and a natural 20 that improved only to success.

Alternative considered: spend the reaction before the roll. Rejected. A later overflow would have spent it. The spec leaves the reaction ready on `AttackOverflow` and `DamageOverflow`.

### Proposed ledger rows

Do not edit `docs/designer/` in this change. Designer records these. The ids continue after DEV-43.

#### DEV-44 No class feat

**Rule**: Reactive Strike is a class feat. https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138).

**Why not**: Classes and feats are out of Phase 1.

**Substitute**: The caller passes `ReactiveStrike::Available` or `ReactiveStrike::Unavailable`. There is no feat and no class. `Unavailable` returns `Unavailable`, draws no die, and spends nothing.

**Intent**: The procedure runs for a creature the caller marks, and a creature who is not marked cannot react.

#### DEV-45 Leaving a square during a move

**Rule**: The trigger also includes a creature who leaves a square during a move action it is using. https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138).

**Why not**: The encounter does not store squares, and this change does not track a path. Grid-movement emits leave and enter events for one Stride or Step and does not resolve a reaction. Positioning does not keep a square on the combatant.

**Substitute**: Only the actor's square at the moment of the action is tested. A move that starts outside reach is `OutOfReach`, with no path argument.

**Intent**: Stand, Drop Prone, Stride, and Crawl still trigger when that square is already inside reach. The path case is `wire-leave-square-into-reactive-strike`.

DEV-19, DEV-36, and R-05 are cited, not restated. FIX-06, FIX-07, and FIX-08 are the trigger this change follows.

## Risks / Trade-offs

- [A move through reach is missed] → Proposed DEV-45. The in-reach Move scenario still covers Stand and Drop Prone. The follow-up is named above.
- [Baked Armor Class ignores off-guard and a raised shield] → Same boundary as `Encounter::strike` (DEV-36). The caller passes an attack modifier for the prone penalty. Shield Block is `add-shields`.
- [`Vital::Dying` is not in the crate yet] → Implementation waits until `Vital::Dying` and `Vital::Unconscious` are public. `NotConscious` stays `vital != Alive`.
- [The Hit Point copy and the reaction spend can diverge] → The spend runs only after the copy succeeds, and only Hit Points, temporary Hit Points, and vital are copied back.
- [Positioning and dying-and-victory are not archived specs] → This change does not modify their headers. It calls `in_reach`, `spend_reaction`, and the Strike damage function.

## Migration Plan

Spec only in this pull request. No changelog bullet and no archive. The implementation pull request adds `reactive.rs`, `Encounter::reactive_strike`, and the re-exports. It adds the Unreleased changelog bullet before it is done. `openspec archive add-reactive-strike --yes` is the last commit of that pull request. Rollback is reverting that pull request. There is no saved encounter to migrate.
