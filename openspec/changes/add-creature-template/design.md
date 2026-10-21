# Design

## Context

`crates/rules` has no creature type yet. On `v1-reference`, `crates/rules/src/combatant.rs` bakes HP, AC, Perception, Speed, one strike, an optional shield, and an optional `stealth_dc`, then runs them through `prepare_combatant`. That strike has no agile flag. The shield has no AC bonus; `defense_ac` in `encounter.rs` adds a flat +2 when the shield is raised. Aid rolls the strike's attack bonus, not a stored modifier. Every creature that hits 0 HP enters dying. `CombatantId(u32)` lives on `encounter.rs`. See the proposal for the dying flag and for leaving `add-modifiers` unwritten. Aid checks call `dice-and-checks`, so that change is applied first.

## Goals / Non-Goals

**Goals:**

- One public baked template, validated, with `CombatantId(u32)`.
- A 0 HP branch that is either dead or "enter dying," with no dying math.
- Name the one v1 test to port and the v1 tests to leave behind.

**Non-Goals:**

- The dying, wounded, recovery, and massive-damage track. Those are encounter-core. This change only chooses dead versus enter-dying.
- Aid's prepared-to-help state, the Aid reaction, and adjacency. Those are the later Aid change.
- Raise a Shield, Shield Block, `raised`, cover, and the flat +2.
- `Side`, level, class features, `shield_block`, synthetics, and a fixed party of 4.
- Circumstance and status buckets (`add-modifiers`).

## Decisions

### Module layout

```text
crates/rules/src/lib.rs          mod combatant; pub use the items below
crates/rules/src/combatant.rs    CombatantId, CreatureTemplate, strike, shield
```

`combatant.rs` calls `crate::check::{degree_of_success, resolve_check, Degree, CheckRoll}` and `crate::dice::Dice`. It does not declare its own degree enum.

### Public types and signatures

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CombatantId(pub u32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MeleeStrike {
    pub attack_bonus: i32,
    pub damage_dice: u8,
    pub damage_faces: u8,
    pub damage_bonus: i32,
    pub reach_feet: u32,
    pub agile: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShieldTemplate {
    pub hardness: i32,
    pub hit_points: i32,
    pub broken_threshold: i32,
    pub shield_ac_bonus: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateParts {
    pub id: CombatantId,
    pub name: String,
    pub hit_points: i32,
    pub armor_class: i32,
    pub perception: i32,
    pub speed_feet: i32,
    pub strike: MeleeStrike,
    pub shield: Option<ShieldTemplate>,
    pub stealth_dc: Option<i32>,
    pub aid_modifier: Option<i32>,
    /// None stores false.
    pub uses_dying_rules: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatureTemplate {
    pub id: CombatantId,
    pub name: String,
    pub hit_points: i32,
    pub armor_class: i32,
    pub perception: i32,
    pub speed_feet: i32,
    pub strike: MeleeStrike,
    pub shield: Option<ShieldTemplate>,
    pub stealth_dc: Option<i32>,
    pub aid_modifier: Option<i32>,
    pub uses_dying_rules: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplateError {
    NonPositiveHitPoints,
    NegativeSpeed,
}

impl CreatureTemplate {
    pub fn try_new(parts: TemplateParts) -> Result<Self, TemplateError>;
}

pub fn stealth_dc_from_modifier(stealth_modifier: i32) -> i32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZeroHp {
    Dead,
    EnterDying,
}

pub fn apply_dice_damage(
    uses_dying_rules: bool,
    current_hp: i32,
    dice: &mut impl Dice,
    faces: u8,
    bonus: i32,
) -> (i32, Option<ZeroHp>);

pub fn aid_circumstance_bonus(degree: Degree) -> i32;

pub fn aid_check(dice: &mut impl Dice, aid_modifier: i32) -> (CheckRoll, i32);
```

`try_new` returns `NonPositiveHitPoints` when `hit_points <= 0` and `NegativeSpeed` when `speed_feet < 0`. Speed 0 is stored. `uses_dying_rules: None` becomes `false`. There is no `prepared` field and no `Default` that invents a party of four. Rosters are `&[CreatureTemplate]` or `Vec<CreatureTemplate>` of any length. Ids are not restricted to `0..4`. Uniqueness is not checked here.

`stealth_dc_from_modifier` returns `10 + stealth_modifier`. The template stores that DC, not the modifier. Seek itself is a later change; this function is how the baked number is produced. An omitted `stealth_dc` stays `None`.

`aid_check` calls `resolve_check(dice, aid_modifier, 15)` and then `aid_circumstance_bonus`. The bonus is +2, +1, 0, or -1 for critical success, success, failure, and critical failure. +2 is not scaled. The four-part deviation (rule, why, substitute, intent) is recorded in the spec and repeated in the test comment. Do not add a proficiency rank argument.

`apply_dice_damage` rolls `faces` once through `Dice`, adds `bonus`, and subtracts from `current_hp`. The stored Hit Points do not go below 0. If the result is still above 0, the `Option` is `None`. If it is 0 and `uses_dying_rules` is false, return `(0, Some(Dead))` and do not set a dying value. If it is 0 and the flag is true, return `(0, Some(EnterDying))` and do not choose dying 1 or dying 2. Encounter-core owns that number.

Shield AC bonus is the stored `shield_ac_bonus`. Nothing in this module adds 2.

### What is ported from v1-reference, and what changes

Port the `CombatantId(pub u32)` newtype from `crates/rules/src/encounter.rs`, including `Display`, `Hash`, `PartialOrd`, and `Ord`. Do not port `Combatant`, the encounter map, or id assignment that fills party slots `0..4`.

From `crates/rules/src/combatant.rs`, keep the idea of authored HP, AC, Perception, Speed, one strike's attack bonus and damage dice, reach, optional shield hardness / Hit Points / broken threshold, and optional `stealth_dc`. Changes on the way in:

- Add `agile: bool` on the strike. v1's `StrikeTemplate` has no trait flag.
- Add `shield_ac_bonus`. Drop `raised` and `max_hit_points`. Starting shield Hit Points are `hit_points`. Do not copy the flat +2 in `defense_ac`.
- Change Speed from `u32` to `i32` so a negative value can be rejected.
- Add `aid_modifier: Option<i32>`. Do not use the strike attack bonus as the Aid modifier.
- Add `uses_dying_rules: bool`, default false.
- Drop `level`, `Side`, `ClassFeature`, `shield_block`, `Synthetics`, `apply_effects`, and `prepare_combatant`. Baked fields are the values. There is no derivation pass.

### v1 tests that become acceptance tests

- Port `preparation_copies_fixture_numbers` from `v1-reference` `crates/rules/tests/procedures.rs`. The v1 body calls `prepare_combatant` on the fighter fixture and asserts AC 18, Speed 25, Perception 5, attack bonus 7, damage faces 8, and Hit Points 20. Rebuild that as `baked_statistics_stay_as_authored` through `try_new`, with the same numbers, plus damage dice 1, damage bonus +4, reach 5, `agile: false`, `uses_dying_rules: false`, no shield, and no stealth or Aid modifier. Cite Player Core for the baked statistics. Do not call `prepare_combatant`.

New tests, because v1 has no equivalent that matches these rules:

- `shield_ac_bonus_is_not_rewritten_to_plus_two` for bonuses +1 and +2.
- `stealth_dc_from_modifier_is_ten_plus` (`+7` becomes 17) and an omitted DC.
- `aid_check_degrees_use_the_fixed_critical_bonus` with scripted naturals 10, 18, 4, 1, and 20 against DC 15.
- `any_roster_length_is_valid` for one template and for six, including `CombatantId(4000000000)`.
- `validation_rejects_non_positive_hp_and_negative_speed`, and Speed 0 is accepted.
- `template_does_not_store_aid_preparation`.
- `zero_hp_without_dying_rules_is_dead` and `zero_hp_with_dying_rules_enters_dying`, each with a scripted d8 of 8 on 1d8+1 from 9 Hit Points. A scripted d8 of 1 leaves 7 Hit Points.

Do not port these v1 tests:

- `a_hit_that_drops_a_creature_sets_dying_and_wounded`
- `a_critical_hit_that_drops_a_creature_sets_dying_two_plus_wounded`
- `wounded_increases_the_next_dying_value`
- `damage_while_dying_increases_dying_without_wounded`
- `dying_four_kills_and_a_dead_party_slot_remains`
- `recovery_checks_cover_each_degree`
- `massive_damage_kills_without_dying`

Those assert the dying track. Encounter-core ports them, and v1's wounded timing is already known to be wrong.

- `raise_shield_spends_one_action_and_adds_two_ac`. It hardcodes +2. This change stores the shield's own bonus and does not raise the shield.
- The `aid_*` tests in `procedures.rs` (`aid_success_gives_plus_one_including_a_diagonal_neighbor`, `aid_critical_success_gives_plus_two`, and the rest). They need the reaction and the prepared state, which this change does not store.

## Risks / Trade-offs

- [Public fields can skip `try_new`] → loaders and tests construct through `try_new`. A later change can seal the fields if a literal bypass shows up.
- [`EnterDying` has no dying value] → encounter-core must not treat that arm as "already dying 1." This change only refuses to mark the creature dead.
- [Aid calls `resolve_check`] → applying this change before `dice-and-checks` will not compile. Apply dice and checks first.
