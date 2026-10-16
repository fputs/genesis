//! Fixture schemas, validation, and `load_encounter`.
//!
//! This crate reads hand-authored JSON. It does not depend on the game engine.

mod load;

pub use load::{
    load_encounter, Cell, Creature, DamageSpec, EncounterFixture, LoadError, Room, Spawn,
    StrikeSpec, Trigger,
};
