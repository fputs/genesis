//! Hit point loss, the dying track, and basic saves.
//!
//! A Strike rolls in `strike`. Writing the blow, including a basic save that never
//! rolls an attack, lives here.
//!
//! Player Core, Hit Points. Player Core pg. 404, Basic Saving Throws.
//! Player Core pg. 410, Getting Knocked Out.

use crate::check::{DegreeOfSuccess, resolve_check};
use crate::combatant::{CombatantId, Vital};
use crate::conditions::condition_modifiers;
use crate::dice::Dice;
use crate::dying::{InitiativeMove, apply_resolved_blow};
use crate::modifiers::{Modifier, ModifierError};
use crate::save::{Save, SaveError, SaveRoll, basic_save_amount, save_modifier};

use super::{Encounter, EncounterError, TurnSlot};

impl Encounter {
    /// Assigns vital. Hit Points, actions, and the reaction stay as they are.
    #[cfg(test)]
    pub(crate) fn set_vital_for_test(&mut self, id: CombatantId, vital: Vital) {
        if let Ok(index) = self.member_index(id)
            && let Some(member) = self.members.get_mut(index)
        {
            member.participant.state.vital = vital;
        }
    }

    pub(super) fn apply_damage(
        &mut self,
        target: CombatantId,
        amount: i32,
        degree: DegreeOfSuccess,
    ) -> Result<(), EncounterError> {
        let index = self.member_index(target)?;
        let member = self
            .members
            .get_mut(index)
            .ok_or(EncounterError::UnknownCombatant)?;
        let participant = &mut member.participant;
        apply_resolved_blow(
            &mut participant.state,
            &participant.creature,
            amount,
            degree,
        )
        .map_err(EncounterError::HitPoints)
    }

    /// Rolls one basic save for `defender` and applies the multiplied damage.
    ///
    /// Unknown, dead, a negative amount, missing training, and a modifier that does not fit
    /// are rejected before the die. A multiplied 0 does not call [`apply_resolved_blow`].
    /// A positive failure is an ordinary blow. A positive critical failure is a critical blow
    /// and is not doubled again. While a turn is open, every new knockout moves `defender`
    /// to directly before that creature. The save spends no action, reaction, or attack.
    ///
    /// Player Core pg. 404, Saving Throws. Player Core pg. 404, Basic Saving Throws.
    /// Player Core pg. 410, Getting Knocked Out.
    pub fn basic_save(
        &mut self,
        defender: CombatantId,
        save: Save,
        dc: i32,
        amount: i32,
        modifiers: &[Modifier],
        dice: &mut impl Dice,
    ) -> Result<SaveRoll, EncounterError> {
        let index = self.member_index(defender)?;
        let member = self
            .members
            .get(index)
            .ok_or(EncounterError::UnknownCombatant)?;
        let vital = member.participant.state.vital;
        let training = member.participant.creature.saves;
        if vital == Vital::Dead {
            return Err(EncounterError::TargetDead);
        }
        if amount < 0 {
            return Err(EncounterError::NegativeAmount);
        }
        let Some(training) = training else {
            return Err(EncounterError::NoSavingThrows);
        };
        let unconscious = matches!(vital, Vital::Dying(_) | Vital::Unconscious);
        let tracked = condition_modifiers(
            &member.participant.state.conditions,
            save.statistic(),
            None,
            None,
        )
        .map_err(|_| {
            EncounterError::SaveOverflow(crate::check::ArithmeticOverflow::SumDoesNotFit)
        })?;
        let mut save_modifiers = tracked;
        save_modifiers.extend_from_slice(modifiers);
        let modifier =
            save_modifier(&training, save, unconscious, &save_modifiers).map_err(|error| {
                match error {
                    SaveError::ModifierOverflow(error) => EncounterError::SaveOverflow(error),
                    SaveError::NegativeAmount => EncounterError::NegativeAmount,
                    SaveError::DoubleOverflow(error) => EncounterError::SaveOverflow(error),
                }
            })?;
        let check = resolve_check(dice, modifier, dc);
        let damage = match basic_save_amount(amount, check.degree) {
            Ok(damage) => damage,
            Err(SaveError::DoubleOverflow(_)) => {
                return Err(EncounterError::DamageOverflow(ModifierError::Overflow));
            }
            Err(SaveError::NegativeAmount) => return Err(EncounterError::NegativeAmount),
            Err(SaveError::ModifierOverflow(error)) => {
                return Err(EncounterError::SaveOverflow(error));
            }
        };
        if damage > 0 {
            let blow = match check.degree {
                DegreeOfSuccess::CriticalFailure => DegreeOfSuccess::CriticalSuccess,
                DegreeOfSuccess::CriticalSuccess
                | DegreeOfSuccess::Success
                | DegreeOfSuccess::Failure => DegreeOfSuccess::Success,
            };
            self.apply_damage(defender, damage, blow)?;
            self.place_save_knockout(defender);
        }
        Ok(SaveRoll {
            natural: check.natural,
            total: check.total,
            degree: check.degree,
            damage,
        })
    }

    /// Moves a save knockout to directly before the open creature.
    ///
    /// Strike still uses [`Self::place_delayed_knockout`], which returns early unless the
    /// target had Delayed. A save uses the open turn as the anchor whenever the blow newly
    /// stored a dying value. A multiplied 0 never reaches this function.
    ///
    /// Player Core pg. 410, Getting Knocked Out.
    pub(crate) fn place_save_knockout(&mut self, defender: CombatantId) {
        let Some(anchor) = self.open else {
            return;
        };
        let pending = self
            .state(defender)
            .ok()
            .and_then(|state| state.initiative_move);
        if pending != Some(InitiativeMove::Pending) {
            return;
        };
        self.order.retain(|existing| *existing != defender);
        match self.order.iter().position(|existing| *existing == anchor) {
            Some(index) => self.order.insert(index, defender),
            None => self.order.push(defender),
        }
        if let Ok(index) = self.member_index(defender)
            && let Some(member) = self.members.get_mut(index)
        {
            member.slot = TurnSlot::Normal;
            member.participant.state.initiative_move = None;
        }
        if let Some(open) = self.open
            && let Some(cursor) = self.order.iter().position(|existing| *existing == open)
        {
            self.cursor = cursor;
        }
    }
}
