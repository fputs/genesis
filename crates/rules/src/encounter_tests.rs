use super::*;
use crate::check::DegreeOfSuccess;
use crate::combatant::{
    BuildParts, CharacterBuild, CombatantId, DieCount, HpError, MeleeStrike, ReachFeet,
    ShieldTemplate, Vital, derive, new_combatant_state,
};
use crate::dice::{Faces, ScriptedDice};
use crate::modifiers::{BonusType, Modifier, ModifierError, PenaltyType, SourceId, Statistic};

fn id(value: u32) -> CombatantId {
    CombatantId(value)
}

fn faces(count: u8) -> Faces {
    Faces::new(count).expect("faces")
}

fn creature(
    hp: i32,
    ac: i32,
    perception: i32,
    dying: bool,
    shield: Option<ShieldTemplate>,
) -> Creature {
    let parts = BuildParts {
        hit_points: hp,
        armor_class: ac,
        perception,
        speed_feet: 25,
        strike: MeleeStrike {
            attack_bonus: 0,
            damage_dice: DieCount::new(1).expect("dice"),
            damage_faces: faces(8),
            damage_bonus: 0,
            reach_feet: ReachFeet::new(5).expect("reach"),
            agile: false,
        },
        shield,
        stealth_dc: None,
        aid_modifier: None,
        uses_dying_rules: Some(dying),
    };
    derive(&CharacterBuild::try_new(parts).expect("build"))
}

fn participant(value: u32, side: Side, creature: Creature) -> Participant {
    let combatant = id(value);
    let state = new_combatant_state(combatant, &creature);
    Participant {
        id: combatant,
        side,
        creature,
        state,
    }
}

fn combatant(
    value: u32,
    side: Side,
    perception: i32,
    hp: i32,
    ac: i32,
    dying: bool,
) -> Participant {
    participant(value, side, creature(hp, ac, perception, dying, None))
}

fn pc(value: u32, perception: i32, hp: i32, ac: i32) -> Participant {
    combatant(value, Side::Pc, perception, hp, ac, false)
}

fn enemy(value: u32, perception: i32, hp: i32, ac: i32) -> Participant {
    combatant(value, Side::Enemy, perception, hp, ac, false)
}

fn dead(mut participant: Participant) -> Participant {
    participant.state.hit_points = 0;
    participant.state.vital = Vital::Dead;
    participant
}

fn with_temp(mut participant: Participant, temp: i32) -> Participant {
    participant.state.temp_hit_points = temp;
    participant
}

fn profile(attack: i32, agility: Agility, dice: u8, face: u8, bonus: i32) -> StrikeProfile {
    StrikeProfile {
        attack_bonus: attack,
        agility,
        damage_dice: DieCount::new(i32::from(dice)).expect("dice"),
        damage_faces: faces(face),
        damage_bonus: bonus,
    }
}

fn begin(participants: Vec<Participant>, rolls: &[u8]) -> (Encounter, ScriptedDice) {
    let mut dice = ScriptedDice::new(rolls.iter().copied());
    let encounter = begin_encounter(participants, &mut dice).expect("encounter");
    (encounter, dice)
}

fn actions(encounter: &Encounter, value: u32) -> u8 {
    encounter.state(id(value)).expect("state").actions_remaining
}

fn reaction(encounter: &Encounter, value: u32) -> bool {
    encounter
        .state(id(value))
        .expect("state")
        .reaction_available()
}

fn hit_points(encounter: &Encounter, value: u32) -> i32 {
    encounter.state(id(value)).expect("state").hit_points
}

fn temp_hp(encounter: &Encounter, value: u32) -> i32 {
    encounter.state(id(value)).expect("state").temp_hit_points
}

fn wounded(encounter: &Encounter, value: u32) -> i32 {
    encounter.state(id(value)).expect("state").wounded
}

fn vital(encounter: &Encounter, value: u32) -> Vital {
    encounter.state(id(value)).expect("state").vital
}

fn initiative(encounter: &Encounter, value: u32) -> i32 {
    encounter.initiative_total(id(value)).expect("initiative")
}

fn penalty(ty: PenaltyType, value: i32, source: u32, statistic: Statistic) -> Modifier {
    Modifier::penalty(ty, value, SourceId(source), source, None, statistic).expect("penalty")
}

fn bonus(ty: BonusType, value: i32, source: u32, statistic: Statistic) -> Modifier {
    Modifier::bonus(ty, value, SourceId(source), source, None, statistic).expect("bonus")
}

fn order_of(ids: &[u32]) -> Vec<CombatantId> {
    ids.iter().copied().map(id).collect()
}

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
        let outcome = encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 1, 8, 4),
                &[],
                &mut dice,
            )
            .expect("strike");
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
    encounter.end_turn().expect("end");
    encounter.start_turn().expect("other");
    encounter.end_turn().expect("end other");
    assert_eq!(encounter.start_turn().expect("next"), id(1));
    assert_eq!(actions(&encounter, 1), 3);
}

/// Player Core, "Step 1: Start Your Turn". Player Core, "Step 2: Act".
#[test]
fn a_spent_reaction_is_ready_again_next_turn() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
    encounter.start_turn().expect("start");
    encounter.spend_reaction(id(1)).expect("spend");
    encounter.end_turn().expect("end");
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
    encounter.end_turn().expect("end");
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
            &mut dice,
        )
        .expect("knockout");
    assert_eq!(hit_points(&encounter, 2), 0);
    assert_ne!(vital(&encounter, 2), Vital::Alive);
    assert_ne!(vital(&encounter, 2), Vital::Dead);
    encounter.end_turn().expect("end");
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
                &mut dice,
            )
            .expect_err("strike"),
        EncounterError::NotConscious
    );
    assert!(!reaction(&encounter, 2));
    assert_eq!(actions(&encounter, 2), 0);
    assert_eq!(dice.remaining(), 1);
    encounter.end_turn().expect("close");
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
    encounter.end_turn().expect("first end");
    assert_eq!(
        encounter.end_turn().expect_err("second end"),
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
    let (mut encounter, dice) = begin(vec![pc(1, 0, 12, 10)], &[10, 8]);
    encounter.start_turn().expect("start");
    encounter.spend_actions(id(1), 1).expect("spend");
    assert_eq!(actions(&encounter, 1), 2);
    assert!(reaction(&encounter, 1));
    encounter.end_turn().expect("end");
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
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10), pc(2, 0, 20, 10)], &[10, 8]);
    assert_eq!(
        encounter.end_turn().expect_err("end"),
        EncounterError::TurnNotOpen
    );
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(actions(&encounter, 2), 0);
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
    encounter.end_turn().expect("end");
    assert_eq!(
        encounter.end_turn().expect_err("second"),
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
    encounter.end_turn().expect("end one");
    assert_eq!(encounter.start_turn().expect("two"), id(2));
    encounter.end_turn().expect("end two");
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
    encounter.end_turn().expect("end one");
    assert_eq!(encounter.start_turn().expect("two"), id(2));
    encounter.end_turn().expect("end two");
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
    encounter.end_turn().expect("end one");
    assert_eq!(encounter.start_turn().expect("two"), id(2));
    encounter.set_round_for_test(u32::MAX);
    assert_eq!(
        encounter.end_turn().expect_err("overflow"),
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
    encounter.end_turn().expect("end");
    assert_eq!(encounter.start_turn().expect("next"), id(3));
    assert_eq!(encounter.round(), 1);
    assert_eq!(actions(&encounter, 2), 0);
}

/// Player Core, "Step 1: Start Your Turn".
#[test]
fn the_only_living_combatant_starts_the_next_round() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10), dead(pc(2, 0, 20, 10))], &[10, 8]);
    assert_eq!(encounter.order(), order_of(&[1, 2]).as_slice());
    assert_eq!(encounter.start_turn().expect("first"), id(1));
    encounter.end_turn().expect("end");
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

/// Player Core, Strike.
///
/// Reach substitute: rule, a melee Strike targets a creature within reach (Player Core pg. 418,
/// Strike); why not, reach, flanking, and prone are add-positioning; substitute, the caller
/// names the target and this module stores no squares; intent, Strike is testable without the grid.
#[test]
fn the_first_strike_totals_the_d20_plus_the_attack_bonus() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 10]);
    assert_eq!(encounter.start_turn().expect("start"), id(1));
    let outcome = encounter
        .strike(
            id(1),
            id(2),
            &profile(8, Agility::NotAgile, 0, 8, 0),
            &[],
            &mut dice,
        )
        .expect("strike");
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
            &mut dice,
        )
        .expect("knockout");
    assert_ne!(vital(&encounter, 2), Vital::Alive);
    assert_ne!(vital(&encounter, 2), Vital::Dead);
    encounter.end_turn().expect("end");
    assert_eq!(encounter.start_turn().expect("empty"), id(2));
    encounter.end_turn().expect("close empty");
    assert_eq!(encounter.start_turn().expect("again"), id(1));
    let outcome = encounter
        .strike(
            id(1),
            id(2),
            &profile(8, Agility::NotAgile, 0, 8, 0),
            &[],
            &mut dice,
        )
        .expect("second");
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
    let outcome = encounter
        .strike(
            id(1),
            id(2),
            &profile(0, Agility::NotAgile, 0, 8, 0),
            &[modifier],
            &mut dice,
        )
        .expect("strike");
    assert_eq!(outcome.total, 10);
    assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
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
            &mut dice,
        )
        .expect("first");
    let modifier = penalty(PenaltyType::Circumstance, -2, 1, Statistic::Attack);
    let outcome = encounter
        .strike(
            id(1),
            id(2),
            &profile(9, Agility::NotAgile, 0, 8, 0),
            &[modifier],
            &mut dice,
        )
        .expect("second");
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
    let outcome = encounter
        .strike(
            id(1),
            id(2),
            &profile(0, Agility::NotAgile, 0, 8, 0),
            &modifiers,
            &mut dice,
        )
        .expect("strike");
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
    let outcome = encounter
        .strike(
            id(1),
            id(2),
            &profile(0, Agility::NotAgile, 0, 8, 0),
            &modifiers,
            &mut dice,
        )
        .expect("strike");
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
    let outcome = encounter
        .strike(
            id(1),
            id(2),
            &profile(8, Agility::NotAgile, 0, 8, 0),
            &modifiers,
            &mut dice,
        )
        .expect("strike");
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
            &mut dice,
        )
        .expect("first");
    let modifier = penalty(PenaltyType::Untyped, -5, 1, Statistic::Attack);
    let outcome = encounter
        .strike(
            id(1),
            id(2),
            &profile(0, Agility::NotAgile, 0, 8, 0),
            &[modifier],
            &mut dice,
        )
        .expect("second");
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
            &mut dice,
        )
        .expect("first");
    let outcome = encounter
        .strike(
            id(1),
            id(2),
            &profile(8, Agility::Agile, 0, 8, 0),
            &[],
            &mut dice,
        )
        .expect("second");
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
        .strike(id(1), id(2), &weapon, &[], &mut dice)
        .expect("first");
    encounter
        .strike(id(1), id(2), &weapon, &[], &mut dice)
        .expect("second");
    let outcome = encounter
        .strike(id(1), id(2), &weapon, &[], &mut dice)
        .expect("third");
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
                &mut dice,
            )
            .expect_err("early"),
        EncounterError::TurnNotOpen
    );
    assert_eq!(dice.remaining(), 1);
    encounter.start_turn().expect("start");
    let outcome = encounter
        .strike(
            id(1),
            id(2),
            &profile(8, Agility::NotAgile, 0, 8, 0),
            &[],
            &mut dice,
        )
        .expect("strike");
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
        .strike(id(1), id(2), &weapon, &[], &mut dice)
        .expect("first");
    encounter
        .strike(id(1), id(2), &weapon, &[], &mut dice)
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

fn strike_damage(
    attack: i32,
    armor_class: i32,
    dice_count: u8,
    face: u8,
    bonus: i32,
    rolls: &[u8],
    modifiers: &[Modifier],
) -> (StrikeOutcome, usize) {
    let mut prefix = vec![14, 10];
    prefix.extend_from_slice(rolls);
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 40, armor_class), pc(2, 0, 40, armor_class)],
        &prefix,
    );
    encounter.start_turn().expect("start");
    let outcome = encounter
        .strike(
            id(1),
            id(2),
            &profile(attack, Agility::NotAgile, dice_count, face, bonus),
            modifiers,
            &mut dice,
        )
        .expect("strike");
    (outcome, dice.remaining())
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
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 9, 8]);
    encounter.start_turn().expect("start");
    let result = encounter.strike(
        id(1),
        id(2),
        &profile(0, Agility::NotAgile, 1, 8, 4),
        &[],
        &mut dice,
    );
    assert_ne!(result, Err(EncounterError::HitPoints(HpError::NotAHit)));
    let outcome = result.expect("short-circuit");
    assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
    assert_eq!(outcome.damage, 0);
    assert_eq!(actions(&encounter, 1), 2);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(hit_points(&encounter, 2), 20);
    assert_eq!(temp_hp(&encounter, 2), 0);
    assert_eq!(wounded(&encounter, 2), 0);
    assert_eq!(vital(&encounter, 2), Vital::Alive);
}

/// Player Core, Strike.
///
/// A CriticalFailure spends the action and deals no damage. `HpError::NotAHit` stays off this path.
#[test]
fn a_critical_failure_short_circuits_with_no_damage() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 5, 8]);
    encounter.start_turn().expect("start");
    let result = encounter.strike(
        id(1),
        id(2),
        &profile(0, Agility::NotAgile, 1, 8, 4),
        &[],
        &mut dice,
    );
    assert_ne!(result, Err(EncounterError::HitPoints(HpError::NotAHit)));
    let outcome = result.expect("short-circuit");
    assert_eq!(outcome.degree, DegreeOfSuccess::CriticalFailure);
    assert_eq!(outcome.damage, 0);
    assert_eq!(actions(&encounter, 1), 2);
    assert_eq!(dice.remaining(), 1);
    assert_eq!(hit_points(&encounter, 2), 20);
    assert_eq!(temp_hp(&encounter, 2), 0);
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

/// Player Core, "Temporary Hit Points". Ruling R-06.
#[test]
fn damage_removes_temporary_hit_points_then_hit_points() {
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 11), with_temp(pc(2, 0, 20, 11), 5)],
        &[14, 10, 11, 3],
    );
    encounter.start_turn().expect("start");
    encounter
        .strike(
            id(1),
            id(2),
            &profile(0, Agility::NotAgile, 1, 8, 4),
            &[],
            &mut dice,
        )
        .expect("strike");
    assert_eq!(temp_hp(&encounter, 2), 0);
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
            &mut dice,
        )
        .expect("strike");
    assert_eq!(hit_points(&encounter, 2), 0);
    assert_eq!(vital(&encounter, 2), Vital::Dead);
    assert_eq!(wounded(&encounter, 2), 0);
    assert_eq!(encounter.creature(id(2)).expect("creature").hit_points, 9);
}

/// Player Core, "Getting Knocked Out".
#[test]
fn a_strike_does_not_change_the_shield() {
    let shield = ShieldTemplate {
        hardness: 0,
        hit_points: 20,
        broken_threshold: 0,
        shield_ac_bonus: 0,
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
