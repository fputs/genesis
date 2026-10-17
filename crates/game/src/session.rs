//! Explore / combat session. Rules stay in `rules`; this module owns the mode machine.

use bevy::prelude::*;
use content::{load_encounter, EncounterFixture, Spawn};
use rules::{
    advance_if_turn_done, declare_action, end_turn, legal_actions, start_encounter, Action,
    CombatantId, CreatureTemplate, DamageFormula, Encounter, Event, Grid, GridPos, Outcome, Side,
    StrikeTemplate,
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
    Strike,
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

pub struct SessionPlugin;

impl Plugin for SessionPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameMode>()
            .init_resource::<ExploreDirection>()
            .init_resource::<PendingOrder>()
            .init_resource::<EndTurnRequest>()
            .init_resource::<EnemyClock>()
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
    mut clock: ResMut<EnemyClock>,
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

    let mut events = advance_if_turn_done(&mut encounter);
    let mut waiting = false;
    if events.is_empty() {
        if end_turn_request.0 {
            end_turn_request.0 = false;
            session.aim = Aim::None;
            match end_turn(&mut encounter) {
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
                            events.extend(resolved);
                        }
                        Err(_) => session.modal = Some("That action is not legal.".into()),
                    }
                }
            } else if side == Some(Side::Enemy) {
                clock.0.tick(time.delta());
                if clock.0.finished() {
                    clock.0.reset();
                    if let Some(action) = enemy_action(&encounter, active) {
                        if let Ok(resolved) =
                            declare_action(&mut encounter, active, action, &mut session.rng)
                        {
                            events.extend(resolved);
                        }
                    } else if let Ok(resolved) = end_turn(&mut encounter) {
                        events.extend(resolved);
                    }
                } else {
                    waiting = true;
                }
            }
        }
    }
    if !waiting && encounter.outcome().is_none() {
        events.extend(advance_if_turn_done(&mut encounter));
    }

    let outcome = encounter.outcome();
    for event in &events {
        session.log.push(describe(&encounter, event));
    }
    if session.log.len() > 6 {
        let drain = session.log.len() - 6;
        session.log.drain(0..drain);
    }
    if !events.is_empty() || session.modal.is_some() {
        hud.0 += 1;
    }
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

fn enemy_action(encounter: &Encounter, id: CombatantId) -> Option<Action> {
    let legal = legal_actions(encounter, id);
    let mut strikes: Vec<CombatantId> = legal
        .iter()
        .filter_map(|action| match action {
            Action::Strike { target } => Some(*target),
            Action::Stride { .. } => None,
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
            Action::Strike { .. } => None,
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
        Event::CombatantDowned { id } => format!("{} is down.", name(*id)),
        Event::TurnStarted { id, round, actions } => {
            format!("Round {round}. {} — {actions} actions.", name(*id))
        }
        Event::EncounterEnded { outcome } => match outcome {
            Outcome::Victory => "Victory.".into(),
            Outcome::Defeat => "Defeat.".into(),
        },
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

/// Strike if one is legal. Otherwise Stride to the square that ends closest to an opponent.
fn closing_action(encounter: &Encounter, id: CombatantId) -> Option<Action> {
    let legal = legal_actions(encounter, id);
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
            Action::Strike { .. } => None,
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

    for _ in 0..8 {
        if encounter.outcome().is_some() {
            break;
        }
        let Some(active) = encounter.active() else {
            break;
        };
        let action = closing_action(&encounter, active);
        let events = if let Some(action) = action {
            declare_action(&mut encounter, active, action, &mut rng)
                .map_err(|error| error.to_string())?
        } else {
            end_turn(&mut encounter).map_err(|error| error.to_string())?
        };
        for event in &events {
            lines.push(describe(&encounter, event));
        }
        if events
            .iter()
            .any(|event| matches!(event, Event::StrikeResolved { .. }))
        {
            break;
        }
        let more = advance_if_turn_done(&mut encounter);
        for event in &more {
            lines.push(describe(&encounter, event));
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
}
