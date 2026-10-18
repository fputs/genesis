//! Explore / combat session. Rules stay in `rules`; this module owns the mode machine.

use bevy::prelude::*;
use content::{load_encounter, EncounterFixture, Feature, Spawn};
use rules::{
    advance_if_turn_done, declare_action, end_turn, legal_actions, reactors_for_left_square,
    resolve_delay_resume, resolve_pending_aid, resolve_pending_shield_block, start_encounter,
    Action, ClassFeature, CombatantId, CreatureTemplate, DamageFormula, Encounter, Event, Gear,
    Grid, GridPos, Outcome, ReadyTrigger, Shield, Side, StrikeTemplate,
};

use crate::world::{cell_in_trigger, cell_to_world, member_world, snap_to_grid, world_to_cell};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, States)]
pub enum GameMode {
    #[default]
    Explore,
    TransitioningToCombat,
    Combat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriggerState {
    Armed,
    Spent,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Aim {
    #[default]
    None,
    Stride,
    Step,
    Strike,
    Crawl,
    Leap,
    PointOut,
    ReadyStride,
    ReadyStep,
    ReadyStrike,
}

/// Which Ready choice is open. Strike, Stride, and Step still need a target or square.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadyUi {
    Trigger,
    Action(ReadyTrigger),
}

#[derive(Resource)]
pub struct EncounterPath(pub std::path::PathBuf);

impl EncounterPath {
    pub fn fixture() -> Self {
        Self(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/encounters/room_one.json"),
        )
    }
}

#[derive(Resource, Default)]
pub struct ExploreDirection(pub Vec2);

#[derive(Resource, Default)]
pub struct PendingOrder(pub Option<Action>);

#[derive(Resource, Default)]
pub struct HudBump(pub u32);

#[derive(Resource)]
pub struct Session {
    pub fixture: EncounterFixture,
    pub encounter: Option<Encounter>,
    pub leader: usize,
    pub leader_world: Vec2,
    pub trigger: TriggerState,
    pub aim: Aim,
    pub modal: Option<String>,
    pub shield_prompt: bool,
    pub delay_prompt: bool,
    pub aid_prompt: bool,
    pub ready: Option<ReadyUi>,
    pub log: Vec<String>,
    pub rng: rules::SeededDice,
}

impl Session {
    pub fn member_world(&self, slot: usize) -> Vec2 {
        let (x, y) = member_world(
            (self.leader_world.x, self.leader_world.y),
            slot,
            self.leader,
        );
        Vec2::new(x, y)
    }

    pub fn member_cell(&self, slot: usize) -> (i32, i32) {
        let world = self.member_world(slot);
        world_to_cell(world.x, world.y)
    }

    pub fn push_log(&mut self, line: String) {
        self.log.push(line);
        if self.log.len() > 6 {
            let drain = self.log.len() - 6;
            self.log.drain(0..drain);
        }
    }
}

#[derive(Resource)]
pub struct EnemyClock(pub Timer);

impl Default for EnemyClock {
    fn default() -> Self {
        Self(Timer::from_seconds(0.45, TimerMode::Once))
    }
}

#[derive(Resource, Default)]
pub struct EndTurnRequest(pub bool);

/// `Some(true)` uses Shield Block. `Some(false)` declines it.
#[derive(Resource, Default)]
pub struct ShieldAnswer(pub Option<bool>);

/// `Some(true)` resumes a delayed turn. `Some(false)` keeps delaying.
#[derive(Resource, Default)]
pub struct DelayAnswer(pub Option<bool>);

/// `Some(true)` uses Aid. `Some(false)` declines it.
#[derive(Resource, Default)]
pub struct AidAnswer(pub Option<bool>);

pub struct SessionPlugin;

impl Plugin for SessionPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameMode>()
            .init_resource::<ExploreDirection>()
            .init_resource::<PendingOrder>()
            .init_resource::<EndTurnRequest>()
            .init_resource::<ShieldAnswer>()
            .init_resource::<DelayAnswer>()
            .init_resource::<AidAnswer>()
            .init_resource::<EnemyClock>()
            .init_resource::<EnemyTurnOrigin>()
            .init_resource::<HudBump>()
            .add_systems(Startup, load_session)
            .add_systems(
                Update,
                (explore_move, explore_trigger)
                    .chain()
                    .run_if(in_state(GameMode::Explore)),
            )
            .add_systems(OnEnter(GameMode::TransitioningToCombat), begin_combat)
            .add_systems(Update, combat_tick.run_if(in_state(GameMode::Combat)));
    }
}

pub fn load_session(path: Res<EncounterPath>, mut commands: Commands) {
    let fixture = load_encounter(&path.0).unwrap_or_else(|error| {
        panic!("load {}: {error}", path.0.display());
    });
    let origin = cell_to_world(fixture.party[0].x, fixture.party[0].y);
    let seed = fixture.rng_seed;
    info!(
        "loaded encounter {} ({}x{} room, seed {seed})",
        fixture.id, fixture.room.width, fixture.room.height
    );
    commands.insert_resource(Session {
        rng: rules::SeededDice::new(seed),
        leader_world: Vec2::new(origin.0, origin.1),
        fixture,
        encounter: None,
        leader: 0,
        trigger: TriggerState::Armed,
        aim: Aim::None,
        modal: None,
        shield_prompt: false,
        delay_prompt: false,
        aid_prompt: false,
        ready: None,
        log: vec!["The hall is quiet. Walk east.".into()],
    });
}

pub(crate) fn explore_move(
    time: Res<Time>,
    direction: Res<ExploreDirection>,
    mut session: ResMut<Session>,
) {
    if session.modal.is_some() {
        return;
    }
    if direction.0 == Vec2::ZERO {
        return;
    }
    let delta = direction.0.normalize_or_zero() * 140.0 * time.delta_secs();
    if delta == Vec2::ZERO {
        return;
    }
    let proposed = session.leader_world + delta;
    if formation_walkable(&session, proposed) {
        session.leader_world = proposed;
        return;
    }
    let slide_x = session.leader_world + Vec2::new(delta.x, 0.0);
    if formation_walkable(&session, slide_x) {
        session.leader_world = slide_x;
        return;
    }
    let slide_y = session.leader_world + Vec2::new(0.0, delta.y);
    if formation_walkable(&session, slide_y) {
        session.leader_world = slide_y;
    }
}

fn formation_walkable(session: &Session, leader_world: Vec2) -> bool {
    (0..4).all(|slot| {
        let (ox, oy) = member_world((leader_world.x, leader_world.y), slot, session.leader);
        let (cx, cy) = world_to_cell(ox, oy);
        session.fixture.room.walkable(cx, cy)
    })
}

fn explore_trigger(mut session: ResMut<Session>, mut next: ResMut<NextState<GameMode>>) {
    if session.trigger != TriggerState::Armed || session.encounter.is_some() {
        return;
    }
    let trigger = &session.fixture.trigger;
    let tripped = (0..4).any(|slot| {
        let (cx, cy) = session.member_cell(slot);
        cell_in_trigger(cx, cy, trigger.x, trigger.y, trigger.radius_feet)
    });
    if tripped {
        session.aim = Aim::None;
        session.push_log("Steel clears the scabbard.".into());
        next.set(GameMode::TransitioningToCombat);
    }
}

fn begin_combat(
    mut session: ResMut<Session>,
    mut next: ResMut<NextState<GameMode>>,
    mut hud: ResMut<HudBump>,
) {
    let party_cells = [
        session.member_cell(0),
        session.member_cell(1),
        session.member_cell(2),
        session.member_cell(3),
    ];
    let enemy = (session.fixture.enemies[0].x, session.fixture.enemies[0].y);
    let snapped = snap_to_grid(&session.fixture.room, party_cells, enemy);
    let (party_pos, enemy_pos) = match snapped {
        Ok(cells) => cells,
        Err(reason) => {
            session.modal = Some(reason);
            next.set(GameMode::Explore);
            hud.0 += 1;
            return;
        }
    };

    let party = [
        template_of(&session.fixture.party[0], Side::Party),
        template_of(&session.fixture.party[1], Side::Party),
        template_of(&session.fixture.party[2], Side::Party),
        template_of(&session.fixture.party[3], Side::Party),
    ];
    let enemies = vec![template_of(&session.fixture.enemies[0], Side::Enemy)];
    let party_grid = [
        GridPos::new(party_pos[0].0, party_pos[0].1),
        GridPos::new(party_pos[1].0, party_pos[1].1),
        GridPos::new(party_pos[2].0, party_pos[2].1),
        GridPos::new(party_pos[3].0, party_pos[3].1),
    ];
    let enemy_grid = [GridPos::new(enemy_pos.0, enemy_pos.1)];
    let grid = grid_of(&session.fixture);

    match start_encounter(
        &party,
        &enemies,
        &party_grid,
        &enemy_grid,
        grid,
        &mut session.rng,
    ) {
        Ok(encounter) => {
            let names: Vec<String> = encounter
                .initiative()
                .iter()
                .map(|id| {
                    let combatant = encounter.combatant(*id).unwrap();
                    format!("{} {}", combatant.name, combatant.initiative_total)
                })
                .collect();
            session.push_log(format!("Initiative: {}.", names.join(", ")));
            if let Some(active) = encounter.active() {
                let name = encounter.combatant(active).unwrap().name.clone();
                session.push_log(format!("{name}'s turn. Three actions."));
            }
            session.encounter = Some(encounter);
            session.trigger = TriggerState::Spent;
            session.aim = Aim::None;
            hud.0 += 1;
            next.set(GameMode::Combat);
        }
        Err(error) => {
            session.modal = Some(error.to_string());
            session.trigger = TriggerState::Armed;
            hud.0 += 1;
            next.set(GameMode::Explore);
        }
    }
}

fn combat_tick(
    time: Res<Time>,
    mut session: ResMut<Session>,
    mut pending: ResMut<PendingOrder>,
    mut end_turn_request: ResMut<EndTurnRequest>,
    mut shield_answer: ResMut<ShieldAnswer>,
    mut delay_answer: ResMut<DelayAnswer>,
    mut aid_answer: ResMut<AidAnswer>,
    mut clock: ResMut<EnemyClock>,
    mut turn_origin: ResMut<EnemyTurnOrigin>,
    mut next: ResMut<NextState<GameMode>>,
    mut hud: ResMut<HudBump>,
) {
    let Some(mut encounter) = session.encounter.take() else {
        return;
    };
    if encounter.outcome().is_some() {
        session.encounter = Some(encounter);
        return;
    }

    let mut events = Vec::new();
    if encounter.pending_aid().is_some() {
        let offer = encounter.pending_aid().unwrap();
        let party = encounter
            .combatant(offer.aider)
            .is_some_and(|combatant| combatant.side == Side::Party);
        let answer = if party {
            aid_answer.0.take()
        } else {
            Some(false)
        };
        if let Some(accept) = answer {
            match resolve_pending_aid(&mut encounter, accept, &mut session.rng) {
                Ok(resolved) => events.extend(resolved),
                Err(error) => session.modal = Some(error.to_string()),
            }
            if session.aid_prompt {
                session.aid_prompt = false;
                hud.0 += 1;
            }
        } else {
            if !session.aid_prompt {
                session.aid_prompt = true;
                hud.0 += 1;
            }
            session.encounter = Some(encounter);
            return;
        }
    }

    if encounter.pending_aid().is_some() {
        session.aid_prompt = encounter
            .pending_aid()
            .and_then(|offer| encounter.combatant(offer.aider))
            .is_some_and(|combatant| combatant.side == Side::Party);
        log_events(&mut session, &encounter, &events, &mut hud);
        session.encounter = Some(encounter);
        return;
    }

    if encounter.pending_shield_block().is_some() {
        let offer = encounter.pending_shield_block().unwrap();
        let party = encounter
            .combatant(offer.defender)
            .is_some_and(|combatant| combatant.side == Side::Party);
        let answer = if party {
            shield_answer.0.take()
        } else {
            Some(true)
        };
        if let Some(accept) = answer {
            match resolve_pending_shield_block(&mut encounter, accept, &mut session.rng) {
                Ok(resolved) => events.extend(resolved),
                Err(error) => session.modal = Some(error.to_string()),
            }
            if session.shield_prompt {
                session.shield_prompt = false;
                hud.0 += 1;
            }
        } else {
            if !session.shield_prompt {
                session.shield_prompt = true;
                hud.0 += 1;
            }
            session.encounter = Some(encounter);
            return;
        }
    }

    if encounter.pending_shield_block().is_some() {
        session.shield_prompt = encounter
            .pending_shield_block()
            .and_then(|offer| encounter.combatant(offer.defender))
            .is_some_and(|combatant| combatant.side == Side::Party);
        log_events(&mut session, &encounter, &events, &mut hud);
        session.encounter = Some(encounter);
        return;
    }

    match drain_delay(&mut encounter, &mut session, &mut delay_answer, &mut hud) {
        Ok(resolved) => events.extend(resolved),
        Err(()) => {
            log_events(&mut session, &encounter, &events, &mut hud);
            session.encounter = Some(encounter);
            return;
        }
    }

    events.extend(advance_if_turn_done(&mut encounter, &mut session.rng));
    let mut waiting = false;
    if events.is_empty() {
        if end_turn_request.0 {
            end_turn_request.0 = false;
            session.aim = Aim::None;
            session.ready = None;
            match end_turn(&mut encounter, &mut session.rng) {
                Ok(resolved) => events.extend(resolved),
                Err(_) => session.modal = Some("The turn will not end.".into()),
            }
        } else if let Some(active) = encounter.active() {
            let side = encounter.combatant(active).map(|combatant| combatant.side);
            if side == Some(Side::Party) {
                if let Some(action) = pending.0.take() {
                    match declare_action(&mut encounter, active, action, &mut session.rng) {
                        Ok(resolved) => {
                            session.aim = Aim::None;
                            session.ready = None;
                            events.extend(resolved);
                        }
                        Err(_) => session.modal = Some("That action is not legal.".into()),
                    }
                }
            } else if side == Some(Side::Enemy) {
                let left_square = left_square_this_turn(&encounter, active, &mut turn_origin);
                clock.0.tick(time.delta());
                if clock.0.finished() {
                    clock.0.reset();
                    if let Some(action) = enemy_action(&encounter, active, left_square) {
                        if let Ok(resolved) =
                            declare_action(&mut encounter, active, action, &mut session.rng)
                        {
                            events.extend(resolved);
                        }
                    } else if let Ok(resolved) = end_turn(&mut encounter, &mut session.rng) {
                        events.extend(resolved);
                    }
                } else {
                    waiting = true;
                }
            }
        }
    }
    if !waiting && encounter.outcome().is_none() {
        events.extend(advance_if_turn_done(&mut encounter, &mut session.rng));
    }
    if !waiting && encounter.outcome().is_none() && encounter.pending_shield_block().is_none() {
        match drain_delay(&mut encounter, &mut session, &mut delay_answer, &mut hud) {
            Ok(resolved) => events.extend(resolved),
            Err(()) => {
                log_events(&mut session, &encounter, &events, &mut hud);
                session.encounter = Some(encounter);
                return;
            }
        }
    }

    let outcome = encounter.outcome();
    if encounter.pending_shield_block().is_some() {
        let party = encounter
            .pending_shield_block()
            .and_then(|offer| encounter.combatant(offer.defender))
            .is_some_and(|combatant| combatant.side == Side::Party);
        if party && !session.shield_prompt {
            session.shield_prompt = true;
            hud.0 += 1;
        }
    } else if session.shield_prompt {
        session.shield_prompt = false;
        hud.0 += 1;
    }
    if encounter.pending_aid().is_some() {
        let party = encounter
            .pending_aid()
            .and_then(|offer| encounter.combatant(offer.aider))
            .is_some_and(|combatant| combatant.side == Side::Party);
        if party && !session.aid_prompt {
            session.aid_prompt = true;
            hud.0 += 1;
        }
    } else if session.aid_prompt {
        session.aid_prompt = false;
        hud.0 += 1;
    }
    log_events(&mut session, &encounter, &events, &mut hud);
    if outcome.is_some() {
        session.aim = Aim::None;
        session.modal = Some(match outcome {
            Some(Outcome::Victory) => "The guard falls. The hall is quiet.".into(),
            Some(Outcome::Defeat) => "The party is down.".into(),
            None => "The fight is over.".into(),
        });
        let anchor = encounter.party().into_iter().find_map(|id| {
            let combatant = encounter.combatant(id)?;
            combatant.is_conscious().then_some(combatant.position)
        });
        if let Some(pos) = anchor {
            let (x, y) = cell_to_world(pos.x, pos.y);
            session.leader_world = Vec2::new(x, y);
            session.leader = 0;
        }
        hud.0 += 1;
        next.set(GameMode::Explore);
    }
    session.encounter = Some(encounter);
}

#[derive(Clone, Copy)]
struct TurnOrigin {
    actor: CombatantId,
    round: u32,
    square: GridPos,
}

#[derive(Resource, Default)]
struct EnemyTurnOrigin(Option<TurnOrigin>);

fn left_square_this_turn(
    encounter: &Encounter,
    id: CombatantId,
    origin: &mut EnemyTurnOrigin,
) -> bool {
    let Some(actor) = encounter.combatant(id) else {
        return false;
    };
    let round = encounter.round();
    if let Some(start) = origin.0 {
        if start.actor == id && start.round == round {
            return actor.position != start.square;
        }
    }
    origin.0 = Some(TurnOrigin {
        actor: id,
        round,
        square: actor.position,
    });
    false
}

/// Finish a dying or unconscious enemy, then leave a threatened square once per turn, then Strike or close.
fn enemy_action(encounter: &Encounter, id: CombatantId, left_square: bool) -> Option<Action> {
    let legal = legal_actions(encounter, id);
    if let Some(action) = strike_unconscious(encounter, &legal) {
        return Some(action);
    }
    if !left_square {
        if let Some(action) = stride_out_of_reach(encounter, id, &legal) {
            return Some(action);
        }
    }
    if shield_is_down(encounter, id)
        && legal
            .iter()
            .any(|action| matches!(action, Action::RaiseShield))
    {
        return Some(Action::RaiseShield);
    }
    let mut strikes: Vec<CombatantId> = legal
        .iter()
        .filter_map(|action| match action {
            Action::Strike { target } => Some(*target),
            Action::Stride { .. }
            | Action::Step { .. }
            | Action::Crawl { .. }
            | Action::Leap { .. }
            | Action::DropProne
            | Action::Stand
            | Action::RaiseShield
            | Action::Seek
            | Action::PointOut { .. }
            | Action::TakeCover
            | Action::Release { .. }
            | Action::Interact
            | Action::Delay
            | Action::Ready { .. } => None,
        })
        .collect();
    strikes.sort_by_key(|target| {
        let hp = encounter
            .combatant(*target)
            .map(|c| c.hit_points)
            .unwrap_or(i32::MAX);
        (hp, target.0)
    });
    if let Some(target) = strikes.first().copied() {
        return Some(Action::Strike { target });
    }

    let actor = encounter.combatant(id)?;
    let living: Vec<GridPos> = encounter
        .party()
        .into_iter()
        .filter_map(|slot| {
            let combatant = encounter.combatant(slot)?;
            combatant.is_conscious().then_some(combatant.position)
        })
        .collect();
    let here = living
        .iter()
        .map(|pos| rules::minimum_feet(actor.position, *pos))
        .min()
        .unwrap_or(u32::MAX);
    legal
        .into_iter()
        .filter_map(|action| match action {
            Action::Stride { destination } => Some(destination),
            Action::Step { .. }
            | Action::Crawl { .. }
            | Action::Leap { .. }
            | Action::Strike { .. }
            | Action::DropProne
            | Action::Stand
            | Action::RaiseShield
            | Action::Seek
            | Action::PointOut { .. }
            | Action::TakeCover
            | Action::Release { .. }
            | Action::Interact
            | Action::Delay
            | Action::Ready { .. } => None,
        })
        .filter_map(|destination| {
            let dist = living
                .iter()
                .map(|pos| rules::minimum_feet(destination, *pos))
                .min()
                .unwrap_or(u32::MAX);
            (dist < here).then_some((dist, destination))
        })
        .min_by_key(|(dist, destination)| (*dist, destination.x, destination.y))
        .map(|(_, destination)| Action::Stride { destination })
}

fn strike_unconscious(encounter: &Encounter, legal: &[Action]) -> Option<Action> {
    legal.iter().copied().find(|action| match action {
        Action::Strike { target } => encounter
            .combatant(*target)
            .is_some_and(|combatant| combatant.is_unconscious()),
        Action::Stride { .. }
        | Action::Step { .. }
        | Action::Crawl { .. }
        | Action::Leap { .. }
        | Action::DropProne
        | Action::Stand
        | Action::RaiseShield
        | Action::Seek
        | Action::PointOut { .. }
        | Action::TakeCover
        | Action::Release { .. }
        | Action::Interact
        | Action::Delay
        | Action::Ready { .. } => false,
    })
}

fn stride_out_of_reach(encounter: &Encounter, id: CombatantId, legal: &[Action]) -> Option<Action> {
    let (square, side) = {
        let actor = encounter.combatant(id)?;
        (actor.position, actor.side)
    };
    let reactors = reactors_for_left_square(encounter, id, square);
    if reactors.is_empty() {
        return None;
    }
    let reaches: Vec<(GridPos, u32)> = reactors
        .iter()
        .filter_map(|reactor| {
            let combatant = encounter.combatant(*reactor)?;
            Some((combatant.position, combatant.strike.reach_feet))
        })
        .collect();
    let opponents: Vec<GridPos> = encounter
        .combatants()
        .filter(|combatant| combatant.side != side && combatant.is_conscious())
        .map(|combatant| combatant.position)
        .collect();
    legal
        .iter()
        .filter_map(|action| match *action {
            Action::Stride { destination } => Some(destination),
            Action::Step { .. }
            | Action::Crawl { .. }
            | Action::Leap { .. }
            | Action::Strike { .. }
            | Action::DropProne
            | Action::Stand
            | Action::RaiseShield
            | Action::Seek
            | Action::PointOut { .. }
            | Action::TakeCover
            | Action::Release { .. }
            | Action::Interact
            | Action::Delay
            | Action::Ready { .. } => None,
        })
        .filter(|destination| {
            reaches
                .iter()
                .all(|(pos, reach)| rules::minimum_feet(*destination, *pos) > *reach)
        })
        .min_by_key(|destination| {
            let dist = opponents
                .iter()
                .map(|pos| rules::minimum_feet(*destination, *pos))
                .min()
                .unwrap_or(u32::MAX);
            (dist, destination.x, destination.y)
        })
        .map(|destination| Action::Stride { destination })
}

/// `Err` means a party member is waiting on U or H.
fn drain_delay(
    encounter: &mut Encounter,
    session: &mut Session,
    delay_answer: &mut DelayAnswer,
    hud: &mut HudBump,
) -> Result<Vec<Event>, ()> {
    let mut events = Vec::new();
    loop {
        let Some(id) = encounter.pending_delay_resume() else {
            if session.delay_prompt {
                session.delay_prompt = false;
                hud.0 += 1;
            }
            return Ok(events);
        };
        let party = encounter
            .combatant(id)
            .is_some_and(|combatant| combatant.side == Side::Party);
        let answer = if party {
            delay_answer.0.take()
        } else {
            Some(true)
        };
        let Some(resume) = answer else {
            session.aim = Aim::None;
            session.ready = None;
            if !session.delay_prompt {
                session.delay_prompt = true;
                hud.0 += 1;
            }
            return Err(());
        };
        match resolve_delay_resume(encounter, resume, &mut session.rng) {
            Ok(resolved) => events.extend(resolved),
            Err(error) => {
                session.modal = Some(error.to_string());
                session.delay_prompt = false;
                return Err(());
            }
        }
    }
}

fn log_events(session: &mut Session, encounter: &Encounter, events: &[Event], hud: &mut HudBump) {
    for event in events {
        session.log.push(describe(encounter, event));
    }
    if session.log.len() > 12 {
        let drain = session.log.len() - 12;
        session.log.drain(0..drain);
    }
    if !events.is_empty() || session.modal.is_some() {
        hud.0 += 1;
    }
}

fn describe(encounter: &Encounter, event: &Event) -> String {
    let name = |id: CombatantId| {
        encounter
            .combatant(id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| format!("#{id}"))
    };
    match event {
        Event::Moved {
            id,
            to,
            feet,
            actions_remaining,
            ..
        } => format!(
            "{} strides to ({}, {}), {feet} ft. {actions_remaining} left.",
            name(*id),
            to.x,
            to.y
        ),
        Event::Stepped {
            id,
            to,
            feet,
            actions_remaining,
            ..
        } => format!(
            "{} steps to ({}, {}), {feet} ft. {actions_remaining} left.",
            name(*id),
            to.x,
            to.y
        ),
        Event::DroppedProne {
            id,
            actions_remaining,
        } => format!("{} drops prone. {actions_remaining} left.", name(*id)),
        Event::Stood {
            id,
            actions_remaining,
        } => format!("{} stands. {actions_remaining} left.", name(*id)),
        Event::Crawled {
            id,
            to,
            feet,
            actions_remaining,
            ..
        } => format!(
            "{} crawls to ({}, {}), {feet} ft. {actions_remaining} left.",
            name(*id),
            to.x,
            to.y
        ),
        Event::Leapt {
            id,
            to,
            feet,
            actions_remaining,
            ..
        } => format!(
            "{} leaps to ({}, {}), {feet} ft. {actions_remaining} left.",
            name(*id),
            to.x,
            to.y
        ),
        Event::StrikeResolved {
            attacker,
            target,
            natural,
            total,
            armor_class,
            degree,
            damage,
            target_hit_points,
            ..
        } => {
            let word = match degree {
                rules::Degree::CriticalSuccess => "critical hit",
                rules::Degree::Success => "hit",
                rules::Degree::Failure => "miss",
                rules::Degree::CriticalFailure => "critical miss",
            };
            format!(
                "{} — {word} on {} for {damage} (d20 {natural}, {total} vs AC {armor_class}). {} HP left.",
                name(*attacker),
                name(*target),
                target_hit_points
            )
        }
        Event::ReactiveStrike {
            attacker,
            target,
            natural,
            total,
            armor_class,
            damage,
            target_hit_points,
            ..
        } => format!(
            "{} — reactive strike on {} for {damage} (d20 {natural}, {total} vs AC {armor_class}). {target_hit_points} HP left.",
            name(*attacker),
            name(*target),
        ),
        Event::CombatantDowned { id } => format!("{} is dying.", name(*id)),
        Event::CombatantDied { id } => format!("{} is dead.", name(*id)),
        Event::RecoveryCheck {
            id, degree, dying, ..
        } => {
            let word = match degree {
                rules::Degree::CriticalSuccess => "critical success",
                rules::Degree::Success => "success",
                rules::Degree::Failure => "failure",
                rules::Degree::CriticalFailure => "critical failure",
            };
            format!("{} recovery {word}. Dying {dying}.", name(*id))
        }
        Event::TurnStarted { id, round, actions } => {
            format!("Round {round}. {} — {actions} actions.", name(*id))
        }
        Event::ShieldRaised {
            id,
            actions_remaining,
        } => format!(
            "{} raises a shield. {actions_remaining} left.",
            name(*id)
        ),
        Event::ShieldBlock {
            defender,
            prevented,
            damage,
            shield_hit_points,
            ..
        } => format!(
            "{} — shield block stops {prevented}. {damage} gets through. Shield {shield_hit_points} HP.",
            name(*defender)
        ),
        Event::EncounterEnded { outcome } => match outcome {
            Outcome::Victory => "Victory.".into(),
            Outcome::Defeat => "Defeat.".into(),
        },
        Event::Sought {
            id,
            actions_remaining,
            ..
        } => format!("{} seeks. {actions_remaining} left.", name(*id)),
        Event::PointedOut {
            id,
            target,
            actions_remaining,
        } => format!(
            "{} points out {}. {actions_remaining} left.",
            name(*id),
            name(*target)
        ),
        Event::TookCover {
            id,
            actions_remaining,
        } => format!("{} takes cover. {actions_remaining} left.", name(*id)),
        Event::FlatCheckFailed {
            attacker,
            target,
            natural,
            dc,
            actions_remaining,
        } => format!(
            "{} misses {} (flat check {natural} vs DC {dc}). {actions_remaining} left.",
            name(*attacker),
            name(*target)
        ),
        Event::Released {
            id,
            item,
            actions_remaining,
        } => format!(
            "{} releases the {}. {actions_remaining} left.",
            name(*id),
            gear_name(*item)
        ),
        Event::PickedUp {
            id,
            item,
            actions_remaining,
        } => format!(
            "{} picks up the {}. {actions_remaining} left.",
            name(*id),
            gear_name(*item)
        ),
        Event::Delayed { id, .. } => format!("{} delays.", name(*id)),
        Event::Resumed { id } => format!("{} resumes.", name(*id)),
        Event::DelayLost { id } => format!("{} loses the delayed turn.", name(*id)),
        Event::Readied {
            id,
            actions_remaining,
        } => format!(
            "{} readies an action. {actions_remaining} left.",
            name(*id)
        ),
        Event::ReadiedExpired { id } => format!("{} loses the readied action.", name(*id)),
        Event::Aided { aider, attacker, .. } => {
            format!("{} aids {}.", name(*aider), name(*attacker))
        }
    }
}

fn gear_name(item: Gear) -> &'static str {
    match item {
        Gear::Weapon => "weapon",
        Gear::Shield => "shield",
    }
}

fn template_of(spawn: &Spawn, side: Side) -> CreatureTemplate {
    CreatureTemplate {
        name: spawn.name.clone(),
        level: spawn.creature.level,
        hit_points: spawn.creature.hit_points,
        armor_class: spawn.creature.armor_class,
        perception: spawn.creature.perception,
        speed_feet: spawn.creature.speed_feet,
        strike: StrikeTemplate {
            name: spawn.creature.strike.name.clone(),
            attack_bonus: spawn.creature.strike.attack_bonus,
            damage: DamageFormula {
                dice: spawn.creature.strike.damage.dice,
                faces: spawn.creature.strike.damage.faces,
                bonus: spawn.creature.strike.damage.bonus,
            },
            reach_feet: spawn.creature.strike.reach_feet,
        },
        side,
        features: spawn
            .creature
            .features
            .iter()
            .copied()
            .map(|feature| match feature {
                Feature::ReactiveStrike => ClassFeature::ReactiveStrike,
            })
            .collect(),
        shield: spawn.creature.shield.as_ref().map(|shield| Shield {
            hardness: shield.hardness,
            hit_points: shield.hit_points,
            max_hit_points: shield.max_hit_points,
            broken_threshold: shield.broken_threshold,
            raised: false,
        }),
        shield_block: spawn.creature.shield_block,
        stealth_dc: None,
    }
}

fn grid_of(fixture: &EncounterFixture) -> Grid {
    Grid::new(
        fixture.room.width,
        fixture.room.height,
        fixture
            .room
            .blocked
            .iter()
            .map(|cell| GridPos::new(cell.x, cell.y)),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProneScript {
    Await,
    Stand,
    Seek,
    Done,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GearScript {
    Ready,
    Pickup,
    Done,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DelayScript {
    Await,
    Done,
}

struct HeadlessScript {
    prone: ProneScript,
    gear: GearScript,
    delay: DelayScript,
}

/// The first party turn drops prone and stands, once. A later party turn Seeks once.
/// One party turn outside the Hall Guard's reach releases the shield and picks it up.
/// One later party turn Delays. Headless resumes at the next end of turn. It does not Ready.
fn headless_action(
    encounter: &Encounter,
    id: CombatantId,
    script: &mut HeadlessScript,
) -> Option<Action> {
    let party = encounter
        .combatant(id)
        .is_some_and(|combatant| combatant.side == Side::Party);
    if party {
        let legal = legal_actions(encounter, id);
        if script.gear == GearScript::Ready && outside_hall_guard_reach(encounter, id) {
            let action = Action::Release { item: Gear::Shield };
            if legal.contains(&action) {
                script.gear = GearScript::Pickup;
                return Some(action);
            }
        }
        if script.gear == GearScript::Pickup {
            script.gear = GearScript::Done;
            if legal.contains(&Action::Interact) {
                return Some(Action::Interact);
            }
        }
        if script.prone != ProneScript::Done {
            if script.prone == ProneScript::Await && legal.contains(&Action::DropProne) {
                script.prone = ProneScript::Stand;
                return Some(Action::DropProne);
            }
            if script.prone == ProneScript::Stand && legal.contains(&Action::Stand) {
                script.prone = ProneScript::Seek;
                return Some(Action::Stand);
            }
            if script.prone == ProneScript::Seek
                && encounter.actions_remaining() == 3
                && legal.contains(&Action::Seek)
            {
                script.prone = ProneScript::Done;
                return Some(Action::Seek);
            }
        }
        if script.prone == ProneScript::Done
            && script.gear == GearScript::Done
            && script.delay == DelayScript::Await
            && encounter.actions_remaining() == 3
            && legal.contains(&Action::Delay)
        {
            script.delay = DelayScript::Done;
            return Some(Action::Delay);
        }
    }
    closing_action(encounter, id)
}

fn outside_hall_guard_reach(encounter: &Encounter, id: CombatantId) -> bool {
    let Some(actor) = encounter.combatant(id) else {
        return false;
    };
    let Some(guard) = encounter
        .combatants()
        .find(|combatant| combatant.name == "Hall Guard" && !combatant.dead)
    else {
        return false;
    };
    rules::minimum_feet(actor.position, guard.position) > guard.strike.reach_feet
}

fn shield_is_down(encounter: &Encounter, id: CombatantId) -> bool {
    encounter
        .combatant(id)
        .and_then(|combatant| combatant.shield.as_ref())
        .is_none_or(|shield| !shield.raised)
}

/// Strike if one is legal. Otherwise Stride to the square that ends closest to an opponent.
fn closing_action(encounter: &Encounter, id: CombatantId) -> Option<Action> {
    let legal = legal_actions(encounter, id);
    if shield_is_down(encounter, id) {
        if let Some(action) = legal
            .iter()
            .copied()
            .find(|action| matches!(action, Action::RaiseShield))
        {
            return Some(action);
        }
    }
    if let Some(action) = legal
        .iter()
        .copied()
        .find(|action| matches!(action, Action::Strike { .. }))
    {
        return Some(action);
    }
    let actor = encounter.combatant(id)?;
    let foes: Vec<GridPos> = encounter
        .combatants()
        .filter(|combatant| combatant.side != actor.side && combatant.is_conscious())
        .map(|combatant| combatant.position)
        .collect();
    legal
        .into_iter()
        .filter_map(|action| match action {
            Action::Stride { destination } => Some(destination),
            Action::Step { .. }
            | Action::Crawl { .. }
            | Action::Leap { .. }
            | Action::Strike { .. }
            | Action::DropProne
            | Action::Stand
            | Action::RaiseShield
            | Action::Seek
            | Action::PointOut { .. }
            | Action::TakeCover
            | Action::Release { .. }
            | Action::Interact
            | Action::Delay
            | Action::Ready { .. } => None,
        })
        .min_by_key(|destination| {
            let dist = foes
                .iter()
                .map(|pos| rules::minimum_feet(*destination, *pos))
                .min()
                .unwrap_or(u32::MAX);
            (dist, destination.x, destination.y)
        })
        .map(|destination| Action::Stride { destination })
}

pub fn headless_report() -> Result<String, String> {
    let path = EncounterPath::fixture().0;
    let fixture = load_encounter(&path).map_err(|error| error.to_string())?;
    let mut lines = Vec::new();
    lines.push(format!("loaded {} from {}", fixture.id, path.display()));
    lines.push(format!(
        "room {}x{} ({} ft squares), party {}, enemy {}",
        fixture.room.width,
        fixture.room.height,
        fixture.room.cell_feet,
        fixture.party.len(),
        fixture.enemies[0].name
    ));
    let outside = fixture.party.iter().all(|spawn| {
        !cell_in_trigger(
            spawn.x,
            spawn.y,
            fixture.trigger.x,
            fixture.trigger.y,
            fixture.trigger.radius_feet,
        )
    });
    lines.push(format!(
        "trigger ({}, {}) radius {} ft; party spawns outside it: {outside}",
        fixture.trigger.x, fixture.trigger.y, fixture.trigger.radius_feet
    ));

    let party_cells = [
        (fixture.trigger.x, fixture.trigger.y),
        (fixture.trigger.x, fixture.trigger.y),
        (fixture.trigger.x, fixture.trigger.y),
        (fixture.trigger.x, fixture.trigger.y),
    ];
    let enemy = (fixture.enemies[0].x, fixture.enemies[0].y);
    let (party_pos, enemy_pos) =
        snap_to_grid(&fixture.room, party_cells, enemy).map_err(|error| error)?;
    lines.push(format!("snap party {:?}, enemy {:?}", party_pos, enemy_pos));

    let party = [
        template_of(&fixture.party[0], Side::Party),
        template_of(&fixture.party[1], Side::Party),
        template_of(&fixture.party[2], Side::Party),
        template_of(&fixture.party[3], Side::Party),
    ];
    let enemies = vec![template_of(&fixture.enemies[0], Side::Enemy)];
    let mut rng = rules::SeededDice::new(fixture.rng_seed);
    let mut encounter = start_encounter(
        &party,
        &enemies,
        &[
            GridPos::new(party_pos[0].0, party_pos[0].1),
            GridPos::new(party_pos[1].0, party_pos[1].1),
            GridPos::new(party_pos[2].0, party_pos[2].1),
            GridPos::new(party_pos[3].0, party_pos[3].1),
        ],
        &[GridPos::new(enemy_pos.0, enemy_pos.1)],
        grid_of(&fixture),
        &mut rng,
    )
    .map_err(|error| error.to_string())?;

    let order = encounter
        .initiative()
        .iter()
        .map(|id| {
            let combatant = encounter.combatant(*id).unwrap();
            format!("{} {}", combatant.name, combatant.initiative_total)
        })
        .collect::<Vec<_>>()
        .join(", ");
    lines.push(format!("initiative: {order}"));

    let mut prone_script = HeadlessScript {
        prone: ProneScript::Await,
        gear: GearScript::Ready,
        delay: DelayScript::Await,
    };
    let mut aided = false;
    for _ in 0..800 {
        if encounter.outcome().is_some() {
            break;
        }
        while encounter.pending_shield_block().is_some() {
            let events = resolve_pending_shield_block(&mut encounter, true, &mut rng)
                .map_err(|error| error.to_string())?;
            for event in &events {
                lines.push(describe(&encounter, event));
            }
        }
        if encounter.outcome().is_some() {
            break;
        }
        let Some(active) = encounter.active() else {
            break;
        };
        let action = headless_action(&encounter, active, &mut prone_script);
        let events = if let Some(action) = action {
            declare_action(&mut encounter, active, action, &mut rng)
                .map_err(|error| error.to_string())?
        } else {
            end_turn(&mut encounter, &mut rng).map_err(|error| error.to_string())?
        };
        for event in &events {
            lines.push(describe(&encounter, event));
        }
        while encounter.pending_aid().is_some() {
            let accept = !aided
                && encounter.pending_aid().is_some_and(|offer| {
                    encounter
                        .combatant(offer.attacker)
                        .is_some_and(|combatant| combatant.side == Side::Party)
                });
            if accept {
                aided = true;
            }
            let resolved = resolve_pending_aid(&mut encounter, accept, &mut rng)
                .map_err(|error| error.to_string())?;
            for event in &resolved {
                lines.push(describe(&encounter, event));
            }
        }
        let more = advance_if_turn_done(&mut encounter, &mut rng);
        for event in &more {
            lines.push(describe(&encounter, event));
        }
        while encounter.pending_delay_resume().is_some() {
            let resumed = resolve_delay_resume(&mut encounter, true, &mut rng)
                .map_err(|error| error.to_string())?;
            for event in &resumed {
                lines.push(describe(&encounter, event));
            }
        }
    }
    lines.push("headless ok".into());
    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::cell_to_world;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    #[test]
    fn trigger_snaps_to_combat_and_a_player_action_resolves() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
            .insert_resource(EncounterPath::fixture())
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                500,
            )))
            .add_plugins(SessionPlugin);

        for _ in 0..3 {
            app.update();
        }
        assert_eq!(
            *app.world().resource::<State<GameMode>>().get(),
            GameMode::Explore
        );

        {
            let mut session = app.world_mut().resource_mut::<Session>();
            let trigger = session.fixture.trigger.clone();
            let (x, y) = cell_to_world(trigger.x, trigger.y);
            session.leader_world = Vec2::new(x, y);
        }

        let mut reached_combat = false;
        for _ in 0..8 {
            app.update();
            if *app.world().resource::<State<GameMode>>().get() == GameMode::Combat {
                reached_combat = true;
                break;
            }
        }
        assert!(reached_combat, "trigger should enter combat");

        let encounter = app
            .world()
            .resource::<Session>()
            .encounter
            .as_ref()
            .unwrap();
        assert_eq!(encounter.party().len(), 4);
        assert_eq!(encounter.actions_remaining(), 3);
        assert!(encounter.grid().width() > 0);

        let mut acted = false;
        for _ in 0..12 {
            let action = {
                let session = app.world().resource::<Session>();
                let encounter = session.encounter.as_ref().unwrap();
                if encounter.outcome().is_some() {
                    break;
                }
                let Some(active) = encounter.active() else {
                    break;
                };
                let side = encounter.combatant(active).unwrap().side;
                if side != Side::Party {
                    None
                } else {
                    let legal = legal_actions(encounter, active);
                    legal
                        .iter()
                        .copied()
                        .find(|action| matches!(action, Action::Strike { .. }))
                        .or_else(|| legal.first().copied())
                }
            };
            if let Some(action) = action {
                app.world_mut().resource_mut::<PendingOrder>().0 = Some(action);
                acted = true;
            }
            app.update();
            for _ in 0..4 {
                if !app.world().resource::<Session>().aid_prompt {
                    break;
                }
                app.world_mut().resource_mut::<AidAnswer>().0 = Some(false);
                app.update();
            }
            if acted {
                let log = app.world().resource::<Session>().log.join("\n");
                assert!(
                    log.contains("strides") || log.contains("hit") || log.contains("miss"),
                    "{log}"
                );
                break;
            }
        }
        assert!(acted, "a party member should resolve Stride or Strike");
    }

    #[test]
    fn guard_in_a_fighters_reach_strides_outside_that_reach() {
        let fighter = |name: &str, features: Vec<ClassFeature>| CreatureTemplate {
            name: name.into(),
            level: 1,
            hit_points: 20,
            armor_class: 18,
            perception: 5,
            speed_feet: 25,
            strike: StrikeTemplate {
                name: "longsword".into(),
                attack_bonus: 7,
                damage: DamageFormula {
                    dice: 1,
                    faces: 8,
                    bonus: 4,
                },
                reach_feet: 5,
            },
            side: Side::Party,
            features,
            shield: None,
            shield_block: false,
            stealth_dc: None,
        };
        let party = [
            fighter("Sera Voss", vec![ClassFeature::ReactiveStrike]),
            fighter("Bram Holt", vec![ClassFeature::ReactiveStrike]),
            fighter("Quill Ames", vec![]),
            fighter("Doss Kade", vec![]),
        ];
        let guard_template = CreatureTemplate {
            name: "Hall Guard".into(),
            level: 1,
            hit_points: 18,
            armor_class: 14,
            perception: 3,
            speed_feet: 25,
            strike: StrikeTemplate {
                name: "spear".into(),
                attack_bonus: 6,
                damage: DamageFormula {
                    dice: 1,
                    faces: 6,
                    bonus: 2,
                },
                reach_feet: 5,
            },
            side: Side::Enemy,
            features: Vec::new(),
            shield: None,
            shield_block: false,
            stealth_dc: None,
        };
        let mut dice = rules::ScriptedDice::new([1, 1, 1, 1, 1]);
        let mut encounter = start_encounter(
            &party,
            &[guard_template],
            &[
                GridPos::new(3, 3),
                GridPos::new(4, 3),
                GridPos::new(0, 0),
                GridPos::new(0, 7),
            ],
            &[GridPos::new(3, 2)],
            Grid::new(8, 8, []),
            &mut dice,
        )
        .unwrap();
        let guard = CombatantId(4);
        while encounter.active() != Some(guard) {
            end_turn(&mut encounter, &mut dice).unwrap();
        }

        let square = encounter.combatant(guard).unwrap().position;
        let reactors = reactors_for_left_square(&encounter, guard, square);
        assert_eq!(reactors, vec![CombatantId(0), CombatantId(1)]);

        let action = enemy_action(&encounter, guard, false).expect("a legal action");
        let Action::Stride { destination } = action else {
            panic!("expected Stride, got {action:?}");
        };
        assert!(
            reactors.iter().all(|reactor| {
                let combatant = encounter.combatant(*reactor).unwrap();
                rules::minimum_feet(combatant.position, destination) > combatant.strike.reach_feet
            }),
            "destination {destination:?} is still inside a reactor's reach"
        );

        let side = encounter.combatant(guard).unwrap().side;
        let opponents: Vec<GridPos> = encounter
            .combatants()
            .filter(|combatant| combatant.side != side && combatant.is_conscious())
            .map(|combatant| combatant.position)
            .collect();
        let nearest = |pos: GridPos| {
            opponents
                .iter()
                .map(|opponent| rules::minimum_feet(pos, *opponent))
                .min()
                .unwrap_or(u32::MAX)
        };
        let chosen = nearest(destination);
        for other in legal_actions(&encounter, guard) {
            let Action::Stride {
                destination: candidate,
            } = other
            else {
                continue;
            };
            let outside = reactors.iter().all(|reactor| {
                let combatant = encounter.combatant(*reactor).unwrap();
                rules::minimum_feet(combatant.position, candidate) > combatant.strike.reach_feet
            });
            if !outside {
                continue;
            }
            let dist = nearest(candidate);
            assert!(
                (dist, candidate.x, candidate.y) >= (chosen, destination.x, destination.y),
                "closer square {candidate:?} was skipped for {destination:?}"
            );
        }

        assert!(matches!(
            enemy_action(&encounter, guard, true),
            Some(Action::Strike { .. })
        ));
        assert!(matches!(
            closing_action(&encounter, guard),
            Some(Action::Strike { .. })
        ));
    }

    #[test]
    fn guard_strikes_a_dying_fighter_before_leaving_reach() {
        let fighter = |name: &str, hit_points: i32| CreatureTemplate {
            name: name.into(),
            level: 1,
            hit_points,
            armor_class: 18,
            perception: 5,
            speed_feet: 25,
            strike: StrikeTemplate {
                name: "longsword".into(),
                attack_bonus: 7,
                damage: DamageFormula {
                    dice: 1,
                    faces: 8,
                    bonus: 4,
                },
                reach_feet: 5,
            },
            side: Side::Party,
            features: vec![ClassFeature::ReactiveStrike],
            shield: None,
            shield_block: false,
            stealth_dc: None,
        };
        let party = [
            fighter("Sera Voss", 20),
            fighter("Bram Holt", 8),
            fighter("Quill Ames", 20),
            fighter("Doss Kade", 20),
        ];
        let guard_template = CreatureTemplate {
            name: "Hall Guard".into(),
            level: 1,
            hit_points: 18,
            armor_class: 14,
            perception: 3,
            speed_feet: 25,
            strike: StrikeTemplate {
                name: "spear".into(),
                attack_bonus: 12,
                damage: DamageFormula {
                    dice: 1,
                    faces: 4,
                    bonus: 10,
                },
                reach_feet: 5,
            },
            side: Side::Enemy,
            features: Vec::new(),
            shield: None,
            shield_block: false,
            stealth_dc: None,
        };
        let mut dice = rules::ScriptedDice::new([1, 1, 1, 1, 20]);
        let mut encounter = start_encounter(
            &party,
            &[guard_template],
            &[
                GridPos::new(3, 3),
                GridPos::new(4, 3),
                GridPos::new(0, 0),
                GridPos::new(0, 7),
            ],
            &[GridPos::new(3, 2)],
            Grid::new(8, 8, []),
            &mut dice,
        )
        .unwrap();
        let guard = CombatantId(4);
        while encounter.active() != Some(guard) {
            end_turn(&mut encounter, &mut dice).unwrap();
        }
        assert!(!reactors_for_left_square(&encounter, guard, GridPos::new(3, 2)).is_empty());

        // 12 + 12 = 24 vs AC 18 is a success. Damage 1 + 10 = 11 drops 8 HP and is short of massive damage.
        let mut dice = rules::ScriptedDice::new([12, 1]);
        declare_action(
            &mut encounter,
            guard,
            Action::Strike {
                target: CombatantId(1),
            },
            &mut dice,
        )
        .unwrap();
        assert!(encounter
            .combatant(CombatantId(1))
            .unwrap()
            .is_unconscious());
        assert!(!encounter.combatant(CombatantId(1)).unwrap().dead);
        assert_eq!(
            enemy_action(&encounter, guard, false),
            Some(Action::Strike {
                target: CombatantId(1)
            })
        );
    }
}
