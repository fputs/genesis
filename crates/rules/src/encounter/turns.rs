//! Initiative, the round cursor, and the three-action turn.
//!
//! Player Core, "Step 1: Roll Initiative". Player Core, "Step 2: Act".
//! Player Core, "Step 3: End Your Turn".

use crate::aid::AidPreparation;
use crate::combatant::{CombatantId, CombatantState, Creature, Vital};
use crate::dice::Dice;
use crate::shield::lower_raised_shield;

use super::{Encounter, EncounterError, Member, Participant, Side, TurnSlot, TurnState};

/// Rolls initiative and builds the order.
///
/// Dice are drawn in ascending [`CombatantId`] order, including a dead combatant. The total is
/// the face plus the baked Perception modifier. A natural 20 does not add 10. Sort is higher
/// total, then [`Side::Enemy`] before [`Side::Pc`], then the lower id. Same-Side ties use that
/// lower id (DEV-05). A duplicate id, an empty roster, or a Perception modifier that cannot be
/// added to 20 is rejected before any die is drawn. Every participant's Aid preparation is set
/// to [`AidPreparation::NotPrepared`].
///
/// Player Core, "Step 1: Roll Initiative".
pub fn begin_encounter(
    participants: Vec<Participant>,
    dice: &mut impl Dice,
) -> Result<Encounter, EncounterError> {
    if participants.is_empty() {
        return Err(EncounterError::EmptyRoster);
    }
    let mut ids: Vec<CombatantId> = participants
        .iter()
        .map(|participant| participant.id)
        .collect();
    ids.sort();
    if ids
        .windows(2)
        .any(|pair| matches!(pair, [left, right] if left == right))
    {
        return Err(EncounterError::DuplicateCombatant);
    }
    for participant in &participants {
        if participant.creature.perception.checked_add(20).is_none() {
            return Err(EncounterError::PerceptionOutOfRange);
        }
    }

    let mut members: Vec<Member> = participants
        .into_iter()
        .map(|mut participant| {
            let alive = participant.state.vital == Vital::Alive;
            participant.state.actions_remaining = 0;
            participant.state.reaction_ready = alive;
            participant.state.aid_preparation = AidPreparation::NotPrepared;
            Member {
                participant,
                attack_count: 0,
                initiative: 0,
                slot: TurnSlot::Normal,
                ready: None,
            }
        })
        .collect();

    let mut roll_order: Vec<usize> = (0..members.len()).collect();
    roll_order.sort_by(|&left, &right| {
        let left_id = members.get(left).map(|member| member.participant.id);
        let right_id = members.get(right).map(|member| member.participant.id);
        left_id.cmp(&right_id)
    });
    for index in roll_order {
        let face = dice.d20();
        let member = members
            .get_mut(index)
            .ok_or(EncounterError::UnknownCombatant)?;
        member.initiative = member
            .participant
            .creature
            .perception
            .checked_add(i32::from(face))
            .ok_or(EncounterError::PerceptionOutOfRange)?;
    }

    let mut ranked: Vec<(CombatantId, i32, Side)> = members
        .iter()
        .map(|member| {
            (
                member.participant.id,
                member.initiative,
                member.participant.side,
            )
        })
        .collect();
    ranked.sort_by(|left, right| {
        right
            .1
            .cmp(&left.1)
            .then_with(|| side_rank(left.2).cmp(&side_rank(right.2)))
            .then_with(|| left.0.cmp(&right.0))
    });
    let order = ranked.into_iter().map(|(id, _total, _side)| id).collect();

    Ok(Encounter {
        members,
        order,
        cursor: 0,
        round: 1,
        open: None,
        last_ended: None,
        turn_state: TurnState::Continuing,
    })
}

pub(super) fn side_rank(side: Side) -> u8 {
    match side {
        Side::Enemy => 0,
        Side::Pc => 1,
    }
}

impl Encounter {
    /// The round, starting at 1.
    ///
    /// Player Core, "Step 2: Act".
    pub fn round(&self) -> u32 {
        self.round
    }

    /// Initiative order. Dead combatants stay in it.
    ///
    /// Player Core, "Step 1: Roll Initiative".
    pub fn order(&self) -> &[CombatantId] {
        &self.order
    }

    /// The combatant whose turn is open, if any.
    ///
    /// Player Core, "Step 2: Act".
    pub fn turn_open(&self) -> Option<CombatantId> {
        self.open
    }

    /// Borrows the current state for `id`.
    ///
    /// Player Core, Encounter Mode.
    pub fn state(&self, id: CombatantId) -> Result<&CombatantState, EncounterError> {
        let index = self.member_index(id)?;
        self.members
            .get(index)
            .map(|member| &member.participant.state)
            .ok_or(EncounterError::UnknownCombatant)
    }

    /// Borrows the creature for `id`.
    ///
    /// Player Core, Hit Points.
    pub fn creature(&self, id: CombatantId) -> Result<&Creature, EncounterError> {
        let index = self.member_index(id)?;
        self.members
            .get(index)
            .map(|member| &member.participant.creature)
            .ok_or(EncounterError::UnknownCombatant)
    }

    /// Opens the next turn that is not [`Vital::Dead`].
    ///
    /// An Alive creature gets 3 actions and one reaction after the empty start step, and that
    /// creature's attack count becomes 0. Any other vital still opens a turn with 0 actions and
    /// no reaction. A successful start sets that creature's Aid preparation to
    /// [`AidPreparation::NotPrepared`] after the round increment has succeeded (DEV-50). A
    /// second start while a turn is open is [`EncounterError::TurnInProgress`].
    ///
    /// Player Core, "Step 1: Start Your Turn". Player Core, "Step 2: Act". Player Core, Aid.
    pub fn start_turn(&mut self) -> Result<CombatantId, EncounterError> {
        if self.open.is_some() {
            return Err(EncounterError::TurnInProgress);
        }
        if self.no_living() {
            return Err(EncounterError::NoLivingCombatant);
        }
        if self.turn_state == TurnState::PendingWrap {
            self.round = self
                .round
                .checked_add(1)
                .ok_or(EncounterError::RoundOverflow)?;
            self.turn_state = TurnState::Continuing;
            self.cursor = 0;
        }
        let len = self.order.len();
        let mut index = self.cursor;
        let mut round = self.round;
        let mut steps = 0usize;
        let mut clear_delayed = Vec::new();
        let id = loop {
            if steps > len {
                return Err(EncounterError::NoLivingCombatant);
            }
            steps += 1;
            let id = *self
                .order
                .get(index)
                .ok_or(EncounterError::NoLivingCombatant)?;
            if !self.is_dead(id) {
                break id;
            }
            clear_delayed.push(id);
            if index + 1 >= len {
                round = round.checked_add(1).ok_or(EncounterError::RoundOverflow)?;
                index = 0;
            } else {
                index += 1;
            }
        };
        for skipped in clear_delayed {
            self.clear_delay_mark(skipped);
        }
        self.cursor = index;
        self.round = round;
        self.open_at(id);
        Ok(id)
    }

    /// Closes the open turn and advances once.
    ///
    /// The empty end step runs, actions drop to 0, and an unspent reaction stays available.
    /// Aid preparation is left as it was. Moving past the last combatant wraps the cursor and
    /// adds 1 to the round. [`EncounterError::RoundOverflow`] returns before that advance.
    /// A second call is [`EncounterError::TurnNotOpen`] and does not advance again.
    ///
    /// Player Core, "Step 3: End Your Turn". Player Core, Aid.
    pub fn end_turn(&mut self) -> Result<(), EncounterError> {
        let Some(id) = self.open else {
            return Err(EncounterError::TurnNotOpen);
        };
        let advance = self.plan_advance(TurnAdvance::End)?;
        self.end_of_turn_step(id);
        self.close_actions(id);
        self.open = None;
        self.last_ended = Some(id);
        self.apply_advance(advance);
        Ok(())
    }

    /// Spends 1, 2, or 3 actions at once on the open Alive turn.
    ///
    /// Player Core, Activities. Player Core, "Step 2: Act".
    pub fn spend_actions(&mut self, id: CombatantId, n: u8) -> Result<(), EncounterError> {
        self.require_active_alive(id)?;
        if n == 0 || n > 3 {
            return Err(EncounterError::InvalidActionCost);
        }
        let index = self.member_index(id)?;
        let member = self
            .members
            .get_mut(index)
            .ok_or(EncounterError::UnknownCombatant)?;
        let left = member.participant.state.actions_remaining;
        if left < n {
            return Err(EncounterError::InsufficientActions);
        }
        member.participant.state.actions_remaining = left - n;
        Ok(())
    }

    /// Spends an untriggered free action. The action count does not change.
    ///
    /// Player Core, Activities. Player Core, "Step 2: Act".
    pub fn spend_free_action(&mut self, id: CombatantId) -> Result<(), EncounterError> {
        self.require_active_alive(id)
    }

    /// Assigns the round. The increment itself has no test-only branch.
    #[cfg(test)]
    pub(crate) fn set_round_for_test(&mut self, round: u32) {
        self.round = round;
    }

    /// The initiative total rolled for `id`. Delay does not change it.
    ///
    /// Player Core, "Step 1: Roll Initiative". Player Core, Delay.
    pub fn initiative_total(&self, id: CombatantId) -> Result<i32, EncounterError> {
        let index = self.member_index(id)?;
        self.members
            .get(index)
            .map(|member| member.initiative)
            .ok_or(EncounterError::UnknownCombatant)
    }

    /// The slot `id` occupies. This does not roll.
    ///
    /// Player Core, Delay.
    pub fn turn_slot(&self, id: CombatantId) -> Result<TurnSlot, EncounterError> {
        self.slot_of(id)
    }

    /// The creature `start_turn` would open. This does not mutate.
    ///
    /// An open turn, including a returned one, is [`EncounterError::TurnInProgress`].
    /// An empty roster is [`EncounterError::NoCreature`]. A pending Delay wrap returns
    /// the creature at the top and leaves the round unchanged. A wrap past the maximum
    /// round is [`EncounterError::RoundOverflow`].
    ///
    /// Player Core, "Step 2: Act". Player Core, Delay.
    pub fn next_start(&self) -> Result<CombatantId, EncounterError> {
        if self.members.is_empty() {
            return Err(EncounterError::NoCreature);
        }
        if self.open.is_some() {
            return Err(EncounterError::TurnInProgress);
        }
        if self.no_living() {
            return Err(EncounterError::NoLivingCombatant);
        }
        if self.turn_state == TurnState::PendingWrap {
            if self.round.checked_add(1).is_none() {
                return Err(EncounterError::RoundOverflow);
            }
            return self.first_living(0);
        }
        self.hypothetical_start()
    }

    pub(super) fn member_index(&self, id: CombatantId) -> Result<usize, EncounterError> {
        self.members
            .iter()
            .position(|member| member.participant.id == id)
            .ok_or(EncounterError::UnknownCombatant)
    }

    fn is_dead(&self, id: CombatantId) -> bool {
        self.state(id)
            .map(|state| state.vital == Vital::Dead)
            .unwrap_or(false)
    }

    fn no_living(&self) -> bool {
        self.order.is_empty() || self.order.iter().copied().all(|id| self.is_dead(id))
    }

    fn open_at(&mut self, id: CombatantId) {
        let skip_hook = matches!(self.slot_of(id), Ok(TurnSlot::SkipStartHook));
        if !skip_hook {
            self.start_of_turn_step(id);
        }
        self.open = Some(id);
        let alive = self
            .state(id)
            .map(|state| state.vital == Vital::Alive)
            .unwrap_or(false);
        if let Ok(index) = self.member_index(id)
            && let Some(member) = self.members.get_mut(index)
        {
            // DEV-50: this creature's preparation ends when their turn starts.
            member.participant.state.aid_preparation = AidPreparation::NotPrepared;
            if !skip_hook {
                member.ready = None;
            }
            if alive {
                member.participant.state.actions_remaining = 3;
                member.participant.state.reaction_ready = true;
                if !skip_hook {
                    member.attack_count = 0;
                }
            } else {
                member.participant.state.actions_remaining = 0;
                member.participant.state.reaction_ready = false;
            }
            member.slot = TurnSlot::Normal;
        }
    }

    pub(super) fn require_active_alive(&self, id: CombatantId) -> Result<(), EncounterError> {
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
        Ok(())
    }

    fn start_of_turn_step(&mut self, id: CombatantId) {
        if self.is_dead(id) {
            return;
        }
        if let Ok(index) = self.member_index(id)
            && let Some(member) = self.members.get_mut(index)
        {
            lower_raised_shield(&mut member.participant.state);
        }
    }

    pub(super) fn end_of_turn_step(&mut self, _id: CombatantId) {}

    pub(super) fn require_conscious(&self, id: CombatantId) -> Result<(), EncounterError> {
        let state = self.state(id)?;
        if state.vital != Vital::Alive {
            return Err(EncounterError::NotConscious);
        }
        Ok(())
    }

    pub(super) fn slot_of(&self, id: CombatantId) -> Result<TurnSlot, EncounterError> {
        let index = self.member_index(id)?;
        self.members
            .get(index)
            .map(|member| member.slot)
            .ok_or(EncounterError::UnknownCombatant)
    }

    pub(super) fn side_of(&self, id: CombatantId) -> Result<Side, EncounterError> {
        let index = self.member_index(id)?;
        self.members
            .get(index)
            .map(|member| member.participant.side)
            .ok_or(EncounterError::UnknownCombatant)
    }

    pub(super) fn set_slot(
        &mut self,
        id: CombatantId,
        slot: TurnSlot,
    ) -> Result<(), EncounterError> {
        let index = self.member_index(id)?;
        let member = self
            .members
            .get_mut(index)
            .ok_or(EncounterError::UnknownCombatant)?;
        member.slot = slot;
        Ok(())
    }

    fn any_delayed(&self) -> bool {
        self.members
            .iter()
            .any(|member| member.slot == TurnSlot::Delayed)
    }

    pub(super) fn end_would_overflow(&self) -> bool {
        matches!(
            self.plan_advance(TurnAdvance::End),
            Err(EncounterError::RoundOverflow)
        )
    }

    pub(super) fn plan_advance(&self, kind: TurnAdvance) -> Result<AdvancePlan, EncounterError> {
        let len = self.order.len();
        let next = self.cursor.saturating_add(1);
        let wrapping = next >= len;
        match kind {
            TurnAdvance::Delay => Ok(AdvancePlan {
                cursor: if wrapping { 0 } else { next },
                round: None,
                turn_state: if wrapping {
                    TurnState::PendingWrap
                } else {
                    TurnState::Continuing
                },
            }),
            TurnAdvance::End if wrapping && self.any_delayed() => Ok(AdvancePlan {
                cursor: 0,
                round: None,
                turn_state: TurnState::PendingWrap,
            }),
            TurnAdvance::End if wrapping => {
                let round = self
                    .round
                    .checked_add(1)
                    .ok_or(EncounterError::RoundOverflow)?;
                Ok(AdvancePlan {
                    cursor: 0,
                    round: Some(round),
                    turn_state: TurnState::Continuing,
                })
            }
            TurnAdvance::End => Ok(AdvancePlan {
                cursor: next,
                round: None,
                turn_state: TurnState::Continuing,
            }),
        }
    }

    pub(super) fn apply_advance(&mut self, plan: AdvancePlan) {
        if let Some(round) = plan.round {
            self.round = round;
        }
        self.cursor = plan.cursor;
        self.turn_state = plan.turn_state;
    }

    pub(super) fn close_actions(&mut self, id: CombatantId) {
        if let Ok(index) = self.member_index(id)
            && let Some(member) = self.members.get_mut(index)
        {
            member.participant.state.actions_remaining = 0;
            if matches!(member.slot, TurnSlot::Returned | TurnSlot::SkipStartHook) {
                member.slot = TurnSlot::Normal;
            }
        }
    }

    fn clear_delay_mark(&mut self, id: CombatantId) {
        if let Ok(index) = self.member_index(id)
            && let Some(member) = self.members.get_mut(index)
            && member.slot == TurnSlot::Delayed
        {
            member.slot = TurnSlot::Normal;
        }
    }

    fn first_living(&self, start: usize) -> Result<CombatantId, EncounterError> {
        let len = self.order.len();
        if len == 0 {
            return Err(EncounterError::NoLivingCombatant);
        }
        let mut index = start;
        for _ in 0..len {
            let id = *self
                .order
                .get(index)
                .ok_or(EncounterError::NoLivingCombatant)?;
            if !self.is_dead(id) {
                return Ok(id);
            }
            index = if index + 1 >= len { 0 } else { index + 1 };
        }
        Err(EncounterError::NoLivingCombatant)
    }

    fn hypothetical_start(&self) -> Result<CombatantId, EncounterError> {
        let len = self.order.len();
        let mut index = self.cursor;
        let mut round = self.round;
        for _ in 0..=len {
            let id = *self
                .order
                .get(index)
                .ok_or(EncounterError::NoLivingCombatant)?;
            if !self.is_dead(id) {
                return Ok(id);
            }
            if index + 1 >= len {
                round = round.checked_add(1).ok_or(EncounterError::RoundOverflow)?;
                index = 0;
            } else {
                index += 1;
            }
        }
        Err(EncounterError::NoLivingCombatant)
    }
}

pub(super) enum TurnAdvance {
    End,
    Delay,
}

pub(super) struct AdvancePlan {
    cursor: usize,
    round: Option<u32>,
    turn_state: TurnState,
}
