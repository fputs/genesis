//! Room sprites, party tokens, and the portrait / action bar.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use content::Floor;
use rules::{legal_actions, Action, Detection, Side};

use crate::session::{
    Aim, EndTurnRequest, ExploreDirection, GameMode, HudBump, PendingOrder, Session, ShieldAnswer,
};
use crate::world::{cell_to_world, world_to_cell, TILE_H, TILE_W};

const PARTY_COLOR: [[f32; 3]; 4] = [
    [0.31, 0.47, 0.70],
    [0.28, 0.50, 0.34],
    [0.58, 0.26, 0.30],
    [0.66, 0.50, 0.24],
];
const ENEMY_COLOR: [f32; 3] = [0.55, 0.18, 0.16];
const INK: Color = Color::srgb(0.93, 0.88, 0.74);
const PANEL: Color = Color::srgb(0.14, 0.10, 0.07);
const GOLD: Color = Color::srgb(0.62, 0.48, 0.22);

#[derive(Component)]
struct FloorTile;

#[derive(Component)]
enum Token {
    Party(usize),
    Enemy,
}

#[derive(Component)]
struct Highlight;

#[derive(Component)]
struct HudRoot;

#[derive(Component, Clone, Copy)]
enum HudButton {
    Portrait(usize),
    Stride,
    Step,
    Strike,
    RaiseShield,
    DropProne,
    Stand,
    Crawl,
    Leap,
    TakeCover,
    Seek,
    PointOut,
    EndTurn,
    ShieldYes,
    ShieldNo,
    Dismiss,
}

pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_world.after(crate::session::load_session))
            .add_systems(Update, read_keys.before(crate::session::explore_move))
            .add_systems(
                Update,
                (
                    aim_click,
                    hud_clicks,
                    sync_tokens,
                    sync_highlights,
                    sync_hud,
                )
                    .chain(),
            );
    }
}

fn spawn_world(mut commands: Commands, session: Res<Session>, mut images: ResMut<Assets<Image>>) {
    let (cx, cy) = cell_to_world(5, 3);
    commands.spawn((
        Camera2d,
        IsDefaultUiCamera,
        Transform::from_xyz(cx, cy, 1_000.0),
        OrthographicProjection {
            scale: 0.5,
            ..OrthographicProjection::default_2d()
        },
    ));

    let stone = images.add(diamond(
        TILE_W as u32,
        TILE_H as u32,
        [92, 84, 70, 255],
        [58, 52, 42, 255],
    ));
    let grass = images.add(diamond(
        TILE_W as u32,
        TILE_H as u32,
        [72, 96, 58, 255],
        [42, 60, 34, 255],
    ));
    let dirt = images.add(diamond(
        TILE_W as u32,
        TILE_H as u32,
        [112, 80, 48, 255],
        [70, 48, 28, 255],
    ));
    let wall = images.add(diamond(
        TILE_W as u32,
        TILE_H as u32,
        [46, 40, 36, 255],
        [24, 20, 18, 255],
    ));
    let room = &session.fixture.room;
    for y in 0..room.height {
        for x in 0..room.width {
            let (wx, wy) = cell_to_world(x, y);
            let blocked = !room.walkable(x, y);
            let image = if blocked {
                wall.clone()
            } else {
                match room.floor(x, y).unwrap_or(Floor::Stone) {
                    Floor::Stone => stone.clone(),
                    Floor::Grass => grass.clone(),
                    Floor::Dirt => dirt.clone(),
                }
            };
            commands.spawn((
                Sprite {
                    image,
                    custom_size: Some(Vec2::new(TILE_W, TILE_H)),
                    ..default()
                },
                Transform::from_xyz(wx, wy, (x + y) as f32 * 0.01),
                FloorTile,
            ));
        }
    }

    for slot in 0..4 {
        let image = images.add(pawn(PARTY_COLOR[slot]));
        commands.spawn((
            Sprite {
                image,
                custom_size: Some(Vec2::new(22.0, 32.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 2.0),
            Token::Party(slot),
        ));
    }
    let image = images.add(pawn(ENEMY_COLOR));
    commands.spawn((
        Sprite {
            image,
            custom_size: Some(Vec2::new(24.0, 34.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 2.0),
        Token::Enemy,
    ));
}

fn sync_tokens(
    mode: Res<State<GameMode>>,
    session: Res<Session>,
    mut tokens: Query<(&Token, &mut Transform, &mut Visibility)>,
) {
    let mode = *mode.get();
    for (token, mut transform, mut visibility) in &mut tokens {
        match token {
            Token::Party(slot) => {
                let (world, cell, hidden) = party_pose(&session, mode, *slot);
                transform.translation = Vec3::new(
                    world.x,
                    world.y + 10.0,
                    1.0 + (cell.0 + cell.1) as f32 * 0.01,
                );
                *visibility = if hidden {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
            }
            Token::Enemy => {
                let (cell, hidden) = enemy_pose(&session);
                let (x, y) = cell_to_world(cell.0, cell.1);
                transform.translation =
                    Vec3::new(x, y + 10.0, 1.0 + (cell.0 + cell.1) as f32 * 0.01);
                *visibility = if hidden {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
            }
        }
    }
}

fn party_pose(session: &Session, mode: GameMode, slot: usize) -> (Vec2, (i32, i32), bool) {
    if matches!(mode, GameMode::Combat | GameMode::TransitioningToCombat) {
        if let Some(encounter) = &session.encounter {
            let id = encounter.party()[slot];
            if let Some(combatant) = encounter.combatant(id) {
                let cell = (combatant.position.x, combatant.position.y);
                let (x, y) = cell_to_world(cell.0, cell.1);
                return (Vec2::new(x, y), cell, false);
            }
        }
    }
    let world = session.member_world(slot);
    let cell = world_to_cell(world.x, world.y);
    (world, cell, false)
}

fn enemy_pose(session: &Session) -> ((i32, i32), bool) {
    if let Some(encounter) = &session.encounter {
        if let Some(enemy) = encounter.combatants().find(|c| c.side == Side::Enemy) {
            let undetected = encounter.active().is_some_and(|active| {
                encounter.detection(active, enemy.id) == Some(Detection::Undetected)
            });
            return (
                (enemy.position.x, enemy.position.y),
                enemy.dead || undetected,
            );
        }
    }
    (
        (session.fixture.enemies[0].x, session.fixture.enemies[0].y),
        false,
    )
}

fn sync_highlights(
    mut commands: Commands,
    mode: Res<State<GameMode>>,
    session: Res<Session>,
    mut images: ResMut<Assets<Image>>,
    existing: Query<Entity, With<Highlight>>,
    mut cache: Local<Option<Handle<Image>>>,
) {
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    if *mode.get() != GameMode::Combat || session.aim == Aim::None {
        return;
    }
    let Some(encounter) = &session.encounter else {
        return;
    };
    let Some(active) = encounter.active() else {
        return;
    };
    if encounter.combatant(active).map(|c| c.side) != Some(Side::Party) {
        return;
    }
    let image = cache
        .get_or_insert_with(|| {
            images.add(diamond(
                TILE_W as u32,
                TILE_H as u32,
                [212, 176, 92, 90],
                [212, 176, 92, 140],
            ))
        })
        .clone();
    for action in legal_actions(encounter, active) {
        let show = match (session.aim, action) {
            (Aim::Stride, Action::Stride { destination }) => Some((destination.x, destination.y)),
            (Aim::Step, Action::Step { destination }) => Some((destination.x, destination.y)),
            (Aim::Crawl, Action::Crawl { destination }) => Some((destination.x, destination.y)),
            (Aim::Leap, Action::Leap { destination }) => Some((destination.x, destination.y)),
            (Aim::Strike, Action::Strike { target }) => encounter
                .combatant(target)
                .map(|c| (c.position.x, c.position.y)),
            (Aim::PointOut, Action::PointOut { target }) => encounter
                .combatant(target)
                .map(|c| (c.position.x, c.position.y)),
            _ => None,
        };
        if let Some((x, y)) = show {
            let (wx, wy) = cell_to_world(x, y);
            commands.spawn((
                Sprite {
                    image: image.clone(),
                    custom_size: Some(Vec2::new(TILE_W, TILE_H)),
                    ..default()
                },
                Transform::from_xyz(wx, wy, 0.5 + (x + y) as f32 * 0.01),
                Highlight,
            ));
        }
    }
}

fn read_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mode: Res<State<GameMode>>,
    mut session: ResMut<Session>,
    mut direction: ResMut<ExploreDirection>,
    mut aim_bump: ResMut<HudBump>,
    mut end_turn: ResMut<EndTurnRequest>,
    mut pending: ResMut<PendingOrder>,
    mut shield_answer: ResMut<ShieldAnswer>,
) {
    let mode = *mode.get();
    if mode != GameMode::Explore {
        direction.0 = Vec2::ZERO;
    } else if session.modal.is_none() {
        let mut gx = 0.0;
        let mut gy = 0.0;
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
            gx += 1.0;
        }
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
            gx -= 1.0;
        }
        if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
            gy += 1.0;
        }
        if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
            gy -= 1.0;
        }
        direction.0 = Vec2::new((gx - gy) * 32.0, -(gx + gy) * 16.0);
        if keys.just_pressed(KeyCode::Digit1) {
            select_leader(&mut session, 0);
        }
        if keys.just_pressed(KeyCode::Digit2) {
            select_leader(&mut session, 1);
        }
        if keys.just_pressed(KeyCode::Digit3) {
            select_leader(&mut session, 2);
        }
        if keys.just_pressed(KeyCode::Digit4) {
            select_leader(&mut session, 3);
        }
    } else {
        direction.0 = Vec2::ZERO;
    }

    if mode == GameMode::Combat && session.shield_prompt {
        if keys.just_pressed(KeyCode::KeyB) {
            shield_answer.0 = Some(true);
        }
        if keys.just_pressed(KeyCode::KeyN) {
            shield_answer.0 = Some(false);
        }
    } else if mode == GameMode::Combat && session.modal.is_none() {
        if keys.just_pressed(KeyCode::KeyM) {
            session.aim = Aim::Stride;
            aim_bump.0 += 1;
        }
        if keys.just_pressed(KeyCode::KeyT) {
            session.aim = Aim::Step;
            aim_bump.0 += 1;
        }
        if keys.just_pressed(KeyCode::KeyF) {
            session.aim = Aim::Strike;
            aim_bump.0 += 1;
        }
        if keys.just_pressed(KeyCode::KeyR) {
            queue_raise(&mut session, &mut pending, &mut aim_bump);
        }
        if keys.just_pressed(KeyCode::KeyP) {
            queue_immediate(&mut session, &mut pending, &mut aim_bump, Action::DropProne);
        }
        if keys.just_pressed(KeyCode::KeyG) {
            queue_immediate(&mut session, &mut pending, &mut aim_bump, Action::Stand);
        }
        if keys.just_pressed(KeyCode::KeyC) {
            aim_move(&mut session, &mut aim_bump, Aim::Crawl);
        }
        if keys.just_pressed(KeyCode::KeyL) {
            aim_move(&mut session, &mut aim_bump, Aim::Leap);
        }
        if keys.just_pressed(KeyCode::KeyV) {
            queue_immediate(&mut session, &mut pending, &mut aim_bump, Action::TakeCover);
        }
        if keys.just_pressed(KeyCode::KeyK) {
            queue_immediate(&mut session, &mut pending, &mut aim_bump, Action::Seek);
        }
        if keys.just_pressed(KeyCode::KeyO) {
            aim_point_out(&mut session, &mut aim_bump);
        }
        if keys.just_pressed(KeyCode::Space) {
            end_turn.0 = true;
        }
    }
    if keys.just_pressed(KeyCode::Escape) {
        if session.modal.is_some() {
            session.modal = None;
        } else {
            session.aim = Aim::None;
        }
        aim_bump.0 += 1;
    }
}

fn party_legal(session: &Session) -> Vec<Action> {
    let Some(encounter) = &session.encounter else {
        return Vec::new();
    };
    let Some(active) = encounter.active() else {
        return Vec::new();
    };
    if encounter.combatant(active).map(|combatant| combatant.side) != Some(Side::Party) {
        return Vec::new();
    }
    legal_actions(encounter, active)
}

fn queue_immediate(
    session: &mut Session,
    pending: &mut PendingOrder,
    hud: &mut HudBump,
    action: Action,
) {
    if party_legal(session).contains(&action) {
        pending.0 = Some(action);
        session.aim = Aim::None;
        hud.0 += 1;
    }
}

fn aim_move(session: &mut Session, hud: &mut HudBump, aim: Aim) {
    let legal = party_legal(session);
    let available = legal.iter().any(|action| match (aim, action) {
        (Aim::Crawl, Action::Crawl { .. }) => true,
        (Aim::Leap, Action::Leap { .. }) => true,
        _ => false,
    });
    if available {
        session.aim = aim;
        hud.0 += 1;
    }
}

fn aim_point_out(session: &mut Session, hud: &mut HudBump) {
    if party_legal(session)
        .iter()
        .any(|action| matches!(action, Action::PointOut { .. }))
    {
        session.aim = Aim::PointOut;
        hud.0 += 1;
    }
}

fn queue_raise(session: &mut Session, pending: &mut PendingOrder, hud: &mut HudBump) {
    let Some(encounter) = &session.encounter else {
        return;
    };
    let Some(active) = encounter.active() else {
        return;
    };
    let party = encounter
        .combatant(active)
        .is_some_and(|combatant| combatant.side == Side::Party);
    if !party {
        return;
    }
    if legal_actions(encounter, active).contains(&Action::RaiseShield) {
        pending.0 = Some(Action::RaiseShield);
        session.aim = Aim::None;
        hud.0 += 1;
    } else {
        session.modal = Some("That shield cannot be raised.".into());
        hud.0 += 1;
    }
}

fn select_leader(session: &mut Session, slot: usize) {
    let world = session.member_world(slot);
    session.leader = slot;
    session.leader_world = world;
}

fn aim_click(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mode: Res<State<GameMode>>,
    mut session: ResMut<Session>,
    mut pending: ResMut<PendingOrder>,
    mut hud: ResMut<HudBump>,
    hud_nodes: Query<&Interaction, With<HudButton>>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    if *mode.get() != GameMode::Combat || session.aim == Aim::None || session.modal.is_some() {
        return;
    }
    if hud_nodes
        .iter()
        .any(|interaction| *interaction != Interaction::None)
    {
        return;
    }
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, transform)) = camera.get_single() else {
        return;
    };
    let Ok(world) = camera.viewport_to_world_2d(transform, cursor) else {
        return;
    };
    let (x, y) = world_to_cell(world.x, world.y);
    let Some(encounter) = &session.encounter else {
        return;
    };
    let Some(active) = encounter.active() else {
        return;
    };
    let legal = legal_actions(encounter, active);
    let chosen = match session.aim {
        Aim::Stride => legal.into_iter().find(|action| {
            matches!(action, Action::Stride { destination } if destination.x == x && destination.y == y)
        }),
        Aim::Step => legal.into_iter().find(|action| {
            matches!(action, Action::Step { destination } if destination.x == x && destination.y == y)
        }),
        Aim::Crawl => legal.into_iter().find(|action| {
            matches!(action, Action::Crawl { destination } if destination.x == x && destination.y == y)
        }),
        Aim::Leap => legal.into_iter().find(|action| {
            matches!(action, Action::Leap { destination } if destination.x == x && destination.y == y)
        }),
        Aim::Strike => legal.into_iter().find(|action| match action {
            Action::Strike { target } => encounter
                .combatant(*target)
                .is_some_and(|combatant| combatant.position.x == x && combatant.position.y == y),
            _ => false,
        }),
        Aim::PointOut => legal.into_iter().find(|action| match action {
            Action::PointOut { target } => encounter
                .combatant(*target)
                .is_some_and(|combatant| combatant.position.x == x && combatant.position.y == y),
            _ => false,
        }),
        Aim::None => None,
    };
    if let Some(action) = chosen {
        pending.0 = Some(action);
    } else {
        session.modal = Some(match session.aim {
            Aim::Stride => "That square is out of the stride.".into(),
            Aim::Step => "That square is out of the step.".into(),
            Aim::Crawl => "That square is out of the crawl.".into(),
            Aim::Leap => "That square is out of the leap.".into(),
            Aim::Strike => "No enemy in reach on that square.".into(),
            Aim::PointOut => "You cannot point out a creature on that square.".into(),
            Aim::None => "Nothing to confirm.".into(),
        });
        hud.0 += 1;
    }
}

fn hud_clicks(
    mode: Res<State<GameMode>>,
    mut session: ResMut<Session>,
    mut pending: ResMut<PendingOrder>,
    mut end_turn: ResMut<EndTurnRequest>,
    mut shield_answer: ResMut<ShieldAnswer>,
    mut hud: ResMut<HudBump>,
    buttons: Query<(&Interaction, &HudButton), Changed<Interaction>>,
) {
    for (interaction, button) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if session.shield_prompt {
            match button {
                HudButton::ShieldYes => shield_answer.0 = Some(true),
                HudButton::ShieldNo => shield_answer.0 = Some(false),
                _ => {}
            }
            continue;
        }
        match button {
            HudButton::Dismiss => {
                session.modal = None;
                hud.0 += 1;
            }
            HudButton::Portrait(slot) => {
                if session.modal.is_none() && *mode.get() == GameMode::Explore {
                    select_leader(&mut session, *slot);
                    hud.0 += 1;
                }
            }
            HudButton::Stride => {
                session.aim = Aim::Stride;
                hud.0 += 1;
            }
            HudButton::Step => {
                session.aim = Aim::Step;
                hud.0 += 1;
            }
            HudButton::Strike => {
                session.aim = Aim::Strike;
                hud.0 += 1;
            }
            HudButton::RaiseShield => {
                queue_raise(&mut session, &mut pending, &mut hud);
            }
            HudButton::DropProne => {
                queue_immediate(&mut session, &mut pending, &mut hud, Action::DropProne);
            }
            HudButton::Stand => {
                queue_immediate(&mut session, &mut pending, &mut hud, Action::Stand);
            }
            HudButton::Crawl => {
                session.aim = Aim::Crawl;
                hud.0 += 1;
            }
            HudButton::Leap => {
                session.aim = Aim::Leap;
                hud.0 += 1;
            }
            HudButton::TakeCover => {
                queue_immediate(&mut session, &mut pending, &mut hud, Action::TakeCover);
            }
            HudButton::Seek => {
                queue_immediate(&mut session, &mut pending, &mut hud, Action::Seek);
            }
            HudButton::PointOut => {
                aim_point_out(&mut session, &mut hud);
            }
            HudButton::ShieldYes | HudButton::ShieldNo => {}
            HudButton::EndTurn => {
                end_turn.0 = true;
            }
        }
    }
}

fn sync_hud(
    mut commands: Commands,
    mode: Res<State<GameMode>>,
    session: Res<Session>,
    bump: Res<HudBump>,
    mut drawn: Local<u32>,
    mut ready: Local<bool>,
    existing: Query<Entity, With<HudRoot>>,
) {
    if *ready && *drawn == bump.0 {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn_recursive();
    }
    *ready = true;
    *drawn = bump.0;

    let mode = *mode.get();
    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn(Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                top: Val::Px(16.0),
                width: Val::Px(196.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|column| {
                for slot in 0..4 {
                    portrait(column, &session, mode, slot);
                }
            });
        });

    commands
        .spawn((
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(16.0),
                top: Val::Px(16.0),
                width: Val::Px(620.0),
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(1.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                ..default()
            },
            BackgroundColor(PANEL),
            BorderColor(GOLD),
        ))
        .with_children(|panel| {
            let heading = match mode {
                GameMode::Explore => "Explore",
                GameMode::TransitioningToCombat => "Drawing steel",
                GameMode::Combat => "Combat",
            };
            text(panel, heading, 20.0, INK);
            if mode == GameMode::Combat {
                if let Some(encounter) = &session.encounter {
                    let actions = encounter.actions_remaining();
                    let round = encounter.round();
                    let active = encounter.active();
                    let who = active
                        .and_then(|id| encounter.combatant(id))
                        .map(|c| c.name.as_str())
                        .unwrap_or("—");
                    let reaction = match active.and_then(|id| encounter.reaction_ready(id)) {
                        Some(true) => "  ·  Reaction ready",
                        Some(false) => "  ·  Reaction spent",
                        None => "",
                    };
                    text(
                        panel,
                        &format!("Round {round}  ·  {who}  ·  {actions} actions{reaction}"),
                        16.0,
                        INK,
                    );
                }
            } else {
                text(panel, "D walks east, toward the guard.", 16.0, INK);
            }
            for line in &session.log {
                text(panel, line, 14.0, Color::srgb(0.80, 0.74, 0.62));
            }
        });

    if mode == GameMode::Combat {
        commands
            .spawn((
                HudRoot,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(230.0),
                    bottom: Val::Px(18.0),
                    right: Val::Px(16.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    ..default()
                },
            ))
            .with_children(|bar| {
                let legal = party_legal(&session);
                let offers = |kind: fn(&Action) -> bool| legal.iter().any(kind);
                action_button(
                    bar,
                    "Stride (M)",
                    HudButton::Stride,
                    session.aim == Aim::Stride,
                );
                action_button(bar, "Step (T)", HudButton::Step, session.aim == Aim::Step);
                if offers(|action| matches!(action, Action::DropProne)) {
                    action_button(bar, "Drop Prone (P)", HudButton::DropProne, false);
                }
                if offers(|action| matches!(action, Action::Stand)) {
                    action_button(bar, "Stand (G)", HudButton::Stand, false);
                }
                if offers(|action| matches!(action, Action::Crawl { .. })) {
                    action_button(
                        bar,
                        "Crawl (C)",
                        HudButton::Crawl,
                        session.aim == Aim::Crawl,
                    );
                }
                if offers(|action| matches!(action, Action::Leap { .. })) {
                    action_button(bar, "Leap (L)", HudButton::Leap, session.aim == Aim::Leap);
                }
                if offers(|action| matches!(action, Action::TakeCover)) {
                    action_button(bar, "Take Cover (V)", HudButton::TakeCover, false);
                }
                if offers(|action| matches!(action, Action::Seek)) {
                    action_button(bar, "Seek (K)", HudButton::Seek, false);
                }
                if offers(|action| matches!(action, Action::PointOut { .. })) {
                    action_button(
                        bar,
                        "Point Out (O)",
                        HudButton::PointOut,
                        session.aim == Aim::PointOut,
                    );
                }
                action_button(
                    bar,
                    "Strike  (F)",
                    HudButton::Strike,
                    session.aim == Aim::Strike,
                );
                action_button(bar, "Raise Shield (R)", HudButton::RaiseShield, false);
                action_button(bar, "End Turn  (Space)", HudButton::EndTurn, false);
            });
    }

    if session.shield_prompt {
        if let Some(offer) = session
            .encounter
            .as_ref()
            .and_then(|encounter| encounter.pending_shield_block())
        {
            let name = session
                .encounter
                .as_ref()
                .and_then(|encounter| encounter.combatant(offer.defender))
                .map(|combatant| combatant.name.clone())
                .unwrap_or_else(|| "Someone".into());
            commands
                .spawn((
                    HudRoot,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(30.0),
                        top: Val::Percent(34.0),
                        width: Val::Px(480.0),
                        padding: UiRect::all(Val::Px(16.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.10, 0.08, 0.06)),
                    BorderColor(GOLD),
                ))
                .with_children(|modal| {
                    text(
                        modal,
                        &format!("{name} would take {} damage. Shield Block?", offer.damage),
                        20.0,
                        INK,
                    );
                    text(modal, "B blocks. N takes the hit.", 16.0, INK);
                    modal
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|row| {
                            action_button(row, "Block (B)", HudButton::ShieldYes, false);
                            action_button(row, "Take the hit (N)", HudButton::ShieldNo, false);
                        });
                });
        }
    }

    if let Some(message) = &session.modal {
        commands
            .spawn((
                HudRoot,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(30.0),
                    top: Val::Percent(38.0),
                    width: Val::Px(460.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(12.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.10, 0.08, 0.06)),
                BorderColor(GOLD),
            ))
            .with_children(|modal| {
                text(modal, message, 20.0, INK);
                action_button(modal, "Continue", HudButton::Dismiss, false);
            });
    }
}

fn portrait(parent: &mut ChildBuilder, session: &Session, mode: GameMode, slot: usize) {
    let spawn = &session.fixture.party[slot];
    let (hp, max_hp, down, active) = if let Some(encounter) = &session.encounter {
        let id = encounter.party()[slot];
        let combatant = encounter.combatant(id).unwrap();
        let active = encounter.active() == Some(id) && mode == GameMode::Combat;
        (
            combatant.hit_points,
            combatant.max_hit_points,
            !combatant.is_conscious(),
            active,
        )
    } else {
        (
            spawn.creature.hit_points,
            spawn.creature.hit_points,
            false,
            false,
        )
    };
    let border = if active {
        Color::srgb(0.93, 0.82, 0.45)
    } else if session.leader == slot && mode == GameMode::Explore {
        GOLD
    } else {
        Color::srgb(0.32, 0.26, 0.18)
    };
    parent
        .spawn((
            Button,
            HudButton::Portrait(slot),
            Node {
                width: Val::Px(196.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(PANEL),
            BorderColor(border),
        ))
        .with_children(|row| {
            let swatch = if down {
                Color::srgb(0.25, 0.24, 0.23)
            } else {
                Color::srgb(
                    PARTY_COLOR[slot][0],
                    PARTY_COLOR[slot][1],
                    PARTY_COLOR[slot][2],
                )
            };
            row.spawn((
                Node {
                    width: Val::Px(36.0),
                    height: Val::Px(48.0),
                    ..default()
                },
                BackgroundColor(swatch),
            ));
            row.spawn(Node {
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|text_col| {
                text(
                    text_col,
                    &spawn.name,
                    16.0,
                    if down {
                        Color::srgb(0.55, 0.52, 0.48)
                    } else {
                        INK
                    },
                );
                let line = if let Some(encounter) = &session.encounter {
                    let combatant = encounter.combatant(encounter.party()[slot]).unwrap();
                    if combatant.dead {
                        "Dead".to_string()
                    } else if combatant.dying > 0 {
                        format!("Dying {}", combatant.dying)
                    } else if down {
                        "Unconscious".to_string()
                    } else {
                        format!("{hp} / {max_hp} HP")
                    }
                } else {
                    format!("{hp} / {max_hp} HP")
                };
                text(text_col, &line, 14.0, Color::srgb(0.75, 0.68, 0.52));
            });
        });
}

fn action_button(parent: &mut ChildBuilder, label: &str, kind: HudButton, selected: bool) {
    parent
        .spawn((
            Button,
            kind,
            Node {
                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(if selected {
                Color::srgb(0.36, 0.26, 0.12)
            } else {
                Color::srgb(0.22, 0.16, 0.10)
            }),
            BorderColor(GOLD),
        ))
        .with_children(|button| {
            text(button, label, 16.0, INK);
        });
}

fn text(parent: &mut ChildBuilder, value: &str, size: f32, color: Color) {
    parent.spawn((
        Text::new(value),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
    ));
}

fn diamond(width: u32, height: u32, fill: [u8; 4], edge: [u8; 4]) -> Image {
    let mut data = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let dx = (x as f32 - (width as f32 - 1.0) * 0.5).abs() / (width as f32 * 0.5);
            let dy = (y as f32 - (height as f32 - 1.0) * 0.5).abs() / (height as f32 * 0.5);
            let score = dx + dy;
            if score <= 1.0 {
                let pixel = if score > 0.86 { edge } else { fill };
                let index = ((y * width + x) * 4) as usize;
                data[index..index + 4].copy_from_slice(&pixel);
            }
        }
    }
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

fn pawn(rgb: [f32; 3]) -> Image {
    let width = 22u32;
    let height = 32u32;
    let fill = [
        (rgb[0] * 255.0) as u8,
        (rgb[1] * 255.0) as u8,
        (rgb[2] * 255.0) as u8,
        255,
    ];
    let mut data = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let nx = (x as f32 + 0.5) / width as f32 - 0.5;
            let ny = (y as f32 + 0.5) / height as f32;
            let in_head = ny < 0.34 && nx * nx + (ny - 0.18) * (ny - 0.18) * 4.0 < 0.035;
            let in_body =
                (0.30..0.92).contains(&ny) && nx.abs() < 0.22 + (0.55 - ny).max(0.0) * 0.15;
            if in_head || in_body {
                let index = ((y * width + x) * 4) as usize;
                data[index..index + 4].copy_from_slice(&fill);
            }
        }
    }
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}
