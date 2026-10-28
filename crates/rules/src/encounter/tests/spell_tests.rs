use super::{actions, begin, dead, enemy, hit_points, id, pc, reaction, relate, sq, vital};
use crate::check::DegreeOfSuccess;
use crate::combatant::{DyingValue, Vital};
use crate::detection::{Concealment, Detection, Detections};
use crate::dice::{Dice, ScriptedDice};
use crate::encounter::{
    BreatheFireOutcome, Encounter, EncounterError, SpellAttackHit, SpellAttackOutcome,
};
use crate::grid::Size;
use crate::modifiers::Statistic;
use crate::positioning::{ActionCost, ActionTrait};
use crate::save::{Level, Proficiency, SavingThrows};
use crate::spell::{Aim, AreaCreature, Spellcasting};

fn casting(level: i32, attribute: i32, proficiency: Proficiency) -> Spellcasting {
    Spellcasting {
        level: Level::new(level).expect("level"),
        attribute,
        proficiency,
    }
}

fn trained() -> Spellcasting {
    casting(1, 4, Proficiency::Trained)
}

fn with_casting(mut participant: crate::Participant, entry: Spellcasting) -> crate::Participant {
    participant.creature.spellcasting = Some(entry);
    participant
}

fn with_reflex(mut participant: crate::Participant, dexterity: i32) -> crate::Participant {
    participant.creature.saves = Some(SavingThrows {
        level: Level::new(1).expect("level"),
        constitution: 0,
        dexterity,
        wisdom: 0,
        fortitude: Proficiency::Untrained,
        reflex: Proficiency::Untrained,
        will: Proficiency::Untrained,
    });
    participant
}

fn with_dying_rules(mut participant: crate::Participant) -> crate::Participant {
    participant.creature.uses_dying_rules = true;
    participant
}

fn opened(participants: Vec<crate::Participant>, rolls: &[u8]) -> (Encounter, ScriptedDice) {
    let init = vec![8u8; participants.len()];
    let (mut encounter, _) = begin(participants, &init);
    encounter.start_turn().expect("start");
    (encounter, ScriptedDice::new(rolls.iter().copied()))
}

fn caster(hp: i32, ac: i32, entry: Spellcasting) -> crate::Participant {
    with_casting(pc(1, 5, hp, ac), entry)
}

fn foe(value: u32, hp: i32, ac: i32) -> crate::Participant {
    enemy(value, 0, hp, ac)
}

fn place(value: u32, x: i32, y: i32) -> AreaCreature {
    AreaCreature {
        id: id(value),
        square: sq(x, y),
        size: Size::Medium,
    }
}

fn names(cost: ActionCost, present: &[ActionTrait], absent: &[ActionTrait]) {
    let ActionCost::Actions { count: 2, traits } = cost else {
        panic!("expected 2 actions, got {cost:?}");
    };
    for action_trait in present {
        assert!(traits.contains(action_trait), "missing {action_trait:?}");
    }
    for action_trait in absent {
        assert!(!traits.contains(action_trait), "named {action_trait:?}");
    }
}

fn rolled(outcome: SpellAttackOutcome) -> SpellAttackHit {
    match outcome {
        SpellAttackOutcome::Rolled(hit) => hit,
        SpellAttackOutcome::Lost(lost) => panic!("expected a rolled attack, lost {lost:?}"),
    }
}

fn shoot(
    encounter: &mut Encounter,
    dice: &mut ScriptedDice,
    target: u32,
    from: (i32, i32),
    to: (i32, i32),
    modifiers: &[crate::Modifier],
    detections: &Detections,
) -> Result<SpellAttackOutcome, EncounterError> {
    encounter.telekinetic_projectile(
        id(1),
        id(target),
        sq(from.0, from.1),
        sq(to.0, to.1),
        Size::Medium,
        Size::Medium,
        modifiers,
        detections,
        dice,
    )
}

fn breathe(
    encounter: &mut Encounter,
    dice: &mut ScriptedDice,
    creatures: &[AreaCreature],
) -> Result<BreatheFireOutcome, EncounterError> {
    encounter.breathe_fire(
        id(1),
        sq(0, 0),
        Size::Medium,
        Aim::East,
        creatures,
        &[],
        dice,
    )
}

fn attacks(encounter: &Encounter, value: u32) -> u8 {
    encounter.attack_count(id(value)).expect("count")
}

fn set_vital(encounter: &mut Encounter, value: u32, vital: Vital, hp: i32) {
    let member = encounter
        .members
        .iter_mut()
        .find(|member| member.participant.id == id(value))
        .expect("member");
    member.participant.state.vital = vital;
    member.participant.state.hit_points = hp;
}

/// Player Core pg. 363, Telekinetic Projectile.
#[test]
fn the_same_projectile_twice_matches() {
    let setup = || {
        opened(
            vec![caster(20, 16, trained()), pc(2, 0, 20, 16)],
            &[10, 3, 4],
        )
    };
    let (mut first_encounter, mut first_dice) = setup();
    let (mut second_encounter, mut second_dice) = setup();
    let first = shoot(
        &mut first_encounter,
        &mut first_dice,
        2,
        (0, 0),
        (1, 0),
        &[],
        &Detections::new(),
    )
    .expect("cast");
    let second = shoot(
        &mut second_encounter,
        &mut second_dice,
        2,
        (0, 0),
        (1, 0),
        &[],
        &Detections::new(),
    )
    .expect("cast");
    assert_eq!(first, second);
    let hit = rolled(first);
    assert_eq!(hit.total, 17);
    assert_eq!(hit.degree, DegreeOfSuccess::Success);
    assert_eq!(hit.damage, 7);
    assert_eq!(first_dice.remaining(), 0);
    assert_eq!(second_dice.remaining(), 0);
}

/// Player Core pg. 319, Breathe Fire.
#[test]
fn the_same_cone_twice_matches() {
    let setup = || {
        opened(
            vec![caster(20, 16, trained()), with_reflex(foe(2, 20, 16), 0)],
            &[3, 4, 10],
        )
    };
    let (mut first_encounter, mut first_dice) = setup();
    let (mut second_encounter, mut second_dice) = setup();
    let creatures = [place(2, 1, 0)];
    let first = breathe(&mut first_encounter, &mut first_dice, &creatures).expect("cast");
    let second = breathe(&mut second_encounter, &mut second_dice, &creatures).expect("cast");
    assert_eq!(first, second);
    assert_eq!(first.rolled_damage, Some(7));
    assert_eq!(first.saves[0].degree, DegreeOfSuccess::Failure);
    assert_eq!(first_dice.remaining(), 0);
    assert_eq!(second_dice.remaining(), 0);
}

/// Player Core pg. 299, Cast a Spell. Player Core pg. 319, Breathe Fire.
#[test]
fn breathe_fire_leaves_one_action() {
    let (mut encounter, mut dice) = opened(vec![caster(20, 16, trained())], &[3]);
    let outcome = breathe(&mut encounter, &mut dice, &[]).expect("cast");
    assert_eq!(actions(&encounter, 1), 1);
    names(
        outcome.cost,
        &[
            ActionTrait::Concentrate,
            ActionTrait::Fire,
            ActionTrait::Manipulate,
        ],
        &[ActionTrait::Attack, ActionTrait::Ranged],
    );
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 299, Cast a Spell. Player Core pg. 363, Telekinetic Projectile.
#[test]
fn the_projectile_leaves_one_action() {
    let (mut encounter, mut dice) =
        opened(vec![caster(20, 16, trained()), pc(2, 0, 20, 30)], &[10]);
    let outcome = shoot(
        &mut encounter,
        &mut dice,
        2,
        (0, 0),
        (1, 0),
        &[],
        &Detections::new(),
    )
    .expect("cast");
    assert_eq!(actions(&encounter, 1), 1);
    names(
        rolled(outcome).cost,
        &[
            ActionTrait::Attack,
            ActionTrait::Cantrip,
            ActionTrait::Concentrate,
            ActionTrait::Manipulate,
        ],
        &[ActionTrait::Ranged],
    );
    assert!(reaction(&encounter, 1));
}

/// Player Core pg. 319, Breathe Fire.
#[test]
fn breathe_fire_has_no_attack_trait() {
    let (mut encounter, mut dice) = opened(vec![caster(20, 16, trained())], &[]);
    let outcome = breathe(&mut encounter, &mut dice, &[]).expect("cast");
    names(
        outcome.cost,
        &[
            ActionTrait::Concentrate,
            ActionTrait::Fire,
            ActionTrait::Manipulate,
        ],
        &[ActionTrait::Attack, ActionTrait::Ranged],
    );
}

/// Player Core pg. 363, Telekinetic Projectile. Player Core pg. 402, Multiple Attack Penalty.
#[test]
fn the_projectile_has_the_attack_trait() {
    let (mut encounter, mut dice) =
        opened(vec![caster(20, 16, trained()), pc(2, 0, 20, 30)], &[10]);
    let outcome = shoot(
        &mut encounter,
        &mut dice,
        2,
        (0, 0),
        (1, 0),
        &[],
        &Detections::new(),
    )
    .expect("cast");
    names(
        rolled(outcome).cost,
        &[
            ActionTrait::Attack,
            ActionTrait::Cantrip,
            ActionTrait::Concentrate,
            ActionTrait::Manipulate,
        ],
        &[ActionTrait::Ranged],
    );
}

/// Player Core pg. 299, Cast a Spell.
#[test]
fn breathe_fire_before_the_turn_starts_rolls_nothing() {
    let (encounter, _) = begin(vec![caster(20, 16, trained())], &[8]);
    let mut dice = ScriptedDice::new([3]);
    let error = encounter_breathe_without_open(&encounter, &mut dice);
    assert_eq!(error, Err(EncounterError::TurnNotOpen));
    assert_eq!(dice.remaining(), 1);
}

fn encounter_breathe_without_open(
    encounter: &Encounter,
    dice: &mut ScriptedDice,
) -> Result<BreatheFireOutcome, EncounterError> {
    let mut encounter = encounter.clone();
    breathe(&mut encounter, dice, &[])
}

/// Player Core pg. 299, Cast a Spell.
#[test]
fn a_projectile_off_the_active_turn_rolls_nothing() {
    let (mut encounter, mut dice) = opened(
        vec![
            with_casting(pc(1, 5, 20, 16), trained()),
            with_casting(pc(2, 0, 20, 16), trained()),
        ],
        &[10],
    );
    let error = encounter.telekinetic_projectile(
        id(2),
        id(1),
        sq(1, 0),
        sq(0, 0),
        Size::Medium,
        Size::Medium,
        &[],
        &Detections::new(),
        &mut dice,
    );
    assert_eq!(error, Err(EncounterError::NotYourTurn));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 299, Cast a Spell.
#[test]
fn an_unconscious_caster_rolls_nothing() {
    let (mut encounter, mut dice) = opened(vec![caster(20, 16, trained())], &[10]);
    set_vital(&mut encounter, 1, Vital::Unconscious, 0);
    let error = shoot(
        &mut encounter,
        &mut dice,
        1,
        (0, 0),
        (0, 0),
        &[],
        &Detections::new(),
    );
    assert_eq!(error, Err(EncounterError::NotConscious));
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 299, Cast a Spell.
#[test]
fn an_unknown_caster_rolls_nothing() {
    let (mut encounter, mut dice) = opened(vec![caster(20, 16, trained())], &[3]);
    let error = encounter.breathe_fire(
        id(9),
        sq(0, 0),
        Size::Medium,
        Aim::East,
        &[],
        &[],
        &mut dice,
    );
    assert_eq!(error, Err(EncounterError::UnknownCombatant));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 299, Cast a Spell.
#[test]
fn one_action_cannot_cast_either_spell() {
    let (mut fire, mut fire_dice) = opened(vec![caster(20, 16, trained())], &[10]);
    let (mut shot, mut shot_dice) =
        opened(vec![caster(20, 16, trained()), pc(2, 0, 20, 16)], &[10]);
    fire.spend_actions(id(1), 2).expect("spend");
    shot.spend_actions(id(1), 2).expect("spend");
    assert_eq!(
        breathe(&mut fire, &mut fire_dice, &[]),
        Err(EncounterError::InsufficientActions)
    );
    assert_eq!(
        shoot(
            &mut shot,
            &mut shot_dice,
            2,
            (0, 0),
            (1, 0),
            &[],
            &Detections::new()
        ),
        Err(EncounterError::InsufficientActions)
    );
    assert_eq!(actions(&fire, 1), 1);
    assert_eq!(actions(&shot, 1), 1);
    assert_eq!(fire_dice.remaining(), 1);
    assert_eq!(shot_dice.remaining(), 1);
}

/// Player Core pg. 403, Spell Attack Rolls.
#[test]
fn no_spellcasting_rolls_nothing() {
    let (mut encounter, mut dice) = opened(vec![pc(1, 5, 20, 16), pc(2, 0, 20, 16)], &[10]);
    let error = shoot(
        &mut encounter,
        &mut dice,
        2,
        (0, 0),
        (1, 0),
        &[],
        &Detections::new(),
    );
    assert_eq!(error, Err(EncounterError::NoSpellcasting));
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 402, Multiple Attack Penalty. Player Core pg. 403, Spell DC.
#[test]
fn the_second_attack_takes_minus_5_and_the_dc_stays_17() {
    let (mut encounter, mut dice) =
        opened(vec![caster(20, 16, trained()), pc(2, 0, 20, 18)], &[10]);
    encounter.set_attack_count_for_test(id(1), 1);
    let hit = rolled(
        shoot(
            &mut encounter,
            &mut dice,
            2,
            (0, 0),
            (1, 0),
            &[],
            &Detections::new(),
        )
        .expect("cast"),
    );
    assert_eq!(hit.map, -5);
    assert_eq!(hit.total, 12);
    assert_eq!(hit.degree, DegreeOfSuccess::Failure);
    assert_eq!(hit.spell_dc, 17);
    assert_eq!(attacks(&encounter, 1), 2);
}

/// Player Core pg. 402, Multiple Attack Penalty.
#[test]
fn an_agile_strike_does_not_make_the_spell_agile() {
    let mut striker = caster(20, 16, trained());
    striker.creature.strike.agile = true;
    let (mut encounter, mut dice) = opened(vec![striker, pc(2, 0, 20, 18)], &[10]);
    encounter.set_attack_count_for_test(id(1), 1);
    let hit = rolled(
        shoot(
            &mut encounter,
            &mut dice,
            2,
            (0, 0),
            (1, 0),
            &[],
            &Detections::new(),
        )
        .expect("cast"),
    );
    assert_eq!(hit.map, -5);
    assert_eq!(hit.total, 12);
}

/// Player Core pg. 319, Breathe Fire. Player Core pg. 402, Multiple Attack Penalty.
#[test]
fn breathe_fire_leaves_the_attack_count() {
    let (mut encounter, mut dice) = opened(vec![caster(20, 16, trained())], &[3]);
    encounter.set_attack_count_for_test(id(1), 1);
    breathe(&mut encounter, &mut dice, &[]).expect("cast");
    assert_eq!(attacks(&encounter, 1), 1);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 363, Telekinetic Projectile.
#[test]
fn a_natural_10_against_armor_class_16_is_a_success() {
    let (mut encounter, mut dice) = opened(
        vec![caster(20, 16, trained()), pc(2, 0, 20, 16)],
        &[10, 3, 4],
    );
    let hit = rolled(
        shoot(
            &mut encounter,
            &mut dice,
            2,
            (0, 0),
            (1, 0),
            &[],
            &Detections::new(),
        )
        .expect("cast"),
    );
    assert_eq!(hit.total, 17);
    assert_eq!(hit.degree, DegreeOfSuccess::Success);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core pg. 363, Telekinetic Projectile.
#[test]
fn armor_class_in_the_caller_list_does_not_change_the_result() {
    let (mut encounter, mut dice) = opened(
        vec![
            with_casting(pc(1, 5, 20, 12), casting(1, 0, Proficiency::Untrained)),
            pc(2, 0, 20, 12),
        ],
        &[10, 3],
    );
    let penalty = crate::modifiers::Modifier::penalty(
        crate::modifiers::PenaltyType::Circumstance,
        -2,
        crate::modifiers::SourceId(1),
        1,
        None,
        Statistic::ArmorClass,
    )
    .expect("penalty");
    let hit = rolled(
        shoot(
            &mut encounter,
            &mut dice,
            2,
            (0, 0),
            (1, 0),
            &[penalty],
            &Detections::new(),
        )
        .expect("cast"),
    );
    assert_eq!(hit.total, 10);
    assert_eq!(hit.degree, DegreeOfSuccess::Failure);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(dice.d20(), 3);
}

/// Player Core pg. 363, Telekinetic Projectile.
#[test]
fn a_natural_10_against_armor_class_18_rolls_no_damage_die() {
    let (mut encounter, mut dice) =
        opened(vec![caster(20, 16, trained()), pc(2, 0, 20, 18)], &[10, 3]);
    let hit = rolled(
        shoot(
            &mut encounter,
            &mut dice,
            2,
            (0, 0),
            (1, 0),
            &[],
            &Detections::new(),
        )
        .expect("cast"),
    );
    assert_eq!(hit.total, 17);
    assert_eq!(hit.degree, DegreeOfSuccess::Failure);
    assert_eq!(hit_points(&encounter, 2), 20);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 363, Telekinetic Projectile.
#[test]
fn a_natural_10_against_armor_class_16_hits_for_7() {
    let (mut encounter, mut dice) = opened(
        vec![caster(20, 16, trained()), pc(2, 0, 20, 16)],
        &[10, 3, 4],
    );
    let hit = rolled(
        shoot(
            &mut encounter,
            &mut dice,
            2,
            (0, 0),
            (1, 0),
            &[],
            &Detections::new(),
        )
        .expect("cast"),
    );
    assert_eq!(hit.damage, 7);
    assert_eq!(hit_points(&encounter, 2), 13);
    assert_eq!(vital(&encounter, 2), Vital::Alive);
}

/// Player Core pg. 363, Telekinetic Projectile. Player Core pg. 407, Doubling and Halving Damage.
#[test]
fn a_natural_20_doubles_7_to_14_once() {
    let (mut encounter, mut dice) = opened(
        vec![
            caster(20, 16, trained()),
            with_dying_rules(pc(2, 0, 10, 18)),
        ],
        &[20, 3, 4],
    );
    let hit = rolled(
        shoot(
            &mut encounter,
            &mut dice,
            2,
            (0, 0),
            (1, 0),
            &[],
            &Detections::new(),
        )
        .expect("cast"),
    );
    assert_eq!(hit.total, 27);
    assert_eq!(hit.degree, DegreeOfSuccess::CriticalSuccess);
    assert_eq!(hit.damage, 14);
    assert_eq!(hit_points(&encounter, 2), 0);
    assert_eq!(
        vital(&encounter, 2),
        Vital::Dying(DyingValue::new(2).expect("dying"))
    );
}

/// Player Core pg. 401, Natural 1 and Natural 20.
#[test]
fn a_natural_20_at_armor_class_minus_1_is_a_success() {
    let (mut encounter, mut dice) = opened(
        vec![
            with_casting(pc(1, 5, 20, 21), casting(1, 0, Proficiency::Untrained)),
            pc(2, 0, 20, 21),
        ],
        &[20, 3, 4],
    );
    let hit = rolled(
        shoot(
            &mut encounter,
            &mut dice,
            2,
            (0, 0),
            (1, 0),
            &[],
            &Detections::new(),
        )
        .expect("cast"),
    );
    assert_eq!(hit.total, 20);
    assert_eq!(hit.degree, DegreeOfSuccess::Success);
    assert_eq!(hit.damage, 7);
    assert_eq!(hit_points(&encounter, 2), 13);
}

/// Player Core pg. 401, Natural 1 and Natural 20.
#[test]
fn a_natural_1_at_armor_class_plus_10_is_a_success() {
    let (mut encounter, mut dice) = opened(
        vec![
            with_casting(pc(1, 5, 20, 20), casting(1, 29, Proficiency::Untrained)),
            pc(2, 0, 20, 20),
        ],
        &[1, 3, 4],
    );
    let hit = rolled(
        shoot(
            &mut encounter,
            &mut dice,
            2,
            (0, 0),
            (1, 0),
            &[],
            &Detections::new(),
        )
        .expect("cast"),
    );
    assert_eq!(hit.total, 30);
    assert_eq!(hit.degree, DegreeOfSuccess::Success);
    assert_eq!(hit.damage, 7);
    assert_eq!(hit_points(&encounter, 2), 13);
}

/// Player Core pg. 401, Natural 1 and Natural 20.
#[test]
fn a_natural_1_at_armor_class_is_a_failure() {
    let (mut encounter, mut dice) = opened(
        vec![
            with_casting(pc(1, 5, 20, 20), casting(1, 19, Proficiency::Untrained)),
            pc(2, 0, 20, 20),
        ],
        &[1, 3],
    );
    let hit = rolled(
        shoot(
            &mut encounter,
            &mut dice,
            2,
            (0, 0),
            (1, 0),
            &[],
            &Detections::new(),
        )
        .expect("cast"),
    );
    assert_eq!(hit.total, 20);
    assert_eq!(hit.degree, DegreeOfSuccess::Failure);
    assert_eq!(hit_points(&encounter, 2), 20);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 401, Natural 1 and Natural 20.
#[test]
fn a_natural_1_one_below_armor_class_is_a_critical_failure() {
    let (mut encounter, mut dice) = opened(
        vec![
            with_casting(pc(1, 5, 20, 20), casting(1, 18, Proficiency::Untrained)),
            pc(2, 0, 20, 20),
        ],
        &[1, 3],
    );
    let hit = rolled(
        shoot(
            &mut encounter,
            &mut dice,
            2,
            (0, 0),
            (1, 0),
            &[],
            &Detections::new(),
        )
        .expect("cast"),
    );
    assert_eq!(hit.total, 19);
    assert_eq!(hit.degree, DegreeOfSuccess::CriticalFailure);
    assert_eq!(hit_points(&encounter, 2), 20);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 319, Breathe Fire. Player Core pg. 363, Telekinetic Projectile.
#[test]
fn a_level_5_projectile_still_deals_7() {
    let (mut encounter, mut dice) = opened(
        vec![
            with_casting(pc(1, 5, 20, 16), casting(5, 3, Proficiency::Expert)),
            pc(2, 0, 20, 16),
        ],
        &[10, 3, 4],
    );
    let hit = rolled(
        shoot(
            &mut encounter,
            &mut dice,
            2,
            (0, 0),
            (1, 0),
            &[],
            &Detections::new(),
        )
        .expect("cast"),
    );
    assert_eq!(hit.total, 22);
    assert_eq!(hit.damage, 7);
    assert_eq!(hit_points(&encounter, 2), 13);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core pg. 319, Breathe Fire.
#[test]
fn a_level_5_cone_still_deals_7() {
    let (mut encounter, mut dice) = opened(
        vec![
            with_casting(pc(1, 5, 20, 16), casting(5, 3, Proficiency::Expert)),
            with_reflex(foe(2, 20, 16), 0),
        ],
        &[3, 4, 13],
    );
    let outcome = breathe(&mut encounter, &mut dice, &[place(2, 1, 0)]).expect("cast");
    assert_eq!(outcome.saves[0].total, 13);
    assert_eq!(outcome.rolled_damage, Some(7));
    assert_eq!(outcome.saves[0].degree, DegreeOfSuccess::Failure);
    assert_eq!(hit_points(&encounter, 2), 13);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core pg. 363, Telekinetic Projectile. Player Core pg. 426, Range and Reach.
#[test]
fn six_orthogonal_squares_are_in_range() {
    let (mut encounter, mut dice) =
        opened(vec![caster(20, 16, trained()), pc(2, 0, 20, 18)], &[10]);
    let hit = rolled(
        shoot(
            &mut encounter,
            &mut dice,
            2,
            (0, 0),
            (6, 0),
            &[],
            &Detections::new(),
        )
        .expect("cast"),
    );
    assert_eq!(hit.total, 17);
    assert_eq!(hit.map, 0);
}

/// Player Core pg. 363, Telekinetic Projectile.
#[test]
fn seven_orthogonal_squares_spend_nothing() {
    let (mut encounter, mut dice) =
        opened(vec![caster(20, 16, trained()), pc(2, 0, 20, 18)], &[10]);
    let error = shoot(
        &mut encounter,
        &mut dice,
        2,
        (0, 0),
        (7, 0),
        &[],
        &Detections::new(),
    );
    assert_eq!(error, Err(EncounterError::OutOfSpellRange));
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Targets: 1 creature. Player Core pg. 363, Telekinetic Projectile.
#[test]
fn the_caster_may_target_their_own_square() {
    let (mut encounter, mut dice) = opened(vec![caster(20, 16, trained())], &[10, 3, 4]);
    let hit = rolled(
        shoot(
            &mut encounter,
            &mut dice,
            1,
            (0, 0),
            (0, 0),
            &[],
            &Detections::new(),
        )
        .expect("cast"),
    );
    assert_eq!(hit.degree, DegreeOfSuccess::Success);
    assert_eq!(hit.damage, 7);
    assert_eq!(hit_points(&encounter, 1), 13);
}

/// Targets: 1 creature. An ally is legal. Player Core pg. 363, Telekinetic Projectile.
#[test]
fn an_ally_is_a_legal_target() {
    let (mut encounter, mut dice) = opened(
        vec![caster(20, 16, trained()), pc(2, 0, 20, 16)],
        &[10, 3, 4],
    );
    let hit = rolled(
        shoot(
            &mut encounter,
            &mut dice,
            2,
            (0, 0),
            (1, 0),
            &[],
            &Detections::new(),
        )
        .expect("cast"),
    );
    assert_eq!(hit.degree, DegreeOfSuccess::Success);
    assert_eq!(hit.damage, 7);
    assert_eq!(hit_points(&encounter, 2), 13);
}

/// Player Core pg. 363, Telekinetic Projectile.
#[test]
fn a_dead_target_spends_nothing() {
    let (mut encounter, mut dice) = opened(
        vec![caster(20, 16, trained()), dead(pc(2, 0, 20, 16))],
        &[10],
    );
    let error = shoot(
        &mut encounter,
        &mut dice,
        2,
        (0, 0),
        (1, 0),
        &[],
        &Detections::new(),
    );
    assert_eq!(error, Err(EncounterError::TargetDead));
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 363, Telekinetic Projectile.
#[test]
fn an_unknown_target_spends_nothing() {
    let (mut encounter, mut dice) = opened(vec![caster(20, 16, trained())], &[10]);
    let error = shoot(
        &mut encounter,
        &mut dice,
        9,
        (0, 0),
        (1, 0),
        &[],
        &Detections::new(),
    );
    assert_eq!(error, Err(EncounterError::UnknownCombatant));
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 442, Concealed.
#[test]
fn a_lost_concealed_check_spends_the_cast() {
    let (mut encounter, mut dice) =
        opened(vec![caster(20, 16, trained()), pc(2, 0, 20, 16)], &[4, 10]);
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Observed,
        Concealment::Concealed,
        None,
    );
    let outcome = shoot(
        &mut encounter,
        &mut dice,
        2,
        (0, 0),
        (1, 0),
        &[],
        &detections,
    )
    .expect("cast");
    assert!(matches!(outcome, SpellAttackOutcome::Lost(_)));
    assert_eq!(actions(&encounter, 1), 1);
    assert_eq!(attacks(&encounter, 1), 1);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 442, Concealed.
#[test]
fn a_concealed_success_then_rolls_the_attack() {
    let (mut encounter, mut dice) = opened(
        vec![caster(20, 16, trained()), pc(2, 0, 20, 16)],
        &[5, 10, 3, 4],
    );
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Observed,
        Concealment::Concealed,
        None,
    );
    let hit = rolled(
        shoot(
            &mut encounter,
            &mut dice,
            2,
            (0, 0),
            (1, 0),
            &[],
            &detections,
        )
        .expect("cast"),
    );
    assert_eq!(hit.total, 17);
    assert_eq!(hit.damage, 7);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core pg. 442, Concealed.
#[test]
fn empty_detection_draws_no_flat_check() {
    let (mut encounter, mut dice) = opened(
        vec![caster(20, 16, trained()), pc(2, 0, 20, 16)],
        &[10, 3, 4],
    );
    let hit = rolled(
        shoot(
            &mut encounter,
            &mut dice,
            2,
            (0, 0),
            (1, 0),
            &[],
            &Detections::new(),
        )
        .expect("cast"),
    );
    assert_eq!(hit.total, 17);
    assert_eq!(hit.damage, 7);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core pg. 447, Undetected.
#[test]
fn an_undetected_target_spends_nothing() {
    let (mut encounter, mut dice) =
        opened(vec![caster(20, 16, trained()), pc(2, 0, 20, 16)], &[10]);
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Undetected,
        Concealment::NotConcealed,
        Some(15),
    );
    let error = shoot(
        &mut encounter,
        &mut dice,
        2,
        (0, 0),
        (1, 0),
        &[],
        &detections,
    );
    assert_eq!(error, Err(EncounterError::CannotTarget));
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 442, Concealed. Player Core pg. 319, Breathe Fire.
#[test]
fn a_concealed_creature_in_the_cone_draws_no_flat_check() {
    let (mut encounter, mut dice) = opened(
        vec![caster(20, 16, trained()), with_reflex(foe(2, 20, 16), 0)],
        &[3, 4, 10],
    );
    let _detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Observed,
        Concealment::Concealed,
        None,
    );
    let outcome = breathe(&mut encounter, &mut dice, &[place(2, 1, 0)]).expect("cast");
    assert_eq!(outcome.rolled_damage, Some(7));
    assert_eq!(outcome.saves[0].total, 10);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core pg. 319, Breathe Fire. Player Core pg. 404, Basic Saving Throws.
#[test]
fn failure_deals_7_and_success_deals_3() {
    let (mut encounter, mut dice) = opened(
        vec![
            caster(20, 16, trained()),
            with_reflex(foe(2, 20, 16), 0),
            with_reflex(pc(5, 0, 20, 16), 0),
        ],
        &[3, 4, 10, 17],
    );
    let outcome =
        breathe(&mut encounter, &mut dice, &[place(2, 1, 0), place(5, 2, 0)]).expect("cast");
    assert_eq!(outcome.rolled_damage, Some(7));
    assert_eq!(outcome.saves[0].degree, DegreeOfSuccess::Failure);
    assert_eq!(hit_points(&encounter, 2), 13);
    assert_eq!(outcome.saves[1].degree, DegreeOfSuccess::Success);
    assert_eq!(hit_points(&encounter, 5), 17);
    assert_eq!(actions(&encounter, 1), 1);
    assert_eq!(attacks(&encounter, 1), 0);
}

/// Player Core pg. 404, Basic Saving Throws.
#[test]
fn a_critical_success_deals_0() {
    let (mut encounter, mut dice) = opened(
        vec![caster(20, 16, trained()), with_reflex(foe(2, 20, 16), 0)],
        &[3, 4, 20],
    );
    let outcome = breathe(&mut encounter, &mut dice, &[place(2, 1, 0)]).expect("cast");
    assert_eq!(outcome.saves[0].degree, DegreeOfSuccess::CriticalSuccess);
    assert_eq!(outcome.saves[0].damage, 0);
    assert_eq!(hit_points(&encounter, 2), 20);
}

/// Player Core pg. 404, Basic Saving Throws. Player Core pg. 407, Doubling and Halving Damage.
#[test]
fn a_critical_failure_doubles_7_to_14() {
    let (mut encounter, mut dice) = opened(
        vec![caster(20, 16, trained()), with_reflex(foe(2, 20, 16), 0)],
        &[3, 4, 1],
    );
    let outcome = breathe(&mut encounter, &mut dice, &[place(2, 1, 0)]).expect("cast");
    assert_eq!(outcome.saves[0].degree, DegreeOfSuccess::CriticalFailure);
    assert_eq!(outcome.saves[0].damage, 14);
    assert_eq!(hit_points(&encounter, 2), 6);
    assert_eq!(vital(&encounter, 2), Vital::Alive);
}

/// Player Core pg. 401, Natural 1 and Natural 20. Player Core pg. 404, Basic Saving Throws.
#[test]
fn a_natural_1_turns_a_success_into_a_failure() {
    let (mut encounter, mut dice) = opened(
        vec![caster(20, 16, trained()), with_reflex(foe(2, 20, 16), 16)],
        &[3, 4, 1],
    );
    let outcome = breathe(&mut encounter, &mut dice, &[place(2, 1, 0)]).expect("cast");
    assert_eq!(outcome.saves[0].total, 17);
    assert_eq!(outcome.saves[0].degree, DegreeOfSuccess::Failure);
    // A final failure uses the existing basic-save multiplier, so the full 7 is dealt.
    assert_eq!(outcome.saves[0].damage, 7);
    assert_eq!(hit_points(&encounter, 2), 13);
}

/// Player Core pg. 446, Unconscious. Rulings R-08, R-09, and R-10.
#[test]
fn a_dying_creature_who_fails_goes_to_dying_2() {
    let (mut encounter, mut dice) = opened(
        vec![
            caster(20, 16, trained()),
            with_reflex(with_dying_rules(foe(2, 20, 16)), 0),
        ],
        &[3, 4, 13],
    );
    set_vital(
        &mut encounter,
        2,
        Vital::Dying(DyingValue::new(1).expect("dying")),
        0,
    );
    let outcome = breathe(&mut encounter, &mut dice, &[place(2, 1, 0)]).expect("cast");
    assert_eq!(outcome.saves[0].total, 9);
    assert_eq!(outcome.saves[0].degree, DegreeOfSuccess::Failure);
    assert_eq!(outcome.saves[0].damage, 7);
    assert_eq!(
        vital(&encounter, 2),
        Vital::Dying(DyingValue::new(2).expect("dying"))
    );
}

/// Player Core pg. 446, Unconscious. Rulings R-08, R-09, and R-10.
#[test]
fn a_critical_success_leaves_a_dying_creature_at_dying_1() {
    let (mut encounter, mut dice) = opened(
        vec![
            caster(20, 16, trained()),
            with_reflex(with_dying_rules(foe(2, 20, 16)), 1),
        ],
        &[3, 4, 20],
    );
    set_vital(
        &mut encounter,
        2,
        Vital::Dying(DyingValue::new(1).expect("dying")),
        0,
    );
    let outcome = breathe(&mut encounter, &mut dice, &[place(2, 1, 0)]).expect("cast");
    assert_eq!(outcome.saves[0].total, 17);
    assert_eq!(outcome.saves[0].degree, DegreeOfSuccess::CriticalSuccess);
    assert_eq!(outcome.saves[0].damage, 0);
    assert_eq!(
        vital(&encounter, 2),
        Vital::Dying(DyingValue::new(1).expect("dying"))
    );
    assert_eq!(hit_points(&encounter, 2), 0);
}

/// Player Core pg. 428, Areas.
#[test]
fn standard_cover_does_not_raise_the_reflex_save() {
    let (mut encounter, mut dice) = opened(
        vec![caster(20, 16, trained()), with_reflex(foe(2, 20, 16), 0)],
        &[3, 4, 10],
    );
    let outcome = breathe(&mut encounter, &mut dice, &[place(2, 1, 0)]).expect("cast");
    assert_eq!(outcome.saves[0].total, 10);
    assert_eq!(outcome.saves[0].degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 428, Cone.
#[test]
fn the_excluded_adjacent_squares_are_safe() {
    let (mut encounter, mut dice) = opened(
        vec![
            caster(20, 16, trained()),
            with_reflex(foe(2, 20, 16), 0),
            with_reflex(foe(3, 20, 16), 0),
        ],
        &[3],
    );
    breathe(
        &mut encounter,
        &mut dice,
        &[place(2, 1, -1), place(3, 1, 1)],
    )
    .expect("cast");
    assert_eq!(hit_points(&encounter, 2), 20);
    assert_eq!(hit_points(&encounter, 3), 20);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 406, Damage Rolls. Player Core pg. 428, Cone.
#[test]
fn a_dead_creature_in_the_cone_draws_no_save() {
    let (mut encounter, mut dice) = opened(
        vec![
            caster(20, 16, trained()),
            dead(foe(2, 20, 16)),
            with_reflex(foe(5, 20, 16), 0),
        ],
        &[3, 4, 10],
    );
    let outcome =
        breathe(&mut encounter, &mut dice, &[place(2, 1, 0), place(5, 2, 0)]).expect("cast");
    assert_eq!(vital(&encounter, 2), Vital::Dead);
    assert_eq!(outcome.saves.len(), 1);
    assert_eq!(outcome.saves[0].degree, DegreeOfSuccess::Failure);
    assert_eq!(hit_points(&encounter, 5), 13);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn a_duplicate_id_rolls_nothing() {
    let (mut encounter, mut dice) = opened(
        vec![caster(20, 16, trained()), with_reflex(foe(2, 20, 16), 0)],
        &[3],
    );
    let error = breathe(&mut encounter, &mut dice, &[place(2, 1, 0), place(2, 2, 0)]);
    assert_eq!(error, Err(EncounterError::DuplicateAreaTarget));
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn no_save_training_in_the_cone_rolls_nothing() {
    let (mut encounter, mut dice) = opened(vec![caster(20, 16, trained()), foe(2, 20, 16)], &[3]);
    let error = breathe(&mut encounter, &mut dice, &[place(2, 1, 0)]);
    assert_eq!(error, Err(EncounterError::NoSavingThrows));
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn no_save_training_outside_the_cone_still_casts() {
    let (mut encounter, mut dice) = opened(vec![caster(20, 16, trained()), foe(2, 20, 16)], &[3]);
    breathe(&mut encounter, &mut dice, &[place(2, 1, -1)]).expect("cast");
    assert_eq!(actions(&encounter, 1), 1);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 403, Spell DC.
#[test]
fn a_spell_dc_that_does_not_fit_spends_nothing() {
    let (mut encounter, mut dice) = opened(
        vec![with_casting(
            pc(1, 5, 20, 16),
            casting(1, i32::MAX, Proficiency::Trained),
        )],
        &[3],
    );
    let error = breathe(&mut encounter, &mut dice, &[]);
    assert_eq!(
        error,
        Err(EncounterError::AttackOverflow(
            crate::modifiers::ModifierError::Overflow
        ))
    );
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn a_later_save_overflow_leaves_state_unchanged() {
    let (mut encounter, mut dice) = opened(
        vec![
            caster(20, 16, trained()),
            with_reflex(foe(2, 20, 16), 0),
            with_reflex(foe(5, 20, 16), i32::MAX),
        ],
        &[3, 4, 10],
    );
    let error = breathe(&mut encounter, &mut dice, &[place(2, 1, 0), place(5, 2, 0)]);
    assert!(matches!(error, Err(EncounterError::SaveOverflow(_))));
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(hit_points(&encounter, 2), 20);
    assert_eq!(hit_points(&encounter, 5), 20);
    assert_eq!(vital(&encounter, 2), Vital::Alive);
}
