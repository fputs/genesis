use super::{
    actions, aided, begin, creature, enemy, hit_points, id, initiative, order_of, participant, pc,
    profile, reaction, set_prepared, take_report, temp_hp, vital,
};
use crate::aid::{AidPreparation, AidRank};
use crate::check::DegreeOfSuccess;
use crate::combatant::{BuildParts, CharacterBuild, DieCount, MeleeStrike, ReachFeet, derive};
use crate::combatant::{DyingValue, ShieldTemplate, Vital};
use crate::dice::Faces;
use crate::dice::ScriptedDice;
use crate::encounter::{Agility, Encounter, EncounterError, Side, TurnSlot};
use crate::grid::{
    DiagonalCounter, FactionId, GridError, MapBounds, MoveInput, Size, Square,
    stride as grid_stride,
};
use crate::positioning::{
    ActionCost, ActionTrait, CoverBenefit, Mobility, OccupantPresence, Posture,
};
use crate::reactive::{ActionIdentity, ReactiveStrike};
use crate::ready::{
    ActivityKind, ObservedAction, ReactionField, ReactionPlace, ReadyActivity, ReadyError,
    ReadyStrike, ReadySuccess, ReadyTrigger, ResolvedReaction, StoredReady, StrikeRange,
};
use crate::shield::ShieldBlock;

fn sq(x: i32, y: i32) -> Square {
    Square { x, y }
}

fn cost(count: u8) -> ActionCost {
    ActionCost::Actions { count, traits: &[] }
}

fn manipulate() -> ActionCost {
    ActionCost::Actions {
        count: 1,
        traits: &[ActionTrait::Manipulate],
    }
}

fn moving() -> ActionCost {
    ActionCost::Actions {
        count: 1,
        traits: &[ActionTrait::Move],
    }
}

fn melee(target: u32, agility: Agility, dice: u8, bonus: i32) -> ReadyActivity {
    ReadyActivity::Strike {
        target: id(target),
        profile: profile(8, agility, dice, 8, bonus),
        range: StrikeRange::Melee,
    }
}

fn ranged(target: u32, increment_feet: i32) -> ReadyActivity {
    ReadyActivity::Strike {
        target: id(target),
        profile: profile(8, Agility::NotAgile, 0, 8, 0),
        range: StrikeRange::Ranged { increment_feet },
    }
}

fn reach(subject: u32) -> ReadyTrigger {
    ReadyTrigger::MovesWithinReach {
        subject: id(subject),
    }
}

fn begins(subject: u32, action: ObservedAction) -> ReadyTrigger {
    ReadyTrigger::Begins {
        subject: id(subject),
        action: Some(action),
    }
}

fn open(encounter: &mut Encounter, value: u32) {
    assert_eq!(encounter.start_turn().expect("start"), id(value));
}

fn close(encounter: &mut Encounter) {
    encounter.end_turn().expect("end");
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

fn ticks(
    mut participant: crate::encounter::Participant,
    temp: i32,
    rounds: u32,
) -> crate::encounter::Participant {
    participant.state.temp_hit_points = temp;
    participant.state.temp_rounds = rounds;
    participant
}

fn temp_rounds(encounter: &Encounter, value: u32) -> u32 {
    encounter.state(id(value)).expect("state").temp_rounds
}

fn slot(encounter: &Encounter, value: u32) -> TurnSlot {
    encounter.turn_slot(id(value)).expect("slot")
}

fn spot(value: u32, x: i32, y: i32, cover: Option<CoverBenefit>) -> ReactionPlace {
    ReactionPlace {
        id: id(value),
        square: sq(x, y),
        size: Size::Medium,
        cover,
    }
}

fn places(actor: u32, at: Square, cover_on_one: Option<CoverBenefit>) -> Vec<ReactionPlace> {
    (1..=8)
        .map(|value| {
            spot(
                value,
                if value == actor { at.x } else { 0 },
                if value == actor { at.y } else { 0 },
                if value == 1 { cover_on_one } else { None },
            )
        })
        .collect()
}

fn grid(places: &[ReactionPlace]) -> ReactionField<'_> {
    ReactionField {
        places,
        bounds: MapBounds {
            width: 64,
            height: 64,
        },
        terrain: &[],
        occupants: &[],
    }
}

fn ready_on(encounter: &mut Encounter, actor: u32, trigger: ReadyTrigger, activity: ReadyActivity) {
    encounter
        .ready(id(actor), trigger, cost(1), activity)
        .expect("ready");
}

fn offer(
    encounter: &mut Encounter,
    actor: u32,
    action: ObservedAction,
    from: Square,
    to: Square,
    path: &[Square],
    dice: &mut ScriptedDice,
) -> Vec<ResolvedReaction> {
    let laid = places(actor, to, None);
    let field = grid(&laid);
    encounter.resolve_reactions(
        id(actor),
        action,
        cost(1),
        from,
        to,
        path,
        &field,
        &[],
        dice,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "the dismiss tests pass the same offer arguments plus the cover the caller supplies"
)]
fn offer_cover(
    encounter: &mut Encounter,
    actor: u32,
    action: ObservedAction,
    from: Square,
    to: Square,
    path: &[Square],
    cover: CoverBenefit,
    dice: &mut ScriptedDice,
) -> Vec<ResolvedReaction> {
    let laid = places(actor, to, Some(cover));
    let field = grid(&laid);
    encounter.resolve_reactions(
        id(actor),
        action,
        cost(1),
        from,
        to,
        path,
        &field,
        &[],
        dice,
    )
}

fn slow(value: u32) -> crate::encounter::Participant {
    let parts = BuildParts {
        hit_points: 20,
        armor_class: 18,
        perception: 0,
        speed_feet: 5,
        strike: MeleeStrike {
            attack_bonus: 0,
            damage_dice: DieCount::new(1).expect("dice"),
            damage_faces: Faces::new(8).expect("faces"),
            damage_bonus: 0,
            reach_feet: ReachFeet::new(5).expect("reach"),
            agile: false,
        },
        shield: None,
        stealth_dc: None,
        aid_modifier: None,
        aid_rank: None,
        uses_dying_rules: Some(false),
        saves: None,
    };
    participant(
        value,
        Side::Pc,
        derive(&CharacterBuild::try_new(parts).expect("build")),
    )
}

fn one(resolved: &[ResolvedReaction]) -> &Result<ReadySuccess, ReadyError> {
    match resolved {
        [ResolvedReaction::Ready { result, .. }] => result,
        other => panic!("expected one ready, got {other:?}"),
    }
}

fn strike_numbers(resolved: &[ResolvedReaction]) -> ReadyStrike {
    match one(resolved) {
        Ok(ReadySuccess::Strike(strike)) => *strike,
        other => panic!("expected a strike, got {other:?}"),
    }
}

fn dying_at(value: i32) -> Vital {
    Vital::Dying(DyingValue::new(value).expect("dying"))
}

fn steel() -> ShieldTemplate {
    ShieldTemplate {
        hardness: 5,
        hit_points: 20,
        broken_threshold: 10,
        shield_ac_bonus: 2,
    }
}

fn seen_diagonal() -> DiagonalCounter {
    let outcome = grid_stride(MoveInput {
        start: sq(0, 0),
        path: &[sq(1, 1)],
        speed_feet: 25,
        size: Size::Medium,
        immobilized: false,
        faction: FactionId(1),
        bounds: MapBounds {
            width: 64,
            height: 64,
        },
        terrain: &[],
        occupants: &[],
        counter: DiagonalCounter::fresh(),
    })
    .expect("diagonal");
    assert_eq!(outcome.counter.diagonals(), 1);
    outcome.counter
}

/// Player Core, Delay.
#[test]
fn the_same_delay_twice_matches() {
    let run = || {
        let (mut encounter, dice) = begin(
            vec![ticks(pc(1, 0, 20, 18), 4, 2), pc(2, 0, 20, 18)],
            &[15, 10, 8],
        );
        open(&mut encounter, 1);
        encounter.delay(id(1)).expect("delay");
        open(&mut encounter, 2);
        close(&mut encounter);
        encounter.return_from_delay(&[id(1)]).expect("return");
        assert_eq!(dice.remaining(), 1);
        encounter
    };
    let first = run();
    let second = run();
    assert_eq!(first, second);
    assert_eq!(first.order(), order_of(&[2, 1]).as_slice());
    assert_eq!(first.round(), 1);
    assert_eq!(first.turn_open(), Some(id(1)));
    assert_eq!(actions(&first, 1), 3);
    assert!(reaction(&first, 1));
    assert_eq!(initiative(&first, 1), 15);
    assert_eq!(initiative(&first, 2), 10);
    assert_eq!(temp_hp(&first, 1), 4);
    assert_eq!(temp_rounds(&first, 1), 2);
}

/// Player Core, Delay.
#[test]
fn delay_spends_nothing_and_closes_the_turn() {
    let (mut encounter, dice) = begin(
        vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18), pc(3, 0, 20, 18)],
        &[15, 10, 5, 8],
    );
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    assert_eq!(actions(&encounter, 1), 0);
    assert!(reaction(&encounter, 1));
    assert_eq!(encounter.turn_open(), None);
    assert_eq!(encounter.next_start().expect("next"), id(2));
    assert_eq!(slot(&encounter, 1), TurnSlot::Delayed);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Delay.
#[test]
fn an_action_already_spent_is_not_the_start() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 18)], &[10]);
    open(&mut encounter, 1);
    encounter.spend_actions(id(1), 1).expect("spend");
    assert_eq!(
        encounter.delay(id(1)).expect_err("delay"),
        EncounterError::TriggerNotMet
    );
    assert_eq!(actions(&encounter, 1), 2);
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(slot(&encounter, 1), TurnSlot::Normal);
}

/// Player Core, Delay.
#[test]
fn a_spent_reaction_is_not_the_start() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 18)], &[10]);
    open(&mut encounter, 1);
    encounter.spend_reaction(id(1)).expect("spend");
    assert_eq!(
        encounter.delay(id(1)).expect_err("delay"),
        EncounterError::TriggerNotMet
    );
    assert_eq!(actions(&encounter, 1), 3);
    assert!(!reaction(&encounter, 1));
    assert_eq!(encounter.turn_open(), Some(id(1)));
}

/// Player Core, Delay.
#[test]
fn an_unknown_creature_is_not_a_closed_turn() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 18)], &[10]);
    assert_eq!(
        encounter.delay(id(9)).expect_err("unknown"),
        EncounterError::UnknownCombatant
    );
    assert_eq!(slot(&encounter, 1), TurnSlot::Normal);
}

/// Player Core, Delay.
#[test]
fn delay_before_a_start_is_turnnotopen() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 18)], &[10]);
    assert_eq!(
        encounter.delay(id(1)).expect_err("closed"),
        EncounterError::TurnNotOpen
    );
    assert_eq!(actions(&encounter, 1), 0);
}

/// Player Core, Delay.
#[test]
fn delaying_on_another_turn_is_notyourturn() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[10, 8]);
    open(&mut encounter, 1);
    close(&mut encounter);
    open(&mut encounter, 2);
    assert!(reaction(&encounter, 1));
    assert_eq!(
        encounter.delay(id(1)).expect_err("other"),
        EncounterError::NotYourTurn
    );
    assert_eq!(encounter.turn_open(), Some(id(2)));
}

/// Player Core, Delay.
#[test]
fn a_dying_creature_is_notconscious() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 18)], &[10]);
    lay(&mut encounter, 1, dying_at(1), 0);
    open(&mut encounter, 1);
    assert_eq!(actions(&encounter, 1), 0);
    assert!(!reaction(&encounter, 1));
    assert_eq!(
        encounter.delay(id(1)).expect_err("dying"),
        EncounterError::NotConscious
    );
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(slot(&encounter, 1), TurnSlot::Normal);
}

/// Player Core, Delay. Player Core, "Step 2: Act".
#[test]
fn round_overflow_does_not_delay() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10]);
    encounter.set_round_for_test(u32::MAX);
    open(&mut encounter, 1);
    close(&mut encounter);
    open(&mut encounter, 2);
    assert_eq!(actions(&encounter, 2), 3);
    assert!(reaction(&encounter, 2));
    encounter.delay(id(2)).expect("delay");
    assert_eq!(encounter.round(), u32::MAX);
    assert_eq!(encounter.turn_open(), None);
    assert_eq!(slot(&encounter, 2), TurnSlot::Delayed);
    assert_eq!(
        encounter.next_start().expect_err("overflow"),
        EncounterError::RoundOverflow
    );
    assert_eq!(encounter.round(), u32::MAX);
    assert_eq!(encounter.turn_open(), None);
    assert_eq!(slot(&encounter, 2), TurnSlot::Delayed);
}

/// Player Core, Delay.
#[test]
fn start_turn_at_round_4294967295_is_roundoverflow() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10]);
    encounter.set_round_for_test(u32::MAX);
    open(&mut encounter, 1);
    close(&mut encounter);
    open(&mut encounter, 2);
    encounter.delay(id(2)).expect("delay");
    assert_eq!(
        encounter.start_turn().expect_err("overflow"),
        EncounterError::RoundOverflow
    );
    assert_eq!(encounter.round(), u32::MAX);
    assert_eq!(encounter.turn_open(), None);
    assert_eq!(slot(&encounter, 2), TurnSlot::Delayed);
}

/// Player Core, "Step 2: Act".
#[test]
fn next_start_while_a_turn_is_open_is_turninprogress() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10]);
    open(&mut encounter, 1);
    assert_eq!(
        encounter.next_start().expect_err("open"),
        EncounterError::TurnInProgress
    );
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(encounter.round(), 1);
}

/// Player Core, Delay.
#[test]
fn next_start_during_a_pending_wrap_keeps_the_round() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10]);
    open(&mut encounter, 1);
    close(&mut encounter);
    open(&mut encounter, 2);
    encounter.delay(id(2)).expect("delay");
    assert_eq!(encounter.next_start().expect("next"), id(1));
    assert_eq!(encounter.round(), 1);
    assert_eq!(encounter.turn_open(), None);
    assert_eq!(slot(&encounter, 2), TurnSlot::Delayed);
}

/// Player Core, Delay.
#[test]
fn next_start_during_a_returned_turn_is_turninprogress() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    close(&mut encounter);
    encounter.return_from_delay(&[id(1)]).expect("return");
    assert_eq!(
        encounter.next_start().expect_err("returned"),
        EncounterError::TurnInProgress
    );
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(encounter.round(), 1);
    close(&mut encounter);
    assert_eq!(encounter.next_start().expect("wrap"), id(2));
    assert_eq!(encounter.round(), 2);
}

/// Player Core, Encounter Mode.
///
/// `begin_encounter` rejects this roster, and design.md allows no constructor for it.
#[test]
fn next_start_on_an_empty_roster_is_nocreature() {
    let encounter = Encounter {
        members: Vec::new(),
        order: Vec::new(),
        cursor: 0,
        round: 1,
        open: None,
        last_ended: None,
        turn_state: super::super::TurnState::Continuing,
    };
    let dice = ScriptedDice::new([10]);
    assert_eq!(
        encounter.next_start().expect_err("empty"),
        EncounterError::NoCreature
    );
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Delay.
#[test]
fn a_second_delay_after_return_writes_nothing() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    close(&mut encounter);
    encounter.return_from_delay(&[id(1)]).expect("return");
    assert_eq!(
        encounter.delay(id(1)).expect_err("again"),
        EncounterError::TriggerNotMet
    );
    assert_eq!(encounter.order(), order_of(&[2, 1]).as_slice());
    assert_eq!(encounter.round(), 1);
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(actions(&encounter, 1), 3);
    assert_ne!(slot(&encounter, 1), TurnSlot::Delayed);
}

/// Player Core, Delay. Player Core, "Ready".
#[test]
fn ready_ending_the_turn_blocks_delay() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 8]);
    let before = encounter.order().to_vec();
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        reach(2),
        melee(2, Agility::NotAgile, 0, 0),
    );
    assert_eq!(
        encounter.delay(id(1)).expect_err("closed"),
        EncounterError::TurnNotOpen
    );
    assert_ne!(slot(&encounter, 1), TurnSlot::Delayed);
    assert_eq!(encounter.order(), before.as_slice());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Delay.
#[test]
fn return_acts_after_the_creature_who_just_ended() {
    let (mut encounter, _) = begin(
        vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18), pc(3, 0, 20, 18)],
        &[15, 10, 5],
    );
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    close(&mut encounter);
    encounter.return_from_delay(&[id(1)]).expect("return");
    assert_eq!(encounter.order(), order_of(&[2, 1, 3]).as_slice());
    assert_eq!(initiative(&encounter, 1), 15);
    assert_eq!(initiative(&encounter, 2), 10);
    assert_eq!(initiative(&encounter, 3), 5);
    assert_eq!(encounter.round(), 1);
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(actions(&encounter, 1), 3);
    assert!(reaction(&encounter, 1));
}

/// Player Core, Delay.
#[test]
fn two_returns_at_one_end_go_by_the_lower_id() {
    let (mut encounter, _) = begin(
        vec![
            pc(1, 0, 20, 18),
            pc(2, 0, 20, 18),
            pc(3, 0, 20, 18),
            pc(4, 0, 20, 18),
        ],
        &[20, 15, 10, 5],
    );
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay 1");
    open(&mut encounter, 2);
    close(&mut encounter);
    open(&mut encounter, 3);
    encounter.delay(id(3)).expect("delay 3");
    open(&mut encounter, 4);
    close(&mut encounter);
    encounter
        .return_from_delay(&[id(3), id(1)])
        .expect("return");
    assert_eq!(encounter.order(), order_of(&[2, 4, 1, 3]).as_slice());
    assert_eq!(encounter.round(), 1);
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(initiative(&encounter, 1), 20);
    assert_eq!(initiative(&encounter, 2), 15);
    assert_eq!(initiative(&encounter, 3), 10);
    assert_eq!(initiative(&encounter, 4), 5);
    assert_ne!(slot(&encounter, 3), TurnSlot::Delayed);
    assert_ne!(encounter.turn_open(), Some(id(3)));
}

/// Player Core, Delay.
#[test]
fn an_enemy_returns_before_a_pc() {
    let (mut encounter, _) = begin(
        vec![
            pc(1, 0, 20, 18),
            pc(2, 0, 20, 18),
            enemy(3, 0, 20, 18),
            pc(4, 0, 20, 18),
        ],
        &[20, 15, 10, 5],
    );
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay 1");
    open(&mut encounter, 2);
    close(&mut encounter);
    open(&mut encounter, 3);
    encounter.delay(id(3)).expect("delay 3");
    open(&mut encounter, 4);
    close(&mut encounter);
    assert_eq!(initiative(&encounter, 1), 20);
    assert_eq!(initiative(&encounter, 3), 10);
    encounter
        .return_from_delay(&[id(1), id(3)])
        .expect("return");
    assert_eq!(encounter.order(), order_of(&[2, 4, 3, 1]).as_slice());
    assert_eq!(encounter.round(), 1);
    assert_eq!(encounter.turn_open(), Some(id(3)));
    assert_eq!(initiative(&encounter, 3), 10);
    assert_eq!(initiative(&encounter, 1), 20);
}

/// Player Core, Delay.
#[test]
fn return_leaves_temporary_hit_points_as_the_start_left_them() {
    let (mut encounter, dice) = begin(
        vec![ticks(pc(1, 0, 9, 18), 4, 2), pc(2, 0, 20, 18)],
        &[15, 10, 8],
    );
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    close(&mut encounter);
    encounter.return_from_delay(&[id(1)]).expect("return");
    assert_eq!(temp_hp(&encounter, 1), 4);
    assert_eq!(temp_rounds(&encounter, 1), 2);
    assert_eq!(hit_points(&encounter, 1), 9);
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Delay.
#[test]
fn the_later_returned_creature_is_not_ticked() {
    let (mut encounter, dice) = begin(
        vec![
            ticks(pc(1, 0, 9, 18), 4, 2),
            pc(2, 0, 20, 18),
            ticks(pc(3, 0, 9, 18), 4, 2),
            pc(4, 0, 20, 18),
        ],
        &[20, 15, 10, 5, 8],
    );
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay 1");
    open(&mut encounter, 2);
    close(&mut encounter);
    open(&mut encounter, 3);
    encounter.delay(id(3)).expect("delay 3");
    open(&mut encounter, 4);
    close(&mut encounter);
    encounter
        .return_from_delay(&[id(3), id(1)])
        .expect("return");
    close(&mut encounter);
    open(&mut encounter, 3);
    assert_eq!(encounter.round(), 1);
    assert_eq!(encounter.turn_open(), Some(id(3)));
    assert_eq!(actions(&encounter, 3), 3);
    assert!(reaction(&encounter, 3));
    assert_eq!(temp_hp(&encounter, 1), 4);
    assert_eq!(temp_rounds(&encounter, 1), 2);
    assert_eq!(hit_points(&encounter, 1), 9);
    assert_eq!(temp_hp(&encounter, 3), 4);
    assert_eq!(temp_rounds(&encounter, 3), 2);
    assert_eq!(hit_points(&encounter, 3), 9);
    assert_eq!(dice.remaining(), 1);
    close(&mut encounter);
    open(&mut encounter, 2);
    assert_eq!(encounter.round(), 2);
    assert_eq!(encounter.turn_open(), Some(id(2)));
    assert_eq!(actions(&encounter, 2), 3);
}

/// Player Core, Delay. Player Core, "Raise a Shield".
#[test]
fn a_lowered_shield_stays_down() {
    let (mut encounter, _) = begin(
        vec![
            participant(1, Side::Pc, creature(30, 18, 0, false, Some(steel()))),
            pc(2, 0, 20, 18),
        ],
        &[15, 10],
    );
    open(&mut encounter, 1);
    encounter.raise_shield(id(1)).expect("raise");
    let shield_hp = encounter.state(id(1)).expect("state").shield_hit_points;
    close(&mut encounter);
    open(&mut encounter, 2);
    close(&mut encounter);
    open(&mut encounter, 1);
    assert!(!encounter.state(id(1)).expect("state").shield_raised);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    close(&mut encounter);
    encounter.return_from_delay(&[id(1)]).expect("return");
    let state = encounter.state(id(1)).expect("state");
    assert!(!state.shield_raised);
    assert_eq!(state.shield_hit_points, shield_hp);
}

/// Player Core, Delay.
#[test]
fn a_full_round_without_returning_plays_the_next_turn_in_place() {
    let (mut encounter, _) = begin(
        vec![
            ticks(pc(1, 0, 9, 18), 4, 3),
            pc(2, 0, 20, 18),
            pc(3, 0, 20, 18),
        ],
        &[15, 10, 5],
    );
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    close(&mut encounter);
    open(&mut encounter, 3);
    close(&mut encounter);
    open(&mut encounter, 1);
    assert_eq!(encounter.round(), 2);
    assert_eq!(encounter.order(), order_of(&[1, 2, 3]).as_slice());
    assert_eq!(initiative(&encounter, 1), 15);
    assert_eq!(initiative(&encounter, 2), 10);
    assert_eq!(initiative(&encounter, 3), 5);
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(actions(&encounter, 1), 3);
    assert!(reaction(&encounter, 1));
    assert_eq!(temp_hp(&encounter, 1), 4);
    assert_eq!(temp_rounds(&encounter, 1), 3);
}

/// Player Core, Delay.
#[test]
fn a_sole_creatures_next_start_is_the_new_turn() {
    let (mut encounter, _) = begin(vec![ticks(pc(1, 0, 9, 18), 4, 2)], &[10]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 1);
    assert_eq!(encounter.round(), 2);
    assert_eq!(initiative(&encounter, 1), 10);
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(actions(&encounter, 1), 3);
    assert!(reaction(&encounter, 1));
    assert_eq!(temp_hp(&encounter, 1), 4);
    assert_eq!(temp_rounds(&encounter, 1), 2);
    assert_eq!(hit_points(&encounter, 1), 9);
}

/// Player Core, Delay.
#[test]
fn returning_after_the_last_creature_acts_in_this_round() {
    let (mut encounter, _) = begin(
        vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18), pc(3, 0, 20, 18)],
        &[15, 10, 5],
    );
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    close(&mut encounter);
    open(&mut encounter, 3);
    close(&mut encounter);
    encounter.return_from_delay(&[id(1)]).expect("return");
    assert_eq!(encounter.round(), 1);
    assert_eq!(encounter.order(), order_of(&[2, 3, 1]).as_slice());
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(initiative(&encounter, 1), 15);
    assert_eq!(initiative(&encounter, 2), 10);
    assert_eq!(initiative(&encounter, 3), 5);
    close(&mut encounter);
    open(&mut encounter, 2);
    assert_eq!(encounter.round(), 2);
    assert_eq!(encounter.turn_open(), Some(id(2)));
    assert_eq!(actions(&encounter, 2), 3);
}

/// Player Core, Delay.
#[test]
fn a_mid_round_return_keeps_the_next_turn_in_this_round() {
    let (mut encounter, _) = begin(
        vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18), pc(3, 0, 20, 18)],
        &[15, 10, 5],
    );
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    close(&mut encounter);
    encounter.return_from_delay(&[id(1)]).expect("return");
    close(&mut encounter);
    open(&mut encounter, 3);
    assert_eq!(encounter.round(), 1);
    assert_eq!(encounter.turn_open(), Some(id(3)));
    assert_eq!(actions(&encounter, 3), 3);
    assert_eq!(encounter.order(), order_of(&[2, 1, 3]).as_slice());
}

/// Player Core, "Multiple Attack Penalty". Player Core, Delay.
#[test]
fn the_first_strike_after_return_has_no_penalty() {
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)],
        &[15, 10, 10, 10, 10],
    );
    open(&mut encounter, 1);
    let swing = profile(8, Agility::NotAgile, 0, 8, 0);
    encounter
        .strike(id(1), id(2), &swing, &[], ShieldBlock::Declined, &mut dice)
        .expect("first");
    encounter
        .strike(id(1), id(2), &swing, &[], ShieldBlock::Declined, &mut dice)
        .expect("second");
    close(&mut encounter);
    open(&mut encounter, 2);
    close(&mut encounter);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    close(&mut encounter);
    encounter.return_from_delay(&[id(1)]).expect("return");
    let outcome = encounter
        .strike(id(1), id(2), &swing, &[], ShieldBlock::Declined, &mut dice)
        .expect("return strike");
    assert_eq!(outcome.map, 0);
    assert_eq!(outcome.total, 18);
    assert_eq!(outcome.degree, DegreeOfSuccess::Success);
}

/// Player Core, Delay.
#[test]
fn a_dying_creatures_missed_turn_opens_with_no_actions() {
    let (mut encounter, _) = begin(
        vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18), pc(3, 0, 20, 18)],
        &[15, 10, 5],
    );
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    lay(&mut encounter, 1, dying_at(1), 0);
    open(&mut encounter, 2);
    close(&mut encounter);
    open(&mut encounter, 3);
    close(&mut encounter);
    open(&mut encounter, 1);
    assert_eq!(encounter.round(), 2);
    assert_eq!(encounter.order(), order_of(&[1, 2, 3]).as_slice());
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(actions(&encounter, 1), 0);
    assert!(!reaction(&encounter, 1));
    assert_eq!(slot(&encounter, 1), TurnSlot::Normal);
}

/// Player Core, Delay.
#[test]
fn a_dead_creatures_missed_turn_is_skipped() {
    let (mut encounter, _) = begin(
        vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18), pc(3, 0, 20, 18)],
        &[15, 10, 5],
    );
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    lay(&mut encounter, 1, Vital::Dead, 0);
    open(&mut encounter, 2);
    close(&mut encounter);
    open(&mut encounter, 3);
    close(&mut encounter);
    open(&mut encounter, 2);
    assert_eq!(encounter.round(), 2);
    assert_eq!(encounter.turn_open(), Some(id(2)));
    assert_eq!(actions(&encounter, 2), 3);
    assert_eq!(slot(&encounter, 1), TurnSlot::Normal);
    assert_eq!(actions(&encounter, 1), 0);
}

/// Player Core, "Getting Knocked Out". Player Core, Delay.
#[test]
fn a_delayed_creature_knocked_out_moves_before_that_turn() {
    let (mut encounter, mut dice) = begin(
        vec![
            participant(1, Side::Pc, creature(9, 18, 0, true, None)),
            pc(2, 0, 20, 18),
        ],
        &[15, 10, 10, 8],
    );
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    encounter
        .strike(
            id(2),
            id(1),
            &profile(8, Agility::NotAgile, 1, 8, 1),
            &[],
            ShieldBlock::Declined,
            &mut dice,
        )
        .expect("strike");
    assert_eq!(vital(&encounter, 1), dying_at(1));
    assert_eq!(hit_points(&encounter, 1), 0);
    assert_eq!(encounter.order(), order_of(&[1, 2]).as_slice());
    assert_eq!(encounter.turn_open(), Some(id(2)));
    assert_eq!(slot(&encounter, 1), TurnSlot::Normal);
    assert_eq!(take_report(&mut encounter, 1), None);
    assert!(reaction(&encounter, 1));
    assert_eq!(actions(&encounter, 2), 2);
}

/// Player Core, "Getting Knocked Out". Player Core, "Ready". Player Core, Delay.
#[test]
fn a_readied_strike_knocks_a_delayed_creature_out_before_the_readier() {
    let (mut encounter, mut dice) = begin(
        vec![
            pc(1, 0, 20, 18),
            participant(2, Side::Pc, creature(9, 18, 0, true, None)),
        ],
        &[15, 10, 10, 8],
    );
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::NotAgile, 1, 1),
    );
    open(&mut encounter, 2);
    encounter.delay(id(2)).expect("delay");
    assert_eq!(slot(&encounter, 2), TurnSlot::Delayed);
    assert_eq!(encounter.order(), order_of(&[1, 2]).as_slice());
    let strike = strike_numbers(&fire_begins(&mut encounter, 2, &mut dice));
    assert_eq!(strike.degree, DegreeOfSuccess::Success);
    assert_eq!(strike.damage, 9);
    assert_eq!(vital(&encounter, 2), dying_at(1));
    assert_eq!(hit_points(&encounter, 2), 0);
    assert_eq!(encounter.order(), order_of(&[2, 1]).as_slice());
    assert_eq!(slot(&encounter, 2), TurnSlot::Normal);
    assert_eq!(take_report(&mut encounter, 2), None);
    assert_eq!(encounter.turn_open(), None);
    assert!(!reaction(&encounter, 1));
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Delay. Player Core, "Ready".
#[test]
fn return_follows_a_turn_that_ready_ended() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    ready_on(
        &mut encounter,
        2,
        reach(1),
        melee(1, Agility::NotAgile, 0, 0),
    );
    encounter.return_from_delay(&[id(1)]).expect("return");
    assert_eq!(encounter.order(), order_of(&[2, 1]).as_slice());
    assert_eq!(encounter.round(), 1);
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(actions(&encounter, 1), 3);
    assert!(encounter.stored_ready(id(2)).expect("ready").is_some());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Delay. Player Core, "Ready".
#[test]
fn a_missed_return_clears_a_stored_ready() {
    let (mut encounter, dice) = begin(
        vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18), pc(3, 0, 20, 18)],
        &[15, 10, 5, 10],
    );
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    encounter.set_ready_for_test(
        id(1),
        StoredReady {
            trigger: reach(2),
            activity: melee(2, Agility::NotAgile, 0, 0),
            frozen_attack_count: 0,
        },
    );
    open(&mut encounter, 2);
    close(&mut encounter);
    open(&mut encounter, 3);
    close(&mut encounter);
    open(&mut encounter, 1);
    assert_eq!(encounter.round(), 2);
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
    assert_eq!(dice.remaining(), 1);
    assert!(reaction(&encounter, 1));
}

/// Player Core, Delay.
#[test]
fn return_during_a_turn_writes_nothing() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    assert_eq!(
        encounter.return_from_delay(&[id(1)]).expect_err("open"),
        EncounterError::TurnInProgress
    );
    assert_eq!(slot(&encounter, 1), TurnSlot::Delayed);
    assert_eq!(encounter.turn_open(), Some(id(2)));
}

/// Player Core, Delay.
#[test]
fn a_duplicate_return_writes_nothing() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    close(&mut encounter);
    assert_eq!(
        encounter
            .return_from_delay(&[id(1), id(1)])
            .expect_err("duplicate"),
        EncounterError::DuplicateCombatant
    );
    assert_eq!(slot(&encounter, 1), TurnSlot::Delayed);
}

/// Player Core, Delay.
#[test]
fn a_dying_delayed_creature_cannot_return() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    close(&mut encounter);
    lay(&mut encounter, 1, dying_at(1), 0);
    assert_eq!(
        encounter.return_from_delay(&[id(1)]).expect_err("dying"),
        EncounterError::NotConscious
    );
    assert_eq!(slot(&encounter, 1), TurnSlot::Delayed);
}

/// Player Core, Delay. Player Core, "Reactions in Encounters".
#[test]
fn a_spend_while_delayed_leaves_the_reaction_ready() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18)], &[10, 8]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    assert_eq!(
        encounter.spend_reaction(id(1)).expect_err("spend"),
        EncounterError::Delayed
    );
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Delay.
#[test]
fn return_leaves_one_reaction() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    close(&mut encounter);
    encounter.return_from_delay(&[id(1)]).expect("return");
    encounter.spend_reaction(id(1)).expect("first");
    assert_eq!(
        encounter.spend_reaction(id(1)).expect_err("second"),
        EncounterError::NoReaction
    );
}

/// Player Core, Delay. Player Core, "Shield Block".
#[test]
fn a_shield_block_while_delayed_spends_nothing() {
    let (mut encounter, mut dice) = begin(
        vec![
            participant(1, Side::Pc, creature(30, 18, 0, false, Some(steel()))),
            enemy(2, 0, 20, 18),
        ],
        &[15, 10, 10, 5],
    );
    open(&mut encounter, 1);
    encounter.raise_shield(id(1)).expect("raise");
    close(&mut encounter);
    open(&mut encounter, 2);
    close(&mut encounter);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    assert_eq!(
        encounter
            .strike(
                id(2),
                id(1),
                &profile(8, Agility::NotAgile, 1, 8, 4),
                &[],
                ShieldBlock::Requested,
                &mut dice,
            )
            .expect_err("block"),
        EncounterError::Delayed
    );
    assert_eq!(actions(&encounter, 2), 3);
    assert_eq!(dice.remaining(), 2);
    assert!(reaction(&encounter, 1));
    assert_eq!(slot(&encounter, 1), TurnSlot::Delayed);
}

/// Player Core, Delay. Player Core, "Reactive Strike".
#[test]
fn a_reactive_strike_while_delayed_spends_nothing() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    let error = encounter
        .reactive_strike(
            id(1),
            id(2),
            manipulate(),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            sq(0, 0),
            sq(0, 1),
            Size::Medium,
            Size::Medium,
            &profile(8, Agility::NotAgile, 0, 8, 0),
            &[],
            &mut dice,
        )
        .expect_err("reactive");
    assert_eq!(error, crate::reactive::ReactiveError::Delayed);
    assert_eq!(actions(&encounter, 2), 3);
    assert_eq!(dice.remaining(), 1);
    assert!(reaction(&encounter, 1));
}

/// Player Core, Delay. Player Core, Aid.
#[test]
fn aid_while_delayed_spends_nothing() {
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 18),
        ],
        &[15, 10, 10],
    );
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    set_prepared(&mut encounter, 1, 2);
    open(&mut encounter, 2);
    assert_eq!(
        encounter.aid(id(1), id(2), &mut dice).expect_err("aid"),
        EncounterError::Delayed
    );
    assert_eq!(actions(&encounter, 2), 3);
    assert_eq!(dice.remaining(), 1);
    assert!(reaction(&encounter, 1));
    assert_eq!(
        encounter.state(id(1)).expect("state").aid_preparation,
        AidPreparation::Prepared { ally: id(2) }
    );
}

/// Player Core, "Shield Block". Player Core, Delay.
#[test]
fn a_delayed_creature_is_rejected_before_the_shield_is_read() {
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 30, 18), enemy(2, 0, 20, 18)],
        &[15, 10, 10, 5],
    );
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    assert_eq!(
        encounter
            .strike(
                id(2),
                id(1),
                &profile(8, Agility::NotAgile, 1, 8, 0),
                &[],
                ShieldBlock::Requested,
                &mut dice,
            )
            .expect_err("no shield"),
        EncounterError::Delayed
    );
    assert_eq!(hit_points(&encounter, 1), 30);
    assert!(reaction(&encounter, 1));
    assert_eq!(actions(&encounter, 2), 3);
    assert_eq!(dice.remaining(), 2);
}
/// Player Core, "Ready".
#[test]
fn storing_the_same_ready_twice_matches() {
    let run = || {
        let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 8]);
        open(&mut encounter, 1);
        assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
        ready_on(
            &mut encounter,
            1,
            reach(2),
            melee(2, Agility::NotAgile, 0, 0),
        );
        assert_eq!(dice.remaining(), 1);
        encounter
    };
    let first = run();
    let second = run();
    assert_eq!(first, second);
    let stored = first.stored_ready(id(1)).expect("ready").expect("some");
    assert_eq!(stored.trigger, reach(2));
    assert_eq!(stored.activity, melee(2, Agility::NotAgile, 0, 0));
    assert_eq!(first.frozen_attack_count(id(1)).expect("frozen"), Some(0));
    assert_eq!(first.turn_open(), None);
    assert_eq!(actions(&first, 1), 0);
    assert!(reaction(&first, 1));
}

/// Player Core, "Ready". Player Core, "Multiple Attack Penalty".
#[test]
fn one_strike_then_ready_stores_count_1() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10, 8]);
    open(&mut encounter, 1);
    encounter
        .strike(
            id(1),
            id(2),
            &profile(8, Agility::NotAgile, 0, 8, 0),
            &[],
            ShieldBlock::Declined,
            &mut dice,
        )
        .expect("strike");
    ready_on(
        &mut encounter,
        1,
        reach(2),
        melee(2, Agility::NotAgile, 0, 0),
    );
    assert_eq!(
        encounter.frozen_attack_count(id(1)).expect("frozen"),
        Some(1)
    );
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 1);
    assert_eq!(encounter.turn_open(), None);
    assert_eq!(actions(&encounter, 1), 0);
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready".
#[test]
fn ready_with_no_earlier_attack_stores_count_0() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        reach(2),
        melee(2, Agility::NotAgile, 0, 0),
    );
    assert_eq!(
        encounter.frozen_attack_count(id(1)).expect("frozen"),
        Some(0)
    );
    assert_eq!(encounter.turn_open(), None);
    assert!(reaction(&encounter, 1));
}

/// Player Core, "Ready".
#[test]
fn ready_is_legal_after_the_reaction_was_spent() {
    let (mut encounter, _) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10]);
    open(&mut encounter, 1);
    encounter.spend_reaction(id(1)).expect("spend");
    ready_on(
        &mut encounter,
        1,
        reach(2),
        melee(2, Agility::NotAgile, 0, 0),
    );
    assert_eq!(
        encounter.frozen_attack_count(id(1)).expect("frozen"),
        Some(0)
    );
    assert!(!reaction(&encounter, 1));
}

/// Player Core, "Ready".
#[test]
fn the_next_start_clears_the_ready() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        reach(2),
        melee(2, Agility::NotAgile, 0, 0),
    );
    open(&mut encounter, 2);
    close(&mut encounter);
    open(&mut encounter, 1);
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
    let resolved = offer(
        &mut encounter,
        2,
        ObservedAction::Stride,
        sq(0, 2),
        sq(0, 1),
        &[],
        &mut dice,
    );
    assert!(resolved.is_empty());
    assert_eq!(dice.remaining(), 1);
    assert!(reaction(&encounter, 1));
}

/// Player Core, "Ready".
#[test]
fn a_two_action_activity_is_rejected() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    assert_eq!(
        encounter
            .ready(id(1), reach(2), cost(2), melee(2, Agility::NotAgile, 0, 0))
            .expect_err("two"),
        ReadyError::NotASingleAction
    );
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready". Player Core, Delay.
#[test]
fn delay_cannot_be_readied() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    assert_eq!(
        encounter
            .ready(id(1), reach(2), cost(1), ReadyActivity::NoFire)
            .expect_err("delay"),
        ReadyError::TriggeredFreeAction
    );
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready".
#[test]
fn a_readied_strike_cannot_name_a_third_creature() {
    let (mut encounter, dice) = begin(
        vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18), pc(3, 0, 20, 18)],
        &[15, 10, 5, 10],
    );
    open(&mut encounter, 1);
    assert_eq!(
        encounter
            .ready(id(1), reach(2), cost(1), melee(3, Agility::NotAgile, 0, 0))
            .expect_err("third"),
        ReadyError::NotTheTriggerSubject
    );
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready".
#[test]
fn an_unknown_creature_is_rejected_first() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18)], &[10, 10]);
    assert_eq!(
        encounter
            .ready(id(9), reach(1), cost(1), melee(1, Agility::NotAgile, 0, 0))
            .expect_err("unknown"),
        ReadyError::UnknownCombatant
    );
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready".
#[test]
fn an_unknown_subject_is_rejected_before_a_closed_turn() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18)], &[10, 10]);
    assert_eq!(
        encounter
            .ready(id(1), reach(9), cost(1), melee(9, Agility::NotAgile, 0, 0))
            .expect_err("subject"),
        ReadyError::UnknownCombatant
    );
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready".
#[test]
fn a_trigger_about_the_readier_is_rejected() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18)], &[10, 10]);
    open(&mut encounter, 1);
    assert_eq!(
        encounter
            .ready(id(1), reach(1), cost(1), melee(1, Agility::NotAgile, 0, 0))
            .expect_err("self"),
        ReadyError::TriggerSubjectIsSelf
    );
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready".
#[test]
fn one_action_left_is_rejected_before_a_round_past_the_maximum() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    encounter.set_round_for_test(u32::MAX);
    open(&mut encounter, 1);
    close(&mut encounter);
    open(&mut encounter, 2);
    encounter.spend_actions(id(2), 2).expect("spend");
    assert_eq!(
        encounter
            .ready(id(2), reach(1), cost(1), melee(1, Agility::NotAgile, 0, 0))
            .expect_err("actions"),
        ReadyError::InsufficientActions
    );
    assert_eq!(encounter.round(), u32::MAX);
    assert_eq!(actions(&encounter, 2), 1);
    assert_eq!(encounter.stored_ready(id(2)).expect("ready"), None);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready".
#[test]
fn ready_at_the_maximum_round_changes_nothing() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    encounter.set_round_for_test(u32::MAX);
    open(&mut encounter, 1);
    close(&mut encounter);
    open(&mut encounter, 2);
    assert_eq!(
        encounter
            .ready(id(2), reach(1), cost(1), melee(1, Agility::NotAgile, 0, 0))
            .expect_err("round"),
        ReadyError::RoundOverflow
    );
    assert_eq!(encounter.round(), u32::MAX);
    assert_eq!(actions(&encounter, 2), 3);
    assert_eq!(encounter.stored_ready(id(2)).expect("ready"), None);
    assert_eq!(encounter.turn_open(), Some(id(2)));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready". Player Core, Delay.
#[test]
fn a_returned_creature_may_ready() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    close(&mut encounter);
    encounter.return_from_delay(&[id(1)]).expect("return");
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(encounter.order().get(encounter.cursor), Some(&id(1)));
    ready_on(
        &mut encounter,
        1,
        reach(2),
        melee(2, Agility::NotAgile, 0, 0),
    );
    assert_eq!(
        encounter.frozen_attack_count(id(1)).expect("frozen"),
        Some(0)
    );
    assert_eq!(encounter.turn_open(), None);
    assert_eq!(actions(&encounter, 1), 0);
    assert!(reaction(&encounter, 1));
    assert_eq!(encounter.round(), 2);
    assert_eq!(encounter.next_start().expect("next"), id(2));
    assert!(encounter.stored_ready(id(1)).expect("ready").is_some());
    assert_eq!(dice.remaining(), 1);
    open(&mut encounter, 2);
    close(&mut encounter);
    open(&mut encounter, 1);
    assert_eq!(encounter.round(), 2);
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
}

fn fire_begins(
    encounter: &mut Encounter,
    actor: u32,
    dice: &mut ScriptedDice,
) -> Vec<ResolvedReaction> {
    offer(
        encounter,
        actor,
        ObservedAction::Strike,
        sq(0, 1),
        sq(0, 1),
        &[],
        dice,
    )
}

/// Player Core, "Ready". Player Core, "Multiple Attack Penalty".
#[test]
fn a_readied_non_agile_strike_after_one_strike_takes_minus_5() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10, 15]);
    open(&mut encounter, 1);
    encounter
        .strike(
            id(1),
            id(2),
            &profile(8, Agility::NotAgile, 0, 8, 0),
            &[],
            ShieldBlock::Declined,
            &mut dice,
        )
        .expect("strike");
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::NotAgile, 0, 0),
    );
    open(&mut encounter, 2);
    let strike = strike_numbers(&fire_begins(&mut encounter, 2, &mut dice));
    assert_eq!(strike.map, -5);
    assert_eq!(strike.total, 18);
    assert_eq!(strike.degree, DegreeOfSuccess::Success);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 1);
    close(&mut encounter);
    open(&mut encounter, 1);
    assert_eq!(
        encounter
            .attack_trait_penalty(id(1), Agility::NotAgile)
            .expect("penalty"),
        0
    );
}

/// Player Core, "Ready". Player Core, "Multiple Attack Penalty".
#[test]
fn the_first_readied_non_agile_strike_takes_0() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::NotAgile, 0, 0),
    );
    open(&mut encounter, 2);
    let strike = strike_numbers(&fire_begins(&mut encounter, 2, &mut dice));
    assert_eq!(strike.map, 0);
    assert_eq!(strike.total, 18);
    assert_eq!(strike.degree, DegreeOfSuccess::Success);
}

/// Player Core, "Ready". Player Core, "Multiple Attack Penalty".
#[test]
fn a_readied_non_agile_strike_at_count_2_takes_minus_10() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 15]);
    open(&mut encounter, 1);
    encounter.set_attack_count_for_test(id(1), 2);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::NotAgile, 0, 0),
    );
    open(&mut encounter, 2);
    let strike = strike_numbers(&fire_begins(&mut encounter, 2, &mut dice));
    assert_eq!(strike.map, -10);
    assert_eq!(strike.total, 13);
    assert_eq!(strike.degree, DegreeOfSuccess::Failure);
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 2);
    close(&mut encounter);
    open(&mut encounter, 1);
    assert_eq!(
        encounter
            .attack_trait_penalty(id(1), Agility::NotAgile)
            .expect("penalty"),
        0
    );
}

/// Player Core, "Ready". Player Core, "Multiple Attack Penalty".
#[test]
fn a_readied_agile_strike_at_count_2_takes_minus_8() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 18]);
    open(&mut encounter, 1);
    encounter.set_attack_count_for_test(id(1), 2);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::Agile, 0, 0),
    );
    open(&mut encounter, 2);
    let strike = strike_numbers(&fire_begins(&mut encounter, 2, &mut dice));
    assert_eq!(strike.map, -8);
    assert_eq!(strike.total, 18);
    assert_eq!(strike.degree, DegreeOfSuccess::Success);
    assert_ne!(strike.map, -10);
}

/// Player Core, "Ready".
#[test]
fn a_hit_spends_the_reaction_only() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::NotAgile, 0, 1),
    );
    open(&mut encounter, 2);
    let _ = fire_begins(&mut encounter, 2, &mut dice);
    assert!(!reaction(&encounter, 1));
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
    assert_eq!(actions(&encounter, 1), 0);
    assert_eq!(actions(&encounter, 2), 3);
    assert_eq!(encounter.turn_open(), Some(id(2)));
}

/// Player Core, "Ready".
#[test]
fn a_miss_still_spends_the_reaction() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 1]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::NotAgile, 0, 0),
    );
    open(&mut encounter, 2);
    let strike = strike_numbers(&fire_begins(&mut encounter, 2, &mut dice));
    assert_eq!(strike.total, 9);
    assert_eq!(strike.degree, DegreeOfSuccess::CriticalFailure);
    assert_eq!(strike.damage, 0);
    assert!(!reaction(&encounter, 1));
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
}

/// Player Core, "Ready".
#[test]
fn no_reaction_leaves_the_ready_stored() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::NotAgile, 0, 0),
    );
    encounter.spend_reaction(id(1)).expect("spend");
    let resolved = fire_begins(&mut encounter, 2, &mut dice);
    assert_eq!(one(&resolved), &Err(ReadyError::NoReaction));
    assert!(encounter.stored_ready(id(1)).expect("ready").is_some());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready". Player Core, "Getting Knocked Out".
#[test]
fn dying_keeps_baked_ac_on_a_readied_miss() {
    let (mut encounter, mut dice) = begin(
        vec![
            pc(1, 0, 20, 18),
            participant(2, Side::Pc, creature(20, 18, 0, true, None)),
        ],
        &[15, 10, 4],
    );
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::NotAgile, 0, 0),
    );
    lay(&mut encounter, 2, dying_at(1), 0);
    let strike = strike_numbers(&fire_begins(&mut encounter, 2, &mut dice));
    assert_eq!(strike.map, 0);
    assert_eq!(strike.total, 12);
    assert_eq!(strike.degree, DegreeOfSuccess::Failure);
    assert_eq!(vital(&encounter, 2), dying_at(1));
    assert!(!reaction(&encounter, 1));
}

/// Player Core, "Getting Knocked Out". Player Core, "Ready".
#[test]
fn damage_to_dying_1_makes_it_dying_2() {
    let (mut encounter, mut dice) = begin(
        vec![
            pc(1, 0, 20, 18),
            participant(2, Side::Pc, creature(20, 18, 0, true, None)),
        ],
        &[15, 10, 10],
    );
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::NotAgile, 0, 1),
    );
    lay(&mut encounter, 2, dying_at(1), 0);
    let strike = strike_numbers(&fire_begins(&mut encounter, 2, &mut dice));
    assert_eq!(strike.total, 18);
    assert_eq!(strike.degree, DegreeOfSuccess::Success);
    assert_eq!(vital(&encounter, 2), dying_at(2));
}

/// Player Core, "Getting Knocked Out". Player Core, "Ready".
#[test]
fn damage_to_unconscious_at_0_hp_is_a_knockout() {
    let (mut encounter, mut dice) = begin(
        vec![
            pc(1, 0, 20, 18),
            participant(2, Side::Pc, creature(20, 18, 0, true, None)),
        ],
        &[15, 10, 10],
    );
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::NotAgile, 0, 1),
    );
    lay(&mut encounter, 2, Vital::Unconscious, 0);
    let _ = fire_begins(&mut encounter, 2, &mut dice);
    assert_eq!(vital(&encounter, 2), dying_at(1));
    assert_eq!(hit_points(&encounter, 2), 0);
}

/// Player Core, "Ready".
#[test]
fn a_dead_target_spends_nothing() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::NotAgile, 0, 0),
    );
    lay(&mut encounter, 2, Vital::Dead, 0);
    let resolved = fire_begins(&mut encounter, 2, &mut dice);
    assert_eq!(one(&resolved), &Err(ReadyError::TargetDead));
    assert!(reaction(&encounter, 1));
    assert!(encounter.stored_ready(id(1)).expect("ready").is_some());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready". Player Core, Delay.
#[test]
fn a_delay_does_not_rewrite_a_stored_count() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10, 10]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    encounter
        .strike(
            id(2),
            id(1),
            &profile(8, Agility::NotAgile, 0, 8, 0),
            &[],
            ShieldBlock::Declined,
            &mut dice,
        )
        .expect("strike");
    ready_on(
        &mut encounter,
        2,
        reach(1),
        melee(1, Agility::NotAgile, 0, 0),
    );
    encounter.return_from_delay(&[id(1)]).expect("return");
    let stored = encounter.stored_ready(id(2)).expect("ready").expect("some");
    assert_eq!(stored.activity, melee(1, Agility::NotAgile, 0, 0));
    assert_eq!(
        encounter.frozen_attack_count(id(2)).expect("frozen"),
        Some(1)
    );
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready". Player Core, "Hidden and Undetected".
#[test]
fn a_readied_strike_draws_no_flat_check() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10, 5]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::NotAgile, 0, 0),
    );
    let strike = strike_numbers(&fire_begins(&mut encounter, 2, &mut dice));
    assert_eq!(strike.total, 18);
    assert_eq!(strike.degree, DegreeOfSuccess::Success);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready". Player Core, "Getting Knocked Out".
#[test]
fn a_knockout_abandons_the_triggering_strike() {
    let (mut encounter, mut dice) = begin(
        vec![
            pc(1, 0, 20, 18),
            participant(2, Side::Pc, creature(9, 18, 0, true, None)),
        ],
        &[15, 10, 10, 8, 4],
    );
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::NotAgile, 1, 1),
    );
    open(&mut encounter, 2);
    let _ = fire_begins(&mut encounter, 2, &mut dice);
    assert_eq!(vital(&encounter, 2), dying_at(1));
    assert_eq!(hit_points(&encounter, 2), 0);
    assert!(!reaction(&encounter, 1));
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
    assert_eq!(
        encounter
            .strike(
                id(2),
                id(1),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect_err("abandoned"),
        EncounterError::NotConscious
    );
    assert_eq!(actions(&encounter, 2), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready". Player Core, "Getting Knocked Out".
#[test]
fn a_knockout_abandons_the_triggering_stride() {
    let (mut encounter, mut dice) = begin(
        vec![
            pc(1, 0, 20, 18),
            participant(2, Side::Pc, creature(9, 18, 0, true, None)),
        ],
        &[15, 10, 10, 8, 4],
    );
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Stride),
        melee(2, Agility::NotAgile, 1, 1),
    );
    open(&mut encounter, 2);
    let resolved = offer(
        &mut encounter,
        2,
        ObservedAction::Stride,
        sq(0, 1),
        sq(0, 1),
        &[],
        &mut dice,
    );
    let _ = strike_numbers(&resolved);
    assert_eq!(vital(&encounter, 2), dying_at(1));
    assert_eq!(hit_points(&encounter, 2), 0);
    assert!(!reaction(&encounter, 1));
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
    assert_eq!(
        encounter
            .stride(
                id(2),
                &grid(&places(2, sq(0, 1), None)),
                &[sq(1, 1)],
                DiagonalCounter::fresh(),
            )
            .expect_err("stride"),
        EncounterError::NotConscious
    );
    assert_eq!(actions(&encounter, 2), 3);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready".
#[test]
fn a_dead_readier_spends_nothing() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::NotAgile, 0, 0),
    );
    lay(&mut encounter, 1, Vital::Dead, 0);
    let resolved = fire_begins(&mut encounter, 2, &mut dice);
    assert_eq!(one(&resolved), &Err(ReadyError::NotConscious));
    assert!(reaction(&encounter, 1));
    assert!(encounter.stored_ready(id(1)).expect("ready").is_some());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready".
#[test]
fn the_first_readied_reaction_kills_and_the_second_keeps_ready() {
    let (mut encounter, mut dice) = begin(
        vec![
            enemy(1, 0, 20, 18),
            pc(2, 0, 20, 18),
            participant(3, Side::Pc, creature(9, 18, 0, false, None)),
        ],
        &[15, 10, 5, 10, 8, 12],
    );
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(3, ObservedAction::Strike),
        melee(3, Agility::NotAgile, 1, 1),
    );
    open(&mut encounter, 2);
    ready_on(
        &mut encounter,
        2,
        begins(3, ObservedAction::Strike),
        melee(3, Agility::NotAgile, 1, 1),
    );
    let resolved = offer(
        &mut encounter,
        3,
        ObservedAction::Strike,
        sq(0, 1),
        sq(0, 1),
        &[],
        &mut dice,
    );
    assert_eq!(vital(&encounter, 3), Vital::Dead);
    assert_eq!(hit_points(&encounter, 3), 0);
    assert!(!reaction(&encounter, 1));
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
    assert_eq!(
        resolved.get(1),
        Some(&ResolvedReaction::Ready {
            id: id(2),
            result: Err(ReadyError::TargetDead),
        })
    );
    assert!(reaction(&encounter, 2));
    assert!(encounter.stored_ready(id(2)).expect("ready").is_some());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready". Player Core, "Step 1: Roll Initiative".
#[test]
fn an_enemy_reaction_resolves_before_a_tied_pc() {
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 18), pc(3, 0, 30, 18), enemy(4, 0, 20, 18)],
        &[12, 5, 12, 10, 10],
    );
    assert_eq!(initiative(&encounter, 4), 12);
    assert_eq!(initiative(&encounter, 1), 12);
    assert_eq!(encounter.order(), order_of(&[4, 1, 3]).as_slice());
    open(&mut encounter, 4);
    ready_on(
        &mut encounter,
        4,
        begins(3, ObservedAction::Strike),
        melee(3, Agility::NotAgile, 0, 1),
    );
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(3, ObservedAction::Strike),
        melee(3, Agility::NotAgile, 0, 1),
    );
    let resolved = offer(
        &mut encounter,
        3,
        ObservedAction::Strike,
        sq(0, 1),
        sq(0, 1),
        &[],
        &mut dice,
    );
    assert_eq!(hit_points(&encounter, 3), 28);
    assert_eq!(
        resolved
            .iter()
            .map(|step| match step {
                ResolvedReaction::Ready { id, .. } => *id,
                ResolvedReaction::ReactiveStrike { id, .. } => *id,
            })
            .collect::<Vec<_>>(),
        order_of(&[4, 1])
    );
    assert!(!reaction(&encounter, 4));
    assert!(!reaction(&encounter, 1));
}

/// Player Core, "Ready". Player Core, "Step 1: Roll Initiative".
#[test]
fn a_lower_id_resolves_before_a_tied_higher_id() {
    let (mut encounter, mut dice) = begin(
        vec![pc(1, 0, 20, 18), pc(3, 0, 30, 18), pc(4, 0, 20, 18)],
        &[12, 5, 12, 10, 10],
    );
    assert_eq!(encounter.order(), order_of(&[1, 4, 3]).as_slice());
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(3, ObservedAction::Strike),
        melee(3, Agility::NotAgile, 0, 1),
    );
    open(&mut encounter, 4);
    ready_on(
        &mut encounter,
        4,
        begins(3, ObservedAction::Strike),
        melee(3, Agility::NotAgile, 0, 1),
    );
    let resolved = offer(
        &mut encounter,
        3,
        ObservedAction::Strike,
        sq(0, 1),
        sq(0, 1),
        &[],
        &mut dice,
    );
    assert_eq!(
        resolved
            .iter()
            .map(|step| match step {
                ResolvedReaction::Ready { id, .. } => *id,
                ResolvedReaction::ReactiveStrike { id, .. } => *id,
            })
            .collect::<Vec<_>>(),
        order_of(&[1, 4])
    );
}

/// Player Core, "Ready". Player Core, "Reactive Strike".
#[test]
fn a_reactive_strike_and_a_ready_resolve_in_one_call() {
    let (mut encounter, mut dice) = begin(
        vec![enemy(1, 0, 20, 18), pc(2, 0, 20, 18), pc(3, 0, 20, 18)],
        &[15, 10, 5, 10, 10],
    );
    open(&mut encounter, 1);
    close(&mut encounter);
    open(&mut encounter, 2);
    ready_on(
        &mut encounter,
        2,
        begins(3, ObservedAction::Stride),
        melee(3, Agility::NotAgile, 0, 1),
    );
    open(&mut encounter, 3);
    let input = crate::ready::ReactiveStrikeInput {
        id: id(1),
        availability: ReactiveStrike::Available,
        profile: profile(8, Agility::NotAgile, 0, 8, 1),
        modifiers: &[],
        reactor_size: Size::Medium,
        actor_size: Size::Medium,
        square: sq(0, 0),
    };
    let resolved = encounter.resolve_reactions(
        id(3),
        ObservedAction::Stride,
        moving(),
        sq(0, 1),
        sq(0, 1),
        &[],
        &grid(&places(3, sq(0, 1), None)),
        &[input],
        &mut dice,
    );
    assert_eq!(hit_points(&encounter, 3), 18);
    assert_eq!(
        resolved
            .iter()
            .map(|step| match step {
                ResolvedReaction::ReactiveStrike { id, .. } => *id,
                ResolvedReaction::Ready { id, .. } => *id,
            })
            .collect::<Vec<_>>(),
        order_of(&[1, 2])
    );
    assert!(!reaction(&encounter, 1));
    assert!(!reaction(&encounter, 2));
    assert_eq!(actions(&encounter, 3), 3);
    assert_eq!(
        resolved.iter().filter(|step| matches!(step, ResolvedReaction::ReactiveStrike { id: reactor, .. } if *reactor == id(2))).count(),
        0
    );
}

/// Player Core, "Ready".
#[test]
fn no_stored_ready_is_rejected_before_a_dead_target() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    lay(&mut encounter, 2, Vital::Dead, 0);
    let resolved = fire_begins(&mut encounter, 2, &mut dice);
    assert!(resolved.is_empty());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready". Player Core, Delay.
#[test]
fn the_wrong_action_is_rejected_before_a_delayed_readier() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    encounter.set_ready_for_test(
        id(1),
        StoredReady {
            trigger: begins(2, ObservedAction::Strike),
            activity: melee(2, Agility::NotAgile, 0, 0),
            frozen_attack_count: 0,
        },
    );
    assert_eq!(
        encounter
            .check_ready_fire(
                id(1),
                id(2),
                ObservedAction::Stride,
                sq(0, 1),
                sq(0, 1),
                &[],
                &grid(&places(2, sq(0, 1), None)),
                ActivityKind::Stride,
            )
            .expect_err("wrong"),
        ReadyError::WrongAction
    );
    assert!(encounter.stored_ready(id(1)).expect("ready").is_some());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready". Player Core, Delay.
#[test]
fn a_delayed_readier_is_rejected_before_the_trigger_is_matched() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    encounter.set_ready_for_test(
        id(1),
        StoredReady {
            trigger: begins(2, ObservedAction::Strike),
            activity: melee(2, Agility::NotAgile, 0, 0),
            frozen_attack_count: 0,
        },
    );
    let resolved = fire_begins(&mut encounter, 2, &mut dice);
    assert_eq!(one(&resolved), &Err(ReadyError::Delayed));
    assert!(encounter.stored_ready(id(1)).expect("ready").is_some());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready".
#[test]
fn a_dying_readier_is_rejected_before_a_dead_target() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::NotAgile, 0, 0),
    );
    assert!(reaction(&encounter, 1));
    lay(&mut encounter, 1, dying_at(1), 0);
    lay(&mut encounter, 2, Vital::Dead, 0);
    let resolved = fire_begins(&mut encounter, 2, &mut dice);
    assert_eq!(one(&resolved), &Err(ReadyError::NotConscious));
    assert!(reaction(&encounter, 1));
    assert!(encounter.stored_ready(id(1)).expect("ready").is_some());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready".
#[test]
fn a_dead_target_is_rejected_before_a_missing_reaction() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::NotAgile, 0, 0),
    );
    encounter.spend_reaction(id(1)).expect("spend");
    lay(&mut encounter, 2, Vital::Dead, 0);
    let resolved = fire_begins(&mut encounter, 2, &mut dice);
    assert_eq!(one(&resolved), &Err(ReadyError::TargetDead));
    assert!(encounter.stored_ready(id(1)).expect("ready").is_some());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready". Player Core, "Take Cover".
#[test]
fn dismiss_drops_to_no_cover() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    encounter
        .ready(
            id(1),
            reach(2),
            crate::positioning::dismiss_cost(),
            ReadyActivity::Dismiss,
        )
        .expect("ready");
    let resolved = offer_cover(
        &mut encounter,
        2,
        ObservedAction::Step,
        sq(0, 2),
        sq(0, 1),
        &[sq(0, 1)],
        CoverBenefit::Standard,
        &mut dice,
    );
    assert_eq!(one(&resolved), &Ok(ReadySuccess::Dismiss { benefit: None }));
    assert!(!reaction(&encounter, 1));
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready". Player Core, "Range and Reach".
#[test]
fn entering_a_square_within_5_feet_fires() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        reach(2),
        melee(2, Agility::NotAgile, 0, 0),
    );
    let resolved = offer(
        &mut encounter,
        2,
        ObservedAction::Stride,
        sq(0, 2),
        sq(0, 1),
        &[],
        &mut dice,
    );
    let strike = strike_numbers(&resolved);
    assert_eq!(strike.map, 0);
    assert_eq!(strike.total, 18);
    assert_eq!(strike.degree, DegreeOfSuccess::Success);
    assert!(!reaction(&encounter, 1));
}

/// Player Core, "Ready". Player Core, "Range and Reach".
#[test]
fn entering_a_square_15_feet_away_does_not_fire() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        reach(2),
        melee(2, Agility::NotAgile, 0, 0),
    );
    let resolved = offer(
        &mut encounter,
        2,
        ObservedAction::Stride,
        sq(0, 4),
        sq(0, 3),
        &[],
        &mut dice,
    );
    assert_eq!(one(&resolved), &Err(ReadyError::TriggerMismatch));
    assert!(reaction(&encounter, 1));
    assert!(encounter.stored_ready(id(1)).expect("ready").is_some());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready".
#[test]
fn moving_inside_reach_does_not_fire() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        reach(2),
        melee(2, Agility::NotAgile, 0, 0),
    );
    let resolved = offer(
        &mut encounter,
        2,
        ObservedAction::Step,
        sq(0, 1),
        sq(1, 0),
        &[],
        &mut dice,
    );
    assert_eq!(one(&resolved), &Err(ReadyError::TriggerMismatch));
    assert!(reaction(&encounter, 1));
    assert!(encounter.stored_ready(id(1)).expect("ready").is_some());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready".
#[test]
fn a_step_into_reach_fires() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    encounter
        .ready(
            id(1),
            reach(2),
            crate::positioning::dismiss_cost(),
            ReadyActivity::Dismiss,
        )
        .expect("ready");
    let resolved = offer_cover(
        &mut encounter,
        2,
        ObservedAction::Step,
        sq(0, 2),
        sq(0, 1),
        &[sq(0, 1)],
        CoverBenefit::Standard,
        &mut dice,
    );
    assert_eq!(one(&resolved), &Ok(ReadySuccess::Dismiss { benefit: None }));
    assert!(!reaction(&encounter, 1));
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready".
#[test]
fn beginning_a_strike_fires() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::NotAgile, 0, 0),
    );
    let strike = strike_numbers(&fire_begins(&mut encounter, 2, &mut dice));
    assert_eq!(strike.map, 0);
    assert_eq!(strike.total, 18);
    assert_eq!(strike.degree, DegreeOfSuccess::Success);
    assert!(!reaction(&encounter, 1));
}

/// Player Core, "Ready". Player Core, Seek.
#[test]
fn beginning_a_seek_does_not_fire() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::NotAgile, 0, 0),
    );
    let resolved = offer(
        &mut encounter,
        2,
        ObservedAction::Other,
        sq(0, 1),
        sq(0, 1),
        &[],
        &mut dice,
    );
    assert_eq!(one(&resolved), &Err(ReadyError::TriggerMismatch));
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready". Player Core, "Range and Reach".
#[test]
fn a_begun_strike_out_of_reach_spends_nothing() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        melee(2, Agility::NotAgile, 0, 0),
    );
    let resolved = offer(
        &mut encounter,
        2,
        ObservedAction::Strike,
        sq(0, 3),
        sq(0, 3),
        &[],
        &mut dice,
    );
    assert_eq!(one(&resolved), &Err(ReadyError::OutOfReach));
    assert!(reaction(&encounter, 1));
    assert!(encounter.stored_ready(id(1)).expect("ready").is_some());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready". Player Core, "Range Penalty".
#[test]
fn a_readied_ranged_strike_in_the_second_increment_still_fires() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 12]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        ranged(2, 10),
    );
    let strike = strike_numbers(&offer(
        &mut encounter,
        2,
        ObservedAction::Strike,
        sq(0, 3),
        sq(0, 3),
        &[],
        &mut dice,
    ));
    assert_eq!(strike.range_penalty, -2);
    assert_eq!(strike.total, 18);
    assert_eq!(strike.degree, DegreeOfSuccess::Success);
    assert!(!reaction(&encounter, 1));
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
}

/// Player Core, "Ready". Player Core, "Range Penalty".
#[test]
fn a_readied_ranged_strike_past_six_increments_spends_nothing() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        ranged(2, 5),
    );
    let resolved = offer(
        &mut encounter,
        2,
        ObservedAction::Strike,
        sq(0, 7),
        sq(0, 7),
        &[],
        &mut dice,
    );
    assert_eq!(
        one(&resolved),
        &Err(ReadyError::Grid(GridError::BeyondRange))
    );
    assert!(reaction(&encounter, 1));
    assert!(encounter.stored_ready(id(1)).expect("ready").is_some());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready". Player Core, "Stride".
#[test]
fn an_orthogonal_stride_spends_the_reaction() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        ReadyActivity::Stride,
    );
    let held = seen_diagonal();
    let resolved = offer(
        &mut encounter,
        2,
        ObservedAction::Strike,
        sq(0, 1),
        sq(0, 1),
        &[sq(1, 0)],
        &mut dice,
    );
    match one(&resolved) {
        Ok(ReadySuccess::Move { end, distance_feet }) => {
            assert_eq!(*end, sq(1, 0));
            assert_eq!(*distance_feet, 5);
        }
        other => panic!("move {other:?}"),
    }
    assert!(!reaction(&encounter, 1));
    assert_eq!(held.diagonals(), 1);
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready". Player Core, "Diagonal Movement".
#[test]
fn the_first_diagonal_of_a_readied_stride_costs_5() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        ReadyActivity::Stride,
    );
    let resolved = offer(
        &mut encounter,
        2,
        ObservedAction::Strike,
        sq(0, 1),
        sq(0, 1),
        &[sq(1, 1)],
        &mut dice,
    );
    match one(&resolved) {
        Ok(ReadySuccess::Move { end, distance_feet }) => {
            assert_eq!(*distance_feet, 5);
            assert_eq!(*end, sq(1, 1));
        }
        other => panic!("move {other:?}"),
    }
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready". Player Core, "Stride".
#[test]
fn a_path_beyond_speed_spends_nothing() {
    let (mut encounter, mut dice) = begin(vec![slow(1), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        ReadyActivity::Stride,
    );
    let resolved = offer(
        &mut encounter,
        2,
        ObservedAction::Strike,
        sq(0, 1),
        sq(0, 1),
        &[sq(1, 0), sq(2, 0)],
        &mut dice,
    );
    assert_eq!(
        one(&resolved),
        &Err(ReadyError::Grid(GridError::ExceedsSpeed))
    );
    assert!(reaction(&encounter, 1));
    assert!(encounter.stored_ready(id(1)).expect("ready").is_some());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready". Player Core, "Step".
#[test]
fn a_readied_step_moves_one_square() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        ReadyActivity::Step,
    );
    let resolved = offer(
        &mut encounter,
        2,
        ObservedAction::Strike,
        sq(0, 1),
        sq(0, 1),
        &[sq(1, 0)],
        &mut dice,
    );
    match one(&resolved) {
        Ok(ReadySuccess::Move { end, distance_feet }) => {
            assert_eq!(*end, sq(1, 0));
            assert_eq!(*distance_feet, 5);
        }
        other => panic!("step {other:?}"),
    }
    assert!(!reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Multiple Attack Penalty". Player Core, "Ready".
#[test]
fn a_readied_attack_keeps_the_stored_count_off_the_open_turn() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 4]);
    open(&mut encounter, 1);
    encounter.set_attack_count_for_test(id(1), 2);
    ready_on(
        &mut encounter,
        1,
        reach(2),
        melee(2, Agility::NotAgile, 0, 0),
    );
    assert_eq!(encounter.turn_open(), None);
    assert_eq!(
        encounter.readied_attack_penalty(id(1)).expect("penalty"),
        -10
    );
    assert_eq!(dice.remaining(), 1);
    assert_eq!(
        encounter.frozen_attack_count(id(1)).expect("frozen"),
        Some(2)
    );
    assert_eq!(encounter.attack_count(id(1)).expect("count"), 2);
    assert_eq!(actions(&encounter, 1), 0);
}

/// Player Core, "Shield Block".
#[test]
fn a_dying_defender_is_targetnotconscious_on_both_block_paths() {
    let (mut encounter, mut dice) =
        begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10, 5]);
    open(&mut encounter, 1);
    close(&mut encounter);
    open(&mut encounter, 2);
    lay(&mut encounter, 1, dying_at(1), 0);
    let before = encounter.clone();
    assert_eq!(
        encounter
            .shield_block(id(1), 5, DegreeOfSuccess::Success)
            .expect_err("direct"),
        EncounterError::TargetNotConscious
    );
    assert_eq!(
        encounter
            .strike(
                id(2),
                id(1),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Requested,
                &mut dice,
            )
            .expect_err("strike"),
        EncounterError::TargetNotConscious
    );
    assert_eq!(encounter, before);
    assert_eq!(actions(&encounter, 2), 3);
    assert_eq!(dice.remaining(), 2);
}

/// Player Core, "Shield Block". Player Core, Delay.
#[test]
fn shield_block_on_a_delayed_creature_with_no_shield_is_delayed() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 30, 18), pc(2, 0, 20, 18)], &[15, 10]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    assert_eq!(
        encounter
            .shield_block(id(1), 5, DegreeOfSuccess::Success)
            .expect_err("block"),
        EncounterError::Delayed
    );
    assert!(reaction(&encounter, 1));
    assert_eq!(slot(&encounter, 1), TurnSlot::Delayed);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, "Shield Block". Player Core, Delay.
#[test]
fn shield_block_on_a_delayed_creature_with_a_lowered_shield_is_delayed() {
    let (mut encounter, dice) = begin(
        vec![
            participant(1, Side::Pc, creature(30, 18, 0, false, Some(steel()))),
            pc(2, 0, 20, 18),
        ],
        &[15, 10],
    );
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    assert!(!encounter.state(id(1)).expect("state").shield_raised);
    assert_eq!(
        encounter
            .shield_block(id(1), 5, DegreeOfSuccess::Success)
            .expect_err("block"),
        EncounterError::Delayed
    );
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, "Drop Prone".
#[test]
fn an_unconscious_creature_cannot_drop_prone() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18)], &[10]);
    lay(&mut encounter, 1, Vital::Unconscious, 0);
    assert_eq!(
        encounter
            .drop_prone(id(1), Posture::Standing, sq(0, 0), Mobility::Free, None,)
            .expect_err("prone"),
        EncounterError::NotConscious
    );
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, "Drop Prone".
#[test]
fn a_conscious_creature_drops_prone() {
    let (encounter, dice) = begin(vec![pc(1, 0, 20, 18)], &[10]);
    let change = encounter
        .drop_prone(
            id(1),
            Posture::Standing,
            sq(1, 2),
            Mobility::Free,
            Some(CoverBenefit::Standard),
        )
        .expect("prone");
    assert_eq!(change.posture, Posture::Prone);
    assert_eq!(change.square, sq(1, 2));
    assert_eq!(change.benefit, Some(CoverBenefit::Standard));
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, "Stand".
#[test]
fn an_unconscious_creature_cannot_stand() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18)], &[10]);
    lay(&mut encounter, 1, Vital::Unconscious, 0);
    assert_eq!(
        encounter
            .stand(
                id(1),
                Posture::Prone,
                sq(0, 0),
                Mobility::Free,
                None,
                OccupantPresence::Absent,
            )
            .expect_err("stand"),
        EncounterError::NotConscious
    );
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, "Stand".
#[test]
fn a_conscious_creature_stands() {
    let (encounter, dice) = begin(vec![pc(1, 0, 20, 18)], &[10]);
    let change = encounter
        .stand(
            id(1),
            Posture::Prone,
            sq(2, 3),
            Mobility::Free,
            Some(CoverBenefit::Standard),
            OccupantPresence::Present,
        )
        .expect("stand");
    assert_eq!(change.posture, Posture::Standing);
    assert_eq!(change.square, sq(2, 3));
    assert_eq!(change.benefit, Some(CoverBenefit::Standard));
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, "Reactive Strike". Player Core, "Range and Reach".
#[test]
fn a_reactive_strike_measures_reach_at_the_square_left() {
    let (mut encounter, mut dice) = begin(
        vec![enemy(1, 0, 20, 18), pc(3, 0, 20, 18)],
        &[15, 10, 10, 3],
    );
    let input = crate::ready::ReactiveStrikeInput {
        id: id(1),
        availability: ReactiveStrike::Available,
        profile: profile(8, Agility::NotAgile, 1, 8, 4),
        modifiers: &[],
        reactor_size: Size::Medium,
        actor_size: Size::Medium,
        square: sq(0, 0),
    };
    let resolved = encounter.resolve_reactions(
        id(3),
        ObservedAction::Stride,
        moving(),
        sq(0, 1),
        sq(0, 4),
        &[],
        &grid(&places(3, sq(0, 4), None)),
        &[input],
        &mut dice,
    );
    assert_eq!(
        resolved,
        vec![ResolvedReaction::ReactiveStrike {
            id: id(1),
            result: Ok(()),
        }]
    );
    assert!(!reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Aid. Player Core, Delay.
#[test]
fn aid_checks_delayed_after_not_conscious() {
    {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                pc(2, 0, 20, 18),
            ],
            &[15, 10, 10],
        );
        open(&mut encounter, 1);
        encounter.delay(id(1)).expect("delay");
        lay(&mut encounter, 1, dying_at(1), 0);
        open(&mut encounter, 2);
        assert_eq!(
            encounter.aid(id(1), id(2), &mut dice).expect_err("aid"),
            EncounterError::NotConscious
        );
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }
    let (mut encounter, mut dice) = begin(
        vec![
            aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            pc(2, 0, 20, 18),
        ],
        &[15, 10, 10],
    );
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    assert_eq!(
        encounter.aid(id(1), id(2), &mut dice).expect_err("aid"),
        EncounterError::Delayed
    );
    assert_eq!(
        encounter.state(id(1)).expect("state").aid_preparation,
        AidPreparation::NotPrepared
    );
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Reactive Strike". Player Core, Delay.
#[test]
fn reactive_strike_checks_delayed_before_reach() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    lay(&mut encounter, 1, dying_at(1), 0);
    assert_eq!(
        encounter
            .reactive_strike(
                id(1),
                id(2),
                moving(),
                ActionIdentity::Other,
                Some(crate::positioning::MoveAction::Stride),
                ReactiveStrike::Unavailable,
                sq(0, 0),
                sq(0, 4),
                Size::Medium,
                Size::Medium,
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                &mut dice,
            )
            .expect_err("dying"),
        crate::reactive::ReactiveError::NotConscious
    );
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    assert_eq!(
        encounter
            .reactive_strike(
                id(1),
                id(2),
                moving(),
                ActionIdentity::Other,
                Some(crate::positioning::MoveAction::Stride),
                ReactiveStrike::Available,
                sq(0, 0),
                sq(0, 4),
                Size::Medium,
                Size::Medium,
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                &mut dice,
            )
            .expect_err("far"),
        crate::reactive::ReactiveError::Delayed
    );
    assert!(reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready". Player Core, Delay.
#[test]
fn ready_firing_checks_delayed_before_the_trigger() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    encounter.set_ready_for_test(
        id(1),
        StoredReady {
            trigger: begins(2, ObservedAction::Strike),
            activity: melee(2, Agility::NotAgile, 0, 0),
            frozen_attack_count: 0,
        },
    );
    lay(&mut encounter, 1, dying_at(1), 0);
    let resolved = offer(
        &mut encounter,
        2,
        ObservedAction::Stride,
        sq(0, 4),
        sq(0, 3),
        &[],
        &mut dice,
    );
    assert_eq!(one(&resolved), &Err(ReadyError::NotConscious));
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    encounter.set_ready_for_test(
        id(1),
        StoredReady {
            trigger: begins(2, ObservedAction::Strike),
            activity: melee(2, Agility::NotAgile, 0, 0),
            frozen_attack_count: 0,
        },
    );
    let resolved = offer(
        &mut encounter,
        2,
        ObservedAction::Stride,
        sq(0, 4),
        sq(0, 3),
        &[],
        &mut dice,
    );
    assert_eq!(one(&resolved), &Err(ReadyError::Delayed));
    assert!(encounter.stored_ready(id(1)).expect("ready").is_some());
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, Delay.
#[test]
fn later_returners_get_skipstarthook() {
    let (mut encounter, dice) = begin(
        vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18), pc(3, 0, 20, 18)],
        &[15, 10, 5],
    );
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay 1");
    open(&mut encounter, 2);
    encounter.delay(id(2)).expect("delay 2");
    open(&mut encounter, 3);
    close(&mut encounter);
    encounter
        .return_from_delay(&[id(2), id(1)])
        .expect("return");
    assert_eq!(slot(&encounter, 1), TurnSlot::Returned);
    assert_eq!(slot(&encounter, 2), TurnSlot::SkipStartHook);
    assert_eq!(encounter.turn_open(), Some(id(1)));
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, Delay.
#[test]
fn return_right_after_delay_writes_nothing() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10]);
    open(&mut encounter, 1);
    close(&mut encounter);
    open(&mut encounter, 2);
    encounter.delay(id(2)).expect("delay");
    let before = encounter.clone();
    assert_eq!(
        encounter.return_from_delay(&[id(2)]).expect_err("return"),
        EncounterError::TriggerNotMet
    );
    assert_eq!(encounter, before);
    assert_eq!(slot(&encounter, 2), TurnSlot::Delayed);
    assert_eq!(encounter.turn_open(), None);
    assert_eq!(encounter.round(), 1);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, "Step 2: Act". Player Core, Delay.
#[test]
fn next_start_skipping_a_dead_creature_at_the_maximum_round_is_roundoverflow() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10]);
    open(&mut encounter, 1);
    close(&mut encounter);
    lay(&mut encounter, 2, Vital::Dead, 0);
    encounter.set_round_for_test(u32::MAX);
    assert_eq!(
        encounter.next_start().expect_err("overflow"),
        EncounterError::RoundOverflow
    );
    assert_eq!(encounter.round(), u32::MAX);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, "Ready".
#[test]
fn ready_cost_is_two_concentrate_actions() {
    assert_eq!(
        crate::ready::ready_cost(),
        ActionCost::Actions {
            count: 2,
            traits: &[ActionTrait::Concentrate],
        }
    );
}

/// Player Core, Delay.
#[test]
fn a_returned_turn_ends_as_normal() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10]);
    open(&mut encounter, 1);
    encounter.delay(id(1)).expect("delay");
    open(&mut encounter, 2);
    close(&mut encounter);
    encounter.return_from_delay(&[id(1)]).expect("return");
    assert_eq!(slot(&encounter, 1), TurnSlot::Returned);
    close(&mut encounter);
    assert_eq!(slot(&encounter, 1), TurnSlot::Normal);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, "Stride". Player Core, "Getting Knocked Out".
#[test]
fn a_knocked_out_creature_with_a_bad_path_is_notconscious() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18)], &[10]);
    open(&mut encounter, 1);
    let path = [sq(1, 0), sq(2, 0), sq(3, 0), sq(4, 0), sq(5, 0), sq(6, 0)];
    let laid = places(1, sq(0, 0), None);
    let field = grid(&laid);
    assert_eq!(
        encounter
            .stride(id(1), &field, &path, DiagonalCounter::fresh())
            .expect_err("speed"),
        EncounterError::Grid(GridError::ExceedsSpeed)
    );
    lay(&mut encounter, 1, dying_at(1), 0);
    assert_eq!(
        encounter
            .stride(id(1), &field, &path, DiagonalCounter::fresh())
            .expect_err("down"),
        EncounterError::NotConscious
    );
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(dice.remaining(), 0);
}

/// Player Core, "Ready". Player Core, "Stride".
#[test]
fn a_readied_move_clears_the_ready() {
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10, 10]);
    open(&mut encounter, 1);
    ready_on(
        &mut encounter,
        1,
        begins(2, ObservedAction::Strike),
        ReadyActivity::Stride,
    );
    let resolved = offer(
        &mut encounter,
        2,
        ObservedAction::Strike,
        sq(0, 1),
        sq(0, 1),
        &[sq(1, 0)],
        &mut dice,
    );
    assert!(matches!(one(&resolved), Ok(ReadySuccess::Move { .. })));
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
    assert!(!reaction(&encounter, 1));
    assert_eq!(dice.remaining(), 1);
}

/// Player Core, "Ready".
#[test]
fn a_dying_creature_trying_to_ready_is_notconscious() {
    let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[15, 10]);
    open(&mut encounter, 1);
    lay(&mut encounter, 1, dying_at(1), 0);
    assert_eq!(
        encounter
            .ready(id(1), reach(2), cost(1), melee(2, Agility::NotAgile, 0, 0),)
            .expect_err("ready"),
        ReadyError::NotConscious
    );
    assert_eq!(actions(&encounter, 1), 3);
    assert_eq!(encounter.stored_ready(id(1)).expect("ready"), None);
    assert_eq!(dice.remaining(), 0);
}
