//! Pathfinder Remaster dice, checks, and creature templates.
//!
//! Callers inject [`Dice`]. [`derive`] does not roll.
//! `ScriptedDice` is compiled for tests and the `test-support` feature.

#![forbid(unsafe_code)]

mod check;
mod combatant;
mod dice;

pub use check::{
    CheckRoll, DegreeOfSuccess, FlatRoll, MapInput, attack_check_total, dc_from_modifier,
    degree_of_success, fallback_unlisted_critical, flat_check, flat_checks_for_same_thing,
    multiple_attack_penalty, resolve_check,
};
pub use combatant::{
    BuildParts, CharacterBuild, CombatantId, CombatantState, ConditionSlot, Contribution, Creature,
    DieCount, DieCountError, FoldError, MeleeStrike, ReachError, ReachFeet, ShieldTemplate,
    TemplateError, Vital, aid_check, aid_circumstance_bonus, apply_dice_damage, base_contribution,
    contributions, derive, fold_contributions, new_combatant_state, stealth_dc_from_modifier,
};
pub use dice::{Dice, FaceCountError, Faces, SeededDice};

#[cfg(any(test, feature = "test-support"))]
pub use dice::ScriptedDice;
