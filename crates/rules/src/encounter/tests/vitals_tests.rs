use super::super::delay_ready::ready_from_encounter;
use super::super::reactions::{as_damage_overflow, from_spend};
use super::*;
use crate::ready::{ReadyEncounter, ReadyError};

/// Player Core, "Temporary Hit Points". Ruling R-06.
#[test]
fn damage_removes_temporary_hit_points_then_hit_points() {
    let mut target = with_temp(pc(2, 0, 20, 11), 5);
    target.state.temp_rounds = 3;
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 11), target], &[14, 10, 11, 3]);
    encounter.start_turn().expect("start");
    encounter
        .strike(
            id(1),
            id(2),
            &profile(0, Agility::NotAgile, 1, 8, 4),
            &[],
            ShieldBlock::Declined,
            &Detections::new(),
            &mut dice,
        )
        .expect("strike");
    assert_eq!(temp_hp(&encounter, 2), 0);
    assert_eq!(encounter.state(id(2)).expect("state").temp_rounds, 0);
    assert_eq!(hit_points(&encounter, 2), 18);
}

/// Player Core, "Temporary Hit Points". Ruling R-06.
#[test]
fn damage_within_temporary_hit_points_leaves_hit_points() {
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 11), with_temp(pc(2, 0, 20, 11), 5)],
        &[14, 10, 11, 3],
    );
    encounter.start_turn().expect("start");
    encounter
        .strike(
            id(1),
            id(2),
            &profile(0, Agility::NotAgile, 1, 4, 0),
            &[],
            ShieldBlock::Declined,
            &Detections::new(),
            &mut dice,
        )
        .expect("strike");
    assert_eq!(temp_hp(&encounter, 2), 2);
    assert_eq!(hit_points(&encounter, 2), 20);
}

/// Player Core, "Temporary Hit Points". Ruling R-06.
#[test]
fn zero_damage_leaves_temporary_hit_points() {
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 11), with_temp(pc(2, 0, 20, 11), 5)],
        &[14, 10, 11, 1],
    );
    let before = hit_points(&encounter, 2);
    encounter.start_turn().expect("start");
    encounter
        .strike(
            id(1),
            id(2),
            &profile(0, Agility::NotAgile, 1, 4, -5),
            &[],
            ShieldBlock::Declined,
            &Detections::new(),
            &mut dice,
        )
        .expect("strike");
    assert_eq!(temp_hp(&encounter, 2), 5);
    assert_eq!(hit_points(&encounter, 2), before);
}

/// Player Core, "Getting Knocked Out".
#[test]
fn without_dying_rules_0_hit_points_is_death() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 11), pc(2, 0, 9, 11)], &[14, 10, 11, 8]);
    encounter.start_turn().expect("start");
    encounter
        .strike(
            id(1),
            id(2),
            &profile(0, Agility::NotAgile, 1, 8, 1),
            &[],
            ShieldBlock::Declined,
            &Detections::new(),
            &mut dice,
        )
        .expect("strike");
    assert_eq!(hit_points(&encounter, 2), 0);
    assert_eq!(vital(&encounter, 2), Vital::Dead);
    assert_eq!(wounded(&encounter, 2), 0);
    assert_eq!(encounter.creature(id(2)).expect("creature").hit_points, 9);
}

/// Player Core, "Damage Rolls".
///
/// A negative amount is not damage and is not an overflow.
#[test]
fn a_negative_amount_is_not_overflow() {
    let mut participant = pc(1, 0, 20, 18);
    let creature = participant.creature.clone();
    let result = apply_resolved_blow(
        &mut participant.state,
        &creature,
        -1,
        DegreeOfSuccess::Success,
    );
    assert_eq!(result, Err(HpError::NegativeAmount));
    assert_ne!(result, Err(HpError::Overflow));
    assert_eq!(participant.state.hit_points, 20);
    assert_eq!(participant.state.temp_hit_points, 0);
    assert_eq!(participant.state.wounded, 0);
    assert_eq!(participant.state.vital, Vital::Alive);
}

/// Player Core pg. 410, "Getting Knocked Out".
#[test]
fn with_dying_rules_a_strike_to_0_sets_dying1_and_a_pending_report() {
    let (mut encounter, mut dice) = begin(
        vec![
            combatant(1, Side::Pc, 0, 20, 11, false),
            combatant(2, Side::Pc, 0, 9, 11, true),
        ],
        &[14, 10, 11, 8],
    );
    let order = encounter.order().to_vec();
    encounter.start_turn().expect("start");
    encounter
        .strike(
            id(1),
            id(2),
            &profile(0, Agility::NotAgile, 1, 8, 1),
            &[],
            ShieldBlock::Declined,
            &Detections::new(),
            &mut dice,
        )
        .expect("strike");
    assert_eq!(hit_points(&encounter, 2), 0);
    assert_eq!(
        vital(&encounter, 2),
        Vital::Dying(DyingValue::new(1).expect("dying 1"))
    );
    assert_eq!(wounded(&encounter, 2), 0);
    assert_eq!(encounter.order(), order.as_slice());
    assert_eq!(
        take_report(&mut encounter, 2),
        Some(InitiativeMove::Pending)
    );

    let (mut crit_encounter, mut crit_dice) = begin(
        vec![
            combatant(1, Side::Pc, 0, 20, 11, false),
            combatant(2, Side::Pc, 0, 9, 11, true),
        ],
        &[14, 10, 20, 4, 20, 1],
    );
    crit_encounter.start_turn().expect("crit start");
    let crit = rolled(
        crit_encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 1, 8, 1),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut crit_dice,
            )
            .expect("crit strike"),
    );
    assert_eq!(crit.degree, DegreeOfSuccess::CriticalSuccess);
    assert_eq!(
        vital(&crit_encounter, 2),
        Vital::Dying(DyingValue::new(2).expect("dying 2"))
    );
    assert_eq!(crit_dice.remaining(), 2);

    let (mut inconsistent, mut inconsistent_dice) = begin(
        vec![
            combatant(1, Side::Pc, 0, 20, 11, false),
            combatant(2, Side::Pc, 0, 9, 11, true),
        ],
        &[14, 10, 11, 8],
    );
    {
        let target = inconsistent
            .members
            .iter_mut()
            .find(|member| member.participant.id == id(2))
            .expect("target");
        target.participant.state.vital = Vital::Dying(DyingValue::new(1).expect("dying 1"));
        target.participant.state.hit_points = 4;
    }
    inconsistent.start_turn().expect("inconsistent start");
    let attacker_actions = actions(&inconsistent, 1);
    let target_actions = actions(&inconsistent, 2);
    assert_eq!(
        inconsistent.strike(
            id(1),
            id(2),
            &profile(0, Agility::NotAgile, 1, 8, 1),
            &[],
            ShieldBlock::Declined,
            &Detections::new(),
            &mut inconsistent_dice,
        ),
        Err(EncounterError::HitPoints(HpError::InconsistentVital))
    );
    assert_eq!(actions(&inconsistent, 1), attacker_actions);
    assert_eq!(actions(&inconsistent, 2), target_actions);
    assert_eq!(
        vital(&inconsistent, 2),
        Vital::Dying(DyingValue::new(1).expect("dying 1"))
    );
    assert_eq!(hit_points(&inconsistent, 2), 4);
}

/// Player Core pg. 412, Massive Damage.
#[test]
fn a_massive_strike_leaves_no_dying_value() {
    let (mut encounter, mut dice) = begin(
        vec![
            combatant(1, Side::Pc, 0, 20, 11, false),
            combatant(2, Side::Pc, 0, 10, 11, true),
        ],
        &[14, 10, 11, 8],
    );
    encounter.start_turn().expect("start");
    encounter
        .strike(
            id(1),
            id(2),
            &profile(0, Agility::NotAgile, 1, 8, 12),
            &[],
            ShieldBlock::Declined,
            &Detections::new(),
            &mut dice,
        )
        .expect("strike");
    assert_eq!(vital(&encounter, 2), Vital::Dead);
    assert!(!matches!(vital(&encounter, 2), Vital::Dying(_)));
    assert_eq!(take_report(&mut encounter, 2), None);
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn no_saving_throws_keeps_its_own_arms() {
    let error = EncounterError::NoSavingThrows;
    assert_eq!(
        ready_from_encounter(error),
        ReadyError::Encounter(ReadyEncounter::NoSavingThrows)
    );
    assert_eq!(from_spend(error), ReactiveError::NoReaction);
    assert_eq!(
        as_damage_overflow(error),
        ReactiveError::DamageOverflow(ModifierError::Overflow)
    );
}

/// Player Core pg. 404, Saving Throws.
#[test]
fn save_overflow_keeps_its_own_arms() {
    let error = EncounterError::SaveOverflow(ArithmeticOverflow::SumDoesNotFit);
    assert_eq!(
        ready_from_encounter(error),
        ReadyError::Encounter(ReadyEncounter::SaveOverflow(
            ArithmeticOverflow::SumDoesNotFit
        ))
    );
    assert_eq!(from_spend(error), ReactiveError::NoReaction);
    assert_eq!(
        as_damage_overflow(error),
        ReactiveError::DamageOverflow(ModifierError::Overflow)
    );
}
