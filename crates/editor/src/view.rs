//! Edit grid and isometric preview for one room file.

use std::path::PathBuf;

use bevy::asset::RenderAssetUsages;
use bevy::log::info;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::ui::FocusPolicy;
use bevy::window::WindowPosition;
use content::Floor;

use crate::iso::{diamond_offset, diamond_span, preview_tile_size};
use crate::session::EditorSession;

const FLOORS: [Floor; 3] = [Floor::Stone, Floor::Grass, Floor::Dirt];

#[derive(Resource)]
struct Editor {
    session: EditorSession,
}

#[derive(Resource, Default)]
struct PreviewImages {
    diamonds: Option<[Handle<Image>; 3]>,
}

#[derive(Component)]
struct GridHost;

#[derive(Component)]
struct PreviewHost;

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct MapNameText;

#[derive(Component)]
struct CellSizeText;

#[derive(Component, Clone, Copy)]
struct CellButton {
    x: i32,
    y: i32,
}

#[derive(Component, Clone, Copy)]
struct PreviewTile {
    x: i32,
    y: i32,
}

#[derive(Component, Clone, Copy)]
struct PaletteButton {
    floor: Floor,
}

#[derive(Component, Clone, Copy)]
enum CommandButton {
    Load,
    Save,
}

pub fn run(path: PathBuf) {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Genesis Editor".into(),
                resolution: (1600.0_f32, 1000.0_f32).into(),
                position: WindowPosition::At(IVec2::new(40, 40)),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(rgb(12, 11, 9)))
        .insert_resource(Editor {
            session: EditorSession::open(path),
        })
        .init_resource::<PreviewImages>()
        .add_systems(Startup, (spawn_camera, spawn_chrome, log_loaded).chain())
        .add_systems(
            Update,
            (
                palette_clicks,
                command_clicks,
                paint_clicks,
                sync_map,
                sync_labels,
                style_palette,
                style_commands,
                esc_quits,
            )
                .chain(),
        )
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn log_loaded(editor: Res<Editor>) {
    match &editor.session.room {
        Some(room) => info!(
            "loaded room {} ({}×{}, {} ft)",
            room.id, room.width, room.height, room.cell_feet
        ),
        None => info!("room load failed: {}", editor.session.detail),
    }
}

fn spawn_chrome(mut commands: Commands, editor: Res<Editor>) {
    let name = editor.session.map_name();
    let cell = editor.session.cell_size_label();
    let status = editor.session.status();
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(rgb(12, 11, 9)),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(72.0),
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(0.0)),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    border: UiRect::bottom(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(rgb(26, 23, 19)),
                BorderColor(rgb(58, 50, 40)),
            ))
            .with_children(|bar| {
                command_button(bar, "Load", CommandButton::Load);
                command_button(bar, "Save", CommandButton::Save);
                labeled_value(bar, "Map name", &name, MapNameText, 220.0);
                labeled_value(bar, "Cell size", &cell, CellSizeText, 88.0);
            });

            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                ..default()
            })
            .with_children(|body| {
                body.spawn((
                    Node {
                        width: Val::Px(248.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(16.0)),
                        row_gap: Val::Px(10.0),
                        border: UiRect::right(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(rgb(26, 23, 19)),
                    BorderColor(rgb(58, 50, 40)),
                ))
                .with_children(|palette| {
                    for floor in FLOORS {
                        palette_row(palette, floor);
                    }
                });

                body.spawn((
                    Node {
                        flex_grow: 1.0,
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(rgb(16, 14, 12)),
                    GridHost,
                ));

                body.spawn((
                    Node {
                        width: Val::Px(460.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::left(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(rgb(26, 23, 19)),
                    BorderColor(rgb(58, 50, 40)),
                    PreviewHost,
                ));
            });

            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(40.0),
                    padding: UiRect::horizontal(Val::Px(16.0)),
                    align_items: AlignItems::Center,
                    border: UiRect::top(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(rgb(26, 23, 19)),
                BorderColor(rgb(58, 50, 40)),
            ))
            .with_children(|bar| {
                bar.spawn((
                    Text::new(status),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(rgb(240, 230, 212)),
                    StatusText,
                ));
            });
        });
}

fn command_button(parent: &mut ChildBuilder, label: &str, command: CommandButton) {
    parent
        .spawn((
            Button,
            command,
            Node {
                height: Val::Px(36.0),
                padding: UiRect::horizontal(Val::Px(16.0)),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(rgb(91, 75, 55)),
            BorderColor(rgb(122, 102, 72)),
        ))
        .with_children(|button| {
            pass_text(button, label, 16.0);
        });
}

fn labeled_value(
    parent: &mut ChildBuilder,
    label: &str,
    value: &str,
    marker: impl Component,
    width: f32,
) {
    parent
        .spawn(Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(rgb(176, 160, 136)),
            ));
            row.spawn((
                Node {
                    width: Val::Px(width),
                    height: Val::Px(36.0),
                    padding: UiRect::horizontal(Val::Px(10.0)),
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(rgb(14, 12, 10)),
                BorderColor(rgb(82, 67, 50)),
            ))
            .with_children(|field| {
                field.spawn((
                    Text::new(value),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(rgb(240, 230, 212)),
                    marker,
                ));
            });
        });
}

fn palette_row(parent: &mut ChildBuilder, floor: Floor) {
    parent
        .spawn((
            Button,
            PaletteButton { floor },
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(52.0),
                padding: UiRect::horizontal(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(rgb(26, 23, 19)),
            BorderColor(rgb(58, 50, 40)),
        ))
        .with_children(|row| {
            row.spawn((
                Node {
                    width: Val::Px(32.0),
                    height: Val::Px(32.0),
                    ..default()
                },
                BackgroundColor(floor_color(floor)),
                FocusPolicy::Pass,
            ));
            pass_text(row, floor_name(floor), 16.0);
        });
}

fn pass_text(parent: &mut ChildBuilder, value: &str, size: f32) {
    parent.spawn((
        Text::new(value),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(rgb(240, 230, 212)),
        FocusPolicy::Pass,
    ));
}

fn palette_clicks(
    mut editor: ResMut<Editor>,
    query: Query<(&Interaction, &PaletteButton), Changed<Interaction>>,
) {
    for (interaction, button) in &query {
        if *interaction == Interaction::Pressed {
            editor.session.brush = button.floor;
        }
    }
}

fn command_clicks(
    mut editor: ResMut<Editor>,
    query: Query<(&Interaction, &CommandButton), Changed<Interaction>>,
) {
    for (interaction, command) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match command {
            CommandButton::Load => {
                if editor.session.reload() {
                    if let Some(room) = &editor.session.room {
                        info!("loaded room {}", room.id);
                    }
                }
            }
            CommandButton::Save => editor.session.save(),
        }
    }
}

fn paint_clicks(
    mut editor: ResMut<Editor>,
    mut cells: Query<
        (&Interaction, &CellButton, &mut BackgroundColor),
        (Changed<Interaction>, Without<PreviewTile>),
    >,
    mut tiles: Query<(&PreviewTile, &mut ImageNode), Without<CellButton>>,
    images: Res<PreviewImages>,
) {
    let brush = editor.session.brush;
    let mut painted = Vec::new();
    for (interaction, cell, mut background) in &mut cells {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if editor.session.paint(cell.x, cell.y) {
            *background = BackgroundColor(floor_color(brush));
            painted.push((cell.x, cell.y));
        }
    }
    let Some(diamonds) = images.diamonds.as_ref() else {
        return;
    };
    if painted.is_empty() {
        return;
    }
    let handle = diamonds[floor_slot(brush)].clone();
    for (tile, mut image) in &mut tiles {
        if painted.iter().any(|(x, y)| *x == tile.x && *y == tile.y) {
            image.image = handle.clone();
        }
    }
}

fn sync_map(
    mut commands: Commands,
    editor: Res<Editor>,
    mut images: ResMut<Assets<Image>>,
    mut preview_images: ResMut<PreviewImages>,
    grid: Query<Entity, With<GridHost>>,
    preview: Query<Entity, With<PreviewHost>>,
    mut seen: Local<Option<u32>>,
) {
    let generation = editor.session.generation;
    if *seen == Some(generation) {
        return;
    }
    *seen = Some(generation);
    let Ok(grid_entity) = grid.get_single() else {
        return;
    };
    let Ok(preview_entity) = preview.get_single() else {
        return;
    };
    commands.entity(grid_entity).despawn_descendants();
    commands.entity(preview_entity).despawn_descendants();

    let Some(room) = editor.session.room.clone() else {
        preview_images.diamonds = None;
        commands.entity(grid_entity).with_children(|host| {
            host.spawn((
                Text::new("Could not load this map."),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(rgb(176, 160, 136)),
            ));
        });
        return;
    };

    let cell = cell_side(room.width, room.height);
    let gap = 4.0;
    let grid_w = room.width as f32 * cell + (room.width - 1).max(0) as f32 * gap;
    commands.entity(grid_entity).with_children(|host| {
        host.spawn(Node {
            width: Val::Px(grid_w),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(gap),
            row_gap: Val::Px(gap),
            ..default()
        })
        .with_children(|grid| {
            for y in 0..room.height {
                for x in 0..room.width {
                    let floor = room.floor(x, y).unwrap_or(Floor::Stone);
                    grid.spawn((
                        Button,
                        CellButton { x, y },
                        Node {
                            width: Val::Px(cell),
                            height: Val::Px(cell),
                            ..default()
                        },
                        BackgroundColor(floor_color(floor)),
                    ));
                }
            }
        });
    });

    let (tile_w, tile_h) = preview_tile_size(room.width, room.height, 400.0, 760.0);
    let (span_w, span_h) = diamond_span(room.width, room.height, tile_w, tile_h);
    let mut built = [(); 3].map(|_| Handle::default());
    for (index, floor) in FLOORS.into_iter().enumerate() {
        let fill = floor_rgba(floor);
        built[index] = images.add(diamond(tile_w as u32, tile_h as u32, fill, edge_rgba(fill)));
    }
    let diamonds = built;
    preview_images.diamonds = Some(diamonds.clone());

    let mut order = Vec::new();
    for y in 0..room.height {
        for x in 0..room.width {
            order.push((x, y));
        }
    }
    order.sort_by_key(|(x, y)| (x + y, *x));

    commands.entity(preview_entity).with_children(|host| {
        host.spawn(Node {
            width: Val::Px(span_w),
            height: Val::Px(span_h),
            ..default()
        })
        .with_children(|stack| {
            for (x, y) in order {
                let floor = room.floor(x, y).unwrap_or(Floor::Stone);
                let (left, top) = diamond_offset(x, y, room.height, tile_w, tile_h);
                stack.spawn((
                    ImageNode::new(diamonds[floor_slot(floor)].clone()),
                    PreviewTile { x, y },
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(left),
                        top: Val::Px(top),
                        width: Val::Px(tile_w),
                        height: Val::Px(tile_h),
                        ..default()
                    },
                ));
            }
        });
    });
}

fn sync_labels(
    editor: Res<Editor>,
    mut status: Query<
        &mut Text,
        (
            With<StatusText>,
            Without<MapNameText>,
            Without<CellSizeText>,
        ),
    >,
    mut names: Query<
        &mut Text,
        (
            With<MapNameText>,
            Without<StatusText>,
            Without<CellSizeText>,
        ),
    >,
    mut sizes: Query<
        &mut Text,
        (
            With<CellSizeText>,
            Without<StatusText>,
            Without<MapNameText>,
        ),
    >,
) {
    if let Ok(mut text) = status.get_single_mut() {
        write_text(&mut text, editor.session.status());
    }
    if let Ok(mut text) = names.get_single_mut() {
        write_text(&mut text, editor.session.map_name());
    }
    if let Ok(mut text) = sizes.get_single_mut() {
        write_text(&mut text, editor.session.cell_size_label());
    }
}

fn write_text(text: &mut Text, value: String) {
    if text.0 != value {
        text.0 = value;
    }
}

fn style_palette(
    editor: Res<Editor>,
    mut query: Query<(
        &PaletteButton,
        &Interaction,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    for (button, interaction, mut background, mut border) in &mut query {
        let selected = button.floor == editor.session.brush;
        *border = BorderColor(if selected {
            rgb(240, 216, 155)
        } else {
            rgb(58, 50, 40)
        });
        *background = BackgroundColor(match interaction {
            Interaction::Pressed | Interaction::Hovered => rgb(48, 40, 30),
            Interaction::None if selected => rgb(36, 30, 22),
            Interaction::None => rgb(26, 23, 19),
        });
    }
}

fn style_commands(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (With<CommandButton>, Changed<Interaction>),
    >,
) {
    for (interaction, mut background) in &mut query {
        *background = BackgroundColor(match interaction {
            Interaction::Pressed | Interaction::Hovered => rgb(122, 102, 72),
            Interaction::None => rgb(91, 75, 55),
        });
    }
}

fn esc_quits(keys: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}

fn cell_side(width: i32, height: i32) -> f32 {
    let gap = 4.0;
    let max_w = 860.0;
    let max_h = 820.0;
    let by_w = (max_w - gap * (width - 1).max(0) as f32) / width.max(1) as f32;
    let by_h = (max_h - gap * (height - 1).max(0) as f32) / height.max(1) as f32;
    by_w.min(by_h).clamp(8.0, 72.0).floor()
}

fn floor_slot(floor: Floor) -> usize {
    match floor {
        Floor::Stone => 0,
        Floor::Grass => 1,
        Floor::Dirt => 2,
    }
}

fn floor_name(floor: Floor) -> &'static str {
    match floor {
        Floor::Stone => "Stone",
        Floor::Grass => "Grass",
        Floor::Dirt => "Dirt",
    }
}

fn floor_color(floor: Floor) -> Color {
    let [r, g, b, _] = floor_rgba(floor);
    rgb(r, g, b)
}

fn floor_rgba(floor: Floor) -> [u8; 4] {
    match floor {
        Floor::Stone => [139, 134, 128, 255],
        Floor::Grass => [94, 107, 60, 255],
        Floor::Dirt => [143, 94, 52, 255],
    }
}

fn edge_rgba(fill: [u8; 4]) -> [u8; 4] {
    [
        (fill[0] as f32 * 0.55) as u8,
        (fill[1] as f32 * 0.55) as u8,
        (fill[2] as f32 * 0.55) as u8,
        255,
    ]
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::srgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
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
