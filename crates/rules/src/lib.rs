//! Pathfinder Remaster procedures used by the encounter: preparation, initiative,
//! action economy, Stride, Step, Crawl, Leap, Drop Prone, Stand, Strike, Seek, Point Out,
//! Take Cover, Interact, Release, Delay, Ready, Aid, Raise a Shield, Shield Block, Reactive Strike,
//! cover, difficult terrain, and the dying track.
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
    PreparedCombatant, PreparedStrike, Shield, Side, StrikeTemplate, Synthetics,
};
pub use dice::{Dice, ScriptedDice, SeededDice};
pub use encounter::{
    advance_if_turn_done, declare_action, end_turn, legal_actions, reactors_for_left_square,
    resolve_delay_resume, resolve_pending_aid, resolve_pending_shield_block, start_encounter,
    Action, AidOffer, Combatant, CombatantId, Detection, Encounter, Event, Gear, Illegal, Outcome,
    Readied, ReadiedAction, ReadyTrigger, SeekRoll, ShieldBlockOffer, StartError,
};
pub use grid::{
    minimum_feet, reachable_destinations, squares_within_feet, Grid, GridPos, Terrain,
    FEET_PER_SQUARE,
};
