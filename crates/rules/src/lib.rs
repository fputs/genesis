//! Pathfinder Remaster dice, checks, modifiers, and creature templates.
//!
//! Callers inject [`Dice`]. [`derive`] does not roll.
//! `ScriptedDice` is compiled for tests and the `test-support` feature.

#![forbid(unsafe_code)]

mod check;
mod combatant;
mod dice;
mod modifiers;

pub use check::{
    ArithmeticOverflow, AttackTotalTerm, CheckModifier, CheckRoll, DcModifier, DegreeOfSuccess,
    FlatRoll, MapInput, attack_check_total, dc_from_modifier, degree_of_success,
    fallback_unlisted_critical, flat_check, flat_checks_for_same_thing, multiple_attack_penalty,
    resolve_check,
};
pub use combatant::{
    BuildParts, CharacterBuild, CombatantId, CombatantState, ConditionSlot, Contribution, Creature,
    DieCount, DieCountError, FoldError, HpError, MeleeStrike, ReachError, ReachFeet,
    ShieldTemplate, TemplateError, Vital, aid_check, aid_circumstance_bonus, apply_dice_damage,
    apply_healing, base_contribution, contributions, damage_dealt, derive, fold_contributions,
    new_combatant_state, stealth_dc_from_modifier,
};
pub use dice::{Dice, FaceCountError, Faces, SeededDice};
pub use modifiers::{
    BonusType, ConditionId, Cover, Modifier, ModifierError, ModifierTotal, PenaltyType, SourceId,
    ac_modifiers, attack_modifiers, dc_from_total, perception_modifiers, total,
};

#[cfg(any(test, feature = "test-support"))]
pub use dice::ScriptedDice;
