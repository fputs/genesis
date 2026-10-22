# Design

## Context

`CombatantState` stores current Hit Points, `temp_hit_points` (always 0 today), `wounded` (always 0), and `Vital::{Alive, Dead, EnterDying}`. `dying_entered` is a bool, not a dying value. `apply_dice_damage` and `apply_healing` subtract or add Hit Points and then mark 0 Hit Points as `Dead` or `EnterDying`. They do not spend temporary Hit Points, set a dying value, or end the encounter. `ac_modifiers` and `perception_modifiers` already take unconscious as a status −4. `Statistic` has no Reflex. `flat_check` rolls one bare d20. Sides already exist as `FactionId` on the grid. See proposal.md for why. Rulings live in `docs/designer/rules-rulings.md`; this design cites them and does not copy that log.

`add-encounter-turns` is a parallel change for initiative, turn order, and Strike. This change merges after it. It does not import that change's types. Turn start and the initiative move are hooks below. Damage goes through `damage_dealt`, `apply_dice_damage`, and `apply_healing`.

This change names no file on `v1-reference`, so none is ported.

## Goals / Non-Goals

**Goals:**

- One module, `crates/rules/src/dying.rs`, for dying, recovery, temporary Hit Points, the initiative-move report, and victory.
- `Vital::Dying` stores 1, 2, or 3. Dying 4 or higher is `Dead` before the call returns.
- Unconscious is "vital is Dying". Reflex gets the same status −4 the other two statistics already produce.
- Temporary Hit Points follow R-06 / DEV-13, including every edge case in the spec.
- Victory is a pure function of a roster of `(CombatantId, FactionId, state)`.

**Non-Goals:**

- Initiative order, Strike, multiple attack penalty, and spending actions. `actions_remaining` and `reaction_ready` stay whatever the caller set.
- Prone, off-guard, blinded, and grid position. DEV-12 says unconscious applies prone and does not drop held items. This change drops nothing, because items are not modeled. It also does not set prone or off-guard. Positioning owns those effects. Recording that here so it is not a silent skip.
- Doomed. The death threshold stays 4.
- Treat Wounds and a 10-minute rest. Wounded does not fall during the encounter.
- A Reflex save, or any other save. The Reflex output is the penalty total only.
- Condition ids on the unconscious penalty, and any new test of that penalty against R-01, R-02, R-03, or R-04. See Open questions.
- R-07 / DEV-14. `damage_dealt` already applies the penalty floor before doubling. Massive damage uses that returned blow.

## Decisions

### Module boundary

`dying.rs` owns the new rules and is re-exported from `lib.rs`. `apply_dice_damage` and `apply_healing` stay the public damage and healing entries in `combatant.rs` and call into `dying`. `dying` uses `CombatantState` and `Creature` and does not call back into the dice roller except through `flat_check`, which the start-of-turn hook calls.

Alternative considered: put the whole track in `combatant.rs`. Rejected. Dying, recovery, and victory are their own concept, and that file is already the creature template.

`ConditionSlot` stays an empty vec. Unconscious is derived from `Vital`, so it cannot disagree with dying.

### Vital

```rust
pub enum Vital {
    Alive,
    Dying(DyingValue), // 1, 2, or 3 only
    Dead,
}
```

`DyingValue` rejects 0 and anything above 3 at construction. `EnterDying` and `dying_entered` go away. **BREAKING** for in-crate matches on `Vital`. There is no saved encounter to migrate.

A creature is conscious only when `Vital::Alive`. Dying and dead are not.

### Blow order

`apply_dice_damage` still rolls first and calls `damage_dealt`. An overflowing total returns `HpError::Overflow` and writes nothing.

Then, for a resolved amount:

1. A dead combatant stays dead. The die was still rolled.
2. Massive damage: amount >= 2 × maximum Hit Points, compared in `i64`, using the Creature's maximum and not temporary Hit Points. The creature dies. Current Hit Points become 0, temporary Hit Points clear, wounded is unchanged, and no dying value is stored. No initiative report.
3. Otherwise spend temporary Hit Points, then current Hit Points, floored at 0.
4. Amount 0 does not change dying.
5. Without `uses_dying_rules`, 0 Hit Points is death. No initiative report.
6. With the flag, a conscious creature reduced to 0 gains dying 1, or 2 when `critical` is set, plus wounded. The sum is computed before storing. 4 or more dies instead, with no stored dying value and no initiative report. A stored 1, 2, or 3 sets the initiative report.
7. A creature that was already dying and takes amount > 0 increases dying by 1, or by 2 when `critical` is set. 4 or more dies. Wounded does not change. This step does not run on the blow that first caused dying.

`critical` on the existing damage API is the only critical signal. A critical failure on a save is out of scope until saves exist.

Death from dying 4 does not clear temporary Hit Points. The spec says that only for massive damage. Recovery that kills leaves any remaining pool in place.

### Temporary Hit Points

`CombatantState` keeps `temp_hit_points` and gains `temp_rounds: u32`. Amount 0 means none, and rounds are then 0. Fresh state stays amount 0, rounds 0.

`grant_temporary_hit_points(state, amount, rounds)`:

- Amount 0 returns success and does not change the pool, at any `rounds` value.
- A negative amount returns a typed error and does not change the pool.
- `rounds == 0` with a positive amount returns a typed error and does not change the pool.
- A strictly greater positive amount replaces both the amount and the rounds.
- An equal or smaller positive amount keeps the existing amount and rounds.

`start_of_turn` decrements `temp_rounds` when it is above 0, then clears the amount when the count hits 0, and only then runs recovery. Wounded uses `checked_add`. Overflow returns `HpError::Overflow` and leaves the state unchanged.

### Healing

`apply_healing` rejects amount <= 0 with a typed error. It rejects a dead combatant with a different typed error. Both leave the state unchanged. A positive heal still caps at the Creature's maximum and still returns `HpError::Overflow` without writing when the add does not fit. If the combatant was dying, the successful heal removes dying, adds 1 to wounded, and leaves them conscious at the capped Hit Points. It does not set Hit Points to 1. Recovery is the path that sets 1.

### Recovery and the turn hook

`start_of_turn(state, creature, dice)` ticks temporary Hit Points, then if vital is `Dying` calls `flat_check(dice, 10 + dying)`. It does not roll for `Alive` or `Dead`, and it does not touch actions, the reaction, or the initiative report.

Degree deltas are −2, −1, +1, +2. A result of 0 or less removes dying, sets current Hit Points to 1, adds 1 wounded, and becomes `Alive`. A result of 4 or more becomes `Dead` with no dying value. A result of 1, 2, or 3 stays `Dying` and does not change Hit Points or wounded.

`add-encounter-turns` calls this hook at the start of the creature's turn, before it grants 3 actions and 1 reaction. Until that change is in the tree, tests call `start_of_turn` directly.

### Initiative report

`CombatantState` gains `initiative_move: bool`, default false. It becomes true only when a blow stores a new dying value. `take_initiative_move` returns that bool and sets it false. No other operation clears it. This module does not take a turn order and does not reorder anyone. `add-encounter-turns` takes the report when it applies the blow and moves that `CombatantId` to just before the current turn.

Alternative considered: return the flag from `apply_dice_damage` instead of storing it. Rejected. The existing success type stays `()`, and the turn change can read the report after the blow returns.

### Unconscious penalty

Add `Statistic::Reflex` and `reflex_modifiers(unconscious: bool)`, shaped like `perception_modifiers`: a reserved source, status −4, no condition id. `ac_modifiers` and `perception_modifiers` stay as they are. Callers pass `matches!(vital, Vital::Dying(_))`. Tests assert the status total of −4 and that the three statistics do not share a stack. They do not assert sequence numbers or condition ids.

### Victory

```rust
pub enum EncounterOutcome {
    Ongoing,
    Victory { side: FactionId },
    MutualDefeat,
}
```

The roster is a slice of combatant id, `FactionId`, and state. Conscious means `Vital::Alive`. Sides with no combatants are absent. With fewer than two sides present, the result is `Ongoing`, including an empty roster and a single side that is wiped out. With two or more sides, exactly one conscious side is `Victory`, and zero conscious sides is `MutualDefeat`. Order does not matter. The same `FactionId` is the movement faction and the battle side, so one creature cannot be an ally on the grid and an enemy for victory.

Alternative considered: a separate `SideId`. Rejected. Two team ids would drift.

### Errors and purity

New error variants name the case: non-positive healing, healing the dead, a negative temporary amount, a zero duration. No string errors. Library code does not panic. Arithmetic is checked. No new dependency, no `unsafe`, no clock, no global dice. `Creature` and `CharacterBuild` are only borrowed for reading.

## Risks / Trade-offs

- [In-crate tests match `Vital::EnterDying`] → Update those assertions to dying 1 in this implementation. Do not delete the tests.
- [`add-encounter-turns` is not in the tree yet] → Hooks are local functions. Nothing here imports that change.
- [Designer later requires a condition id on unconscious] → The −4 still ships. The grouping change is a follow-up spec, not a guess in this one.
- [Prone is ledgered on DEV-12 and not set here] → Positioning applies it. This change does not invent a prone bit that nothing reads.
- [Dying-4 death leaves temporary Hit Points] → Only massive death clears them, which is what the spec says. A later ruling can clear every death.

## Migration Plan

This is a library API break inside `crates/rules`. Update the `Vital` matches in `combatant.rs` tests in the implementation pull request. There is no serialized roster. Rollback is reverting that pull request. The spec pull request does not change Rust.

The implementation pull request adds the Unreleased changelog bullet before it is done. `openspec archive add-dying-and-victory --yes` is the last commit of that pull request, not of this spec pull request.

## Open questions

**Testing the unconscious −4 against R-01 through R-04.** Designer has not said which cases to run. The spec requires a status penalty of −4 on Armor Class, Perception, and Reflex, in separate collections, and nothing else about that penalty. It does not assign a condition id, and it does not say how the penalty groups with frightened or sickened, how duplicate-effect keys treat it, or how the valued-condition rules (R-03, R-04) apply. R-02 is the damage-floor ruling already implemented as R-07 / DEV-14; this question does not reopen it.

Until Designer answers, implement Reflex like the existing unconscious penalties: reserved source, no condition id, no new interaction tests. Do not invent those scenarios. An answer that requires a condition id or those tests amends the spec first.
