//! Pathfinder Remaster dice, checks, modifiers, grid movement, creature templates, encounters, and dying.
//!
//! Callers inject [`Dice`]. [`derive`] does not roll.
//! `ScriptedDice` is compiled for tests and the `test-support` feature.

#![forbid(unsafe_code)]

mod check;
mod combatant;
mod dice;
mod dying;
mod encounter;
mod grid;
mod modifiers;

pub use check::{
    ArithmeticOverflow, AttackTotalTerm, CheckModifier, CheckRoll, DcModifier, DegreeOfSuccess,
    FlatRoll, MapInput, attack_check_total, dc_from_modifier, degree_of_success,
    fallback_unlisted_critical, flat_check, flat_checks_for_same_thing, multiple_attack_penalty,
    resolve_check,
};
pub use combatant::{
    BuildParts, CharacterBuild, CombatantId, CombatantState, ConditionSlot, Contribution, Creature,
    DieCount, DieCountError, DyingValue, DyingValueError, FoldError, HpError, MeleeStrike,
    ReachError, ReachFeet, ShieldTemplate, TemplateError, Vital, aid_check, aid_circumstance_bonus,
    apply_dice_damage, apply_healing, base_contribution, contributions, damage_dealt, derive,
    fold_contributions, new_combatant_state, stealth_dc_from_modifier,
};
pub use dice::{Dice, FaceCountError, Faces, SeededDice};
pub use dying::{
    EncounterOutcome, InitiativeMove, RosterEntry, apply_resolved_blow, encounter_outcome,
    grant_temporary_hit_points, start_of_turn, take_initiative_move,
};
pub use encounter::{
    Agility, Encounter, EncounterError, Participant, Side, StrikeOutcome, StrikeProfile,
    begin_encounter,
};
pub use modifiers::{
    BonusType, ConditionId, Cover, Modifier, ModifierError, ModifierTotal, PenaltyType, SourceId,
    Statistic, ac_modifiers, attack_modifiers, dc_from_total, perception_modifiers,
    reflex_modifiers, total,
};

#[cfg(any(test, feature = "test-support"))]
pub use dice::ScriptedDice;
pub use grid::{
    DiagonalCounter, FactionId, GridError, MapBounds, MoveInput, MoveOutcome, Occupant, RangeBand,
    Size, SizeData, Space, Square, SquareEvent, Terrain, adjacent, distance, in_reach,
    move_trait_allowed, range_band, size_data, step, stride, validate_speed,
};
