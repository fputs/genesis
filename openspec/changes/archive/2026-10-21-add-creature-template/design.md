# Design

## Context

`crates/rules` has no creature type yet. On `v1-reference`, `prepare_combatant` copies a `CreatureTemplate` into derived stats and the encounter then stores current Hit Points on the same combatant. That mixes the build with runtime state. This change separates them. `derive` does not use Dice. Aid checks and damage rolls do, through `dice-and-checks`, so apply that change first. The rules crate still has no serde. See the proposal for `uses_dying_rules` and for leaving `add-modifiers` unwritten.

## Goals / Non-Goals

**Goals:**

- `CharacterBuild` -> pure `derive` -> `Creature`, then `CombatantState` keyed by `CombatantId`.
- A contribution list so later sources can be added without changing `derive`'s callers.
- The same build always yields the same Creature. Mutating state never writes the build or the Creature.

**Non-Goals:**

- Ancestry, class, feat, attribute, and proficiency fields. Phase 1 does not add them. The contribution list is the slot they will use.
- Serde. Equality is `Clone` + `PartialEq`. "Serialized-equal" means a build that compares equal, not a format.
- The dying value, wounded changes, recovery, and massive damage. Encounter-core owns those. This change only chooses dead versus enter-dying on the state.
- Aid's prepared state, the Aid reaction, and adjacency.
- Raise a Shield, Shield Block, and the flat +2.
- `Side`, level, class features, `shield_block`, synthetics, and a fixed party of 4.
- Circumstance and status buckets (`add-modifiers`).

## Decisions

### Module layout

```text
crates/rules/src/lib.rs          mod combatant; pub use the items below
crates/rules/src/combatant.rs    CharacterBuild, derive, Creature, CombatantState
```

`combatant.rs` calls `crate::check::{resolve_check, Degree, CheckRoll}` and `crate::dice::Dice` only from Aid and from applying damage to a state. `derive` does not call them.

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
pub struct BuildParts {
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
pub struct CharacterBuild { /* private fields, no setters */ }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplateError {
    NonPositiveHitPoints,
    NegativeSpeed,
}

impl CharacterBuild {
    pub fn try_new(parts: BuildParts) -> Result<Self, TemplateError>;
}

/// One layer. Numerics are added. `Option` fields replace when `Some`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contribution {
    pub hit_points: i32,
    pub armor_class: i32,
    pub perception: i32,
    pub speed_feet: i32,
    pub attack_bonus: i32,
    pub damage_dice: i32,
    pub damage_faces: i32,
    pub damage_bonus: i32,
    pub reach_feet: i32,
    pub agile: Option<bool>,
    pub shield: Option<ShieldTemplate>,
    pub stealth_dc: Option<i32>,
    pub aid_modifier: Option<i32>,
    pub uses_dying_rules: Option<bool>,
}

pub fn base_contribution(build: &CharacterBuild) -> Contribution;
pub fn contributions(build: &CharacterBuild) -> Vec<Contribution>;
pub fn fold_contributions(ordered: &[Contribution]) -> Creature;
pub fn derive(build: &CharacterBuild) -> Creature;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Creature {
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
pub enum Vital {
    Alive,
    Dead,
    EnterDying,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombatantState {
    pub id: CombatantId,
    pub hit_points: i32,
    pub temp_hit_points: i32,
    pub conditions: Vec<ConditionSlot>,
    pub dying_entered: bool,
    pub wounded: i32,
    pub shield_hit_points: Option<i32>,
    pub shield_raised: bool,
    pub actions_remaining: u8,
    pub reaction_ready: bool,
    pub vital: Vital,
}

/// Empty in Phase 1. The vec is the slot later conditions will fill.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConditionSlot;

pub fn new_combatant_state(id: CombatantId, creature: &Creature) -> CombatantState;

pub fn apply_dice_damage(
    state: &mut CombatantState,
    creature: &Creature,
    dice: &mut impl Dice,
    faces: u8,
    bonus: i32,
);

pub fn stealth_dc_from_modifier(stealth_modifier: i32) -> i32;
pub fn aid_circumstance_bonus(degree: Degree) -> i32;
pub fn aid_check(dice: &mut impl Dice, creature: &Creature) -> Option<(CheckRoll, i32)>;
```

`try_new` returns `NonPositiveHitPoints` when `hit_points <= 0` and `NegativeSpeed` when `speed_feet < 0`. Speed 0 is stored. `uses_dying_rules: None` becomes false. There is no `CombatantId` on the build and no prepared-to-help field. `CharacterBuild` has private fields and no mutating methods. `derive` takes `&CharacterBuild`.

`contributions` returns `vec![base_contribution(build)]` in Phase 1. `base_contribution` copies the authored numbers as the first layer: Hit Points, Armor Class, Perception, Speed, strike numbers, `Some(agile)`, and the optional shield, stealth DC, Aid modifier, and `uses_dying_rules`. `derive` is `fold_contributions(&contributions(build))`.

`fold_contributions` starts from zeros and `None`, then walks the slice in order. It adds every numeric field. `Some` on agile, shield, stealth DC, Aid modifier, or `uses_dying_rules` replaces the current value. A later source is a `Contribution` appended inside `contributions`. Callers of `derive` do not gain a parameter. A test may call `fold_contributions` with a base layer plus `{ armor_class: 1, ..zero }` and get Armor Class 19. That is the extension point, not a Phase 1 character option.

`Creature` is the read-only result. Its Hit Points are the maximum. Its shield Hit Points are the maximum. It has no current Hit Points, temporary Hit Points, conditions, dying, wounded, raised flag, actions, or reaction.

`new_combatant_state` copies maximum Hit Points into `hit_points`, sets `temp_hit_points` to 0, `conditions` to empty, `dying_entered` to false, `wounded` to 0, `vital` to `Alive`, `actions_remaining` to 0, and `reaction_ready` to false. If the Creature has a shield, `shield_hit_points` is that maximum and `shield_raised` is false. Otherwise `shield_hit_points` is `None`. Encounter-core is what sets actions to 3 and readies the reaction at the start of a turn. Two calls with the same id and the same Creature compare equal. The id is not restricted to `0..4`. `CombatantId(4000000000)` is a valid `u32`.

`apply_dice_damage` rolls `faces` once, adds `bonus`, and subtracts from `state.hit_points`, floored at 0. It reads `creature.uses_dying_rules` and does not write `creature` or any `CharacterBuild`. Above 0, `vital` stays `Alive`. At 0 with the flag false, `vital` is `Dead` and `dying_entered` stays false. At 0 with the flag true, `vital` is `EnterDying` and `dying_entered` is true. No dying number is chosen.

`stealth_dc_from_modifier` returns `10 + stealth_modifier` and is used while filling `BuildParts`, not inside `derive`. `aid_check` returns `None` when the Creature has no Aid modifier. Otherwise it calls `resolve_check(dice, modifier, 15)` and maps the degree to +2, +1, 0, or -1. It does not take `&mut Creature`.

No function in this module takes `&mut CharacterBuild` or `&mut Creature`.

### What is ported from v1-reference, and what changes

Port the `CombatantId(pub u32)` newtype from `crates/rules/src/encounter.rs`, including `Display`, `Hash`, `PartialOrd`, and `Ord`. Do not port `Combatant` or the assignment of party slots `0..4`. The id keys `CombatantState` only.

From `crates/rules/src/combatant.rs`, keep authored HP, AC, Perception, Speed, one strike's attack bonus and damage dice, reach, optional shield hardness / Hit Points / broken threshold, and optional `stealth_dc`. Changes on the way in:

- Those numbers are `CharacterBuild` inputs. `derive` produces a `Creature`. Do not port `prepare_combatant`, `Synthetics`, or `apply_effects`.
- Add `agile: bool`. v1's `StrikeTemplate` has no trait flag.
- Add `shield_ac_bonus`. Drop `raised` and `max_hit_points` from the build. Raised and current shield Hit Points are on `CombatantState`.
- Change Speed from `u32` to `i32` so a negative value can be rejected.
- Add `aid_modifier: Option<i32>` on the build and the Creature. Do not use the strike attack bonus as the Aid modifier.
- Add `uses_dying_rules`, default false, copied onto the Creature. The 0 HP branch writes `CombatantState` only.
- Drop `level`, `Side`, `ClassFeature`, and `shield_block`.

### v1 tests that become acceptance tests

- Port `preparation_copies_fixture_numbers` from `v1-reference` `crates/rules/tests/procedures.rs` as `baked_statistics_stay_as_authored`. The v1 body calls `prepare_combatant` and asserts AC 18, Speed 25, Perception 5, attack bonus 7, damage faces 8, and Hit Points 20. Rebuild it as `try_new` plus `derive`, with those numbers plus damage dice 1, damage bonus +4, reach 5, `agile: false`, `uses_dying_rules: false`, no shield, and no stealth or Aid modifier. Assert the Creature, and assert the build is unchanged. Cite Player Core for the baked statistics.

New tests:

- `derive_is_deterministic`: two `derive` calls, a clone, and an equal rebuild match, and a scripted d20 is unused.
- `later_contribution_folds_after_the_base`: base AC 18 then +1 folds to 19, and `derive` still takes only `&CharacterBuild`.
- `shield_ac_bonus_is_not_rewritten_to_plus_two`.
- `stealth_dc_from_modifier_is_ten_plus` (`+7` becomes 17).
- `aid_check_degrees_use_the_fixed_critical_bonus` with scripted naturals 10, 18, 4, 1, and 20. The Creature compares equal after the check. The comment records rule, why, substitute, and intent.
- `fresh_state_matches_and_mutation_does_not_change_creature`.
- `any_roster_length_is_valid` for six states and `CombatantId(4000000000)`.
- `validation_rejects_non_positive_hp_and_negative_speed`.
- `template_does_not_store_aid_preparation`.
- `zero_hp_without_dying_rules_is_dead` and `zero_hp_with_dying_rules_enters_dying`, scripted d8 of 8 on 1d8+1 from 9 Hit Points, asserting the Creature still has 9. A scripted d8 of 1 leaves the state at 7.

Do not port these v1 tests:

- `a_hit_that_drops_a_creature_sets_dying_and_wounded`
- `a_critical_hit_that_drops_a_creature_sets_dying_two_plus_wounded`
- `wounded_increases_the_next_dying_value`
- `damage_while_dying_increases_dying_without_wounded`
- `dying_four_kills_and_a_dead_party_slot_remains`
- `recovery_checks_cover_each_degree`
- `massive_damage_kills_without_dying`
- `raise_shield_spends_one_action_and_adds_two_ac`
- The `aid_*` encounter tests in `procedures.rs` (`aid_success_gives_plus_one_including_a_diagonal_neighbor`, `aid_critical_success_gives_plus_two`, and the rest)

## Risks / Trade-offs

- [Fold starts at zero and adds] → the base contribution must carry the full authored numbers, not a delta on top of a hidden default. A test with one contribution locks that.
- [`EnterDying` has no dying number] → encounter-core must not treat `dying_entered` as dying 1.
- [No serde] → equal builds are `PartialEq`, which is what "serialized-equal" can mean in this crate. A format can wait.
- [Aid and damage call Dice] → `derive` must not. The purity test leaves a scripted die unused.
