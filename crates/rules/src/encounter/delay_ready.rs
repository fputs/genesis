//! Delay, return, Ready, and the move or posture change that can fire them.
//!
//! Player Core, Delay. Player Core, "Ready". Player Core, "Stride". Player Core, "Step".

use crate::check::{
    AttackTotalTerm, DegreeOfSuccess, MapInput, attack_check_total, degree_of_success,
    multiple_attack_penalty,
};
use crate::combatant::{CombatantId, Vital, damage_dealt};
use crate::detection::{Detections, resolve_targeting};
use crate::dice::Dice;
use crate::dying::InitiativeMove;
use crate::grid::{
    DiagonalCounter, FactionId, GridError, MoveInput, MoveOutcome, Size, Square, distance,
    in_reach, range_band, step as grid_step, stride as grid_stride,
};
use crate::modifiers::ModifierError;
use crate::positioning::{
    ActionCost, CoverBenefit, Mobility, MoveAction, OccupantPresence, Posture, PostureChange,
    dismiss, drop_prone as positioning_drop_prone, stand as positioning_stand,
};
use crate::reactive::ActionIdentity;
use crate::ready::{
    ActivityKind, ObservedAction, ReactionField, ReactionPlace, ReactiveStrikeInput, ReadyActivity,
    ReadyEncounter, ReadyError, ReadyStrike, ReadySuccess, ReadyTrigger, ResolvedReaction,
    StoredReady, StrikeRange, entered_reach, single_action, stored_agility,
};

use super::strike::{FlatGate, flat_gate, misses, roll_strike_damage};
use super::turns::{TurnAdvance, side_rank};
use super::{
    Agility, Encounter, EncounterError, Side, StrikeProfile, TargetDelay, TurnSlot, TurnState,
};
impl Encounter {
    /// The ready stored for `id`, if any.
    ///
    /// Player Core, "Ready".
    pub fn stored_ready(&self, id: CombatantId) -> Result<Option<StoredReady>, EncounterError> {
        let index = self.member_index(id)?;
        self.members
            .get(index)
            .map(|member| member.ready)
            .ok_or(EncounterError::UnknownCombatant)
    }

    /// The attack count frozen when Ready resolved.
    ///
    /// `Ok(None)` is a creature with no stored ready.
    ///
    /// Player Core, "Ready". Player Core, "Multiple Attack Penalty".
    pub fn frozen_attack_count(&self, id: CombatantId) -> Result<Option<u8>, EncounterError> {
        Ok(self
            .stored_ready(id)?
            .map(|ready| ready.frozen_attack_count))
    }

    /// Leaves the order at the start of your turn.
    ///
    /// Legal only on the turn `start_turn` just granted: 3 actions, attack count 0, and the
    /// reaction ready. A turn opened by return is [`EncounterError::TriggerNotMet`]. Success
    /// spends nothing, sets actions to 0, marks the creature delayed, closes the turn, and
    /// does not advance the round.
    ///
    /// Player Core, Delay. Player Core, "Step 1: Start Your Turn".
    pub fn delay(&mut self, id: CombatantId) -> Result<(), EncounterError> {
        let _known = self.member_index(id)?;
        let Some(active) = self.open else {
            return Err(EncounterError::TurnNotOpen);
        };
        if active != id {
            return Err(EncounterError::NotYourTurn);
        }
        let state = self.state(id)?;
        if state.vital != Vital::Alive {
            return Err(EncounterError::NotConscious);
        }
        let triggered = state.actions_remaining == 3
            && self.attack_count(id)? == 0
            && state.reaction_ready
            && self.slot_of(id)? != TurnSlot::Returned;
        if !triggered {
            return Err(EncounterError::TriggerNotMet);
        }
        let advance = self.plan_advance(TurnAdvance::Delay)?;
        self.end_of_turn_step(id);
        self.close_actions(id);
        self.set_slot(id, TurnSlot::Delayed)?;
        self.open = None;
        self.last_ended = None;
        self.apply_advance(advance);
        Ok(())
    }

    /// Returns delayed creatures at the end of the turn that just ended.
    ///
    /// Enemies come before PCs, then the lower [`CombatantId`] (DEV-52, DEV-05). The first
    /// opens with 3 actions. Each later id waits as [`TurnSlot::SkipStartHook`]. The round
    /// does not advance. The reaction, the attack count, and temporary Hit Points stay.
    ///
    /// Player Core, Delay.
    pub fn return_from_delay(
        &mut self,
        ids: &[CombatantId],
    ) -> Result<CombatantId, EncounterError> {
        if ids.is_empty() {
            return Err(EncounterError::NoCreature);
        }
        if duplicate_ids(ids) {
            return Err(EncounterError::DuplicateCombatant);
        }
        for id in ids {
            let _known = self.member_index(*id)?;
        }
        if self.open.is_some() {
            return Err(EncounterError::TurnInProgress);
        }
        for id in ids {
            if self.slot_of(*id)? != TurnSlot::Delayed {
                return Err(EncounterError::NotDelayed);
            }
        }
        for id in ids {
            if self.state(*id)?.vital != Vital::Alive {
                return Err(EncounterError::NotConscious);
            }
        }
        let Some(anchor) = self.last_ended else {
            return Err(EncounterError::TriggerNotMet);
        };
        let _anchor = self.member_index(anchor)?;
        let mut ranked = Vec::new();
        for id in ids {
            let side = self.side_of(*id)?;
            ranked.push((*id, side));
        }
        ranked.sort_by(|left, right| {
            side_rank(left.1)
                .cmp(&side_rank(right.1))
                .then(left.0.cmp(&right.0))
        });
        let sorted: Vec<CombatantId> = ranked.into_iter().map(|(id, _side)| id).collect();
        self.insert_after(anchor, &sorted);
        let Some(first) = sorted.first().copied() else {
            return Err(EncounterError::NoCreature);
        };
        for (offset, id) in sorted.iter().copied().enumerate() {
            let slot = if offset == 0 {
                TurnSlot::Returned
            } else {
                TurnSlot::SkipStartHook
            };
            self.set_slot(id, slot)?;
        }
        self.open_return(first)?;
        self.turn_state = TurnState::Continuing;
        Ok(first)
    }

    /// Stores one trigger and one action, then ends the turn.
    ///
    /// The attack count is snapshotted and not changed. The reaction is not spent.
    /// Two actions are required. Ending the last turn of a round past the maximum is
    /// [`ReadyError::RoundOverflow`] and writes nothing.
    ///
    /// Player Core, "Ready".
    pub fn ready(
        &mut self,
        actor: CombatantId,
        trigger: ReadyTrigger,
        cost: ActionCost,
        activity: ReadyActivity,
    ) -> Result<(), ReadyError> {
        let actor_index = self.member_index(actor).map_err(ready_from_encounter)?;
        let subject = trigger.subject();
        let _subject = self.member_index(subject).map_err(ready_from_encounter)?;
        let Some(active) = self.open else {
            return Err(ReadyError::TurnNotOpen);
        };
        if active != actor {
            return Err(ReadyError::NotYourTurn);
        }
        let state = self.state(actor).map_err(ready_from_encounter)?;
        if state.vital != Vital::Alive {
            return Err(ReadyError::NotConscious);
        }
        single_action(activity, cost)?;
        if subject == actor {
            return Err(ReadyError::TriggerSubjectIsSelf);
        }
        if let ReadyActivity::Strike { target, .. } = activity
            && target != subject
        {
            return Err(ReadyError::NotTheTriggerSubject);
        }
        if state.actions_remaining < 2 {
            return Err(ReadyError::InsufficientActions);
        }
        if self.end_would_overflow() {
            return Err(ReadyError::RoundOverflow);
        }
        let frozen = self.attack_count(actor).map_err(ready_from_encounter)?;
        let stored = StoredReady {
            trigger,
            activity,
            frozen_attack_count: frozen,
        };
        let member = self
            .members
            .get_mut(actor_index)
            .ok_or(ReadyError::UnknownCombatant)?;
        member.ready = Some(stored);
        if let Err(error) = self.end_turn() {
            if let Some(member) = self.members.get_mut(actor_index) {
                member.ready = None;
            }
            return Err(ready_from_encounter(error));
        }
        Ok(())
    }

    /// The penalty a stored readied attack would take. This does not roll or write.
    ///
    /// Player Core, "Multiple Attack Penalty". Player Core, "Ready".
    pub fn readied_attack_penalty(&self, id: CombatantId) -> Result<i32, ReadyError> {
        let ready = self
            .stored_ready(id)
            .map_err(ready_from_encounter)?
            .ok_or(ReadyError::NotReadied)?;
        let agility = stored_agility(ready.activity).ok_or(ReadyError::WrongAction)?;
        Ok(multiple_attack_penalty(MapInput {
            on_own_turn: false,
            prior_attack_trait_checks_this_turn: 0,
            current_weapon_agile: matches!(agility, Agility::Agile),
            readied_attack_count: Some(ready.frozen_attack_count),
        }))
    }

    /// Runs every matching ready and every caller-supplied Reactive Strike, in initiative order.
    ///
    /// `reactors` is not stored. An id with no entry does not take a Reactive Strike.
    /// Each reaction finishes, including its damage, before the next one starts.
    /// A readied move takes `path` and a fresh diagonal counter. `field` supplies
    /// squares, sizes, cover, the grid, terrain, and occupants. Reactive Strike
    /// measures reach at `from`, the square the mover is leaving. A ready whose
    /// trigger is "moves within reach" still compares `to`.
    ///
    /// Player Core, "Ready". Player Core, "Reactive Strike". Player Core, "Range and Reach".
    #[expect(
        clippy::too_many_arguments,
        reason = "the ready design names the actor, the observed action, its cost, both squares, the readied path, the caller-supplied field, the reactors, detections, and the dice"
    )]
    pub fn resolve_reactions(
        &mut self,
        actor: CombatantId,
        action: ObservedAction,
        cost: ActionCost,
        from: Square,
        to: Square,
        path: &[Square],
        field: &ReactionField<'_>,
        reactors: &[ReactiveStrikeInput<'_>],
        detections: &Detections,
        dice: &mut impl Dice,
    ) -> Vec<ResolvedReaction> {
        let mut plan = Vec::new();
        for id in self.order.clone() {
            let subject_matches = self
                .stored_ready(id)
                .ok()
                .flatten()
                .is_some_and(|ready| ready.trigger.subject() == actor);
            if subject_matches {
                plan.push(ReactionPlan::Ready(id));
            }
            for (index, input) in reactors.iter().enumerate() {
                if input.id == id {
                    plan.push(ReactionPlan::Reactive(index));
                }
            }
        }
        let mut resolved = Vec::new();
        for step in plan {
            match step {
                ReactionPlan::Ready(id) => {
                    let result =
                        self.fire_ready(id, actor, action, from, to, path, field, detections, dice);
                    resolved.push(ResolvedReaction::Ready { id, result });
                }
                ReactionPlan::Reactive(index) => {
                    let Some(input) = reactors.get(index) else {
                        continue;
                    };
                    let id = input.id;
                    let outcome = self.reactive_strike(
                        id,
                        actor,
                        cost,
                        ActionIdentity::Other,
                        observed_move(action),
                        input.availability,
                        input.square,
                        from,
                        input.reactor_size,
                        input.actor_size,
                        &input.profile,
                        input.modifiers,
                        detections,
                        dice,
                    );
                    resolved.push(ResolvedReaction::ReactiveStrike {
                        id,
                        result: outcome,
                    });
                }
            }
        }
        resolved
    }

    /// Moves `id` along `path` on their own turn.
    ///
    /// Consciousness is checked before the path. A creature a reaction left Dying or
    /// Dead is [`EncounterError::NotConscious`] even when the path would also fail.
    /// The encounter does not store the square. `field` is the caller's grid.
    ///
    /// Player Core, "Stride". Player Core, "Getting Knocked Out".
    pub fn stride(
        &mut self,
        id: CombatantId,
        field: &ReactionField<'_>,
        path: &[Square],
        counter: DiagonalCounter,
    ) -> Result<MoveOutcome, EncounterError> {
        self.move_on_turn(id, field, path, counter, OwnMove::Stride)
    }

    /// Steps `id` one square on their own turn.
    ///
    /// Consciousness is checked before the path. A creature a reaction left Dying or
    /// Dead is [`EncounterError::NotConscious`] even when the path would also fail.
    /// The encounter does not store the square. `field` is the caller's grid.
    ///
    /// Player Core, "Step". Player Core, "Getting Knocked Out".
    pub fn step(
        &mut self,
        id: CombatantId,
        field: &ReactionField<'_>,
        path: &[Square],
        counter: DiagonalCounter,
    ) -> Result<MoveOutcome, EncounterError> {
        self.move_on_turn(id, field, path, counter, OwnMove::Step)
    }

    /// Drops prone in the caller's square after checking the creature is conscious.
    ///
    /// The positioning function reports the cost. This does not spend it.
    ///
    /// Player Core, "Drop Prone". Player Core, "Getting Knocked Out".
    pub fn drop_prone(
        &self,
        id: CombatantId,
        posture: Posture,
        square: Square,
        mobility: Mobility,
        benefit: Option<CoverBenefit>,
    ) -> Result<PostureChange, EncounterError> {
        self.require_conscious(id)?;
        positioning_drop_prone(posture, square, mobility, benefit)
            .map_err(EncounterError::Positioning)
    }

    /// Stands in the caller's square after checking the creature is conscious.
    ///
    /// The positioning function reports the cost. This does not spend it. An occupant
    /// does not block the stand.
    ///
    /// Player Core, "Stand". Player Core, "Getting Knocked Out".
    pub fn stand(
        &self,
        id: CombatantId,
        posture: Posture,
        square: Square,
        mobility: Mobility,
        benefit: Option<CoverBenefit>,
        occupant: OccupantPresence,
    ) -> Result<PostureChange, EncounterError> {
        self.require_conscious(id)?;
        positioning_stand(posture, square, mobility, benefit, occupant)
            .map_err(EncounterError::Positioning)
    }

    /// Rejects a stored ready before any die, in the fire order.
    ///
    /// `offered` is the action being checked. A stored Strike checked as a Stride is
    /// [`ReadyError::WrongAction`] before [`ReadyError::Delayed`].
    ///
    /// Player Core, "Ready".
    #[cfg(test)]
    #[expect(
        clippy::too_many_arguments,
        reason = "the fire order names the readier, the subject, the observed action, both squares, the path, and the offered kind"
    )]
    pub(crate) fn check_ready_fire(
        &self,
        readier: CombatantId,
        subject: CombatantId,
        observed: ObservedAction,
        from: Square,
        to: Square,
        path: &[Square],
        field: &ReactionField<'_>,
        offered: ActivityKind,
    ) -> Result<(), ReadyError> {
        self.fire_checks(readier, subject, observed, from, to, path, field, offered)?;
        Ok(())
    }

    /// Stores a ready on `id`, including a creature who is already Delayed.
    ///
    /// This is not a branch inside [`Self::ready`] or [`Self::resolve_reactions`].
    #[cfg(test)]
    pub(crate) fn set_ready_for_test(&mut self, id: CombatantId, ready: StoredReady) {
        if let Ok(index) = self.member_index(id)
            && let Some(member) = self.members.get_mut(index)
        {
            member.ready = Some(ready);
        }
    }

    pub(super) fn delayed_target(&self, id: CombatantId) -> Result<TargetDelay, EncounterError> {
        if self.slot_of(id)? == TurnSlot::Delayed {
            Ok(TargetDelay::Delayed)
        } else {
            Ok(TargetDelay::NotDelayed)
        }
    }

    fn insert_after(&mut self, anchor: CombatantId, ids: &[CombatantId]) {
        self.order.retain(|existing| !ids.contains(existing));
        let insert_at = match self.order.iter().position(|existing| *existing == anchor) {
            Some(index) => index.saturating_add(1),
            None => self.order.len(),
        };
        for (offset, id) in ids.iter().copied().enumerate() {
            let at = insert_at.saturating_add(offset);
            if at > self.order.len() {
                self.order.push(id);
            } else {
                self.order.insert(at, id);
            }
        }
    }

    fn open_return(&mut self, id: CombatantId) -> Result<(), EncounterError> {
        let index = self.member_index(id)?;
        let member = self
            .members
            .get_mut(index)
            .ok_or(EncounterError::UnknownCombatant)?;
        member.participant.state.actions_remaining = 3;
        member.slot = TurnSlot::Returned;
        self.open = Some(id);
        self.cursor = self
            .order
            .iter()
            .position(|existing| *existing == id)
            .ok_or(EncounterError::UnknownCombatant)?;
        Ok(())
    }

    pub(super) fn place_delayed_knockout(
        &mut self,
        target: CombatantId,
        attacker: CombatantId,
        delay: TargetDelay,
    ) {
        if delay != TargetDelay::Delayed {
            return;
        }
        let pending = self
            .state(target)
            .ok()
            .and_then(|state| state.initiative_move);
        if pending != Some(InitiativeMove::Pending) {
            return;
        }
        self.order.retain(|existing| *existing != target);
        match self.order.iter().position(|existing| *existing == attacker) {
            Some(anchor) => self.order.insert(anchor, target),
            None => self.order.push(target),
        }
        if let Ok(index) = self.member_index(target)
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

    fn move_on_turn(
        &mut self,
        id: CombatantId,
        field: &ReactionField<'_>,
        path: &[Square],
        counter: DiagonalCounter,
        kind: OwnMove,
    ) -> Result<MoveOutcome, EncounterError> {
        self.require_active_alive(id)?;
        let state = self.state(id)?;
        if state.actions_remaining < 1 {
            return Err(EncounterError::InsufficientActions);
        }
        let outcome = self
            .measure_move(id, field, path, counter, kind)
            .map_err(EncounterError::Grid)?;
        self.spend_actions(id, 1)?;
        Ok(outcome)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "firing a ready names the readier, the subject, the observed action, both squares, the path, the field, detections, and the dice"
    )]
    fn fire_ready(
        &mut self,
        readier: CombatantId,
        subject: CombatantId,
        observed: ObservedAction,
        from: Square,
        to: Square,
        path: &[Square],
        field: &ReactionField<'_>,
        detections: &Detections,
        dice: &mut impl Dice,
    ) -> Result<ReadySuccess, ReadyError> {
        let stored_kind = self
            .stored_ready(readier)
            .map_err(ready_from_encounter)?
            .ok_or(ReadyError::NotReadied)?
            .activity
            .kind()
            .ok_or(ReadyError::WrongAction)?;
        let checked = self.fire_checks(
            readier,
            subject,
            observed,
            from,
            to,
            path,
            field,
            stored_kind,
        )?;
        match stored_kind {
            ActivityKind::Strike => self.fire_strike(readier, checked, detections, dice),
            ActivityKind::Stride | ActivityKind::Step => self.fire_move(readier, checked),
            ActivityKind::Dismiss => self.fire_dismiss(readier, checked),
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the fire order names the readier, the subject, the observed action, both squares, the path, the field, and the offered kind"
    )]
    fn fire_checks(
        &self,
        readier: CombatantId,
        subject: CombatantId,
        observed: ObservedAction,
        from: Square,
        to: Square,
        path: &[Square],
        field: &ReactionField<'_>,
        offered: ActivityKind,
    ) -> Result<CheckedFire, ReadyError> {
        let _readier = self.member_index(readier).map_err(ready_from_encounter)?;
        let _subject = self.member_index(subject).map_err(ready_from_encounter)?;
        let ready = self
            .stored_ready(readier)
            .map_err(ready_from_encounter)?
            .ok_or(ReadyError::NotReadied)?;
        let stored_kind = ready.activity.kind().ok_or(ReadyError::WrongAction)?;
        if stored_kind != offered {
            return Err(ReadyError::WrongAction);
        }
        let state = self.state(readier).map_err(ready_from_encounter)?;
        if state.vital != Vital::Alive {
            return Err(ReadyError::NotConscious);
        }
        if self.slot_of(readier).map_err(ready_from_encounter)? == TurnSlot::Delayed {
            return Err(ReadyError::Delayed);
        }
        let readier_place = find_place(field.places, readier)?;
        let subject_place = find_place(field.places, subject)?;
        let reach = self
            .creature(readier)
            .map_err(ready_from_encounter)?
            .strike
            .reach_feet
            .get();
        let matches = entered_reach(
            ready.trigger,
            subject,
            observed,
            from,
            to,
            readier_place.square,
            readier_place.size,
            subject_place.size,
            reach,
        )?;
        if !matches {
            return Err(ReadyError::TriggerMismatch);
        }
        let mut strike = None;
        if let ReadyActivity::Strike {
            target,
            range,
            profile,
        } = ready.activity
        {
            let range_penalty = strike_range_penalty(
                range,
                readier_place.square,
                to,
                readier_place.size,
                subject_place.size,
                reach,
            )?;
            let target_state = self.state(target).map_err(ready_from_encounter)?;
            if target_state.vital == Vital::Dead {
                return Err(ReadyError::TargetDead);
            }
            let map = multiple_attack_penalty(MapInput {
                on_own_turn: false,
                prior_attack_trait_checks_this_turn: 0,
                current_weapon_agile: matches!(profile.agility, Agility::Agile),
                readied_attack_count: Some(ready.frozen_attack_count),
            });
            strike_fits(profile.attack_bonus, map, range_penalty)?;
            strike = Some(CheckedStrike {
                target,
                profile,
                map,
                range_penalty,
            });
        }
        let mut movement = None;
        if matches!(ready.activity, ReadyActivity::Stride | ReadyActivity::Step) {
            let kind = if matches!(ready.activity, ReadyActivity::Step) {
                OwnMove::Step
            } else {
                OwnMove::Stride
            };
            movement = Some(
                self.measure_move(readier, field, path, DiagonalCounter::fresh(), kind)
                    .map_err(ReadyError::Grid)?,
            );
        }
        let mut dismissed = None;
        if matches!(ready.activity, ReadyActivity::Dismiss) {
            dismissed = Some(dismiss(readier_place.cover).map_err(ReadyError::Positioning)?);
        }
        self.require_reaction(readier)
            .map_err(ready_from_encounter)?;
        Ok(CheckedFire {
            strike,
            movement,
            dismissed,
        })
    }

    fn fire_strike(
        &mut self,
        readier: CombatantId,
        checked: CheckedFire,
        detections: &Detections,
        dice: &mut impl Dice,
    ) -> Result<ReadySuccess, ReadyError> {
        let Some(strike) = checked.strike else {
            return Err(ReadyError::WrongAction);
        };
        match flat_gate(resolve_targeting(
            detections,
            readier,
            strike.target,
            false,
            dice,
        )) {
            FlatGate::CannotTarget => return Err(ReadyError::CannotTarget),
            FlatGate::Lost(lost) => {
                self.spend_reaction(readier).map_err(ready_from_encounter)?;
                self.clear_ready(readier)?;
                return Ok(ReadySuccess::Lost(lost));
            }
            FlatGate::Roll => {}
        }
        let natural = dice.d20();
        let total = attack_check_total(
            natural,
            term(strike.profile.attack_bonus)?,
            term(strike.map)?,
            term(0)?,
            term(strike.range_penalty)?,
            &[],
        )
        .map_err(ReadyError::AttackOverflow)?;
        let armor_class = self
            .creature(strike.target)
            .map_err(ready_from_encounter)?
            .armor_class;
        let degree = degree_of_success(natural, total, armor_class);
        let damage = if misses(degree) {
            self.spend_reaction(readier).map_err(ready_from_encounter)?;
            0
        } else {
            let rolled = roll_strike_damage(dice, &strike.profile).map_err(ready_from_encounter)?;
            let dealt = damage_dealt(rolled, &[], degree == DegreeOfSuccess::CriticalSuccess)
                .map_err(ReadyError::DamageOverflow)?;
            self.commit_ready_blow(readier, strike.target, dealt, degree)?;
            dealt
        };
        self.clear_ready(readier)?;
        Ok(ReadySuccess::Strike(ReadyStrike {
            map: strike.map,
            range_penalty: strike.range_penalty,
            total,
            degree,
            damage,
        }))
    }

    fn commit_ready_blow(
        &mut self,
        readier: CombatantId,
        target: CombatantId,
        damage: i32,
        degree: DegreeOfSuccess,
    ) -> Result<(), ReadyError> {
        let index = self.member_index(target).map_err(ready_from_encounter)?;
        let snapshot = self
            .members
            .get(index)
            .ok_or(ReadyError::UnknownCombatant)?
            .participant
            .state
            .clone();
        let target_delay = self.delayed_target(target).map_err(ready_from_encounter)?;
        if let Err(error) = self.apply_damage(target, damage, degree) {
            return Err(ready_from_encounter(error));
        }
        if let Err(error) = self.spend_reaction(readier) {
            if let Some(member) = self.members.get_mut(index) {
                member.participant.state = snapshot;
            }
            return Err(ready_from_encounter(error));
        }
        self.place_delayed_knockout(target, readier, target_delay);
        Ok(())
    }

    fn fire_move(
        &mut self,
        readier: CombatantId,
        checked: CheckedFire,
    ) -> Result<ReadySuccess, ReadyError> {
        let Some(outcome) = checked.movement else {
            return Err(ReadyError::WrongAction);
        };
        self.spend_reaction(readier).map_err(ready_from_encounter)?;
        self.clear_ready(readier)?;
        Ok(ReadySuccess::Move {
            end: outcome.end,
            distance_feet: outcome.cost_feet,
        })
    }

    fn fire_dismiss(
        &mut self,
        readier: CombatantId,
        checked: CheckedFire,
    ) -> Result<ReadySuccess, ReadyError> {
        let Some(benefit) = checked.dismissed else {
            return Err(ReadyError::WrongAction);
        };
        self.spend_reaction(readier).map_err(ready_from_encounter)?;
        self.clear_ready(readier)?;
        Ok(ReadySuccess::Dismiss { benefit })
    }

    fn measure_move(
        &self,
        id: CombatantId,
        field: &ReactionField<'_>,
        path: &[Square],
        counter: DiagonalCounter,
        kind: OwnMove,
    ) -> Result<MoveOutcome, GridError> {
        let index = self.member_index(id).map_err(|_| GridError::OffGrid)?;
        let member = self.members.get(index).ok_or(GridError::OffGrid)?;
        let place = field
            .places
            .iter()
            .find(|place| place.id == id)
            .ok_or(GridError::OffGrid)?;
        let input = MoveInput {
            start: place.square,
            path,
            speed_feet: member.participant.creature.speed_feet,
            size: place.size,
            immobilized: false,
            faction: faction_of(member.participant.side),
            bounds: field.bounds,
            terrain: field.terrain,
            occupants: field.occupants,
            counter,
        };
        match kind {
            OwnMove::Stride => grid_stride(input),
            OwnMove::Step => grid_step(input),
        }
    }

    fn clear_ready(&mut self, id: CombatantId) -> Result<(), ReadyError> {
        let index = self.member_index(id).map_err(ready_from_encounter)?;
        let member = self
            .members
            .get_mut(index)
            .ok_or(ReadyError::UnknownCombatant)?;
        member.ready = None;
        Ok(())
    }
}

struct CheckedStrike {
    target: CombatantId,
    profile: StrikeProfile,
    map: i32,
    range_penalty: i32,
}

struct CheckedFire {
    strike: Option<CheckedStrike>,
    movement: Option<MoveOutcome>,
    dismissed: Option<Option<CoverBenefit>>,
}

enum ReactionPlan {
    Ready(CombatantId),
    Reactive(usize),
}

enum OwnMove {
    Stride,
    Step,
}

fn find_place(places: &[ReactionPlace], id: CombatantId) -> Result<&ReactionPlace, ReadyError> {
    places
        .iter()
        .find(|place| place.id == id)
        .ok_or(ReadyError::Grid(GridError::OffGrid))
}

fn duplicate_ids(ids: &[CombatantId]) -> bool {
    let mut seen = Vec::new();
    for id in ids {
        if seen.contains(id) {
            return true;
        }
        seen.push(*id);
    }
    false
}

fn observed_move(action: ObservedAction) -> Option<MoveAction> {
    match action {
        ObservedAction::Stride => Some(MoveAction::Stride),
        ObservedAction::Step => Some(MoveAction::Step),
        ObservedAction::DropProne => Some(MoveAction::DropProne),
        ObservedAction::Strike | ObservedAction::Stand | ObservedAction::Other => None,
    }
}

fn faction_of(side: Side) -> FactionId {
    match side {
        Side::Pc => FactionId(1),
        Side::Enemy => FactionId(2),
    }
}

fn term(value: i32) -> Result<AttackTotalTerm, ReadyError> {
    AttackTotalTerm::new(value).map_err(|_| ReadyError::AttackOverflow(ModifierError::Overflow))
}

fn strike_fits(attack_bonus: i32, map: i32, range_penalty: i32) -> Result<(), ReadyError> {
    let _bonus = term(attack_bonus)?;
    let _map = term(map)?;
    let _range = term(range_penalty)?;
    let sum = i64::from(attack_bonus) + i64::from(map) + i64::from(range_penalty) + 20;
    i32::try_from(sum)
        .map(|_| ())
        .map_err(|_| ReadyError::AttackOverflow(ModifierError::Overflow))
}

fn strike_range_penalty(
    range: StrikeRange,
    readier: Square,
    target: Square,
    readier_size: Size,
    target_size: Size,
    reach_feet: u32,
) -> Result<i32, ReadyError> {
    match range {
        StrikeRange::Melee => {
            let inside = in_reach(readier, target, readier_size, target_size, reach_feet)
                .map_err(ReadyError::Grid)?;
            if inside {
                Ok(0)
            } else {
                Err(ReadyError::OutOfReach)
            }
        }
        StrikeRange::Ranged { increment_feet } => {
            let feet =
                distance(readier, target, readier_size, target_size).map_err(ReadyError::Grid)?;
            let feet = i32::try_from(feet).map_err(|_| ReadyError::Grid(GridError::Overflow))?;
            let band = range_band(feet, increment_feet).map_err(ReadyError::Grid)?;
            Ok(band.penalty)
        }
    }
}

/// Maps an encounter error onto Ready. Each variant keeps its own cause.
pub(super) fn ready_from_encounter(error: EncounterError) -> ReadyError {
    match error {
        EncounterError::UnknownCombatant => ReadyError::UnknownCombatant,
        EncounterError::TurnNotOpen => ReadyError::TurnNotOpen,
        EncounterError::NotYourTurn => ReadyError::NotYourTurn,
        EncounterError::NotConscious => ReadyError::NotConscious,
        EncounterError::InsufficientActions => ReadyError::InsufficientActions,
        EncounterError::RoundOverflow => ReadyError::RoundOverflow,
        EncounterError::Delayed => ReadyError::Delayed,
        EncounterError::NoReaction => ReadyError::NoReaction,
        EncounterError::HitPoints(error) => ReadyError::HitPoints(error),
        EncounterError::DamageOverflow(error) => ReadyError::DamageOverflow(error),
        EncounterError::AttackOverflow(error) => ReadyError::AttackOverflow(error),
        EncounterError::Grid(error) => ReadyError::Grid(error),
        EncounterError::TargetDead => ReadyError::TargetDead,
        EncounterError::Positioning(error) => ReadyError::Positioning(error),
        EncounterError::DuplicateCombatant => {
            ReadyError::Encounter(ReadyEncounter::DuplicateCombatant)
        }
        EncounterError::EmptyRoster => ReadyError::Encounter(ReadyEncounter::EmptyRoster),
        EncounterError::PerceptionOutOfRange => {
            ReadyError::Encounter(ReadyEncounter::PerceptionOutOfRange)
        }
        EncounterError::TurnInProgress => ReadyError::Encounter(ReadyEncounter::TurnInProgress),
        EncounterError::InvalidActionCost => {
            ReadyError::Encounter(ReadyEncounter::InvalidActionCost)
        }
        EncounterError::NoShield => ReadyError::Encounter(ReadyEncounter::NoShield),
        EncounterError::NotRaised => ReadyError::Encounter(ReadyEncounter::NotRaised),
        EncounterError::Broken => ReadyError::Encounter(ReadyEncounter::Broken),
        EncounterError::Destroyed => ReadyError::Encounter(ReadyEncounter::Destroyed),
        EncounterError::NegativeAmount => ReadyError::Encounter(ReadyEncounter::NegativeAmount),
        EncounterError::TargetNotConscious => {
            ReadyError::Encounter(ReadyEncounter::TargetNotConscious)
        }
        EncounterError::SelfTarget => ReadyError::Encounter(ReadyEncounter::SelfTarget),
        EncounterError::NoLivingCombatant => {
            ReadyError::Encounter(ReadyEncounter::NoLivingCombatant)
        }
        EncounterError::AttackCountOverflow => {
            ReadyError::Encounter(ReadyEncounter::AttackCountOverflow)
        }
        EncounterError::NotPrepared => ReadyError::Encounter(ReadyEncounter::NotPrepared),
        EncounterError::NoAidModifier => ReadyError::Encounter(ReadyEncounter::NoAidModifier),
        EncounterError::NotAnAlly => ReadyError::Encounter(ReadyEncounter::NotAnAlly),
        EncounterError::NotThePreparedAlly => {
            ReadyError::Encounter(ReadyEncounter::NotThePreparedAlly)
        }
        EncounterError::AllyDead => ReadyError::Encounter(ReadyEncounter::AllyDead),
        EncounterError::AidOverflow(error) => {
            ReadyError::Encounter(ReadyEncounter::AidOverflow(error))
        }
        EncounterError::NoCreature => ReadyError::Encounter(ReadyEncounter::NoCreature),
        EncounterError::NotDelayed => ReadyError::Encounter(ReadyEncounter::NotDelayed),
        EncounterError::TriggerNotMet => ReadyError::Encounter(ReadyEncounter::TriggerNotMet),
        EncounterError::NoSavingThrows => ReadyError::Encounter(ReadyEncounter::NoSavingThrows),
        EncounterError::SaveOverflow(error) => {
            ReadyError::Encounter(ReadyEncounter::SaveOverflow(error))
        }
        EncounterError::NoSkills => ReadyError::Encounter(ReadyEncounter::NoSkills),
        EncounterError::SkillOverflow(error) => {
            ReadyError::Encounter(ReadyEncounter::SkillOverflow(error))
        }
        EncounterError::CannotTarget => ReadyError::CannotTarget,
    }
}
