# Design

## Context

`crates/rules` already folds a `CharacterBuild` into a `Creature` and rolls checks through injected dice. `multiple_attack_penalty` returns 0, then -5, then -10, or 0, then -4, then -8 when the current weapon is agile. `attack_check_total` still adds loose `i32` penalties. `apply_dice_damage` clamps a negative dice-plus-bonus total to 0, so Hit Points are not raised. `try_new` rejects non-positive Hit Points and a negative Speed. `fold_contributions` does not. See proposal.md for why this change exists. The behavior is `specs/modifiers/spec.md` and `specs/creature-template/spec.md`.

## Goals / Non-Goals

**Goals:**

- One modifier fold: highest bonus of a type, worst penalty of a type, every untyped penalty added, bonus and penalty of one type both kept.
- Construction fails with a typed error for an untyped bonus and for zero.
- Same source id contributes one modifier before the sum. Same condition keeps its highest rank before the sum.
- Separate totals for the attack roll, Armor Class, Perception, and damage.
- The public fold cannot return a Creature that `try_new` would reject.
- A non-positive damage total after modifiers or penalties deals 1, and that 1 is doubled on a critical hit.

**Non-Goals:**

- Proficiency bonuses. Baked attack bonuses and Armor Class already include them. They are not modifiers.
- A second multiple attack penalty. Agile -4/-8 stays inside `multiple_attack_penalty`.
- Computing range increments. The range scenario passes a synthetic untyped -2. Player Core pg. 403 is the citation, not a producer.
- Shield bonuses. The shields change owns Raise a Shield. This change's Armor Class total does not read `shield_ac_bonus`.
- Resistance, weakness, and immunity. Resistance can reduce damage to 0 (Player Core pg. 408, https://2e.aonprd.com/Rules.aspx?ID=2312). This change does not apply it.
- The Speed floor of 5 feet. That applies when penalties reduce Speed, and Phase 1 has no Speed-penalty producer. Speed 0 remains a legal baked value.
- Reflex saves, Aid's preparation, and a condition tracker with durations. Frightened and sickened enter as ranks. Durations stay with encounter-core.
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
2. Modifiers that share a `SourceId` collapse next. The higher `level` wins. If level does not decide, the algebraically higher `value` wins, so -2 beats -5. If value also ties, the greater sequence number wins. This happens before any sum.
3. For each of circumstance, item, and status, add the highest positive value and the lowest negative value. Both are kept when both exist. When different sources tie for that highest bonus or that worst penalty, the applied modifier is the one with the lowest sequence number. The result reports that modifier, not only the number. Add every remaining untyped penalty with `checked_add`. `None` is `ModifierError::Overflow`. Do not use saturating arithmetic.

The same inputs produce the same total. Insertion order does not matter. Same-source ties still keep the greater sequence number. Different-source ties for one type's winning value keep the lowest sequence number.

`dc_from_total` is 10 plus that total, with the same checked add, and it does not clamp. A total of -6 yields DC 4.

`flat_check` is not given this total. The flat-check scenario calls the existing function and shows the +9 was not added.

Alternative considered: take the worse of a bonus and a penalty of one type. Rejected. Player Core pg. 400 applies both.

### 4. Phase 1 producers call the existing multiple attack penalty

`attack_modifiers` calls `multiple_attack_penalty` and, when the result is not 0, inserts that value as one untyped penalty. It does not copy the -5/-10 or -4/-8 table. Prone inserts circumstance -2. An Aid result of +1, +2, or -1 inserts a circumstance modifier. An Aid result of 0 inserts nothing, because zero is rejected at construction.

`ac_modifiers` inserts cover +1, +2, or +4 circumstance, off-guard -2 circumstance, and unconscious -4 status. It has no shield argument. `perception_modifiers` inserts unconscious -4 status only. Damage has no Phase 1 producer; callers pass modifiers into `damage_dealt`.

`attack_check_total` keeps its signature. It builds modifiers from its existing penalty arguments and uses `total`, so the dice-and-checks scenarios that stack MAP with a circumstance penalty and with another untyped penalty still pass. A 0 argument is omitted rather than constructed.

Attack-roll modifiers and Armor Class modifiers are different slices. The fold of one does not see the other.

### 5. The public fold uses the template checks

`try_new` still returns `TemplateError::NonPositiveHitPoints` when Hit Points are not at least 1, and `TemplateError::NegativeSpeed` when Speed is negative. Speed 0 stays valid.

After `fold_contributions` has summed a non-empty list, it applies those same two checks. Failure returns `FoldError::Statistics` and not a Creature. Overflow, an empty list, and a bad damage die stay the errors they already are. `derive` still builds from the validated base and does not call the fallible fold. Folding only that base still equals `derive`.

```rust
pub enum FoldError {
    Overflow,
    DamageDie,
    Empty,
    Statistics,
}
```

### 6. Damage floor, then critical doubling

This is a documented deviation.

- **Rule**: If penalties reduce damage to 0 or below, deal 1 (Player Core pg. 406, https://2e.aonprd.com/Rules.aspx?ID=2301). A critical hit rolls normally, then doubles the total (Player Core pg. 407, https://2e.aonprd.com/Rules.aspx?ID=2307).
- **Why not**: Flooring after doubling would make the smallest critical hit 1. The sentence on page 406 is about penalties on the roll. Resistance, which can reduce damage to 0, is a different rule on page 408.
- **Substitute**: Apply the penalty floor of 1 before critical doubling. The smallest critical hit deals 2.
- **Intent**: A critical hit still doubles the damage the penalties left, including the 1 those penalties cannot erase.

```rust
pub enum HpError { Overflow }

pub fn damage_dealt(rolled_plus_baked: i32, modifiers: &[Modifier], critical: bool) -> Result<i32, ModifierError>;

pub fn apply_dice_damage(...) -> Result<(), HpError>;
pub fn apply_healing(state: &mut CombatantState, creature: &Creature, amount: i32) -> Result<(), HpError>;
```

`damage_dealt` folds `modifiers`, adds that total to `rolled_plus_baked` with `checked_add`, and on overflow returns `ModifierError::Overflow`. If the sum is 0 or below, it becomes 1. If `critical` is true, it then doubles with `checked_mul`. A rolled 4 and a penalty of -6 deal 1, or 2 on a critical hit. Resistance is not an argument.

`apply_dice_damage` rolls, calls `damage_dealt` with `critical: false` unless the caller passes a critical hit, and subtracts the result from current Hit Points, floored at 0. On overflow it returns `HpError::Overflow` and leaves the state unchanged. It no longer turns a negative total into 0 damage.

`apply_healing` adds a positive amount and caps at the creature's maximum. An add that does not fit returns `HpError::Overflow` and leaves the state unchanged. Current Hit Points on both paths stay in 0 through the maximum.

The clamp-to-0 behavior lives in `apply_dice_damage` and in tests `a_negative_total_does_not_raise_hit_points`, `healing_past_i32_max_leaves_hit_points_unchanged`, and `damage_past_i32_floors_hit_points_at_0`. It is not a requirement in `openspec/specs/creature-template/spec.md`. The delta therefore adds "Penalties leave at least 1 damage" instead of modifying a header archive cannot find. Those three tests stay, and their assertions change to this rule. They are not deleted.

Alternative considered: floor the doubled total, so a critical hit of a non-positive roll deals 1. Rejected. Designer sign-off on the floor-then-double order is still pending; the spec records the ruling anyway.

### 7. What "higher value" means

For two modifiers of one source, level is compared first because duplicate effects name the higher level or rank (Player Core pg. 398). Value is next, and it is the signed number: -2 is higher than -5, so the harsher untyped penalty does not win just by being harsher. Sequence breaks a tie. Condition rank is a different number, the positive condition value, and it is applied in step 1 of the fold, not by this comparison.

## Risks / Trade-offs

- [A repeated harsher untyped penalty from the same source is discarded when its value is lower] → That is the higher-value rule above. The equal-value duplicate still counts once. Designer sign-off is pending.
- [Negative damage totals that used to deal 0 now deal 1] → Required by page 406. The three existing tests change their assertions and keep their names.
- [A folded creature with 0 Hit Points used to be returned] → It is now `FoldError::Statistics`. `derive` of a valid build does not change.
- [`attack_check_total` grows a modifier allocation] → The signature stays, and the existing totals stay. A 0 penalty is omitted so construction does not fail.

## Migration Plan

This pull request adds only `openspec/changes/add-modifiers/`. The implementation edits `modifiers.rs`, `combatant.rs`, `check.rs`, and `lib.rs`, updates the three damage tests in place, adds the scenario tests, adds the Unreleased changelog bullet, runs the proof commands, and ends with `openspec archive add-modifiers --yes`. Rollback of that later pull request is reverting it.

## Open Questions

None. The floor-before-doubling order and the algebraic higher-value comparison are decisions recorded above, with Designer sign-off still pending.
