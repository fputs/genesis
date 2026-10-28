use super::*;

/// Player Core, "Step 1: Roll Initiative".
///
/// Arithmetic: 10 + 5 = 15, 14 + 0 = 14, then 12 against Armor Class 11, then 3 + 4 = 7.
#[test]
fn the_same_encounter_twice_matches() {
    let run = |script: [u8; 4]| {
        let (mut encounter, mut dice) = begin(
            vec![
                combatant(1, Side::Pc, 5, 20, 11, false),
                combatant(2, Side::Pc, 0, 20, 11, false),
            ],
            &script,
        );
        assert_eq!(initiative(&encounter, 1), 15);
        assert_eq!(initiative(&encounter, 2), 14);
        let order = encounter.order().to_vec();
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        let outcome = rolled(
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
                .expect("strike"),
        );
        (order, outcome.total, outcome.damage, dice.remaining())
    };
    let first = run([10, 14, 12, 3]);
    let second = run([10, 14, 12, 3]);
    assert_eq!(first, second);
    assert_eq!(first.0, order_of(&[1, 2]));
    assert_eq!(first.1, 12);
    assert_eq!(first.2, 7);
    assert_eq!(first.3, 0);
}

/// Player Core, "Step 1: Roll Initiative".
#[test]
fn perception_is_added_to_the_d20() {
    let (encounter, dice) = begin(vec![pc(1, 5, 20, 10)], &[10]);
    assert_eq!(initiative(&encounter, 1), 15);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, "Step 1: Roll Initiative".
#[test]
fn a_natural_20_does_not_add_10() {
    let (encounter, _) = begin(vec![pc(1, 0, 20, 10), pc(2, 5, 20, 10)], &[20, 14]);
    assert_eq!(initiative(&encounter, 1), 20);
    assert_eq!(initiative(&encounter, 2), 19);
    assert_eq!(encounter.order(), order_of(&[1, 2]).as_slice());
}

/// Player Core, "Step 1: Roll Initiative".
#[test]
fn a_natural_1_does_not_subtract_10() {
    let (encounter, _) = begin(vec![pc(1, 19, 20, 10)], &[1]);
    assert_eq!(initiative(&encounter, 1), 20);
}

/// Player Core, "Step 1: Roll Initiative".
#[test]
fn a_negative_perception_modifier_is_added() {
    let (encounter, _) = begin(vec![pc(1, -3, 20, 10)], &[8]);
    assert_eq!(initiative(&encounter, 1), 5);
}

/// Player Core, "Step 1: Roll Initiative".
#[test]
fn roster_order_does_not_choose_the_die() {
    let (encounter, _) = begin(vec![pc(5, 0, 20, 10), pc(1, 5, 20, 10)], &[10, 14]);
    assert_eq!(initiative(&encounter, 1), 15);
    assert_eq!(initiative(&encounter, 5), 14);
}

/// Player Core, "Step 1: Roll Initiative".
#[test]
fn a_duplicate_id_rolls_nothing() {
    let mut dice = ScriptedDice::new([8]);
    let error = begin_encounter(vec![pc(1, 0, 20, 10), pc(1, 0, 20, 10)], &mut dice)
        .expect_err("duplicate");
    assert_eq!(error, EncounterError::DuplicateCombatant);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Step 1: Roll Initiative".
#[test]
fn an_empty_roster_rolls_nothing() {
    let mut dice = ScriptedDice::new([8]);
    let error = begin_encounter(vec![], &mut dice).expect_err("empty");
    assert_eq!(error, EncounterError::EmptyRoster);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Step 1: Roll Initiative".
#[test]
fn perception_2147483627_and_a_natural_20_fit() {
    let (encounter, dice) = begin(vec![pc(1, 2_147_483_627, 20, 10)], &[20]);
    assert_eq!(initiative(&encounter, 1), 2_147_483_647);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, "Step 1: Roll Initiative".
#[test]
fn perception_2147483628_rolls_nothing() {
    let mut dice = ScriptedDice::new([20]);
    let error = begin_encounter(vec![pc(1, 2_147_483_628, 20, 10)], &mut dice).expect_err("range");
    assert_eq!(error, EncounterError::PerceptionOutOfRange);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Step 1: Roll Initiative".
#[test]
fn perception_at_i32min_is_added() {
    let (encounter, dice) = begin(vec![pc(1, i32::MIN, 20, 10)], &[1]);
    assert_eq!(initiative(&encounter, 1), -2_147_483_647);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, "Step 1: Roll Initiative".
#[test]
fn a_higher_pc_total_acts_before_a_lower_enemy_total() {
    let (encounter, _) = begin(vec![pc(1, 8, 20, 10), enemy(2, 5, 20, 10)], &[10, 10]);
    assert_eq!(initiative(&encounter, 1), 18);
    assert_eq!(initiative(&encounter, 2), 15);
    assert_eq!(encounter.order(), order_of(&[1, 2]).as_slice());
}

/// Player Core, "Step 1: Roll Initiative".
///
/// DEV-05 substitute: rule, tied PCs choose who goes first and the GM orders tied foes
/// (Player Core pg. 435, Rules 2423); why not, there are no players and no GM; substitute,
/// the lower CombatantId acts earlier on a same-Side tie; intent, the same roster and dice
/// always produce the same order. A PC–enemy tie is the book, not that substitute: enemy
/// CombatantId(9) acts before PC CombatantId(1) even though 9 is the higher id.
#[test]
fn an_enemy_acts_before_a_tied_pc() {
    let (encounter, _) = begin(vec![pc(1, 0, 20, 10), enemy(9, 0, 20, 10)], &[10, 10]);
    assert_eq!(initiative(&encounter, 1), 10);
    assert_eq!(initiative(&encounter, 9), 10);
    assert_eq!(encounter.order(), order_of(&[9, 1]).as_slice());
}

/// Player Core, "Step 1: Roll Initiative".
///
/// DEV-05 substitute: same-Side ties, including two PCs, act by the lower CombatantId.
#[test]
fn two_tied_pcs_act_by_the_lower_id() {
    let (encounter, _) = begin(vec![pc(4, 0, 20, 10), pc(2, 0, 20, 10)], &[8, 8]);
    assert_eq!(initiative(&encounter, 4), 8);
    assert_eq!(initiative(&encounter, 2), 8);
    assert_eq!(encounter.order(), order_of(&[2, 4]).as_slice());
}

/// Player Core, "Step 1: Roll Initiative".
///
/// DEV-05 substitute: same-Side ties, including two enemies, act by the lower CombatantId.
#[test]
fn two_tied_enemies_act_by_the_lower_id() {
    let (encounter, _) = begin(vec![enemy(8, 0, 20, 10), enemy(3, 0, 20, 10)], &[7, 7]);
    assert_eq!(initiative(&encounter, 8), 7);
    assert_eq!(initiative(&encounter, 3), 7);
    assert_eq!(encounter.order(), order_of(&[3, 8]).as_slice());
}

/// Player Core, "Step 1: Roll Initiative".
///
/// DEV-05 substitute: enemies share a Side, so the lower id is earlier, and both enemies
/// are before the tied PC.
#[test]
fn a_three_way_tie_puts_both_enemies_first() {
    let (encounter, _) = begin(
        vec![pc(1, 0, 20, 10), enemy(2, 0, 20, 10), enemy(5, 0, 20, 10)],
        &[12, 12, 12],
    );
    assert_eq!(initiative(&encounter, 1), 12);
    assert_eq!(initiative(&encounter, 2), 12);
    assert_eq!(initiative(&encounter, 5), 12);
    assert_eq!(encounter.order(), order_of(&[2, 5, 1]).as_slice());
}

/// Player Core, "Step 1: Roll Initiative".
///
/// DEV-05 substitute: each same-Side tie uses the lower CombatantId, after the higher total
/// and after an enemy-before-PC tie.
#[test]
fn six_combatants_all_appear_in_the_order() {
    let (encounter, _) = begin(
        vec![
            pc(0, 0, 20, 10),
            enemy(1, 0, 20, 10),
            pc(2, 0, 20, 10),
            enemy(3, 0, 20, 10),
            pc(4, 0, 20, 10),
            enemy(5, 0, 20, 10),
        ],
        &[10, 10, 12, 12, 8, 8],
    );
    assert_eq!(encounter.order(), order_of(&[3, 2, 1, 0, 5, 4]).as_slice());
}

/// Player Core, "Step 1: Start Your Turn".
#[test]
fn an_alive_start_rolls_nothing() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 12, 10)], &[10, 8]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    assert_eq!(dice.remaining(), 1);
    assert_eq!(hit_points(&encounter, 1), 12);
    assert_eq!(wounded(&encounter, 1), 0);
    assert_eq!(actions(&encounter, 1), 3);
}

/// Player Core, "Step 1: Start Your Turn".
#[test]
fn start_sets_3_actions_and_a_reaction() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    assert_eq!(initiative(&encounter, 1), 10);
    assert_eq!(encounter.round(), 1);
    assert_eq!(actions(&encounter, 1), 3);
    assert!(reaction(&encounter, 1));
}

/// Player Core, "Step 1: Start Your Turn".
#[test]
fn a_second_start_does_not_add_actions() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
    encounter.start_turn().expect("start");
    assert_eq!(
        encounter.start_turn().expect_err("second"),
        EncounterError::TurnInProgress
    );
    assert_eq!(actions(&encounter, 1), 3);
    assert!(reaction(&encounter, 1));
}

/// Player Core, "Step 1: Start Your Turn".
#[test]
fn the_next_start_sets_3_rather_than_adding_leftovers() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10), pc(2, 0, 20, 10)], &[10, 8]);
    assert_eq!(encounter.start_turn().expect("first"), id(1));
    encounter.spend_actions(id(1), 1).expect("spend");
    assert_eq!(actions(&encounter, 1), 2);
    encounter
        .end_turn(&mut crate::dice::ScriptedDice::new([]))
        .expect("end");
    encounter.start_turn().expect("other");
    encounter
        .end_turn(&mut crate::dice::ScriptedDice::new([]))
        .expect("end other");
    assert_eq!(encounter.start_turn().expect("next"), id(1));
    assert_eq!(actions(&encounter, 1), 3);
}

/// Player Core, "Step 1: Start Your Turn". Player Core, "Step 2: Act".
#[test]
fn a_spent_reaction_is_ready_again_next_turn() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
    encounter.start_turn().expect("start");
    encounter.spend_reaction(id(1)).expect("spend");
    encounter
        .end_turn(&mut crate::dice::ScriptedDice::new([]))
        .expect("end");
    encounter.start_turn().expect("next");
    assert!(reaction(&encounter, 1));
    encounter.spend_reaction(id(1)).expect("first");
    assert_eq!(
        encounter.spend_reaction(id(1)).expect_err("second"),
        EncounterError::NoReaction
    );
}

/// Player Core, "Step 1: Start Your Turn". Player Core, "Step 2: Act".
#[test]
fn an_unused_reaction_is_still_one_reaction() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
    encounter.start_turn().expect("start");
    assert!(reaction(&encounter, 1));
    encounter
        .end_turn(&mut crate::dice::ScriptedDice::new([]))
        .expect("end");
    encounter.start_turn().expect("next");
    encounter.spend_reaction(id(1)).expect("first");
    assert_eq!(
        encounter.spend_reaction(id(1)).expect_err("second"),
        EncounterError::NoReaction
    );
}

/// Player Core, "Reactions in Encounters".
///
/// DEV-06 substitute: rule, the GM decides whether reactions work before the first turn
/// (Player Core pg. 436, Rules 2432); why not, there is no GM; substitute, an Alive
/// combatant's reaction is available at encounter start; intent, one fixed answer.
#[test]
fn a_reaction_before_the_first_turn_spends_once() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 10), dead(pc(2, 0, 20, 10))], &[10, 8, 8]);
    assert_eq!(actions(&encounter, 1), 0);
    assert!(reaction(&encounter, 1));
    assert!(!reaction(&encounter, 2));
    encounter.spend_reaction(id(1)).expect("first");
    assert_eq!(dice.remaining(), 1);
    assert_eq!(
        encounter.spend_reaction(id(1)).expect_err("second"),
        EncounterError::NoReaction
    );
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Step 2: Act".
#[test]
fn a_not_alive_turn_opens_with_no_actions() {
    let (mut encounter, mut dice) = begin(
        vec![
            combatant(1, Side::Pc, 0, 20, 18, false),
            combatant(2, Side::Pc, 0, 9, 11, true),
        ],
        &[10, 8, 11, 8, 4],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
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
        .expect("knockout");
    assert_eq!(hit_points(&encounter, 2), 0);
    assert_ne!(vital(&encounter, 2), Vital::Alive);
    assert_ne!(vital(&encounter, 2), Vital::Dead);
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("next"), id(2));
    assert_eq!(actions(&encounter, 2), 0);
    assert!(!reaction(&encounter, 2));
    assert_eq!(
        encounter.spend_reaction(id(2)).expect_err("reaction"),
        EncounterError::NotConscious
    );
    assert_eq!(
        encounter.spend_actions(id(2), 1).expect_err("action"),
        EncounterError::NotConscious
    );
    assert_eq!(
        encounter.spend_free_action(id(2)).expect_err("free"),
        EncounterError::NotConscious
    );
    assert_eq!(
        encounter
            .strike(
                id(2),
                id(1),
                &profile(0, Agility::NotAgile, 1, 8, 0),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect_err("strike"),
        EncounterError::NotConscious
    );
    assert!(!reaction(&encounter, 2));
    assert_eq!(actions(&encounter, 2), 0);
    assert_eq!(dice.remaining(), 1);
    encounter.end_turn(&mut dice).expect("close");
    assert_eq!(encounter.turn_open(), None);
    assert_eq!(actions(&encounter, 2), 0);
}

/// Player Core, "Step 2: Act".
#[test]
fn an_active_creature_that_stops_being_alive_cannot_keep_acting() {
    let (mut encounter, mut dice) = begin(
        vec![
            combatant(1, Side::Pc, 0, 20, 18, false),
            combatant(2, Side::Pc, 0, 9, 11, true),
        ],
        &[10, 8, 11, 8, 8],
    );
    assert_eq!(encounter.start_turn().expect("start"), id(1));
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
        .expect("knockout");
    let knocked = vital(&encounter, 2);
    assert_ne!(knocked, Vital::Alive);
    assert_ne!(knocked, Vital::Dead);
    encounter.set_vital_for_test(id(1), knocked);
    assert_eq!(
        encounter.spend_actions(id(1), 1).expect_err("spend"),
        EncounterError::NotConscious
    );
    assert_eq!(actions(&encounter, 1), 2);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
    encounter.end_turn(&mut dice).expect("first end");
    assert_eq!(
        encounter.end_turn(&mut dice).expect_err("second end"),
        EncounterError::TurnNotOpen
    );
    assert_eq!(encounter.round(), 1);
    assert_eq!(encounter.start_turn().expect("next"), id(2));
}

/// Player Core, "Reactions in Encounters".
#[test]
fn a_dead_combatant_cannot_spend_a_reaction() {
    let (mut encounter, _) = begin(vec![dead(pc(1, 0, 20, 10)), pc(2, 0, 20, 10)], &[10, 8]);
    assert!(reaction(&encounter, 2));
    let dead_reaction = reaction(&encounter, 1);
    assert_eq!(
        encounter.spend_reaction(id(1)).expect_err("dead"),
        EncounterError::NotConscious
    );
    assert!(reaction(&encounter, 2));
    assert_eq!(reaction(&encounter, 1), dead_reaction);
}

/// Player Core, "Step 3: End Your Turn".
#[test]
fn ending_early_drops_leftover_actions_and_keeps_the_reaction() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 12, 10)], &[10, 8]);
    encounter.start_turn().expect("start");
    encounter.spend_actions(id(1), 1).expect("spend");
    assert_eq!(actions(&encounter, 1), 2);
    assert!(reaction(&encounter, 1));
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(actions(&encounter, 1), 0);
    assert!(reaction(&encounter, 1));
    assert_eq!(hit_points(&encounter, 1), 12);
    assert_eq!(temp_hp(&encounter, 1), 0);
    assert_eq!(wounded(&encounter, 1), 0);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Step 3: End Your Turn".
#[test]
fn end_before_a_start_returns_an_error() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 10), pc(2, 0, 20, 10)], &[10, 8, 8]);
    assert_eq!(
        encounter.end_turn(&mut dice).expect_err("end"),
        EncounterError::TurnNotOpen
    );
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(actions(&encounter, 2), 0);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Step 2: Act".
///
/// Rules 2429 ends the turn after the third action. Engineering handoff: the caller calls
/// end_turn, nothing further can be spent, and no end-of-turn effect fires before that call.
#[test]
fn spending_the_last_action_leaves_the_turn_open() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10), pc(2, 0, 20, 10)], &[10, 8]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.spend_actions(id(1), 3).expect("spend");
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(actions(&encounter, 1), 0);
    assert!(reaction(&encounter, 1));
    encounter
        .end_turn(&mut crate::dice::ScriptedDice::new([]))
        .expect("end");
    assert_eq!(
        encounter
            .end_turn(&mut crate::dice::ScriptedDice::new([]))
            .expect_err("second"),
        EncounterError::TurnNotOpen
    );
    assert_eq!(encounter.round(), 1);
    assert_eq!(encounter.start_turn().expect("next"), id(2));
}

/// Player Core, Activities. Player Core, "Step 2: Act".
#[test]
fn spending_1_leaves_2() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
    encounter.start_turn().expect("start");
    encounter.spend_actions(id(1), 1).expect("spend");
    assert_eq!(actions(&encounter, 1), 2);
    assert_eq!(encounter.turn_open(), Some(id(1)));
}

/// Player Core, Activities. Player Core, "Step 2: Act".
#[test]
fn spending_2_leaves_1_and_rolls_nothing() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 10)], &[10, 8]);
    encounter.start_turn().expect("start");
    encounter.spend_actions(id(1), 2).expect("spend");
    assert_eq!(actions(&encounter, 1), 1);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Activities. Player Core, "Step 2: Act".
#[test]
fn spending_2_with_1_left_fails() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
    encounter.start_turn().expect("start");
    encounter.spend_actions(id(1), 2).expect("spend");
    assert_eq!(
        encounter.spend_actions(id(1), 2).expect_err("short"),
        EncounterError::InsufficientActions
    );
    assert_eq!(actions(&encounter, 1), 1);
}

/// Player Core, Activities. Player Core, "Step 2: Act".
#[test]
fn a_cost_of_0_or_4_is_rejected() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
    encounter.start_turn().expect("start");
    assert_eq!(
        encounter.spend_actions(id(1), 0).expect_err("zero"),
        EncounterError::InvalidActionCost
    );
    assert_eq!(
        encounter.spend_actions(id(1), 4).expect_err("four"),
        EncounterError::InvalidActionCost
    );
    assert_eq!(actions(&encounter, 1), 3);
}

/// Player Core, Activities. Player Core, "Step 2: Act".
#[test]
fn a_free_action_leaves_the_remaining_action() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
    encounter.start_turn().expect("start");
    encounter.spend_actions(id(1), 2).expect("spend");
    assert!(reaction(&encounter, 1));
    encounter.spend_free_action(id(1)).expect("free");
    assert_eq!(actions(&encounter, 1), 1);
    assert!(reaction(&encounter, 1));
    assert_eq!(encounter.turn_open(), Some(id(1)));
}

/// Player Core, Activities. Player Core, "Step 2: Act".
#[test]
fn an_untriggered_free_action_off_turn_fails() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
    assert_eq!(
        encounter.spend_free_action(id(1)).expect_err("off"),
        EncounterError::TurnNotOpen
    );
}

/// Player Core, "Reactions in Encounters".
#[test]
fn a_reaction_during_the_turn_leaves_the_actions() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 10)], &[10, 8]);
    encounter.start_turn().expect("start");
    encounter.spend_reaction(id(1)).expect("reaction");
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert!(!reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Reactions in Encounters".
#[test]
fn another_creature_spends_a_reaction_during_the_turn() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 10), pc(2, 0, 20, 10)], &[14, 10, 8]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    encounter.spend_reaction(id(2)).expect("reaction");
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(dice.remaining(), 1);
    assert_eq!(
        encounter.spend_reaction(id(2)).expect_err("second"),
        EncounterError::NoReaction
    );
}

/// Player Core, "Step 2: Act". Player Core, "Step 1: Start Your Turn".
#[test]
fn the_second_round_starts_with_the_first_living_combatant() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10), pc(2, 0, 20, 10)], &[10, 8]);
    assert_eq!(encounter.start_turn().expect("one"), id(1));
    encounter
        .end_turn(&mut crate::dice::ScriptedDice::new([]))
        .expect("end one");
    assert_eq!(encounter.start_turn().expect("two"), id(2));
    encounter
        .end_turn(&mut crate::dice::ScriptedDice::new([]))
        .expect("end two");
    assert_eq!(encounter.round(), 2);
    assert_eq!(encounter.start_turn().expect("again"), id(1));
}

/// Player Core, "Step 2: Act". Player Core, "Step 1: Start Your Turn".
#[test]
fn after_2_ends_a_dead_3_wraps_to_1_in_round_2() {
    let (mut encounter, _) = begin(
        vec![pc(1, 0, 20, 10), pc(2, 0, 20, 10), dead(pc(3, 0, 20, 10))],
        &[12, 11, 10],
    );
    assert_eq!(encounter.order(), order_of(&[1, 2, 3]).as_slice());
    assert_eq!(encounter.start_turn().expect("one"), id(1));
    encounter
        .end_turn(&mut crate::dice::ScriptedDice::new([]))
        .expect("end one");
    assert_eq!(encounter.start_turn().expect("two"), id(2));
    encounter
        .end_turn(&mut crate::dice::ScriptedDice::new([]))
        .expect("end two");
    assert_eq!(encounter.round(), 1);
    assert_eq!(encounter.start_turn().expect("wrap"), id(1));
    assert_eq!(encounter.round(), 2);
}

/// Player Core, "Step 1: Start Your Turn".
#[test]
fn a_dead_first_combatant_starts_the_other_in_round_1() {
    let (mut encounter, _) = begin(vec![dead(pc(1, 0, 20, 10)), pc(2, 0, 20, 10)], &[10, 5]);
    assert_eq!(encounter.start_turn().expect("start"), id(2));
    assert_eq!(encounter.round(), 1);
}

/// Player Core, "Step 2: Act".
#[test]
fn a_round_at_the_u32_maximum_does_not_advance() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10), pc(2, 0, 20, 10)], &[10, 8]);
    let order = encounter.order().to_vec();
    encounter.start_turn().expect("one");
    encounter
        .end_turn(&mut crate::dice::ScriptedDice::new([]))
        .expect("end one");
    assert_eq!(encounter.start_turn().expect("two"), id(2));
    encounter.set_round_for_test(u32::MAX);
    assert_eq!(
        encounter
            .end_turn(&mut crate::dice::ScriptedDice::new([]))
            .expect_err("overflow"),
        EncounterError::RoundOverflow
    );
    assert_eq!(encounter.round(), u32::MAX);
    assert_eq!(encounter.order(), order.as_slice());
    assert_eq!(encounter.turn_open(), Some(id(2)));
}

/// Player Core, "Step 1: Start Your Turn".
#[test]
fn a_dead_combatant_in_the_middle_is_skipped() {
    let (mut encounter, _) = begin(
        vec![pc(1, 0, 20, 10), dead(pc(2, 0, 20, 10)), pc(3, 0, 20, 10)],
        &[12, 11, 10],
    );
    assert_eq!(encounter.order(), order_of(&[1, 2, 3]).as_slice());
    assert_eq!(encounter.start_turn().expect("first"), id(1));
    encounter
        .end_turn(&mut crate::dice::ScriptedDice::new([]))
        .expect("end");
    assert_eq!(encounter.start_turn().expect("next"), id(3));
    assert_eq!(encounter.round(), 1);
    assert_eq!(actions(&encounter, 2), 0);
}

/// Player Core, "Step 1: Start Your Turn".
///
/// QA probe p07. Order is dead, alive, dead, dead. After the living turn, the next start
/// skips two dead combatants and the dead combatant at the wrap, and opens that same
/// living turn again. A skip that stops after one dead returns no living combatant.
#[test]
fn two_dead_combatants_in_a_row_wrap_back_to_the_living_one() {
    let (mut encounter, _) = begin(
        vec![
            dead(pc(1, 0, 20, 10)),
            pc(2, 0, 20, 10),
            dead(pc(3, 0, 20, 10)),
            dead(pc(4, 0, 20, 10)),
        ],
        &[12, 11, 10, 9],
    );
    assert_eq!(encounter.order(), order_of(&[1, 2, 3, 4]).as_slice());
    assert_eq!(encounter.start_turn().expect("living"), id(2));
    assert_eq!(encounter.round(), 1);
    encounter
        .end_turn(&mut crate::dice::ScriptedDice::new([]))
        .expect("end");
    assert_eq!(encounter.start_turn().expect("wrap"), id(2));
    assert_eq!(encounter.round(), 2);
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(actions(&encounter, 3), 0);
    assert_eq!(actions(&encounter, 4), 0);
}

/// Player Core, "Step 1: Start Your Turn".
#[test]
fn the_only_living_combatant_starts_the_next_round() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10), dead(pc(2, 0, 20, 10))], &[10, 8]);
    assert_eq!(encounter.order(), order_of(&[1, 2]).as_slice());
    assert_eq!(encounter.start_turn().expect("first"), id(1));
    encounter
        .end_turn(&mut crate::dice::ScriptedDice::new([]))
        .expect("end");
    assert_eq!(encounter.start_turn().expect("again"), id(1));
    assert_eq!(encounter.round(), 2);
}

/// Player Core, "Step 1: Start Your Turn".
#[test]
fn no_living_combatant_returns_an_error() {
    let (mut encounter, _) = begin(
        vec![dead(pc(1, 0, 20, 10)), dead(pc(2, 0, 20, 10))],
        &[10, 8],
    );
    let order = encounter.order().to_vec();
    assert_eq!(encounter.round(), 1);
    assert_eq!(
        encounter.start_turn().expect_err("none"),
        EncounterError::NoLivingCombatant
    );
    assert_eq!(encounter.round(), 1);
    assert_eq!(encounter.order(), order.as_slice());
}
