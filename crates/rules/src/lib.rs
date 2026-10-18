//! Pathfinder Remaster procedures used by the encounter: preparation, initiative,
//! action economy, Stride, Step, Strike, and Reactive Strike.
//!
//! Rendering and engine types stay outside this crate. Callers supply dice.

mod check;
mod combatant;
mod dice;
mod encounter;
mod grid;

pub use check::{degree_of_success, multiple_attack_penalty, Degree};
pub use combatant::{
    apply_effects, prepare_combatant, ClassFeature, CreatureTemplate, DamageFormula,
    PreparedCombatant, PreparedStrike, Side, StrikeTemplate, Synthetics,
};
pub use dice::{Dice, ScriptedDice, SeededDice};
pub use encounter::{
    advance_if_turn_done, declare_action, end_turn, legal_actions, reactors_for_left_square,
    start_encounter, Action, Combatant, CombatantId, Encounter, Event, Illegal, Outcome,
    StartError,
};
pub use grid::{
    minimum_feet, reachable_destinations, squares_within_feet, Grid, GridPos, FEET_PER_SQUARE,
};
