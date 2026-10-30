use super::super::delay_ready::ready_from_encounter;
use super::*;
use crate::dice::Dice;
use crate::ready::{ReadyEncounter, ReadyError};
use crate::shield::raise_shield as raise_combatant_shield;

/// Player Core, "Raise a Shield". Player Core, "Step 2: Act".
///
/// QA probe p11. `Encounter::raise_shield` spends the action.
#[test]
fn a_raise_through_the_encounter_leaves_2_actions() {
    let shield = ShieldTemplate {
        hardness: 5,
        hit_points: 20,
        broken_threshold: 10,
        shield_ac_bonus: 2,
    };
    let (mut encounter, _) = begin(
        vec![participant(
            1,
            Side::Pc,
            creature(20, 11, 0, false, Some(shield)),
        )],
        &[10],
    );
    encounter.start_turn().expect("start");
    assert_eq!(actions(&encounter, 1), 3);
    encounter.raise_shield(id(1)).expect("raise");
    let state = encounter.state(id(1)).expect("state");
    assert_eq!(state.actions_remaining, 2);
    assert!(state.shield_raised);
}

/// Player Core, "Raise a Shield". Player Core, "Step 2: Act".
#[test]
fn a_second_raise_only_spends_an_action() {
    let shield = ShieldTemplate {
        hardness: 5,
        hit_points: 20,
        broken_threshold: 10,
        shield_ac_bonus: 2,
    };
    let (mut encounter, _) = begin(
        vec![participant(
            1,
            Side::Pc,
            creature(20, 11, 0, false, Some(shield)),
        )],
        &[10],
    );
    encounter.start_turn().expect("start");
    {
        let member = encounter
            .members
            .iter_mut()
            .find(|member| member.participant.id == id(1))
            .expect("member");
        raise_combatant_shield(&mut member.participant.state, &member.participant.creature)
            .expect("already raised");
    }
    let before = encounter.state(id(1)).expect("state");
    assert!(before.shield_raised);
    assert_eq!(before.actions_remaining, 3);
    encounter.raise_shield(id(1)).expect("raise");
    let state = encounter.state(id(1)).expect("state");
    let defender = encounter.creature(id(1)).expect("creature");
    assert!(state.shield_raised);
    assert_eq!(state.actions_remaining, 2);
    let modifier = crate::shield::shield_ac_modifier(state, defender).expect("bonus");
    assert_eq!(modifier.value(), 2);
    let total = crate::positioning::armor_class(
        None,
        None,
        crate::positioning::OffGuard::No,
        crate::positioning::AttackKind::Melee,
        &[modifier],
    )
    .expect("armor class");
    assert_eq!(total.get(Statistic::ArmorClass), 2);
    assert_ne!(total.get(Statistic::ArmorClass), 4);
}

/// Player Core, "Raise a Shield". Player Core, "Step 2: Act".
#[test]
fn another_combatant_cannot_raise_on_this_turn() {
    let shield = ShieldTemplate {
        hardness: 5,
        hit_points: 20,
        broken_threshold: 10,
        shield_ac_bonus: 2,
    };
    let (mut encounter, dice) = begin(
        vec![
            pc(1, 0, 20, 11),
            participant(2, Side::Pc, creature(20, 11, 0, false, Some(shield))),
        ],
        &[10, 8],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    {
        let member = encounter
            .members
            .iter_mut()
            .find(|member| member.participant.id == id(2))
            .expect("member");
        member.participant.state.actions_remaining = 3;
        member.participant.state.shield_raised = false;
    }
    let error = encounter.raise_shield(id(2)).expect_err("other turn");
    assert_eq!(error, EncounterError::NotYourTurn);
    let state = encounter.state(id(2)).expect("state");
    assert_eq!(state.actions_remaining, 3);
    assert!(!state.shield_raised);
    assert_eq!(dice.remaining(), 0);
}

/// Covers shields scenario "Another combatant cannot raise on this turn".
/// CombatantId(2) has no shield, so the shield check would be NoShield if the
/// turn guard did not return first. Player Core, "Raise a Shield".
/// Player Core, "Step 2: Act".
#[test]
fn raise_shield_off_turn_returns_notyourturn_and_spends_nothing() {
    let (mut encounter, dice) = begin(
        vec![
            pc(1, 0, 20, 11),
            participant(2, Side::Pc, creature(20, 11, 0, false, None)),
        ],
        &[10, 8],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let before = encounter.state(id(2)).expect("state").clone();
    assert_eq!(before.actions_remaining, 0);
    assert!(!before.shield_raised);
    assert_eq!(encounter.creature(id(2)).expect("creature").shield, None);
    let error = encounter.raise_shield(id(2)).expect_err("off turn");
    assert_eq!(error, EncounterError::NotYourTurn);
    assert_eq!(encounter.state(id(2)).expect("state"), &before);
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Strike. Player Core, "Shield Block".
#[test]
fn a_legal_shield_block_changes_shield_hit_points() {
    let shield = ShieldTemplate {
        hardness: 5,
        hit_points: 20,
        broken_threshold: 10,
        shield_ac_bonus: 2,
    };
    let (mut encounter, mut dice) = begin(
        vec![
            participant(1, Side::Pc, creature(30, 11, 0, false, Some(shield))),
            participant(2, Side::Pc, creature(20, 11, 0, false, None)),
        ],
        &[10, 8, 11, 8],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.raise_shield(id(1)).expect("raise");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("attacker"), id(2));
    let outcome = rolled(
        encounter
            .strike(
                id(2),
                id(1),
                &profile(0, Agility::NotAgile, 1, 8, 4),
                &[],
                ShieldBlock::Requested,
                &Detections::new(),
                &mut dice,
            )
            .expect("strike"),
    );
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
    assert_eq!(hit_points(&encounter, 1), 23);
    assert_eq!(
        encounter.state(id(1)).expect("state").shield_hit_points,
        Some(13)
    );
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Aid.
#[test]
fn the_same_aid_twice_matches() {
    let once = || {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                aided(2, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            ],
            &[10, 8, 10],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.end_turn(&mut dice).expect("end");
        assert_eq!(encounter.start_turn().expect("ally"), id(2));
        let outcome = encounter.aid(id(1), id(2), &mut dice).expect("aid");
        assert_eq!(dice.remaining(), 0);
        outcome
    };
    let first = once();
    let second = once();
    assert_eq!(first, second);
    assert_eq!(first.total, 17);
    assert_eq!(first.degree, DegreeOfSuccess::Success);
    assert_eq!(first.circumstance, AidCircumstance::Plus1);
}

/// Player Core, Aid.
/// Player Core, "Step 2: Act".
#[test]
fn preparing_spends_one_action() {
    let (mut encounter, dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    assert_eq!(actions(&encounter, 1), 2);
    assert_eq!(
        preparation(&encounter, 1),
        AidPreparation::Prepared { ally: id(2) }
    );
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 2: Act".
#[test]
fn preparing_again_only_spends_an_action() {
    let (mut encounter, dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 8],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    set_prepared(&mut encounter, 1, 2);
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    assert_eq!(actions(&encounter, 1), 2);
    assert_eq!(
        preparation(&encounter, 1),
        AidPreparation::Prepared { ally: id(2) }
    );
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 2: Act".
#[test]
fn preparing_again_for_a_new_ally_replaces_the_name() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
            pc(3, 0, 20, 11),
        ],
        &[10, 8, 6, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    set_prepared(&mut encounter, 1, 2);
    assert!(reaction(&encounter, 1));
    encounter.prepare_to_aid(id(1), id(3)).expect("prepare");
    let error = encounter
        .aid(id(1), id(2), &mut dice)
        .expect_err("old ally");
    assert_eq!(actions(&encounter, 1), 2);
    assert_eq!(
        preparation(&encounter, 1),
        AidPreparation::Prepared { ally: id(3) }
    );
    assert_eq!(error, EncounterError::NotThePreparedAlly);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 2: Act".
#[test]
fn preparing_off_turn_is_notyourturn() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 11), pc(2, 0, 20, 11)], &[10, 9, 8]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.set_vital_for_test(id(2), Vital::Unconscious);
    assert_eq!(preparation(&encounter, 2), AidPreparation::NotPrepared);
    assert!(reaction(&encounter, 2));
    let error = encounter
        .prepare_to_aid(id(2), id(1))
        .expect_err("off turn");
    assert_eq!(error, EncounterError::NotYourTurn);
    assert_eq!(preparation(&encounter, 2), AidPreparation::NotPrepared);
    assert!(reaction(&encounter, 2));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 2: Act".
#[test]
fn a_dying_preparer_on_their_turn_returns_notconscious() {
    let (mut encounter, dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 8],
    );
    encounter.set_vital_for_test(id(1), Vital::Dying(dying_value()));
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let error = encounter.prepare_to_aid(id(1), id(2)).expect_err("dying");
    assert_eq!(error, EncounterError::NotConscious);
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 2: Act".
#[test]
fn no_actions_does_not_prepare() {
    let (mut encounter, dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 8],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.spend_actions(id(1), 3).expect("spend");
    let error = encounter
        .prepare_to_aid(id(1), id(2))
        .expect_err("no actions");
    assert_eq!(error, EncounterError::InsufficientActions);
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
#[test]
fn preparing_with_no_aid_modifier_does_not_spend_the_action() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 11), pc(2, 0, 20, 11)], &[10, 8, 8]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    let error = encounter
        .prepare_to_aid(id(1), id(2))
        .expect_err("no modifier");
    assert_eq!(error, EncounterError::NoAidModifier);
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
#[test]
fn preparing_for_an_enemy_is_notanally() {
    let (mut encounter, dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            enemy(2, 0, 20, 11),
        ],
        &[10, 8, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let error = encounter.prepare_to_aid(id(1), id(2)).expect_err("enemy");
    assert_eq!(error, EncounterError::NotAnAlly);
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
#[test]
fn preparing_for_an_unknown_ally_is_unknowncombatant() {
    let (mut encounter, dice) = begin(
        vec![aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster))],
        &[10, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let error = encounter.prepare_to_aid(id(1), id(9)).expect_err("unknown");
    assert_eq!(error, EncounterError::UnknownCombatant);
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
#[test]
fn preparing_for_yourself_is_selftarget() {
    let (mut encounter, dice) = begin(
        vec![aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster))],
        &[10, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let error = encounter.prepare_to_aid(id(1), id(1)).expect_err("self");
    assert_eq!(error, EncounterError::SelfTarget);
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, Unconscious.
/// Player Core, "Getting Knocked Out".
#[test]
fn preparing_for_an_unconscious_or_dying_ally_is_legal() {
    let (mut encounter, dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
            pc(3, 0, 20, 11),
        ],
        &[10, 8, 6, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.set_vital_for_test(id(2), Vital::Unconscious);
    encounter.set_vital_for_test(id(3), Vital::Dying(dying_value()));
    encounter.prepare_to_aid(id(1), id(2)).expect("unconscious");
    assert_eq!(actions(&encounter, 1), 2);
    assert_eq!(
        preparation(&encounter, 1),
        AidPreparation::Prepared { ally: id(2) }
    );
    encounter.prepare_to_aid(id(1), id(3)).expect("dying");
    assert_eq!(actions(&encounter, 1), 1);
    assert_eq!(
        preparation(&encounter, 1),
        AidPreparation::Prepared { ally: id(3) }
    );
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, Unconscious.
/// Player Core, "Getting Knocked Out".
#[test]
fn preparing_for_a_dead_ally_is_rejected() {
    let (mut encounter, dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            dead(pc(2, 0, 20, 11)),
        ],
        &[10, 8, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let error = encounter.prepare_to_aid(id(1), id(2)).expect_err("dead");
    assert_eq!(error, EncounterError::AllyDead);
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, Unconscious.
/// Player Core, "Getting Knocked Out".
#[test]
fn notanally_wins_over_allydead() {
    let (mut encounter, dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            dead(enemy(2, 0, 20, 11)),
        ],
        &[10, 8, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let error = encounter.prepare_to_aid(id(1), id(2)).expect_err("enemy");
    assert_eq!(error, EncounterError::NotAnAlly);
    assert_ne!(error, EncounterError::AllyDead);
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, Unconscious.
/// Player Core, "Getting Knocked Out".
#[test]
fn allydead_wins_over_noaidmodifier_when_preparing() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 11), dead(pc(2, 0, 20, 11))], &[10, 8, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let error = encounter.prepare_to_aid(id(1), id(2)).expect_err("dead");
    assert_eq!(error, EncounterError::AllyDead);
    assert_ne!(error, EncounterError::NoAidModifier);
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 1: Start Your Turn".
#[test]
fn a_participant_starts_not_prepared() {
    let mut starter = pc(1, 0, 20, 11);
    starter.state.aid_preparation = AidPreparation::Prepared { ally: id(2) };
    let (encounter, dice) = begin(vec![starter], &[10]);
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert!(reaction(&encounter, 1));
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(initiative(&encounter, 1), 10);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Aid.
/// Player Core, "Step 1: Start Your Turn".
#[test]
fn another_turn_does_not_clear_preparation() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            aided(2, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
        ],
        &[10, 8, 8],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("ally"), id(2));
    assert_eq!(
        preparation(&encounter, 1),
        AidPreparation::Prepared { ally: id(2) }
    );
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 1: Start Your Turn".
#[test]
fn your_next_turn_clears_preparation() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            aided(2, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
        ],
        &[10, 8, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("ally"), id(2));
    encounter.end_turn(&mut dice).expect("end ally");
    assert_eq!(encounter.start_turn().expect("next"), id(1));
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert_eq!(actions(&encounter, 1), 3);
    assert!(reaction(&encounter, 1));
    let error = encounter.aid(id(1), id(2), &mut dice).expect_err("cleared");
    assert_eq!(error, EncounterError::NotPrepared);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 1: Start Your Turn".
#[test]
fn an_unused_preparation_expires_at_the_next_start() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            aided(2, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
        ],
        &[10, 8, 8],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("ally"), id(2));
    encounter.end_turn(&mut dice).expect("end ally");
    assert_eq!(encounter.start_turn().expect("next"), id(1));
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 1: Start Your Turn".
#[test]
fn a_successful_aid_clears_preparation() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 10, 8],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("ally"), id(2));
    let first = encounter.aid(id(1), id(2), &mut dice).expect("aid");
    assert_eq!(first.total, 17);
    assert_eq!(first.degree, DegreeOfSuccess::Success);
    assert_eq!(first.circumstance, AidCircumstance::Plus1);
    assert!(!reaction(&encounter, 1));
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    let error = encounter.aid(id(1), id(2), &mut dice).expect_err("again");
    assert_eq!(error, EncounterError::NotPrepared);
    assert_ne!(error, EncounterError::NoReaction);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 1: Start Your Turn".
#[test]
fn aiding_a_different_ally_is_notthepreparedally() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
            pc(3, 0, 20, 11),
        ],
        &[10, 8, 6, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    let error = encounter.aid(id(1), id(3), &mut dice).expect_err("other");
    assert_eq!(error, EncounterError::NotThePreparedAlly);
    assert_eq!(
        preparation(&encounter, 1),
        AidPreparation::Prepared { ally: id(2) }
    );
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 1: Start Your Turn".
#[test]
fn round_overflow_keeps_preparation() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 8],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("ally"), id(2));
    encounter.round = 4_294_967_295;
    let error = encounter.end_turn(&mut dice).expect_err("overflow");
    assert_eq!(error, EncounterError::RoundOverflow);
    assert_eq!(
        preparation(&encounter, 1),
        AidPreparation::Prepared { ally: id(2) }
    );
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Reactions in Encounters".
#[test]
fn a_success_returns_plus_1() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::Master)),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("ally"), id(2));
    let hp = hit_points(&encounter, 2);
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
    assert!(reaction(&encounter, 1));
    let outcome = encounter.aid(id(1), id(2), &mut dice).expect("aid");
    assert_eq!(outcome.total, 17);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
    assert_eq!(outcome.circumstance, AidCircumstance::Plus1);
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
    assert!(!reaction(&encounter, 1));
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert_eq!(actions(&encounter, 2), 3);
    assert_eq!(hit_points(&encounter, 2), hp);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Aid.
/// Player Core, "Reactions in Encounters".
#[test]
fn a_master_critical_success_returns_plus_3() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::Master)),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 18],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("ally"), id(2));
    let outcome = encounter.aid(id(1), id(2), &mut dice).expect("aid");
    assert_eq!(outcome.total, 25);
    assert_eq!(outcome.degree, DegreeOfSuccess::CriticalSuccess);
    assert_eq!(outcome.circumstance, AidCircumstance::Plus3);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Aid.
/// Player Core, "Reactions in Encounters".
#[test]
fn a_legendary_critical_success_returns_plus_4() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::Legendary)),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 18],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("ally"), id(2));
    let outcome = encounter.aid(id(1), id(2), &mut dice).expect("aid");
    assert_eq!(outcome.total, 25);
    assert_eq!(outcome.degree, DegreeOfSuccess::CriticalSuccess);
    assert_eq!(outcome.circumstance, AidCircumstance::Plus4);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Aid.
/// Player Core, "Reactions in Encounters".
#[test]
fn a_failure_returns_zero() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 4],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("ally"), id(2));
    let outcome = encounter.aid(id(1), id(2), &mut dice).expect("aid");
    assert_eq!(outcome.total, 11);
    assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
    assert_eq!(outcome.circumstance, AidCircumstance::Zero);
    assert!(!reaction(&encounter, 1));
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Aid.
/// Player Core, "Reactions in Encounters".
#[test]
fn a_critical_failure_returns_minus_1() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(4), Some(AidRank::Master)),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 1],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("ally"), id(2));
    let outcome = encounter.aid(id(1), id(2), &mut dice).expect("aid");
    assert_eq!(outcome.total, 5);
    assert_eq!(outcome.degree, DegreeOfSuccess::CriticalFailure);
    assert_eq!(outcome.circumstance, AidCircumstance::Minus1);
    assert!(!reaction(&encounter, 1));
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Aid.
/// Player Core, Unconscious.
/// Player Core, "Getting Knocked Out".
/// Player Core, "Reactions in Encounters".
#[test]
fn aid_against_a_dead_ally_is_rejected() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            dead(pc(2, 0, 20, 11)),
            pc(3, 0, 20, 11),
        ],
        &[10, 8, 6, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    set_prepared(&mut encounter, 1, 2);
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("third"), id(3));
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
    assert!(reaction(&encounter, 1));
    let error = encounter.aid(id(1), id(2), &mut dice).expect_err("dead");
    assert_eq!(error, EncounterError::AllyDead);
    assert!(reaction(&encounter, 1));
    assert_eq!(
        preparation(&encounter, 1),
        AidPreparation::Prepared { ally: id(2) }
    );
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, Unconscious.
/// Player Core, "Getting Knocked Out".
/// Player Core, "Reactions in Encounters".
#[test]
fn an_ally_who_dies_after_preparation_is_rejected() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
            pc(3, 0, 20, 11),
        ],
        &[10, 8, 6, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.set_vital_for_test(id(2), Vital::Unconscious);
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    assert_eq!(actions(&encounter, 1), 2);
    encounter.set_vital_for_test(id(2), Vital::Dead);
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("third"), id(3));
    let error = encounter.aid(id(1), id(2), &mut dice).expect_err("died");
    assert_eq!(error, EncounterError::AllyDead);
    assert!(reaction(&encounter, 1));
    assert_eq!(
        preparation(&encounter, 1),
        AidPreparation::Prepared { ally: id(2) }
    );
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, Unconscious.
/// Player Core, "Getting Knocked Out".
/// Player Core, "Reactions in Encounters".
#[test]
fn aid_succeeds_for_an_unconscious_ally() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.set_vital_for_test(id(2), Vital::Unconscious);
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    assert!(reaction(&encounter, 1));
    let outcome = encounter.aid(id(1), id(2), &mut dice).expect("aid");
    assert_eq!(outcome.total, 17);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
    assert_eq!(outcome.circumstance, AidCircumstance::Plus1);
    assert_eq!(outcome.circumstance.value(), 1);
    assert!(!reaction(&encounter, 1));
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert_eq!(vital(&encounter, 2), Vital::Unconscious);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Aid.
/// Player Core, Unconscious.
/// Player Core, "Getting Knocked Out".
/// Player Core, "Reactions in Encounters".
#[test]
fn aid_succeeds_for_a_dying_ally() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.set_vital_for_test(id(2), Vital::Dying(dying_value()));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    assert!(reaction(&encounter, 1));
    let outcome = encounter.aid(id(1), id(2), &mut dice).expect("aid");
    assert_eq!(outcome.total, 17);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
    assert_eq!(outcome.circumstance, AidCircumstance::Plus1);
    assert_eq!(outcome.circumstance.value(), 1);
    assert!(!reaction(&encounter, 1));
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert_eq!(vital(&encounter, 2), Vital::Dying(dying_value()));
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Aid.
/// Player Core, Unconscious.
/// Player Core, "Getting Knocked Out".
/// Player Core, "Reactions in Encounters".
#[test]
fn notprepared_wins_over_allydead() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            dead(pc(2, 0, 20, 11)),
        ],
        &[10, 8, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    let error = encounter
        .aid(id(1), id(2), &mut dice)
        .expect_err("unprepared");
    assert_eq!(error, EncounterError::NotPrepared);
    assert_ne!(error, EncounterError::AllyDead);
    assert!(reaction(&encounter, 1));
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, Unconscious.
/// Player Core, "Getting Knocked Out".
/// Player Core, "Reactions in Encounters".
#[test]
fn notthepreparedally_wins_over_allydead() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
            dead(pc(3, 0, 20, 11)),
        ],
        &[10, 8, 6, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    let error = encounter.aid(id(1), id(3), &mut dice).expect_err("other");
    assert_eq!(error, EncounterError::NotThePreparedAlly);
    assert_ne!(error, EncounterError::AllyDead);
    assert!(reaction(&encounter, 1));
    assert_eq!(
        preparation(&encounter, 1),
        AidPreparation::Prepared { ally: id(2) }
    );
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, Unconscious.
/// Player Core, "Getting Knocked Out".
/// Player Core, "Reactions in Encounters".
#[test]
fn allydead_wins_over_noaidmodifier() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 11), dead(pc(2, 0, 20, 11))], &[10, 8, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    set_prepared(&mut encounter, 1, 2);
    let error = encounter.aid(id(1), id(2), &mut dice).expect_err("dead");
    assert_eq!(error, EncounterError::AllyDead);
    assert_ne!(error, EncounterError::NoAidModifier);
    assert!(reaction(&encounter, 1));
    assert_eq!(
        preparation(&encounter, 1),
        AidPreparation::Prepared { ally: id(2) }
    );
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, Unconscious.
/// Player Core, "Getting Knocked Out".
/// Player Core, "Reactions in Encounters".
#[test]
fn allydead_wins_over_aidoverflow() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(
                1,
                Side::Pc,
                0,
                Some(2_147_483_628),
                Some(AidRank::BelowMaster),
            ),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 20],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.set_vital_for_test(id(2), Vital::Dead);
    assert!(reaction(&encounter, 1));
    let error = encounter.aid(id(1), id(2), &mut dice).expect_err("dead");
    assert_eq!(error, EncounterError::AllyDead);
    assert_ne!(
        error,
        EncounterError::AidOverflow(ArithmeticOverflow::SumDoesNotFit)
    );
    assert!(reaction(&encounter, 1));
    assert_eq!(
        preparation(&encounter, 1),
        AidPreparation::Prepared { ally: id(2) }
    );
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, Unconscious.
/// Player Core, "Getting Knocked Out".
/// Player Core, "Reactions in Encounters".
#[test]
fn allydead_wins_over_noreaction() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.set_vital_for_test(id(2), Vital::Dead);
    encounter.spend_reaction(id(1)).expect("spend reaction");
    let error = encounter.aid(id(1), id(2), &mut dice).expect_err("dead");
    assert_eq!(error, EncounterError::AllyDead);
    assert_ne!(error, EncounterError::NoReaction);
    assert!(!reaction(&encounter, 1));
    assert_eq!(
        preparation(&encounter, 1),
        AidPreparation::Prepared { ally: id(2) }
    );
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, Unconscious.
/// Player Core, "Getting Knocked Out".
#[test]
fn ready_from_encounter_surfaces_allydead() {
    let mapped = ready_from_encounter(EncounterError::AllyDead);
    assert_eq!(mapped, ReadyError::Encounter(ReadyEncounter::AllyDead));
    assert_ne!(mapped, ReadyError::Encounter(ReadyEncounter::NotAnAlly));
}

/// Player Core, Aid.
/// Player Core, Unconscious.
/// Player Core, "Getting Knocked Out".
/// Player Core, "Reactions in Encounters".
#[test]
fn a_trigger_that_never_fires_spends_nothing() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            aided(2, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
        ],
        &[10, 8, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    assert!(reaction(&encounter, 1));
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("ally"), id(2));
    encounter.end_turn(&mut dice).expect("end ally");
    assert_eq!(encounter.start_turn().expect("next"), id(1));
    assert_eq!(dice.remaining(), 1);
    assert!(reaction(&encounter, 1));
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
}

/// Player Core, Aid.
/// Player Core, "Reactions in Encounters".
#[test]
fn aid_with_the_reaction_already_spent_is_noreaction() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.spend_reaction(id(1)).expect("spend reaction");
    let error = encounter.aid(id(1), id(2), &mut dice).expect_err("spent");
    assert_eq!(error, EncounterError::NoReaction);
    assert_eq!(
        preparation(&encounter, 1),
        AidPreparation::Prepared { ally: id(2) }
    );
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
#[test]
fn aid_does_not_change_the_attack_count() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("ally"), id(2));
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
    encounter.aid(id(1), id(2), &mut dice).expect("aid");
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Aid.
#[test]
fn an_enemy_may_aid_an_enemy() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Enemy, 0, Some(7), Some(AidRank::BelowMaster)),
            aided(2, Side::Enemy, 0, Some(7), Some(AidRank::BelowMaster)),
        ],
        &[10, 8, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    let outcome = encounter.aid(id(1), id(2), &mut dice).expect("aid");
    assert_eq!(outcome.total, 17);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
    assert_eq!(outcome.circumstance, AidCircumstance::Plus1);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Aid.
/// Player Core, "Step 2: Act".
/// Player Core, Unconscious.
#[test]
fn an_unknown_aider_rolls_nothing() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 11)], &[1, 8]);
    assert!(encounter.turn_open().is_none());
    let error = encounter.aid(id(9), id(1), &mut dice).expect_err("unknown");
    assert_eq!(error, EncounterError::UnknownCombatant);
    assert_ne!(error, EncounterError::TurnNotOpen);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 2: Act".
/// Player Core, Unconscious.
#[test]
fn an_unknown_ally_rolls_nothing() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
        ],
        &[1, 1, 8],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.end_turn(&mut dice).expect("end");
    assert!(reaction(&encounter, 1));
    assert!(encounter.turn_open().is_none());
    let error = encounter.aid(id(1), id(9), &mut dice).expect_err("unknown");
    assert_eq!(error, EncounterError::UnknownCombatant);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 2: Act".
/// Player Core, Unconscious.
#[test]
fn aid_with_no_open_turn_is_turnnotopen() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
        ],
        &[1, 1, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.end_turn(&mut dice).expect("end");
    assert!(reaction(&encounter, 1));
    assert!(encounter.turn_open().is_none());
    let error = encounter.aid(id(1), id(2), &mut dice).expect_err("closed");
    assert_eq!(error, EncounterError::TurnNotOpen);
    assert!(reaction(&encounter, 1));
    assert_eq!(
        preparation(&encounter, 1),
        AidPreparation::Prepared { ally: id(2) }
    );
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 2: Act".
/// Player Core, Unconscious.
#[test]
fn turnnotopen_wins_over_notconscious() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
        ],
        &[1, 1, 8],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.set_vital_for_test(id(1), Vital::Unconscious);
    encounter.end_turn(&mut dice).expect("end");
    assert!(encounter.turn_open().is_none());
    let error = encounter.aid(id(1), id(2), &mut dice).expect_err("closed");
    assert_eq!(error, EncounterError::TurnNotOpen);
    assert_ne!(error, EncounterError::NotConscious);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 2: Act".
/// Player Core, Unconscious.
#[test]
fn notconscious_wins_over_later_errors() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 11)], &[1, 8]);
    encounter.set_vital_for_test(id(1), Vital::Unconscious);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    assert!(!reaction(&encounter, 1));
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    let error = encounter.aid(id(1), id(1), &mut dice).expect_err("down");
    assert_eq!(error, EncounterError::NotConscious);
    assert!(!reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 2: Act".
/// Player Core, Unconscious.
#[test]
fn a_dying_aider_returns_notconscious() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 18],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.set_vital_for_test(id(1), Vital::Dying(dying_value()));
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("ally"), id(2));
    assert!(reaction(&encounter, 1));
    let error = encounter.aid(id(1), id(2), &mut dice).expect_err("dying");
    assert_eq!(error, EncounterError::NotConscious);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 2: Act".
/// Player Core, Unconscious.
#[test]
fn a_dead_aider_returns_notconscious() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.set_vital_for_test(id(1), Vital::Dead);
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("ally"), id(2));
    assert!(reaction(&encounter, 1));
    let error = encounter.aid(id(1), id(2), &mut dice).expect_err("dead");
    assert_eq!(error, EncounterError::NotConscious);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
#[test]
fn aiding_yourself_is_selftarget() {
    let (mut encounter, mut dice) = begin(
        vec![aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster))],
        &[10, 10],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert!(reaction(&encounter, 1));
    let error = encounter.aid(id(1), id(1), &mut dice).expect_err("self");
    assert_eq!(error, EncounterError::SelfTarget);
    assert_ne!(error, EncounterError::NotPrepared);
    assert!(reaction(&encounter, 1));
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
#[test]
fn a_different_side_is_notanally() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 11), enemy(2, 0, 20, 11)], &[10, 8, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.spend_reaction(id(1)).expect("spend reaction");
    assert!(!reaction(&encounter, 1));
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    let error = encounter.aid(id(1), id(2), &mut dice).expect_err("side");
    assert_eq!(error, EncounterError::NotAnAlly);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 2: Act".
/// Player Core, Unconscious.
#[test]
fn an_unprepared_aider_does_not_roll() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 11), pc(2, 0, 20, 11)], &[10, 8, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.spend_reaction(id(1)).expect("spend reaction");
    assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
    let error = encounter
        .aid(id(1), id(2), &mut dice)
        .expect_err("unprepared");
    assert_eq!(error, EncounterError::NotPrepared);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 2: Act".
/// Player Core, Unconscious.
#[test]
fn no_aid_modifier_does_not_spend_the_reaction() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 11), pc(2, 0, 20, 11)], &[10, 8, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    set_prepared(&mut encounter, 1, 2);
    encounter.spend_reaction(id(1)).expect("spend reaction");
    assert!(!reaction(&encounter, 1));
    let error = encounter
        .aid(id(1), id(2), &mut dice)
        .expect_err("modifier");
    assert_eq!(error, EncounterError::NoAidModifier);
    assert!(!reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid.
/// Player Core, "Step 2: Act".
/// Player Core, Unconscious.
#[test]
fn an_aid_modifier_at_the_d20_limit_is_rolled() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(
                1,
                Side::Pc,
                0,
                Some(2_147_483_627),
                Some(AidRank::BelowMaster),
            ),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 20],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    let outcome = encounter.aid(id(1), id(2), &mut dice).expect("aid");
    assert_eq!(outcome.total, 2_147_483_647);
    assert_eq!(outcome.degree, DegreeOfSuccess::CriticalSuccess);
    assert_eq!(outcome.circumstance, AidCircumstance::Plus2);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Aid.
/// Player Core, "Step 2: Act".
/// Player Core, Unconscious.
#[test]
fn the_next_aid_modifier_above_the_d20_limit_is_aidoverflow() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(
                1,
                Side::Pc,
                0,
                Some(2_147_483_628),
                Some(AidRank::BelowMaster),
            ),
            pc(2, 0, 20, 11),
        ],
        &[10, 8, 20],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    assert!(reaction(&encounter, 1));
    let error = encounter
        .aid(id(1), id(2), &mut dice)
        .expect_err("overflow");
    assert_eq!(
        error,
        EncounterError::AidOverflow(ArithmeticOverflow::SumDoesNotFit)
    );
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Aid. Player Core, "Shield Block".
#[test]
fn a_reaction_spent_by_shield_block_still_reports_aidoverflow() {
    let shield = ShieldTemplate {
        hardness: 5,
        hit_points: 20,
        broken_threshold: 10,
        shield_ac_bonus: 2,
    };
    let parts = BuildParts {
        hit_points: 30,
        armor_class: 11,
        perception: 0,
        speed_feet: 25,
        strike: MeleeStrike {
            attack_bonus: 0,
            damage_dice: DieCount::new(1).expect("dice"),
            damage_faces: faces(8),
            damage_bonus: 0,
            reach_feet: ReachFeet::new(5).expect("reach"),
            agile: false,
            damage_type: crate::damage::DamageType::Slashing,
        },
        shield: Some(shield),
        stealth_dc: None,
        aid_modifier: Some(i32::MAX - 19),
        aid_rank: Some(AidRank::BelowMaster),
        uses_dying_rules: Some(false),
        saves: None,
        skills: None,
        defenses: crate::damage::DamageDefenses::empty(),
        spellcasting: None,
    };
    let aider = participant(
        1,
        Side::Pc,
        derive(&CharacterBuild::try_new(parts).expect("build")),
    );
    let (mut encounter, mut dice) = begin(
        vec![aider, pc(2, 0, 20, 11), enemy(3, 0, 20, 11)],
        &[10, 8, 6, 12, 8, 20],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
    encounter.raise_shield(id(1)).expect("raise");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("ally"), id(2));
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("attacker"), id(3));
    encounter
        .strike(
            id(3),
            id(1),
            &profile(0, Agility::NotAgile, 1, 8, 4),
            &[],
            ShieldBlock::Requested,
            &Detections::new(),
            &mut dice,
        )
        .expect("strike");
    assert!(!reaction(&encounter, 1));
    let error = encounter
        .aid(id(1), id(2), &mut dice)
        .expect_err("overflow");
    assert_eq!(
        error,
        EncounterError::AidOverflow(ArithmeticOverflow::SumDoesNotFit)
    );
    assert_ne!(error, EncounterError::NoReaction);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 138, "Reactive Strike". Player Core pg. 442, Concealed.
#[test]
fn a_natural_5_at_dc_5_then_the_reactive_attack() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Observed,
        Concealment::Concealed,
        None,
    );
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 5, 10, 3]);
    assert_eq!(encounter.turn_open(), None);
    let outcome = react_seen(
        &mut encounter,
        &mut dice,
        &detections,
        profile(8, Agility::NotAgile, 1, 8, 4),
    )
    .expect("reactive");
    let StrikeDelivery::Rolled(strike) = outcome.strike else {
        panic!("expected a rolled strike, got {outcome:?}");
    };
    assert_eq!(strike.degree, DegreeOfSuccess::Success);
    assert_eq!(strike.total, 18);
    assert_eq!(strike.map, 0);
    assert_eq!(strike.damage, 7);
    assert_eq!(outcome.disruption, Disruption::NotDisrupted);
    assert_eq!(hit_points(&encounter, 2), 13);
    assert!(!reaction(&encounter, 1));
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
    assert_eq!(
        detections.get(id(1), id(2)),
        Relation {
            detection: Detection::Observed,
            concealment: Concealment::Concealed,
        }
    );
}

/// Player Core pg. 138, "Reactive Strike". Player Core pg. 442, Concealed.
#[test]
fn a_natural_4_at_dc_5_loses_the_reactive_strike() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Observed,
        Concealment::Concealed,
        None,
    );
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 4, 10, 3]);
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
    assert!(reaction(&encounter, 1));
    let outcome = react_seen(
        &mut encounter,
        &mut dice,
        &detections,
        profile(8, Agility::NotAgile, 1, 8, 4),
    )
    .expect("lost");
    let lost = lost(outcome.strike);
    assert_eq!(lost.natural, 4);
    assert_eq!(lost.dc, 5);
    assert_eq!(lost.degree, DegreeOfSuccess::Failure);
    assert_eq!(outcome.disruption, Disruption::NotDisrupted);
    assert!(!reaction(&encounter, 1));
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
    assert_eq!(hit_points(&encounter, 2), 20);
    assert_eq!(dice.remaining(), 2);
}

/// Player Core pg. 138, "Reactive Strike". Player Core pg. 444, Hidden.
#[test]
fn a_natural_10_at_dc_11_loses_the_reactive_strike() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Hidden,
        Concealment::NotConcealed,
        Some(17),
    );
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 12]);
    let outcome = react_seen(
        &mut encounter,
        &mut dice,
        &detections,
        profile(8, Agility::NotAgile, 0, 8, 0),
    )
    .expect("lost");
    let lost = lost(outcome.strike);
    assert_eq!(lost.natural, 10);
    assert_eq!(lost.dc, 11);
    assert_eq!(lost.degree, DegreeOfSuccess::Failure);
    assert!(!reaction(&encounter, 1));
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(hit_points(&encounter, 2), 20);
}

/// Player Core pg. 138, "Reactive Strike". Player Core pg. 444, Hidden. Player Core pg. 442, Concealed.
#[test]
fn hidden_and_concealed_use_one_dc_11_roll() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Hidden,
        Concealment::Concealed,
        Some(17),
    );
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 20]);
    let outcome = react_seen(
        &mut encounter,
        &mut dice,
        &detections,
        profile(8, Agility::NotAgile, 0, 8, 0),
    )
    .expect("lost");
    let lost = lost(outcome.strike);
    assert_eq!(lost.natural, 10);
    assert_eq!(lost.dc, 11);
    assert_eq!(dice.remaining(), 1);
    assert!(!reaction(&encounter, 1));
    assert_eq!(hit_points(&encounter, 2), 20);
}

/// Player Core pg. 138, "Reactive Strike". Player Core pg. 402, "Multiple Attack Penalty".
#[test]
fn a_lost_reactive_strike_does_not_count_toward_map() {
    let detections = relate(
        Detections::new(),
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
        &[15, 10, 5, 4, 12, 8],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.set_attack_count_for_test(id(1), 1);
    encounter.spend_actions(id(1), 1).expect("spend");
    assert!(reaction(&encounter, 1));
    let outcome = react_seen(
        &mut encounter,
        &mut dice,
        &detections,
        profile(8, Agility::NotAgile, 0, 8, 0),
    )
    .expect("lost");
    assert!(matches!(outcome.strike, StrikeDelivery::Lost(_)));
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 1);
    let strike = rolled(
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
            .expect("strike"),
    );
    assert_eq!(strike.natural, 12);
    assert_eq!(strike.map, -5);
    assert_eq!(strike.total, 15);
    assert_ne!(strike.map, -10);
    assert_ne!(strike.total, 11);
    assert_eq!(strike.degree, DegreeOfSuccess::Failure);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(actions(&encounter, 1), 1);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 2);
}

/// Player Core pg. 447, Undetected. Player Core pg. 138, "Reactive Strike".
#[test]
fn an_undetected_actor_cannot_be_the_target() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Undetected,
        Concealment::NotConcealed,
        Some(17),
    );
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 11]);
    let error = react_seen(
        &mut encounter,
        &mut dice,
        &detections,
        profile(8, Agility::NotAgile, 0, 8, 0),
    )
    .expect_err("undetected");
    assert_eq!(error, ReactiveError::CannotTarget);
    assert_ne!(error, ReactiveError::NoReaction);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core pg. 138, "Reactive Strike". Player Core pg. 447, Undetected.
#[test]
fn no_reaction_beats_an_undetected_actor() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Undetected,
        Concealment::NotConcealed,
        Some(17),
    );
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 11]);
    encounter.spend_reaction(id(1)).expect("spend");
    let error = react_seen(
        &mut encounter,
        &mut dice,
        &detections,
        profile(8, Agility::NotAgile, 0, 8, 0),
    )
    .expect_err("spent");
    assert_eq!(error, ReactiveError::NoReaction);
    assert_ne!(error, ReactiveError::CannotTarget);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Reactive Strike".
#[test]
fn the_same_reactive_strike_twice_matches() {
    let run = || {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 3]);
        react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (-1, 0),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("reactive strike")
    };
    let first = run();
    let second = run();
    assert_eq!(first, second);
    assert_eq!(delivered(&first).natural, 10);
    assert_eq!(delivered(&first).total, 18);
    assert_eq!(delivered(&first).degree, DegreeOfSuccess::Success);
    assert_eq!(delivered(&first).map, 0);
    assert_eq!(delivered(&first).damage, 7);
    assert_eq!(first.disruption, Disruption::NotDisrupted);
}

/// Player Core, "Reactive Strike". Player Core, "Interact". Player Core, "Point Out".
///
/// Interact is one action with manipulate. Point Out is manipulate when the caller passes it.
#[test]
fn a_manipulate_action_in_reach_triggers() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Enemy), &[10, 8, 10, 3]);
    assert_eq!(encounter.turn_open(), None);
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (-1, 0),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("manipulate");
    assert_eq!(delivered(&outcome).degree, DegreeOfSuccess::Success);
    assert_eq!(delivered(&outcome).total, 18);
    assert_eq!(delivered(&outcome).map, 0);
    assert_eq!(delivered(&outcome).damage, 7);
    assert_eq!(outcome.disruption, Disruption::NotDisrupted);
    assert_eq!(hit_points(&encounter, 2), 13);
    assert!(!reaction(&encounter, 1));
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(encounter.turn_open(), None);
}

/// Player Core, "Reactive Strike". Player Core, "Drop Prone". Player Core, "Stand".
///
/// Drop Prone, Stand, Stride, and Crawl are Move. Stand is Actions 2303 (Player Core pg. 418).
/// This call passes the move trait and does not resolve those actions.
#[test]
fn a_move_action_in_reach_triggers() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 3]);
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MOVE_ONLY),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (-1, 0),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("move");
    assert_eq!(delivered(&outcome).degree, DegreeOfSuccess::Success);
    assert_eq!(delivered(&outcome).map, 0);
    assert_eq!(delivered(&outcome).damage, 7);
    assert_eq!(outcome.disruption, Disruption::NotDisrupted);
    assert!(!reaction(&encounter, 1));
    assert_eq!(actions(&encounter, 1), 0);
}

/// Player Core, "Reactive Strike".
///
/// `Ranged` means this action is a ranged attack. It is not a Player Core action trait.
#[test]
fn a_ranged_attack_in_reach_triggers() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 3]);
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(RANGED_ATTACK),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (-1, 0),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("ranged");
    assert_eq!(delivered(&outcome).degree, DegreeOfSuccess::Success);
    assert_eq!(delivered(&outcome).map, 0);
    assert_eq!(delivered(&outcome).damage, 7);
    assert_eq!(outcome.disruption, Disruption::NotDisrupted);
    assert!(!reaction(&encounter, 1));
    assert_eq!(actions(&encounter, 1), 0);
}

/// Player Core, "Reactive Strike".
///
/// Both creatures are Pc. Side is not read.
#[test]
fn the_same_side_still_triggers() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 3]);
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (-1, 0),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("same side");
    assert_eq!(delivered(&outcome).degree, DegreeOfSuccess::Success);
    assert_eq!(delivered(&outcome).map, 0);
    assert_eq!(delivered(&outcome).damage, 7);
    assert_eq!(outcome.disruption, Disruption::NotDisrupted);
    assert_eq!(hit_points(&encounter, 2), 13);
    assert!(!reaction(&encounter, 1));
    assert_eq!(actions(&encounter, 1), 0);
}

/// Player Core, "Reactive Strike".
#[test]
fn move_and_manipulate_on_one_cost_trigger_once() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 3]);
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MOVE_AND_MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("one outcome");
    assert_eq!(delivered(&outcome).map, 0);
    assert_eq!(delivered(&outcome).damage, 7);
    assert_eq!(outcome.disruption, Disruption::NotDisrupted);
    assert_eq!(hit_points(&encounter, 2), 13);
    assert!(!reaction(&encounter, 1));
    assert_eq!(actions(&encounter, 1), 0);
}

/// Player Core, "Reactive Strike". Player Core, "Release".
#[test]
fn a_free_manipulate_action_other_than_release_triggers() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 3]);
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        free_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("free manipulate");
    assert_eq!(delivered(&outcome).degree, DegreeOfSuccess::Success);
    assert_eq!(delivered(&outcome).map, 0);
    assert_eq!(delivered(&outcome).damage, 7);
    assert_eq!(outcome.disruption, Disruption::NotDisrupted);
    assert!(!reaction(&encounter, 1));
    assert_eq!(actions(&encounter, 1), 0);
}

/// Player Core, "Step".
#[test]
fn step_does_not_trigger() {
    let (mut encounter, mut dice) = begin(
        vec![
            with_reach(pc(1, 0, 20, 18), 7),
            combatant(2, Side::Pc, 0, 20, 18, false),
        ],
        &[10, 8, 10],
    );
    assert!(reaction(&encounter, 1));
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MOVE_ONLY),
        ActionIdentity::Other,
        Some(MoveAction::Step),
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Large,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("step");
    assert_eq!(error, ReactiveError::NotATrigger);
    assert_ne!(error, ReactiveError::UnsupportedSize);
    assert_ne!(error, ReactiveError::InvalidReach);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Strike.
#[test]
fn a_melee_strike_does_not_trigger() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(NO_TRIGGER),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("melee");
    assert_eq!(error, ReactiveError::NotATrigger);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Release".
#[test]
fn release_does_not_trigger() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        free_action(MANIPULATE),
        ActionIdentity::Release,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("release");
    assert_eq!(error, ReactiveError::NotATrigger);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Seek".
#[test]
fn seek_does_not_trigger() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(NO_TRIGGER),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("seek");
    assert_eq!(error, ReactiveError::NotATrigger);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Range and Reach". Player Core, "Reactive Strike".
#[test]
fn out_of_reach_does_not_trigger() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 2),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("reach");
    assert_eq!(error, ReactiveError::OutOfReach);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Range and Reach".
#[test]
fn reach_0_does_not_trigger() {
    let (mut encounter, mut dice) = begin(
        vec![
            with_reach(pc(1, 0, 20, 18), 0),
            combatant(2, Side::Pc, 0, 20, 18, false),
        ],
        &[10, 8, 10],
    );
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("reach 0");
    assert_eq!(error, ReactiveError::OutOfReach);
    assert_ne!(error, ReactiveError::InvalidReach);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Range and Reach".
#[test]
fn the_same_square_is_not_in_reach() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 0),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("same square");
    assert_eq!(error, ReactiveError::OutOfReach);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Range and Reach". Player Core, "Size, Space, and Reach".
#[test]
fn a_reactive_reach_that_is_not_a_multiple_of_5_is_rejected() {
    let (mut encounter, mut dice) = begin(
        vec![
            with_reach(pc(1, 0, 20, 18), 7),
            combatant(2, Side::Pc, 0, 20, 18, false),
        ],
        &[10, 8, 10],
    );
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("reach 7");
    assert_eq!(error, ReactiveError::InvalidReach);
    assert_ne!(error, ReactiveError::OutOfReach);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Size, Space, and Reach".
#[test]
fn a_large_creature_is_unsupported() {
    let (mut encounter, mut dice) = begin(
        vec![
            with_reach(pc(1, 0, 20, 18), 7),
            combatant(2, Side::Pc, 0, 20, 18, false),
        ],
        &[10, 8, 10],
    );
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Large,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("large");
    assert_eq!(error, ReactiveError::UnsupportedSize);
    assert_ne!(error, ReactiveError::InvalidReach);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Range and Reach".
#[test]
fn a_distance_that_does_not_fit_is_overflow() {
    let (mut encounter, mut dice) = begin(
        vec![
            with_reach(pc(1, 0, 20, 18), 7),
            combatant(2, Side::Pc, 0, 20, 18, false),
        ],
        &[10, 8, 10],
    );
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (i32::MIN, 0),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("overflow");
    assert_eq!(error, ReactiveError::Overflow);
    assert_ne!(error, ReactiveError::InvalidReach);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Range and Reach".
#[test]
fn a_10_ft_reach_diagonal_is_in_reach() {
    let (mut encounter, mut dice) = begin(
        vec![
            with_reach(pc(1, 0, 20, 18), 10),
            combatant(2, Side::Pc, 0, 20, 18, false),
        ],
        &[10, 8, 10, 3],
    );
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (2, 2),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("diagonal");
    assert_eq!(delivered(&outcome).degree, DegreeOfSuccess::Success);
    assert_eq!(delivered(&outcome).damage, 7);
    assert!(!reaction(&encounter, 1));
}

/// Player Core, "Range and Reach".
#[test]
fn a_10_ft_reach_does_not_reach_three_squares_straight() {
    let (mut encounter, mut dice) = begin(
        vec![
            with_reach(pc(1, 0, 20, 18), 10),
            combatant(2, Side::Pc, 0, 20, 18, false),
        ],
        &[10, 8, 10],
    );
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (3, 0),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("three squares");
    assert_eq!(error, ReactiveError::OutOfReach);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Reactive Strike".
///
/// Proposed DEV-45: only the actor's square is tested. No path is read, and this test
/// does not read a SquareEvent.
#[test]
fn a_move_that_starts_outside_reach_does_not_trigger() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MOVE_ONLY),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 2),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("outside");
    assert_eq!(error, ReactiveError::OutOfReach);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Reactions in Encounters".
#[test]
fn a_second_reactive_strike_is_noreaction() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 3, 4]);
    let first = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (-1, 0),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("first");
    assert_eq!(delivered(&first).damage, 7);
    assert_eq!(hit_points(&encounter, 2), 13);
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (-1, 0),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("second");
    assert_eq!(error, ReactiveError::NoReaction);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(hit_points(&encounter, 2), 13);
}

/// Player Core, "Reactions in Encounters".
///
/// The reaction was spent through Encounter::spend_reaction, as Aid would.
#[test]
fn a_reaction_already_spent_is_noreaction() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
    encounter.spend_reaction(id(1)).expect("spend");
    assert!(!reaction(&encounter, 1));
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("spent");
    assert_eq!(error, ReactiveError::NoReaction);
    assert_eq!(dice.remaining(), 1);
    assert!(!reaction(&encounter, 1));
}

/// Player Core, "Reactions in Encounters".
#[test]
fn actor_actions_stay_unchanged() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[8, 10, 10, 3]);
    assert_eq!(encounter.start_turn().expect("start"), id(2));
    assert_eq!(actions(&encounter, 2), 3);
    react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("spend");
    assert_eq!(actions(&encounter, 2), 3);
    assert_eq!(encounter.turn_open(), Some(id(2)));
    assert_eq!(actions(&encounter, 1), 0);
    assert!(!reaction(&encounter, 1));
}

/// Player Core, "Reactions in Encounters".
#[test]
fn a_reactive_strike_on_your_turn_spends_no_action() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 3]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    assert_eq!(actions(&encounter, 1), 3);
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("own turn");
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert!(!reaction(&encounter, 1));
    assert_eq!(delivered(&outcome).map, 0);
    assert_eq!(delivered(&outcome).damage, 7);
    assert_eq!(hit_points(&encounter, 2), 13);
}

/// Player Core, Strike. Player Core, "Multiple Attack Penalty".
#[test]
fn a_reactive_strike_off_your_turn_spends_no_action() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[8, 10, 10, 3]);
    assert_eq!(encounter.start_turn().expect("start"), id(2));
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("off turn");
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(delivered(&outcome).map, 0);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
    assert!(!reaction(&encounter, 1));
    assert_eq!(encounter.turn_open(), Some(id(2)));
}

/// Player Core, Strike. Player Core, "Multiple Attack Penalty".
#[test]
fn a_reactive_strike_off_your_turn_leaves_the_count() {
    let (mut encounter, mut dice) =
        begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 2, 2, 10, 3]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let miss = profile(8, Agility::NotAgile, 0, 8, 0);
    encounter
        .strike(
            id(1),
            id(2),
            &miss,
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
            &miss,
            &[],
            ShieldBlock::Declined,
            &Detections::new(),
            &mut dice,
        )
        .expect("second");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("next"), id(2));
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 2);
    assert_eq!(actions(&encounter, 1), 0);
    assert!(reaction(&encounter, 1));
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("off turn");
    assert_eq!(delivered(&outcome).map, 0);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 2);
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(encounter.turn_open(), Some(id(2)));
}

/// Player Core, "Reactive Strike". Player Core, "Multiple Attack Penalty".
#[test]
fn reactive_strike_does_not_use_map() {
    let (mut encounter, mut dice) =
        begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 2, 2, 18, 3]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let miss = profile(8, Agility::NotAgile, 0, 8, 0);
    encounter
        .strike(
            id(1),
            id(2),
            &miss,
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
            &miss,
            &[],
            ShieldBlock::Declined,
            &Detections::new(),
            &mut dice,
        )
        .expect("second");
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 2);
    assert_eq!(actions(&encounter, 1), 1);
    let modifiers = [penalty(PenaltyType::Circumstance, -2, 1, Statistic::Attack)];
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::Agile, 1, 8, 4),
        &modifiers,
    )
    .expect("agile reaction");
    assert_eq!(delivered(&outcome).map, 0);
    assert_eq!(delivered(&outcome).total, 24);
    assert_ne!(delivered(&outcome).total, 16);
    assert_ne!(delivered(&outcome).total, 14);
    assert_eq!(delivered(&outcome).degree, DegreeOfSuccess::Success);
    assert_eq!(delivered(&outcome).damage, 7);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 2);
    assert_eq!(actions(&encounter, 1), 1);
}

/// Player Core, "Reactive Strike". Player Core, "Multiple Attack Penalty".
#[test]
fn reactive_strike_does_not_count_toward_map() {
    let (mut encounter, mut dice) =
        begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 2, 10, 3, 12]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
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
        .expect("first");
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 1);
    assert_eq!(actions(&encounter, 1), 2);
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("reaction");
    assert_eq!(delivered(&outcome).map, 0);
    assert_eq!(delivered(&outcome).damage, 7);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 1);
    let follow = rolled(
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
            .expect("follow"),
    );
    assert_eq!(follow.map, -5);
    assert_eq!(follow.total, 15);
    assert_ne!(follow.map, -10);
    assert_ne!(follow.total, 10);
}

/// Player Core, "Reactive Strike". Player Core, "Disrupting Actions".
///
/// The result is Disrupted. The caller voids the manipulate action's effects.
/// This function leaves the cost spent.
#[test]
fn a_critical_hit_disrupts_a_manipulate_action() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 15, Side::Pc), &[8, 10, 15, 3]);
    assert_eq!(encounter.start_turn().expect("start"), id(2));
    encounter.spend_actions(id(2), 1).expect("spent cost");
    assert_eq!(actions(&encounter, 2), 2);
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(10, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("crit");
    assert_eq!(delivered(&outcome).total, 25);
    assert_eq!(delivered(&outcome).degree, DegreeOfSuccess::CriticalSuccess);
    assert_eq!(delivered(&outcome).damage, 14);
    assert_eq!(outcome.disruption, Disruption::Disrupted);
    assert_eq!(hit_points(&encounter, 2), 6);
    assert_eq!(actions(&encounter, 2), 2);
    assert_eq!(encounter.turn_open(), Some(id(2)));
    assert!(!reaction(&encounter, 1));
}

/// Player Core, "Reactive Strike". Player Core, "Disrupting Actions".
#[test]
fn a_critical_hit_does_not_disrupt_a_move() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 15, Side::Pc), &[10, 8, 15, 3]);
    assert_eq!(encounter.turn_open(), None);
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MOVE_ONLY),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(10, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("move crit");
    assert_eq!(delivered(&outcome).degree, DegreeOfSuccess::CriticalSuccess);
    assert_eq!(delivered(&outcome).damage, 14);
    assert_eq!(outcome.disruption, Disruption::NotDisrupted);
    assert!(!reaction(&encounter, 1));
}

/// Player Core, "Reactive Strike". Player Core, "Disrupting Actions".
#[test]
fn a_critical_hit_does_not_disrupt_a_ranged_attack() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 15, Side::Pc), &[10, 8, 15, 3]);
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(RANGED_ATTACK),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(10, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("ranged crit");
    assert_eq!(delivered(&outcome).degree, DegreeOfSuccess::CriticalSuccess);
    assert_eq!(delivered(&outcome).damage, 14);
    assert_eq!(outcome.disruption, Disruption::NotDisrupted);
}

/// Player Core, "Reactive Strike". Player Core, "Disrupting Actions".
#[test]
fn a_natural_20_that_is_only_a_success_does_not_disrupt() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 25, Side::Pc), &[10, 8, 20, 3]);
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(0, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("natural 20");
    assert_eq!(delivered(&outcome).total, 20);
    assert_eq!(delivered(&outcome).degree, DegreeOfSuccess::Success);
    assert_eq!(delivered(&outcome).damage, 7);
    assert_ne!(delivered(&outcome).damage, 14);
    assert_eq!(outcome.disruption, Disruption::NotDisrupted);
    assert_eq!(hit_points(&encounter, 2), 13);
    assert_eq!(vital(&encounter, 1), Vital::Alive);
    assert_eq!(vital(&encounter, 2), Vital::Alive);
}

/// Player Core, "Reactive Strike". Player Core, "Disrupting Actions".
#[test]
fn actor_actions_are_unchanged_on_disrupted() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 15, Side::Pc), &[8, 10, 15, 3]);
    assert_eq!(encounter.start_turn().expect("start"), id(2));
    encounter.spend_actions(id(2), 2).expect("two spent");
    assert_eq!(actions(&encounter, 2), 1);
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(10, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("disrupted");
    assert_eq!(outcome.disruption, Disruption::Disrupted);
    assert_eq!(actions(&encounter, 2), 1);
    assert_ne!(actions(&encounter, 2), 2);
    assert_ne!(actions(&encounter, 2), 0);
    assert_eq!(encounter.turn_open(), Some(id(2)));
}

/// Player Core, "Step 2: Act".
#[test]
fn a_dying_creature_cannot_react() {
    let reactor = with_current(with_reach(pc(1, 0, 20, 18), 5), 0);
    let (mut encounter, mut dice) = begin(
        vec![reactor, combatant(2, Side::Pc, 0, 20, 18, false)],
        &[10, 8, 10],
    );
    encounter.set_vital_for_test(id(1), Vital::Dying(DyingValue::new(1).expect("dying")));
    assert!(reaction(&encounter, 1));
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("dying");
    assert_eq!(error, ReactiveError::NotConscious);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Unconscious". Player Core, "Step 2: Act".
#[test]
fn an_unconscious_creature_cannot_react() {
    let reactor = with_current(with_reach(pc(1, 0, 20, 18), 5), 0);
    let (mut encounter, mut dice) = begin(
        vec![reactor, combatant(2, Side::Pc, 0, 20, 18, false)],
        &[10, 8, 10],
    );
    encounter.set_vital_for_test(id(1), Vital::Unconscious);
    assert!(reaction(&encounter, 1));
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("unconscious");
    assert_eq!(error, ReactiveError::NotConscious);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Step 2: Act".
#[test]
fn a_dead_creature_cannot_react() {
    let (mut encounter, mut dice) = begin(
        vec![
            with_reach(pc(1, 0, 20, 18), 5),
            combatant(2, Side::Pc, 0, 20, 18, false),
        ],
        &[10, 8, 10],
    );
    assert!(reaction(&encounter, 1));
    encounter.set_vital_for_test(id(1), Vital::Dead);
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("dead");
    assert_eq!(error, ReactiveError::NotConscious);
    assert_ne!(error, ReactiveError::NoReaction);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Step 2: Act". Player Core, "Reactive Strike".
#[test]
fn not_conscious_wins_over_unavailable() {
    let reactor = with_current(with_reach(pc(1, 0, 20, 18), 5), 0);
    let (mut encounter, mut dice) = begin(
        vec![reactor, combatant(2, Side::Pc, 0, 20, 18, false)],
        &[10, 8, 10],
    );
    encounter.set_vital_for_test(id(1), Vital::Dying(DyingValue::new(1).expect("dying")));
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Unavailable,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("dying");
    assert_eq!(error, ReactiveError::NotConscious);
    assert_ne!(error, ReactiveError::Unavailable);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Step 2: Act".
#[test]
fn a_dying_self_target_is_not_conscious() {
    let reactor = with_current(with_reach(pc(1, 0, 20, 18), 5), 0);
    let (mut encounter, mut dice) = begin(vec![reactor], &[10, 10]);
    encounter.set_vital_for_test(id(1), Vital::Dying(DyingValue::new(1).expect("dying")));
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        1,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 0),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("self");
    assert_eq!(error, ReactiveError::NotConscious);
    assert_ne!(error, ReactiveError::SelfTarget);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Reactive Strike".
///
/// Proposed DEV-44: the caller passes Unavailable. There is no class and no feat.
#[test]
fn unavailable_spends_nothing() {
    let (mut encounter, mut dice) = begin(
        vec![
            with_reach(pc(1, 0, 20, 18), 7),
            combatant(2, Side::Pc, 0, 20, 18, false),
        ],
        &[10, 8, 10],
    );
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MOVE_ONLY),
        ActionIdentity::Other,
        Some(MoveAction::Step),
        ReactiveStrike::Unavailable,
        (0, 0),
        (0, 1),
        Size::Large,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("unavailable");
    assert_eq!(error, ReactiveError::Unavailable);
    assert_ne!(error, ReactiveError::NotATrigger);
    assert_ne!(error, ReactiveError::InvalidReach);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Strike. Player Core, "Temporary Hit Points".
#[test]
fn temporary_hit_points_are_spent_first() {
    let actor = with_temp(combatant(2, Side::Pc, 0, 20, 18, false), 5);
    let (mut encounter, mut dice) = begin(
        vec![with_reach(pc(1, 0, 20, 18), 5), actor],
        &[10, 8, 10, 3],
    );
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("temp");
    assert_eq!(delivered(&outcome).damage, 7);
    assert_eq!(temp_hp(&encounter, 2), 0);
    assert_eq!(hit_points(&encounter, 2), 18);
    assert_eq!(wounded(&encounter, 2), 0);
    assert_eq!(vital(&encounter, 2), Vital::Alive);
}

/// Player Core, "Temporary Hit Points".
///
/// Strike's `damage_removes_temporary_hit_points_then_hit_points` clears `temp_rounds`
/// when the same 5 temporary Hit Points take 7 damage. This reaction does the same.
#[test]
fn a_reactive_strike_clears_temp_rounds_when_temporary_hit_points_are_spent() {
    let mut actor = with_temp(combatant(2, Side::Pc, 0, 20, 18, false), 5);
    actor.state.temp_rounds = 3;
    let (mut encounter, mut dice) = begin(
        vec![with_reach(pc(1, 0, 20, 18), 5), actor],
        &[10, 8, 10, 3],
    );
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("temp rounds");
    assert_eq!(delivered(&outcome).damage, 7);
    assert_eq!(temp_hp(&encounter, 2), 0);
    assert_eq!(hit_points(&encounter, 2), 18);
    assert_eq!(encounter.state(id(2)).expect("state").temp_rounds, 0);
}

/// Player Core, Strike.
#[test]
fn a_failure_spends_the_reaction_and_deals_no_damage() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 9, 8]);
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(0, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("miss");
    assert_eq!(delivered(&outcome).total, 9);
    assert_eq!(delivered(&outcome).degree, DegreeOfSuccess::Failure);
    assert_eq!(delivered(&outcome).damage, 0);
    assert_eq!(outcome.disruption, Disruption::NotDisrupted);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(hit_points(&encounter, 2), 20);
    assert!(!reaction(&encounter, 1));
    assert_eq!(delivered(&outcome).map, 0);
}

/// Player Core, Strike.
#[test]
fn a_critical_failure_spends_the_reaction_and_deals_no_damage() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 5, 8]);
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(0, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("crit miss");
    assert_eq!(delivered(&outcome).degree, DegreeOfSuccess::CriticalFailure);
    assert_eq!(delivered(&outcome).damage, 0);
    assert_eq!(outcome.disruption, Disruption::NotDisrupted);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(hit_points(&encounter, 2), 20);
    assert!(!reaction(&encounter, 1));
}

/// Player Core, "Getting Knocked Out".
#[test]
fn a_critical_hit_drops_a_dying_rules_target_to_dying_2() {
    let actor = with_current(combatant(2, Side::Pc, 0, 20, 15, true), 5);
    let (mut encounter, mut dice) = begin(
        vec![with_reach(pc(1, 0, 20, 15), 5), actor],
        &[10, 8, 15, 3],
    );
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(10, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("knockout");
    assert_eq!(delivered(&outcome).damage, 14);
    assert_eq!(hit_points(&encounter, 2), 0);
    assert_eq!(
        vital(&encounter, 2),
        Vital::Dying(DyingValue::new(2).expect("dying 2"))
    );
    assert_eq!(wounded(&encounter, 2), 0);
    assert_eq!(
        take_report(&mut encounter, 2),
        Some(InitiativeMove::Pending)
    );
    assert!(!reaction(&encounter, 1));
}

/// Player Core, "Shield Block".
#[test]
fn a_reactive_strike_does_not_reduce_a_raised_shield() {
    let shield = ShieldTemplate {
        hardness: 5,
        hit_points: 20,
        broken_threshold: 10,
        shield_ac_bonus: 2,
    };
    let mut actor = participant(2, Side::Pc, creature(20, 18, 0, false, Some(shield)));
    shield::raise_shield(&mut actor.state, &actor.creature).expect("raise");
    let (mut encounter, mut dice) = begin(
        vec![with_reach(pc(1, 0, 20, 18), 5), actor],
        &[10, 8, 10, 3],
    );
    assert!(reaction(&encounter, 2));
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect("shield");
    assert_eq!(delivered(&outcome).damage, 7);
    assert_eq!(hit_points(&encounter, 2), 13);
    assert_eq!(
        encounter.state(id(2)).expect("state").shield_hit_points,
        Some(20)
    );
    assert!(reaction(&encounter, 2));
}

/// Player Core, "Reactive Strike".
#[test]
fn an_unconscious_target_returns_targetnotconscious() {
    let actor = with_current(combatant(2, Side::Pc, 0, 20, 18, false), 0);
    let (mut encounter, mut dice) =
        begin(vec![with_reach(pc(1, 0, 20, 18), 5), actor], &[10, 8, 10]);
    encounter.set_vital_for_test(id(2), Vital::Unconscious);
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("unconscious target");
    assert_eq!(error, ReactiveError::TargetNotConscious);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Reactive Strike".
#[test]
fn a_dying_target_in_reach_is_targetnotconscious() {
    let actor = with_current(combatant(2, Side::Pc, 0, 20, 18, true), 0);
    let (mut encounter, mut dice) =
        begin(vec![with_reach(pc(1, 0, 20, 18), 5), actor], &[10, 8, 10]);
    encounter.set_vital_for_test(id(2), Vital::Dying(DyingValue::new(1).expect("dying")));
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("dying target");
    assert_eq!(error, ReactiveError::TargetNotConscious);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Reactive Strike".
#[test]
fn a_dead_target_in_reach_is_targetnotconscious() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
    encounter.spend_reaction(id(1)).expect("spend");
    encounter.set_vital_for_test(id(2), Vital::Dead);
    assert!(!reaction(&encounter, 1));
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("dead target");
    assert_eq!(error, ReactiveError::TargetNotConscious);
    assert_ne!(error, ReactiveError::NoReaction);
    assert!(!reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Reactive Strike".
#[test]
fn a_reactive_strike_on_oneself_rolls_nothing() {
    let (mut encounter, mut dice) = begin(vec![with_reach(pc(1, 0, 20, 18), 5)], &[10, 10]);
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        1,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 0),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("self");
    assert_eq!(error, ReactiveError::SelfTarget);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Reactive Strike".
#[test]
fn an_unknown_reactor_rolls_nothing() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
    let ready = reaction(&encounter, 2);
    let error = react(
        &mut encounter,
        &mut dice,
        9,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("unknown reactor");
    assert_eq!(error, ReactiveError::UnknownCombatant);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(reaction(&encounter, 2), ready);
}

/// Player Core, "Reactive Strike".
#[test]
fn an_unknown_actor_rolls_nothing() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        9,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("unknown actor");
    assert_eq!(error, ReactiveError::UnknownCombatant);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Reactive Strike". Player Core, "Range and Reach".
#[test]
fn a_dead_actor_out_of_reach_is_out_of_reach() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
    encounter.set_vital_for_test(id(2), Vital::Dead);
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 2),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("out of reach");
    assert_eq!(error, ReactiveError::OutOfReach);
    assert_ne!(error, ReactiveError::TargetNotConscious);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Reactive Strike". Player Core, Penalties.
#[test]
fn an_attack_bonus_that_does_not_fit_spends_nothing() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(2_147_483_647, Agility::NotAgile, 1, 8, 4),
        &[],
    )
    .expect_err("overflow");
    assert_eq!(
        error,
        ReactiveError::AttackOverflow(ModifierError::Overflow)
    );
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Reactive Strike".
#[test]
fn an_attack_bonus_of_2147483627_and_a_natural_20_fit() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 20]);
    let outcome = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(2_147_483_627, Agility::NotAgile, 0, 8, 0),
        &[],
    )
    .expect("fits");
    assert_eq!(delivered(&outcome).total, 2_147_483_647);
    assert_eq!(delivered(&outcome).degree, DegreeOfSuccess::CriticalSuccess);
    assert!(!reaction(&encounter, 1));
}

/// Player Core, "Reactive Strike". Player Core, "Damage Rolls".
#[test]
fn a_damage_total_that_does_not_fit_spends_nothing() {
    let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 1, 4]);
    let error = react(
        &mut encounter,
        &mut dice,
        1,
        2,
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        (0, 0),
        (0, 1),
        Size::Medium,
        Size::Medium,
        profile(8, Agility::NotAgile, 1, 8, 2_147_483_647),
        &[],
    )
    .expect_err("damage overflow");
    assert_eq!(
        error,
        ReactiveError::DamageOverflow(ModifierError::Overflow)
    );
    assert_eq!(dice.remaining(), 1);
    assert!(reaction(&encounter, 1));
    assert_eq!(hit_points(&encounter, 2), 20);
}

/// Player Core, "Reactive Strike". Player Core, "Damage Rolls". Player Core pg. 442, Concealed.
#[test]
fn a_concealed_damage_total_that_does_not_fit_consumes_three_dice() {
    let detections = relate(
        Detections::new(),
        1,
        2,
        Detection::Observed,
        Concealment::Concealed,
        None,
    );
    let (mut encounter, mut dice) =
        begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 5, 10, 1, 4]);
    let error = react_seen(
        &mut encounter,
        &mut dice,
        &detections,
        profile(8, Agility::NotAgile, 1, 8, 2_147_483_647),
    )
    .expect_err("damage overflow");
    assert_eq!(
        error,
        ReactiveError::DamageOverflow(ModifierError::Overflow)
    );
    assert_eq!(dice.remaining(), 1);
    assert_eq!(dice.d20(), 4);
    assert!(reaction(&encounter, 1));
    assert_eq!(hit_points(&encounter, 2), 20);
}
