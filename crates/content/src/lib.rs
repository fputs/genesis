//! Fixture schemas, validation, and `load_encounter`.
//!
//! This crate reads hand-authored JSON. It does not depend on the game engine.

mod load;

pub use load::{
    load_encounter, load_room, save_room, Cell, Creature, DamageSpec, EncounterFixture, Floor,
    LoadError, Room, Spawn, StrikeSpec, Trigger,
};
