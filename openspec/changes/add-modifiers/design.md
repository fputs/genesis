# Design

## Context

`crates/rules` already folds a `CharacterBuild` into a `Creature` and rolls checks through injected dice. `multiple_attack_penalty` returns 0, then -5, then -10, or 0, then -4, then -8 when the current weapon is agile. `attack_check_total` still adds loose `i32` penalties. `apply_dice_damage` clamps a negative dice-plus-bonus total to 0, so Hit Points are not raised. `try_new` rejects non-positive Hit Points and a negative Speed. `fold_contributions` does not. See proposal.md for why this change exists. The behavior is `specs/modifiers/spec.md` and `specs/creature-template/spec.md`.

## Goals / Non-Goals

**Goals:**

- One modifier fold: highest bonus of a type, worst penalty of a type, every untyped penalty added, bonus and penalty of one type both kept.
- Construction fails with a typed error unless the value is a bonus greater than 0 or a penalty less than 0.
- Same source id contributes one modifier before the sum. Same condition keeps its highest rank before the sum.
- Separate totals for the attack roll, Armor Class, Perception, and damage.
- The public fold cannot return a Creature that `try_new` would reject.
- Penalties on an attack that reduce damage to 0 or below deal 1, and that 1 is doubled on a critical hit (DEV-14).

**Non-Goals:**

- Proficiency bonuses (DEV-01). AC, attack, Perception, and DCs are baked numbers. There are no attributes or proficiency ranks, so proficiency is not a modifier type.
- A second multiple attack penalty. Agile -4/-8 stays inside `multiple_attack_penalty`.
- Computing range increments. The range scenario passes a synthetic untyped -2. Player Core pg. 403 is the citation, not a producer.
- Shield bonuses. The shields change owns Raise a Shield. This change's Armor Class total does not read `shield_ac_bonus`.
- Resistance, weakness, and immunity. Resistance has its own floor of 0 (Rules 2318, Player Core pg. 408, https://2e.aonprd.com/Rules.aspx?ID=2318). This change does not apply it.
- The Speed floor of 5 feet. That applies when penalties reduce Speed, and Phase 1 has no Speed-penalty producer. Speed 0 remains a legal baked value.
- Reflex saves, the Aid action, and a condition tracker with durations. Aid supplies only a circumstance modifier. This change does not prepare, roll, or resolve Aid. Frightened and sickened enter as ranks. Durations stay with encounter-core.
- A new crate or a new dependency.

## Decisions

### 1. One module, no new dependency

```text
crates/rules/src/lib.rs           mod modifiers; pub use the items below
crates/rules/src/modifiers.rs     types, construction, fold
crates/rules/src/check.rs         multiple_attack_penalty stays here
crates/rules/src/combatant.rs     fold validation, damage floor, heal cap
```

`Cargo.toml` gains no dependency. The new code does not read a clock, a file, the network, or a global, and it does not use `Cell`, `RefCell`, `Mutex`, or `unsafe`. Sequence numbers are caller-supplied. Newer means a greater sequence number, not a timestamp.

Every fallible function returns `Result`. Sums use `checked_add`. Overflow is `ModifierError::Overflow`, not a panic, not a wrapped `i32`, and not saturation to the minimum or maximum value. Library code has no `unwrap`, `expect`, or `panic`.

### 2. Illegal modifiers cannot be built

```rust
pub enum BonusType { Circumstance, Item, Status }
pub enum PenaltyType { Circumstance, Item, Status, Untyped }

pub enum ModifierError {
    UntypedBonus,
    Zero,
    Sign,
    Overflow,
}

pub struct Modifier { /* source, sequence, level, kind, nonzero value */ }

impl Modifier {
    pub fn bonus(ty: BonusType, value: i32, source: SourceId, sequence: u32, level: Option<u32>) -> Result<Self, ModifierError>;
    pub fn penalty(ty: PenaltyType, value: i32, source: SourceId, sequence: u32, level: Option<u32>) -> Result<Self, ModifierError>;
}
```

There is no proficiency variant and no untyped bonus variant. A bonus value must be greater than 0. A penalty value must be less than 0. `bonus` returns `Zero` for 0 and `Sign` for a negative value. `penalty` returns `Zero` for 0, `UntypedBonus` for a positive untyped value, and `Sign` for any other positive value. `SourceId` is a `u32` newtype. Equal ids are the same effect.

Alternative considered: store untyped bonuses in a bucket that sums. Rejected. Remaster text says penalties can be untyped, unlike bonuses (Player Core pg. 400). Nothing in Phase 1 needs that bucket.

### 3. Fold order

`total(modifiers)` returns `Result<i32, ModifierError>`:

1. Modifiers that share a `ConditionId` collapse first. The kept condition has the highest rank. Frightened and sickened then become one status penalty equal to the negation of that rank, so frightened 2 is -2, not a second modifier of -1. A condition with no rank, such as off-guard, is kept once. Rank is the condition's own number. It is not the algebraic maximum of the penalty, which would keep frightened 1 over frightened 2.
2. Modifiers that share a `SourceId` collapse next. The higher level or rank wins. If level and rank do not decide, the newer effect wins, and newer means the greater sequence number. Value is not compared. An untyped penalty of -5 with sequence 9 beats an untyped penalty of -2 with sequence 1 from the same source. This happens before any sum. Rules 2266 (Player Core pg. 398) is the citation.
3. For each of circumstance, item, and status, add the highest positive value and the lowest negative value. Both are kept when both exist. When different sources tie for that highest bonus or that worst penalty, the applied modifier is the one with the lowest sequence number. The result reports that modifier, not only the number. Add every remaining untyped penalty with `checked_add`. `None` is `ModifierError::Overflow`. Do not use saturating arithmetic.

The same inputs produce the same total. Insertion order does not matter. Same-source duplicates keep the higher level or rank, otherwise the greater sequence number. Different-source ties for one type's winning value keep the lowest sequence number.

`dc_from_total` is 10 plus that total, with the same checked add, and it does not clamp. A total of -6 yields DC 4.

`flat_check` is not given this total. The flat-check scenario calls the existing function and shows the +9 was not added.

Alternative considered: take the worse of a bonus and a penalty of one type. Rejected. Player Core pg. 400 applies both.

### 4. Phase 1 producers call the existing multiple attack penalty

`attack_modifiers` takes the multiple attack penalty as a caller-supplied value and, when that value is not 0, inserts it as one untyped penalty. It does not copy the -5/-10 or -4/-8 table. Phase 1 passes the result of `multiple_attack_penalty` for an attack on the creature's turn. Prone inserts circumstance -2. An Aid result of +1, +2, or -1 inserts a circumstance modifier. An Aid result of 0 inserts nothing, because zero is rejected at construction. The Aid action is out of scope: these functions accept that circumstance modifier and do not prepare, roll, or resolve Aid.

Seam for a later ready change (FIX-16). This change does not implement it. Because the penalty is an argument, a later change can pass the MAP frozen when an attack was readied. Reaction attacks and the Ready action stay out of this change.

`ac_modifiers` inserts cover +1, +2, or +4 circumstance, off-guard -2 circumstance, and unconscious -4 status. It has no shield argument. `perception_modifiers` inserts unconscious -4 status only. Damage has no Phase 1 producer; callers pass modifiers into `damage_dealt`.

`attack_check_total` keeps its signature. It builds modifiers from its existing penalty arguments and uses `total`, so the dice-and-checks scenarios that stack MAP with a circumstance penalty and with another untyped penalty still pass. A 0 argument is omitted rather than constructed.

Attack-roll modifiers and Armor Class modifiers are different slices. The fold of one does not see the other.

### 5. The public fold uses the template checks

`try_new` still returns `TemplateError::NonPositiveHitPoints` when Hit Points are not at least 1, and `TemplateError::NegativeSpeed` when Speed is negative (Rules 2346, Player Core pg. 420). Speed 0 stays valid. A Speed that is not a multiple of 5, such as 7, returns `TemplateError::SpeedNotMultipleOfFive`. This is validation (DEV-21).

- **Rule**: Rules 2347 (Player Core pg. 420, https://2e.aonprd.com/Rules.aspx?ID=2347) defines Speed. Rules 2356 (Player Core pg. 421, https://2e.aonprd.com/Rules.aspx?ID=2356) defines a square as 5 feet. Player Core has no explicit multiple-of-5 rule.
- **Why not**: A Speed of 7 cannot be spent in 5-foot squares without a rounding rule this change does not own.
- **Substitute**: Reject that Speed at `try_new` and in `fold_contributions`. Speed 0, which is a multiple of 5, stays valid.
- **Intent**: Baked Speeds that reach the grid are whole squares. The 5-foot penalty floor stays out of scope.

After `fold_contributions` has summed a non-empty list, it applies those same checks, including the multiple-of-5 Speed check. Failure returns `FoldError::Statistics` and not a Creature. Overflow, an empty list, and a bad damage die stay the errors they already are. `derive` still builds from the validated base and does not call the fallible fold. Folding only that base still equals `derive`.

```rust
pub enum FoldError {
    Overflow,
    DamageDie,
    Empty,
    Statistics,
}
```

### 6. Damage floor, then critical doubling

This is DEV-14, our ruling. RAW does not state the order.

- **Rule**: Penalties on an attack that reduce damage to 0 or below still deal 1 (Rules 2302, Player Core pg. 406, https://2e.aonprd.com/Rules.aspx?ID=2302). A critical hit doubles the total (Rules 2307, Player Core pg. 407, https://2e.aonprd.com/Rules.aspx?ID=2307). RAW does not say whether that floor comes before or after doubling.
- **Why not**: Flooring after doubling would make the smallest critical hit 1. Resistance has a separate floor of 0 (Rules 2318, Player Core pg. 408, https://2e.aonprd.com/Rules.aspx?ID=2318) and stays out of scope.
- **Substitute**: DEV-14 applies the floor of 1 to penalties on an attack before critical doubling. The smallest critical hit deals 2.
- **Intent**: A critical hit still doubles the damage those penalties left, including the 1 they cannot erase.

```rust
pub enum HpError { Overflow }

pub fn damage_dealt(rolled_plus_baked: i32, modifiers: &[Modifier], critical: bool) -> Result<i32, ModifierError>;

pub fn apply_dice_damage(...) -> Result<(), HpError>;
pub fn apply_healing(state: &mut CombatantState, creature: &Creature, amount: i32) -> Result<(), HpError>;
```

`damage_dealt` folds `modifiers`, adds that total to `rolled_plus_baked` with `checked_add`, and on overflow returns `ModifierError::Overflow`. When penalties on an attack bring the sum to 0 or below, it becomes 1. If `critical` is true, it then doubles with `checked_mul`. A rolled 4 and a penalty of -6 deal 1, or 2 on a critical hit. Resistance is not an argument.

`apply_dice_damage` rolls, calls `damage_dealt` with `critical: false` unless the caller passes a critical hit, and subtracts the result from current Hit Points, floored at 0. On overflow it returns `HpError::Overflow` and leaves the state unchanged. It no longer turns a negative total into 0 damage.

`apply_healing` adds a positive amount and caps at the creature's maximum (Rules 2320, Player Core pg. 410, https://2e.aonprd.com/Rules.aspx?ID=2320). An add that does not fit returns `HpError::Overflow` and leaves the state unchanged. Current Hit Points on both paths stay in 0 through the maximum.

The clamp-to-0 behavior lives in `apply_dice_damage` and in tests `a_negative_total_does_not_raise_hit_points`, `healing_past_i32_max_leaves_hit_points_unchanged`, and `damage_past_i32_floors_hit_points_at_0`. It is not a requirement in `openspec/specs/creature-template/spec.md`. The delta therefore adds "Penalties leave at least 1 damage" instead of modifying a header archive cannot find. Those three tests stay, and their assertions change to this rule. They are not deleted.

Alternative considered: floor the doubled total, so a critical hit of a non-positive roll deals 1. Rejected. DEV-14 applies the floor before doubling. RAW does not state the order; this is our ruling.

### 7. Duplicate effects keep level or rank, then the newer effect

Rules 2266 (Player Core pg. 398, https://2e.aonprd.com/Rules.aspx?ID=2266): only one instance applies, using the higher level or rank, or the newer effect if those are equal. Newer means the greater sequence number. Value does not break the tie. The earlier step that kept the algebraically higher value, so an older -2 beat a newer -5, is removed. A newer -5 with sequence 9 wins over an older -2 with sequence 1 from the same source.

Condition rank in step 1 is still the positive condition value for a repeated condition. It is applied before this comparison.

Different sources that tie for one type's highest bonus or worst penalty still report the lowest sequence number. That is which tied winner is reported. It is not the duplicate-effects tie-break.

## Risks / Trade-offs

- [A newer penalty from the same source replaces an older one when level and rank are equal, even when its value is lower] → Rules 2266 keeps the newer effect. Value is not a tie-break. The equal-value duplicate still counts once.
- [Penalties on an attack that used to deal 0 now deal 1, and a critical hit doubles that 1] → DEV-14. RAW does not state the order. The three existing tests change their assertions and keep their names.
- [A folded creature with 0 Hit Points used to be returned] → It is now `FoldError::Statistics`. `derive` of a valid build does not change.
- [`attack_check_total` grows a modifier allocation] → The signature stays, and the existing totals stay. A 0 penalty is omitted so construction does not fail.

## Migration Plan

This pull request adds only `openspec/changes/add-modifiers/`. The implementation edits `modifiers.rs`, `combatant.rs`, `check.rs`, and `lib.rs`, updates the three damage tests in place, adds the scenario tests, adds the Unreleased changelog bullet, runs the proof commands, and ends with `openspec archive add-modifiers --yes`. Rollback of that later pull request is reverting it.

## Open Questions

None. DEV-14 records the floor-before-doubling order. Rules 2266 records the duplicate-effects tie-break: higher level or rank, then the greater sequence number.

## QA questions

| Q | Answer |
| --- | --- |
| Q1 Untyped bonuses | Scenario: An untyped bonus is rejected |
| Q2 Negative or zero bonus | Scenarios: A negative bonus is rejected; A zero modifier is rejected; A positive penalty is rejected |
| Q3 Double counting | Scenario: A duplicate same-source untyped penalty counts once |
| Q4 Overflow | Scenario: An overflowing modifier sum returns an error |
| Q5 Tie reporting | Scenarios: A tied highest bonus reports the lowest sequence; A tied worst penalty reports the lowest sequence |
| Q6 Item bonuses | Scenarios: An item bonus is accepted; Phase 1 produces no item modifier |
| Q7 Scope | Scenarios under Same-type stacking and Phase 1 application points, including MAP -5 plus range -2 equals -7 and Agile MAP stays the existing untyped penalty. Aid circumstance only: An Aid success adds +1 circumstance; An Aid failure adds no modifier. Out of scope: The Aid action is out of scope |
