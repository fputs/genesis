//! Strike on the active turn: the flat check, the attack roll, and the multiple attack penalty.
//!
//! Player Core, Strike. Player Core, "Multiple Attack Penalty".
//! Player Core pg. 400, "Checks (Flat Checks)".

use crate::check::{
    AttackTotalTerm, DegreeOfSuccess, MapInput, attack_check_total, degree_of_success,
    multiple_attack_penalty,
};
use crate::combatant::{CombatantId, Vital, damage_dealt};
use crate::conditions::condition_modifiers;
use crate::damage::{adjust_damage, shield_trigger};
use crate::detection::{DetectionError, Detections, TargetingResult, resolve_targeting};
use crate::dice::Dice;
use crate::modifiers::{Modifier, ModifierError, Statistic, total};
use crate::shield::ShieldBlock;

use super::{
    Agility, Encounter, EncounterError, LostStrike, StrikeDelivery, StrikeOutcome, StrikeProfile,
};

impl Encounter {
    /// Multiple attack penalty for an attack-trait check, without incrementing the count.
    ///
    /// It is this creature's turn only when the open turn is `id`. Off that turn the penalty
    /// is 0. This does not store a snapshot of an earlier penalty.
    ///
    /// Player Core, "Multiple Attack Penalty".
    pub fn attack_trait_penalty(
        &self,
        id: CombatantId,
        agility: Agility,
    ) -> Result<i32, EncounterError> {
        let on_own_turn = self.open == Some(id);
        if self.member_index(id).is_err() {
            return Err(EncounterError::UnknownCombatant);
        }
        let prior = if on_own_turn {
            self.attack_count(id)?
        } else {
            0
        };
        Ok(multiple_attack_penalty(MapInput {
            on_own_turn,
            prior_attack_trait_checks_this_turn: prior,
            current_weapon_agile: matches!(agility, Agility::Agile),
            readied_attack_count: None,
        }))
    }

    /// Strikes `target` with `profile` on the active creature's turn.
    ///
    /// `detections` is the caller's map. An empty map draws no flat-check die. A concealed or
    /// hidden target rolls that check before the attack d20. A lost check spends the action,
    /// increments the attack count, and rolls no attack d20 and no damage die. An undetected
    /// target is [`EncounterError::CannotTarget`] and spends nothing. This does not call
    /// `note_strike`. A miss spends the action, records damage 0, and does not roll a damage die.
    /// [`HpError::NotAHit`] is not returned from this path. Success and critical success roll
    /// the damage dice once; a critical success doubles that total inside [`damage_dealt`].
    /// A hit passes that total and `block` through the encounter block once. It does not also
    /// call [`apply_resolved_blow`]. [`ShieldBlock::Requested`] checks the shield before any
    /// die is drawn and spends no action when that check fails. A miss does not resolve a
    /// block. The degree uses the target's baked Armor Class. A requested reaction is spent
    /// only through [`Self::spend_reaction`].
    ///
    /// Player Core, Strike. Player Core, "Shield Block".
    #[expect(
        clippy::too_many_arguments,
        reason = "the strike design names the attacker, target, profile, modifiers, shield block, detections, and dice as separate arguments"
    )]
    pub fn strike(
        &mut self,
        attacker: CombatantId,
        target: CombatantId,
        profile: &StrikeProfile,
        modifiers: &[Modifier],
        block: ShieldBlock,
        detections: &Detections,
        dice: &mut impl Dice,
    ) -> Result<StrikeDelivery, EncounterError> {
        self.prepare_strike(attacker, target)?;
        let delayed_target = self.delayed_target(target)?;
        let damage_type = profile.damage_type;
        let defenses = self.creature(target)?.defenses.clone();
        if let ShieldBlock::Requested = block {
            self.reject_requested_block(target)?;
            if shield_trigger(damage_type, &defenses) {
                self.block_is_ready(target)?;
            }
        }
        let map = self.attack_trait_penalty(attacker, profile.agility)?;
        let attack_term = bounded_term(profile.attack_bonus)?;
        let map_term = bounded_term(map)?;
        let zero = bounded_term(0)?;
        match flat_gate(resolve_targeting(detections, attacker, target, false, dice)) {
            FlatGate::CannotTarget => return Err(EncounterError::CannotTarget),
            FlatGate::Lost(lost) => {
                self.finish_strike(attacker)?;
                return Ok(StrikeDelivery::Lost(lost));
            }
            FlatGate::Roll => {}
        }
        let attack_conditions = condition_modifiers(
            &self.state(attacker)?.conditions,
            Statistic::Attack,
            Some(profile.attack_basis),
            None,
        )
        .map_err(EncounterError::DamageOverflow)?;
        let damage_conditions = condition_modifiers(
            &self.state(attacker)?.conditions,
            Statistic::Damage,
            None,
            Some(profile.damage_basis),
        )
        .map_err(EncounterError::DamageOverflow)?;
        let armor_conditions = condition_modifiers(
            &self.state(target)?.conditions,
            Statistic::ArmorClass,
            None,
            None,
        )
        .map_err(EncounterError::DamageOverflow)?;
        let armor_delta = total(&armor_conditions)
            .map_err(EncounterError::DamageOverflow)?
            .get(Statistic::ArmorClass);
        let armor_class = self
            .creature(target)?
            .armor_class
            .checked_add(armor_delta)
            .ok_or(EncounterError::DamageOverflow(ModifierError::Overflow))?;
        let natural = dice.d20();
        let mut folded_attack = attack_conditions;
        folded_attack.extend(attack_modifiers(modifiers));
        let total = attack_check_total(natural, attack_term, map_term, zero, zero, &folded_attack)
            .map_err(EncounterError::AttackOverflow)?;
        let degree = degree_of_success(natural, total, armor_class);
        let damage = if misses(degree) {
            0
        } else {
            let rolled = roll_strike_damage(dice, profile)?;
            let mut damage_modifiers = modifiers.to_vec();
            damage_modifiers.extend(damage_conditions);
            let dealt = damage_dealt(
                rolled,
                &damage_modifiers,
                degree == DegreeOfSuccess::CriticalSuccess,
            )
            .map_err(EncounterError::DamageOverflow)?;
            let adjusted = adjust_damage(dealt, damage_type, &defenses)
                .map_err(EncounterError::DamageOverflow)?;
            if adjusted > 0 {
                if shield_trigger(damage_type, &defenses) && matches!(block, ShieldBlock::Requested)
                {
                    self.shield_block(target, adjusted, degree)?;
                } else {
                    self.apply_damage(target, adjusted, degree)?;
                }
                self.place_delayed_knockout(target, attacker, delayed_target);
            }
            adjusted
        };
        self.finish_strike(attacker)?;
        Ok(StrikeDelivery::Rolled(StrikeOutcome {
            natural,
            total,
            degree,
            map,
            damage,
        }))
    }

    /// Assigns the stored attack count for `id`.
    #[cfg(test)]
    pub(crate) fn set_attack_count_for_test(&mut self, id: CombatantId, count: u8) {
        if let Ok(index) = self.member_index(id)
            && let Some(member) = self.members.get_mut(index)
        {
            member.attack_count = count;
        }
    }

    /// Attack-trait checks this creature has already made.
    ///
    /// Player Core, "Multiple Attack Penalty".
    pub fn attack_count(&self, id: CombatantId) -> Result<u8, EncounterError> {
        let index = self.member_index(id)?;
        self.members
            .get(index)
            .map(|member| member.attack_count)
            .ok_or(EncounterError::UnknownCombatant)
    }

    fn prepare_strike(
        &self,
        attacker: CombatantId,
        target: CombatantId,
    ) -> Result<(), EncounterError> {
        let Some(active) = self.open else {
            return Err(EncounterError::TurnNotOpen);
        };
        if attacker != active {
            return Err(EncounterError::NotYourTurn);
        }
        let attacker_state = self.state(attacker)?;
        if attacker_state.vital != Vital::Alive {
            return Err(EncounterError::NotConscious);
        }
        if attacker_state.actions_remaining < 1 {
            return Err(EncounterError::InsufficientActions);
        }
        let target_state = self.state(target)?;
        if attacker == target {
            return Err(EncounterError::SelfTarget);
        }
        if target_state.vital == Vital::Dead {
            return Err(EncounterError::TargetDead);
        }
        let _next = self
            .attack_count(attacker)?
            .checked_add(1)
            .ok_or(EncounterError::AttackCountOverflow)?;
        Ok(())
    }

    fn finish_strike(&mut self, attacker: CombatantId) -> Result<(), EncounterError> {
        let index = self.member_index(attacker)?;
        let member = self
            .members
            .get_mut(index)
            .ok_or(EncounterError::UnknownCombatant)?;
        let actions = member
            .participant
            .state
            .actions_remaining
            .checked_sub(1)
            .ok_or(EncounterError::InsufficientActions)?;
        let count = member
            .attack_count
            .checked_add(1)
            .ok_or(EncounterError::AttackCountOverflow)?;
        member.participant.state.actions_remaining = actions;
        member.attack_count = count;
        Ok(())
    }
}

pub(super) enum FlatGate {
    Roll,
    Lost(LostStrike),
    CannotTarget,
}

/// A lost flat check needs the face, the DC, and the degree. Anything else that does not
/// affect the target spends nothing: today's DCs do not produce that result.
pub(super) fn flat_gate(result: Result<TargetingResult, DetectionError>) -> FlatGate {
    let result = match result {
        Ok(result) => result,
        Err(
            DetectionError::CannotTarget
            | DetectionError::NotConscious
            | DetectionError::SameCreature
            | DetectionError::NoStealthDc
            | DetectionError::EmptyArea
            | DetectionError::AreaTooLarge
            | DetectionError::DuplicateId
            | DetectionError::Overflow
            | DetectionError::PerceptionOutOfRange
            | DetectionError::RequirementNotMet,
        ) => return FlatGate::CannotTarget,
    };
    if result.affected {
        return FlatGate::Roll;
    }
    if let (Some(natural), Some(dc), Some(degree)) = (result.natural, result.dc, result.degree) {
        FlatGate::Lost(LostStrike {
            natural,
            dc,
            degree,
        })
    } else {
        FlatGate::CannotTarget
    }
}

pub(super) fn misses(degree: DegreeOfSuccess) -> bool {
    matches!(
        degree,
        DegreeOfSuccess::Failure | DegreeOfSuccess::CriticalFailure
    )
}

fn bounded_term(value: i32) -> Result<AttackTotalTerm, EncounterError> {
    AttackTotalTerm::new(value).map_err(|_| EncounterError::AttackOverflow(ModifierError::Overflow))
}

fn attack_modifiers(modifiers: &[Modifier]) -> Vec<Modifier> {
    modifiers
        .iter()
        .copied()
        .filter(|modifier| modifier.statistic() == Statistic::Attack)
        .collect()
}

pub(super) fn roll_strike_damage(
    dice: &mut impl Dice,
    profile: &StrikeProfile,
) -> Result<i32, EncounterError> {
    let mut sum = 0i32;
    for _ in 0..profile.damage_dice.get() {
        let face = dice.roll(profile.damage_faces);
        sum = sum
            .checked_add(i32::from(face))
            .ok_or(EncounterError::DamageOverflow(ModifierError::Overflow))?;
    }
    sum.checked_add(profile.damage_bonus)
        .ok_or(EncounterError::DamageOverflow(ModifierError::Overflow))
}
