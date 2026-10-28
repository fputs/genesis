use super::*;
use crate::combatant::ConditionSlot;
use crate::conditions::{AttackBasis, ConditionDuration, ConditionKind, DamageBasis};
use crate::damage::DamageType;
use crate::ready::{ReactionField, ReactionPlace};

fn until_removed() -> ConditionDuration {
    ConditionDuration::UntilRemoved
}

fn rounds_on(remaining: u32, on: u32) -> ConditionDuration {
    ConditionDuration::Rounds {
        remaining,
        on: id(on),
    }
}

fn until_end(on: u32) -> ConditionDuration {
    ConditionDuration::UntilEndOfNextTurn {
        on: id(on),
        due: false,
    }
}

fn give(
    encounter: &mut Encounter,
    who: u32,
    kind: ConditionKind,
    value: Option<u32>,
    duration: ConditionDuration,
) {
    encounter
        .apply_condition(id(who), kind, value, duration)
        .expect("condition");
}

fn value_of(encounter: &Encounter, who: u32, kind: ConditionKind) -> Option<u32> {
    encounter
        .state(id(who))
        .expect("state")
        .conditions
        .iter()
        .filter(|slot| slot.kind() == Some(kind))
        .filter_map(|slot| slot.value())
        .max()
}

fn has(encounter: &Encounter, who: u32, kind: ConditionKind) -> bool {
    encounter
        .state(id(who))
        .expect("state")
        .conditions
        .iter()
        .any(|slot| slot.kind() == Some(kind))
}

fn rounds_left(encounter: &Encounter, who: u32) -> u32 {
    encounter
        .state(id(who))
        .expect("state")
        .conditions
        .iter()
        .find_map(|slot| match slot.duration() {
            Some(ConditionDuration::Rounds { remaining, .. }) => Some(remaining),
            _ => None,
        })
        .expect("rounds")
}

fn strike_at(
    encounter: &mut Encounter,
    dice: &mut ScriptedDice,
    attack: i32,
    dice_count: u8,
    bonus: i32,
    basis: AttackBasis,
    damage: DamageBasis,
) -> StrikeOutcome {
    let mut attack_profile = profile(attack, Agility::NotAgile, dice_count, 8, bonus);
    attack_profile.attack_basis = basis;
    attack_profile.damage_basis = damage;
    rolled(
        encounter
            .strike(
                id(1),
                id(2),
                &attack_profile,
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                dice,
            )
            .expect("strike"),
    )
}

/// Player Core pg. 435, Step 1: Start Your Turn.
#[test]
fn a_3_round_duration_decreases_on_the_clock_s_next_start() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 16), pc(2, 0, 20, 16)], &[15, 10]);
    give(
        &mut encounter,
        2,
        ConditionKind::Sickened,
        Some(1),
        rounds_on(3, 1),
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    assert_eq!(rounds_left(&encounter, 2), 2);
    assert_eq!(value_of(&encounter, 2, ConditionKind::Sickened), Some(1));
    assert_eq!(hit_points(&encounter, 2), 20);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core pg. 302, Durations.
#[test]
fn the_bearer_s_start_does_not_decrease_another_clock() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 16), pc(2, 0, 20, 16)], &[15, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.end_turn(&mut dice).expect("end");
    give(
        &mut encounter,
        2,
        ConditionKind::Sickened,
        Some(1),
        rounds_on(3, 1),
    );
    assert_eq!(encounter.start_turn().expect("bearer"), id(2));
    assert_eq!(rounds_left(&encounter, 2), 3);
}

/// Player Core pg. 302, Durations. Player Core pg. 435, Step 1: Start Your Turn.
#[test]
fn a_skipped_dead_slot_still_ticks_its_round_clock() {
    let mut dead = enemy(2, 0, 10, 16);
    dead.state.vital = Vital::Dead;
    dead.state.hit_points = 0;
    let (mut encounter, _dice) =
        begin(vec![pc(1, 0, 20, 16), dead, pc(3, 0, 20, 16)], &[15, 10, 5]);
    give(
        &mut encounter,
        3,
        ConditionKind::Sickened,
        Some(1),
        rounds_on(2, 2),
    );
    assert_eq!(encounter.start_turn().expect("first"), id(1));
    encounter.end_turn(&mut ScriptedDice::new([])).expect("end");
    assert_eq!(encounter.start_turn().expect("skip"), id(3));
    assert_eq!(encounter.turn_slot(id(2)).expect("slot"), TurnSlot::Normal);
    assert_eq!(rounds_left(&encounter, 3), 1);
}

/// Player Core pg. 436, Step 3: End Your Turn.
#[test]
fn an_open_turn_s_end_is_not_the_next_end() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 16), pc(2, 0, 20, 16)], &[15, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    give(
        &mut encounter,
        2,
        ConditionKind::OffGuard,
        None,
        until_end(1),
    );
    encounter.end_turn(&mut dice).expect("end");
    assert!(has(&encounter, 2, ConditionKind::OffGuard));
    assert_eq!(encounter.start_turn().expect("two"), id(2));
    encounter.end_turn(&mut dice).expect("end two");
    assert_eq!(encounter.start_turn().expect("again"), id(1));
    encounter.end_turn(&mut dice).expect("following");
    assert!(!has(&encounter, 2, ConditionKind::OffGuard));
}

/// Player Core pg. 436, Step 3: End Your Turn.
#[test]
fn a_closed_turn_s_next_end_removes_it() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 16), pc(2, 0, 20, 16)], &[15, 10]);
    assert_eq!(encounter.start_turn().expect("one"), id(1));
    encounter.end_turn(&mut dice).expect("end one");
    assert_eq!(encounter.start_turn().expect("two"), id(2));
    give(
        &mut encounter,
        2,
        ConditionKind::OffGuard,
        None,
        until_end(1),
    );
    encounter.end_turn(&mut dice).expect("end two");
    assert_eq!(encounter.start_turn().expect("one again"), id(1));
    encounter.end_turn(&mut dice).expect("next end");
    assert!(!has(&encounter, 2, ConditionKind::OffGuard));
}

/// Player Core pg. 444, Frightened. Player Core pg. 436, Step 3: End Your Turn.
#[test]
fn ending_the_turn_lowers_frightened_and_leaves_hit_points() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 12, 16)], &[10, 8]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    give(
        &mut encounter,
        1,
        ConditionKind::Frightened,
        Some(2),
        until_removed(),
    );
    give(
        &mut encounter,
        1,
        ConditionKind::Sickened,
        Some(1),
        until_removed(),
    );
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(value_of(&encounter, 1, ConditionKind::Frightened), Some(1));
    assert_eq!(value_of(&encounter, 1, ConditionKind::Sickened), Some(1));
    assert_eq!(hit_points(&encounter, 1), 12);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 436, Step 3: End Your Turn. Player Core pg. 445, Persistent Damage.
///
/// A non-persistent condition stays when the end step rewrites persistent damage.
#[test]
fn a_non_persistent_condition_survives_end_turn() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 16)], &[10, 10]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 2)
        .expect("persistent");
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    give(
        &mut encounter,
        1,
        ConditionKind::Sickened,
        Some(1),
        until_removed(),
    );
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(hit_points(&encounter, 1), 18);
    assert_eq!(value_of(&encounter, 1, ConditionKind::Sickened), Some(1));
    assert!(
        encounter
            .state(id(1))
            .expect("state")
            .conditions
            .iter()
            .any(|slot| matches!(
                slot,
                ConditionSlot::PersistentDamage {
                    damage_type: DamageType::Fire,
                    amount: 2,
                }
            ))
    );
}

/// Player Core pg. 436, Step 3: End Your Turn. Player Core pg. 445, Persistent Damage.
#[test]
fn the_persistent_hook_runs_before_frightened_decreases() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 16)], &[10, 10]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("persistent");
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    give(
        &mut encounter,
        1,
        ConditionKind::Frightened,
        Some(2),
        until_removed(),
    );
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(hit_points(&encounter, 1), 15);
    assert_eq!(value_of(&encounter, 1, ConditionKind::Frightened), Some(1));
    assert!(
        encounter
            .state(id(1))
            .expect("state")
            .conditions
            .iter()
            .any(|slot| matches!(
                slot,
                ConditionSlot::PersistentDamage {
                    damage_type: DamageType::Fire,
                    amount: 5,
                }
            ))
    );
}

/// Player Core pg. 444, Frightened.
#[test]
fn frightened_2_lowers_the_attack_total_by_2() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 16), enemy(2, 0, 20, 16)], &[15, 10, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    give(
        &mut encounter,
        1,
        ConditionKind::Frightened,
        Some(2),
        until_removed(),
    );
    let outcome = strike_at(
        &mut encounter,
        &mut dice,
        6,
        0,
        0,
        AttackBasis::StrengthMelee,
        DamageBasis::Strength,
    );
    assert_eq!(outcome.total, 14);
    assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 400, Penalties.
#[test]
fn frightened_2_and_enfeebled_2_stay_at_minus_2() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 16), enemy(2, 0, 20, 16)], &[15, 10, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    give(
        &mut encounter,
        1,
        ConditionKind::Frightened,
        Some(2),
        until_removed(),
    );
    give(
        &mut encounter,
        1,
        ConditionKind::Enfeebled,
        Some(2),
        until_removed(),
    );
    let outcome = strike_at(
        &mut encounter,
        &mut dice,
        8,
        0,
        0,
        AttackBasis::StrengthMelee,
        DamageBasis::Strength,
    );
    assert_eq!(outcome.total, 16);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
}

/// Player Core pg. 442, Clumsy.
#[test]
fn clumsy_2_does_not_lower_a_strength_melee_attack() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 16), enemy(2, 0, 20, 16)], &[15, 10, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    give(
        &mut encounter,
        1,
        ConditionKind::Clumsy,
        Some(2),
        until_removed(),
    );
    let outcome = strike_at(
        &mut encounter,
        &mut dice,
        6,
        0,
        0,
        AttackBasis::StrengthMelee,
        DamageBasis::Strength,
    );
    assert_eq!(outcome.total, 16);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
}

/// Player Core pg. 442, Clumsy.
#[test]
fn clumsy_2_lowers_a_dexterity_melee_attack() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 16), enemy(2, 0, 20, 16)], &[15, 10, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    give(
        &mut encounter,
        1,
        ConditionKind::Clumsy,
        Some(2),
        until_removed(),
    );
    let outcome = strike_at(
        &mut encounter,
        &mut dice,
        6,
        0,
        0,
        AttackBasis::DexterityMelee,
        DamageBasis::Strength,
    );
    assert_eq!(outcome.total, 14);
    assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 443, Enfeebled.
#[test]
fn enfeebled_2_does_not_lower_a_dexterity_melee_attack() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 16), enemy(2, 0, 20, 16)], &[15, 10, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    give(
        &mut encounter,
        1,
        ConditionKind::Enfeebled,
        Some(2),
        until_removed(),
    );
    let outcome = strike_at(
        &mut encounter,
        &mut dice,
        6,
        0,
        0,
        AttackBasis::DexterityMelee,
        DamageBasis::Strength,
    );
    assert_eq!(outcome.total, 16);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
}

/// Player Core pg. 443, Enfeebled.
#[test]
fn enfeebled_2_lowers_dexterity_melee_damage() {
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 16), enemy(2, 0, 20, 11)],
        &[15, 10, 10, 4],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    give(
        &mut encounter,
        1,
        ConditionKind::Enfeebled,
        Some(2),
        until_removed(),
    );
    let outcome = strike_at(
        &mut encounter,
        &mut dice,
        10,
        1,
        0,
        AttackBasis::DexterityMelee,
        DamageBasis::Strength,
    );
    assert_eq!(outcome.total, 20);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
    assert_eq!(outcome.damage, 2);
}

/// Player Core pg. 442, Clumsy.
#[test]
fn clumsy_2_lowers_a_ranged_attack() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 16), enemy(2, 0, 20, 16)], &[15, 10, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    give(
        &mut encounter,
        1,
        ConditionKind::Clumsy,
        Some(2),
        until_removed(),
    );
    let outcome = strike_at(
        &mut encounter,
        &mut dice,
        6,
        0,
        0,
        AttackBasis::Ranged,
        DamageBasis::NotStrength,
    );
    assert_eq!(outcome.total, 14);
    assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 443, Enfeebled.
#[test]
fn enfeebled_2_lowers_strength_melee_damage() {
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 16), enemy(2, 0, 20, 11)],
        &[15, 10, 10, 4],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    give(
        &mut encounter,
        1,
        ConditionKind::Enfeebled,
        Some(2),
        until_removed(),
    );
    let outcome = strike_at(
        &mut encounter,
        &mut dice,
        10,
        1,
        0,
        AttackBasis::StrengthMelee,
        DamageBasis::Strength,
    );
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
    assert_eq!(outcome.damage, 2);
}

/// Player Core pg. 443, Enfeebled.
#[test]
fn enfeebled_2_does_not_lower_ranged_damage() {
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 16), enemy(2, 0, 20, 11)],
        &[15, 10, 10, 4],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    give(
        &mut encounter,
        1,
        ConditionKind::Enfeebled,
        Some(2),
        until_removed(),
    );
    let outcome = strike_at(
        &mut encounter,
        &mut dice,
        10,
        1,
        0,
        AttackBasis::Ranged,
        DamageBasis::NotStrength,
    );
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
    assert_eq!(outcome.damage, 4);
}

/// Player Core pg. 444, Frightened.
#[test]
fn frightened_2_on_the_target_changes_the_degree() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 16), enemy(2, 0, 20, 16)], &[15, 10, 14]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    give(
        &mut encounter,
        2,
        ConditionKind::Frightened,
        Some(2),
        until_removed(),
    );
    let outcome = strike_at(
        &mut encounter,
        &mut dice,
        0,
        0,
        0,
        AttackBasis::StrengthMelee,
        DamageBasis::Strength,
    );
    assert_eq!(outcome.total, 14);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
}

/// Player Core pg. 400, Penalties.
#[test]
fn frightened_2_and_sickened_1_do_not_sum_on_armor_class() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 16), enemy(2, 0, 20, 15)], &[15, 10, 12]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    give(
        &mut encounter,
        2,
        ConditionKind::Frightened,
        Some(2),
        until_removed(),
    );
    give(
        &mut encounter,
        2,
        ConditionKind::Sickened,
        Some(1),
        until_removed(),
    );
    let outcome = strike_at(
        &mut encounter,
        &mut dice,
        0,
        0,
        0,
        AttackBasis::StrengthMelee,
        DamageBasis::Strength,
    );
    assert_eq!(outcome.total, 12);
    assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 444, Frightened. Player Core pg. 445, Off-Guard.
#[test]
fn frightened_2_and_off_guard_both_lower_armor_class() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 16), enemy(2, 0, 20, 18)], &[15, 10, 14]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    give(
        &mut encounter,
        2,
        ConditionKind::Frightened,
        Some(2),
        until_removed(),
    );
    give(
        &mut encounter,
        2,
        ConditionKind::OffGuard,
        None,
        until_removed(),
    );
    let outcome = strike_at(
        &mut encounter,
        &mut dice,
        0,
        0,
        0,
        AttackBasis::StrengthMelee,
        DamageBasis::Strength,
    );
    assert_eq!(outcome.total, 14);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
}

/// Player Core pg. 435, Step 1: Roll Initiative.
#[test]
fn frightened_2_and_sickened_1_lower_initiative_by_2() {
    let mut hero = pc(1, 5, 20, 16);
    hero.state.conditions.push(
        ConditionSlot::try_new(ConditionKind::Frightened, Some(2), until_removed())
            .expect("frightened"),
    );
    hero.state.conditions.push(
        ConditionSlot::try_new(ConditionKind::Sickened, Some(1), until_removed())
            .expect("sickened"),
    );
    let (encounter, _dice) = begin(vec![hero], &[10]);
    assert_eq!(encounter.initiative_total(id(1)).expect("initiative"), 13);
}

/// Player Core pg. 416, Delay. Player Core pg. 444, Frightened.
#[test]
fn frightened_drops_at_the_end_of_the_returned_turn() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 16), pc(2, 0, 20, 16)], &[15, 10]);
    give(
        &mut encounter,
        1,
        ConditionKind::Frightened,
        Some(2),
        until_removed(),
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter
        .delay(id(1), &mut ScriptedDice::new([]))
        .expect("delay");
    assert_eq!(value_of(&encounter, 1, ConditionKind::Frightened), Some(2));
    assert_eq!(encounter.start_turn().expect("two"), id(2));
    encounter.end_turn(&mut dice).expect("end two");
    encounter.return_from_delay(&[id(1)]).expect("return");
    encounter.end_turn(&mut dice).expect("returned end");
    assert_eq!(value_of(&encounter, 1, ConditionKind::Frightened), Some(1));
    assert_eq!(encounter.start_turn().expect("two again"), id(2));
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("ordinary"), id(1));
    encounter.end_turn(&mut dice).expect("ordinary end");
    assert!(value_of(&encounter, 1, ConditionKind::Frightened).is_none());
}

/// Player Core pg. 411, Death.
#[test]
fn death_clears_frightened() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 16), enemy(2, 0, 4, 11)], &[15, 10, 10, 4]);
    give(
        &mut encounter,
        2,
        ConditionKind::Frightened,
        Some(2),
        until_removed(),
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let _outcome = strike_at(
        &mut encounter,
        &mut dice,
        10,
        1,
        0,
        AttackBasis::StrengthMelee,
        DamageBasis::Strength,
    );
    assert_eq!(encounter.state(id(2)).expect("state").vital, Vital::Dead);
    assert!(!has(&encounter, 2, ConditionKind::Frightened));
}

/// Player Core pg. 436, Step 3: End Your Turn.
#[test]
fn a_skipped_dead_slot_does_not_decrement() {
    let mut dead = enemy(2, 0, 10, 16);
    dead.state.vital = Vital::Dead;
    dead.state.hit_points = 0;
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 16), dead, pc(3, 0, 20, 16)],
        &[15, 10, 5, 8],
    );
    give(
        &mut encounter,
        1,
        ConditionKind::Frightened,
        Some(2),
        until_removed(),
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("skip"), id(3));
    assert_eq!(value_of(&encounter, 1, ConditionKind::Frightened), Some(1));
    assert_ne!(encounter.turn_open(), Some(id(2)));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 416, Delay. Player Core pg. 444, Frightened.
#[test]
fn delay_leaves_frightened_at_2() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 16), pc(2, 0, 20, 16)], &[15, 10]);
    give(
        &mut encounter,
        1,
        ConditionKind::Frightened,
        Some(2),
        until_removed(),
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter
        .delay(id(1), &mut ScriptedDice::new([]))
        .expect("delay");
    assert_eq!(value_of(&encounter, 1, ConditionKind::Frightened), Some(2));
    assert_eq!(dice.remaining(), 0);
}

/// Player Core pg. 416, Delay. Player Core pg. 445, Off-Guard.
#[test]
fn off_guard_lasts_through_delay_until_the_returned_turn_ends() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 16), pc(2, 0, 20, 16)], &[15, 10]);
    give(
        &mut encounter,
        1,
        ConditionKind::OffGuard,
        None,
        until_end(1),
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter
        .delay(id(1), &mut ScriptedDice::new([]))
        .expect("delay");
    assert!(has(&encounter, 1, ConditionKind::OffGuard));
    assert_eq!(encounter.start_turn().expect("two"), id(2));
    encounter.end_turn(&mut dice).expect("end two");
    encounter.return_from_delay(&[id(1)]).expect("return");
    encounter.end_turn(&mut dice).expect("returned");
    assert!(!has(&encounter, 1, ConditionKind::OffGuard));
}

/// Player Core pg. 416, Delay. Player Core pg. 445, Persistent Damage.
#[test]
fn persistent_fire_2_ticks_at_delay() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 16), pc(2, 0, 20, 16)], &[15, 10, 10]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 2)
        .expect("persistent");
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.delay(id(1), &mut dice).expect("delay");
    assert_eq!(hit_points(&encounter, 1), 18);
    assert!(
        encounter
            .state(id(1))
            .expect("state")
            .conditions
            .iter()
            .any(|slot| matches!(
                slot,
                ConditionSlot::PersistentDamage {
                    damage_type: DamageType::Fire,
                    amount: 2,
                }
            ))
    );
}

/// Player Core pg. 302, Durations. Player Core pg. 416, Delay.
#[test]
fn a_missed_return_ticks_the_round_duration_on_the_next_start() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 16), pc(2, 0, 20, 16)], &[15, 10]);
    give(
        &mut encounter,
        1,
        ConditionKind::Sickened,
        Some(1),
        rounds_on(2, 1),
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter
        .delay(id(1), &mut ScriptedDice::new([]))
        .expect("delay");
    assert_eq!(encounter.start_turn().expect("two"), id(2));
    encounter.end_turn(&mut dice).expect("end two");
    assert_eq!(encounter.start_turn().expect("missed"), id(1));
    assert_eq!(encounter.round(), 2);
    assert!(!has(&encounter, 1, ConditionKind::Sickened));
}

/// Player Core pg. 445, Prone.
#[test]
fn prone_does_not_stop_a_stride() {
    let (mut encounter, _dice) = begin(vec![pc(1, 0, 20, 16)], &[10]);
    give(
        &mut encounter,
        1,
        ConditionKind::Prone,
        None,
        until_removed(),
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let places = [ReactionPlace {
        id: id(1),
        square: sq(0, 0),
        size: crate::grid::Size::Medium,
        cover: None,
    }];
    let field = ReactionField {
        places: &places,
        bounds: crate::grid::MapBounds {
            width: 64,
            height: 64,
        },
        terrain: &[],
        occupants: &[],
    };
    encounter
        .stride(
            id(1),
            &field,
            &[sq(1, 0)],
            crate::grid::DiagonalCounter::fresh(),
        )
        .expect("stride");
    assert!(has(&encounter, 1, ConditionKind::Prone));
}

/// Player Core pg. 442, Clumsy. Player Core pg. 443, Enfeebled.
#[test]
fn a_strength_melee_profile_keeps_the_old_fields() {
    let attack = StrikeProfile {
        attack_bonus: 4,
        agility: Agility::NotAgile,
        damage_dice: DieCount::new(1).expect("dice"),
        damage_faces: faces(8),
        damage_bonus: 1,
        damage_type: DamageType::Slashing,
        attack_basis: AttackBasis::StrengthMelee,
        damage_basis: DamageBasis::Strength,
    };
    assert_eq!(attack.attack_bonus, 4);
    assert_eq!(attack.agility, Agility::NotAgile);
    assert_eq!(attack.damage_dice.get(), 1);
    assert_eq!(attack.damage_faces.get(), 8);
    assert_eq!(attack.damage_bonus, 1);
    assert_eq!(attack.damage_type, DamageType::Slashing);
    assert_eq!(attack.attack_basis, AttackBasis::StrengthMelee);
    assert_eq!(attack.damage_basis, DamageBasis::Strength);
}

/// Player Core pg. 442, Appendix entry, Redundant Conditions.
#[test]
fn a_fresh_combatant_has_no_conditions() {
    let creature = creature(20, 16, 0, false, None);
    let state = new_combatant_state(id(1), &creature);
    assert!(state.conditions.is_empty());
}

/// Player Core pg. 444, Frightened.
#[test]
fn a_stored_frightened_2_can_be_read() {
    let (mut encounter, _dice) = begin(vec![pc(1, 0, 20, 16)], &[10]);
    give(
        &mut encounter,
        1,
        ConditionKind::Frightened,
        Some(2),
        until_removed(),
    );
    let slot = &encounter.state(id(1)).expect("state").conditions[0];
    assert_eq!(slot.kind(), Some(ConditionKind::Frightened));
    assert_eq!(slot.value(), Some(2));
    assert_eq!(slot.duration(), Some(ConditionDuration::UntilRemoved));
}

/// Player Core pg. 442, Appendix entry, Redundant Conditions.
#[test]
fn removing_frightened_leaves_persistent_damage() {
    let (mut encounter, _dice) = begin(vec![pc(1, 0, 20, 16)], &[10]);
    encounter
        .gain_persistent_damage(id(1), DamageType::Fire, 5)
        .expect("persistent");
    give(
        &mut encounter,
        1,
        ConditionKind::Frightened,
        Some(2),
        until_removed(),
    );
    encounter
        .remove_condition(id(1), ConditionKind::Frightened)
        .expect("remove");
    assert!(!has(&encounter, 1, ConditionKind::Frightened));
    assert!(
        encounter
            .state(id(1))
            .expect("state")
            .conditions
            .iter()
            .any(|slot| matches!(
                slot,
                ConditionSlot::PersistentDamage {
                    damage_type: DamageType::Fire,
                    amount: 5,
                }
            ))
    );
}
