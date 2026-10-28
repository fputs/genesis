//! Aid, Raise a Shield, Shield Block, and Reactive Strike.
//!
//! Player Core, Aid. Player Core, "Raise a Shield". Player Core, "Shield Block".
//! Player Core, "Reactive Strike". Player Core, "Reactions in Encounters".

use crate::aid::{AidOutcome, AidPreparation, FittingAidModifier, roll_aid};
use crate::check::{DegreeOfSuccess, degree_of_success};
use crate::combatant::{CombatantId, CombatantState, Vital, damage_dealt};
use crate::damage::adjust_damage;
use crate::detection::{Detections, resolve_targeting};
use crate::dice::Dice;
use crate::dying::apply_resolved_blow;
use crate::grid::{Size, Square};
use crate::modifiers::{Modifier, ModifierError};
use crate::positioning::{ActionCost, MoveAction};
use crate::reactive::{
    ActionIdentity, Disruption, ReactiveError, ReactiveOutcome, ReactiveStrike, TriggerClass,
    attack_bonus_fits, disruption, not_a_trigger, outside_reach, reactive_attack_total,
};
use crate::shield::{
    AttackBlowError, PlannedBlow, ShieldBlock, ShieldError, plan_attack_blow,
    raise_shield as raise_combatant_shield, reduce_shield, validate_raise,
};

use super::strike::{FlatGate, flat_gate, misses, roll_strike_damage};
use super::{Encounter, EncounterError, StrikeDelivery, StrikeOutcome, StrikeProfile, TurnSlot};
impl Encounter {
    /// Spends a reaction. This does not take an action or roll a die.
    ///
    /// An Alive combatant's reaction is available from encounter start (DEV-06). A combatant
    /// who is not Alive is [`EncounterError::NotConscious`] and the reaction is left as it was.
    ///
    /// Player Core, "Reactions in Encounters".
    pub fn spend_reaction(&mut self, id: CombatantId) -> Result<(), EncounterError> {
        self.require_reaction(id)?;
        let index = self.member_index(id)?;
        let member = self
            .members
            .get_mut(index)
            .ok_or(EncounterError::UnknownCombatant)?;
        member.participant.state.reaction_ready = false;
        Ok(())
    }

    /// Spends one action and names `ally` for Aid.
    ///
    /// Refusals follow [`Self::spend_actions`] first: no open turn, not your turn, not Alive,
    /// then fewer than 1 action. The named ally is next: unknown, then self, then a different
    /// side, then [`EncounterError::AllyDead`] when that ally is [`Vital::Dead`]. An Unconscious
    /// or Dying ally is accepted, because they may wake before the aider's next turn. A missing
    /// Aid modifier is next. Nothing is spent or stored on those errors. Only then does this
    /// call [`Self::spend_actions`] and record the ally. A second call spends another action and
    /// replaces the named ally.
    ///
    /// Player Core, Aid. Player Core, Unconscious. Player Core, "Getting Knocked Out".
    /// Player Core, "Step 2: Act".
    pub fn prepare_to_aid(
        &mut self,
        aider: CombatantId,
        ally: CombatantId,
    ) -> Result<(), EncounterError> {
        self.require_active_alive(aider)?;
        let aider_index = self.member_index(aider)?;
        let (actions, aider_side, has_aid_modifier) = {
            let aider_member = self
                .members
                .get(aider_index)
                .ok_or(EncounterError::UnknownCombatant)?;
            (
                aider_member.participant.state.actions_remaining,
                aider_member.participant.side,
                aider_member.participant.creature.aid_modifier.is_some(),
            )
        };
        if actions < 1 {
            return Err(EncounterError::InsufficientActions);
        }
        let ally_index = self.member_index(ally)?;
        if aider == ally {
            return Err(EncounterError::SelfTarget);
        }
        let ally_member = self
            .members
            .get(ally_index)
            .ok_or(EncounterError::UnknownCombatant)?;
        let ally_side = ally_member.participant.side;
        let ally_vital = ally_member.participant.state.vital;
        if aider_side != ally_side {
            return Err(EncounterError::NotAnAlly);
        }
        if ally_vital == Vital::Dead {
            return Err(EncounterError::AllyDead);
        }
        if !has_aid_modifier {
            return Err(EncounterError::NoAidModifier);
        }
        self.spend_actions(aider, 1)?;
        self.members
            .get_mut(aider_index)
            .ok_or(EncounterError::UnknownCombatant)?
            .participant
            .state
            .aid_preparation = AidPreparation::Prepared { ally };
        Ok(())
    }

    /// Rolls one Aid check for an attack roll and spends the reaction through [`Self::spend_reaction`].
    ///
    /// The book's trigger is an ally about to use an action that requires a skill check or an
    /// attack roll. This narrows that trigger to an attack roll (DEV-51). Willing is the same
    /// [`Side`], which is DEV-17 and DEV-26. An `Ok` result clears preparation. The preparation
    /// names one ally and ends at the next `start_turn` or at `Ok`, whichever comes first
    /// (DEV-50). An error does not clear it and draws no die. A Dead ally is
    /// [`EncounterError::AllyDead`]. Unconscious and Dying are not an error. The reaction is
    /// spent only through [`Self::spend_reaction`], after the roll, and only when this call
    /// produces that roll. Actions, Hit Points, and the attack count stay as they were.
    ///
    /// Player Core, Aid. Player Core, "Step 2: Act". Player Core, Unconscious.
    /// Player Core, "Getting Knocked Out". Player Core, "Reactions in Encounters".
    pub fn aid(
        &mut self,
        aider: CombatantId,
        ally: CombatantId,
        dice: &mut impl Dice,
    ) -> Result<AidOutcome, EncounterError> {
        let aider_index = self.member_index(aider)?;
        let ally_index = self.member_index(ally)?;
        if self.open.is_none() {
            return Err(EncounterError::TurnNotOpen);
        }
        let (vital, aider_side, preparation, modifier, rank) = {
            let aider_member = self
                .members
                .get(aider_index)
                .ok_or(EncounterError::UnknownCombatant)?;
            (
                aider_member.participant.state.vital,
                aider_member.participant.side,
                aider_member.participant.state.aid_preparation,
                aider_member.participant.creature.aid_modifier,
                aider_member.participant.creature.aid_rank,
            )
        };
        if vital != Vital::Alive {
            return Err(EncounterError::NotConscious);
        }
        self.reject_delayed(aider)?;
        if aider == ally {
            return Err(EncounterError::SelfTarget);
        }
        let (ally_side, ally_vital) = {
            let ally_member = self
                .members
                .get(ally_index)
                .ok_or(EncounterError::UnknownCombatant)?;
            (
                ally_member.participant.side,
                ally_member.participant.state.vital,
            )
        };
        if aider_side != ally_side {
            return Err(EncounterError::NotAnAlly);
        }
        match preparation {
            AidPreparation::Prepared { ally: named } if named != ally => {
                return Err(EncounterError::NotThePreparedAlly);
            }
            AidPreparation::NotPrepared => return Err(EncounterError::NotPrepared),
            AidPreparation::Prepared { .. } => {}
        }
        if ally_vital == Vital::Dead {
            return Err(EncounterError::AllyDead);
        }
        let modifier = match modifier {
            Some(modifier) => modifier,
            None => return Err(EncounterError::NoAidModifier),
        };
        let modifier = FittingAidModifier::new(modifier).map_err(EncounterError::AidOverflow)?;
        self.require_reaction(aider)?;
        let (roll, circumstance) =
            roll_aid(dice, modifier, rank).map_err(EncounterError::AidOverflow)?;
        self.spend_reaction(aider)?;
        self.members
            .get_mut(aider_index)
            .ok_or(EncounterError::UnknownCombatant)?
            .participant
            .state
            .aid_preparation = AidPreparation::NotPrepared;
        Ok(AidOutcome {
            total: roll.total,
            degree: roll.degree,
            circumstance,
        })
    }

    /// Raises the active combatant's shield and spends 1 action through [`Self::spend_actions`].
    ///
    /// [`EncounterError::NotYourTurn`] is returned before any write when `id` is not the
    /// active combatant. The shield checks run next. A broken or destroyed shield does not
    /// spend the action.
    ///
    /// Player Core, "Raise a Shield". Player Core, "Step 2: Act".
    pub fn raise_shield(&mut self, id: CombatantId) -> Result<(), EncounterError> {
        if self.open != Some(id) {
            return Err(EncounterError::NotYourTurn);
        }
        let index = self.member_index(id)?;
        {
            let member = self
                .members
                .get(index)
                .ok_or(EncounterError::UnknownCombatant)?;
            validate_raise(&member.participant.state, &member.participant.creature)
                .map_err(map_raise_error)?;
        }
        self.spend_actions(id, 1)?;
        let member = self
            .members
            .get_mut(index)
            .ok_or(EncounterError::UnknownCombatant)?;
        raise_combatant_shield(&mut member.participant.state, &member.participant.creature)
            .map_err(map_raise_error)
    }

    /// Resolves a requested Shield Block for `defender`.
    ///
    /// The defender is checked in this order: [`EncounterError::TargetNotConscious`],
    /// [`EncounterError::Delayed`], [`EncounterError::NoShield`], [`EncounterError::NotRaised`],
    /// [`EncounterError::Broken`], [`EncounterError::NoReaction`]. [`Self::strike`] with
    /// [`ShieldBlock::Requested`] uses that same order. The reaction is spent only through
    /// [`Self::spend_reaction`], and only after the block is legal. This does not call
    /// [`crate::shield::apply_attack_blow`]. A blow that fails restores the state, so the
    /// reaction spend is not kept.
    ///
    /// Player Core, "Shield Block".
    pub fn shield_block(
        &mut self,
        defender: CombatantId,
        amount: i32,
        degree: DegreeOfSuccess,
    ) -> Result<(), EncounterError> {
        let index = self.member_index(defender)?;
        {
            let member = self
                .members
                .get(index)
                .ok_or(EncounterError::UnknownCombatant)?;
            if member.participant.state.vital != Vital::Alive {
                return Err(EncounterError::TargetNotConscious);
            }
        }
        self.reject_delayed(defender)?;
        let plan = {
            let member = self
                .members
                .get(index)
                .ok_or(EncounterError::UnknownCombatant)?;
            plan_attack_blow(
                &member.participant.state,
                &member.participant.creature,
                amount,
                degree,
                ShieldBlock::Requested,
            )
            .map_err(map_attack_blow_error)?
        };
        match plan {
            PlannedBlow::NoWrite => Ok(()),
            PlannedBlow::Unblocked(amount) => self.apply_damage(defender, amount, degree),
            PlannedBlow::Blocked(remainder) => {
                let snapshot = self
                    .members
                    .get(index)
                    .ok_or(EncounterError::UnknownCombatant)?
                    .participant
                    .state
                    .clone();
                self.spend_reaction(defender)?;
                let member = self
                    .members
                    .get_mut(index)
                    .ok_or(EncounterError::UnknownCombatant)?;
                let participant = &mut member.participant;
                if let Err(error) = apply_resolved_blow(
                    &mut participant.state,
                    &participant.creature,
                    remainder,
                    degree,
                ) {
                    participant.state = snapshot;
                    return Err(EncounterError::HitPoints(error));
                }
                reduce_shield(&mut participant.state, remainder);
                Ok(())
            }
        }
    }

    /// Resolves one Reactive Strike from `reactor` against `actor`.
    ///
    /// Reach is the reactor's stored melee reach. Squares and sizes are the caller's.
    /// The reaction is spent only through [`Self::spend_reaction`], and only after a hit
    /// has been applied on a copy of the actor. A miss spends the reaction and rolls no
    /// damage die. The multiple attack penalty is 0 and the stored attack count is left
    /// as it was. Neither creature's actions change, and the turn is not ended.
    ///
    /// This does not call [`Self::strike`]. There is no Shield Block argument.
    ///
    /// Player Core, "Reactive Strike". Player Core, "Reactions in Encounters".
    /// Player Core, "Disrupting Actions". Player Core, "Multiple Attack Penalty".
    #[expect(
        clippy::too_many_arguments,
        reason = "the reactive-strike design names the reactor, actor, trigger, squares, sizes, strike, modifiers, detections, and dice as separate arguments"
    )]
    pub fn reactive_strike(
        &mut self,
        reactor: CombatantId,
        actor: CombatantId,
        cost: ActionCost,
        identity: ActionIdentity,
        move_action: Option<MoveAction>,
        availability: ReactiveStrike,
        reactor_square: Square,
        actor_square: Square,
        reactor_size: Size,
        actor_size: Size,
        profile: &StrikeProfile,
        modifiers: &[Modifier],
        detections: &Detections,
        dice: &mut impl Dice,
    ) -> Result<ReactiveOutcome, ReactiveError> {
        let reactor_vital = self
            .state(reactor)
            .map_err(|_| ReactiveError::UnknownCombatant)?
            .vital;
        let actor_vital = self
            .state(actor)
            .map_err(|_| ReactiveError::UnknownCombatant)?
            .vital;
        if reactor_vital != Vital::Alive {
            return Err(ReactiveError::NotConscious);
        }
        self.reject_delayed(reactor).map_err(from_spend)?;
        if reactor == actor {
            return Err(ReactiveError::SelfTarget);
        }
        if matches!(availability, ReactiveStrike::Unavailable) {
            return Err(ReactiveError::Unavailable);
        }
        if not_a_trigger(cost, identity, move_action) == TriggerClass::NotATrigger {
            return Err(ReactiveError::NotATrigger);
        }
        let reach = self
            .creature(reactor)
            .map_err(|_| ReactiveError::UnknownCombatant)?
            .strike
            .reach_feet
            .get();
        outside_reach(
            reactor_square,
            actor_square,
            reactor_size,
            actor_size,
            reach,
        )?;
        if actor_vital != Vital::Alive {
            return Err(ReactiveError::TargetNotConscious);
        }
        self.require_reaction(reactor).map_err(from_spend)?;
        attack_bonus_fits(profile.attack_bonus, modifiers)
            .map_err(ReactiveError::AttackOverflow)?;
        match flat_gate(resolve_targeting(detections, reactor, actor, false, dice)) {
            FlatGate::CannotTarget => return Err(ReactiveError::CannotTarget),
            FlatGate::Lost(lost) => {
                self.spend_reaction(reactor).map_err(from_spend)?;
                return Ok(ReactiveOutcome {
                    strike: StrikeDelivery::Lost(lost),
                    disruption: Disruption::NotDisrupted,
                });
            }
            FlatGate::Roll => {}
        }

        let natural = dice.d20();
        let total = reactive_attack_total(natural, profile.attack_bonus, modifiers)
            .map_err(ReactiveError::AttackOverflow)?;
        let armor_class = self
            .creature(actor)
            .map_err(|_| ReactiveError::UnknownCombatant)?
            .armor_class;
        let degree = degree_of_success(natural, total, armor_class);
        let damage = if misses(degree) {
            0
        } else {
            let defenses = self
                .creature(actor)
                .map_err(|_| ReactiveError::UnknownCombatant)?
                .defenses
                .clone();
            let rolled = roll_strike_damage(dice, profile).map_err(as_damage_overflow)?;
            let dealt = damage_dealt(
                rolled,
                modifiers,
                degree == DegreeOfSuccess::CriticalSuccess,
            )
            .map_err(ReactiveError::DamageOverflow)?;
            adjust_damage(dealt, profile.damage_type, &defenses)
                .map_err(ReactiveError::DamageOverflow)?
        };
        let actor_delay = self
            .delayed_target(actor)
            .map_err(|_| ReactiveError::UnknownCombatant)?;
        if !misses(degree) && damage > 0 {
            let mut updated = self
                .state(actor)
                .map_err(|_| ReactiveError::UnknownCombatant)?
                .clone();
            apply_resolved_blow(
                &mut updated,
                self.creature(actor)
                    .map_err(|_| ReactiveError::UnknownCombatant)?,
                damage,
                degree,
            )
            .map_err(ReactiveError::HitPoints)?;
            self.spend_reaction(reactor).map_err(from_spend)?;
            self.commit_reactive_blow(actor, &updated)?;
            self.place_delayed_knockout(actor, reactor, actor_delay);
        } else {
            self.spend_reaction(reactor).map_err(from_spend)?;
        }
        Ok(ReactiveOutcome {
            strike: StrikeDelivery::Rolled(StrikeOutcome {
                natural,
                total,
                degree,
                map: 0,
                damage,
            }),
            disruption: disruption(degree, cost),
        })
    }

    /// Copies the blow back. The reactor's reaction is already cleared by [`Self::spend_reaction`].
    ///
    /// `initiative_move` is the dying report [`apply_resolved_blow`] stores on the copy.
    /// Hit Points, temporary Hit Points, and vital are the fields the design names.
    /// `temp_rounds` is copied with the temporary Hit Points: spending the last of them
    /// clears the round count, and leaving the old count in place would disagree with Strike.
    /// Copying the whole state would also be the actor's state; the report still has to
    /// move, because the critical-hit scenario takes it.
    fn commit_reactive_blow(
        &mut self,
        actor: CombatantId,
        updated: &CombatantState,
    ) -> Result<(), ReactiveError> {
        let index = self
            .member_index(actor)
            .map_err(|_| ReactiveError::UnknownCombatant)?;
        let member = self
            .members
            .get_mut(index)
            .ok_or(ReactiveError::UnknownCombatant)?;
        let state = &mut member.participant.state;
        state.hit_points = updated.hit_points;
        state.temp_hit_points = updated.temp_hit_points;
        state.temp_rounds = updated.temp_rounds;
        state.vital = updated.vital;
        state.initiative_move = updated.initiative_move;
        // Death clears every condition on the copy. That clear has to land here.
        state.conditions.clone_from(&updated.conditions);
        Ok(())
    }

    pub(super) fn block_is_ready(&self, target: CombatantId) -> Result<(), EncounterError> {
        let state = self.state(target)?;
        let creature = self.creature(target)?;
        plan_attack_blow(
            state,
            creature,
            1,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .map(|_| ())
        .map_err(map_attack_blow_error)
    }

    fn reaction_gate(&self, id: CombatantId, gate: ReactionGate) -> Result<(), EncounterError> {
        let index = self.member_index(id)?;
        let member = self
            .members
            .get(index)
            .ok_or(EncounterError::UnknownCombatant)?;
        if member.participant.state.vital != Vital::Alive {
            return Err(EncounterError::NotConscious);
        }
        if member.slot == TurnSlot::Delayed {
            return Err(EncounterError::Delayed);
        }
        if gate == ReactionGate::Spend && !member.participant.state.reaction_ready {
            return Err(EncounterError::NoReaction);
        }
        Ok(())
    }

    pub(super) fn require_reaction(&self, id: CombatantId) -> Result<(), EncounterError> {
        self.reaction_gate(id, ReactionGate::Spend)
    }

    fn reject_delayed(&self, id: CombatantId) -> Result<(), EncounterError> {
        self.reaction_gate(id, ReactionGate::Delay)
    }

    pub(super) fn reject_requested_block(&self, target: CombatantId) -> Result<(), EncounterError> {
        let state = self.state(target)?;
        if state.vital != Vital::Alive {
            return Err(EncounterError::TargetNotConscious);
        }
        self.reject_delayed(target)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReactionGate {
    Delay,
    Spend,
}

fn map_raise_error(error: ShieldError) -> EncounterError {
    match error {
        ShieldError::NoShield => EncounterError::NoShield,
        ShieldError::Broken => EncounterError::Broken,
        ShieldError::Destroyed => EncounterError::Destroyed,
        ShieldError::NotConscious => EncounterError::NotConscious,
    }
}

fn map_attack_blow_error(error: AttackBlowError) -> EncounterError {
    match error {
        AttackBlowError::NotConscious => EncounterError::TargetNotConscious,
        AttackBlowError::NoShield => EncounterError::NoShield,
        AttackBlowError::NotRaised => EncounterError::NotRaised,
        AttackBlowError::Broken => EncounterError::Broken,
        AttackBlowError::NoReaction => EncounterError::NoReaction,
        AttackBlowError::NegativeAmount => EncounterError::NegativeAmount,
        AttackBlowError::HitPoints(error) => EncounterError::HitPoints(error),
    }
}

/// Maps a reaction spend. Each variant has its own arm so a new one does not
/// fall into `NoReaction`.
#[expect(
    clippy::match_same_arms,
    reason = "each EncounterError variant is named so a new one cannot collapse into NoReaction"
)]
pub(super) fn from_spend(error: EncounterError) -> ReactiveError {
    match error {
        EncounterError::UnknownCombatant => ReactiveError::UnknownCombatant,
        EncounterError::NotConscious => ReactiveError::NotConscious,
        EncounterError::NoReaction => ReactiveError::NoReaction,
        EncounterError::DuplicateCombatant => ReactiveError::NoReaction,
        EncounterError::EmptyRoster => ReactiveError::NoReaction,
        EncounterError::PerceptionOutOfRange => ReactiveError::NoReaction,
        EncounterError::TurnInProgress => ReactiveError::NoReaction,
        EncounterError::TurnNotOpen => ReactiveError::NoReaction,
        EncounterError::NotYourTurn => ReactiveError::NoReaction,
        EncounterError::InsufficientActions => ReactiveError::NoReaction,
        EncounterError::InvalidActionCost => ReactiveError::NoReaction,
        EncounterError::NoShield => ReactiveError::NoReaction,
        EncounterError::NotRaised => ReactiveError::NoReaction,
        EncounterError::Broken => ReactiveError::NoReaction,
        EncounterError::Destroyed => ReactiveError::NoReaction,
        EncounterError::NegativeAmount => ReactiveError::NoReaction,
        EncounterError::TargetDead => ReactiveError::NoReaction,
        EncounterError::TargetNotConscious => ReactiveError::NoReaction,
        EncounterError::SelfTarget => ReactiveError::NoReaction,
        EncounterError::NoLivingCombatant => ReactiveError::NoReaction,
        EncounterError::AttackCountOverflow => ReactiveError::NoReaction,
        EncounterError::RoundOverflow => ReactiveError::NoReaction,
        EncounterError::AttackOverflow(_) => ReactiveError::NoReaction,
        EncounterError::DamageOverflow(_) => ReactiveError::NoReaction,
        EncounterError::HitPoints(_) => ReactiveError::NoReaction,
        EncounterError::NotPrepared => ReactiveError::NoReaction,
        EncounterError::NoAidModifier => ReactiveError::NoReaction,
        EncounterError::NotAnAlly => ReactiveError::NoReaction,
        EncounterError::NotThePreparedAlly => ReactiveError::NoReaction,
        EncounterError::AllyDead => ReactiveError::NoReaction,
        EncounterError::AidOverflow(_) => ReactiveError::NoReaction,
        EncounterError::NoCreature => ReactiveError::NoReaction,
        EncounterError::NotDelayed => ReactiveError::NoReaction,
        EncounterError::TriggerNotMet => ReactiveError::NoReaction,
        EncounterError::Delayed => ReactiveError::Delayed,
        EncounterError::Grid(_) => ReactiveError::NoReaction,
        EncounterError::Positioning(_) => ReactiveError::NoReaction,
        EncounterError::NoSavingThrows => ReactiveError::NoReaction,
        EncounterError::SaveOverflow(_) => ReactiveError::NoReaction,
        EncounterError::NoSkills => ReactiveError::NoReaction,
        EncounterError::SkillOverflow(_) => ReactiveError::NoReaction,
        EncounterError::CannotTarget => ReactiveError::CannotTarget,
        EncounterError::NonPositivePersistent => ReactiveError::NoReaction,
        EncounterError::NoSpellcasting => ReactiveError::NoReaction,
        EncounterError::OutOfSpellRange => ReactiveError::NoReaction,
        EncounterError::UnsupportedCone => ReactiveError::NoReaction,
        EncounterError::DuplicateAreaTarget => ReactiveError::NoReaction,
        EncounterError::Condition(_) => ReactiveError::NoReaction,
    }
}

/// Maps a damage-roll failure. Each variant has its own arm so a new one does not
/// fall into [`ModifierError::Overflow`].
#[expect(
    clippy::match_same_arms,
    reason = "each EncounterError variant is named so a new one cannot collapse into DamageOverflow"
)]
pub(super) fn as_damage_overflow(error: EncounterError) -> ReactiveError {
    match error {
        EncounterError::DamageOverflow(error) => ReactiveError::DamageOverflow(error),
        EncounterError::DuplicateCombatant => {
            ReactiveError::DamageOverflow(ModifierError::Overflow)
        }
        EncounterError::EmptyRoster => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::PerceptionOutOfRange => {
            ReactiveError::DamageOverflow(ModifierError::Overflow)
        }
        EncounterError::UnknownCombatant => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::TurnInProgress => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::TurnNotOpen => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::NotYourTurn => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::InsufficientActions => {
            ReactiveError::DamageOverflow(ModifierError::Overflow)
        }
        EncounterError::InvalidActionCost => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::NoReaction => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::NotConscious => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::TargetNotConscious => {
            ReactiveError::DamageOverflow(ModifierError::Overflow)
        }
        EncounterError::NoShield => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::NotRaised => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::Broken => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::Destroyed => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::NegativeAmount => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::TargetDead => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::SelfTarget => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::NoLivingCombatant => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::AttackCountOverflow => {
            ReactiveError::DamageOverflow(ModifierError::Overflow)
        }
        EncounterError::RoundOverflow => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::AttackOverflow(error) => {
            let _ = error;
            ReactiveError::DamageOverflow(ModifierError::Overflow)
        }
        EncounterError::HitPoints(error) => {
            let _ = error;
            ReactiveError::DamageOverflow(ModifierError::Overflow)
        }
        EncounterError::NotPrepared => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::NoAidModifier => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::NotAnAlly => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::NotThePreparedAlly => {
            ReactiveError::DamageOverflow(ModifierError::Overflow)
        }
        EncounterError::AllyDead => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::AidOverflow(_) => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::NoCreature => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::NotDelayed => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::TriggerNotMet => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::Delayed => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::Grid(_) => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::Positioning(_) => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::NoSavingThrows => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::SaveOverflow(_) => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::NoSkills => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::SkillOverflow(_) => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::CannotTarget => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::NonPositivePersistent => {
            ReactiveError::DamageOverflow(ModifierError::Overflow)
        }
        EncounterError::NoSpellcasting => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::OutOfSpellRange => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::UnsupportedCone => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::DuplicateAreaTarget => {
            ReactiveError::DamageOverflow(ModifierError::Overflow)
        }
        EncounterError::Condition(_) => ReactiveError::DamageOverflow(ModifierError::Overflow),
    }
}
