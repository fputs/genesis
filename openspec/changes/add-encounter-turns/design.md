# Design

## Context

`crates/rules` already rolls checks, folds modifiers, derives a `Creature`, and stores actions and the reaction on `CombatantState` at 0 and not ready. `attack_check_total` adds the natural roll, the attack bonus, and three internal terms. Commit `d6da856` removed the caller-modifier list because no spec named it, and the reserved-source tests then called `attack_check_parts`. `damage_dealt` already applies the R-07 order. See proposal.md for why this change exists. The behavior to implement is `specs/encounter-turns/spec.md` and `specs/modifiers/spec.md`.

`crates/rules` stays dependency-free. Library code does not panic on bad input. This change names no file on `v1-reference`, so none is ported.

## Goals / Non-Goals

**Goals:**

- One new module, `crates/rules/src/encounter.rs`, re-exported from `lib.rs`.
- Initiative, rounds, the start and end steps, action accounting, and Strike as public functions on an encounter value.
- `attack_check_total` takes `&[Modifier]` again and returns `Result<i32, ModifierError>`. Scenario tests call that function.

**Non-Goals:**

- Dying values, wounded changes, unconscious, recovery checks, massive damage, and victory. Those are `add-dying-and-victory`.
- Gaining temporary Hit Points, choosing between two amounts (DEV-13), durations, expiry, and healing. R-06 applies here only as "damage removes temporary Hit Points first."
- A readied attack's frozen multiple attack penalty (R-05). A reaction check in this change is penalty 0.
- Stride, Step, reach, flanking, prone, cover, and shields. No call to `stride` or `step`. The diagonal counter stays with the caller of those functions.
- A callback list for the start and end steps. The steps are ordered and empty.

## Decisions

### Module and state

`Encounter` owns the participants. Each participant is a `CombatantId`, a `Side` (`Pc` or `Enemy`), a `Creature`, and a `CombatantState`. The attack-trait count for the current turn lives on the encounter, keyed by `CombatantId`, not as a new field of `CombatantState`. Actions and `reaction_ready` stay the fields creature-template already defined. The encounter sets them.

Alternative considered: store the attack count on `CombatantState`. Rejected. That amends the creature-template state for a count only this turn needs.

`begin_encounter` rejects a duplicate id, an empty roster, and a Perception modifier that `CheckModifier::new` rejects, before any die is drawn.

### Initiative

Draw one d20 per combatant in ascending `CombatantId` order. The total is the face plus the baked Perception modifier. Do not call `degree_of_success`. A natural 20 does not add 10.

Sort the order by higher total, then `Enemy` before `Pc`, then lower `CombatantId`. Side is the participant's `Side`, not `uses_dying_rules`.

Each combatant rolls its own check. Player Core allows the GM to roll identical creatures as a group or one by one. This change always rolls one by one. That is the book's individual option, not a deviation.

Tabletop substitute for ties other than PC versus enemy. Rule: tied PCs choose who goes first, and the GM orders tied foes (Player Core pg. 435, Rules 2423). Why not: there are no players and no GM. Substitute: the lower `CombatantId` acts earlier. Intent: the same roster and the same dice always produce the same order. A PC–enemy tie is not this substitute. The enemy acts earlier, which is the book. This is DEV-05, using the same id order for two enemies.

### Start, end, and the action count

`start_turn` skips dead combatants, runs the start step, then sets actions to 3, the reaction to ready, and that creature's attack count to 0. The start step rolls no die and does not change Hit Points, temporary Hit Points, or wounded. A creature that has entered dying still starts. Only `Vital::Dead` is skipped.

Tabletop substitute for a dying creature's turn. Rule: an unconscious creature cannot act (Player Core pg. 446). Why not: unconscious and dying are `add-dying-and-victory`. Substitute: `Vital::EnterDying` still regains 3 actions. Intent: this change does not invent the skip those rules will add.

`end_turn`, and any spend that leaves 0 actions, runs the end step, sets actions to 0, leaves an unspent reaction ready, and advances the index. The end step rolls no die. The round starts at 1 and increases by 1 when the index wraps. The next turn starts only when `start_turn` is called. If a skip wraps the order, the round increases there too. If every combatant is dead, `start_turn` returns `NoLivingCombatant` and leaves the round and the order unchanged. There is no victory variant.

Tabletop substitute for the handoff. Rule: spending the last action ends the turn and the next creature's turn begins (Player Core pg. 436, Rules 2429). Why not: the function that spent the action cannot also play the next creature. Substitute: the turn ends, and the caller starts the next one. Intent: the empty end step is visible before the next start.

Reactions are ready after `begin_encounter`, before any start (DEV-06). Rule: the GM decides whether reactions work before the first turn (Player Core pg. 436, Rules 2432). Why not: there is no GM. Substitute: a living combatant's reaction is ready at encounter start. Intent: one fixed answer. Spending it does not roll, does not change actions, and does not change the attack count. A dead combatant cannot spend it.

`spend_actions` accepts 1, 2, or 3, subtracts them at once, and returns `InsufficientActions` or `InvalidActionCost` without a partial spend. A free action during an open turn changes neither count and does not end the turn. Off an open turn it returns `TurnNotOpen`.

### Strike

`strike` requires the open turn's active id, a different target id, and at least one action. On those failures it rolls nothing and does not increment the count. It rolls one d20, passes the strike's attack bonus, the current multiple attack penalty, a circumstance term of 0, a range term of 0, and the caller's attack-statistic modifiers to `attack_check_total`. Damage-statistic modifiers go to `damage_dealt`. Any other statistic is ignored, so a caller Armor Class penalty does not change the total or the baked Armor Class.

Tabletop substitute for reach. Rule: a melee Strike targets a creature within reach (Player Core pg. 418, Strike). Why not: reach, flanking, and prone are a later change. Substitute: the caller names the target, and this module stores no squares. Intent: Strike is testable without the grid.

The degree after the natural 20 or natural 1 selects damage. Success rolls the strike's damage dice once and adds `damage_bonus`. Critical success doubles that total. Failure and critical failure roll no damage die. Doubling rolls the dice once and doubles the total. The book's option to roll the dice twice and double the modifiers is not used (Player Core pg. 407, Rules 2307).

Damage then follows `damage_dealt`: dice plus the baked bonus, then penalties, then the floor of 1 only when penalties reduce a pre-penalty total of at least 1, or a clamp at 0 otherwise, then doubling. R-07 and DEV-14 are this order. The smallest floored crit is 2. A clamped 0 stays 0 on a crit. Apply the result to temporary Hit Points first and the remainder to current Hit Points, floored at 0 (R-06, the damage sentence only). Reuse `note_hit_points` for the existing dead versus enter-dying result. Do not change wounded, do not move the initiative entry, do not touch shield Hit Points, and do not treat a blow of at least twice maximum Hit Points as a separate result.

An overflow from the attack total or from damage returns a typed error before any of those writes. Dice already drawn stay drawn. The attack count stays where it was, so the next Strike on that turn is still the first if the failed Strike was the first.

`attack_trait_penalty(id, on_own_turn, agile)` reads the count and does not increment it. Off that creature's turn it returns 0. It does not store a Ready snapshot.

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

The scenario tests for the removed requirement call `attack_check_total`. They do not call `attack_check_parts`. The private helper may remain.

### Public shape

```rust
pub enum Side { Pc, Enemy }

pub struct Participant {
    pub id: CombatantId,
    pub side: Side,
    pub creature: Creature,
    pub state: CombatantState,
}

pub enum EncounterError {
    DuplicateCombatant,
    EmptyRoster,
    PerceptionModifier,
    UnknownCombatant,
    TurnInProgress,
    TurnNotOpen,
    NotYourTurn,
    InsufficientActions,
    InvalidActionCost,
    NoReaction,
    Dead,
    SelfTarget,
    NoLivingCombatant,
    DamageOverflow,
}

pub struct StrikeOutcome {
    pub natural: u8,
    pub total: i32,
    pub degree: DegreeOfSuccess,
    pub map: i32,
    pub damage: i32,
}
```

`round`, `order`, and `turn_open` are readable. `state` and `creature` are borrowed by id. `strike` returns `StrikeOutcome` on success. Every public item gets a `///` comment citing the Player Core section when it states a rule. No `unwrap`, `expect`, `panic!`, or unchecked indexing outside tests. Sums use checked arithmetic.

## Risks / Trade-offs

- [Call sites of `attack_check_total` still expect `i32`] → Update them in the same change. An empty caller list keeps the existing success totals. The proof run is `cargo test --workspace`.
- [`damage_dealt` ignores temporary Hit Points] → Strike applies the dealt amount temporary-Hit-Points first. Extending `apply_dice_damage` is fine only when a state at 0 temporary Hit Points still matches the creature-template tests. Do not add a second damage formula.
- [A creature that has entered dying can still Strike] → Called out above. `add-dying-and-victory` is the change that stops that turn.
- [The attack count is invisible if only `strike` exists] → `attack_trait_penalty` is the read the reaction scenario uses, and it does not increment.

## Migration Plan

This is a library change with no stored data. Update every `attack_check_total` call to pass the caller list and handle `Result`. Rollback is reverting the implementation commit. The spec-only pull request does not add a changelog bullet and does not archive. The implementation pull request adds one Unreleased bullet, then `openspec archive add-encounter-turns --yes` as its last commit.

R-05, R-06, and R-07 are settled in the decisions above. Nothing in this change is waiting on a Designer answer.
