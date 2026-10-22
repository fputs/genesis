# Design

## Context

`Encounter::spend_reaction` already clears one ready reaction for an Alive combatant and returns `NotConscious` without writing when the vital is not Alive. `Encounter::strike` spends an action, reads the multiple attack penalty from the open turn, and increments the attack count. `in_reach` already answers reach. The reactor's melee reach is `ReachFeet` on the creature's strike. Squares are not stored on the encounter. `Vital::Dying` and `Vital::Unconscious` are named by `add-dying-and-victory` and are not in the crate yet. `apply_resolved_blow` is that change's damage path. Positioning names `ActionTrait::{Move}` and `ActionCost`. See proposal.md for why this change exists.

This change names no file on `v1-reference`, so none is ported. It does not edit `docs/designer/`.

## Goals / Non-Goals

**Goals:**

- One module, `crates/rules/src/reactive.rs`, for the trigger, the reach test, and the disruption result.
- `Encounter::reactive_strike` spends the reaction only through `spend_reaction`.
- The Strike uses penalty 0 and does not change the stored attack count, including off the reactor's own turn.
- A critical hit on a cost that includes `Manipulate` is `Disruption::Disrupted`. On the actor's own turn the action's cost stays spent and its effects are voided.
- Damage goes through `apply_resolved_blow`.

**Non-Goals:**

- Reading a path, a `SquareEvent`, or any square other than the actor's square at the moment of the action (proposed DEV-45). Grid-movement keeps emitting leave and enter events and still resolves no reaction. Follow-up: `wire-leave-square-into-reactive-strike`.
- Storing a square or a reach on `CombatantState` or `Encounter`.
- A class or a feat. Reactive Strike is granted by a class (e.g. Fighter, Player Core pg. 138). That absence is proposed DEV-44.
- Calling `Encounter::strike`. That function spends an action and applies the multiple attack penalty.
- Shield Block. This Strike does not take that argument and does not change shield Hit Points. Follow-up: `wire-shield-block-into-reactive-strike`.
- Hidden, concealed, and undetected targeting. The caller resolves those with `resolve_targeting` from seek-and-hidden (proposed DEV-46). Follow-up: `wire-targeting-into-reactive-strike`.
- Off-guard from just dropping prone. Armor Class is baked, so that circumstance penalty is not applied (DEV-01).
- Resolving Point Out, Stride, Drop Prone, Stand, Crawl, or Tumble Through. The caller passes the `ActionCost`.
- A second `ActionTrait` enum or a second `ActionCost`. `Manipulate` and `Ranged` are added to positioning's enum.
- A second damage function, or a second dying formula. `apply_resolved_blow` is the only creature-damage path.
- `set_reaction_ready_for_test`. The existing `set_vital_for_test` leaves the reaction as it was.
- Editing `docs/designer/rules-deviations.md` or `docs/designer/rules-rulings.md`.

## Decisions

### One call, caller squares

```rust
pub enum ActionTrait {
    Move,
    Manipulate,
    Ranged,
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

`ActionTrait` and `ActionCost` stay positioning's types. This change adds `Manipulate` and `Ranged` to that one enum. There is no `Activity` enum.

`Encounter::reactive_strike` takes the reactor id, the actor id, `ActionCost`, `Option<MoveAction>`, `ReactiveStrike`, both squares, both `Size` values, a `StrikeProfile`, the caller modifiers, and the dice. Reach is `creature.strike.reach_feet` for the reactor. There is no reach argument and no `bool`.

A cost triggers when its trait slice contains `Move`, `Manipulate`, or `Ranged`. Both `Move` and `Manipulate` on one cost are one triggering action, and a critical hit disrupts it because `Manipulate` is present. Interact is `ActionCost::Actions { count: 1, traits: &[ActionTrait::Manipulate] }`. Release is mapped to that manipulate classification and is `ActionCost::Free`: the book gives it the manipulate trait and says it does not trigger Reactive Strike, and `Free` has no trait slice. `ActionCost::Free` is `NotATrigger`. Seek is one action with none of those traits (concentrate only), which is the non-trigger. A melee Strike is that same non-trigger.

`Some(MoveAction::Step)` returns `NotATrigger` before the trait slice is read. Step's cost includes `Move` and still does not trigger (Actions 2304). `None`, `Some(Stride)`, and `Some(DropProne)` follow the traits. Stand is not a `MoveAction`. Its posture cost includes `Move`, so it triggers (FIX-06). Cite https://2e.aonprd.com/Actions.aspx?ID=2303 (Player Core pg. 418, Stand).

Alternative considered: keep an `Activity` enum. Rejected. The review uses positioning's `ActionCost` and `ActionTrait`.

Alternative considered: a `triggers: bool` on the cost. Rejected. Step is `MoveAction::Step`, which already exists. A flag would hide that.

Alternative considered: walk `MoveOutcome.events`. Rejected. That is the path case in proposed DEV-45. The encounter does not keep those events.

### Reach

`ReactiveError` uses `UnsupportedSize`, `Overflow`, and `InvalidReach`, copied from `PositioningError` on purpose. The order matches positioning's flanking checks: `UnsupportedSize`, then `Overflow`, then `InvalidReach`, then the boolean. Either creature not Small or Medium is `UnsupportedSize` (DEV-19). A distance that does not fit is `Overflow`, including when the reach is also not a multiple of 5. A reach that is not a multiple of 5 is `InvalidReach`. Reach 0 is a multiple of 5; `in_reach` then returns false and the error is `OutOfReach`. The same square is `OutOfReach`. False from `in_reach` is `OutOfReach`. Reach 10 includes the two-square diagonal `(2, 2)` from `(0, 0)` and does not include `(3, 0)`.

`in_reach` is the only distance function. A reach of 5, 10, or greater than 10 keeps that function's diagonal rule.

### Errors

`ReactiveError` is `UnknownCombatant`, `NotConscious`, `SelfTarget`, `Unavailable`, `NotATrigger`, `UnsupportedSize`, `Overflow`, `InvalidReach`, `OutOfReach`, `TargetNotConscious`, `NoReaction`, `AttackOverflow(ModifierError)`, `DamageOverflow(ModifierError)`, and `HitPoints(HpError)`. No string errors. No `TargetDead`. The encounter method returns `ReactiveError`. It does not add those variants to `EncounterError`.

The order is the same in this design, in the spec, and in tasks 7.1: `UnknownCombatant` for the reactor, then the actor, `NotConscious`, `SelfTarget`, `Unavailable`, `NotATrigger`, `UnsupportedSize`, `Overflow`, `InvalidReach`, `OutOfReach`, `TargetNotConscious`, then `NoReaction`. `NotConscious` is the reactor's `vital != Vital::Alive`. `TargetNotConscious` is the actor's `vital != Vital::Alive`, in the position `TargetDead` had, and it replaces `TargetDead`. `Unavailable` is `ReactiveStrike::Unavailable`. `NotATrigger` is Step, `ActionCost::Free` (Release), or a cost with none of the three traits. `NoReaction` is a reactor whose reaction is not ready.

An error before the d20 draws no attack die. `AttackOverflow` is the precheck that the attack bonus plus the caller modifiers plus 20 does not fit in `i32`, before the d20. `DamageOverflow` is the exception: the attack die and the damage die are both consumed, then the damage sum fails. Both leave the reaction ready and leave Hit Points unchanged.

Do not add `set_reaction_ready_for_test`. `begin_encounter` readies an Alive combatant. `set_vital_for_test` then assigns `Dying`, `Unconscious`, or `Dead` and leaves `reaction_ready` as it was (encounter.rs: that setter does not write the reaction). AGENTS.md §8 forbids a `cfg(test)` branch inside the procedure. A new setter is not required, and `reactive_strike` has no `cfg(test)` branch. The Dead-reactor scenario uses that existing seam so the reaction is still ready when the error is `NotConscious` rather than `NoReaction`.

### The Strike

Do not call `Encounter::strike`. Build the attack total with `attack_check_total`, penalty term 0, and the caller modifiers on the attack statistic only. Do not call `attack_trait_penalty`. Agile does not change the penalty. The degree is `degree_of_success` against the actor's baked Armor Class (DEV-01). A miss deals 0 and draws no damage die. A hit rolls damage once and calls `damage_dealt`, which doubles a critical success. Apply that amount with `apply_resolved_blow`. Do not copy that function and do not call `note_hit_points` from this module.

The stored attack count is not incremented. Actions are not changed. The turn is not ended. A Reactive Strike off the reactor's own turn is legal: no action spent, penalty 0, count unchanged. The two `encounter-turns` requirements say that. `Encounter::strike` still returns `NotYourTurn` off the active turn.

Resolve the Hit Point write on a copy of the actor's state. Call `spend_reaction` only after that copy succeeds. Then copy Hit Points, temporary Hit Points, and vital back. `spend_reaction` is what clears `reaction_ready`. Copying the whole state back would restore the reaction. On `spend_reaction` `Err`, leave the actor's Hit Points as they were.

`Disruption::Disrupted` is only `DegreeOfSuccess::CriticalSuccess` together with a cost that includes `Manipulate`. Every other `Ok` is `NotDisrupted`, including a critical hit on a move-only cost or on `Ranged`, and a natural 20 that improved only to success. When the actor's turn is open, that action's cost stays spent and its effects are voided. This function does not change the actor's action count on `Disrupted`.

Alternative considered: spend the reaction before the roll. Rejected. A later overflow would have spent it. The spec leaves the reaction ready on `AttackOverflow` and `DamageOverflow`.

### Proposed ledger rows

Do not edit `docs/designer/` in this change. Designer records these. The ids continue after DEV-43.

#### DEV-44 Granted by a class

**Rule**: Reactive Strike is granted by a class (e.g. Fighter, Player Core pg. 138). https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138).

**Why not**: Classes and feats are out of Phase 1.

**Substitute**: The caller passes `ReactiveStrike::Available` or `ReactiveStrike::Unavailable`. There is no class and no feat. `Unavailable` returns `Unavailable`, draws no die, and spends nothing.

**Intent**: The procedure runs for a creature the caller marks, and a creature who is not marked cannot react.

#### DEV-45 Leaving a square during a move

**Rule**: The trigger also includes a creature who leaves a square during a move action it is using. https://2e.aonprd.com/Actions.aspx?ID=2256 (Player Core pg. 138).

**Why not**: The encounter does not store squares, and this change does not track a path. Grid-movement emits leave and enter events for one Stride or Step and does not resolve a reaction. Positioning does not keep a square on the combatant.

**Substitute**: Only the actor's square at the moment of the action is tested. A move that starts outside reach is `OutOfReach`, with no path argument.

**Intent**: Stand, Drop Prone, Stride, and Crawl still trigger when that square is already inside reach. The path case is `wire-leave-square-into-reactive-strike`.

#### DEV-46 Hidden, concealed, and undetected

**Rule**: Targeting a concealed, hidden, or undetected creature uses that condition's flat check. https://2e.aonprd.com/Conditions.aspx?ID=62 (Player Core pg. 442, Concealed). https://2e.aonprd.com/Conditions.aspx?ID=79 (Player Core pg. 444, Hidden). https://2e.aonprd.com/Conditions.aspx?ID=96 (Player Core pg. 447, Undetected).

**Why not**: seek-and-hidden owns `resolve_targeting`. This change does not roll a flat check and does not read a detection condition.

**Substitute**: The caller resolves targeting before this call. A creature that is not a legal target never reaches `reactive_strike`.

**Intent**: The melee reaction stays the reach and trait check. The follow-up is `wire-targeting-into-reactive-strike`.

DEV-01, DEV-19, and R-05 are cited, not restated. FIX-06, FIX-07, and FIX-08 are the trigger this change follows. DEV-36 stays on the encounter-turns Strike sentence this change does not rewrite: that interim Armor Class belongs to `wire-dying-into-turns`. This Strike's own baked Armor Class is DEV-01.

## Risks / Trade-offs

- [A move through reach is missed] → Proposed DEV-45. The in-reach Move scenario still covers Stand and Drop Prone. The follow-up is named above.
- [Baked Armor Class ignores off-guard and a raised shield] → Armor Class is a baked number (DEV-01). Off-guard from just dropping prone is not applied. The caller can pass an attack modifier. Shield Block is the named follow-up, not this call.
- [`Vital::Dying` is not in the crate yet] → Implementation waits until dying-and-victory's `Vital::Dying` and `Vital::Unconscious` are public, and until `apply_resolved_blow` exists. `NotConscious` and `TargetNotConscious` stay `vital != Alive`.
- [The Hit Point copy and the reaction spend can diverge] → The spend runs only after the copy succeeds, and only Hit Points, temporary Hit Points, and vital are copied back.
- [Positioning is not an archived spec] → This change does not emit a positioning MODIFIED header. It extends the one `ActionTrait` when that type is in the crate.

## Migration Plan

Spec only in this pull request. No changelog bullet and no archive. The implementation pull request adds `reactive.rs`, `Encounter::reactive_strike`, the `ActionTrait` variants, and the re-exports. It adds the Unreleased changelog bullet before it is done. `openspec archive add-reactive-strike --yes` is the last commit of that pull request. Rollback is reverting that pull request. There is no saved encounter to migrate.
