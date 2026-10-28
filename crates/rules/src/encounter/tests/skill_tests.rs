use super::*;
use crate::check::degree_of_success;
use crate::combatant::Vital;
use crate::modifiers::{BonusType, ConditionId, PenaltyType, SourceId};
use crate::save::{Level, Proficiency};
use crate::skill::{Skill, Skills};

fn level(value: i32) -> Level {
    Level::new(value).expect("level")
}

fn blank(level_value: i32) -> Skills {
    Skills {
        level: level(level_value),
        strength: 0,
        dexterity: 0,
        intelligence: 0,
        wisdom: 0,
        charisma: 0,
        acrobatics: Proficiency::Untrained,
        arcana: Proficiency::Untrained,
        athletics: Proficiency::Untrained,
        crafting: Proficiency::Untrained,
        deception: Proficiency::Untrained,
        diplomacy: Proficiency::Untrained,
        intimidation: Proficiency::Untrained,
        medicine: Proficiency::Untrained,
        nature: Proficiency::Untrained,
        occultism: Proficiency::Untrained,
        performance: Proficiency::Untrained,
        religion: Proficiency::Untrained,
        society: Proficiency::Untrained,
        stealth: Proficiency::Untrained,
        survival: Proficiency::Untrained,
        thievery: Proficiency::Untrained,
    }
}

fn athletics(level_value: i32, strength: i32, rank: Proficiency) -> Skills {
    Skills {
        strength,
        athletics: rank,
        ..blank(level_value)
    }
}

fn society(level_value: i32, intelligence: i32, rank: Proficiency) -> Skills {
    Skills {
        intelligence,
        society: rank,
        ..blank(level_value)
    }
}

fn with_skills(who: u32, skills: Option<Skills>) -> Participant {
    let mut person = pc(who, 0, 30, 11);
    person.creature.skills = skills;
    person
}

fn opened(people: Vec<Participant>) -> Encounter {
    let faces = vec![10u8; people.len()];
    let mut encounter = begin(people, &faces).0;
    encounter.start_turn().expect("start");
    encounter
}

fn check(
    encounter: &mut Encounter,
    who: u32,
    skill: Skill,
    dc: i32,
    modifiers: &[Modifier],
    face: u8,
) -> (
    Result<crate::skill::SkillRoll, EncounterError>,
    ScriptedDice,
) {
    let mut dice = ScriptedDice::new([face]);
    let result = encounter.skill_check(id(who), skill, dc, modifiers, &mut dice);
    (result, dice)
}

fn recall(
    encounter: &mut Encounter,
    who: u32,
    skill: Skill,
    dc: i32,
    modifiers: &[Modifier],
    face: u8,
) -> (
    Result<crate::skill::SkillRoll, EncounterError>,
    ScriptedDice,
) {
    let mut dice = ScriptedDice::new([face]);
    let result = encounter.recall_knowledge(id(who), skill, dc, modifiers, &mut dice);
    (result, dice)
}

fn rolled_check(
    skills: Skills,
    skill: Skill,
    dc: i32,
    modifiers: &[Modifier],
    face: u8,
) -> crate::skill::SkillRoll {
    let mut encounter = opened(vec![with_skills(1, Some(skills))]);
    let (result, dice) = check(&mut encounter, 1, skill, dc, modifiers, face);
    assert_eq!(dice.remaining(), 0);
    result.expect("check")
}

fn item(value: i32, source: u32, statistic: Statistic) -> Modifier {
    Modifier::bonus(
        BonusType::Item,
        value,
        SourceId(source),
        source,
        None,
        statistic,
    )
    .expect("item")
}

fn circumstance(value: i32, source: u32, statistic: Statistic) -> Modifier {
    Modifier::bonus(
        BonusType::Circumstance,
        value,
        SourceId(source),
        source,
        None,
        statistic,
    )
    .expect("circumstance")
}

fn penalty(ty: PenaltyType, value: i32, source: u32, statistic: Statistic) -> Modifier {
    Modifier::penalty(ty, value, SourceId(source), source, None, statistic).expect("penalty")
}

fn frightened(rank: u32, statistic: Statistic) -> Modifier {
    penalty(
        PenaltyType::Status,
        -i32::try_from(rank).expect("rank"),
        9,
        statistic,
    )
    .with_condition(ConditionId::FRIGHTENED, Some(rank))
    .expect("frightened")
}

fn numeric(total: i32, dc: i32) -> DegreeOfSuccess {
    degree_of_success(10, total, dc)
}

/// Player Core pg. 405, Skill Checks.
#[test]
fn athletics_is_strength_plus_trained_at_level_1() {
    let roll = rolled_check(
        athletics(1, 2, Proficiency::Trained),
        Skill::Athletics,
        15,
        &[],
        10,
    );
    assert_eq!(roll.total - i32::from(roll.natural), 5);
    assert_eq!(roll.total, 15);
    assert_eq!(roll.degree, DegreeOfSuccess::Success);
}

/// Player Core pg. 405, Skill Checks.
#[test]
fn acrobatics_is_dexterity_plus_expert_at_level_5() {
    let mut skills = athletics(5, 0, Proficiency::Untrained);
    skills.dexterity = -1;
    skills.acrobatics = Proficiency::Expert;
    let roll = rolled_check(skills, Skill::Acrobatics, 18, &[], 10);
    assert_eq!(roll.total - i32::from(roll.natural), 8);
    assert_eq!(roll.total, 18);
    assert_eq!(roll.degree, DegreeOfSuccess::Success);
}

/// Player Core pg. 405, Skill Checks.
#[test]
fn arcana_is_intelligence_plus_master_at_level_10() {
    let mut skills = athletics(10, 0, Proficiency::Untrained);
    skills.intelligence = 1;
    skills.arcana = Proficiency::Master;
    let roll = rolled_check(skills, Skill::Arcana, 27, &[], 10);
    assert_eq!(roll.total - i32::from(roll.natural), 17);
    assert_eq!(roll.total, 27);
    assert_eq!(roll.degree, DegreeOfSuccess::Success);
}

/// Player Core pg. 405, Skill Checks.
#[test]
fn medicine_is_wisdom_plus_legendary_at_level_20() {
    let mut skills = athletics(20, 0, Proficiency::Untrained);
    skills.wisdom = 4;
    skills.medicine = Proficiency::Legendary;
    let roll = rolled_check(skills, Skill::Medicine, 42, &[], 10);
    assert_eq!(roll.total - i32::from(roll.natural), 32);
    assert_eq!(roll.total, 42);
    assert_eq!(roll.degree, DegreeOfSuccess::Success);
}

/// Player Core pg. 405, Skill Checks.
#[test]
fn diplomacy_is_charisma_plus_untrained() {
    let mut skills = athletics(20, 0, Proficiency::Untrained);
    skills.charisma = 3;
    let roll = rolled_check(skills, Skill::Diplomacy, 13, &[], 10);
    assert_eq!(roll.total - i32::from(roll.natural), 3);
    assert_eq!(roll.total, 13);
    assert_eq!(roll.degree, DegreeOfSuccess::Success);
}

/// Player Core pg. 405, Skill Checks. Player Core pg. 400, Bonuses.
#[test]
fn an_item_bonus_adds_when_untrained() {
    let roll = rolled_check(
        athletics(1, 0, Proficiency::Untrained),
        Skill::Athletics,
        11,
        &[item(1, 1, Statistic::Athletics)],
        10,
    );
    assert_eq!(roll.total - i32::from(roll.natural), 1);
    assert_eq!(roll.total, 11);
}

/// Player Core pg. 400, Bonuses.
#[test]
fn two_item_bonuses_keep_the_highest() {
    let roll = rolled_check(
        athletics(1, 0, Proficiency::Untrained),
        Skill::Athletics,
        12,
        &[
            item(2, 1, Statistic::Athletics),
            item(1, 2, Statistic::Athletics),
        ],
        10,
    );
    assert_eq!(roll.total - i32::from(roll.natural), 2);
    assert_eq!(roll.total, 12);
}

/// Player Core pg. 400, Bonuses.
#[test]
fn a_circumstance_bonus_adds_beside_an_item_bonus() {
    let roll = rolled_check(
        athletics(1, 0, Proficiency::Trained),
        Skill::Athletics,
        15,
        &[
            item(1, 1, Statistic::Athletics),
            circumstance(1, 2, Statistic::Athletics),
        ],
        10,
    );
    assert_eq!(roll.total - i32::from(roll.natural), 5);
    assert_eq!(roll.total, 15);
}

/// Player Core pg. 400, Penalties.
#[test]
fn a_status_penalty_applies() {
    let roll = rolled_check(
        athletics(1, 0, Proficiency::Trained),
        Skill::Athletics,
        11,
        &[penalty(PenaltyType::Status, -2, 1, Statistic::Athletics)],
        10,
    );
    assert_eq!(roll.total - i32::from(roll.natural), 1);
    assert_eq!(roll.total, 11);
}

/// Player Core pg. 400, Penalties.
#[test]
fn an_item_bonus_and_an_item_penalty_both_apply() {
    let roll = rolled_check(
        athletics(1, 0, Proficiency::Untrained),
        Skill::Athletics,
        10,
        &[
            item(1, 1, Statistic::Athletics),
            penalty(PenaltyType::Item, -1, 2, Statistic::Athletics),
        ],
        10,
    );
    assert_eq!(roll.total - i32::from(roll.natural), 0);
    assert_eq!(roll.total, 10);
}

/// Player Core pg. 398, Duplicate Effects.
#[test]
fn the_same_skill_modifier_twice_matches() {
    let bonus = item(1, 1, Statistic::Athletics);
    let roll = rolled_check(
        athletics(1, 0, Proficiency::Untrained),
        Skill::Athletics,
        11,
        &[bonus, bonus],
        10,
    );
    assert_eq!(roll.total - i32::from(roll.natural), 1);
    assert_eq!(roll.total, 11);
}

/// Player Core pg. 400, Penalties.
#[test]
fn two_untyped_penalties_of_minus_1_on_athletics_give_minus_2() {
    let roll = rolled_check(
        athletics(1, 0, Proficiency::Untrained),
        Skill::Athletics,
        8,
        &[
            penalty(PenaltyType::Untyped, -1, 1, Statistic::Athletics),
            penalty(PenaltyType::Untyped, -1, 2, Statistic::Athletics),
        ],
        10,
    );
    assert_eq!(roll.total - i32::from(roll.natural), -2);
    assert_eq!(roll.total, 8);
}

/// Player Core pg. 400, Penalties.
#[test]
fn two_status_penalties_keep_only_the_worst() {
    let roll = rolled_check(
        athletics(1, 0, Proficiency::Untrained),
        Skill::Athletics,
        8,
        &[
            penalty(PenaltyType::Status, -2, 1, Statistic::Athletics),
            penalty(PenaltyType::Status, -1, 2, Statistic::Athletics),
        ],
        10,
    );
    assert_eq!(roll.total - i32::from(roll.natural), -2);
    assert_eq!(roll.total, 8);
}

/// Player Core pg. 405, Skill Checks.
#[test]
fn an_acrobatics_bonus_does_not_change_athletics() {
    let roll = rolled_check(
        athletics(1, 0, Proficiency::Untrained),
        Skill::Athletics,
        10,
        &[item(2, 1, Statistic::Acrobatics)],
        10,
    );
    assert_eq!(roll.total - i32::from(roll.natural), 0);
    assert_eq!(roll.total, 10);
}

/// Player Core pg. 405, Skill Checks.
#[test]
fn an_attack_bonus_does_not_change_athletics() {
    let roll = rolled_check(
        athletics(1, 0, Proficiency::Untrained),
        Skill::Athletics,
        10,
        &[item(2, 1, Statistic::Attack)],
        10,
    );
    assert_eq!(roll.total - i32::from(roll.natural), 0);
    assert_eq!(roll.total, 10);
}

/// Player Core pg. 401, Step 4: Degree of Success.
#[test]
fn total_dc_minus_1_on_a_skill_check_is_a_failure() {
    let roll = rolled_check(
        athletics(1, 0, Proficiency::Untrained),
        Skill::Athletics,
        15,
        &[],
        14,
    );
    assert_eq!(roll.total, 14);
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 401, Natural 1 and Natural 20.
#[test]
fn natural_1_on_a_skill_check_at_a_total_equal_to_the_dc_is_a_failure() {
    let roll = rolled_check(
        athletics(1, 14, Proficiency::Untrained),
        Skill::Athletics,
        15,
        &[],
        1,
    );
    assert_eq!(roll.total, 15);
    assert_eq!(numeric(roll.total, 15), DegreeOfSuccess::Success);
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 401, Natural 1 and Natural 20.
#[test]
fn natural_20_on_a_skill_check_at_dc_minus_11_is_a_failure() {
    let roll = rolled_check(
        athletics(1, 0, Proficiency::Untrained),
        Skill::Athletics,
        31,
        &[],
        20,
    );
    assert_eq!(roll.total, 20);
    assert_eq!(numeric(roll.total, 31), DegreeOfSuccess::CriticalFailure);
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 401, Step 4: Degree of Success.
#[test]
fn beating_a_skill_dc_by_10_is_a_critical_success() {
    let roll = rolled_check(
        athletics(1, 0, Proficiency::Untrained),
        Skill::Athletics,
        5,
        &[],
        15,
    );
    assert_eq!(roll.total, 15);
    assert_eq!(roll.degree, DegreeOfSuccess::CriticalSuccess);
}

/// Player Core pg. 401, Step 4: Degree of Success.
#[test]
fn a_negative_skill_total_against_dc_5_is_a_failure() {
    let roll = rolled_check(
        athletics(1, -5, Proficiency::Untrained),
        Skill::Athletics,
        5,
        &[],
        4,
    );
    assert_eq!(roll.total, -1);
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 401, Natural 1 and Natural 20.
#[test]
fn natural_20_on_a_skill_check_at_the_dc_is_a_critical_success() {
    let roll = rolled_check(
        athletics(1, 0, Proficiency::Untrained),
        Skill::Athletics,
        20,
        &[],
        20,
    );
    assert_eq!(roll.total, 20);
    assert_eq!(numeric(roll.total, 20), DegreeOfSuccess::Success);
    assert_eq!(roll.degree, DegreeOfSuccess::CriticalSuccess);
}

/// Player Core pg. 401, Natural 1 and Natural 20.
#[test]
fn natural_20_on_a_skill_check_at_dc_minus_10_is_a_failure() {
    let roll = rolled_check(
        athletics(1, 0, Proficiency::Untrained),
        Skill::Athletics,
        30,
        &[],
        20,
    );
    assert_eq!(roll.total, 20);
    assert_eq!(numeric(roll.total, 30), DegreeOfSuccess::CriticalFailure);
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 401, Natural 1 and Natural 20.
#[test]
fn natural_1_on_a_skill_check_at_dc_plus_10_is_a_success() {
    let roll = rolled_check(
        athletics(1, 14, Proficiency::Untrained),
        Skill::Athletics,
        5,
        &[],
        1,
    );
    assert_eq!(roll.total, 15);
    assert_eq!(numeric(roll.total, 5), DegreeOfSuccess::CriticalSuccess);
    assert_eq!(roll.degree, DegreeOfSuccess::Success);
}

/// Player Core pg. 401, Step 4: Degree of Success.
#[test]
fn a_total_of_exactly_dc_minus_10_is_a_critical_failure() {
    let roll = rolled_check(
        athletics(1, 0, Proficiency::Untrained),
        Skill::Athletics,
        15,
        &[],
        5,
    );
    assert_eq!(roll.total, 5);
    assert_eq!(roll.degree, DegreeOfSuccess::CriticalFailure);
}

/// Player Core pg. 231, Recall Knowledge.
#[test]
fn recall_knowledge_success_returns_that_degree() {
    let mut encounter = opened(vec![with_skills(
        1,
        Some(society(1, 0, Proficiency::Untrained)),
    )]);
    let (result, dice) = recall(&mut encounter, 1, Skill::Society, 15, &[], 15);
    assert_eq!(dice.remaining(), 0);
    let roll = result.expect("recall");
    assert_eq!(roll.total, 15);
    assert_eq!(roll.degree, DegreeOfSuccess::Success);
    assert_eq!(roll.natural, 15);
}

/// Player Core pg. 231, Recall Knowledge.
#[test]
fn recall_knowledge_critical_success_returns_that_degree() {
    let mut encounter = opened(vec![with_skills(
        1,
        Some(society(1, 0, Proficiency::Untrained)),
    )]);
    let roll = recall(&mut encounter, 1, Skill::Society, 5, &[], 15)
        .0
        .expect("recall");
    assert_eq!(roll.total, 15);
    assert_eq!(roll.degree, DegreeOfSuccess::CriticalSuccess);
}

/// Player Core pg. 231, Recall Knowledge.
#[test]
fn recall_knowledge_failure_returns_that_degree() {
    let mut encounter = opened(vec![with_skills(
        1,
        Some(society(1, 0, Proficiency::Untrained)),
    )]);
    let roll = recall(&mut encounter, 1, Skill::Society, 15, &[], 14)
        .0
        .expect("recall");
    assert_eq!(roll.total, 14);
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 231, Recall Knowledge.
#[test]
fn recall_knowledge_critical_failure_returns_that_degree() {
    let mut encounter = opened(vec![with_skills(
        1,
        Some(society(1, 0, Proficiency::Untrained)),
    )]);
    let roll = recall(&mut encounter, 1, Skill::Society, 15, &[], 5)
        .0
        .expect("recall");
    assert_eq!(roll.total, 5);
    assert_eq!(roll.degree, DegreeOfSuccess::CriticalFailure);
}

/// Player Core pg. 231, Recall Knowledge.
#[test]
fn untrained_recall_knowledge_is_legal() {
    let mut encounter = opened(vec![with_skills(
        1,
        Some(society(1, 0, Proficiency::Untrained)),
    )]);
    let result = recall(&mut encounter, 1, Skill::Society, 15, &[], 15).0;
    assert_eq!(result.expect("recall").degree, DegreeOfSuccess::Success);
}

/// Player Core pg. 231, Recall Knowledge.
#[test]
fn the_same_recall_twice_matches() {
    let skills = society(1, 0, Proficiency::Untrained);
    let mut first = opened(vec![with_skills(1, Some(skills))]);
    let mut second = opened(vec![with_skills(1, Some(skills))]);
    let left = recall(&mut first, 1, Skill::Society, 15, &[], 15)
        .0
        .expect("recall");
    let right = recall(&mut second, 1, Skill::Society, 15, &[], 15)
        .0
        .expect("recall");
    assert_eq!(left, right);
}

/// Player Core pg. 405, Skill Checks.
#[test]
fn a_modifier_that_does_not_fit_rolls_nothing() {
    let mut encounter = opened(vec![with_skills(
        1,
        Some(athletics(1, i32::MAX, Proficiency::Untrained)),
    )]);
    let mut dice = ScriptedDice::new([10]);
    let check = encounter.skill_check(id(1), Skill::Athletics, 15, &[], &mut dice);
    let recall = encounter.recall_knowledge(id(1), Skill::Athletics, 15, &[], &mut dice);
    assert_eq!(
        check,
        Err(EncounterError::SkillOverflow(
            crate::check::ArithmeticOverflow::SumDoesNotFit
        ))
    );
    assert_eq!(
        recall,
        Err(EncounterError::SkillOverflow(
            crate::check::ArithmeticOverflow::SumDoesNotFit
        ))
    );
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 231, Recall Knowledge. Player Core pg. 436, Step 2: Act.
#[test]
fn recall_knowledge_spends_one_action() {
    let mut encounter = opened(vec![with_skills(
        1,
        Some(society(1, 0, Proficiency::Untrained)),
    )]);
    let roll = recall(&mut encounter, 1, Skill::Society, 15, &[], 15)
        .0
        .expect("recall");
    assert_eq!(actions(&encounter, 1), 2);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
    assert_eq!(roll.degree, DegreeOfSuccess::Success);
}

/// Player Core pg. 231, Recall Knowledge.
#[test]
fn recall_knowledge_does_not_change_the_attack_count() {
    let mut encounter = opened(vec![with_skills(
        1,
        Some(society(1, 0, Proficiency::Untrained)),
    )]);
    encounter.spend_actions(id(1), 1).expect("spend");
    encounter.set_attack_count_for_test(id(1), 1);
    let (_result, dice) = recall(&mut encounter, 1, Skill::Society, 15, &[], 15);
    assert_eq!(dice.remaining(), 0);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 1);
    assert_eq!(actions(&encounter, 1), 1);
}

/// Player Core pg. 405, Skill Checks.
#[test]
fn a_skill_check_off_the_active_turn_rolls() {
    let mut encounter = opened(vec![
        with_skills(1, Some(athletics(1, 0, Proficiency::Untrained))),
        with_skills(2, Some(athletics(1, 0, Proficiency::Untrained))),
    ]);
    let (result, _) = check(&mut encounter, 2, Skill::Athletics, 15, &[], 6);
    assert!(!matches!(result, Err(EncounterError::NotYourTurn)));
    let roll = result.expect("check");
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(actions(&encounter, 2), 0);
}

/// Player Core pg. 405, Skill Checks.
#[test]
fn a_skill_check_before_any_turn_rolls() {
    let people = vec![with_skills(
        1,
        Some(athletics(1, 0, Proficiency::Untrained)),
    )];
    let mut encounter = begin(people, &[10]).0;
    let (result, _) = check(&mut encounter, 1, Skill::Athletics, 15, &[], 6);
    assert!(!matches!(result, Err(EncounterError::TurnNotOpen)));
    assert_eq!(result.expect("check").degree, DegreeOfSuccess::Failure);
    assert_eq!(actions(&encounter, 1), 0);
}

/// Player Core pg. 231, Recall Knowledge.
#[test]
fn recall_knowledge_before_any_turn_rolls_nothing() {
    let people = vec![with_skills(1, Some(society(1, 0, Proficiency::Untrained)))];
    let mut encounter = begin(people, &[10]).0;
    let mut dice = ScriptedDice::new([10]);
    let result = encounter.recall_knowledge(id(1), Skill::Society, 15, &[], &mut dice);
    assert_eq!(result, Err(EncounterError::TurnNotOpen));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 231, Recall Knowledge.
#[test]
fn recall_knowledge_off_the_active_turn_rolls_nothing() {
    let mut encounter = opened(vec![
        with_skills(1, Some(society(1, 0, Proficiency::Untrained))),
        with_skills(2, Some(society(1, 0, Proficiency::Untrained))),
    ]);
    let mut dice = ScriptedDice::new([10]);
    let result = encounter.recall_knowledge(id(2), Skill::Society, 15, &[], &mut dice);
    assert_eq!(result, Err(EncounterError::NotYourTurn));
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 446, Unconscious. Player Core pg. 231, Recall Knowledge.
#[test]
fn an_unconscious_actor_rolls_nothing() {
    let mut encounter = opened(vec![with_skills(
        1,
        Some(society(1, 0, Proficiency::Untrained)),
    )]);
    encounter.set_vital_for_test(id(1), Vital::Unconscious);
    let mut dice = ScriptedDice::new([8]);
    let result = encounter.recall_knowledge(id(1), Skill::Society, 15, &[], &mut dice);
    assert_eq!(result, Err(EncounterError::NotConscious));
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 446, Unconscious. Player Core pg. 405, Skill Checks.
#[test]
fn an_unconscious_creature_can_still_roll_a_skill_check() {
    let mut person = with_skills(1, Some(athletics(1, 0, Proficiency::Untrained)));
    person.state.vital = Vital::Unconscious;
    let mut encounter = begin(vec![person], &[10]).0;
    let (result, _) = check(&mut encounter, 1, Skill::Athletics, 15, &[], 6);
    assert!(!matches!(result, Err(EncounterError::NotConscious)));
    let roll = result.expect("check");
    assert_eq!(roll.total, 6);
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 436, Step 2: Act.
#[test]
fn insufficient_actions_roll_nothing() {
    let mut encounter = opened(vec![with_skills(
        1,
        Some(society(1, 0, Proficiency::Untrained)),
    )]);
    encounter.spend_actions(id(1), 3).expect("spend");
    let mut dice = ScriptedDice::new([8]);
    let result = encounter.recall_knowledge(id(1), Skill::Society, 15, &[], &mut dice);
    assert_eq!(result, Err(EncounterError::InsufficientActions));
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 231, Recall Knowledge.
#[test]
fn no_skill_training_rolls_nothing() {
    let mut encounter = opened(vec![pc(1, 0, 30, 11)]);
    let mut dice = ScriptedDice::new([10]);
    let result = encounter.recall_knowledge(id(1), Skill::Society, 15, &[], &mut dice);
    assert_eq!(result, Err(EncounterError::NoSkills));
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 405, Skill Checks.
#[test]
fn a_skill_check_with_no_training_rolls_nothing() {
    let mut encounter = begin(vec![pc(1, 0, 30, 11)], &[10]).0;
    let mut dice = ScriptedDice::new([10]);
    let result = encounter.skill_check(id(1), Skill::Athletics, 15, &[], &mut dice);
    assert_eq!(result, Err(EncounterError::NoSkills));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 405, Skill Checks.
#[test]
fn a_skill_check_draws_no_flat_check_die() {
    let mut encounter = opened(vec![with_skills(
        1,
        Some(athletics(1, 0, Proficiency::Untrained)),
    )]);
    let mut dice = ScriptedDice::new([6, 19]);
    let roll = encounter
        .skill_check(id(1), Skill::Athletics, 15, &[], &mut dice)
        .expect("check");
    assert_eq!(roll.natural, 6);
    assert_eq!(roll.degree, DegreeOfSuccess::Failure);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 405, Skill Checks.
#[test]
fn an_unknown_creature_rolls_nothing() {
    let mut encounter = begin(vec![pc(1, 0, 30, 11)], &[10]).0;
    let mut dice = ScriptedDice::new([10]);
    let result = encounter.skill_check(id(2), Skill::Athletics, 15, &[], &mut dice);
    assert_eq!(result, Err(EncounterError::UnknownCombatant));
    assert_eq!(dice.remaining(), 1);
}

/// The frightened producer is covered on the modifier. This check still sees it.
///
/// Player Core pg. 444, Frightened.
#[test]
fn frightened_on_the_attack_reaches_the_skill_check() {
    let roll = rolled_check(
        athletics(1, 0, Proficiency::Trained),
        Skill::Athletics,
        11,
        &[frightened(2, Statistic::Attack)],
        10,
    );
    assert_eq!(roll.total, 11);
}
