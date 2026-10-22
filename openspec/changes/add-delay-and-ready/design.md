# Design

## Context

`Encounter` owns the order, the cursor, the round, the open turn, and each member's `attack_count`. `start_turn` grants 3 actions and one reaction to an Alive creature and sets that creature's attack count to 0. That start does not change Hit Points. `end_turn` drops actions and keeps an unspent reaction. `spend_reaction` returns `NotConscious` when vital is not `Alive`, then `NoReaction`. `MapInput.readied_attack_count` is on main (`0185586`). `attack_trait_penalty` passes `None` and returns 0 off the open turn. `Encounter::strike` spends an action, and `ShieldBlock::Requested` is checked before any die. See proposal.md. The behavior is `specs/delay/spec.md`, `specs/ready/spec.md`, and `specs/encounter-turns/spec.md`.

`dying::start_of_turn` is not called from `start_turn`. This change does not call it. No file from `v1-reference` is ported. This change does not edit `docs/designer/`.

## Goals / Non-Goals

**Goals:**

- One `TurnSlot` on each member. No `delayed` bool, no `resume` bool, and no `return_open` bool.
- Read-only accessors for the stored ready, the frozen count, the slot, the next start, and the initiative total.
- A readied Strike calls `multiple_attack_penalty` with `readied_attack_count: Some(snapshot)` and does not write the count.
- Every reaction precheck returns `Delayed` after `NotConscious` and before any action or die.

**Non-Goals:**

- A trigger outside the two kinds, or a readied action outside Strike, Stride, Step, and Dismiss.
- A flat check. Hidden and undetected stay the caller's job.
- Adding persistent damage or frightened. When those effects exist, they fire on Delay. Phase 1 adds no end-of-turn step.
- A second penalty table. The count field is already on `MapInput`.

## Decisions

### 1. One slot enum

```rust
pub enum TurnSlot {
    Normal,
    Delayed,
    Returned,
    SkipStartHook,
}
```

`Returned` is the open turn a return produced. Delay on that turn is `TriggerNotMet` (R-C): the trigger is "Your turn begins," and return continues the same turn. `start_turn` sets `Normal`. There is no `return_open` field. `last_ended: Option<CombatantId>` is set by `end_turn` and cleared by a successful `delay`. Return requires `last_ended` to be `Some`.

`end_turn` of the last creature does not increment the round while any member is `Delayed`. Return then inserts after `last_ended` and opens in the current round (R-B). The round increments when the last of those returned turns ends, and the next start is the creature at the top. `start_turn` with nobody returning applies that pending wrap, then opens the original slot. A Dying creature opens with 0 actions and no reaction. A Dead creature is skipped and the slot becomes `Normal`.

Return sorts enemies before PCs, then lower `CombatantId` (DEV-52, DEV-05). The first opens with 3 actions. `Returned` is that id. Each later id is `SkipStartHook`. Return does not write the reaction, the attack count, or temporary Hit Points. `SkipStartHook` on a later `start_turn` grants 3 actions and a reaction, does not write the attack count, and does not run the start step.

```rust
pub fn delay(&mut self, id: CombatantId) -> Result<(), EncounterError>
pub fn return_from_delay(&mut self, ids: &[CombatantId]) -> Result<CombatantId, EncounterError>
```

`delay` checks: `UnknownCombatant`, `TurnNotOpen`, `NotYourTurn`, `NotConscious`, `TriggerNotMet`, `RoundOverflow`. `TriggerNotMet` is also actions not 3, attack count not 0, or the reaction not ready. Success sets actions to 0, sets `Delayed`, closes the turn, clears `last_ended`, and advances like `end_turn`.

`return_from_delay` checks: empty `NoCreature`, duplicate `DuplicateCombatant`, `UnknownCombatant`, `TurnInProgress`, `NotDelayed`, `NotConscious`, `TriggerNotMet`.

`spend_reaction` checks `NotConscious`, then `Delayed`, then `NoReaction`. `Delayed` does not clear `reaction_ready`. `strike` with `ShieldBlock::Requested` uses that same `NotConscious` then `Delayed` gate before `block_is_ready`'s shield errors, before the attacker's action is spent, and before any die. `reactive_strike` and `aid` call the same gate before their d20. This change does not invent those two functions. The gate is `require_reaction`.

Read-only accessors, named for the scenarios:

```rust
pub fn stored_ready(&self, id: CombatantId) -> Result<Option<StoredReady>, EncounterError>
pub fn frozen_attack_count(&self, id: CombatantId) -> Result<Option<u8>, EncounterError>
pub fn turn_slot(&self, id: CombatantId) -> Result<TurnSlot, EncounterError>
pub fn next_start(&self) -> Result<CombatantId, EncounterError>
pub fn initiative_total(&self, id: CombatantId) -> Result<i32, EncounterError>
```

`round` already exists. `next_start` does not mutate. `StoredReady` holds the trigger, the action, and the frozen count.

### 2. Ready takes a real cost

Types live in `crates/rules/src/ready.rs`. Encounter methods stay on `Encounter`.

```rust
pub enum ReadyActivity {
    Strike { target: CombatantId, profile: StrikeProfile },
    Stride,
    Step,
    Dismiss,
    Delay,
    ReturnFromDelay,
}

pub fn ready(
    &mut self,
    actor: CombatantId,
    trigger: ReadyTrigger,
    cost: ActionCost,
    activity: ReadyActivity,
) -> Result<(), ReadyError>
```

There is no `SubmittedAction::Activity` and no `TriggeredFree`. A cost whose count is not 1, and not `Free` for Dismiss, is `NotASingleAction`. `Delay` and `ReturnFromDelay` are `TriggeredFreeAction`. A Strike whose target is not the trigger subject is `NotTheTriggerSubject`. `ready_cost` remains two actions with `ActionTrait::Concentrate`. Do not change `ActionCost::Free` or `dismiss_cost`.

`ReadyTrigger::MovesWithinReach` fires only when the entered square is in reach and the square left was not. `Begins` with `action: None` is Seek or any other unnamed action.

`fire_strike` on a `Begins` trigger calls `in_reach` for a melee weapon and the grid distance against the weapon's range increment for a ranged weapon. Failure is `OutOfReach` and spends nothing. A readied Strike does not call `resolve_targeting` and does not draw a flat-check die.

Matching readies on one offer resolve in encounter order, each fully before the next (DEV-55). That order already places an enemy before a PC on a tied total (DEV-05). One call returns each outcome. A later `TargetDead` does not roll back the earlier kill.

If the readied Strike leaves the triggering creature Dying or Dead, the caller's following `strike` for that creature returns `NotConscious`, spends no action, and draws no die. The readied result stays.

```rust
pub enum ReadyError {
    UnknownCombatant,
    TurnNotOpen,
    NotYourTurn,
    NotConscious,
    NotASingleAction,
    TriggeredFreeAction,
    TriggerSubjectIsSelf,
    NotTheTriggerSubject,
    InsufficientActions,
    RoundOverflow,
    NotReadied,
    WrongAction,
    Delayed,
    TriggerMismatch,
    OutOfReach,
    TargetDead,
    NoReaction,
    Grid(GridError),
    Positioning(PositioningError),
    AttackOverflow(ModifierError),
    DamageOverflow(ModifierError),
    HitPoints(HpError),
}
```

`fire_dismiss` maps `positioning::dismiss`'s `Err` to `ReadyError::Positioning`. It does not unwrap. That check is after the trigger matches and before the reaction is spent. `Grid` for an illegal path is in that same stretch. `AttackOverflow` is before the d20.

`readied_attack_penalty` calls `multiple_attack_penalty` with `on_own_turn: false`, prior 0, agile from the stored profile, and `readied_attack_count: Some(snapshot)`. It does not write the count and does not roll. `attack_trait_penalty` stays the non-readied reader.

A readied move uses `DiagonalCounter::fresh()`. `start_turn` clears that creature's ready after a successful round increment. `SkipStartHook` and `Returned` do not clear another creature's ready.

The count-2 case uses `set_attack_count_for_test`. One `#[cfg(test)]` setter may store a ready on a creature that is already `Delayed`, so the `WrongAction` then `Delayed` order can be asserted. It is not a branch inside `ready` or `fire`. Do not add `set_reaction_ready_for_test`.

### 3. Proposed ledger rows

Main ends at DEV-43, including DEV-38 through DEV-43. `add-reactive-strike` proposes DEV-44 and DEV-45 (and DEV-46 for hidden targeting). `add-aid` proposes DEV-50 and DEV-51. This change uses DEV-52, DEV-53, DEV-54, and DEV-55. It does not edit the ledger file.

| ID | Rule | RAW citation | Our behavior | Reason | Owning change |
|---|---|---|---|---|---|
| DEV-52 | Several creatures return together | https://2e.aonprd.com/Actions.aspx?ID=2294 (PC 416). The book does not order them. | Enemies before PCs, then lower `CombatantId` (DEV-05, Rules 2423). | Replay would otherwise follow call order. | 11 |
| DEV-53 | Ready's trigger | https://2e.aonprd.com/Actions.aspx?ID=2299 (PC 417). The creature names a trigger. | `MovesWithinReach` from outside reach, or `BeginsAction` of Strike, Stride, Step, Drop Prone, or Stand. | Those are the observations Phase 1 already has. | 12 |
| DEV-54 | Ready's action | Same page. One single action or free action. | Strike, Stride, Step, or Dismiss. The Strike's target is the trigger subject. The cost is `ActionCost`. | Phase 1 can resolve those four. | 12 |
| DEV-55 | Several reactions on one trigger | The book does not order reactions declared on the same trigger. | Current initiative order, each fully before the next. Ties use DEV-05. | Two readied Strikes must not depend on call order. | 12 |

### 4. Error enums on the encounter

`EncounterError` gains `NoCreature`, `NotDelayed`, `TriggerNotMet`, and `Delayed`. No variant is a string. Library paths return these. They do not panic.

## Risks / Trade-offs

- [`reactive_strike` and `aid` are not in this crate yet] → `require_reaction` is the gate. `strike`'s Shield Block path calls it now. Those two functions call it before their d20 once they exist. This change does not stub them.
- [Two Strikes and then Ready do not fit in 3 actions] → Snapshot 2 uses `set_attack_count_for_test`. Reachable play is snapshot 0 and snapshot 1.
- [Adding `Concentrate` touches `ActionTrait`] → Add `Concentrate` only. Do not edit `Free`.
- [A later change wires the temporary Hit Point tick] → Return and `SkipStartHook` still skip it. The missed arrival ticks once. Return never ticks.

## Migration Plan

This pull request adds only `openspec/changes/add-delay-and-ready/`. No changelog bullet and no archive. The implementation pull request adds `ready.rs`, `TurnSlot`, the accessors, and `EncounterError::Delayed` on the reaction prechecks. It adds the Unreleased changelog bullet before it is done. The last commit of that pull request has the subject `openspec archive add-delay-and-ready --yes`. Rollback is reverting that pull request.

## Open questions for Designer

R-A through R-G are settled in the specs. Still open, and not blocking this text:

1. Ready after the reaction was already spent. Chosen: Ready succeeds. Fire with no reaction is `NoReaction`, and the ready stays until the next start.
2. Another creature's Delay is not "the end of their turn." Return's trigger is `end_turn`.
3. A reaction spent after `start_turn` and before `delay` is `TriggerNotMet`.
4. Strike twice, then Ready, is 4 actions. The count-2 scenarios set the count. No fourth action.
5. A Step into reach from outside satisfies `MovesWithinReach`. A Step between two squares already in reach does not.
