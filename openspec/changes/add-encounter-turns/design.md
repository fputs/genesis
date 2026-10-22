# Design

## Context

`crates/rules` already rolls checks, folds modifiers, derives a `Creature`, and stores actions and the reaction on `CombatantState` at 0 and not ready. `attack_check_total` adds the natural roll, the attack bonus, and three internal terms. Commit `d6da856` removed the caller-modifier list because no spec named it, and the reserved-source tests then called `attack_check_parts`. `damage_dealt` already applies the R-07 order. See proposal.md for why this change exists. The behavior to implement is `specs/encounter-turns/spec.md` and `specs/modifiers/spec.md`.

`crates/rules` stays dependency-free. Library code does not panic on bad input. This change names no file on `v1-reference`, so none is ported.

`dice-and-checks` names multiple-attack totals as scenario results. It does not name `attack_check_total`'s return type, so this change does not modify that spec. `attack_check_parts` stays. Its callers in `check.rs` and `modifiers.rs` keep compiling.

## Goals / Non-Goals

**Goals:**

- One new module, `crates/rules/src/encounter.rs`, re-exported from `lib.rs`.
- Initiative, rounds, the start and end steps, action accounting, and Strike as methods on `Encounter`.
- `attack_check_total` takes `&[Modifier]` again and returns `Result<i32, ModifierError>`. Scenario tests call that function.

**Non-Goals:**

- Dying values, wounded changes, unconscious, massive damage, and victory. Those are `add-dying-and-victory`.
- Wiring dying into the turn. `wire-dying-into-turns` owns the initiative move, the recovery check at start, temporary-Hit-Point ticks, outcome and victory checks, and rulings R-08 and R-10. This change only leaves the empty steps and the not-Alive turn seam.
- Gaining temporary Hit Points, choosing between two amounts (DEV-13), durations, expiry, and healing. R-06 applies here only as "damage removes temporary Hit Points first."
- A readied attack's frozen multiple attack penalty (R-05). A reaction check in this change is penalty 0.
- Delay, Ready, quickened, slowed, and stunned. An Alive creature regains exactly 3 actions.
- Joining or leaving mid-encounter.
- Initiative skills other than Perception (DEV-37).
- Dying or unconscious Armor Class reduction. Until `wire-dying-into-turns`, Strike uses the baked Armor Class (DEV-36).
- Stride and Step exist in grid-movement; spending encounter actions on them is out of scope here (callers spend actions via `spend_actions`). Reach, flanking, prone, cover, and shields. No call to `stride` or `step` from the encounter module. The diagonal counter stays with the caller of those functions.
- A callback list for the start and end steps. The steps are private, ordered, and empty.

## Decisions

### Module and state

`Encounter` owns the participants. Each participant is a `CombatantId`, a `Side` (`Pc` or `Enemy`), a `Creature`, and a `CombatantState`. `Side` is the only team type. In Phase 1 a faction is a `Side` (DEV-26). This module does not read `FactionId`.

The attack-trait count for the current turn lives on the encounter, keyed by `CombatantId`, as a `u8`. It is not a new field of `CombatantState`. The increment is `checked_add(1)`. A failure returns `AttackCountOverflow` and writes nothing, including drawing no die. Actions and `reaction_ready` stay the fields creature-template already defined. Inside the encounter module, `Encounter` methods are the only writers of actions, the reaction, the order, and Strike's Hit Points. No public free function in that module takes `&mut CombatantState` or `&mut Participant`. `apply_dice_damage` in `combatant.rs` stays public.

The order is a `Vec<CombatantId>`. Only the encounter mutates it. `order` returns a shared slice.

Alternative considered: store the attack count on `CombatantState`. Rejected. That amends the creature-template state for a count only this turn needs.

`begin_encounter` takes ownership of the participants and borrows the dice (`&mut impl Dice`). It rejects `DuplicateCombatant` and `EmptyRoster` before any die is drawn. It then rejects a Perception modifier that cannot be added to a d20 of 20, as `PerceptionOutOfRange`, also before any die is drawn. That gate is `checked_add(20)`, not `CheckModifier::new`. `CheckModifier::new` rejects a modifier that cannot be added to 255, which would refuse Perception 2147483627. A natural 20 on that modifier totals `i32::MAX` and is `Ok`. Perception 2147483628 does not fit and draws no die. `i32::MIN` fits in that gate. A die is drawn, and the total is the face plus `i32::MIN` (a face of 1 totals -2147483647). Every accepted face in 1..=20 fits in `i32` once the +20 gate has passed.

### Initiative

Draw one d20 per combatant in ascending `CombatantId` order, including a dead combatant. The total is the face plus the baked Perception modifier. Do not call `degree_of_success`. A natural 20 does not add 10.

Sort the order by higher total, then `Enemy` before `Pc`, then lower `CombatantId`. Side is the participant's `Side`, not `uses_dying_rules`.

Each combatant rolls its own check. Player Core allows the GM to roll identical creatures as a group or one by one. This change always rolls one by one. That is the book's individual option, not a deviation.

Tabletop substitute for same-Side ties. Rule: tied PCs choose who goes first, and the GM orders tied foes (Player Core pg. 435, Rules 2423). Why not: there are no players and no GM. Substitute: the lower `CombatantId` acts earlier. Intent: the same roster and the same dice always produce the same order. This is DEV-05, amended so the id order covers every same-Side tie, including two enemies. A PC–enemy tie is not this substitute. The enemy acts earlier, which is the book. Enemy `CombatantId(9)` before PC `CombatantId(1)` on equal totals is the case that fails if the sort uses the lower id alone.

### Start, end, and the action count

`start_of_turn_step(&mut self, id)` and `end_of_turn_step(&mut self, id)` are private and empty. `start_turn` and `end_turn` call them in that order: the start step, then the action regain; the end step, then dropping actions. `wire-dying-into-turns` fills the steps. Deferred work, not done here: duration ticks, the recovery check, fast healing, persistent damage at the end of the turn (Player Core pg. 436, Rules 2430), and frightened decreasing by 1 at the end of the turn.

The round is a `u32` starting at 1. `end_turn` moves the cursor one step. `start_turn` skips `Vital::Dead` without opening a turn. Whoever moves the cursor past the last index wraps it to the first index and increments the round with `checked_add(1)`. A failure returns `RoundOverflow` and leaves the round, the order, the cursor, and the open turn unchanged. The end step does not commit on that failure.

For order `[1, 2, 3]` with 3 dead, ending 2 leaves the cursor on 3 and the round at 1. The next start skips 3, wraps, increments the round, and opens 1 in round 2. For order `[1, 2]` with 1 dead at the start, the first start skips 1 and opens 2 in round 1.

`start_turn` on an Alive creature runs the empty start step, then sets actions to 3, the reaction to ready, and that creature's attack count to 0. The Alive start step rolls no die and does not change Hit Points, temporary Hit Points, or wounded. A second start while a turn is open returns `TurnInProgress`.

A creature that is not Alive does not act (Player Core pg. 436, Rules 2429). `Vital::Dead` is skipped and stays in the order. Any vital other than Alive or Dead still has a turn: `start_turn` opens it, the start step runs, actions are 0, and the reaction is not ready. `end_turn` runs the end step and closes it. The turn does not close inside `start_turn`. That empty open and empty close is the seam. The not-Alive state in tests comes from a 0 Hit Point knockout through Strike, not from naming a dying variant. `spend_actions`, `spend_free_action`, `spend_reaction`, and `strike` for that creature return `NotConscious` and do not change the reaction. If the active creature leaves Alive during its open turn, later spends by it return `NotConscious`, and `end_turn` still closes that turn once and runs the end step once.

`end_turn` is the only closer. `spend_actions`, `spend_free_action`, `spend_reaction`, and `strike` do not call it. Spending down to 0 actions leaves the turn open. `end_turn` runs the end step once, sets actions to 0, leaves an unspent reaction ready, and advances once. A second `end_turn` returns `TurnNotOpen` and does not advance again. `end_turn` with no open turn returns `TurnNotOpen`. The end step rolls no die. The next turn starts only when `start_turn` is called. If every combatant is dead, `start_turn` returns `NoLivingCombatant` and leaves the round and the order unchanged. There is no victory variant.

Tabletop substitute for the handoff. Rule: spending the last action ends the turn and the next creature's turn begins (Player Core pg. 436, Rules 2429). Why not: the spend cannot also play the next creature. Substitute: the caller must call `end_turn`. The spend leaves 0 actions and the turn open. Nothing further can be spent, so the handoff has no rules effect. No end-of-turn effect fires before `end_turn`. `end_turn` runs the empty end step once and advances once. The caller starts the next turn. Intent: the end step runs once, in `end_turn`, and not inside the spend.

Reactions of Alive creatures are ready after `begin_encounter`, before any start (DEV-06). A non-Alive creature's reaction is not ready. Rule: the GM decides whether reactions work before the first turn (Player Core pg. 436, Rules 2432). Why not: there is no GM. Substitute: an Alive combatant's reaction is ready at encounter start. Intent: one fixed answer. An Alive creature may spend it during another creature's turn. That spend does not roll, does not change the active creature's actions, and does not change the attack count. A dead creature, or any other creature that is not Alive, returns `NotConscious`, and the reaction stays as it was. A second spend by an Alive creature returns `NoReaction`.

`spend_actions` takes a `u8` cost. It accepts 1, 2, or 3, subtracts them at once, and returns `InsufficientActions` or `InvalidActionCost` without a partial spend. A free action during an open turn is an untriggered free action: it changes neither count and does not end the turn. An untriggered free action with no open turn returns `TurnNotOpen`. Triggered free actions are not in this change.

### Strike

`strike` checks, in order: a turn is open (`TurnNotOpen`); the attacker is the active id (`NotYourTurn`); the attacker is Alive (`NotConscious`); the attacker has at least one action (`InsufficientActions`); the target is known (`UnknownCombatant`); the target is not the attacker (`SelfTarget`); the target is not dead (`TargetDead`). Each failure rolls nothing and does not spend an action or increment the count. `TargetDead` spends no action.

It then requires `checked_add(1)` on the attack count. `AttackCountOverflow` draws no die. Otherwise it rolls one d20 and passes the profile's attack bonus, the current multiple attack penalty, a circumstance term of 0, a range term of 0, and the caller's attack-statistic modifiers to `attack_check_total`. `Err` becomes `AttackOverflow(ModifierError)` with that cause. The attack die stays drawn. No damage die is drawn. Actions and the count stay as they were.

Damage-statistic modifiers go to `damage_dealt`. Any other statistic is ignored, so a caller Armor Class penalty does not change the total or the baked Armor Class. There is no dying or unconscious Armor Class reduction (DEV-36).

Tabletop substitute for reach. Rule: a melee Strike targets a creature within reach (Player Core pg. 418, Strike). Why not: reach, flanking, and prone are `add-positioning`. Substitute: the caller names the target, and this module stores no squares. Intent: Strike is testable without the grid.

The degree after the natural 20 or natural 1 selects damage. `strike` does not take a `critical: bool`. It calls `damage_dealt` with `degree == CriticalSuccess`. Success rolls the profile's damage dice once and adds `damage_bonus`. Critical success doubles that total inside `damage_dealt`. Failure and critical failure roll no damage die. A natural 20 that is still a failure, because the total is 10 or more under Armor Class, deals no damage. Doubling rolls the dice once and doubles the total. The book's option to roll the dice twice and double the modifiers is not used (Player Core pg. 407, Rules 2307).

`StrikeProfile.damage_dice` is a `DieCount` (`u8`) and `damage_faces` is `Faces` (at most 255). The sum of that many faces fits in `i32` (at most 255 times 255). The sum still uses `checked_add`. Adding `damage_bonus` uses `checked_add`. A failure is `DamageOverflow(ModifierError::Overflow)`. `damage_dealt`'s `Err` is `DamageOverflow` of that `ModifierError`, so the cause is kept. Dice already drawn stay drawn. Actions, Hit Points, temporary Hit Points, and the attack count stay as they were.

`HpError` gains `NotAHit`. Resolving a blow whose degree is `Failure` or `CriticalFailure` returns `HpError::NotAHit` before any damage die is drawn, and it leaves Hit Points and the rest of the state unchanged. `add-dying-and-victory` references `HpError::NotAHit` and does not define its own miss error. `NotAHit` is the miss result, not an `EncounterError::HitPoints` failure: Strike still spends the action, reports that degree, and records damage 0.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HpError {
    Overflow,
    NotAHit,
}

pub fn apply_blow(
    state: &mut CombatantState,
    creature: &Creature,
    dice: &mut impl Dice,
    faces: Faces,
    bonus: i32,
    modifiers: &[Modifier],
    degree: DegreeOfSuccess,
) -> Result<(), HpError>;
```

On success, one `pub(crate)` helper applies the dealt amount to temporary Hit Points first and the remainder to current Hit Points, floored at 0 (R-06, the damage sentence only). It does not roll. It returns `Result<(), HpError>`. An overflow `Err` becomes `HitPoints(HpError)` with that cause, and the strike writes no Hit Points, spends no action, and does not increment the count. `apply_dice_damage` is left alone, including for a state whose temporary Hit Points are 0. Do not add a second damage formula. The Strike helper is interim. `add-dying-and-victory` reroutes Strike through its single blow path and removes `note_hit_points`. This change does not promise that `note_hit_points` survives. Do not choose a dying value, do not change wounded, do not move the initiative entry, and do not treat a blow of at least twice maximum Hit Points as its own result. Those belong to `add-dying-and-victory` and `wire-dying-into-turns`. Until `wire-dying-into-turns`, a Strike against a target whose vital is neither Alive nor Dead uses the baked Armor Class (DEV-36).

Damage follows `damage_dealt`: dice plus the baked bonus, then penalties, then the floor of 1 only when penalties reduce a pre-penalty total of at least 1, or a clamp at 0 otherwise, then doubling. R-07 and DEV-14 are this order. The smallest floored crit is 2. A clamped 0 stays 0 on a crit.

`attack_trait_penalty(&self, id, agility)` derives on-turn from the open turn: it is that creature's turn only when the open turn is `id`. The caller cannot pass a bool. `Agility` is `Agile` or `NotAgile`. The method builds `MapInput` and calls `multiple_attack_penalty`. It does not increment the count. Off that creature's turn the penalty is 0. It does not store a Ready snapshot. An unknown id is `UnknownCombatant`.

### The caller list on the attack total

```rust
pub fn attack_check_total(
    natural: u8,
    attack_bonus: AttackTotalTerm,
    map: AttackTotalTerm,
    circumstance_penalty: AttackTotalTerm,
    range_penalty: AttackTotalTerm,
    caller: &[Modifier],
) -> Result<i32, ModifierError>
```

Fold the internal terms and `caller` with `total`. The success value is the natural roll, plus the attack bonus, plus the attack-statistic total, plus the raw positive internal addend the function already uses for a positive multiple-attack or range argument. A positive untyped argument stays a raw addend so the existing boundary test still has a total. Zero internal arguments stay omitted. `Err` from `total`, or a sum that does not fit in `i32`, is `ModifierError::Overflow`. It is not the natural roll and it does not panic.

Alternative considered: keep the `i32` return and map a fold error to the natural roll, which is what the function did before `d6da856`. Rejected. A caller list can overflow, and returning the face would drop the modifiers. `Result` matches the modifiers overflow rule and AGENTS.md §7. An empty list still returns `Ok` with the totals dice-and-checks already specifies. Call sites in tests pass `&[]` and read the `Ok` value.

The scenario tests for the removed requirement call `attack_check_total`. They do not call `attack_check_parts`. The private helper remains, and its callers stay.

### Public shape

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Pc,
    Enemy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Agility {
    Agile,
    NotAgile,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Participant {
    pub id: CombatantId,
    pub side: Side,
    pub creature: Creature,
    pub state: CombatantState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StrikeProfile {
    pub attack_bonus: i32,
    pub agility: Agility,
    pub damage_dice: DieCount,
    pub damage_faces: Faces,
    pub damage_bonus: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncounterError {
    DuplicateCombatant,
    EmptyRoster,
    PerceptionOutOfRange,
    UnknownCombatant,
    TurnInProgress,
    TurnNotOpen,
    NotYourTurn,
    InsufficientActions,
    InvalidActionCost,
    NoReaction,
    NotConscious,
    TargetDead,
    SelfTarget,
    NoLivingCombatant,
    AttackCountOverflow,
    RoundOverflow,
    AttackOverflow(ModifierError),
    DamageOverflow(ModifierError),
    HitPoints(HpError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrikeOutcome {
    pub natural: u8,
    pub total: i32,
    pub degree: DegreeOfSuccess,
    pub map: i32,
    pub damage: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Encounter {
    // private fields
}

pub fn begin_encounter(
    participants: Vec<Participant>,
    dice: &mut impl Dice,
) -> Result<Encounter, EncounterError>;

impl Encounter {
    pub fn round(&self) -> u32;
    pub fn order(&self) -> &[CombatantId];
    pub fn turn_open(&self) -> Option<CombatantId>;
    pub fn state(&self, id: CombatantId) -> Result<&CombatantState, EncounterError>;
    pub fn creature(&self, id: CombatantId) -> Result<&Creature, EncounterError>;
    pub fn start_turn(&mut self) -> Result<CombatantId, EncounterError>;
    pub fn end_turn(&mut self) -> Result<(), EncounterError>;
    pub fn spend_actions(&mut self, id: CombatantId, n: u8) -> Result<(), EncounterError>;
    pub fn spend_free_action(&mut self, id: CombatantId) -> Result<(), EncounterError>;
    pub fn spend_reaction(&mut self, id: CombatantId) -> Result<(), EncounterError>;
    pub fn attack_trait_penalty(
        &self,
        id: CombatantId,
        agility: Agility,
    ) -> Result<i32, EncounterError>;
    pub fn strike(
        &mut self,
        attacker: CombatantId,
        target: CombatantId,
        profile: &StrikeProfile,
        modifiers: &[Modifier],
        dice: &mut impl Dice,
    ) -> Result<StrikeOutcome, EncounterError>;

    #[cfg(test)]
    pub(crate) fn set_round_for_test(&mut self, round: u32);
    #[cfg(test)]
    pub(crate) fn set_attack_count_for_test(&mut self, id: CombatantId, count: u8);
    #[cfg(test)]
    pub(crate) fn set_vital_for_test(&mut self, id: CombatantId, vital: Vital);
}
```

The three setters exist so tests can reach a round of `u32::MAX`, an attack count of 255, and an active creature that leaves Alive mid-turn. Each setter assigns that field. The round increment and the attack-count increment do not gain a `#[cfg(test)]` branch. The setters are not public. The mid-turn test copies the vital that a 0 Hit Point Strike produced, so the test does not name a dying variant.

Scenario errors map to those variants: duplicate id `DuplicateCombatant`; empty roster `EmptyRoster`; Perception 2147483628 `PerceptionOutOfRange`; a second start `TurnInProgress`; end with no open turn `TurnNotOpen`; a strike by someone other than the active id `NotYourTurn`; a cost of 0 or greater than 3 `InvalidActionCost`; a cost the creature cannot pay `InsufficientActions`; a Strike with 0 actions `InsufficientActions`; a second reaction `NoReaction`; a reaction, action, free action, or Strike for a non-Alive creature `NotConscious`; a dead target `TargetDead`; striking oneself `SelfTarget`; an id that is not in the encounter `UnknownCombatant`; every combatant dead `NoLivingCombatant`; a `u8` attack-count increment that does not fit `AttackCountOverflow`; a round increment that does not fit `RoundOverflow`; an attack-total `Err` `AttackOverflow` of that `ModifierError`; a damage sum or `damage_dealt` `Err` `DamageOverflow` of that `ModifierError`; the Hit Point helper's `Err` `HitPoints` of that `HpError`.

`round`, `order`, and `turn_open` are readable. `state` and `creature` are borrowed by id. Every public item gets a `///` comment citing the Player Core section when it states a rule. No `unwrap`, `expect`, `panic!`, or unchecked indexing outside tests. Sums use checked arithmetic.

## Risks / Trade-offs

- [Call sites of `attack_check_total` still expect `i32`] → Update them in the same change. An empty caller list keeps the existing success totals. The proof run is `cargo test --workspace`.
- [`damage_dealt` ignores temporary Hit Points] → The `pub(crate)` helper applies the dealt amount temporary-Hit-Points first. `apply_dice_damage` stays as it is, so a state at 0 temporary Hit Points still matches the creature-template tests.
- [A not-Alive creature still occupies a turn] → The turn is empty. `wire-dying-into-turns` fills the steps. This change does not give that creature 3 actions.
- [`add-dying-and-victory` owns massive damage and the initiative move] → This change does not specify those results, so that change does not inherit a test it must loosen.
- [The attack count is invisible if only `strike` exists] → `attack_trait_penalty` is the read. It derives the turn from the encounter and does not increment.

## Migration Plan

This is a library change with no stored data. Update every `attack_check_total` call to pass the caller list and handle `Result`. Rollback is reverting the implementation commit. The spec-only pull request does not add a changelog bullet and does not archive. The implementation pull request adds one Unreleased bullet, then `openspec archive add-encounter-turns --yes` as its last commit.

R-05, R-06, and R-07 are settled in the decisions above. Nothing in this change is waiting on a Designer answer.
