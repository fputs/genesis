//! Skill checks and Recall Knowledge.
//!
//! `skill_check` is a primitive for a caller-adjudicated check. It is not the creature
//! taking an action. Recall Knowledge is one action on the actor's open Alive turn.
//!
//! Player Core pg. 405, Skill Checks. Player Core pg. 231, Recall Knowledge.
//! Player Core pg. 446, Unconscious.

use crate::check::resolve_check;
use crate::dice::Dice;
use crate::modifiers::Modifier;
use crate::skill::{Skill, SkillError, SkillRoll, skill_modifier};

use super::{Encounter, EncounterError};

impl Encounter {
    /// Rolls `skill` for `actor` against the caller DC.
    ///
    /// This is a primitive for a check the caller has already decided to resolve.
    /// It does not read vital, the open turn, actions, or the attack count, and it
    /// spends nothing. An unconscious creature still rolls, with no unconscious penalty.
    /// Unknown, missing training, and a modifier that does not fit are rejected before the die.
    ///
    /// Player Core pg. 405, Skill Checks. Player Core pg. 446, Unconscious.
    pub fn skill_check(
        &mut self,
        actor: crate::combatant::CombatantId,
        skill: Skill,
        dc: i32,
        modifiers: &[Modifier],
        dice: &mut impl Dice,
    ) -> Result<SkillRoll, EncounterError> {
        let index = self.member_index(actor)?;
        let training = self
            .members
            .get(index)
            .ok_or(EncounterError::UnknownCombatant)?
            .participant
            .creature
            .skills;
        let Some(training) = training else {
            return Err(EncounterError::NoSkills);
        };
        let modifier = skill_modifier(&training, skill, modifiers).map_err(skill_overflow)?;
        let check = resolve_check(dice, modifier, dc);
        Ok(SkillRoll {
            natural: check.natural,
            total: check.total,
            degree: check.degree,
        })
    }

    /// Recall Knowledge: one action, then the named skill check.
    ///
    /// The order is the open Alive turn, at least one action, training, a modifier that
    /// fits, then the spend, then the die. A rejection draws no die and spends nothing.
    /// The attack count stays unchanged. The result is the degree and the natural roll.
    /// The concentrate trait is not stored.
    ///
    /// Player Core pg. 231, Recall Knowledge. Player Core pg. 436, Step 2: Act.
    pub fn recall_knowledge(
        &mut self,
        actor: crate::combatant::CombatantId,
        skill: Skill,
        dc: i32,
        modifiers: &[Modifier],
        dice: &mut impl Dice,
    ) -> Result<SkillRoll, EncounterError> {
        self.require_active_alive(actor)?;
        let index = self.member_index(actor)?;
        let actions = self
            .members
            .get(index)
            .ok_or(EncounterError::UnknownCombatant)?
            .participant
            .state
            .actions_remaining;
        if actions < 1 {
            return Err(EncounterError::InsufficientActions);
        }
        let training = self
            .members
            .get(index)
            .ok_or(EncounterError::UnknownCombatant)?
            .participant
            .creature
            .skills;
        let Some(training) = training else {
            return Err(EncounterError::NoSkills);
        };
        let modifier = skill_modifier(&training, skill, modifiers).map_err(skill_overflow)?;
        self.spend_actions(actor, 1)?;
        let check = resolve_check(dice, modifier, dc);
        Ok(SkillRoll {
            natural: check.natural,
            total: check.total,
            degree: check.degree,
        })
    }
}

fn skill_overflow(error: SkillError) -> EncounterError {
    match error {
        SkillError::ModifierOverflow(error) => EncounterError::SkillOverflow(error),
    }
}
