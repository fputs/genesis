use super::*;
use crate::check::degree_of_success;
use crate::dice::Dice;
use crate::save::{Level, Proficiency, Save, SavingThrows};

fn level(value: i32) -> Level {
    Level::new(value).expect("level")
}

fn training(
    level_value: i32,
    constitution: i32,
    dexterity: i32,
    wisdom: i32,
    fortitude: Proficiency,
    reflex: Proficiency,
    will: Proficiency,
) -> SavingThrows {
    SavingThrows {
        level: level(level_value),
        constitution,
        dexterity,
        wisdom,
        fortitude,
        reflex,
        will,
    }
}

fn ranks(
    level_value: i32,
    constitution: i32,
    dexterity: i32,
    wisdom: i32,
    fortitude: Proficiency,
    reflex: Proficiency,
    will: Proficiency,
) -> Option<SavingThrows> {
    Some(training(
        level_value,
        constitution,
        dexterity,
        wisdom,
        fortitude,
        reflex,
        will,
    ))
}

fn fort(level_value: i32, constitution: i32, rank: Proficiency) -> Option<SavingThrows> {
    ranks(
        level_value,
        constitution,
        0,
        0,
        rank,
        Proficiency::Untrained,
        Proficiency::Untrained,
    )
}

fn melee() -> MeleeStrike {
    MeleeStrike {
        attack_bonus: 0,
        damage_dice: DieCount::new(1).expect("dice"),
        damage_faces: faces(8),
        damage_bonus: 0,
        reach_feet: ReachFeet::new(5).expect("reach"),
        agile: false,
        damage_type: crate::damage::DamageType::Slashing,
    }
}

fn steel() -> ShieldTemplate {
    ShieldTemplate {
        hardness: 5,
        hit_points: 20,
        broken_threshold: 10,
        shield_ac_bonus: 2,
    }
}

fn dying_at(value: i32) -> Vital {
    Vital::Dying(DyingValue::new(value).expect("dying"))
}

struct Body {
    id: u32,
    current: i32,
    max_hp: i32,
    dying_rules: bool,
    saves: Option<SavingThrows>,
    vital: Vital,
    temp: i32,
    shield: Option<ShieldTemplate>,
    raised: bool,
    ac: i32,
}

fn body(spec: Body) -> Participant {
    let built = CharacterBuild::try_new(BuildParts {
        hit_points: spec.max_hp,
        armor_class: spec.ac,
        perception: 0,
        speed_feet: 25,
        strike: melee(),
        shield: spec.shield,
        stealth_dc: None,
        aid_modifier: None,
        aid_rank: None,
        uses_dying_rules: Some(spec.dying_rules),
        saves: spec.saves,
        skills: None,
        defenses: crate::damage::DamageDefenses::empty(),
    })
    .expect("build");
    let mut person = participant(spec.id, Side::Pc, derive(&built));
    person.state.hit_points = spec.current;
    person.state.vital = spec.vital;
    person.state.temp_hit_points = spec.temp;
    if spec.temp > 0 {
        person.state.temp_rounds = 1;
    }
    person.state.shield_raised = spec.raised;
    person
}

fn alive(who: u32, current: i32, saves: Option<SavingThrows>) -> Participant {
    body(Body {
        id: who,
        current,
        max_hp: 30,
        dying_rules: true,
        saves,
        vital: Vital::Alive,
        temp: 0,
        shield: None,
        raised: false,
        ac: 11,
    })
}

fn plain(who: u32, current: i32) -> Participant {
    alive(who, current, fort(1, 0, Proficiency::Untrained))
}

fn modifier_of(roll: &crate::save::SaveRoll) -> i32 {
    roll.total - i32::from(roll.natural)
}

fn opened(people: Vec<Participant>, initiative: &[u8]) -> Encounter {
    begin(people, initiative).0
}

fn attempt(
    encounter: &mut Encounter,
    who: u32,
    save: Save,
    dc: i32,
    amount: i32,
    modifiers: &[Modifier],
    faces: &[u8],
) -> (Result<crate::save::SaveRoll, EncounterError>, ScriptedDice) {
    let mut dice = ScriptedDice::new(faces.iter().copied());
    let result = encounter.basic_save(id(who), save, dc, amount, modifiers, &mut dice);
    (result, dice)
}

fn resolved(
    who: Participant,
    save: Save,
    dc: i32,
    amount: i32,
    modifiers: &[Modifier],
    face: u8,
) -> (crate::save::SaveRoll, Encounter) {
    let mut encounter = opened(vec![who], &[10]);
    let (result, dice) = attempt(&mut encounter, 1, save, dc, amount, modifiers, &[face]);
    assert_eq!(dice.remaining(), 0);
    (result.expect("save"), encounter)
}

fn on_fortitude(
    constitution: i32,
    rank: Proficiency,
    modifiers: &[Modifier],
    face: u8,
    dc: i32,
) -> crate::save::SaveRoll {
    let (roll, _) = resolved(
        alive(1, 30, fort(1, constitution, rank)),
        Save::Fortitude,
        dc,
        0,
        modifiers,
        face,
    );
    roll
}

fn circumstance_bonus(value: i32, source: u32, statistic: Statistic) -> Modifier {
    bonus(BonusType::Circumstance, value, source, statistic)
}

fn circumstance_penalty(value: i32, source: u32, statistic: Statistic) -> Modifier {
    penalty(PenaltyType::Circumstance, value, source, statistic)
}

/// Player Core pg. 401, Step 4: Degree of Success.
#[test]
fn total_dc_minus_1_is_a_failure() {
    let roll = on_fortitude(0, Proficiency::Untrained, &[], 14, 15);
    assert_eq!(roll.total, 14);
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 401, Natural 1 and Natural 20.
#[test]
fn natural_1_at_a_total_equal_to_the_dc_is_a_failure() {
    let (roll, _) = resolved(
        alive(1, 30, fort(1, 14, Proficiency::Untrained)),
        Save::Fortitude,
        15,
        0,
        &[],
        1,
    );
    assert_eq!(roll.total, 15);
    assert_eq!(roll.natural, 1);
    assert_eq!(
        degree_of_success(10, roll.total, 15),
        DegreeOfSuccess::Success
    );
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 401, Natural 1 and Natural 20.
#[test]
fn natural_20_at_dc_minus_11_is_a_failure() {
    let roll = on_fortitude(0, Proficiency::Untrained, &[], 20, 31);
    assert_eq!(roll.total, 20);
    assert_eq!(roll.natural, 20);
    assert_eq!(
        degree_of_success(10, roll.total, 31),
        DegreeOfSuccess::CriticalFailure
    );
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 404, Basic Saving Throws.
#[test]
fn a_negative_amount_precedes_no_saving_throws() {
    let mut encounter = opened(vec![alive(1, 20, None)], &[10]);
    let (error, mut dice) = attempt(&mut encounter, 1, Save::Fortitude, 15, -1, &[], &[10]);
    assert_eq!(error, Err(EncounterError::NegativeAmount));
    assert_ne!(error, Err(EncounterError::NoSavingThrows));
    assert_eq!(hit_points(&encounter, 1), 20);
    assert_eq!(vital(&encounter, 1), Vital::Alive);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(dice.d20(), 10);
}

/// Player Core pg. 404, Basic Saving Throws.
#[test]
fn a_negative_amount_rolls_nothing() {
    let mut encounter = opened(vec![plain(1, 20)], &[10]);
    let (error, mut dice) = attempt(&mut encounter, 1, Save::Fortitude, 15, -1, &[], &[10]);
    assert_eq!(error, Err(EncounterError::NegativeAmount));
    assert_eq!(hit_points(&encounter, 1), 20);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(dice.d20(), 10);
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn fortitude_is_constitution_plus_trained_at_level_1() {
    let roll = on_fortitude(2, Proficiency::Trained, &[], 10, 15);
    assert_eq!(modifier_of(&roll), 5);
    assert_eq!(roll.total, 15);
    assert_eq!(roll.degree, DegreeOfSuccess::Success);
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn reflex_is_dexterity_plus_expert_at_level_5() {
    let (roll, _) = resolved(
        alive(
            1,
            30,
            ranks(
                5,
                0,
                -1,
                0,
                Proficiency::Untrained,
                Proficiency::Expert,
                Proficiency::Untrained,
            ),
        ),
        Save::Reflex,
        18,
        0,
        &[],
        10,
    );
    assert_eq!(modifier_of(&roll), 8);
    assert_eq!(roll.total, 18);
    assert_eq!(roll.degree, DegreeOfSuccess::Success);
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn will_is_wisdom_plus_legendary_at_level_20() {
    let (roll, _) = resolved(
        alive(
            1,
            30,
            ranks(
                20,
                0,
                0,
                4,
                Proficiency::Untrained,
                Proficiency::Untrained,
                Proficiency::Legendary,
            ),
        ),
        Save::Will,
        40,
        0,
        &[],
        10,
    );
    assert_eq!(modifier_of(&roll), 32);
    assert_eq!(roll.total, 42);
    assert_eq!(roll.degree, DegreeOfSuccess::Success);
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn untrained_is_0_at_level_20() {
    let (roll, _) = resolved(
        alive(1, 30, fort(20, 3, Proficiency::Untrained)),
        Save::Fortitude,
        13,
        0,
        &[],
        10,
    );
    assert_eq!(modifier_of(&roll), 3);
    assert_eq!(roll.total, 13);
    assert_eq!(roll.degree, DegreeOfSuccess::Success);
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn trained_at_level_2_with_wisdom_0_is_plus_4() {
    let (roll, _) = resolved(
        alive(
            1,
            30,
            ranks(
                2,
                0,
                0,
                0,
                Proficiency::Untrained,
                Proficiency::Untrained,
                Proficiency::Trained,
            ),
        ),
        Save::Will,
        14,
        0,
        &[],
        10,
    );
    assert_eq!(modifier_of(&roll), 4);
    assert_eq!(roll.total, 14);
}

/// Player Core pg. 404, Saving Throws. Player Core pg. 400, Bonuses.
#[test]
fn an_item_bonus_adds_beside_proficiency() {
    let item = bonus(BonusType::Item, 1, 1, Statistic::Fortitude);
    let roll = on_fortitude(2, Proficiency::Trained, &[item], 10, 16);
    assert_eq!(modifier_of(&roll), 6);
    assert_eq!(roll.total, 16);
}

/// Player Core pg. 404, Saving Throws. Player Core pg. 400, Bonuses.
#[test]
fn two_circumstance_bonuses_keep_the_highest() {
    let modifiers = [
        circumstance_bonus(2, 1, Statistic::Fortitude),
        circumstance_bonus(1, 2, Statistic::Fortitude),
    ];
    let roll = on_fortitude(2, Proficiency::Trained, &modifiers, 10, 17);
    assert_eq!(modifier_of(&roll), 7);
    assert_ne!(modifier_of(&roll), 8);
    assert_eq!(roll.total, 17);
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn an_attack_penalty_does_not_change_fortitude() {
    let attack = circumstance_penalty(-2, 1, Statistic::Attack);
    let roll = on_fortitude(2, Proficiency::Trained, &[attack], 10, 15);
    assert_eq!(modifier_of(&roll), 5);
    assert_eq!(roll.total, 15);
    assert_ne!(roll.total, 13);
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn fortitude_and_will_do_not_share_a_bonus() {
    let modifiers = [
        circumstance_bonus(2, 1, Statistic::Fortitude),
        circumstance_bonus(1, 2, Statistic::Will),
    ];
    let mut encounter = opened(
        vec![alive(
            1,
            30,
            ranks(
                1,
                2,
                0,
                0,
                Proficiency::Trained,
                Proficiency::Untrained,
                Proficiency::Trained,
            ),
        )],
        &[10],
    );
    let (fortitude, _) = attempt(&mut encounter, 1, Save::Fortitude, 14, 0, &modifiers, &[10]);
    let (will, _) = attempt(&mut encounter, 1, Save::Will, 14, 0, &modifiers, &[10]);
    let fortitude = fortitude.expect("fortitude");
    let will = will.expect("will");
    assert_eq!(modifier_of(&fortitude), 7);
    assert_eq!(modifier_of(&will), 4);
}

/// Player Core pg. 404, Saving Throws. Player Core pg. 446, Unconscious.
#[test]
fn a_negative_total_against_dc_5_is_a_failure() {
    let mut person = alive(
        1,
        0,
        ranks(
            1,
            0,
            -1,
            0,
            Proficiency::Untrained,
            Proficiency::Untrained,
            Proficiency::Untrained,
        ),
    );
    person.state.vital = Vital::Unconscious;
    let (roll, _) = resolved(person, Save::Reflex, 5, 0, &[], 4);
    assert_eq!(modifier_of(&roll), -5);
    assert_eq!(roll.total, -1);
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 404, Saving Throws. Player Core pg. 400, Bonuses.
#[test]
fn a_status_bonus_adds() {
    let status = bonus(BonusType::Status, 2, 1, Statistic::Fortitude);
    let roll = on_fortitude(2, Proficiency::Trained, &[status], 10, 17);
    assert_eq!(modifier_of(&roll), 7);
    assert_eq!(roll.total, 17);
}

/// Player Core pg. 404, Saving Throws. Player Core pg. 400, Penalties.
#[test]
fn a_status_penalty_and_a_circumstance_penalty_both_apply() {
    let modifiers = [
        penalty(PenaltyType::Status, -1, 1, Statistic::Fortitude),
        circumstance_penalty(-2, 2, Statistic::Fortitude),
    ];
    let roll = on_fortitude(2, Proficiency::Trained, &modifiers, 10, 12);
    assert_eq!(modifier_of(&roll), 2);
    assert_ne!(modifier_of(&roll), 4);
    assert_ne!(modifier_of(&roll), 5);
    assert_eq!(roll.total, 12);
}

/// Player Core pg. 404, Saving Throws. Player Core pg. 400, Penalties.
#[test]
fn two_circumstance_penalties_keep_the_worst() {
    let modifiers = [
        circumstance_penalty(-2, 1, Statistic::Fortitude),
        circumstance_penalty(-1, 2, Statistic::Fortitude),
    ];
    let roll = on_fortitude(2, Proficiency::Trained, &modifiers, 10, 13);
    assert_eq!(modifier_of(&roll), 3);
    assert_ne!(modifier_of(&roll), 2);
    assert_eq!(roll.total, 13);
}

/// Player Core pg. 404, Saving Throws. Player Core pg. 400, Penalties.
#[test]
fn an_untyped_penalty_applies() {
    let untyped = penalty(PenaltyType::Untyped, -2, 1, Statistic::Fortitude);
    let roll = on_fortitude(2, Proficiency::Trained, &[untyped], 10, 13);
    assert_eq!(modifier_of(&roll), 3);
    assert_ne!(modifier_of(&roll), 5);
    assert_eq!(roll.total, 13);
}

/// Player Core pg. 446, Unconscious.
#[test]
fn unconscious_reflex_includes_minus_4() {
    let mut person = alive(1, 0, fort(1, 0, Proficiency::Untrained));
    person.state.vital = Vital::Unconscious;
    let (roll, _) = resolved(person, Save::Reflex, 15, 0, &[], 15);
    assert_eq!(modifier_of(&roll), -4);
    assert_eq!(roll.total, 11);
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 446, Unconscious.
#[test]
fn alive_reflex_does_not_include_minus_4() {
    let (roll, _) = resolved(
        alive(1, 30, fort(1, 0, Proficiency::Untrained)),
        Save::Reflex,
        15,
        0,
        &[],
        15,
    );
    assert_eq!(modifier_of(&roll), 0);
    assert_eq!(roll.total, 15);
    assert_eq!(roll.degree, DegreeOfSuccess::Success);
}

/// Player Core pg. 411, Dying. Player Core pg. 446, Unconscious.
#[test]
fn a_dying_creature_s_reflex_includes_minus_4() {
    let mut person = alive(
        1,
        0,
        ranks(
            1,
            0,
            2,
            0,
            Proficiency::Untrained,
            Proficiency::Trained,
            Proficiency::Untrained,
        ),
    );
    person.state.vital = dying_at(1);
    let (roll, encounter) = resolved(person, Save::Reflex, 11, 0, &[], 10);
    assert_eq!(modifier_of(&roll), 1);
    assert_eq!(roll.total, 11);
    assert_eq!(vital(&encounter, 1), dying_at(1));
}

/// Player Core pg. 446, Unconscious.
#[test]
fn unconscious_fortitude_does_not_include_minus_4() {
    let mut person = alive(1, 0, fort(1, 0, Proficiency::Untrained));
    person.state.vital = Vital::Unconscious;
    let (roll, _) = resolved(person, Save::Fortitude, 15, 0, &[], 15);
    assert_eq!(modifier_of(&roll), 0);
    assert_eq!(roll.total, 15);
    assert_eq!(roll.degree, DegreeOfSuccess::Success);
}

/// Player Core pg. 446, Unconscious.
#[test]
fn unconscious_will_does_not_include_minus_4() {
    let mut person = alive(1, 0, fort(1, 0, Proficiency::Untrained));
    person.state.vital = Vital::Unconscious;
    let (roll, _) = resolved(person, Save::Will, 15, 0, &[], 15);
    assert_eq!(modifier_of(&roll), 0);
    assert_eq!(roll.total, 15);
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn a_trained_constitution_at_the_maximum_rolls_nothing() {
    let mut encounter = opened(
        vec![alive(1, 20, fort(1, i32::MAX, Proficiency::Trained))],
        &[8],
    );
    let (error, mut dice) = attempt(&mut encounter, 1, Save::Fortitude, 15, 0, &[], &[10]);
    assert_eq!(
        error,
        Err(EncounterError::SaveOverflow(
            ArithmeticOverflow::SumDoesNotFit
        ))
    );
    assert_eq!(hit_points(&encounter, 1), 20);
    assert_eq!(vital(&encounter, 1), Vital::Alive);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(dice.d20(), 10);
}

/// Player Core pg. 407, Doubling and Halving Damage.
#[test]
fn critical_failure_of_2_to_the_30_writes_nothing() {
    let mut encounter = opened(vec![plain(1, 20)], &[8]);
    let (error, mut dice) = attempt(
        &mut encounter,
        1,
        Save::Fortitude,
        15,
        1 << 30,
        &[],
        &[5, 19],
    );
    assert_eq!(
        degree_of_success(5, 5, 15),
        DegreeOfSuccess::CriticalFailure
    );
    assert_eq!(
        error,
        Err(EncounterError::DamageOverflow(ModifierError::Overflow))
    );
    assert_eq!(hit_points(&encounter, 1), 20);
    assert_eq!(vital(&encounter, 1), Vital::Alive);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(dice.d20(), 19);
}

/// Player Core pg. 407, Doubling and Halving Damage.
#[test]
fn doubling_the_maximum_damage_writes_nothing() {
    let mut encounter = opened(vec![plain(1, 20)], &[8]);
    let (error, mut dice) = attempt(
        &mut encounter,
        1,
        Save::Fortitude,
        15,
        2_147_483_647,
        &[],
        &[5, 19],
    );
    assert_eq!(
        degree_of_success(5, 5, 15),
        DegreeOfSuccess::CriticalFailure
    );
    assert_eq!(
        error,
        Err(EncounterError::DamageOverflow(ModifierError::Overflow))
    );
    assert_eq!(hit_points(&encounter, 1), 20);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(dice.d20(), 19);
}

/// Player Core pg. 410, Getting Knocked Out.
#[test]
fn failure_to_0_sets_dying_1() {
    let (roll, encounter) = resolved(plain(1, 5), Save::Fortitude, 15, 5, &[], 6);
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
    assert_eq!(roll.damage, 5);
    assert_eq!(hit_points(&encounter, 1), 0);
    assert_eq!(vital(&encounter, 1), dying_at(1));
}

/// Player Core pg. 410, Getting Knocked Out. Player Core pg. 411, Taking Damage.
#[test]
fn critical_failure_to_0_sets_dying_2() {
    let (roll, encounter) = resolved(plain(1, 5), Save::Fortitude, 15, 5, &[], 5);
    assert_eq!(roll.damage, 10);
    assert_ne!(roll.damage, 20);
    assert_eq!(hit_points(&encounter, 1), 0);
    assert_eq!(vital(&encounter, 1), dying_at(2));
    assert_ne!(vital(&encounter, 1), dying_at(1));
}

/// Player Core pg. 411, Taking Damage.
#[test]
fn failure_while_dying_1_increases_to_dying_2() {
    let mut person = plain(1, 0);
    person.state.vital = dying_at(1);
    let (roll, encounter) = resolved(person, Save::Fortitude, 15, 1, &[], 6);
    assert_eq!(roll.damage, 1);
    assert_eq!(vital(&encounter, 1), dying_at(2));
}

/// Player Core pg. 411, Taking Damage.
#[test]
fn critical_failure_while_dying_1_increases_by_2() {
    let mut person = plain(1, 0);
    person.state.vital = dying_at(1);
    let (roll, encounter) = resolved(person, Save::Fortitude, 15, 1, &[], 5);
    assert_eq!(roll.damage, 2);
    assert_eq!(vital(&encounter, 1), dying_at(3));
}

/// Player Core pg. 410, Getting Knocked Out.
#[test]
fn without_dying_rules_0_hit_points_is_death() {
    let mut person = plain(1, 5);
    person.creature.uses_dying_rules = false;
    let (roll, encounter) = resolved(person, Save::Fortitude, 15, 5, &[], 6);
    assert_eq!(roll.damage, 5);
    assert_eq!(vital(&encounter, 1), Vital::Dead);
}

/// Player Core pg. 410, Temporary Hit Points.
#[test]
fn critical_failure_spends_temporary_hit_points_first() {
    let mut person = plain(1, 10);
    person.state.temp_hit_points = 4;
    person.state.temp_rounds = 1;
    let (roll, encounter) = resolved(person, Save::Fortitude, 15, 5, &[], 5);
    assert_eq!(roll.damage, 10);
    assert_eq!(temp_hp(&encounter, 1), 0);
    assert_eq!(hit_points(&encounter, 1), 4);
}

/// Player Core pg. 410, Temporary Hit Points. Player Core pg. 411, Taking Damage.
#[test]
fn failure_leaves_1_hit_point_and_no_dying() {
    let mut person = plain(1, 3);
    person.state.temp_hit_points = 5;
    person.state.temp_rounds = 1;
    let (roll, encounter) = resolved(person, Save::Fortitude, 15, 7, &[], 6);
    assert_eq!(roll.damage, 7);
    assert_eq!(temp_hp(&encounter, 1), 0);
    assert_eq!(hit_points(&encounter, 1), 1);
    assert_eq!(vital(&encounter, 1), Vital::Alive);
}

/// Player Core pg. 404, Basic Saving Throws. Player Core pg. 411, Taking Damage.
#[test]
fn dying_1_and_a_critical_success_on_listed_5_stays_dying_1() {
    let mut person = plain(1, 0);
    person.state.vital = dying_at(1);
    let (roll, encounter) = resolved(person, Save::Fortitude, 5, 5, &[], 15);
    assert_eq!(roll.degree, DegreeOfSuccess::CriticalSuccess);
    assert_eq!(roll.damage, 0);
    assert_eq!(hit_points(&encounter, 1), 0);
    assert_eq!(vital(&encounter, 1), dying_at(1));
}

/// Player Core pg. 404, Basic Saving Throws. Player Core pg. 411, Taking Damage.
#[test]
fn dying_1_and_a_success_on_listed_0_stays_dying_1() {
    let mut person = plain(1, 0);
    person.state.vital = dying_at(1);
    let (roll, encounter) = resolved(person, Save::Fortitude, 10, 0, &[], 10);
    assert_eq!(roll.degree, DegreeOfSuccess::Success);
    assert_eq!(roll.damage, 0);
    assert_eq!(hit_points(&encounter, 1), 0);
    assert_eq!(vital(&encounter, 1), dying_at(1));
}

/// Player Core pg. 404, Basic Saving Throws. Player Core pg. 410, Getting Knocked Out.
#[test]
fn unconscious_at_0_hit_points_and_a_critical_success_is_not_a_knockout() {
    let mut second = plain(2, 0);
    second.state.vital = Vital::Unconscious;
    let mut encounter = opened(vec![plain(1, 30), second], &[15, 10]);
    assert_eq!(encounter.order(), order_of(&[1, 2]).as_slice());
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let (result, _) = attempt(&mut encounter, 2, Save::Fortitude, 5, 5, &[], &[15]);
    let roll = result.expect("save");
    assert_eq!(roll.damage, 0);
    assert_eq!(vital(&encounter, 2), Vital::Unconscious);
    assert_eq!(hit_points(&encounter, 2), 0);
    assert_eq!(encounter.order(), order_of(&[1, 2]).as_slice());
}

/// Player Core pg. 262, Shield Block.
#[test]
fn a_raised_shield_does_not_block() {
    let person = body(Body {
        id: 1,
        current: 30,
        max_hp: 30,
        dying_rules: true,
        saves: fort(1, 0, Proficiency::Untrained),
        vital: Vital::Alive,
        temp: 0,
        shield: Some(steel()),
        raised: true,
        ac: 11,
    });
    let (roll, encounter) = resolved(person, Save::Fortitude, 15, 12, &[], 6);
    assert_eq!(roll.damage, 12);
    assert_eq!(hit_points(&encounter, 1), 18);
    assert_eq!(
        encounter.state(id(1)).expect("state").shield_hit_points,
        Some(20)
    );
    assert!(reaction(&encounter, 1));
}

/// Player Core pg. 404, Basic Saving Throws.
#[test]
fn a_damage_modifier_does_not_change_the_amount() {
    let damage = penalty(PenaltyType::Untyped, -6, 1, Statistic::Damage);
    let (roll, encounter) = resolved(plain(1, 30), Save::Fortitude, 15, 5, &[damage], 6);
    assert_eq!(roll.damage, 5);
    assert_eq!(hit_points(&encounter, 1), 25);
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn the_same_save_twice_matches() {
    let run = || {
        let (roll, encounter) = resolved(
            alive(1, 30, fort(1, 2, Proficiency::Trained)),
            Save::Fortitude,
            15,
            5,
            &[],
            10,
        );
        (
            roll.total,
            roll.degree,
            roll.damage,
            hit_points(&encounter, 1),
        )
    };
    let first = run();
    let second = run();
    assert_eq!(first, second);
    assert_eq!(first, (15, DegreeOfSuccess::Success, 2, 28));
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn an_unknown_defender_rolls_nothing() {
    let mut encounter = opened(vec![plain(1, 20)], &[10]);
    let (error, mut dice) = attempt(&mut encounter, 9, Save::Fortitude, 15, 5, &[], &[10]);
    assert_eq!(error, Err(EncounterError::UnknownCombatant));
    assert_eq!(dice.remaining(), 1);
    assert_eq!(dice.d20(), 10);
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn a_dead_defender_rolls_nothing() {
    let mut person = plain(1, 0);
    person.state.vital = Vital::Dead;
    let mut encounter = opened(vec![person], &[10]);
    let (error, mut dice) = attempt(&mut encounter, 1, Save::Fortitude, 15, -1, &[], &[10]);
    assert_eq!(error, Err(EncounterError::TargetDead));
    assert_ne!(error, Err(EncounterError::NegativeAmount));
    assert_eq!(hit_points(&encounter, 1), 0);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(dice.d20(), 10);
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn no_save_training_rolls_nothing() {
    let mut encounter = opened(vec![alive(1, 20, None)], &[10]);
    let (error, mut dice) = attempt(&mut encounter, 1, Save::Fortitude, 15, 5, &[], &[10]);
    assert_eq!(error, Err(EncounterError::NoSavingThrows));
    assert_eq!(hit_points(&encounter, 1), 20);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(dice.d20(), 10);
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn a_creature_may_save_against_itself() {
    let mut encounter = opened(vec![plain(1, 20)], &[10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let (result, _) = attempt(&mut encounter, 1, Save::Fortitude, 15, 5, &[], &[6]);
    assert_ne!(result, Err(EncounterError::SelfTarget));
    let roll = result.expect("save");
    assert_eq!(roll.damage, 5);
    assert_eq!(hit_points(&encounter, 1), 15);
}

/// Player Core pg. 444, Frightened. Player Core pg. 404, Saving Throws.
#[test]
fn frightened_2_lowers_fortitude_by_2() {
    let mut encounter = opened(
        vec![alive(1, 20, fort(1, 0, Proficiency::Untrained))],
        &[10],
    );
    encounter
        .apply_condition(
            id(1),
            crate::conditions::ConditionKind::Frightened,
            Some(2),
            crate::conditions::ConditionDuration::UntilRemoved,
        )
        .expect("frightened");
    let (result, _) = attempt(&mut encounter, 1, Save::Fortitude, 15, 0, &[], &[10]);
    let roll = result.expect("save");
    assert_eq!(roll.total, 8);
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 444, Frightened. Player Core pg. 446, Sickened.
#[test]
fn frightened_2_and_sickened_1_lower_will_by_2() {
    let saves = ranks(
        1,
        0,
        0,
        0,
        Proficiency::Untrained,
        Proficiency::Untrained,
        Proficiency::Untrained,
    );
    let mut encounter = opened(vec![alive(1, 20, saves)], &[10]);
    encounter
        .apply_condition(
            id(1),
            crate::conditions::ConditionKind::Frightened,
            Some(2),
            crate::conditions::ConditionDuration::UntilRemoved,
        )
        .expect("frightened");
    encounter
        .apply_condition(
            id(1),
            crate::conditions::ConditionKind::Sickened,
            Some(1),
            crate::conditions::ConditionDuration::UntilRemoved,
        )
        .expect("sickened");
    let (result, _) = attempt(&mut encounter, 1, Save::Will, 15, 0, &[], &[10]);
    let roll = result.expect("save");
    assert_eq!(roll.total, 8);
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 442, Clumsy.
#[test]
fn clumsy_2_lowers_reflex_and_not_fortitude() {
    let saves = ranks(
        1,
        0,
        0,
        0,
        Proficiency::Untrained,
        Proficiency::Untrained,
        Proficiency::Untrained,
    );
    let mut encounter = opened(vec![alive(1, 20, saves)], &[10, 10]);
    encounter
        .apply_condition(
            id(1),
            crate::conditions::ConditionKind::Clumsy,
            Some(2),
            crate::conditions::ConditionDuration::UntilRemoved,
        )
        .expect("clumsy");
    let (reflex, _) = attempt(&mut encounter, 1, Save::Reflex, 15, 0, &[], &[10]);
    let (fortitude, _) = attempt(&mut encounter, 1, Save::Fortitude, 15, 0, &[], &[10]);
    assert_eq!(reflex.expect("reflex").total, 8);
    assert_eq!(fortitude.expect("fortitude").total, 10);
}

/// Player Core pg. 443, Enfeebled.
#[test]
fn enfeebled_2_does_not_lower_fortitude() {
    let mut encounter = opened(
        vec![alive(1, 20, fort(1, 0, Proficiency::Untrained))],
        &[10],
    );
    encounter
        .apply_condition(
            id(1),
            crate::conditions::ConditionKind::Enfeebled,
            Some(2),
            crate::conditions::ConditionDuration::UntilRemoved,
        )
        .expect("enfeebled");
    let (result, _) = attempt(&mut encounter, 1, Save::Fortitude, 15, 0, &[], &[10]);
    let roll = result.expect("save");
    assert_eq!(roll.total, 10);
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 446, Unconscious. Player Core pg. 444, Frightened.
#[test]
fn frightened_1_and_unconscious_reflex_stay_at_minus_4() {
    let saves = ranks(
        1,
        0,
        0,
        0,
        Proficiency::Untrained,
        Proficiency::Untrained,
        Proficiency::Untrained,
    );
    let mut person = alive(1, 0, saves);
    person.state.vital = Vital::Unconscious;
    let mut encounter = opened(vec![person], &[15]);
    encounter
        .apply_condition(
            id(1),
            crate::conditions::ConditionKind::Frightened,
            Some(1),
            crate::conditions::ConditionDuration::UntilRemoved,
        )
        .expect("frightened");
    let (result, _) = attempt(&mut encounter, 1, Save::Reflex, 15, 0, &[], &[15]);
    let roll = result.expect("save");
    assert_eq!(roll.total, 11);
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
}

fn delay_then_reopen(people: Vec<Participant>, initiative: &[u8]) -> Encounter {
    let mut encounter = opened(people, initiative);
    assert_eq!(encounter.start_turn().expect("first"), id(1));
    encounter
        .end_turn(&mut crate::dice::ScriptedDice::new([]))
        .expect("end");
    assert_eq!(encounter.start_turn().expect("second"), id(2));
    encounter
        .delay(id(2), &mut crate::dice::ScriptedDice::new([]))
        .expect("delay");
    assert_eq!(encounter.start_turn().expect("reopen"), id(1));
    encounter
}

/// Player Core pg. 404, Saving Throws. Player Core pg. 416, Delay.
#[test]
fn a_delayed_creature_can_save() {
    let mut encounter = delay_then_reopen(vec![plain(1, 20), plain(2, 20)], &[15, 10]);
    assert_eq!(encounter.turn_slot(id(2)).expect("slot"), TurnSlot::Delayed);
    let (result, _) = attempt(&mut encounter, 2, Save::Fortitude, 15, 1, &[], &[6]);
    assert_ne!(result, Err(EncounterError::Delayed));
    let roll = result.expect("save");
    assert_eq!(roll.damage, 1);
    assert_eq!(hit_points(&encounter, 2), 19);
    assert!(reaction(&encounter, 2));
}

/// Player Core pg. 404, Saving Throws. Player Core pg. 446, Unconscious.
#[test]
fn an_unconscious_creature_can_save() {
    let mut person = plain(1, 0);
    person.state.vital = Vital::Unconscious;
    let mut encounter = opened(vec![person], &[10]);
    let (result, _) = attempt(&mut encounter, 1, Save::Fortitude, 15, 0, &[], &[6]);
    assert_ne!(result, Err(EncounterError::NotConscious));
    let roll = result.expect("save");
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
    assert_eq!(roll.damage, 0);
    assert_eq!(vital(&encounter, 1), Vital::Unconscious);
}

/// Player Core pg. 410, Getting Knocked Out.
#[test]
fn a_delayed_knockout_is_placed_before_the_open_creature() {
    let mut first = plain(1, 30);
    first.creature.uses_dying_rules = true;
    let mut second = plain(2, 5);
    second.creature.uses_dying_rules = true;
    let mut encounter = delay_then_reopen(vec![first, second], &[15, 10]);
    let (result, _) = attempt(&mut encounter, 2, Save::Fortitude, 15, 5, &[], &[6]);
    let _roll = result.expect("save");
    assert_eq!(vital(&encounter, 2), dying_at(1));
    assert_eq!(encounter.order(), order_of(&[2, 1]).as_slice());
}

/// Player Core pg. 410, Getting Knocked Out.
#[test]
fn a_knockout_that_is_not_delayed_moves_before_the_open_creature() {
    let mut first = plain(1, 30);
    first.creature.uses_dying_rules = true;
    let mut second = plain(2, 5);
    second.creature.uses_dying_rules = true;
    let mut encounter = opened(vec![first, second], &[15, 10]);
    assert_eq!(encounter.order(), order_of(&[1, 2]).as_slice());
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    assert_ne!(encounter.turn_slot(id(2)).expect("slot"), TurnSlot::Delayed);
    let (result, _) = attempt(&mut encounter, 2, Save::Fortitude, 15, 5, &[], &[6]);
    let _roll = result.expect("save");
    assert_eq!(vital(&encounter, 2), dying_at(1));
    assert_eq!(encounter.order(), order_of(&[2, 1]).as_slice());
}

/// Player Core pg. 404, Saving Throws. Player Core pg. 418, Strike.
#[test]
fn a_save_after_a_strike_is_not_a_strike() {
    let striker = alive(1, 30, None);
    let defender = alive(2, 30, fort(1, 0, Proficiency::Untrained));
    let (mut encounter, mut dice) = begin(vec![striker, defender], &[10, 8, 11, 8, 15]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let strike = rolled(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 1, 8, 0),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect("strike"),
    );
    assert_eq!(strike.total, 11);
    assert_eq!(strike.degree, DegreeOfSuccess::Success);
    assert_eq!(strike.damage, 8);
    assert_eq!(actions(&encounter, 1), 2);
    assert_eq!(encounter.attack_count(id(1)).expect("attacks"), 1);
    let roll = encounter
        .basic_save(id(2), Save::Reflex, 15, 5, &[], &mut dice)
        .expect("save");
    assert_eq!(roll.damage, 2);
    assert_eq!(hit_points(&encounter, 2), 20);
    assert_eq!(actions(&encounter, 1), 2);
    assert_eq!(encounter.attack_count(id(1)).expect("attacks"), 1);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core pg. 400, Flat Checks.
#[test]
fn a_save_draws_no_flat_check_die() {
    let mut encounter = opened(vec![plain(1, 20)], &[10]);
    let (result, mut dice) = attempt(&mut encounter, 1, Save::Fortitude, 15, 5, &[], &[6, 19]);
    let roll = result.expect("save");
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
    assert_eq!(roll.damage, 5);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(dice.d20(), 19);
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn a_save_before_any_turn_rolls() {
    let mut encounter = opened(vec![plain(1, 20)], &[10]);
    assert_eq!(encounter.turn_open(), None);
    let (result, _) = attempt(&mut encounter, 1, Save::Fortitude, 15, 5, &[], &[6]);
    assert_ne!(result, Err(EncounterError::TurnNotOpen));
    let roll = result.expect("save");
    assert_eq!(roll.damage, 5);
    assert_eq!(hit_points(&encounter, 1), 15);
    assert_eq!(actions(&encounter, 1), 0);
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn a_save_off_the_active_turn_rolls() {
    let defender = alive(
        2,
        20,
        ranks(
            1,
            0,
            0,
            0,
            Proficiency::Untrained,
            Proficiency::Untrained,
            Proficiency::Untrained,
        ),
    );
    let mut encounter = opened(vec![plain(1, 20), defender], &[15, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    assert_eq!(actions(&encounter, 1), 3);
    let (result, _) = attempt(&mut encounter, 2, Save::Will, 15, 5, &[], &[6]);
    assert_ne!(result, Err(EncounterError::NotYourTurn));
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(hit_points(&encounter, 2), 15);
}
