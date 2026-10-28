use super::{
    actions, begin, combatant, creature, enemy, faces, hit_points, id, pc, plate, profile,
    reaction, rolled, temp_hp, vital,
};
use crate::check::DegreeOfSuccess;
use crate::combatant::{
    BuildParts, CharacterBuild, ConditionSlot, DieCount, DyingValue, MeleeStrike, ReachFeet, Vital,
    derive,
};
use crate::damage::{DamageDefenses, DamageType, Resistance, Weakness};
use crate::detection::Detections;
use crate::dice::ScriptedDice;
use crate::dying::apply_healing;
use crate::encounter::{Agility, Encounter, EncounterError, Side, StrikeProfile};
use crate::modifiers::ModifierError;
use crate::positioning::{ActionCost, ActionTrait};
use crate::reactive::{ActionIdentity, ReactiveStrike};
use crate::save::{Level, Proficiency, Save, SavingThrows};
use crate::shield::ShieldBlock;

fn defenses(
    immunities: &[DamageType],
    weaknesses: &[(DamageType, i32)],
    resistances: &[(DamageType, i32)],
) -> DamageDefenses {
    DamageDefenses {
        immunities: immunities.to_vec(),
        weaknesses: weaknesses
            .iter()
            .copied()
            .map(|(damage_type, value)| Weakness { damage_type, value })
            .collect(),
        resistances: resistances
            .iter()
            .copied()
            .map(|(damage_type, value)| Resistance { damage_type, value })
            .collect(),
    }
}

fn with_defenses(
    mut person: crate::encounter::Participant,
    defenses: DamageDefenses,
) -> crate::encounter::Participant {
    person.creature.defenses = defenses;
    person
}

fn lay(encounter: &mut Encounter, value: u32, vital_state: Vital, hp: i32) {
    encounter.set_vital_for_test(id(value), vital_state);
    let member = encounter
        .members
        .iter_mut()
        .find(|member| member.participant.id == id(value))
        .expect("member");
    member.participant.state.hit_points = hp;
}

fn persistent(encounter: &Encounter, value: u32) -> Vec<(DamageType, i32)> {
    encounter
        .state(id(value))
        .expect("state")
        .conditions
        .iter()
        .map(|slot| match slot {
            ConditionSlot::PersistentDamage {
                damage_type,
                amount,
            } => (*damage_type, *amount),
        })
        .collect()
}

fn dying_at(value: i32) -> Vital {
    Vital::Dying(DyingValue::new(value).expect("dying"))
}

fn fire_profile(dice: u8, bonus: i32) -> StrikeProfile {
    let mut attack = profile(0, Agility::NotAgile, dice, 8, bonus);
    attack.damage_type = DamageType::Fire;
    attack
}

fn strike_hit(
    encounter: &mut Encounter,
    dice: &mut ScriptedDice,
    attack: &StrikeProfile,
    block: ShieldBlock,
) -> Result<i32, EncounterError> {
    encounter
        .strike(id(1), id(2), attack, &[], block, &Detections::new(), dice)
        .map(|delivery| rolled(delivery).damage)
}

fn opened(target: crate::encounter::Participant, rolls: &[u8]) -> (Encounter, ScriptedDice) {
    let (mut encounter, dice) = begin(vec![pc(1, 5, 40, 18), target], rolls);
    assert_eq!(encounter.start_turn().expect("attacker"), id(1));
    (encounter, dice)
}

fn solo(person: crate::encounter::Participant, rolls: &[u8]) -> (Encounter, ScriptedDice) {
    let (mut encounter, dice) = begin(vec![person], rolls);
    assert_eq!(
        encounter.start_turn().expect("start"),
        person_id(&encounter)
    );
    (encounter, dice)
}

fn person_id(encounter: &Encounter) -> crate::combatant::CombatantId {
    encounter.order()[0]
}

/// Player Core pg. 407, Step 3. Player Core pg. 408, Weakness.
#[test]
fn weakness_on_the_maximum_amount_overflows() {
    let target = with_defenses(
        enemy(2, 0, 10, 11),
        defenses(&[], &[(DamageType::Fire, 1)], &[]),
    );
    let (mut encounter, mut dice) = opened(target, &[20, 1, 11, 8]);
    let error = strike_hit(
        &mut encounter,
        &mut dice,
        &fire_profile(0, i32::MAX),
        ShieldBlock::Declined,
    )
    .expect_err("overflow");
    assert_eq!(
        error,
        EncounterError::DamageOverflow(ModifierError::Overflow)
    );
    assert_eq!(hit_points(&encounter, 2), 10);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 436, Step 3: End Your Turn.
#[test]
fn two_maximum_persistent_amounts_overflow_the_end_step() {
    let (mut encounter, mut dice) = solo(pc(1, 0, 10, 11), &[10, 8]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, i32::MAX)
        .expect("fire");
    encounter
        .gain_persistent_damage(id(1), DamageType::Bleed, i32::MAX)
        .expect("bleed");
    let error = encounter.end_turn(&mut dice).expect_err("overflow");
    assert_eq!(
        error,
        EncounterError::DamageOverflow(ModifierError::Overflow)
    );
    assert_eq!(hit_points(&encounter, 1), 10);
    assert_eq!(
        persistent(&encounter, 1),
        vec![(DamageType::Fire, i32::MAX), (DamageType::Bleed, i32::MAX)]
    );
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 408, Weakness.
#[test]
fn weakness_0_is_rejected() {
    let dice = ScriptedDice::new([8]);
    let mut parts = empty_parts();
    parts.defenses = defenses(&[], &[(DamageType::Fire, 0)], &[]);
    assert_eq!(
        CharacterBuild::try_new(parts).expect_err("weakness"),
        crate::combatant::TemplateError::InvalidDefenses
    );
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 408, Resistance.
#[test]
fn a_negative_resistance_is_rejected() {
    let mut parts = empty_parts();
    parts.defenses = defenses(&[], &[], &[(DamageType::Fire, -1)]);
    assert_eq!(
        CharacterBuild::try_new(parts).expect_err("resistance"),
        crate::combatant::TemplateError::InvalidDefenses
    );
}

/// Player Core pg. 410, Hit Points, Healing, and Dying. Player Core pg. 407, Damage Types.
#[test]
fn the_strike_type_and_the_defenses_are_kept() {
    let dice = ScriptedDice::new([8]);
    let mut parts = empty_parts();
    parts.strike.damage_type = DamageType::Slashing;
    parts.defenses = defenses(&[], &[], &[(DamageType::Slashing, 5)]);
    let build = CharacterBuild::try_new(parts).expect("build");
    let first = derive(&build);
    let second = derive(&build);
    assert_eq!(first.strike.damage_type, DamageType::Slashing);
    assert_eq!(first.defenses, second.defenses);
    assert_eq!(first, second);
    assert_eq!(dice.remaining(), 1);
}

fn empty_parts() -> BuildParts {
    BuildParts {
        hit_points: 20,
        armor_class: 18,
        perception: 0,
        speed_feet: 25,
        strike: MeleeStrike {
            attack_bonus: 0,
            damage_dice: DieCount::new(1).expect("dice"),
            damage_faces: faces(8),
            damage_bonus: 0,
            reach_feet: ReachFeet::new(5).expect("reach"),
            agile: false,
            damage_type: DamageType::Slashing,
        },
        shield: None,
        stealth_dc: None,
        aid_modifier: None,
        aid_rank: None,
        uses_dying_rules: None,
        saves: None,
        skills: None,
        defenses: DamageDefenses::empty(),
    }
}

/// Player Core pg. 406, Damage Rolls. Player Core pg. 408, Weakness.
#[test]
fn a_fire_strike_adds_weakness_5() {
    let target = with_defenses(
        enemy(2, 0, 20, 11),
        defenses(&[], &[(DamageType::Fire, 5)], &[]),
    );
    let (mut encounter, mut dice) = opened(target, &[20, 1, 11, 4]);
    let damage = strike_hit(
        &mut encounter,
        &mut dice,
        &fire_profile(1, 0),
        ShieldBlock::Declined,
    )
    .expect("hit");
    assert_eq!(damage, 9);
    assert_eq!(hit_points(&encounter, 2), 11);
}

/// Player Core pg. 406, Damage Rolls.
#[test]
fn empty_defenses_deal_the_rolled_total() {
    let (mut encounter, mut dice) = opened(enemy(2, 0, 20, 11), &[20, 1, 11, 8]);
    let damage = strike_hit(
        &mut encounter,
        &mut dice,
        &profile(0, Agility::NotAgile, 1, 8, 0),
        ShieldBlock::Declined,
    )
    .expect("hit");
    assert_eq!(damage, 8);
    assert_eq!(hit_points(&encounter, 2), 12);
}

/// Player Core pg. 418, Strike.
#[test]
fn a_miss_does_not_apply_weakness() {
    let target = with_defenses(
        enemy(2, 0, 20, 11),
        defenses(&[], &[(DamageType::Fire, 5)], &[]),
    );
    let (mut encounter, mut dice) = opened(target, &[20, 1, 2, 4]);
    let damage = strike_hit(
        &mut encounter,
        &mut dice,
        &fire_profile(1, 0),
        ShieldBlock::Declined,
    )
    .expect("miss");
    assert_eq!(damage, 0);
    assert_eq!(hit_points(&encounter, 2), 20);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 411, Taking Damage.
#[test]
fn dying_1_and_immune_fire_8_stays_dying_1() {
    let target = with_defenses(
        combatant(2, Side::Enemy, 0, 20, 11, true),
        defenses(&[DamageType::Fire], &[], &[]),
    );
    let (mut encounter, mut dice) = opened(target, &[20, 1, 11, 8]);
    lay(&mut encounter, 2, dying_at(1), 0);
    strike_hit(
        &mut encounter,
        &mut dice,
        &fire_profile(1, 0),
        ShieldBlock::Declined,
    )
    .expect("hit");
    assert_eq!(hit_points(&encounter, 2), 0);
    assert_eq!(vital(&encounter, 2), dying_at(1));
}

/// Player Core pg. 411, Taking Damage.
#[test]
fn dying_1_and_resisted_fire_8_stays_dying_1() {
    let target = with_defenses(
        combatant(2, Side::Enemy, 0, 20, 11, true),
        defenses(&[], &[], &[(DamageType::Fire, 10)]),
    );
    let (mut encounter, mut dice) = opened(target, &[20, 1, 11, 8]);
    lay(&mut encounter, 2, dying_at(1), 0);
    strike_hit(
        &mut encounter,
        &mut dice,
        &fire_profile(1, 0),
        ShieldBlock::Declined,
    )
    .expect("hit");
    assert_eq!(hit_points(&encounter, 2), 0);
    assert_eq!(vital(&encounter, 2), dying_at(1));
}

/// Player Core pg. 404, Basic Saving Throws.
#[test]
fn fire_resistance_does_not_reduce_a_basic_save() {
    let mut person = creature(20, 11, 0, true, None);
    person.defenses = defenses(&[], &[], &[(DamageType::Fire, 10)]);
    person.saves = Some(SavingThrows {
        level: Level::new(1).expect("level"),
        constitution: 0,
        dexterity: 0,
        wisdom: 0,
        fortitude: Proficiency::Untrained,
        reflex: Proficiency::Untrained,
        will: Proficiency::Untrained,
    });
    let (mut encounter, mut dice) = solo(super::participant(1, Side::Pc, person), &[10, 6, 8]);
    let roll = encounter
        .basic_save(id(1), Save::Fortitude, 15, 5, &[], &mut dice)
        .expect("save");
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
    assert_eq!(roll.damage, 5);
    assert_eq!(hit_points(&encounter, 1), 15);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 445, Persistent Damage.
#[test]
fn a_fresh_creature_has_no_persistent_damage() {
    let dice = ScriptedDice::new([8]);
    let built = CharacterBuild::try_new(empty_parts()).expect("build");
    let state = crate::combatant::new_combatant_state(id(1), &derive(&built));
    assert!(state.conditions.is_empty());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 445, Persistent Damage.
#[test]
fn fire_5_is_stored() {
    let (mut encounter, dice) = solo(pc(1, 0, 20, 11), &[10]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("gain");
    assert_eq!(persistent(&encounter, 1), vec![(DamageType::Fire, 5)]);
    assert_eq!(actions(&encounter, 1), 3);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 0);
}

/// Player Core pg. 445, Persistent Damage.
#[test]
fn a_critical_persistent_fire_2_is_stored_as_4() {
    let (mut encounter, _) = solo(pc(1, 0, 20, 11), &[10]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 4)
        .expect("doubled");
    assert_eq!(persistent(&encounter, 1), vec![(DamageType::Fire, 4)]);
    assert_ne!(persistent(&encounter, 1), vec![(DamageType::Fire, 6)]);
}

/// Player Core pg. 445, Persistent Damage.
#[test]
fn a_non_positive_amount_is_rejected() {
    let (mut encounter, dice) = solo(pc(1, 0, 20, 11), &[10, 8]);
    let error = encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 0)
        .expect_err("zero");
    assert_eq!(error, EncounterError::NonPositivePersistent);
    assert!(persistent(&encounter, 1).is_empty());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Encounter Mode.
#[test]
fn an_unknown_combatant_is_rejected() {
    let (mut encounter, _) = solo(pc(1, 0, 20, 11), &[10]);
    let error = encounter
        .gain_persistent_damage(id(9), DamageType::Fire, 5)
        .expect_err("unknown");
    assert_eq!(error, EncounterError::UnknownCombatant);
}

/// Player Core, Strike.
#[test]
fn a_dead_combatant_cannot_gain_it() {
    let (mut encounter, _) = solo(pc(1, 0, 20, 11), &[10]);
    lay(&mut encounter, 1, Vital::Dead, 0);
    let error = encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect_err("dead");
    assert_eq!(error, EncounterError::TargetDead);
    assert!(persistent(&encounter, 1).is_empty());
}

/// Player Core pg. 445, Multiple Persistent Damage Conditions.
#[test]
fn fire_8_replaces_fire_5() {
    let (mut encounter, _) = solo(pc(1, 0, 20, 11), &[10]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("five");
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 8)
        .expect("eight");
    assert_eq!(persistent(&encounter, 1), vec![(DamageType::Fire, 8)]);
}

/// Player Core pg. 445, Multiple Persistent Damage Conditions.
#[test]
fn fire_5_does_not_replace_fire_8() {
    let (mut encounter, _) = solo(pc(1, 0, 20, 11), &[10]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 8)
        .expect("eight");
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("five");
    assert_eq!(persistent(&encounter, 1), vec![(DamageType::Fire, 8)]);
}

/// Player Core pg. 445, Multiple Persistent Damage Conditions.
#[test]
fn an_equal_fire_amount_stays() {
    let (mut encounter, _) = solo(pc(1, 0, 20, 11), &[10]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("first");
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("second");
    assert_eq!(persistent(&encounter, 1), vec![(DamageType::Fire, 5)]);
}

/// Player Core pg. 445, Multiple Persistent Damage Conditions.
#[test]
fn fire_and_bleed_are_both_stored() {
    let (mut encounter, _) = solo(pc(1, 0, 20, 11), &[10]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("fire");
    encounter
        .gain_persistent_damage(id(1), DamageType::Bleed, 3)
        .expect("bleed");
    assert_eq!(
        persistent(&encounter, 1),
        vec![(DamageType::Fire, 5), (DamageType::Bleed, 3)]
    );
}

/// Player Core pg. 445, Persistent Damage.
#[test]
fn fire_5_deals_5_then_waits_on_the_flat_check() {
    let (mut encounter, mut dice) = solo(pc(1, 0, 20, 11), &[10, 14]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("gain");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(hit_points(&encounter, 1), 15);
    assert_eq!(persistent(&encounter, 1), vec![(DamageType::Fire, 5)]);
    assert_eq!(actions(&encounter, 1), 0);
}

/// Player Core pg. 436, Step 3: End Your Turn.
#[test]
fn fire_and_bleed_deal_8_as_one_blow() {
    let (mut encounter, mut dice) = solo(pc(1, 0, 20, 11), &[10, 10, 10]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("fire");
    encounter
        .gain_persistent_damage(id(1), DamageType::Bleed, 3)
        .expect("bleed");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(hit_points(&encounter, 1), 12);
    assert_eq!(persistent(&encounter, 1).len(), 2);
}

/// Player Core pg. 411, Taking Damage while Dying.
#[test]
fn two_types_while_dying_1_increase_dying_once() {
    let (mut encounter, mut dice) = solo(combatant(1, Side::Pc, 0, 20, 11, true), &[10, 10, 10]);
    lay(&mut encounter, 1, dying_at(1), 0);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("fire");
    encounter
        .gain_persistent_damage(id(1), DamageType::Bleed, 5)
        .expect("bleed");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(vital(&encounter, 1), dying_at(2));
    assert_ne!(vital(&encounter, 1), dying_at(3));
    assert_ne!(vital(&encounter, 1), Vital::Dead);
}

/// Player Core pg. 410, Getting Knocked Out.
#[test]
fn persistent_damage_to_0_sets_dying_1() {
    let other = pc(2, 0, 20, 11);
    let (mut encounter, mut dice) = begin(
        vec![combatant(1, Side::Pc, 5, 20, 11, true), other],
        &[20, 1, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("gain");
    lay(&mut encounter, 1, Vital::Alive, 5);
    let before = encounter.order().to_vec();
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(hit_points(&encounter, 1), 0);
    assert_eq!(vital(&encounter, 1), dying_at(1));
    assert_eq!(encounter.order(), before.as_slice());
}

/// Player Core pg. 411, Death.
#[test]
fn without_dying_rules_0_hit_points_is_death() {
    let (mut encounter, mut dice) = solo(pc(1, 0, 20, 11), &[10, 10]);
    lay(&mut encounter, 1, Vital::Alive, 5);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("gain");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(vital(&encounter, 1), Vital::Dead);
    assert!(persistent(&encounter, 1).is_empty());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 410, Temporary Hit Points.
#[test]
fn temporary_hit_points_are_spent_once() {
    let mut person = pc(1, 0, 10, 11);
    person.state.temp_hit_points = 3;
    person.state.temp_rounds = 1;
    let (mut encounter, mut dice) = solo(person, &[10, 10, 10]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 2)
        .expect("fire");
    encounter
        .gain_persistent_damage(id(1), DamageType::Bleed, 2)
        .expect("bleed");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(temp_hp(&encounter, 1), 0);
    assert_eq!(hit_points(&encounter, 1), 9);
}

/// Player Core pg. 262, Shield Block.
#[test]
fn persistent_slashing_leaves_the_shield_and_the_reaction() {
    let person = super::participant(1, Side::Pc, creature(30, 11, 0, false, Some(plate())));
    let (mut encounter, mut dice) = begin(vec![person], &[10, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.raise_shield(id(1)).expect("raise");
    encounter
        .gain_persistent_damage(id(1), DamageType::Slashing, 5)
        .expect("gain");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(hit_points(&encounter, 1), 25);
    assert_eq!(
        encounter.state(id(1)).expect("state").shield_hit_points,
        Some(20)
    );
    assert!(reaction(&encounter, 1));
}

/// Player Core pg. 436, Step 3: End Your Turn.
#[test]
fn the_other_creature_s_fire_does_not_tick() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 5, 20, 11), pc(2, 0, 20, 11)], &[20, 1, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter
        .gain_persistent_damage(id(2), DamageType::Fire, 5)
        .expect("gain");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(hit_points(&encounter, 2), 20);
    assert_eq!(persistent(&encounter, 2), vec![(DamageType::Fire, 5)]);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 408, Resistance.
#[test]
fn resistance_10_leaves_hit_points_unchanged() {
    let person = with_defenses(
        pc(1, 0, 20, 11),
        defenses(&[], &[], &[(DamageType::Fire, 10)]),
    );
    let (mut encounter, mut dice) = solo(person, &[10, 14]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("gain");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(hit_points(&encounter, 1), 20);
    assert_eq!(persistent(&encounter, 1), vec![(DamageType::Fire, 5)]);
}

/// Player Core pg. 445, Persistent Damage. Player Core pg. 408, Immunity.
#[test]
fn immune_fire_5_still_rolls_the_flat_check() {
    let person = with_defenses(pc(1, 0, 20, 11), defenses(&[DamageType::Fire], &[], &[]));
    let (mut encounter, mut dice) = solo(person, &[10, 15]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("gain");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(hit_points(&encounter, 1), 20);
    assert!(persistent(&encounter, 1).is_empty());
}

/// Player Core pg. 445, Persistent Damage.
#[test]
fn a_natural_15_ends_fire_after_the_damage() {
    let (mut encounter, mut dice) = solo(pc(1, 0, 20, 11), &[10, 15]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("gain");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(hit_points(&encounter, 1), 15);
    assert!(persistent(&encounter, 1).is_empty());
}

/// Player Core pg. 445, Persistent Damage.
#[test]
fn a_natural_14_keeps_fire() {
    let (mut encounter, mut dice) = solo(pc(1, 0, 20, 11), &[10, 14]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("gain");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(hit_points(&encounter, 1), 15);
    assert_eq!(persistent(&encounter, 1), vec![(DamageType::Fire, 5)]);
}

/// Player Core pg. 400, Flat Checks.
#[test]
fn a_natural_20_ends_fire() {
    let (mut encounter, mut dice) = solo(pc(1, 0, 20, 11), &[10, 20]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("gain");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(hit_points(&encounter, 1), 15);
    assert!(persistent(&encounter, 1).is_empty());
}

/// Player Core pg. 400, Flat Checks.
#[test]
fn a_natural_1_keeps_fire() {
    let (mut encounter, mut dice) = solo(pc(1, 0, 20, 11), &[10, 1]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("gain");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(hit_points(&encounter, 1), 15);
    assert_eq!(persistent(&encounter, 1), vec![(DamageType::Fire, 5)]);
}

/// A failed persistent entry stays when another type's flat check succeeds.
///
/// `ConditionSlot` has only `PersistentDamage`, so this is the stand-in for a
/// non-persistent slot surviving the end step: retain removes the succeeded type
/// and leaves the one that failed.
///
/// Player Core pg. 445, Multiple Persistent Damage Conditions.
#[test]
fn a_failed_persistent_entry_survives_a_successful_check() {
    let (mut encounter, mut dice) = solo(pc(1, 0, 20, 11), &[10, 14, 15]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("fire");
    encounter
        .gain_persistent_damage(id(1), DamageType::Bleed, 3)
        .expect("bleed");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(hit_points(&encounter, 1), 12);
    assert_eq!(persistent(&encounter, 1), vec![(DamageType::Bleed, 3)]);
}

/// Player Core pg. 445, Multiple Persistent Damage Conditions.
#[test]
fn bleed_is_checked_before_fire() {
    let (mut encounter, mut dice) = solo(pc(1, 0, 20, 11), &[10, 15, 10]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("fire");
    encounter
        .gain_persistent_damage(id(1), DamageType::Bleed, 3)
        .expect("bleed");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(hit_points(&encounter, 1), 12);
    assert_eq!(persistent(&encounter, 1), vec![(DamageType::Fire, 5)]);
}

/// Player Core pg. 411, Death.
#[test]
fn massive_persistent_damage_draws_no_flat_check() {
    let (mut encounter, mut dice) = solo(pc(1, 0, 9, 11), &[10, 15]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 20)
        .expect("gain");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(vital(&encounter, 1), Vital::Dead);
    assert!(persistent(&encounter, 1).is_empty());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 411, Death.
#[test]
fn dying_3_and_persistent_bleed_2_draws_no_flat_check() {
    let (mut encounter, mut dice) = solo(combatant(1, Side::Pc, 0, 20, 11, true), &[10, 15]);
    lay(&mut encounter, 1, dying_at(3), 0);
    encounter
        .gain_persistent_damage(id(1), DamageType::Bleed, 2)
        .expect("gain");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(vital(&encounter, 1), Vital::Dead);
    assert!(persistent(&encounter, 1).is_empty());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Step 2: Act". Player Core pg. 445, Persistent Damage.
#[test]
fn a_round_overflow_leaves_persistent_damage_unrolled() {
    let (mut encounter, mut dice) = solo(pc(1, 0, 20, 11), &[10, 14]);
    encounter.set_round_for_test(u32::MAX);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("gain");
    assert_eq!(
        encounter.end_turn(&mut dice).expect_err("overflow"),
        EncounterError::RoundOverflow
    );
    assert_eq!(hit_points(&encounter, 1), 20);
    assert_eq!(persistent(&encounter, 1), vec![(DamageType::Fire, 5)]);
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 411, Death. Player Core pg. 418, Strike.
#[test]
fn a_strike_kill_clears_persistent_damage() {
    let (mut encounter, mut dice) = opened(enemy(2, 0, 8, 11), &[20, 1, 11, 8]);
    encounter
        .gain_persistent_damage(id(2), DamageType::Fire, 3)
        .expect("gain");
    strike_hit(
        &mut encounter,
        &mut dice,
        &profile(0, Agility::NotAgile, 1, 8, 0),
        ShieldBlock::Declined,
    )
    .expect("hit");
    assert_eq!(vital(&encounter, 2), Vital::Dead);
    assert!(persistent(&encounter, 2).is_empty());
}

/// Player Core pg. 411, Death. Player Core pg. 138, Reactive Strike.
#[test]
fn a_reactive_kill_clears_persistent_damage() {
    let target = combatant(2, Side::Pc, 0, 8, 18, false);
    let (mut encounter, mut dice) = begin(
        vec![super::with_reach(pc(1, 0, 20, 18), 5), target],
        &[10, 8, 10, 8],
    );
    encounter
        .gain_persistent_damage(id(2), DamageType::Fire, 3)
        .expect("gain");
    let mut attack = profile(8, Agility::NotAgile, 1, 8, 0);
    attack.damage_type = DamageType::Slashing;
    super::react(
        &mut encounter,
        &mut dice,
        1,
        2,
        ActionCost::Actions {
            count: 1,
            traits: &[ActionTrait::Manipulate],
        },
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        crate::grid::Size::Medium,
        crate::grid::Size::Medium,
        attack,
        &[],
    )
    .expect("reactive");
    assert_eq!(vital(&encounter, 2), Vital::Dead);
    assert!(persistent(&encounter, 2).is_empty());
}

/// Player Core pg. 411, Death.
#[test]
fn a_dead_creature_s_end_step_draws_nothing() {
    let (mut encounter, mut dice) = solo(pc(1, 0, 20, 11), &[10, 15]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("gain");
    lay(&mut encounter, 1, Vital::Dead, 0);
    encounter.end_turn(&mut dice).expect("end");
    assert!(persistent(&encounter, 1).is_empty());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 407, Bleed Damage.
#[test]
fn a_heal_to_maximum_removes_bleed_and_keeps_fire() {
    let (mut encounter, _) = solo(pc(1, 0, 20, 11), &[10]);
    lay(&mut encounter, 1, Vital::Alive, 10);
    encounter
        .gain_persistent_damage(id(1), DamageType::Bleed, 3)
        .expect("bleed");
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("fire");
    heal(&mut encounter, 1, 10);
    assert_eq!(hit_points(&encounter, 1), 20);
    assert_eq!(persistent(&encounter, 1), vec![(DamageType::Fire, 5)]);
}

/// Player Core pg. 407, Bleed Damage.
#[test]
fn a_short_heal_leaves_bleed() {
    let (mut encounter, _) = solo(pc(1, 0, 20, 11), &[10]);
    lay(&mut encounter, 1, Vital::Alive, 10);
    encounter
        .gain_persistent_damage(id(1), DamageType::Bleed, 3)
        .expect("bleed");
    heal(&mut encounter, 1, 4);
    assert_eq!(hit_points(&encounter, 1), 14);
    assert_eq!(persistent(&encounter, 1), vec![(DamageType::Bleed, 3)]);
}

fn heal(encounter: &mut Encounter, value: u32, amount: i32) {
    let member = encounter
        .members
        .iter_mut()
        .find(|member| member.participant.id == id(value))
        .expect("member");
    let creature = member.participant.creature.clone();
    apply_healing(&mut member.participant.state, &creature, amount).expect("heal");
}

/// Player Core pg. 445, Assisted Recovery.
#[test]
fn the_end_step_rolls_one_d20_and_spends_no_action() {
    let (mut encounter, mut dice) = solo(pc(1, 0, 20, 11), &[10, 14, 6]);
    encounter.spend_actions(id(1), 3).expect("spend");
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("gain");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(hit_points(&encounter, 1), 15);
    assert_eq!(persistent(&encounter, 1), vec![(DamageType::Fire, 5)]);
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 262, Shield Block.
#[test]
fn a_fire_strike_does_not_block() {
    let target = super::participant(2, Side::Enemy, creature(20, 11, 0, false, Some(plate())));
    let (mut encounter, mut dice) = opened(target, &[20, 1, 11, 8]);
    strike_hit(
        &mut encounter,
        &mut dice,
        &fire_profile(1, 0),
        ShieldBlock::Requested,
    )
    .expect("hit");
    assert_eq!(hit_points(&encounter, 2), 12);
    assert_eq!(
        encounter.state(id(2)).expect("state").shield_hit_points,
        Some(20)
    );
    assert!(reaction(&encounter, 2));
}

/// Player Core pg. 262, Shield Block. Player Core pg. 408, Immunity.
#[test]
fn immunity_to_slashing_does_not_block() {
    let mut target = super::participant(2, Side::Pc, creature(20, 11, 5, false, Some(plate())));
    target.creature.defenses = defenses(&[DamageType::Slashing], &[], &[]);
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), target], &[1, 20, 11, 8]);
    assert_eq!(encounter.start_turn().expect("defender"), id(2));
    encounter.raise_shield(id(2)).expect("raise");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("attacker"), id(1));
    strike_hit(
        &mut encounter,
        &mut dice,
        &profile(0, Agility::NotAgile, 1, 8, 0),
        ShieldBlock::Requested,
    )
    .expect("hit");
    assert_eq!(hit_points(&encounter, 2), 20);
    assert_eq!(
        encounter.state(id(2)).expect("state").shield_hit_points,
        Some(20)
    );
    assert!(reaction(&encounter, 2));
}

/// Player Core pg. 262, Shield Block. Player Core pg. 408, Resistance.
#[test]
fn resistance_to_0_does_not_spend_the_reaction() {
    let mut target = super::participant(2, Side::Pc, creature(20, 11, 5, false, Some(plate())));
    target.creature.defenses = defenses(&[], &[], &[(DamageType::Slashing, 10)]);
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), target], &[1, 20, 11, 8]);
    assert_eq!(encounter.start_turn().expect("defender"), id(2));
    encounter.raise_shield(id(2)).expect("raise");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("attacker"), id(1));
    strike_hit(
        &mut encounter,
        &mut dice,
        &profile(0, Agility::NotAgile, 1, 8, 0),
        ShieldBlock::Requested,
    )
    .expect("hit");
    assert_eq!(hit_points(&encounter, 2), 20);
    assert_eq!(
        encounter.state(id(2)).expect("state").shield_hit_points,
        Some(20)
    );
    assert!(reaction(&encounter, 2));
}

/// Player Core pg. 262, Shield Block. Player Core pg. 408, Resistance.
#[test]
fn resistance_is_applied_before_hardness() {
    let mut target = super::participant(2, Side::Pc, creature(30, 11, 5, false, Some(plate())));
    target.creature.defenses = defenses(&[], &[], &[(DamageType::Slashing, 5)]);
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 30, 18), target], &[1, 20, 11]);
    assert_eq!(encounter.start_turn().expect("defender"), id(2));
    encounter.raise_shield(id(2)).expect("raise");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("attacker"), id(1));
    strike_hit(
        &mut encounter,
        &mut dice,
        &profile(0, Agility::NotAgile, 0, 8, 12),
        ShieldBlock::Requested,
    )
    .expect("hit");
    assert_eq!(hit_points(&encounter, 2), 28);
    assert_eq!(
        encounter.state(id(2)).expect("state").shield_hit_points,
        Some(18)
    );
}

/// Player Core pg. 138, Reactive Strike. Player Core pg. 408, Resistance.
#[test]
fn a_reactive_fire_hit_subtracts_resistance_5() {
    let mut target = combatant(2, Side::Pc, 0, 20, 18, false);
    target.creature.defenses = defenses(&[], &[], &[(DamageType::Fire, 5)]);
    let (mut encounter, mut dice) = begin(
        vec![super::with_reach(pc(1, 0, 20, 18), 5), target],
        &[10, 8, 10, 8],
    );
    let mut attack = profile(8, Agility::NotAgile, 1, 8, 0);
    attack.damage_type = DamageType::Fire;
    let outcome = super::react(
        &mut encounter,
        &mut dice,
        1,
        2,
        ActionCost::Actions {
            count: 1,
            traits: &[ActionTrait::Manipulate],
        },
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        crate::grid::Size::Medium,
        crate::grid::Size::Medium,
        attack,
        &[],
    )
    .expect("reactive");
    assert_eq!(rolled_reactive(&outcome).damage, 3);
    assert_eq!(hit_points(&encounter, 2), 17);
}

fn rolled_reactive(outcome: &crate::reactive::ReactiveOutcome) -> &crate::encounter::StrikeOutcome {
    match &outcome.strike {
        crate::encounter::StrikeDelivery::Rolled(strike) => strike,
        other => panic!("expected a rolled strike, got {other:?}"),
    }
}
