use super::*;

/// Player Core, Strike.
///
/// Reach substitute: rule, a melee Strike targets a creature within reach (Player Core pg. 418,
/// Strike); why not, reach, flanking, and prone are add-positioning; substitute, the caller
/// names the target and this module stores no squares; intent, Strike is testable without the grid.
#[test]
fn the_first_strike_totals_the_d20_plus_the_attack_bonus() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let outcome = rolled(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect("strike"),
    );
    assert_eq!(outcome.map, 0);
    assert_eq!(outcome.total, 18);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
    assert_eq!(actions(&encounter, 1), 2);
    assert_eq!(encounter.turn_open(), Some(id(1)));
}

/// Player Core, Strike.
///
/// The baked Armor Class result below is replaced by wire-dying-into-turns (DEV-36).
#[test]
fn a_not_alive_target_keeps_baked_armor_class() {
    let (mut encounter, mut dice) = begin(
        vec![
            combatant(1, Side::Pc, 0, 20, 18, false),
            combatant(2, Side::Pc, 0, 9, 18, true),
        ],
        &[10, 8, 15, 8, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter
        .strike(
            id(1),
            id(2),
            &profile(10, Agility::NotAgile, 1, 8, 1),
            &[],
            ShieldBlock::Declined,
            &Detections::new(),
            &mut dice,
        )
        .expect("knockout");
    assert_ne!(vital(&encounter, 2), Vital::Alive);
    assert_ne!(vital(&encounter, 2), Vital::Dead);
    encounter.end_turn().expect("end");
    assert_eq!(encounter.start_turn().expect("empty"), id(2));
    encounter.end_turn().expect("close empty");
    assert_eq!(encounter.start_turn().expect("again"), id(1));
    let outcome = rolled(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect("second"),
    );
    assert_eq!(outcome.total, 18);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
}

/// Player Core, Strike.
#[test]
fn strike_before_the_turn_starts_rolls_nothing() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 10]);
    assert_eq!(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect_err("early"),
        EncounterError::TurnNotOpen
    );
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Strike.
#[test]
fn a_strike_off_the_active_turn_rolls_nothing() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    assert_eq!(
        encounter
            .strike(
                id(2),
                id(1),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect_err("off"),
        EncounterError::NotYourTurn
    );
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Strike.
#[test]
fn an_unknown_target_rolls_nothing() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 10]);
    encounter.start_turn().expect("start");
    assert_eq!(
        encounter
            .strike(
                id(1),
                id(99),
                &profile(0, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect_err("missing"),
        EncounterError::UnknownCombatant
    );
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Strike.
#[test]
fn striking_oneself_rolls_nothing() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 10]);
    encounter.start_turn().expect("start");
    assert_eq!(
        encounter
            .strike(
                id(1),
                id(1),
                &profile(0, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect_err("self"),
        EncounterError::SelfTarget
    );
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Strike.
#[test]
fn a_strike_on_a_dead_target_spends_nothing() {
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 18), dead(pc(2, 0, 20, 18))],
        &[14, 10, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    assert_eq!(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect_err("dead"),
        EncounterError::TargetDead
    );
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Strike.
#[test]
fn a_strike_with_no_actions_rolls_nothing() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 10]);
    encounter.start_turn().expect("start");
    encounter.spend_actions(id(1), 3).expect("spend");
    assert_eq!(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect_err("empty"),
        EncounterError::InsufficientActions
    );
    assert_eq!(dice.remaining(), 1);
    assert_eq!(encounter.turn_open(), Some(id(1)));
}

/// Player Core, Strike.
#[test]
fn armor_class_in_the_caller_list_does_not_change_the_result() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 12)], &[14, 10, 10]);
    encounter.start_turn().expect("start");
    let modifier = penalty(PenaltyType::Circumstance, -2, 1, Statistic::ArmorClass);
    let outcome = rolled(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 0, 8, 0),
                &[modifier],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect("strike"),
    );
    assert_eq!(outcome.total, 10);
    assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 418, Strike. Player Core pg. 442, Concealed. Player Core pg. 400, "Checks (Flat Checks)".
#[test]
fn an_observed_target_draws_the_attack_die_first() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[10, 8, 10, 4]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let outcome = rolled(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect("strike"),
    );
    assert_eq!(outcome.total, 18);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(actions(&encounter, 1), 2);
}

/// Player Core pg. 418, Strike. Player Core pg. 442, Concealed. Player Core pg. 400, "Checks (Flat Checks)".
#[test]
fn a_natural_5_at_dc_5_then_the_attack_roll() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Observed,
        Concealment::Concealed,
        None,
    );
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[10, 8, 5, 10, 3]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let delivery = encounter
        .strike(
            id(1),
            id(2),
            &profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
            ShieldBlock::Declined,
            &detections,
            &mut dice,
        )
        .expect("strike");
    let StrikeDelivery::Rolled(outcome) = delivery else {
        panic!("expected a rolled strike, got {delivery:?}");
    };
    assert_eq!(outcome.total, 18);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
    assert_eq!(outcome.damage, 7);
    assert_eq!(dice.remaining(), 0);
    assert_eq!(
        detections.get(id(1), id(2)),
        Relation {
            detection: Detection::Observed,
            concealment: Concealment::Concealed,
        }
    );
    assert_eq!(actions(&encounter, 1), 2);
}

/// Player Core pg. 418, Strike. Player Core pg. 444, Hidden. Player Core pg. 400, "Checks (Flat Checks)".
#[test]
fn a_natural_11_at_dc_11_then_the_attack_roll() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Hidden,
        Concealment::NotConcealed,
        Some(17),
    );
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)],
        &[10, 8, 11, 10, 3],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let delivery = encounter
        .strike(
            id(1),
            id(2),
            &profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
            ShieldBlock::Declined,
            &detections,
            &mut dice,
        )
        .expect("strike");
    let StrikeDelivery::Rolled(outcome) = delivery else {
        panic!("expected a rolled strike, got {delivery:?}");
    };
    assert_eq!(outcome.total, 18);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
    assert_eq!(outcome.damage, 7);
    assert_eq!(dice.remaining(), 0);
    assert_eq!(
        detections.get(id(1), id(2)),
        Relation {
            detection: Detection::Hidden,
            concealment: Concealment::NotConcealed,
        }
    );
}

/// Player Core pg. 418, Strike. Player Core pg. 444, Hidden. Player Core pg. 442, Concealed.
#[test]
fn hidden_and_concealed_succeed_on_one_dc_11_roll() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Hidden,
        Concealment::Concealed,
        Some(17),
    );
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)],
        &[10, 8, 11, 10, 3],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let outcome = rolled(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 1, 8, 4),
                &[],
                ShieldBlock::Declined,
                &detections,
                &mut dice,
            )
            .expect("strike"),
    );
    assert_eq!(outcome.total, 18);
    assert_eq!(outcome.damage, 7);
    assert_eq!(dice.remaining(), 0);
    assert_eq!(
        detections.get(id(1), id(2)),
        Relation {
            detection: Detection::Hidden,
            concealment: Concealment::Concealed,
        }
    );
}

/// Player Core pg. 442, Concealed. Player Core pg. 400, "Checks (Flat Checks)".
#[test]
fn the_same_lost_strike_twice_matches() {
    let run = || {
        let detections = relate(
            Detections::new(),
            1,
            2,
            Detection::Observed,
            Concealment::Concealed,
            None,
        );
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[10, 8, 4, 10]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        let delivery = encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &detections,
                &mut dice,
            )
            .expect("lost");
        assert_eq!(dice.remaining(), 1);
        assert_eq!(encounter.attack_count(id(1)).expect("count"), 1);
        delivery
    };
    let first = run();
    let second = run();
    assert_eq!(first, second);
    let lost = lost(first);
    assert_eq!(lost.natural, 4);
    assert_eq!(lost.dc, 5);
    assert_eq!(lost.degree, DegreeOfSuccess::Failure);
}

/// Player Core pg. 442, Concealed. Player Core pg. 402, "Multiple Attack Penalty".
#[test]
fn a_natural_4_at_dc_5_loses_the_strike() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Observed,
        Concealment::Concealed,
        None,
    );
    let target = participant(2, Side::Pc, creature(20, 18, 0, false, Some(plate())));
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), target], &[10, 8, 4, 10, 3]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
    let lost = lost(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 1, 8, 4),
                &[],
                ShieldBlock::Declined,
                &detections,
                &mut dice,
            )
            .expect("lost"),
    );
    assert_eq!(lost.natural, 4);
    assert_eq!(lost.dc, 5);
    assert_eq!(lost.degree, DegreeOfSuccess::Failure);
    assert_eq!(actions(&encounter, 1), 2);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 1);
    assert_eq!(dice.remaining(), 2);
    let state = encounter.state(id(2)).expect("state");
    assert_eq!(state.hit_points, 20);
    assert_eq!(state.shield_hit_points, Some(20));
    assert!(state.reaction_ready);
    assert_eq!(
        detections.get(id(1), id(2)),
        Relation {
            detection: Detection::Observed,
            concealment: Concealment::Concealed,
        }
    );
}

/// Player Core pg. 442, Concealed. Player Core pg. 400, "Checks (Flat Checks)".
#[test]
fn a_natural_1_at_dc_5_loses_the_strike_and_counts() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Observed,
        Concealment::Concealed,
        None,
    );
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[10, 8, 1, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
    let lost = lost(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &detections,
                &mut dice,
            )
            .expect("lost"),
    );
    assert_eq!(lost.natural, 1);
    assert_eq!(lost.dc, 5);
    assert_eq!(lost.degree, DegreeOfSuccess::CriticalFailure);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 1);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 444, Hidden. Player Core pg. 402, "Multiple Attack Penalty".
#[test]
fn a_natural_10_at_dc_11_makes_the_next_strike_take_minus_5() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Hidden,
        Concealment::NotConcealed,
        Some(17),
    );
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18), pc(3, 0, 20, 18)],
        &[15, 10, 5, 10, 12],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let lost = lost(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &detections,
                &mut dice,
            )
            .expect("lost"),
    );
    assert_eq!(lost.dc, 11);
    assert_eq!(lost.degree, DegreeOfSuccess::Failure);
    let second = rolled(
        encounter
            .strike(
                id(1),
                id(3),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect("second"),
    );
    assert_eq!(second.map, -5);
    assert_eq!(second.total, 15);
    assert_eq!(second.degree, DegreeOfSuccess::Failure);
    assert_eq!(actions(&encounter, 1), 1);
}

/// Player Core pg. 442, Concealed. Player Core pg. 402, "Multiple Attack Penalty".
#[test]
fn a_lost_strike_makes_the_next_agile_strike_take_minus_4() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Observed,
        Concealment::Concealed,
        None,
    );
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18), pc(3, 0, 20, 18)],
        &[15, 10, 5, 4, 14, 3],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let first = encounter
        .strike(
            id(1),
            id(2),
            &profile(8, Agility::Agile, 1, 8, 4),
            &[],
            ShieldBlock::Declined,
            &detections,
            &mut dice,
        )
        .expect("lost");
    assert!(matches!(first, StrikeDelivery::Lost(_)));
    let second = rolled(
        encounter
            .strike(
                id(1),
                id(3),
                &profile(8, Agility::Agile, 1, 8, 4),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect("second"),
    );
    assert_eq!(second.map, -4);
    assert_eq!(second.total, 18);
    assert_eq!(second.damage, 7);
    assert_ne!(second.map, -5);
}

/// Player Core pg. 442, Concealed. Player Core pg. 402, "Multiple Attack Penalty".
#[test]
fn two_lost_strikes_make_the_third_take_minus_10() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Observed,
        Concealment::Concealed,
        None,
    );
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18), pc(3, 0, 20, 18)],
        &[15, 10, 5, 4, 4, 15],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let attack = profile(8, Agility::NotAgile, 0, 8, 0);
    for _ in 0..2 {
        assert!(matches!(
            encounter
                .strike(
                    id(1),
                    id(2),
                    &attack,
                    &[],
                    ShieldBlock::Declined,
                    &detections,
                    &mut dice,
                )
                .expect("lost"),
            StrikeDelivery::Lost(_)
        ));
    }
    let third = rolled(
        encounter
            .strike(
                id(1),
                id(3),
                &attack,
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect("third"),
    );
    assert_eq!(third.map, -10);
    assert_eq!(third.total, 13);
    assert_eq!(third.degree, DegreeOfSuccess::Failure);
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(encounter.turn_open(), Some(id(1)));
}

/// Player Core pg. 444, Hidden. Player Core pg. 442, Concealed.
#[test]
fn hidden_and_concealed_lose_on_one_dc_11_roll() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Hidden,
        Concealment::Concealed,
        Some(17),
    );
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)],
        &[10, 8, 10, 20, 6],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let lost = lost(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 1, 8, 4),
                &[],
                ShieldBlock::Declined,
                &detections,
                &mut dice,
            )
            .expect("lost"),
    );
    assert_eq!(lost.natural, 10);
    assert_eq!(lost.dc, 11);
    assert_eq!(dice.remaining(), 2);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 1);
    assert_eq!(
        detections.get(id(1), id(2)),
        Relation {
            detection: Detection::Hidden,
            concealment: Concealment::Concealed,
        }
    );
}

/// Player Core pg. 434, Hidden. Player Core pg. 244, Hide. Player Core pg. 442, Concealed.
#[test]
fn a_lost_strike_still_reveals_the_hidden_striker() {
    let detections = relate(
        relate(
            relate(
                Detections::new(),
                2,
                1,
                Detection::Hidden,
                Concealment::NotConcealed,
                Some(17),
            ),
            3,
            1,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        ),
        1,
        2,
        Detection::Observed,
        Concealment::Concealed,
        None,
    );
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18), pc(3, 0, 20, 18)],
        &[15, 10, 5, 4, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let lost = lost(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &detections,
                &mut dice,
            )
            .expect("lost"),
    );
    assert_eq!(lost.natural, 4);
    assert_eq!(lost.dc, 5);
    assert_eq!(lost.degree, DegreeOfSuccess::Failure);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(detections.get(id(2), id(1)).detection, Detection::Hidden);
    let notice = note_strike(detections, id(1), id(2));
    assert_eq!(
        notice.detections.get(id(2), id(1)).detection,
        Detection::Observed
    );
    assert_eq!(
        notice.detections.get(id(3), id(1)).detection,
        Detection::Observed
    );
    assert_eq!(
        notice.detections.get(id(1), id(2)),
        Relation {
            detection: Detection::Observed,
            concealment: Concealment::Concealed,
        }
    );
}

/// Player Core pg. 138, "Reactive Strike". Player Core pg. 244, Hide. Player Core pg. 442, Concealed.
#[test]
fn a_lost_reactive_strike_still_reveals_the_striker() {
    let detections = relate(
        relate(
            relate(
                Detections::new(),
                2,
                1,
                Detection::Hidden,
                Concealment::NotConcealed,
                Some(17),
            ),
            3,
            1,
            Detection::Hidden,
            Concealment::NotConcealed,
            Some(17),
        ),
        1,
        2,
        Detection::Observed,
        Concealment::Concealed,
        None,
    );
    let (mut encounter, mut dice) = begin(
        vec![
            with_reach(pc(1, 0, 20, 18), 5),
            pc(2, 0, 20, 18),
            pc(3, 0, 20, 18),
        ],
        &[15, 10, 5, 4, 10],
    );
    let outcome = react_seen(
        &mut encounter,
        &mut dice,
        &detections,
        profile(8, Agility::NotAgile, 0, 8, 0),
    )
    .expect("lost");
    let lost = lost(outcome.strike);
    assert_eq!(lost.natural, 4);
    assert_eq!(lost.dc, 5);
    assert_eq!(lost.degree, DegreeOfSuccess::Failure);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(detections.get(id(2), id(1)).detection, Detection::Hidden);
    let notice = note_strike(detections, id(1), id(2));
    assert_eq!(
        notice.detections.get(id(2), id(1)).detection,
        Detection::Observed
    );
    assert_eq!(
        notice.detections.get(id(3), id(1)).detection,
        Detection::Observed
    );
    assert_eq!(
        notice.detections.get(id(1), id(2)).concealment,
        Concealment::Concealed
    );
}

/// Player Core pg. 447, Undetected. Player Core pg. 418, Strike.
#[test]
fn an_undetected_target_cannot_be_struck() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Undetected,
        Concealment::NotConcealed,
        Some(17),
    );
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[10, 8, 11]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    assert_eq!(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &detections,
                &mut dice,
            )
            .expect_err("undetected"),
        EncounterError::CannotTarget
    );
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(
        detections.get(id(1), id(2)).detection,
        Detection::Undetected
    );
}

/// Player Core pg. 418, Strike. Player Core pg. 447, Undetected.
#[test]
fn a_dead_undetected_target_is_target_dead() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Undetected,
        Concealment::NotConcealed,
        Some(17),
    );
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 18), dead(pc(2, 0, 20, 18))], &[10, 8, 11]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let error = encounter
        .strike(
            id(1),
            id(2),
            &profile(8, Agility::NotAgile, 0, 8, 0),
            &[],
            ShieldBlock::Declined,
            &detections,
            &mut dice,
        )
        .expect_err("dead");
    assert_eq!(error, EncounterError::TargetDead);
    assert_ne!(error, EncounterError::CannotTarget);
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 418, Strike. Player Core pg. 442, Concealed.
#[test]
fn no_actions_beats_a_concealed_flat_check() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Observed,
        Concealment::Concealed,
        None,
    );
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[10, 8, 4]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.spend_actions(id(1), 3).expect("spend");
    assert_eq!(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &detections,
                &mut dice,
            )
            .expect_err("no actions"),
        EncounterError::InsufficientActions
    );
    assert_eq!(dice.remaining(), 1);
    assert_eq!(encounter.turn_open(), Some(id(1)));
}

/// Player Core pg. 402, "Multiple Attack Penalty". Player Core pg. 442, Concealed.
#[test]
fn an_attack_count_at_the_maximum_draws_no_flat_check() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Observed,
        Concealment::Concealed,
        None,
    );
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[10, 8, 4]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.set_attack_count_for_test(id(1), 255);
    assert_eq!(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &detections,
                &mut dice,
            )
            .expect_err("count"),
        EncounterError::AttackCountOverflow
    );
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 255);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Shield Block". Player Core pg. 442, Concealed.
#[test]
fn a_shield_that_is_not_raised_draws_no_flat_check() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Observed,
        Concealment::Concealed,
        None,
    );
    let target = participant(2, Side::Pc, creature(20, 18, 0, false, Some(plate())));
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), target], &[10, 8, 4]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    assert_eq!(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Requested,
                &detections,
                &mut dice,
            )
            .expect_err("not raised"),
        EncounterError::NotRaised
    );
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Shield Block". Player Core pg. 442, Concealed.
#[test]
fn a_lost_strike_against_a_raised_shield_leaves_it_untouched() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Observed,
        Concealment::Concealed,
        None,
    );
    let (mut encounter, mut dice) = raised_defender(&[8, 14, 4, 10, 3]);
    let lost = lost(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 1, 8, 4),
                &[],
                ShieldBlock::Requested,
                &detections,
                &mut dice,
            )
            .expect("lost"),
    );
    assert_eq!(lost.natural, 4);
    assert_eq!(lost.dc, 5);
    assert_eq!(lost.degree, DegreeOfSuccess::Failure);
    assert_eq!(dice.remaining(), 2);
    let state = encounter.state(id(2)).expect("state");
    assert!(state.reaction_ready);
    assert!(state.shield_raised);
    assert_eq!(state.shield_hit_points, Some(20));
}

/// Player Core, "Shield Block". Player Core pg. 447, Undetected.
#[test]
fn cannot_target_against_a_raised_shield_spends_nothing() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Undetected,
        Concealment::NotConcealed,
        Some(17),
    );
    let (mut encounter, mut dice) = raised_defender(&[8, 14, 11]);
    assert_eq!(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Requested,
                &detections,
                &mut dice,
            )
            .expect_err("undetected"),
        EncounterError::CannotTarget
    );
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
    assert_eq!(dice.remaining(), 1);
    let state = encounter.state(id(2)).expect("state");
    assert!(state.reaction_ready);
    assert!(state.shield_raised);
    assert_eq!(state.shield_hit_points, Some(20));
}

/// Player Core, Penalties. Player Core pg. 442, Concealed.
#[test]
fn a_flat_check_success_can_still_overflow_the_attack() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Observed,
        Concealment::Concealed,
        None,
    );
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 11), pc(2, 0, 20, 11)],
        &[14, 10, 5, 11, 8],
    );
    encounter.start_turn().expect("start");
    let modifiers = [
        penalty(PenaltyType::Untyped, i32::MIN, 1, Statistic::Attack),
        penalty(PenaltyType::Untyped, i32::MIN, 2, Statistic::Attack),
    ];
    assert_eq!(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 0, 8, 0),
                &modifiers,
                ShieldBlock::Declined,
                &detections,
                &mut dice,
            )
            .expect_err("overflow"),
        EncounterError::AttackOverflow(ModifierError::Overflow)
    );
    assert_eq!(dice.remaining(), 1);
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(
        encounter
            .attack_trait_penalty(id(1), Agility::NotAgile)
            .expect("penalty"),
        0
    );
}

/// Player Core, Penalties.
#[test]
fn the_second_strike_includes_map_and_a_caller_circumstance_penalty() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 16), pc(2, 0, 20, 16)], &[14, 10, 2, 12]);
    encounter.start_turn().expect("start");
    encounter
        .strike(
            id(1),
            id(2),
            &profile(9, Agility::NotAgile, 0, 8, 0),
            &[],
            ShieldBlock::Declined,
            &Detections::new(),
            &mut dice,
        )
        .expect("first");
    let modifier = penalty(PenaltyType::Circumstance, -2, 1, Statistic::Attack);
    let outcome = rolled(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(9, Agility::NotAgile, 0, 8, 0),
                &[modifier],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect("second"),
    );
    assert_eq!(outcome.map, -5);
    assert_eq!(outcome.total, 14);
    assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
}

/// Player Core, Penalties.
#[test]
fn two_caller_circumstance_bonuses_keep_the_higher_on_a_strike() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 10), pc(2, 0, 20, 10)], &[14, 10, 10]);
    encounter.start_turn().expect("start");
    let modifiers = [
        bonus(BonusType::Circumstance, 4, 1, Statistic::Attack),
        bonus(BonusType::Circumstance, 1, 2, Statistic::Attack),
    ];
    let outcome = rolled(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 0, 8, 0),
                &modifiers,
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect("strike"),
    );
    assert_eq!(outcome.total, 14);
    assert_ne!(outcome.total, 15);
}

/// Player Core, Penalties. Player Core, "Step 4: Degree of Success".
#[test]
fn a_circumstance_bonus_and_penalty_on_the_attack_both_apply() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 10), pc(2, 0, 20, 10)], &[14, 10, 10]);
    encounter.start_turn().expect("start");
    let modifiers = [
        bonus(BonusType::Circumstance, 2, 1, Statistic::Attack),
        penalty(PenaltyType::Circumstance, -2, 2, Statistic::Attack),
    ];
    let outcome = rolled(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 0, 8, 0),
                &modifiers,
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect("strike"),
    );
    assert_eq!(outcome.total, 10);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
    assert_ne!(outcome.total, 12);
    assert_ne!(outcome.total, 8);
}

/// Player Core, Penalties.
#[test]
fn a_damage_penalty_does_not_change_the_attack_total() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 12]);
    encounter.start_turn().expect("start");
    let modifiers = [
        penalty(PenaltyType::Circumstance, -2, 1, Statistic::Attack),
        penalty(PenaltyType::Untyped, -3, 2, Statistic::Damage),
    ];
    let outcome = rolled(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &modifiers,
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect("strike"),
    );
    assert_eq!(outcome.total, 18);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
}

/// Player Core, Penalties. Player Core, "Step 4: Degree of Success".
#[test]
fn a_caller_untyped_penalty_stacks_with_map() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 11), pc(2, 0, 20, 11)], &[14, 10, 10, 12]);
    encounter.start_turn().expect("start");
    encounter
        .strike(
            id(1),
            id(2),
            &profile(0, Agility::NotAgile, 0, 8, 0),
            &[],
            ShieldBlock::Declined,
            &Detections::new(),
            &mut dice,
        )
        .expect("first");
    let modifier = penalty(PenaltyType::Untyped, -5, 1, Statistic::Attack);
    let outcome = rolled(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 0, 8, 0),
                &[modifier],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect("second"),
    );
    assert_eq!(outcome.map, -5);
    assert_eq!(outcome.total, 2);
    assert_ne!(outcome.total, 7);
    assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
}

/// Player Core, "Multiple Attack Penalty".
#[test]
fn the_second_agile_strike_takes_minus_4() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 2, 14]);
    encounter.start_turn().expect("start");
    encounter
        .strike(
            id(1),
            id(2),
            &profile(8, Agility::Agile, 0, 8, 0),
            &[],
            ShieldBlock::Declined,
            &Detections::new(),
            &mut dice,
        )
        .expect("first");
    let outcome = rolled(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::Agile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect("second"),
    );
    assert_eq!(outcome.map, -4);
    assert_eq!(outcome.total, 18);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
}

/// Player Core, "Multiple Attack Penalty".
#[test]
fn the_third_non_agile_strike_takes_minus_10() {
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)],
        &[14, 10, 2, 2, 15],
    );
    encounter.start_turn().expect("start");
    let weapon = profile(8, Agility::NotAgile, 0, 8, 0);
    encounter
        .strike(
            id(1),
            id(2),
            &weapon,
            &[],
            ShieldBlock::Declined,
            &Detections::new(),
            &mut dice,
        )
        .expect("first");
    encounter
        .strike(
            id(1),
            id(2),
            &weapon,
            &[],
            ShieldBlock::Declined,
            &Detections::new(),
            &mut dice,
        )
        .expect("second");
    let outcome = rolled(
        encounter
            .strike(
                id(1),
                id(2),
                &weapon,
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect("third"),
    );
    assert_eq!(outcome.map, -10);
    assert_eq!(outcome.total, 13);
    assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(encounter.turn_open(), Some(id(1)));
}

/// Player Core, "Multiple Attack Penalty".
#[test]
fn a_rejected_strike_does_not_count() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[12, 8, 10]);
    assert_eq!(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect_err("early"),
        EncounterError::TurnNotOpen
    );
    assert_eq!(dice.remaining(), 1);
    encounter.start_turn().expect("start");
    let outcome = rolled(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect("strike"),
    );
    assert_eq!(outcome.map, 0);
    assert_eq!(outcome.total, 18);
}

/// Player Core, "Multiple Attack Penalty".
#[test]
fn an_attack_count_at_the_u8_maximum_does_not_increment() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 10]);
    encounter.start_turn().expect("start");
    encounter.set_attack_count_for_test(id(1), 255);
    assert_eq!(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect_err("count"),
        EncounterError::AttackCountOverflow
    );
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 255);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Multiple Attack Penalty". Player Core pg. 417, Ready.
#[test]
fn a_reaction_after_two_strikes_keeps_penalty_0() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 2, 2, 8]);
    encounter.start_turn().expect("start");
    let weapon = profile(8, Agility::NotAgile, 0, 8, 0);
    encounter
        .strike(
            id(1),
            id(2),
            &weapon,
            &[],
            ShieldBlock::Declined,
            &Detections::new(),
            &mut dice,
        )
        .expect("first");
    encounter
        .strike(
            id(1),
            id(2),
            &weapon,
            &[],
            ShieldBlock::Declined,
            &Detections::new(),
            &mut dice,
        )
        .expect("second");
    encounter.end_turn().expect("end");
    assert_eq!(
        encounter
            .attack_trait_penalty(id(1), Agility::NotAgile)
            .expect("off turn"),
        0
    );
    assert_eq!(dice.remaining(), 1);
    encounter.start_turn().expect("other");
    encounter.end_turn().expect("end other");
    assert_eq!(encounter.start_turn().expect("next"), id(1));
    assert_eq!(
        encounter
            .attack_trait_penalty(id(1), Agility::NotAgile)
            .expect("next turn"),
        0
    );
}

/// Player Core, Strike.
#[test]
fn a_success_deals_the_damage_roll_once() {
    let (outcome, remaining) = strike_damage(8, 18, 1, 8, 4, &[10, 3], &[]);
    assert_eq!(outcome.damage, 7);
    assert_eq!(remaining, 0);
}

/// Player Core, Strike.
#[test]
fn a_critical_success_doubles_one_damage_roll() {
    let (outcome, remaining) = strike_damage(10, 15, 1, 8, 4, &[15, 3, 6], &[]);
    assert_eq!(outcome.damage, 14);
    assert_eq!(remaining, 1);
}

/// Player Core, Strike.
#[test]
fn a_failure_rolls_no_damage_die() {
    let (outcome, remaining) = strike_damage(0, 18, 1, 8, 0, &[9, 8], &[]);
    assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
    assert_eq!(outcome.damage, 0);
    assert_eq!(remaining, 1);
}

/// Player Core, Strike.
#[test]
fn a_critical_failure_rolls_no_damage_die() {
    let (outcome, remaining) = strike_damage(0, 18, 1, 8, 0, &[5, 8], &[]);
    assert_eq!(outcome.degree, DegreeOfSuccess::CriticalFailure);
    assert_eq!(outcome.damage, 0);
    assert_eq!(remaining, 1);
}

/// Player Core, Strike. Player Core, "Natural 1 and Natural 20".
#[test]
fn a_natural_20_that_becomes_a_success_deals_damage_once() {
    let (outcome, _) = strike_damage(0, 25, 1, 8, 4, &[20, 3], &[]);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
    assert_eq!(outcome.damage, 7);
}

/// Player Core, Strike. Player Core, "Natural 1 and Natural 20".
#[test]
fn a_natural_20_that_stays_a_failure_deals_no_damage() {
    let (outcome, remaining) = strike_damage(0, 30, 1, 8, 0, &[20, 8], &[]);
    assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
    assert_eq!(outcome.damage, 0);
    assert_eq!(remaining, 1);
}

/// Player Core, Strike. Player Core, "Natural 1 and Natural 20".
#[test]
fn a_natural_1_that_becomes_a_failure_deals_no_damage() {
    let (outcome, remaining) = strike_damage(17, 15, 1, 8, 0, &[1, 8], &[]);
    assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
    assert_eq!(outcome.damage, 0);
    assert_eq!(remaining, 1);
}

/// Player Core, Strike. Player Core, "Natural 1 and Natural 20".
#[test]
fn a_natural_1_that_becomes_a_success_deals_damage_once() {
    let (outcome, _) = strike_damage(24, 15, 1, 8, 0, &[1, 4], &[]);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
    assert_eq!(outcome.damage, 4);
}

/// Player Core, Strike.
///
/// A Failure spends the action and deals no damage. `HpError::NotAHit` stays off this path.
#[test]
fn a_failure_short_circuits_with_no_damage() {
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 18), with_temp(pc(2, 0, 20, 18), 5)],
        &[14, 10, 9, 8],
    );
    encounter.start_turn().expect("start");
    let result = encounter.strike(
        id(1),
        id(2),
        &profile(0, Agility::NotAgile, 1, 8, 4),
        &[],
        ShieldBlock::Declined,
        &Detections::new(),
        &mut dice,
    );
    assert_ne!(result, Err(EncounterError::HitPoints(HpError::NotAHit)));
    let outcome = rolled(result.expect("short-circuit"));
    assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
    assert_eq!(outcome.damage, 0);
    assert_eq!(actions(&encounter, 1), 2);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(hit_points(&encounter, 2), 20);
    assert_eq!(temp_hp(&encounter, 2), 5);
    assert_eq!(wounded(&encounter, 2), 0);
    assert_eq!(vital(&encounter, 2), Vital::Alive);
}

/// Player Core, Strike.
///
/// A CriticalFailure spends the action and deals no damage. `HpError::NotAHit` stays off this path.
#[test]
fn a_critical_failure_short_circuits_with_no_damage() {
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 18), with_temp(pc(2, 0, 20, 18), 5)],
        &[14, 10, 5, 8],
    );
    encounter.start_turn().expect("start");
    let result = encounter.strike(
        id(1),
        id(2),
        &profile(0, Agility::NotAgile, 1, 8, 4),
        &[],
        ShieldBlock::Declined,
        &Detections::new(),
        &mut dice,
    );
    assert_ne!(result, Err(EncounterError::HitPoints(HpError::NotAHit)));
    let outcome = rolled(result.expect("short-circuit"));
    assert_eq!(outcome.degree, DegreeOfSuccess::CriticalFailure);
    assert_eq!(outcome.damage, 0);
    assert_eq!(actions(&encounter, 1), 2);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(hit_points(&encounter, 2), 20);
    assert_eq!(temp_hp(&encounter, 2), 5);
    assert_eq!(wounded(&encounter, 2), 0);
    assert_eq!(vital(&encounter, 2), Vital::Alive);
}

/// Player Core, "Damage Rolls". Ruling R-07. DEV-14.
#[test]
fn penalties_reduce_a_hit_to_1_damage() {
    let modifier = penalty(PenaltyType::Untyped, -6, 1, Statistic::Damage);
    let (outcome, _) = strike_damage(0, 11, 1, 8, 0, &[11, 4], &[modifier]);
    assert_eq!(outcome.damage, 1);
}

/// Player Core, "Damage Rolls". Player Core, "Doubling and Halving". Ruling R-07. DEV-14.
#[test]
fn the_same_penalties_on_a_crit_deal_2() {
    let modifier = penalty(PenaltyType::Untyped, -6, 1, Statistic::Damage);
    let (outcome, _) = strike_damage(10, 11, 1, 8, 0, &[11, 4], &[modifier]);
    assert_eq!(outcome.damage, 2);
}

/// Player Core, "Damage Rolls". Player Core, "Doubling and Halving". Ruling R-07. DEV-14.
#[test]
fn a_pre_penalty_2_with_a_minus_2_penalty_deals_1_or_2_on_a_crit() {
    let modifier = penalty(PenaltyType::Untyped, -2, 1, Statistic::Damage);
    let (hit, _) = strike_damage(0, 11, 1, 6, 0, &[11, 2], &[modifier]);
    let (crit, _) = strike_damage(10, 11, 1, 6, 0, &[11, 2], &[modifier]);
    assert_eq!(hit.damage, 1);
    assert_eq!(crit.damage, 2);
}

/// Player Core, "Damage Rolls". Player Core, "Doubling and Halving". Ruling R-07. DEV-14.
#[test]
fn a_clamped_total_stays_0_on_a_hit_and_on_a_crit() {
    let (hit, _) = strike_damage(0, 11, 1, 4, -5, &[11, 1], &[]);
    let (crit, _) = strike_damage(10, 11, 1, 4, -5, &[11, 1], &[]);
    assert_eq!(hit.damage, 0);
    assert_eq!(crit.damage, 0);
}

/// Player Core, "Damage Rolls". Player Core, "Doubling and Halving". Ruling R-07. DEV-14.
#[test]
fn a_negative_modifier_plus_a_penalty_stays_0_on_a_crit() {
    let modifier = penalty(PenaltyType::Untyped, -1, 1, Statistic::Damage);
    let (outcome, _) = strike_damage(10, 11, 1, 4, -5, &[11, 1], &[modifier]);
    assert_eq!(outcome.damage, 0);
}

/// Player Core, "Damage Rolls". Player Core, "Doubling and Halving". Ruling R-07. DEV-14.
#[test]
fn two_damage_dice_are_doubled_once() {
    let (outcome, remaining) = strike_damage(10, 11, 2, 6, 1, &[11, 3, 4], &[]);
    assert_eq!(outcome.damage, 16);
    assert_eq!(remaining, 0);
}

/// Player Core, "Damage Rolls". Ruling R-07. DEV-14.
#[test]
fn zero_damage_dice_deal_the_bonus() {
    let (outcome, remaining) = strike_damage(0, 11, 0, 8, 4, &[11, 8], &[]);
    assert_eq!(outcome.damage, 4);
    assert_eq!(remaining, 1);
}

/// Player Core, "Damage Rolls". Ruling R-07. DEV-14.
#[test]
fn two_circumstance_bonuses_to_damage_keep_the_higher() {
    let modifiers = [
        bonus(BonusType::Circumstance, 4, 1, Statistic::Damage),
        bonus(BonusType::Circumstance, 1, 2, Statistic::Damage),
    ];
    let (outcome, _) = strike_damage(0, 11, 1, 4, 0, &[11, 1], &modifiers);
    assert_eq!(outcome.damage, 5);
}

/// Player Core, "Getting Knocked Out".
#[test]
fn a_strike_does_not_change_the_shield() {
    let shield = ShieldTemplate {
        hardness: 0,
        hit_points: 20,
        broken_threshold: 0,
        // An AC bonus of 0 is InvalidShield. The strike still leaves these Hit Points.
        shield_ac_bonus: 1,
    };
    let target = participant(2, Side::Pc, creature(20, 11, 0, false, Some(shield)));
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 11), target], &[14, 10, 11, 4]);
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
    let state = encounter.state(id(2)).expect("state");
    assert_eq!(state.shield_hit_points, Some(20));
    assert!(!state.shield_raised);
}

/// Player Core, "Damage Rolls".
#[test]
fn an_overflowing_attack_total_leaves_the_turn() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 11), pc(2, 0, 9, 11)], &[14, 10, 11, 8]);
    encounter.start_turn().expect("start");
    let modifiers = [
        penalty(PenaltyType::Untyped, i32::MIN, 1, Statistic::Attack),
        penalty(PenaltyType::Untyped, i32::MIN, 2, Statistic::Attack),
    ];
    assert_eq!(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 0, 8, 0),
                &modifiers,
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect_err("overflow"),
        EncounterError::AttackOverflow(ModifierError::Overflow)
    );
    assert_eq!(dice.remaining(), 1);
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(
        encounter
            .attack_trait_penalty(id(1), Agility::NotAgile)
            .expect("penalty"),
        0
    );
}

/// Player Core, "Damage Rolls".
#[test]
fn an_overflowing_damage_total_leaves_the_turn() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 11), pc(2, 0, 9, 11)], &[14, 10, 11, 1]);
    encounter.start_turn().expect("start");
    assert_eq!(temp_hp(&encounter, 2), 0);
    assert_eq!(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 1, 4, 2_147_483_647),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect_err("overflow"),
        EncounterError::DamageOverflow(ModifierError::Overflow)
    );
    assert_eq!(dice.remaining(), 0);
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(hit_points(&encounter, 2), 9);
    assert_eq!(
        encounter
            .attack_trait_penalty(id(1), Agility::NotAgile)
            .expect("penalty"),
        0
    );
}

/// Player Core, "Step 1: Start Your Turn".
#[test]
fn a_copied_state_does_not_change_the_encounter() {
    let (encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
    assert_eq!(actions(&encounter, 1), 0);
    let mut copied = encounter.state(id(1)).expect("state").clone();
    copied.actions_remaining = 9;
    assert_eq!(actions(&encounter, 1), 0);
}
