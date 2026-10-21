use std::fmt;

use crate::check::{CheckRoll, Degree, resolve_check};
use crate::dice::Dice;

/// Stable id for a creature in one encounter. Rules never use engine entity ids.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CombatantId(pub u32);

impl fmt::Display for CombatantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// One melee strike on a creature.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MeleeStrike {
    pub attack_bonus: i32,
    pub damage_dice: u8,
    pub damage_faces: u8,
    pub damage_bonus: i32,
    pub reach_feet: u32,
    pub agile: bool,
}

/// Shield numbers stored on the build. The AC bonus is this shield's own value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShieldTemplate {
    pub hardness: i32,
    pub hit_points: i32,
    pub broken_threshold: i32,
    pub shield_ac_bonus: i32,
}

/// Phase 1 inputs for one character. `uses_dying_rules: None` stores false.
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

/// Immutable inputs for one character. No setters, and no combatant id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterBuild {
    parts: BuildParts,
}

/// Why a character build was rejected. A rejected build is not derived.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplateError {
    NonPositiveHitPoints,
    NegativeSpeed,
}

impl CharacterBuild {
    pub fn try_new(parts: BuildParts) -> Result<Self, TemplateError> {
        if parts.hit_points <= 0 {
            return Err(TemplateError::NonPositiveHitPoints);
        }
        if parts.speed_feet < 0 {
            return Err(TemplateError::NegativeSpeed);
        }
        Ok(Self { parts })
    }
}

/// One derivation layer. Numerics are added. `Option` fields replace when `Some`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

/// Copies the authored numbers as the first layer.
pub fn base_contribution(build: &CharacterBuild) -> Contribution {
    let parts = &build.parts;
    Contribution {
        hit_points: parts.hit_points,
        armor_class: parts.armor_class,
        perception: parts.perception,
        speed_feet: parts.speed_feet,
        attack_bonus: parts.strike.attack_bonus,
        damage_dice: i32::from(parts.strike.damage_dice),
        damage_faces: i32::from(parts.strike.damage_faces),
        damage_bonus: parts.strike.damage_bonus,
        reach_feet: i32::try_from(parts.strike.reach_feet).expect("reach fits in i32"),
        agile: Some(parts.strike.agile),
        shield: parts.shield.clone(),
        stealth_dc: parts.stealth_dc,
        aid_modifier: parts.aid_modifier,
        uses_dying_rules: parts.uses_dying_rules,
    }
}

/// Phase 1 supplies only the base contribution.
pub fn contributions(build: &CharacterBuild) -> Vec<Contribution> {
    vec![base_contribution(build)]
}

/// Folds layers in order, starting from zeros and `None`.
pub fn fold_contributions(ordered: &[Contribution]) -> Creature {
    let mut hit_points = 0;
    let mut armor_class = 0;
    let mut perception = 0;
    let mut speed_feet = 0;
    let mut attack_bonus = 0;
    let mut damage_dice = 0;
    let mut damage_faces = 0;
    let mut damage_bonus = 0;
    let mut reach_feet = 0;
    let mut agile = None;
    let mut shield = None;
    let mut stealth_dc = None;
    let mut aid_modifier = None;
    let mut uses_dying_rules = None;

    for layer in ordered {
        hit_points += layer.hit_points;
        armor_class += layer.armor_class;
        perception += layer.perception;
        speed_feet += layer.speed_feet;
        attack_bonus += layer.attack_bonus;
        damage_dice += layer.damage_dice;
        damage_faces += layer.damage_faces;
        damage_bonus += layer.damage_bonus;
        reach_feet += layer.reach_feet;
        if let Some(value) = layer.agile {
            agile = Some(value);
        }
        if layer.shield.is_some() {
            shield.clone_from(&layer.shield);
        }
        if let Some(value) = layer.stealth_dc {
            stealth_dc = Some(value);
        }
        if let Some(value) = layer.aid_modifier {
            aid_modifier = Some(value);
        }
        if let Some(value) = layer.uses_dying_rules {
            uses_dying_rules = Some(value);
        }
    }

    Creature {
        hit_points,
        armor_class,
        perception,
        speed_feet,
        strike: MeleeStrike {
            attack_bonus,
            damage_dice: u8::try_from(damage_dice).expect("folded damage dice fit in u8"),
            damage_faces: u8::try_from(damage_faces).expect("folded damage faces fit in u8"),
            damage_bonus,
            reach_feet: u32::try_from(reach_feet).expect("folded reach fits in u32"),
            agile: agile.unwrap_or(false),
        },
        shield,
        stealth_dc,
        aid_modifier,
        uses_dying_rules: uses_dying_rules.unwrap_or(false),
    }
}

/// Pure derivation. Does not roll dice, do I/O, or read global state.
pub fn derive(build: &CharacterBuild) -> Creature {
    fold_contributions(&contributions(build))
}

/// Read-only result of derivation. Hit Points and shield Hit Points are maximums.
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

/// What 0 current Hit Points means for this combatant. Encounter-core owns the dying value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vital {
    Alive,
    Dead,
    EnterDying,
}

/// Empty in Phase 1. The vec is the slot later conditions will fill.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConditionSlot;

/// Runtime combat data, keyed by [`CombatantId`]. Never written back onto the build or the creature.
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

/// Initializes state from a creature. Actions and the reaction stay unset for encounter-core.
pub fn new_combatant_state(id: CombatantId, creature: &Creature) -> CombatantState {
    CombatantState {
        id,
        hit_points: creature.hit_points,
        temp_hit_points: 0,
        conditions: Vec::new(),
        dying_entered: false,
        wounded: 0,
        shield_hit_points: creature.shield.as_ref().map(|shield| shield.hit_points),
        shield_raised: false,
        actions_remaining: 0,
        reaction_ready: false,
        vital: Vital::Alive,
    }
}

/// Rolls `faces` once, adds `bonus`, and subtracts that from current Hit Points, floored at 0.
///
/// At 0 Hit Points, `uses_dying_rules` false marks the state dead. True records that dying applies
/// and does not choose a dying value. The creature and the build are not written.
pub fn apply_dice_damage(
    state: &mut CombatantState,
    creature: &Creature,
    dice: &mut impl Dice,
    faces: u8,
    bonus: i32,
) {
    let amount = i32::from(dice.roll(faces)) + bonus;
    state.hit_points = (state.hit_points - amount).max(0);
    if state.hit_points > 0 {
        state.vital = Vital::Alive;
    } else if creature.uses_dying_rules {
        state.vital = Vital::EnterDying;
        state.dying_entered = true;
    } else {
        state.vital = Vital::Dead;
    }
}

/// Stealth DC placed on a build: 10 plus the Stealth modifier. Not used inside [`derive`].
pub fn stealth_dc_from_modifier(stealth_modifier: i32) -> i32 {
    crate::check::dc_from_modifier(stealth_modifier)
}

/// Circumstance bonus from an Aid degree. Critical success is the fixed +2.
pub fn aid_circumstance_bonus(degree: Degree) -> i32 {
    match degree {
        Degree::CriticalSuccess => 2,
        Degree::Success => 1,
        Degree::Failure => 0,
        Degree::CriticalFailure => -1,
    }
}

/// Aid check against DC 15 using the creature's Aid modifier. `None` when the creature has none.
pub fn aid_check(dice: &mut impl Dice, creature: &Creature) -> Option<(CheckRoll, i32)> {
    let modifier = creature.aid_modifier?;
    let roll = resolve_check(dice, modifier, 15);
    let bonus = aid_circumstance_bonus(roll.degree);
    Some((roll, bonus))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::{Degree, degree_of_success, resolve_check};
    use crate::dice::ScriptedDice;

    fn strike(
        attack: i32,
        dice: u8,
        faces: u8,
        bonus: i32,
        reach: u32,
        agile: bool,
    ) -> MeleeStrike {
        MeleeStrike {
            attack_bonus: attack,
            damage_dice: dice,
            damage_faces: faces,
            damage_bonus: bonus,
            reach_feet: reach,
            agile,
        }
    }

    fn fighter_parts() -> BuildParts {
        BuildParts {
            hit_points: 20,
            armor_class: 18,
            perception: 5,
            speed_feet: 25,
            strike: strike(7, 1, 8, 4, 5, false),
            shield: None,
            stealth_dc: None,
            aid_modifier: None,
            uses_dying_rules: None,
        }
    }

    fn build(parts: BuildParts) -> CharacterBuild {
        CharacterBuild::try_new(parts).expect("valid build")
    }

    /// Player Core, Hit Points are fixed by the creature, not by a roll.
    #[test]
    fn derive_is_deterministic() {
        let parts = fighter_parts();
        let original = build(parts.clone());
        let dice = ScriptedDice::new([8]);
        let first = derive(&original);
        let second = derive(&original);
        assert_eq!(first, second);
        assert_eq!(derive(&original.clone()), first);
        let rebuilt = build(parts);
        assert_eq!(original, rebuilt);
        assert_eq!(derive(&rebuilt), first);
        assert_eq!(dice.remaining(), 1);
    }

    #[test]
    fn later_contribution_folds_after_the_base() {
        let mut parts = fighter_parts();
        parts.stealth_dc = Some(17);
        let original = build(parts);
        let layers = contributions(&original);
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0], base_contribution(&original));
        assert_eq!(derive(&original).armor_class, 18);

        let mut ordered = layers;
        // Stealth is replaced, not added. A reversed fold would keep the base DC of 17.
        ordered.push(Contribution {
            armor_class: 1,
            stealth_dc: Some(4),
            ..Contribution::default()
        });
        assert_eq!(ordered[0].armor_class, 18);
        assert_eq!(ordered[1].armor_class, 1);
        let folded = fold_contributions(&ordered);
        assert_eq!(folded.armor_class, 19);
        assert_eq!(folded.stealth_dc, Some(4));
        assert_eq!(derive(&original).armor_class, 18);
    }

    /// Player Core, Hit Points.
    /// Player Core, Strike.
    /// Player Core, Speed.
    #[test]
    fn baked_statistics_stay_as_authored() {
        let original = build(fighter_parts());
        let before = original.clone();
        let creature = derive(&original);
        assert_eq!(original, before);
        assert_eq!(creature.hit_points, 20);
        assert_eq!(creature.armor_class, 18);
        assert_eq!(creature.perception, 5);
        assert_eq!(creature.speed_feet, 25);
        assert_eq!(creature.strike.attack_bonus, 7);
        assert_eq!(creature.strike.damage_dice, 1);
        assert_eq!(creature.strike.damage_faces, 8);
        assert_eq!(creature.strike.damage_bonus, 4);
        assert_eq!(creature.strike.reach_feet, 5);
        assert!(!creature.strike.agile);
        assert!(creature.shield.is_none());
        assert_eq!(creature.stealth_dc, None);
        assert_eq!(creature.aid_modifier, None);
        assert!(!creature.uses_dying_rules);

        let mut agile_parts = fighter_parts();
        agile_parts.strike = strike(6, 1, 6, 2, 5, true);
        let agile = derive(&build(agile_parts));
        assert!(agile.strike.agile);
        assert_eq!(agile.strike.attack_bonus, 6);
        assert_eq!(agile.strike.damage_dice, 1);
        assert_eq!(agile.strike.damage_faces, 6);
        assert_eq!(agile.strike.damage_bonus, 2);
        assert_eq!(agile.strike.reach_feet, 5);
    }

    /// Player Core, "Shield Statistics". A buckler grants +1.
    #[test]
    fn shield_ac_bonus_is_not_rewritten_to_plus_two() {
        let buckler = ShieldTemplate {
            hardness: 3,
            hit_points: 8,
            broken_threshold: 4,
            shield_ac_bonus: 1,
        };
        let mut buckler_parts = fighter_parts();
        buckler_parts.shield = Some(buckler.clone());
        let creature = derive(&build(buckler_parts));
        assert_eq!(creature.shield, Some(buckler));

        let steel = ShieldTemplate {
            hardness: 5,
            hit_points: 20,
            broken_threshold: 10,
            shield_ac_bonus: 2,
        };
        let mut steel_parts = fighter_parts();
        steel_parts.shield = Some(steel.clone());
        assert_eq!(derive(&build(steel_parts)).shield, Some(steel));

        assert!(derive(&build(fighter_parts())).shield.is_none());
    }

    /// Player Core, "Seek".
    /// Player Core, "Getting a DC from a Modifier".
    #[test]
    fn stealth_dc_from_modifier_is_ten_plus() {
        assert_eq!(stealth_dc_from_modifier(7), 17);

        let mut parts = fighter_parts();
        parts.stealth_dc = Some(stealth_dc_from_modifier(7));
        let creature = derive(&build(parts));
        assert_eq!(creature.stealth_dc, Some(17));

        let before = creature.clone();
        let mut dice = ScriptedDice::new([12]);
        let seek = resolve_check(&mut dice, creature.perception, creature.stealth_dc.unwrap());
        assert_eq!(seek.total, 17);
        assert_eq!(seek.degree, Degree::Success);
        assert_eq!(creature, before);
        assert_eq!(dice.remaining(), 0);

        assert_eq!(derive(&build(fighter_parts())).stealth_dc, None);
    }

    /// Player Core, "Aid".
    ///
    /// Deviation:
    /// - Rule: critical success grants +2 circumstance, +3 at master, and +4 at legendary.
    /// - Why: the baked modifier has no proficiency rank, so master and legendary cannot be chosen.
    /// - Substitute: critical success on this Aid check always grants +2 circumstance.
    /// - Intent: a critical success still helps more than a success.
    #[test]
    fn aid_check_degrees_use_the_fixed_critical_bonus() {
        let mut parts = fighter_parts();
        parts.aid_modifier = Some(7);
        let creature = derive(&build(parts));
        let before = creature.clone();

        let (success, success_bonus) = aid_check(&mut ScriptedDice::new([10]), &creature).unwrap();
        assert_eq!(success.total, 17);
        assert_eq!(success.degree, Degree::Success);
        assert_eq!(success_bonus, 1);
        assert_eq!(creature, before);

        let (critical, critical_bonus) =
            aid_check(&mut ScriptedDice::new([18]), &creature).unwrap();
        assert_eq!(critical.total, 25);
        assert_eq!(critical.degree, Degree::CriticalSuccess);
        assert_eq!(critical_bonus, 2);

        let (failure, failure_bonus) = aid_check(&mut ScriptedDice::new([4]), &creature).unwrap();
        assert_eq!(failure.total, 11);
        assert_eq!(failure.degree, Degree::Failure);
        assert_eq!(failure_bonus, 0);

        let mut crit_fail_parts = fighter_parts();
        crit_fail_parts.aid_modifier = Some(4);
        let crit_fail_creature = derive(&build(crit_fail_parts));
        let (crit_fail, crit_fail_bonus) =
            aid_check(&mut ScriptedDice::new([1]), &crit_fail_creature).unwrap();
        assert_eq!(crit_fail.total, 5);
        assert_eq!(crit_fail.degree, Degree::CriticalFailure);
        assert_eq!(crit_fail_bonus, -1);
        assert_eq!(degree_of_success(10, 5, 15), Degree::CriticalFailure);

        let mut nat_20_parts = fighter_parts();
        nat_20_parts.aid_modifier = Some(0);
        let nat_20_creature = derive(&build(nat_20_parts));
        let (nat_20, nat_20_bonus) =
            aid_check(&mut ScriptedDice::new([20]), &nat_20_creature).unwrap();
        assert_eq!(nat_20.total, 20);
        assert_eq!(degree_of_success(10, 20, 15), Degree::Success);
        assert_eq!(nat_20.degree, Degree::CriticalSuccess);
        assert_eq!(nat_20_bonus, 2);
        assert_eq!(nat_20_creature, derive(&build(fighter_with_aid(0))));

        assert!(
            aid_check(
                &mut ScriptedDice::new([10]),
                &derive(&build(fighter_parts()))
            )
            .is_none()
        );
    }

    fn fighter_with_aid(modifier: i32) -> BuildParts {
        let mut parts = fighter_parts();
        parts.aid_modifier = Some(modifier);
        parts
    }

    #[test]
    fn template_does_not_store_aid_preparation() {
        let creature = derive(&build(fighter_with_aid(6)));
        assert_eq!(creature.aid_modifier, Some(6));
    }

    #[test]
    fn fresh_state_matches_and_mutation_does_not_change_creature() {
        let mut parts = fighter_parts();
        parts.shield = Some(ShieldTemplate {
            hardness: 3,
            hit_points: 8,
            broken_threshold: 4,
            shield_ac_bonus: 1,
        });
        let original = build(parts);
        let before_build = original.clone();
        let creature = derive(&original);
        let before_creature = creature.clone();

        let first = new_combatant_state(CombatantId(7), &creature);
        let second = new_combatant_state(CombatantId(7), &creature);
        assert_eq!(first, second);
        assert_eq!(first.hit_points, creature.hit_points);
        assert_eq!(first.temp_hit_points, 0);
        assert!(!first.shield_raised);
        assert_eq!(first.shield_hit_points, Some(8));
        assert_eq!(first.actions_remaining, 0);
        assert!(!first.reaction_ready);
        assert_eq!(first.vital, Vital::Alive);
        assert!(first.conditions.is_empty());
        assert_eq!(CombatantId(7).to_string(), "7");

        let mut state = new_combatant_state(CombatantId(0), &creature);
        state.hit_points = 12;
        assert_eq!(creature, before_creature);
        assert_eq!(creature.hit_points, 20);
        assert_eq!(original, before_build);
    }

    #[test]
    fn any_roster_length_is_valid() {
        let original = build(fighter_parts());
        let creature = derive(&original);
        let before = creature.clone();
        let roster: Vec<_> = (0..6)
            .map(|id| new_combatant_state(CombatantId(id), &creature))
            .collect();
        assert_eq!(roster.len(), 6);
        assert_eq!(roster[0].id, CombatantId(0));
        assert_eq!(roster[5].id, CombatantId(5));

        let wide = new_combatant_state(CombatantId(4_000_000_000), &creature);
        let also = new_combatant_state(CombatantId(0), &creature);
        assert_eq!(wide.id, CombatantId(4_000_000_000));
        assert_eq!(also.id, CombatantId(0));
        assert_eq!(wide.id.to_string(), "4000000000");
        assert_eq!(creature, before);
    }

    /// Player Core, "Getting Knocked Out".
    /// Player Core, "Movement".
    #[test]
    fn validation_rejects_non_positive_hp_and_negative_speed() {
        let mut zero_hp = fighter_parts();
        zero_hp.hit_points = 0;
        assert_eq!(
            CharacterBuild::try_new(zero_hp).unwrap_err(),
            TemplateError::NonPositiveHitPoints
        );

        let mut negative_hp = fighter_parts();
        negative_hp.hit_points = -1;
        assert_eq!(
            CharacterBuild::try_new(negative_hp).unwrap_err(),
            TemplateError::NonPositiveHitPoints
        );

        let mut negative_speed = fighter_parts();
        negative_speed.speed_feet = -5;
        negative_speed.hit_points = 12;
        assert_eq!(
            CharacterBuild::try_new(negative_speed).unwrap_err(),
            TemplateError::NegativeSpeed
        );

        let mut speed_zero = fighter_parts();
        speed_zero.hit_points = 12;
        speed_zero.speed_feet = 0;
        let creature = derive(&build(speed_zero));
        assert_eq!(creature.speed_feet, 0);
        assert_eq!(creature.hit_points, 12);
    }

    /// Player Core, "Getting Knocked Out".
    #[test]
    fn zero_hp_without_dying_rules_is_dead() {
        let original = build(fighter_with_hp(9, None));
        let creature = derive(&original);
        assert!(!creature.uses_dying_rules);
        assert_eq!(creature.hit_points, 9);
        let before = creature.clone();

        let mut state = new_combatant_state(CombatantId(1), &creature);
        apply_dice_damage(&mut state, &creature, &mut ScriptedDice::new([8]), 8, 1);
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.vital, Vital::Dead);
        assert!(!state.dying_entered);
        assert!(state.conditions.is_empty());
        assert_eq!(state.wounded, 0);
        assert_eq!(creature, before);

        let mut grazed = new_combatant_state(CombatantId(1), &creature);
        apply_dice_damage(&mut grazed, &creature, &mut ScriptedDice::new([1]), 8, 1);
        assert_eq!(grazed.hit_points, 7);
        assert_eq!(grazed.vital, Vital::Alive);
        assert!(!grazed.dying_entered);
        assert_eq!(creature.hit_points, 9);
    }

    /// Player Core, "Getting Knocked Out".
    #[test]
    fn zero_hp_with_dying_rules_enters_dying() {
        let original = build(fighter_with_hp(9, Some(true)));
        let creature = derive(&original);
        assert!(creature.uses_dying_rules);
        let before = creature.clone();

        let mut state = new_combatant_state(CombatantId(1), &creature);
        apply_dice_damage(&mut state, &creature, &mut ScriptedDice::new([8]), 8, 1);
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.vital, Vital::EnterDying);
        assert!(state.dying_entered);
        assert_ne!(state.vital, Vital::Dead);
        assert!(state.conditions.is_empty());
        assert_eq!(state.wounded, 0);
        assert_eq!(creature, before);
        assert_eq!(creature.hit_points, 9);
    }

    fn fighter_with_hp(hit_points: i32, uses_dying_rules: Option<bool>) -> BuildParts {
        let mut parts = fighter_parts();
        parts.hit_points = hit_points;
        parts.uses_dying_rules = uses_dying_rules;
        parts
    }
}
