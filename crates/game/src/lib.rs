//! Bevy host: real-time exploration, then turn-based Stride and Strike.
//!
//! `GameMode` is the only mode switch. Rules and fixture loading live in `rules` and `content`.

mod session;
mod view;
mod world;

pub use session::{headless_report, EncounterPath, GameMode, SessionPlugin};
pub use view::PresentationPlugin;

use bevy::prelude::*;

pub fn run_window() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Genesis".into(),
                resolution: (1280.0_f32, 720.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.045, 0.04)))
        .insert_resource(EncounterPath::fixture())
        .add_plugins((SessionPlugin, PresentationPlugin))
        .run();
}
