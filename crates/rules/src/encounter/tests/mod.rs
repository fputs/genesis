use super::*;
use crate::aid::{AidCircumstance, AidPreparation, AidRank};
use crate::check::{ArithmeticOverflow, DegreeOfSuccess};
use crate::combatant::{
    BuildParts, CharacterBuild, CombatantId, DieCount, DyingValue, HpError, MeleeStrike, ReachFeet,
    ShieldTemplate, Vital, derive, new_combatant_state,
};
use crate::detection::{Concealment, Detection, Detections, Relation, note_strike};
use crate::dice::{Faces, ScriptedDice};
use crate::dying::{InitiativeMove, apply_resolved_blow, take_initiative_move};
use crate::grid::{Size, Square};
use crate::modifiers::{BonusType, Modifier, ModifierError, PenaltyType, SourceId, Statistic};
use crate::positioning::{ActionCost, ActionTrait, MoveAction};
use crate::reactive::{ActionIdentity, Disruption, ReactiveError, ReactiveOutcome, ReactiveStrike};
use crate::shield::{self, ShieldBlock};

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
            damage_type: crate::damage::DamageType::Slashing,
        },
        shield,
        stealth_dc: None,
        aid_modifier: None,
        aid_rank: None,
        uses_dying_rules: Some(dying),
        saves: None,
        skills: None,
        defenses: crate::damage::DamageDefenses::empty(),
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
        damage_type: crate::damage::DamageType::Slashing,
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
    encounter.state(id(value)).expect("state").reaction_ready
}

fn rolled(delivery: StrikeDelivery) -> StrikeOutcome {
    match delivery {
        StrikeDelivery::Rolled(outcome) => outcome,
        StrikeDelivery::Lost(lost) => panic!("expected a rolled strike, lost {lost:?}"),
    }
}

fn delivered(outcome: &ReactiveOutcome) -> &StrikeOutcome {
    match &outcome.strike {
        StrikeDelivery::Rolled(strike) => strike,
        StrikeDelivery::Lost(lost) => panic!("expected a rolled strike, lost {lost:?}"),
    }
}

fn lost(delivery: StrikeDelivery) -> LostStrike {
    match delivery {
        StrikeDelivery::Lost(lost) => lost,
        StrikeDelivery::Rolled(outcome) => panic!("expected a lost strike, rolled {outcome:?}"),
    }
}

fn relate(
    detections: Detections,
    observer: u32,
    subject: u32,
    detection: Detection,
    concealment: Concealment,
    stealth_dc: Option<i32>,
) -> Detections {
    detections
        .store(
            id(observer),
            id(subject),
            Relation {
                detection,
                concealment,
            },
            stealth_dc,
        )
        .expect("relation")
}

fn plate() -> ShieldTemplate {
    ShieldTemplate {
        hardness: 5,
        hit_points: 20,
        broken_threshold: 10,
        shield_ac_bonus: 2,
    }
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

fn raised_defender(rolls: &[u8]) -> (Encounter, ScriptedDice) {
    let target = participant(2, Side::Pc, creature(20, 18, 0, false, Some(plate())));
    let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 18), target], rolls);
    assert_eq!(encounter.start_turn().expect("defender"), id(2));
    encounter.raise_shield(id(2)).expect("raise");
    encounter.end_turn(&mut dice).expect("end");
    assert_eq!(encounter.start_turn().expect("striker"), id(1));
    (encounter, dice)
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
    let outcome = rolled(
        encounter
            .strike(
                id(1),
                id(2),
                &profile(attack, Agility::NotAgile, dice_count, face, bonus),
                modifiers,
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect("strike"),
    );
    (outcome, dice.remaining())
}

fn take_report(encounter: &mut Encounter, value: u32) -> Option<InitiativeMove> {
    let member = encounter
        .members
        .iter_mut()
        .find(|member| member.participant.id == id(value))
        .expect("member");
    take_initiative_move(&mut member.participant.state)
}

fn aided(
    value: u32,
    side: Side,
    perception: i32,
    modifier: Option<i32>,
    rank: Option<AidRank>,
) -> Participant {
    let parts = BuildParts {
        hit_points: 20,
        armor_class: 11,
        perception,
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
        shield: None,
        stealth_dc: None,
        aid_modifier: modifier,
        aid_rank: rank,
        uses_dying_rules: Some(false),
        saves: None,
        skills: None,
        defenses: crate::damage::DamageDefenses::empty(),
    };
    participant(
        value,
        side,
        derive(&CharacterBuild::try_new(parts).expect("build")),
    )
}

fn preparation(encounter: &Encounter, value: u32) -> AidPreparation {
    encounter.state(id(value)).expect("state").aid_preparation
}

// `prepare_to_aid` spends an action and refuses a missing modifier, so those prepared states cannot be reached through it.
fn set_prepared(encounter: &mut Encounter, aider: u32, ally: u32) {
    let member = encounter
        .members
        .iter_mut()
        .find(|member| member.participant.id == id(aider))
        .expect("member");
    member.participant.state.aid_preparation = AidPreparation::Prepared { ally: id(ally) };
}

fn dying_value() -> DyingValue {
    DyingValue::new(1).expect("dying 1")
}

const MANIPULATE: &[ActionTrait] = &[ActionTrait::Manipulate];
const MOVE_ONLY: &[ActionTrait] = &[ActionTrait::Move];
const RANGED_ATTACK: &[ActionTrait] = &[ActionTrait::Ranged];
const MOVE_AND_MANIPULATE: &[ActionTrait] = &[ActionTrait::Move, ActionTrait::Manipulate];
const NO_TRIGGER: &[ActionTrait] = &[];

fn one_action(traits: &'static [ActionTrait]) -> ActionCost {
    ActionCost::Actions { count: 1, traits }
}

fn free_action(traits: &'static [ActionTrait]) -> ActionCost {
    ActionCost::Free { traits }
}

fn sq(x: i32, y: i32) -> Square {
    Square { x, y }
}

fn with_reach(mut participant: Participant, reach: i32) -> Participant {
    participant.creature.strike.reach_feet = ReachFeet::new(reach).expect("reach");
    participant
}

fn with_current(mut participant: Participant, hp: i32) -> Participant {
    participant.state.hit_points = hp;
    participant
}

#[expect(
    clippy::too_many_arguments,
    reason = "forwards reactive_strike's arguments so each scenario stays one call"
)]
fn react(
    encounter: &mut Encounter,
    dice: &mut ScriptedDice,
    reactor: u32,
    actor: u32,
    cost: ActionCost,
    identity: ActionIdentity,
    move_action: Option<MoveAction>,
    availability: ReactiveStrike,
    from: (i32, i32),
    to: (i32, i32),
    from_size: Size,
    to_size: Size,
    attack: StrikeProfile,
    modifiers: &[Modifier],
) -> Result<ReactiveOutcome, ReactiveError> {
    encounter.reactive_strike(
        id(reactor),
        id(actor),
        cost,
        identity,
        move_action,
        availability,
        sq(from.0, from.1),
        sq(to.0, to.1),
        from_size,
        to_size,
        &attack,
        modifiers,
        &Detections::new(),
        dice,
    )
}

fn react_seen(
    encounter: &mut Encounter,
    dice: &mut ScriptedDice,
    detections: &Detections,
    attack: StrikeProfile,
) -> Result<ReactiveOutcome, ReactiveError> {
    encounter.reactive_strike(
        id(1),
        id(2),
        one_action(MANIPULATE),
        ActionIdentity::Other,
        None,
        ReactiveStrike::Available,
        sq(0, 0),
        sq(0, 1),
        Size::Medium,
        Size::Medium,
        &attack,
        &[],
        detections,
        dice,
    )
}

fn standard_pair(reach: i32, actor_hp: i32, actor_ac: i32, actor_side: Side) -> Vec<Participant> {
    vec![
        with_reach(pc(1, 0, 20, actor_ac), reach),
        combatant(2, actor_side, 0, actor_hp, actor_ac, false),
    ]
}

mod damage_tests;
mod delay_ready_tests;
mod reactions_tests;
mod save_tests;
mod skill_tests;
mod strike_tests;
mod target_delay_tests;
mod turns_tests;
mod vitals_tests;
