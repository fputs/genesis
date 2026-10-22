# Design

## Context

`Encounter` in `crates/rules/src/encounter.rs` owns the order, the cursor, the round, the open turn, and each member's `attack_count`. `start_turn` grants 3 actions and one reaction to an Alive creature and sets that creature's attack count to 0. Its start step does not change Hit Points or temporary Hit Points. `end_turn` drops actions, keeps an unspent reaction, and advances the cursor, wrapping the round with `RoundOverflow` before it writes. `spend_reaction` returns `NotConscious` when vital is not `Alive`, then `NoReaction`. `attack_trait_penalty` passes `readied_attack_count: None` once that field exists and returns 0 off the open turn. `Encounter::strike` spends an action and increments the count. See proposal.md for why Delay and Ready are in this change. The behavior to implement is `specs/delay/spec.md`, `specs/ready/spec.md`, and `specs/encounter-turns/spec.md`.

`MapInput.readied_attack_count` is specified by `amend-readied-attack-map` and is not on `MapInput` in this tree yet. This change does not add that field and does not add a second penalty match. `dying::start_of_turn` ticks temporary Hit Points and is not called from `Encounter::start_turn`. This change does not call it. No file from `v1-reference` is named, so none is ported. This change does not edit `docs/designer/`.

## Goals / Non-Goals

**Goals:**

- `Encounter::delay` and `Encounter::return_from_delay` move a creature in the order without a new crate.
- One stored ready per creature: trigger, action, and the attack count at resolution.
- A readied Strike calls `multiple_attack_penalty` with `readied_attack_count: Some(snapshot)`.
- `spend_reaction` returns `Delayed` while that creature is out of the order and does not clear `reaction_ready`.

**Non-Goals:**

- A general trigger string, a readied action outside the four, or a Strike against someone other than the trigger subject.
- Walking a path to decide reach. The caller passes one entered square.
- Persistent damage, frightened, slowed, stunned, or a condition that blocks concentrate.
- Wiring `dying::start_of_turn` into the encounter. DEV-32 stays in dying-and-victory.
- Adding `MapInput.readied_attack_count`. That field belongs to `amend-readied-attack-map`.
- A changelog bullet or an archive commit in the spec-only pull request.

## Decisions

### 1. Delay state lives on the encounter

`Member` gains `delayed: bool` and `resume: Resume`, both private. `Resume` is `Normal` or `SkipStartHook`. It is not a parameter of `start_turn`. `Encounter` also stores `last_ended: Option<CombatantId>` and `return_open: bool`. `end_turn` sets `last_ended` to the creature who just ended and sets `return_open` to true. `delay` sets `return_open` to false and does not change `last_ended`. A return before some other creature's later `end_turn` is `TriggerNotMet`.

```rust
pub fn delay(&mut self, id: CombatantId) -> Result<(), EncounterError>
pub fn return_from_delay(&mut self, ids: &[CombatantId]) -> Result<CombatantId, EncounterError>
```

`delay` checks, and writes nothing on any error: `UnknownCombatant`, `TurnNotOpen`, `NotYourTurn`, `NotConscious`, `TriggerNotMet`, `RoundOverflow`. `TriggerNotMet` is actions not 3, attack count not 0, or the reaction not ready. On success it sets actions to 0, sets `delayed`, closes the turn, and advances the cursor with the same wrap as `end_turn`. It spends no action and no reaction. A wrap past `u32::MAX` is `RoundOverflow` before any write.

`return_from_delay` checks: empty slice `NoCreature`, a repeated id `DuplicateCombatant`, then each id as `UnknownCombatant`, then `TurnInProgress`, then `NotDelayed`, then `NotConscious`, then `TriggerNotMet` when `return_open` is false. It sorts a copy by `CombatantId` ascending (DEV-52). It removes those ids and inserts them, in that sorted order, immediately after `last_ended`. The lowest id's turn opens with 3 actions. The reaction flag and the attack count are not written. `Resume` for that id stays `Normal` because this open is not `start_turn`. Every later id gets `Resume::SkipStartHook`. The returned id is the one whose turn opened.

`start_turn` of a `delayed` creature clears `delayed` and opens the ordinary start. That is the missed return: the delayed actions stay lost, and this arrival is the next turn at the old index. `start_turn` of `SkipStartHook` grants 3 actions and a reaction, clears the flag, and does not run the start step again. It does not write the attack count. `RoundOverflow` still returns before that open.

New `EncounterError` variants, in the order above: `NoCreature`, `NotDelayed`, `TriggerNotMet`, `Delayed`. `spend_reaction` checks `NotConscious`, then `Delayed`, then `NoReaction`. `Delayed` does not clear `reaction_ready`.

Alternative considered: skip the delayed creature when the cursor reaches them, so the missed slot is not a turn. Rejected. The book says the delayed actions are lost and the creature's next turn is the one at the original place. The arrival plays.

Alternative considered: spend the reaction to return, which is what v1 did. Rejected. Player Core pg. 416 says the return is a free action, and FIX-04 records that v1 was wrong.

### 2. Ready stores a count, not a penalty

Types live in `crates/rules/src/ready.rs`. Methods that mutate the encounter are on `Encounter`, which stays the only writer of order, actions, the reaction, and Strike Hit Points.

```rust
pub enum ReadyTrigger {
    MovesWithinReach { subject: CombatantId },
    BeginsAction { subject: CombatantId, action: ObservedAction },
}

pub enum ObservedAction {
    Strike,
    Stride,
    Step,
    DropProne,
    Stand,
}

pub enum ReadiedAction {
    Strike { profile: StrikeProfile },
    Stride,
    Step,
    Dismiss,
}

pub enum SubmittedAction {
    Strike { target: CombatantId, profile: StrikeProfile },
    Stride,
    Step,
    Dismiss,
    Activity { cost: u8 },
    TriggeredFree,
}

pub enum ReadyOffer {
    Entered {
        subject: CombatantId,
        readier_square: Square,
        subject_square: Square,
        readier_size: Size,
        subject_size: Size,
        reach_feet: u32,
    },
    Begins {
        subject: CombatantId,
        action: Option<ObservedAction>,
    },
}
```

`SubmittedAction::Strike` carries the target, which must equal the trigger subject, and the profile that is stored. `Activity` and `TriggeredFree` exist so the rejection scenarios can be called. They are never stored. `ReadyOffer::Begins` with `action: None` is an action outside the five, including Seek. Fire methods:

```rust
pub fn ready(
    &mut self,
    actor: CombatantId,
    trigger: ReadyTrigger,
    action: SubmittedAction,
) -> Result<(), ReadyError>

pub fn fire_strike(
    &mut self,
    readier: CombatantId,
    offer: ReadyOffer,
    modifiers: &[Modifier],
    dice: &mut impl Dice,
) -> Result<StrikeOutcome, ReadyError>

pub fn fire_stride(
    &mut self,
    readier: CombatantId,
    offer: ReadyOffer,
    movement: MoveInput<'_>,
) -> Result<MoveOutcome, ReadyError>

pub fn fire_step(
    &mut self,
    readier: CombatantId,
    offer: ReadyOffer,
    movement: MoveInput<'_>,
) -> Result<MoveOutcome, ReadyError>

pub fn fire_dismiss(
    &mut self,
    readier: CombatantId,
    offer: ReadyOffer,
    stored: Option<CoverBenefit>,
) -> Result<Option<CoverBenefit>, ReadyError>

pub fn readied_attack_penalty(&self, id: CombatantId) -> Result<i32, ReadyError>

pub fn ready_cost() -> ActionCost
```

`ready_cost` returns `ActionCost::Actions { count: 2, traits: &[ActionTrait::Concentrate] }`. Add `ActionTrait::Concentrate` beside `Move`. Do not change `ActionCost::Free` or `dismiss_cost`. Reactive Strike adds `Manipulate` and `Ranged` to the same enum in its own change. This change adds `Concentrate` only.

The stored record is the trigger, the `ReadiedAction`, and `attack_count: u8` copied from the member when `ready` succeeds. The Strike profile is stored then, so agile is the readied weapon (R-05). `ready` then ends the turn with the same cursor and round rules as `end_turn`. It does not spend the reaction and does not write the attack count.

`ready` checks: unknown actor, unknown subject, `TurnNotOpen`, `NotYourTurn`, `NotConscious`, `NotASingleAction`, `TriggeredFreeAction`, `TriggerSubjectIsSelf`, `NotTheTriggerSubject`, `InsufficientActions`, `RoundOverflow`. A Strike uses `SubmittedAction::Strike.target` for `NotTheTriggerSubject`. Every action uses the trigger subject for the unknown-subject check.

`fire_*` checks: unknown readier, unknown offer subject, `NotReadied`, `WrongAction`, `Delayed`, `TriggerMismatch`, `NotConscious`, `TargetDead` for a Strike, `NoReaction`. A reach trigger calls `in_reach` on the one entered square. `Ok(false)`, a different subject, a different observed action, or `Begins { action: None }` is `TriggerMismatch`. `in_reach`'s `Err` is `Grid` and is ordered with the path errors. An illegal path, or a missing path, is `Grid` before the reaction is spent. `AttackOverflow` is before the d20. A miss does not call `apply_resolved_blow`. A hit calls it and maps `HpError` to `HitPoints`. `DamageOverflow` is after the attack die and before `spend_reaction`, and the ready stays. Success, including a miss, spends the reaction and clears the ready. Fire does not call `Encounter::strike`, does not call `attack_trait_penalty`, does not spend an action, and does not change either count.

`readied_attack_penalty` calls `multiple_attack_penalty` with `on_own_turn: false`, `prior_attack_trait_checks_this_turn: 0`, `current_weapon_agile` from the stored profile, and `readied_attack_count: Some(snapshot)`. It rolls nothing. A stored Stride, Step, or Dismiss is `WrongAction`. The encounter-turns scenario "A reaction after two Strikes keeps penalty 0" keeps calling `attack_trait_penalty`, which still passes `None`.

A readied Stride or Step builds `DiagonalCounter::fresh()` and does not take the caller's counter. Dismiss calls `positioning::dismiss`.

`start_turn` clears that creature's stored ready after the round increment has succeeded, including a missed-delay arrival. `SkipStartHook` does not clear it again. Return does not clear another creature's ready and does not write another creature's snapshot. A Dead creature is skipped by `start_turn`, so a ready on a Dead creature stays until fire, which returns `NotConscious`.

The count-2 ready and the delayed creature who also has a ready are not reachable through the public action economy. Tests use the existing `set_attack_count_for_test` for the count. One `#[cfg(test)]` setter stores a ready on an already delayed creature so the `WrongAction` and `Delayed` order can be asserted. It is not a branch in `ready` or `fire`. Do not add `set_reaction_ready_for_test`.

Alternative considered: store the penalty number at Ready time. Rejected. R-05 says the snapshot is the count, because the weapon used later sets agile.

Alternative considered: reject Ready when the reaction is already spent. Rejected. The book spends the reaction when the trigger fires. Fire returns `NoReaction` and leaves the ready stored.

### 3. Proposed ledger rows

These rows are proposed here. This change does not edit `docs/designer/rules-deviations.md`. The file on main ends at DEV-43. `add-reactive-strike` reserves DEV-44 through DEV-49. `add-aid` uses DEV-50 and DEV-51. The next free ids are DEV-52, DEV-53, and DEV-54.

| ID | Rule | RAW citation | Our behavior | Reason | Owning change |
|---|---|---|---|---|---|
| DEV-52 | Several creatures return together | https://2e.aonprd.com/Actions.aspx?ID=2294 (PC 416). The book does not order them. | One call takes the ids and sorts them. The lowest `CombatantId` acts first. | Replay would otherwise follow call order. | 11 |
| DEV-53 | Ready's trigger | https://2e.aonprd.com/Actions.aspx?ID=2299 (PC 417). The creature names a trigger. | `MovesWithinReach` of one entered square, or `BeginsAction` of Strike, Stride, Step, Drop Prone, or Stand. | Those are the observations Phase 1 already has. | 12 |
| DEV-54 | Ready's action | Same page. One single action or free action the creature can use. | Strike, Stride, Step, or Dismiss. The Strike's target is the trigger subject. | Phase 1 can resolve those four. | 12 |

### 4. Error enums

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
    TargetDead,
    NoReaction,
    Grid(GridError),
    AttackOverflow(ModifierError),
    DamageOverflow(ModifierError),
    HitPoints(HpError),
}
```

`EncounterError` gains `NoCreature`, `NotDelayed`, `TriggerNotMet`, and `Delayed`. No variant is a string. Library paths return these. They do not panic.

## Risks / Trade-offs

- [`MapInput` has no `readied_attack_count` until amend-readied-attack-map is implemented] → This change does not add the field. The ready implementation calls it with `Some(snapshot)` and stops if the field is absent, rather than growing a second table.
- [The public start clears a ready before Delay, so a delayed readier with a stored ready cannot be built by `ready` then `delay`] → The order scenario uses the test setter described above. Production `fire` still checks `Delayed` after `WrongAction`.
- [Two Strikes and then Ready do not fit in 3 actions] → The count-2 scenarios set the count with `set_attack_count_for_test`. Reachable play is one Strike then Ready, snapshot 1, penalty -5, and Ready first, snapshot 0.
- [Adding `Concentrate` touches `ActionTrait` while Reactive Strike adds other variants] → Add `Concentrate` only. Do not edit `Free`.
- [A later change wires the temporary Hit Point tick into `start_turn`] → Return and `SkipStartHook` must still skip that hook. The missed arrival must run the ordinary start once. This change's numbers follow today's start, which changes no Hit Points.

## Migration Plan

This pull request adds only `openspec/changes/add-delay-and-ready/`. No changelog bullet and no archive. The implementation pull request adds `ready.rs`, the encounter fields and methods, `ActionTrait::Concentrate`, and `EncounterError::Delayed` on `spend_reaction`. It adds the Unreleased changelog bullet before it is done. The last commit of that pull request has the subject `openspec archive add-delay-and-ready --yes`. Rollback is reverting that pull request. There is no saved encounter to migrate.

## Open questions for Designer

These readings are settled in the specs so implementation can proceed. Each one uses the Remaster text cited. A different call is a behavior change for Chief of Staff.

1. A second Delay on the turn return just opened. The trigger is "Your turn begins" (Player Core pg. 416, Actions 2294). Return opens that window with 3 actions, attack count 0, and the reaction ready, and the fetched text does not forbid Delay again. Chosen: the second Delay is legal.
2. A creature that never returns. Chosen: when its original slot comes up, that arrival is the next turn, including the ordinary start. The delayed actions are not banked. Skipping the arrival entirely is the reading not taken.
3. A reaction spent after `start_turn` and before `delay`. Chosen: `TriggerNotMet`. The trigger is the beginning of the turn, and the start's reaction is no longer unused.
4. Temporary Hit Points. `dying::start_of_turn` ticks them (DEV-32). `Encounter::start_turn` does not call it, and encounter-turns says the Alive start changes no temporary Hit Points. Chosen: this change adds no tick. Return does not tick. The missed arrival uses that same start and also does not tick. Wiring the tick is a later change, and it must not tick return.
5. Reactions while delayed. The book says you can't use reactions until you return (Player Core pg. 416). The start regains the reaction before Delay (Player Core pg. 435, Rules 2428). Chosen: `reaction_ready` stays true and `spend_reaction` returns `Delayed` without clearing it. Letting the spend succeed is the reading not taken.
6. Strike twice, then Ready, in one turn. That is 4 actions. B-06's -10 case is specified by a stored count of 2. Chosen: no fourth action. The reachable penalties are 0 and -5.
7. Ready after the reaction was already spent. The book spends the reaction at the trigger, not at Ready. Chosen: Ready succeeds. Fire with no reaction is `NoReaction`, and the ready stays until the next start.
8. Whether another creature's Delay is "the end of their turn" for a return. Chosen: no. The return trigger is `end_turn` of another creature. The scenarios use that.
9. A Dead readier. `start_turn` skips Dead, so expiry does not run. Chosen: fire returns `NotConscious` and leaves the ready. This change adds no death hook.
10. Two creatures returning at the same end. The book does not order them. Chosen: DEV-52, lowest `CombatantId` first, one call so the caller does not pick the order.
11. A Step into reach. Step does not provoke Reactive Strike. It is still movement. Chosen: it satisfies `MovesWithinReach`. Reactive Strike's own non-trigger stays in that spec.
