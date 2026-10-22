# Design

## Context

`CombatantState` stores current Hit Points, `temp_hit_points` (always 0 today), `wounded` (always 0), and `Vital::{Alive, Dead, EnterDying}`. `dying_entered` is a bool, not a dying value. `apply_dice_damage` and `apply_healing` subtract or add Hit Points and then mark 0 Hit Points as `Dead` or `EnterDying`. They do not spend temporary Hit Points, set a dying value, or end the encounter. `ac_modifiers` and `perception_modifiers` already take unconscious as a status −4, without a condition id. `Statistic` has no Reflex. `flat_check` rolls one bare d20. See proposal.md for why.

`add-encounter-turns` merges first. It defines `Side::{Pc, Enemy}` on each participant and owns initiative, turn order, and Strike. This change uses that `Side` for victory. It owns the semantics of R-08, R-10, and temporary Hit Point ticks. The encounter-turns delta removes "Reaching 0 Hit Points floors at 0", routes a hitting Strike through `apply_resolved_blow`, and records rounds when temporary Hit Points reach 0. It does not change the not-Alive turn rule. `wire-dying-into-turns` only calls `start_of_turn` and `take_initiative_move`. That follow-up, not this change, removes the interim Armor Class workaround (DEV-36 in `docs/designer/rules-deviations.md`): until then, Strike still compares the target's baked Armor Class.

`FactionId` stays the grid movement faction. Victory does not read it.

This change names no file on `v1-reference`, so none is ported. Rulings and deviations are cited from `docs/designer/rules-rulings.md` and `docs/designer/rules-deviations.md`. This change does not edit those files and does not copy the log.

## Goals / Non-Goals

**Goals:**

- One module, `crates/rules/src/dying.rs`, for dying, recovery, temporary Hit Points, the initiative report, and victory.
- `Vital::{Alive, Unconscious, Dying(1..=3), Dead}`. Dying 4 or higher is `Dead` before the call returns.
- Unconscious means 0 Hit Points and not dying. Dying and Unconscious both take the status −4. Armor Class also takes off-guard −2. The 10-minute natural wake-up is not modeled.
- Temporary Hit Points follow R-06 / DEV-13. Durations tick at the recipient's start of turn (DEV-32).
- Victory is a pure query of `(CombatantId, Side, state)`.

**Non-Goals:**

- Initiative order, Strike, multiple attack penalty, and spending actions. `actions_remaining` and `reaction_ready` stay whatever the caller set. Reordering after knockout belongs to `wire-dying-into-turns`, not to a SHALL here.
- Prone and blinded (DEV-31 in `docs/designer/rules-deviations.md`). Held items stay held (DEV-12 in that file). Removing the interim normal Armor Class on a dying or unconscious target is DEV-36, and `wire-dying-into-turns` owns that removal.
- Doomed (DEV-28). The death threshold stays 4.
- Nonlethal damage, persistent damage, death effects, a save critical failure increasing dying by 2, and regeneration (DEV-29). All damage is lethal. `damage_dealt` keeps its existing `critical: bool`. `apply_resolved_blow` takes a `DegreeOfSuccess`.
- Treat Wounds. The 10-minute natural wake-up is not modeled. Wounded does not fall during the encounter.
- A Reflex save. The Reflex output is the penalty total only.
- A second team id. Victory uses `Side` from `add-encounter-turns`.

## Decisions

### Module boundary

`dying.rs` owns the new rules and is re-exported from `lib.rs`. `apply_dice_damage` and `apply_healing` stay the public damage and healing entries in `combatant.rs` and call into `dying`. `dying` uses `CombatantState`, `Creature`, and `Side`. It does not call back into the dice roller except through `flat_check`, which `start_of_turn` calls. It does not define `Side`.

`ConditionSlot` stays an empty vec. Unconscious and dying are derived from `Vital`, so a stored condition cannot disagree with them. `modifiers.rs` does not read `Vital`. Callers pass `unconscious: true` for `Dying` or `Unconscious`, and pass `off_guard: true` into `ac_modifiers` for those same vitals.

### Vital

```rust
pub enum Vital {
    Alive,
    Unconscious,
    Dying(DyingValue), // 1, 2, or 3 only
    Dead,
}
```

`DyingValue` rejects 0 and anything above 3 at construction. `EnterDying` and `dying_entered` go away. **BREAKING** for in-crate matches on `Vital`. There is no saved encounter to migrate.

Alive is the only conscious vital and the only vital that keeps a Side from being defeated. Dying and Unconscious are not Alive. Dead is not Alive.

Public `CombatantState` fields stay, so `Alive` at 0 Hit Points, and `Dying` or `Unconscious` with current Hit Points other than 0, can be represented. Those states are rejected, not made unrepresentable. `apply_resolved_blow`, `apply_healing`, `start_of_turn`, and `grant_temporary_hit_points` return `HpError::InconsistentVital` before drawing a die and write nothing. A successful transition into `Dying` sets current Hit Points to 0 in the same step. `encounter_outcome` does not validate that shape; it only reads whether vital is `Alive`.

The 10-minute natural wake-up is not modeled. Positive healing is the only way Unconscious becomes Alive.

### Blow order

`apply_dice_damage` rolls, calls `damage_dealt`, and on `Ok` calls `apply_resolved_blow`. An overflowing total returns `HpError::Overflow` and writes nothing. The die already drawn stays drawn.

`apply_resolved_blow(state, creature, amount, degree)` is the one resolution path. `degree` is a `DegreeOfSuccess`. `CriticalSuccess` is a critical blow. `Success` is an ordinary blow. `Failure` and `CriticalFailure` return `HpError::NotAHit` and write nothing. `add-encounter-turns` defines that variant. This change does not declare it. `apply_dice_damage` keeps the existing `critical: bool` and maps true to `CriticalSuccess` and false to `Success`. The function does not roll. Tests use it for a resolved 0 and for `2147483647`. There is not a helper per test. `reflex_modifiers` keeps `unconscious: bool`, the same argument `perception_modifiers` and `ac_modifiers` already take on main. A separate consciousness enum on Reflex alone would split the three producers.

Then, for a resolved amount:

1. `InconsistentVital` is returned when Alive is at 0 Hit Points, or when Dying or Unconscious has nonzero Hit Points. Nothing is written and no die is drawn.
2. A Dead combatant stays Dead, including when the amount is massive. Temporary Hit Points stay as they are. No new initiative report. The die was still rolled when the caller was `apply_dice_damage`.
3. Massive damage: amount >= 2 × maximum Hit Points, compared in `i64`, using the Creature's maximum and not temporary Hit Points. `i32::MAX` compared with `2 * i32::MAX` is not massive. Death sets current Hit Points to 0, clears temporary amount and rounds, leaves wounded unchanged, stores no dying value, and clears a stale initiative report. The result is `Ok`.
4. Otherwise spend temporary Hit Points, then current Hit Points, floored at 0. If the temporary amount hits 0, set rounds to 0 too.
5. Amount 0 does not change dying. A scripted d4 of 1 with a baked bonus of −5 clamps to 0 under Rules 2302, because that −5 is not a penalty. The same roll with an untyped damage penalty of −5 floors to 1.
6. Without `uses_dying_rules`, 0 Hit Points is death. Clear temporary Hit Points and rounds. Clear a stale report. Do not create one.
7. With the flag, an Alive creature reduced to 0 gains dying 1, or 2 when the degree is `CriticalSuccess`, plus wounded. An Unconscious creature taking amount > 0 does the same (R-08) and sets the initiative report. The sum is computed with `checked_add` before storing. A sum that is 4 or more, or that does not fit in `i32`, is death and `Ok`: no stored dying value, temporary Hit Points cleared, no new report. A stored 1, 2, or 3 sets current Hit Points to 0 and sets the initiative report.
8. A creature that was already Dying and takes amount > 0 increases dying by 1, or by 2 when the degree is `CriticalSuccess`, even when temporary Hit Points absorbed all of it (R-10). 4 or more dies. Wounded does not change. This step does not run on the blow that first caused dying.

`DegreeOfSuccess::CriticalSuccess` is the critical blow. A save critical failure is not this degree (DEV-29).

Every transition into Dead clears temporary amount and rounds (R-10). A blow against someone who is already Dead does not.

### Temporary Hit Points

`CombatantState` keeps `temp_hit_points` and gains `temp_rounds: u32`. Amount 0 means none, and rounds are then 0. Fresh state stays amount 0, rounds 0.

`grant_temporary_hit_points(state, amount, rounds)`:

- Amount 0 returns success and does not change the pool, at any `rounds` value.
- A negative amount returns `HpError::NegativeTemporaryHitPoints` and does not change the pool.
- `rounds == 0` with a positive amount returns `HpError::ZeroTemporaryDuration` and does not change the pool.
- A strictly greater positive amount replaces both the amount and the rounds.
- An equal or smaller positive amount keeps the existing amount and rounds.
- A Dead combatant returns `HpError::Dead` and the pool is unchanged. `InconsistentVital` is checked first.

Durations tick at the recipient's start of turn, before recovery. RAW counts from the source's turn. DEV-32 in `docs/designer/rules-deviations.md` is that engineering simplification.

### Healing

`apply_healing` rejects amount <= 0 with `HpError::NonPositiveHealing`. It rejects a Dead combatant with `HpError::Dead`. Both leave the state unchanged. A positive heal still caps at the Creature's maximum. When `current + amount` does not fit in `i32`, it returns `HpError::Overflow` and writes nothing. Dying at 0 Hit Points healed by `i32::MAX` up to maximum `i32::MAX` fits and becomes Alive at that cap.

If the combatant was Dying and `wounded + 1` fits, the successful heal removes dying, adds 1 to wounded, and leaves them Alive at the capped Hit Points. It does not set Hit Points to 1. If `wounded` is `i32::MAX`, the increment returns `HpError::Overflow` and leaves the state unchanged. Healing Unconscious does not change wounded. Healing that leaves the creature Alive clears the initiative report. Healing never restores temporary Hit Points.

### Recovery and the turn hook

```rust
fn start_of_turn(
    state: &mut CombatantState,
    creature: &Creature,
    dice: &mut impl Dice,
) -> Result<Option<FlatRoll>, HpError>
```

`None` means nothing was rolled: Alive, Unconscious, or Dead. `Some` is the recovery flat check. The function is intentionally not idempotent. A second call ticks again and, if the creature is still Dying, rolls again. Equal starting inputs still match in state, dice cursor, and return value.

Order: reject `InconsistentVital`, tick `temp_rounds` when it is above 0, clear the amount and the rounds when the count hits 0, then if vital is `Dying` call `flat_check(dice, 10 + dying)`. On `Overflow` the state is unchanged and a die already drawn stays drawn. It does not touch actions, the reaction, or the initiative report.

Degree deltas are −2, −1, +1, +2. A result of 0 or less removes dying, leaves current Hit Points at 0, adds 1 wounded, and becomes `Unconscious`. It does not become Alive. A result of 4 or more becomes `Dead`, clears temporary Hit Points and rounds, and clears a stale initiative report. A result of 1, 2, or 3 stays `Dying` and does not change Hit Points or wounded. Wounded uses `checked_add`. At `i32::MAX`, a recovery that would lose dying returns `Overflow` and leaves the state unchanged.

`wire-dying-into-turns` calls this hook at the start of the creature's turn. Until that change exists, tests call `start_of_turn` directly.

### Initiative report

```rust
pub enum InitiativeMove {
    Pending,
}
```

`CombatantState` gains `initiative_move: Option<InitiativeMove>`, default `None`. A blow that newly stores a dying value sets `Some(InitiativeMove::Pending)`. `take_initiative_move` returns that value and sets `None`. Healing that leaves the creature Alive clears it, so the take is `None`. Death clears a stale report that healing did not already clear. No other operation clears it. This module does not take a turn order and does not move anyone. The report means the caller can take it. It does not say where the turn moves.

Two combatants each newly dropped by their own blow each return `Some(InitiativeMove::Pending)` once, then `None`. Taking one does not clear the other.

### Unconscious penalty

`ConditionId::UNCONSCIOUS` is `ConditionId(4)`. Off-guard stays 1, frightened stays 2, and sickened stays 3. The group key is `(UNCONSCIOUS, statistic, penalty, status)`. There is no rank. It is not a valued condition (R-09). Construction returns `InvalidValuedCondition` when a rank is present, the type is not status, the value is not −4, or the modifier is a bonus.

Reserved sources and sequences, added to the reserved-source match so a caller cannot construct them:

- `UnconsciousArmorClass`, sort key `(6, 0)`, modifier sequence 6
- `UnconsciousPerception`, sort key `(7, 0)`, modifier sequence 7
- `UnconsciousReflex`, sort key `(10, 0)`, modifier sequence 10

`Statistic::Reflex` and `reflex_modifiers(unconscious)` match `perception_modifiers`. The status penalty competes in the status bucket. The worst status penalty wins. It adds to circumstance penalties and to status bonuses. Armor Class for Dying or Unconscious also passes `off_guard: true`, so with no other modifier the total is −6. Perception and Reflex stay −4.

### Victory

```rust
pub enum EncounterOutcome {
    Ongoing,
    Victory { side: Side },
    MutualDefeat,
}
```

`Side` is the `add-encounter-turns` enum `Side::{Pc, Enemy}`. This change does not declare another one. The roster is a slice of combatant id, `Side`, and state. A Side is defeated when none of its combatants is `Vital::Alive` (DEV-30). Sides with no combatants are absent. With fewer than two sides present, the result is `Ongoing`, including an empty roster and a single side that is wiped out. With both sides present, exactly one side that has an Alive combatant is `Victory { side }`, and zero such sides is `MutualDefeat`. Order does not matter.

The query is pure and not terminal. A later heal can make `Victory` become `Ongoing`. The encounter loop decides closure. Calling it after a blow, a recovery, or a heal does not write the roster.

### Errors and purity

`HpError` gains `NonPositiveHealing`, `Dead`, `NegativeTemporaryHitPoints`, `ZeroTemporaryDuration`, and `InconsistentVital`, beside the existing `Overflow`. `NotAHit` is not one of those variants. `add-encounter-turns` defines it, and `apply_resolved_blow` returns it. Healing a Dead combatant and granting temporary Hit Points to a Dead combatant both return `Dead`. No string errors. Library code does not panic. Arithmetic is checked. No new dependency, no `unsafe`, no clock, no global dice. `Creature` and `CharacterBuild` are only borrowed for reading.

## Risks / Trade-offs

- [In-crate tests match `Vital::EnterDying`] → Update those assertions to `Dying(1)` in the implementation. Do not delete the tests. A non-massive non-critical drop to 0 with the flag is `Dying(1)`, not `Unconscious`.
- [`add-encounter-turns` merges first] → This implementation needs its `Side`. Strike damage calls `apply_resolved_blow` and does not call `note_hit_points`. `wire-dying-into-turns` calls `start_of_turn` and `take_initiative_move`, and it removes the DEV-36 normal Armor Class. This change does not.
- [DEV-12 records prone and held items] → `docs/designer/rules-deviations.md` already says prone from unconscious is not applied and held items stay held. This spec matches that row and cites it.
- [A public state can pair Alive with 0 Hit Points, or Dying with nonzero Hit Points] → Operations return `InconsistentVital`. The fields stay public because creature-template owns them.

## Migration Plan

This is a library API break inside `crates/rules`. Update the `Vital` matches in `combatant.rs` tests in the implementation pull request. There is no serialized roster. Rollback is reverting that pull request. The spec pull request does not change Rust and does not edit `docs/designer/rules-deviations.md` or `docs/designer/rules-rulings.md`.

The implementation pull request adds the Unreleased changelog bullet before it is done. `openspec archive add-dying-and-victory --yes` is the last commit of that pull request, not of this spec pull request.
