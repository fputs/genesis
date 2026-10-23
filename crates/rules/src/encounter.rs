//! Initiative, the round, the three-action turn, Strike, Aid, Reactive Strike, Delay, and Ready.
//!
//! Player Core, "Step 1: Roll Initiative". Player Core, "Step 2: Act". Player Core, Strike.
//! Player Core, Aid. Player Core, "Reactive Strike". Player Core, Delay. Player Core, "Ready".

use crate::aid::{AidOutcome, AidPreparation, FittingAidModifier, roll_aid};
use crate::check::{
    ArithmeticOverflow, AttackTotalTerm, DegreeOfSuccess, MapInput, attack_check_total,
    degree_of_success, multiple_attack_penalty,
};
use crate::combatant::{
    CombatantId, CombatantState, Creature, DieCount, HpError, Vital, damage_dealt,
};
use crate::dice::{Dice, Faces};
use crate::dying::{InitiativeMove, apply_resolved_blow};
use crate::grid::{
    DiagonalCounter, FactionId, GridError, MoveInput, MoveOutcome, Size, Square, distance,
    in_reach, range_band, step as grid_step, stride as grid_stride,
};
use crate::modifiers::{Modifier, ModifierError, Statistic};
use crate::positioning::{
    ActionCost, CoverBenefit, Mobility, MoveAction, OccupantPresence, PositioningError, Posture,
    PostureChange, dismiss, drop_prone as positioning_drop_prone, stand as positioning_stand,
};
use crate::reactive::{
    ActionIdentity, ReactiveError, ReactiveOutcome, ReactiveStrike, TriggerClass,
    attack_bonus_fits, disruption, not_a_trigger, outside_reach, reactive_attack_total,
};
use crate::ready::{
    ActivityKind, ObservedAction, ReactionField, ReactionPlace, ReactiveStrikeInput, ReadyActivity,
    ReadyEncounter, ReadyError, ReadyStrike, ReadySuccess, ReadyTrigger, ResolvedReaction,
    StoredReady, StrikeRange, entered_reach, single_action, stored_agility,
};
use crate::shield::{
    AttackBlowError, PlannedBlow, ShieldBlock, ShieldError, lower_raised_shield, plan_attack_blow,
    raise_shield as raise_combatant_shield, reduce_shield, validate_raise,
};

/// Which team a combatant is on. Phase 1 has no other team type (DEV-26).
///
/// Player Core, "Step 1: Roll Initiative".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// A player character. Player Core, "Step 1: Roll Initiative".
    Pc,
    /// An enemy. Player Core, "Step 1: Roll Initiative".
    Enemy,
}

/// Whether the weapon used for this Strike is agile.
///
/// Player Core, "Multiple Attack Penalty".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Agility {
    /// The second and later penalties are −4 and −8. Player Core, "Multiple Attack Penalty".
    Agile,
    /// The second and later penalties are −5 and −10. Player Core, "Multiple Attack Penalty".
    NotAgile,
}

/// One combatant handed to [`begin_encounter`].
///
/// Player Core, Encounter Mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Participant {
    /// Identity in this encounter. Player Core, Encounter Mode.
    pub id: CombatantId,
    /// Team used for initiative ties. Player Core, "Step 1: Roll Initiative".
    pub side: Side,
    /// Baked statistics. Player Core, Hit Points.
    pub creature: Creature,
    /// Current Hit Points, actions, and vital. Player Core, Encounter Mode.
    pub state: CombatantState,
}

/// Numbers for one Strike. The encounter does not store squares or reach.
///
/// Player Core, Strike.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StrikeProfile {
    /// Added to the d20. Player Core, Strike.
    pub attack_bonus: i32,
    /// Agile or not, for this Strike only. Player Core, "Multiple Attack Penalty".
    pub agility: Agility,
    /// How many damage dice to roll. Player Core, "Damage Rolls".
    pub damage_dice: DieCount,
    /// Faces on each damage die. Player Core, "Damage Rolls".
    pub damage_faces: Faces,
    /// Flat damage added to the dice. Player Core, "Damage Rolls".
    pub damage_bonus: i32,
}

/// Why an encounter action was rejected.
///
/// Player Core, Encounter Mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncounterError {
    /// Two participants share a [`CombatantId`]. Player Core, Encounter Mode.
    DuplicateCombatant,
    /// The roster had nobody to roll. Player Core, "Step 1: Roll Initiative".
    EmptyRoster,
    /// Perception cannot be added to a d20 of 20. Player Core, "Step 1: Roll Initiative".
    PerceptionOutOfRange,
    /// That id is not in the encounter. Player Core, Encounter Mode.
    UnknownCombatant,
    /// A turn is open, so another start was rejected. Player Core, "Step 1: Start Your Turn".
    TurnInProgress,
    /// No turn is open. Player Core, "Step 3: End Your Turn".
    TurnNotOpen,
    /// The actor is not the active combatant. Player Core, "Step 2: Act".
    NotYourTurn,
    /// The combatant cannot pay that many actions. Player Core, Activities.
    InsufficientActions,
    /// The cost was not 1, 2, or 3. Player Core, Activities.
    InvalidActionCost,
    /// The reaction was not available. Player Core, "Reactions in Encounters".
    NoReaction,
    /// The combatant is not Alive. Player Core, "Step 2: Act".
    NotConscious,
    /// The target is not Alive, so a requested Shield Block does not apply.
    ///
    /// This is the target's error. [`EncounterError::NotConscious`] stays the attacker's.
    ///
    /// Player Core, "Shield Block".
    TargetNotConscious,
    /// The creature has no shield. Player Core, "Raise a Shield".
    NoShield,
    /// The shield is not raised. Player Core, "Shield Block".
    NotRaised,
    /// The shield is broken, or a block was requested against a destroyed shield.
    ///
    /// Player Core, Broken.
    Broken,
    /// The shield is destroyed, so it cannot be raised. Player Core, "Shield Statistics".
    Destroyed,
    /// The blow amount is negative. Player Core, "Shield Block".
    NegativeAmount,
    /// The target is dead. Player Core, Strike.
    TargetDead,
    /// The target is the attacker. Player Core, Strike.
    SelfTarget,
    /// Every combatant is dead. Player Core, "Step 1: Start Your Turn".
    NoLivingCombatant,
    /// The attack count cannot increase past `u8`. Player Core, "Multiple Attack Penalty".
    AttackCountOverflow,
    /// The round cannot increase past `u32`. Player Core, "Step 2: Act".
    RoundOverflow,
    /// The attack total did not fit. Player Core, Penalties.
    AttackOverflow(ModifierError),
    /// The damage total did not fit. Player Core, "Damage Rolls".
    DamageOverflow(ModifierError),
    /// Applying Hit Points failed. Player Core, Hit Points.
    HitPoints(HpError),
    /// The aider has not prepared for an ally. Player Core, Aid.
    NotPrepared,
    /// The aider has no Aid modifier. Player Core, Aid.
    NoAidModifier,
    /// The other creature is not on the aider's side. Player Core, Aid.
    NotAnAlly,
    /// Preparation names a different ally. Player Core, Aid.
    NotThePreparedAlly,
    /// The Aid modifier plus 20 does not fit in `i32`. Player Core, Aid.
    AidOverflow(ArithmeticOverflow),
    /// The roster has no creature. Player Core, Encounter Mode.
    NoCreature,
    /// Return was asked of a creature who is not delayed. Player Core, Delay.
    NotDelayed,
    /// Delay or Return was not at its trigger. Player Core, Delay.
    TriggerNotMet,
    /// The creature has Delayed and cannot use a reaction yet.
    ///
    /// Checked after [`EncounterError::NotConscious`] and before any action or die.
    ///
    /// Player Core, Delay.
    Delayed,
    /// A begun Stride or Step was rejected by the grid. Player Core, "Grid Movement".
    Grid(GridError),
    /// Stand or Drop Prone was rejected after the creature was conscious.
    ///
    /// Player Core, "Stand". Player Core, "Drop Prone".
    Positioning(PositioningError),
}

/// Where the round sits between turns, and whether a blow's target had Delayed.
///
/// One enum replaces a pending-wrap flag and a was-delayed flag.
///
/// Player Core, Delay.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TurnState {
    /// The round is not waiting to wrap. A blow's target was not Delayed.
    Continuing,
    /// The last turn ended while someone is Delayed. The round has not advanced.
    PendingWrap,
    /// The blow was planned while the target's slot was Delayed.
    DelayedTarget,
}

/// Where a creature sits in the round.
///
/// There is no delayed flag. The slot is the only record.
///
/// Player Core, Delay. Player Core, "Step 1: Start Your Turn".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnSlot {
    /// An ordinary turn. `start_turn` sets this. Player Core, "Step 1: Start Your Turn".
    Normal,
    /// Out of the order until they return or the slot comes up again. Player Core, Delay.
    Delayed,
    /// The open turn a return just produced. Delay on it is [`EncounterError::TriggerNotMet`].
    ///
    /// Player Core, Delay.
    Returned,
    /// A later creature in the same return. The next start grants actions and skips the start step.
    ///
    /// Player Core, Delay.
    SkipStartHook,
}

/// One resolved Strike.
///
/// Player Core, Strike.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrikeOutcome {
    /// The d20. Player Core, Strike.
    pub natural: u8,
    /// Natural roll plus attack bonus, multiple attack penalty, and caller modifiers.
    ///
    /// Player Core, Strike.
    pub total: i32,
    /// Degree after a natural 20 or natural 1. Player Core, "Step 4: Degree of Success".
    pub degree: DegreeOfSuccess,
    /// Multiple attack penalty used on this Strike. Player Core, "Multiple Attack Penalty".
    pub map: i32,
    /// Damage dealt, or 0 on a miss. Player Core, "Damage Rolls".
    pub damage: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Member {
    participant: Participant,
    attack_count: u8,
    initiative: i32,
    slot: TurnSlot,
    ready: Option<StoredReady>,
}

/// One fight: initiative order, the round, the open turn, and Strike.
///
/// Player Core, Encounter Mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Encounter {
    members: Vec<Member>,
    order: Vec<CombatantId>,
    cursor: usize,
    round: u32,
    open: Option<CombatantId>,
    /// The creature whose turn just ended. Delay clears it. Return requires it.
    ///
    /// Player Core, Delay. Player Core, "Step 3: End Your Turn".
    last_ended: Option<CombatantId>,
    /// Whether the next start wraps the round. Delay does not advance it.
    ///
    /// Player Core, Delay.
    turn_state: TurnState,
}

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

fn side_rank(side: Side) -> u8 {
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
    /// side. A missing Aid modifier is next. Nothing is spent or stored on those errors. Only
    /// then does this call [`Self::spend_actions`] and record the ally. A second call spends
    /// another action and replaces the named ally.
    ///
    /// Player Core, Aid. Player Core, "Step 2: Act".
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
        let ally_side = self
            .members
            .get(ally_index)
            .ok_or(EncounterError::UnknownCombatant)?
            .participant
            .side;
        if aider_side != ally_side {
            return Err(EncounterError::NotAnAlly);
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
    /// (DEV-50). An error does not clear it and draws no die. The ally's vital is not read.
    /// Actions, Hit Points, and the attack count stay as they were.
    ///
    /// Player Core, Aid. Player Core, "Step 2: Act". Player Core, Unconscious.
    /// Player Core, "Reactions in Encounters".
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
        let ally_side = self
            .members
            .get(ally_index)
            .ok_or(EncounterError::UnknownCombatant)?
            .participant
            .side;
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

    /// Strikes `target` with `profile` on the active creature's turn.
    ///
    /// A miss spends the action, records damage 0, and does not roll a damage die.
    /// [`HpError::NotAHit`] is not returned from this path. Success and critical success roll
    /// the damage dice once; a critical success doubles that total inside [`damage_dealt`].
    /// A hit passes that total and `block` through the encounter block once. It does not also
    /// call [`apply_resolved_blow`]. [`ShieldBlock::Requested`] checks the shield before any
    /// die is drawn and spends no action when that check fails. A miss does not resolve a
    /// block. The degree uses the target's baked Armor Class. A requested reaction is spent
    /// only through [`Self::spend_reaction`].
    ///
    /// Player Core, Strike. Player Core, "Shield Block".
    pub fn strike(
        &mut self,
        attacker: CombatantId,
        target: CombatantId,
        profile: &StrikeProfile,
        modifiers: &[Modifier],
        block: ShieldBlock,
        dice: &mut impl Dice,
    ) -> Result<StrikeOutcome, EncounterError> {
        self.prepare_strike(attacker, target)?;
        let delayed_target = self.delayed_target(target)?;
        if let ShieldBlock::Requested = block {
            self.reject_requested_block(target)?;
            self.block_is_ready(target)?;
        }
        let map = self.attack_trait_penalty(attacker, profile.agility)?;
        let attack_term = bounded_term(profile.attack_bonus)?;
        let map_term = bounded_term(map)?;
        let zero = bounded_term(0)?;
        let natural = dice.d20();
        let attack_modifiers = attack_modifiers(modifiers);
        let total = attack_check_total(
            natural,
            attack_term,
            map_term,
            zero,
            zero,
            &attack_modifiers,
        )
        .map_err(EncounterError::AttackOverflow)?;
        let armor_class = self.creature(target)?.armor_class;
        let degree = degree_of_success(natural, total, armor_class);
        let damage = if misses(degree) {
            0
        } else {
            let rolled = roll_strike_damage(dice, profile)?;
            let dealt = damage_dealt(
                rolled,
                modifiers,
                degree == DegreeOfSuccess::CriticalSuccess,
            )
            .map_err(EncounterError::DamageOverflow)?;
            match block {
                ShieldBlock::Declined => self.apply_damage(target, dealt, degree)?,
                ShieldBlock::Requested => self.shield_block(target, dealt, degree)?,
            }
            self.place_delayed_knockout(target, attacker, delayed_target);
            dealt
        };
        self.finish_strike(attacker)?;
        Ok(StrikeOutcome {
            natural,
            total,
            degree,
            map,
            damage,
        })
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
        reason = "the reactive-strike design names the reactor, actor, trigger, squares, sizes, strike, modifiers, and dice as separate arguments"
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
            let rolled = roll_strike_damage(dice, profile).map_err(as_damage_overflow)?;
            damage_dealt(
                rolled,
                modifiers,
                degree == DegreeOfSuccess::CriticalSuccess,
            )
            .map_err(ReactiveError::DamageOverflow)?
        };
        let actor_turn = self
            .delayed_target(actor)
            .map_err(|_| ReactiveError::UnknownCombatant)?;
        if !misses(degree) {
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
            self.place_delayed_knockout(actor, reactor, actor_turn);
        } else {
            self.spend_reaction(reactor).map_err(from_spend)?;
        }
        Ok(ReactiveOutcome {
            strike: StrikeOutcome {
                natural,
                total,
                degree,
                map: 0,
                damage,
            },
            disruption: disruption(degree, cost),
        })
    }

    /// Assigns the round. The increment itself has no test-only branch.
    #[cfg(test)]
    pub(crate) fn set_round_for_test(&mut self, round: u32) {
        self.round = round;
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

    /// Assigns vital. Hit Points, actions, and the reaction stay as they are.
    #[cfg(test)]
    pub(crate) fn set_vital_for_test(&mut self, id: CombatantId, vital: Vital) {
        if let Ok(index) = self.member_index(id)
            && let Some(member) = self.members.get_mut(index)
        {
            member.participant.state.vital = vital;
        }
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

    fn member_index(&self, id: CombatantId) -> Result<usize, EncounterError> {
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

    fn require_active_alive(&self, id: CombatantId) -> Result<(), EncounterError> {
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

    fn apply_damage(
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

    fn end_of_turn_step(&mut self, _id: CombatantId) {}

    fn block_is_ready(&self, target: CombatantId) -> Result<(), EncounterError> {
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
        reason = "the ready design names the actor, the observed action, its cost, both squares, the readied path, the caller-supplied field, the reactors, and the dice"
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
                    let result = self.fire_ready(id, actor, action, from, to, path, field, dice);
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
                        dice,
                    );
                    resolved.push(ResolvedReaction::ReactiveStrike {
                        id,
                        result: outcome.map(|_| ()),
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

    fn require_conscious(&self, id: CombatantId) -> Result<(), EncounterError> {
        let state = self.state(id)?;
        if state.vital != Vital::Alive {
            return Err(EncounterError::NotConscious);
        }
        Ok(())
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

    fn require_reaction(&self, id: CombatantId) -> Result<(), EncounterError> {
        self.reaction_gate(id, ReactionGate::Spend)
    }

    fn reject_delayed(&self, id: CombatantId) -> Result<(), EncounterError> {
        self.reaction_gate(id, ReactionGate::Delay)
    }

    fn delayed_target(&self, id: CombatantId) -> Result<TurnState, EncounterError> {
        if self.slot_of(id)? == TurnSlot::Delayed {
            Ok(TurnState::DelayedTarget)
        } else {
            Ok(TurnState::Continuing)
        }
    }

    fn reject_requested_block(&self, target: CombatantId) -> Result<(), EncounterError> {
        let state = self.state(target)?;
        if state.vital != Vital::Alive {
            return Err(EncounterError::TargetNotConscious);
        }
        self.reject_delayed(target)
    }

    fn slot_of(&self, id: CombatantId) -> Result<TurnSlot, EncounterError> {
        let index = self.member_index(id)?;
        self.members
            .get(index)
            .map(|member| member.slot)
            .ok_or(EncounterError::UnknownCombatant)
    }

    fn side_of(&self, id: CombatantId) -> Result<Side, EncounterError> {
        let index = self.member_index(id)?;
        self.members
            .get(index)
            .map(|member| member.participant.side)
            .ok_or(EncounterError::UnknownCombatant)
    }

    fn set_slot(&mut self, id: CombatantId, slot: TurnSlot) -> Result<(), EncounterError> {
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

    fn end_would_overflow(&self) -> bool {
        matches!(
            self.plan_advance(TurnAdvance::End),
            Err(EncounterError::RoundOverflow)
        )
    }

    fn plan_advance(&self, kind: TurnAdvance) -> Result<AdvancePlan, EncounterError> {
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

    fn apply_advance(&mut self, plan: AdvancePlan) {
        if let Some(round) = plan.round {
            self.round = round;
        }
        self.cursor = plan.cursor;
        self.turn_state = plan.turn_state;
    }

    fn close_actions(&mut self, id: CombatantId) {
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

    fn place_delayed_knockout(
        &mut self,
        target: CombatantId,
        attacker: CombatantId,
        turn_state: TurnState,
    ) {
        if turn_state != TurnState::DelayedTarget {
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
        reason = "firing a ready names the readier, the subject, the observed action, both squares, the path, the field, and the dice"
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
            ActivityKind::Strike => self.fire_strike(readier, checked, dice),
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
        dice: &mut impl Dice,
    ) -> Result<ReadySuccess, ReadyError> {
        let Some(strike) = checked.strike else {
            return Err(ReadyError::WrongAction);
        };
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
        let target_turn = self.delayed_target(target).map_err(ready_from_encounter)?;
        if let Err(error) = self.apply_damage(target, damage, degree) {
            return Err(ready_from_encounter(error));
        }
        if let Err(error) = self.spend_reaction(readier) {
            if let Some(member) = self.members.get_mut(index) {
                member.participant.state = snapshot;
            }
            return Err(ready_from_encounter(error));
        }
        self.place_delayed_knockout(target, readier, target_turn);
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

enum TurnAdvance {
    End,
    Delay,
}

struct AdvancePlan {
    cursor: usize,
    round: Option<u32>,
    turn_state: TurnState,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReactionGate {
    Delay,
    Spend,
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
fn ready_from_encounter(error: EncounterError) -> ReadyError {
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
        EncounterError::AidOverflow(error) => {
            ReadyError::Encounter(ReadyEncounter::AidOverflow(error))
        }
        EncounterError::NoCreature => ReadyError::Encounter(ReadyEncounter::NoCreature),
        EncounterError::NotDelayed => ReadyError::Encounter(ReadyEncounter::NotDelayed),
        EncounterError::TriggerNotMet => ReadyError::Encounter(ReadyEncounter::TriggerNotMet),
    }
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
fn from_spend(error: EncounterError) -> ReactiveError {
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
        EncounterError::AidOverflow(_) => ReactiveError::NoReaction,
        EncounterError::NoCreature => ReactiveError::NoReaction,
        EncounterError::NotDelayed => ReactiveError::NoReaction,
        EncounterError::TriggerNotMet => ReactiveError::NoReaction,
        EncounterError::Delayed => ReactiveError::Delayed,
        EncounterError::Grid(_) => ReactiveError::NoReaction,
        EncounterError::Positioning(_) => ReactiveError::NoReaction,
    }
}

/// Maps a damage-roll failure. Each variant has its own arm so a new one does not
/// fall into [`ModifierError::Overflow`].
#[expect(
    clippy::match_same_arms,
    reason = "each EncounterError variant is named so a new one cannot collapse into DamageOverflow"
)]
fn as_damage_overflow(error: EncounterError) -> ReactiveError {
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
        EncounterError::AidOverflow(_) => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::NoCreature => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::NotDelayed => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::TriggerNotMet => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::Delayed => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::Grid(_) => ReactiveError::DamageOverflow(ModifierError::Overflow),
        EncounterError::Positioning(_) => ReactiveError::DamageOverflow(ModifierError::Overflow),
    }
}

fn misses(degree: DegreeOfSuccess) -> bool {
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

fn roll_strike_damage(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aid::{AidCircumstance, AidPreparation, AidRank};
    use crate::check::{ArithmeticOverflow, DegreeOfSuccess};
    use crate::combatant::{
        BuildParts, CharacterBuild, CombatantId, DieCount, DyingValue, HpError, MeleeStrike,
        ReachFeet, ShieldTemplate, Vital, derive, new_combatant_state,
    };
    use crate::dice::{Faces, ScriptedDice};
    use crate::dying::{InitiativeMove, apply_resolved_blow, take_initiative_move};
    use crate::grid::{Size, Square};
    use crate::modifiers::{BonusType, Modifier, ModifierError, PenaltyType, SourceId, Statistic};
    use crate::positioning::{ActionCost, ActionTrait, MoveAction};
    use crate::reactive::{
        ActionIdentity, Disruption, ReactiveError, ReactiveOutcome, ReactiveStrike,
    };
    use crate::shield::{self, ShieldBlock};

    fn id(value: u32) -> CombatantId {
        CombatantId(value)
    }

    fn faces(count: u8) -> Faces {
        Faces::new(count).expect("faces")
    }

    fn creature(
        hp: i32,
        ac: i32,
        perception: i32,
        dying: bool,
        shield: Option<ShieldTemplate>,
    ) -> Creature {
        let parts = BuildParts {
            hit_points: hp,
            armor_class: ac,
            perception,
            speed_feet: 25,
            strike: MeleeStrike {
                attack_bonus: 0,
                damage_dice: DieCount::new(1).expect("dice"),
                damage_faces: faces(8),
                damage_bonus: 0,
                reach_feet: ReachFeet::new(5).expect("reach"),
                agile: false,
            },
            shield,
            stealth_dc: None,
            aid_modifier: None,
            aid_rank: None,
            uses_dying_rules: Some(dying),
        };
        derive(&CharacterBuild::try_new(parts).expect("build"))
    }

    fn participant(value: u32, side: Side, creature: Creature) -> Participant {
        let combatant = id(value);
        let state = new_combatant_state(combatant, &creature);
        Participant {
            id: combatant,
            side,
            creature,
            state,
        }
    }

    fn combatant(
        value: u32,
        side: Side,
        perception: i32,
        hp: i32,
        ac: i32,
        dying: bool,
    ) -> Participant {
        participant(value, side, creature(hp, ac, perception, dying, None))
    }

    fn pc(value: u32, perception: i32, hp: i32, ac: i32) -> Participant {
        combatant(value, Side::Pc, perception, hp, ac, false)
    }

    fn enemy(value: u32, perception: i32, hp: i32, ac: i32) -> Participant {
        combatant(value, Side::Enemy, perception, hp, ac, false)
    }

    fn dead(mut participant: Participant) -> Participant {
        participant.state.hit_points = 0;
        participant.state.vital = Vital::Dead;
        participant
    }

    fn with_temp(mut participant: Participant, temp: i32) -> Participant {
        participant.state.temp_hit_points = temp;
        participant
    }

    fn profile(attack: i32, agility: Agility, dice: u8, face: u8, bonus: i32) -> StrikeProfile {
        StrikeProfile {
            attack_bonus: attack,
            agility,
            damage_dice: DieCount::new(i32::from(dice)).expect("dice"),
            damage_faces: faces(face),
            damage_bonus: bonus,
        }
    }

    fn begin(participants: Vec<Participant>, rolls: &[u8]) -> (Encounter, ScriptedDice) {
        let mut dice = ScriptedDice::new(rolls.iter().copied());
        let encounter = begin_encounter(participants, &mut dice).expect("encounter");
        (encounter, dice)
    }

    fn actions(encounter: &Encounter, value: u32) -> u8 {
        encounter.state(id(value)).expect("state").actions_remaining
    }

    fn reaction(encounter: &Encounter, value: u32) -> bool {
        encounter.state(id(value)).expect("state").reaction_ready
    }

    fn hit_points(encounter: &Encounter, value: u32) -> i32 {
        encounter.state(id(value)).expect("state").hit_points
    }

    fn temp_hp(encounter: &Encounter, value: u32) -> i32 {
        encounter.state(id(value)).expect("state").temp_hit_points
    }

    fn wounded(encounter: &Encounter, value: u32) -> i32 {
        encounter.state(id(value)).expect("state").wounded
    }

    fn vital(encounter: &Encounter, value: u32) -> Vital {
        encounter.state(id(value)).expect("state").vital
    }

    fn initiative(encounter: &Encounter, value: u32) -> i32 {
        encounter.initiative_total(id(value)).expect("initiative")
    }

    fn penalty(ty: PenaltyType, value: i32, source: u32, statistic: Statistic) -> Modifier {
        Modifier::penalty(ty, value, SourceId(source), source, None, statistic).expect("penalty")
    }

    fn bonus(ty: BonusType, value: i32, source: u32, statistic: Statistic) -> Modifier {
        Modifier::bonus(ty, value, SourceId(source), source, None, statistic).expect("bonus")
    }

    fn order_of(ids: &[u32]) -> Vec<CombatantId> {
        ids.iter().copied().map(id).collect()
    }

    /// Player Core, "Step 1: Roll Initiative".
    ///
    /// Arithmetic: 10 + 5 = 15, 14 + 0 = 14, then 12 against Armor Class 11, then 3 + 4 = 7.
    #[test]
    fn the_same_encounter_twice_matches() {
        let run = |script: [u8; 4]| {
            let (mut encounter, mut dice) = begin(
                vec![
                    combatant(1, Side::Pc, 5, 20, 11, false),
                    combatant(2, Side::Pc, 0, 20, 11, false),
                ],
                &script,
            );
            assert_eq!(initiative(&encounter, 1), 15);
            assert_eq!(initiative(&encounter, 2), 14);
            let order = encounter.order().to_vec();
            assert_eq!(encounter.start_turn().expect("start"), id(1));
            let outcome = encounter
                .strike(
                    id(1),
                    id(2),
                    &profile(0, Agility::NotAgile, 1, 8, 4),
                    &[],
                    ShieldBlock::Declined,
                    &mut dice,
                )
                .expect("strike");
            (order, outcome.total, outcome.damage, dice.remaining())
        };
        let first = run([10, 14, 12, 3]);
        let second = run([10, 14, 12, 3]);
        assert_eq!(first, second);
        assert_eq!(first.0, order_of(&[1, 2]));
        assert_eq!(first.1, 12);
        assert_eq!(first.2, 7);
        assert_eq!(first.3, 0);
    }

    /// Player Core, "Step 1: Roll Initiative".
    #[test]
    fn perception_is_added_to_the_d20() {
        let (encounter, dice) = begin(vec![pc(1, 5, 20, 10)], &[10]);
        assert_eq!(initiative(&encounter, 1), 15);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, "Step 1: Roll Initiative".
    #[test]
    fn a_natural_20_does_not_add_10() {
        let (encounter, _) = begin(vec![pc(1, 0, 20, 10), pc(2, 5, 20, 10)], &[20, 14]);
        assert_eq!(initiative(&encounter, 1), 20);
        assert_eq!(initiative(&encounter, 2), 19);
        assert_eq!(encounter.order(), order_of(&[1, 2]).as_slice());
    }

    /// Player Core, "Step 1: Roll Initiative".
    #[test]
    fn a_natural_1_does_not_subtract_10() {
        let (encounter, _) = begin(vec![pc(1, 19, 20, 10)], &[1]);
        assert_eq!(initiative(&encounter, 1), 20);
    }

    /// Player Core, "Step 1: Roll Initiative".
    #[test]
    fn a_negative_perception_modifier_is_added() {
        let (encounter, _) = begin(vec![pc(1, -3, 20, 10)], &[8]);
        assert_eq!(initiative(&encounter, 1), 5);
    }

    /// Player Core, "Step 1: Roll Initiative".
    #[test]
    fn roster_order_does_not_choose_the_die() {
        let (encounter, _) = begin(vec![pc(5, 0, 20, 10), pc(1, 5, 20, 10)], &[10, 14]);
        assert_eq!(initiative(&encounter, 1), 15);
        assert_eq!(initiative(&encounter, 5), 14);
    }

    /// Player Core, "Step 1: Roll Initiative".
    #[test]
    fn a_duplicate_id_rolls_nothing() {
        let mut dice = ScriptedDice::new([8]);
        let error = begin_encounter(vec![pc(1, 0, 20, 10), pc(1, 0, 20, 10)], &mut dice)
            .expect_err("duplicate");
        assert_eq!(error, EncounterError::DuplicateCombatant);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Step 1: Roll Initiative".
    #[test]
    fn an_empty_roster_rolls_nothing() {
        let mut dice = ScriptedDice::new([8]);
        let error = begin_encounter(vec![], &mut dice).expect_err("empty");
        assert_eq!(error, EncounterError::EmptyRoster);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Step 1: Roll Initiative".
    #[test]
    fn perception_2147483627_and_a_natural_20_fit() {
        let (encounter, dice) = begin(vec![pc(1, 2_147_483_627, 20, 10)], &[20]);
        assert_eq!(initiative(&encounter, 1), 2_147_483_647);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, "Step 1: Roll Initiative".
    #[test]
    fn perception_2147483628_rolls_nothing() {
        let mut dice = ScriptedDice::new([20]);
        let error =
            begin_encounter(vec![pc(1, 2_147_483_628, 20, 10)], &mut dice).expect_err("range");
        assert_eq!(error, EncounterError::PerceptionOutOfRange);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Step 1: Roll Initiative".
    #[test]
    fn perception_at_i32min_is_added() {
        let (encounter, dice) = begin(vec![pc(1, i32::MIN, 20, 10)], &[1]);
        assert_eq!(initiative(&encounter, 1), -2_147_483_647);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, "Step 1: Roll Initiative".
    #[test]
    fn a_higher_pc_total_acts_before_a_lower_enemy_total() {
        let (encounter, _) = begin(vec![pc(1, 8, 20, 10), enemy(2, 5, 20, 10)], &[10, 10]);
        assert_eq!(initiative(&encounter, 1), 18);
        assert_eq!(initiative(&encounter, 2), 15);
        assert_eq!(encounter.order(), order_of(&[1, 2]).as_slice());
    }

    /// Player Core, "Step 1: Roll Initiative".
    ///
    /// DEV-05 substitute: rule, tied PCs choose who goes first and the GM orders tied foes
    /// (Player Core pg. 435, Rules 2423); why not, there are no players and no GM; substitute,
    /// the lower CombatantId acts earlier on a same-Side tie; intent, the same roster and dice
    /// always produce the same order. A PC–enemy tie is the book, not that substitute: enemy
    /// CombatantId(9) acts before PC CombatantId(1) even though 9 is the higher id.
    #[test]
    fn an_enemy_acts_before_a_tied_pc() {
        let (encounter, _) = begin(vec![pc(1, 0, 20, 10), enemy(9, 0, 20, 10)], &[10, 10]);
        assert_eq!(initiative(&encounter, 1), 10);
        assert_eq!(initiative(&encounter, 9), 10);
        assert_eq!(encounter.order(), order_of(&[9, 1]).as_slice());
    }

    /// Player Core, "Step 1: Roll Initiative".
    ///
    /// DEV-05 substitute: same-Side ties, including two PCs, act by the lower CombatantId.
    #[test]
    fn two_tied_pcs_act_by_the_lower_id() {
        let (encounter, _) = begin(vec![pc(4, 0, 20, 10), pc(2, 0, 20, 10)], &[8, 8]);
        assert_eq!(initiative(&encounter, 4), 8);
        assert_eq!(initiative(&encounter, 2), 8);
        assert_eq!(encounter.order(), order_of(&[2, 4]).as_slice());
    }

    /// Player Core, "Step 1: Roll Initiative".
    ///
    /// DEV-05 substitute: same-Side ties, including two enemies, act by the lower CombatantId.
    #[test]
    fn two_tied_enemies_act_by_the_lower_id() {
        let (encounter, _) = begin(vec![enemy(8, 0, 20, 10), enemy(3, 0, 20, 10)], &[7, 7]);
        assert_eq!(initiative(&encounter, 8), 7);
        assert_eq!(initiative(&encounter, 3), 7);
        assert_eq!(encounter.order(), order_of(&[3, 8]).as_slice());
    }

    /// Player Core, "Step 1: Roll Initiative".
    ///
    /// DEV-05 substitute: enemies share a Side, so the lower id is earlier, and both enemies
    /// are before the tied PC.
    #[test]
    fn a_three_way_tie_puts_both_enemies_first() {
        let (encounter, _) = begin(
            vec![pc(1, 0, 20, 10), enemy(2, 0, 20, 10), enemy(5, 0, 20, 10)],
            &[12, 12, 12],
        );
        assert_eq!(initiative(&encounter, 1), 12);
        assert_eq!(initiative(&encounter, 2), 12);
        assert_eq!(initiative(&encounter, 5), 12);
        assert_eq!(encounter.order(), order_of(&[2, 5, 1]).as_slice());
    }

    /// Player Core, "Step 1: Roll Initiative".
    ///
    /// DEV-05 substitute: each same-Side tie uses the lower CombatantId, after the higher total
    /// and after an enemy-before-PC tie.
    #[test]
    fn six_combatants_all_appear_in_the_order() {
        let (encounter, _) = begin(
            vec![
                pc(0, 0, 20, 10),
                enemy(1, 0, 20, 10),
                pc(2, 0, 20, 10),
                enemy(3, 0, 20, 10),
                pc(4, 0, 20, 10),
                enemy(5, 0, 20, 10),
            ],
            &[10, 10, 12, 12, 8, 8],
        );
        assert_eq!(encounter.order(), order_of(&[3, 2, 1, 0, 5, 4]).as_slice());
    }

    /// Player Core, "Step 1: Start Your Turn".
    #[test]
    fn an_alive_start_rolls_nothing() {
        let (mut encounter, dice) = begin(vec![pc(1, 0, 12, 10)], &[10, 8]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        assert_eq!(dice.remaining(), 1);
        assert_eq!(hit_points(&encounter, 1), 12);
        assert_eq!(wounded(&encounter, 1), 0);
        assert_eq!(actions(&encounter, 1), 3);
    }

    /// Player Core, "Step 1: Start Your Turn".
    #[test]
    fn start_sets_3_actions_and_a_reaction() {
        let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        assert_eq!(initiative(&encounter, 1), 10);
        assert_eq!(encounter.round(), 1);
        assert_eq!(actions(&encounter, 1), 3);
        assert!(reaction(&encounter, 1));
    }

    /// Player Core, "Step 1: Start Your Turn".
    #[test]
    fn a_second_start_does_not_add_actions() {
        let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
        encounter.start_turn().expect("start");
        assert_eq!(
            encounter.start_turn().expect_err("second"),
            EncounterError::TurnInProgress
        );
        assert_eq!(actions(&encounter, 1), 3);
        assert!(reaction(&encounter, 1));
    }

    /// Player Core, "Step 1: Start Your Turn".
    #[test]
    fn the_next_start_sets_3_rather_than_adding_leftovers() {
        let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10), pc(2, 0, 20, 10)], &[10, 8]);
        assert_eq!(encounter.start_turn().expect("first"), id(1));
        encounter.spend_actions(id(1), 1).expect("spend");
        assert_eq!(actions(&encounter, 1), 2);
        encounter.end_turn().expect("end");
        encounter.start_turn().expect("other");
        encounter.end_turn().expect("end other");
        assert_eq!(encounter.start_turn().expect("next"), id(1));
        assert_eq!(actions(&encounter, 1), 3);
    }

    /// Player Core, "Step 1: Start Your Turn". Player Core, "Step 2: Act".
    #[test]
    fn a_spent_reaction_is_ready_again_next_turn() {
        let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
        encounter.start_turn().expect("start");
        encounter.spend_reaction(id(1)).expect("spend");
        encounter.end_turn().expect("end");
        encounter.start_turn().expect("next");
        assert!(reaction(&encounter, 1));
        encounter.spend_reaction(id(1)).expect("first");
        assert_eq!(
            encounter.spend_reaction(id(1)).expect_err("second"),
            EncounterError::NoReaction
        );
    }

    /// Player Core, "Step 1: Start Your Turn". Player Core, "Step 2: Act".
    #[test]
    fn an_unused_reaction_is_still_one_reaction() {
        let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
        encounter.start_turn().expect("start");
        assert!(reaction(&encounter, 1));
        encounter.end_turn().expect("end");
        encounter.start_turn().expect("next");
        encounter.spend_reaction(id(1)).expect("first");
        assert_eq!(
            encounter.spend_reaction(id(1)).expect_err("second"),
            EncounterError::NoReaction
        );
    }

    /// Player Core, "Reactions in Encounters".
    ///
    /// DEV-06 substitute: rule, the GM decides whether reactions work before the first turn
    /// (Player Core pg. 436, Rules 2432); why not, there is no GM; substitute, an Alive
    /// combatant's reaction is available at encounter start; intent, one fixed answer.
    #[test]
    fn a_reaction_before_the_first_turn_spends_once() {
        let (mut encounter, dice) =
            begin(vec![pc(1, 0, 20, 10), dead(pc(2, 0, 20, 10))], &[10, 8, 8]);
        assert_eq!(actions(&encounter, 1), 0);
        assert!(reaction(&encounter, 1));
        assert!(!reaction(&encounter, 2));
        encounter.spend_reaction(id(1)).expect("first");
        assert_eq!(dice.remaining(), 1);
        assert_eq!(
            encounter.spend_reaction(id(1)).expect_err("second"),
            EncounterError::NoReaction
        );
        assert_eq!(actions(&encounter, 1), 0);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Step 2: Act".
    #[test]
    fn a_not_alive_turn_opens_with_no_actions() {
        let (mut encounter, mut dice) = begin(
            vec![
                combatant(1, Side::Pc, 0, 20, 18, false),
                combatant(2, Side::Pc, 0, 9, 11, true),
            ],
            &[10, 8, 11, 8, 4],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 1, 8, 1),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("knockout");
        assert_eq!(hit_points(&encounter, 2), 0);
        assert_ne!(vital(&encounter, 2), Vital::Alive);
        assert_ne!(vital(&encounter, 2), Vital::Dead);
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("next"), id(2));
        assert_eq!(actions(&encounter, 2), 0);
        assert!(!reaction(&encounter, 2));
        assert_eq!(
            encounter.spend_reaction(id(2)).expect_err("reaction"),
            EncounterError::NotConscious
        );
        assert_eq!(
            encounter.spend_actions(id(2), 1).expect_err("action"),
            EncounterError::NotConscious
        );
        assert_eq!(
            encounter.spend_free_action(id(2)).expect_err("free"),
            EncounterError::NotConscious
        );
        assert_eq!(
            encounter
                .strike(
                    id(2),
                    id(1),
                    &profile(0, Agility::NotAgile, 1, 8, 0),
                    &[],
                    ShieldBlock::Declined,
                    &mut dice,
                )
                .expect_err("strike"),
            EncounterError::NotConscious
        );
        assert!(!reaction(&encounter, 2));
        assert_eq!(actions(&encounter, 2), 0);
        assert_eq!(dice.remaining(), 1);
        encounter.end_turn().expect("close");
        assert_eq!(encounter.turn_open(), None);
        assert_eq!(actions(&encounter, 2), 0);
    }

    /// Player Core, "Step 2: Act".
    #[test]
    fn an_active_creature_that_stops_being_alive_cannot_keep_acting() {
        let (mut encounter, mut dice) = begin(
            vec![
                combatant(1, Side::Pc, 0, 20, 18, false),
                combatant(2, Side::Pc, 0, 9, 11, true),
            ],
            &[10, 8, 11, 8, 8],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 1, 8, 1),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("knockout");
        let knocked = vital(&encounter, 2);
        assert_ne!(knocked, Vital::Alive);
        assert_ne!(knocked, Vital::Dead);
        encounter.set_vital_for_test(id(1), knocked);
        assert_eq!(
            encounter.spend_actions(id(1), 1).expect_err("spend"),
            EncounterError::NotConscious
        );
        assert_eq!(actions(&encounter, 1), 2);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
        encounter.end_turn().expect("first end");
        assert_eq!(
            encounter.end_turn().expect_err("second end"),
            EncounterError::TurnNotOpen
        );
        assert_eq!(encounter.round(), 1);
        assert_eq!(encounter.start_turn().expect("next"), id(2));
    }

    /// Player Core, "Reactions in Encounters".
    #[test]
    fn a_dead_combatant_cannot_spend_a_reaction() {
        let (mut encounter, _) = begin(vec![dead(pc(1, 0, 20, 10)), pc(2, 0, 20, 10)], &[10, 8]);
        assert!(reaction(&encounter, 2));
        let dead_reaction = reaction(&encounter, 1);
        assert_eq!(
            encounter.spend_reaction(id(1)).expect_err("dead"),
            EncounterError::NotConscious
        );
        assert!(reaction(&encounter, 2));
        assert_eq!(reaction(&encounter, 1), dead_reaction);
    }

    /// Player Core, "Step 3: End Your Turn".
    #[test]
    fn ending_early_drops_leftover_actions_and_keeps_the_reaction() {
        let (mut encounter, dice) = begin(vec![pc(1, 0, 12, 10)], &[10, 8]);
        encounter.start_turn().expect("start");
        encounter.spend_actions(id(1), 1).expect("spend");
        assert_eq!(actions(&encounter, 1), 2);
        assert!(reaction(&encounter, 1));
        encounter.end_turn().expect("end");
        assert_eq!(actions(&encounter, 1), 0);
        assert!(reaction(&encounter, 1));
        assert_eq!(hit_points(&encounter, 1), 12);
        assert_eq!(temp_hp(&encounter, 1), 0);
        assert_eq!(wounded(&encounter, 1), 0);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Step 3: End Your Turn".
    #[test]
    fn end_before_a_start_returns_an_error() {
        let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10), pc(2, 0, 20, 10)], &[10, 8]);
        assert_eq!(
            encounter.end_turn().expect_err("end"),
            EncounterError::TurnNotOpen
        );
        assert_eq!(actions(&encounter, 1), 0);
        assert_eq!(actions(&encounter, 2), 0);
    }

    /// Player Core, "Step 2: Act".
    ///
    /// Rules 2429 ends the turn after the third action. Engineering handoff: the caller calls
    /// end_turn, nothing further can be spent, and no end-of-turn effect fires before that call.
    #[test]
    fn spending_the_last_action_leaves_the_turn_open() {
        let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10), pc(2, 0, 20, 10)], &[10, 8]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.spend_actions(id(1), 3).expect("spend");
        assert_eq!(encounter.turn_open(), Some(id(1)));
        assert_eq!(actions(&encounter, 1), 0);
        assert!(reaction(&encounter, 1));
        encounter.end_turn().expect("end");
        assert_eq!(
            encounter.end_turn().expect_err("second"),
            EncounterError::TurnNotOpen
        );
        assert_eq!(encounter.round(), 1);
        assert_eq!(encounter.start_turn().expect("next"), id(2));
    }

    /// Player Core, Activities. Player Core, "Step 2: Act".
    #[test]
    fn spending_1_leaves_2() {
        let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
        encounter.start_turn().expect("start");
        encounter.spend_actions(id(1), 1).expect("spend");
        assert_eq!(actions(&encounter, 1), 2);
        assert_eq!(encounter.turn_open(), Some(id(1)));
    }

    /// Player Core, Activities. Player Core, "Step 2: Act".
    #[test]
    fn spending_2_leaves_1_and_rolls_nothing() {
        let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 10)], &[10, 8]);
        encounter.start_turn().expect("start");
        encounter.spend_actions(id(1), 2).expect("spend");
        assert_eq!(actions(&encounter, 1), 1);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Activities. Player Core, "Step 2: Act".
    #[test]
    fn spending_2_with_1_left_fails() {
        let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
        encounter.start_turn().expect("start");
        encounter.spend_actions(id(1), 2).expect("spend");
        assert_eq!(
            encounter.spend_actions(id(1), 2).expect_err("short"),
            EncounterError::InsufficientActions
        );
        assert_eq!(actions(&encounter, 1), 1);
    }

    /// Player Core, Activities. Player Core, "Step 2: Act".
    #[test]
    fn a_cost_of_0_or_4_is_rejected() {
        let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
        encounter.start_turn().expect("start");
        assert_eq!(
            encounter.spend_actions(id(1), 0).expect_err("zero"),
            EncounterError::InvalidActionCost
        );
        assert_eq!(
            encounter.spend_actions(id(1), 4).expect_err("four"),
            EncounterError::InvalidActionCost
        );
        assert_eq!(actions(&encounter, 1), 3);
    }

    /// Player Core, Activities. Player Core, "Step 2: Act".
    #[test]
    fn a_free_action_leaves_the_remaining_action() {
        let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
        encounter.start_turn().expect("start");
        encounter.spend_actions(id(1), 2).expect("spend");
        assert!(reaction(&encounter, 1));
        encounter.spend_free_action(id(1)).expect("free");
        assert_eq!(actions(&encounter, 1), 1);
        assert!(reaction(&encounter, 1));
        assert_eq!(encounter.turn_open(), Some(id(1)));
    }

    /// Player Core, Activities. Player Core, "Step 2: Act".
    #[test]
    fn an_untriggered_free_action_off_turn_fails() {
        let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
        assert_eq!(
            encounter.spend_free_action(id(1)).expect_err("off"),
            EncounterError::TurnNotOpen
        );
    }

    /// Player Core, "Reactions in Encounters".
    #[test]
    fn a_reaction_during_the_turn_leaves_the_actions() {
        let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 10)], &[10, 8]);
        encounter.start_turn().expect("start");
        encounter.spend_reaction(id(1)).expect("reaction");
        assert_eq!(actions(&encounter, 1), 3);
        assert_eq!(encounter.turn_open(), Some(id(1)));
        assert!(!reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Reactions in Encounters".
    #[test]
    fn another_creature_spends_a_reaction_during_the_turn() {
        let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 10), pc(2, 0, 20, 10)], &[14, 10, 8]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.spend_reaction(id(2)).expect("reaction");
        assert_eq!(actions(&encounter, 1), 3);
        assert_eq!(encounter.turn_open(), Some(id(1)));
        assert_eq!(dice.remaining(), 1);
        assert_eq!(
            encounter.spend_reaction(id(2)).expect_err("second"),
            EncounterError::NoReaction
        );
    }

    /// Player Core, "Step 2: Act". Player Core, "Step 1: Start Your Turn".
    #[test]
    fn the_second_round_starts_with_the_first_living_combatant() {
        let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10), pc(2, 0, 20, 10)], &[10, 8]);
        assert_eq!(encounter.start_turn().expect("one"), id(1));
        encounter.end_turn().expect("end one");
        assert_eq!(encounter.start_turn().expect("two"), id(2));
        encounter.end_turn().expect("end two");
        assert_eq!(encounter.round(), 2);
        assert_eq!(encounter.start_turn().expect("again"), id(1));
    }

    /// Player Core, "Step 2: Act". Player Core, "Step 1: Start Your Turn".
    #[test]
    fn after_2_ends_a_dead_3_wraps_to_1_in_round_2() {
        let (mut encounter, _) = begin(
            vec![pc(1, 0, 20, 10), pc(2, 0, 20, 10), dead(pc(3, 0, 20, 10))],
            &[12, 11, 10],
        );
        assert_eq!(encounter.order(), order_of(&[1, 2, 3]).as_slice());
        assert_eq!(encounter.start_turn().expect("one"), id(1));
        encounter.end_turn().expect("end one");
        assert_eq!(encounter.start_turn().expect("two"), id(2));
        encounter.end_turn().expect("end two");
        assert_eq!(encounter.round(), 1);
        assert_eq!(encounter.start_turn().expect("wrap"), id(1));
        assert_eq!(encounter.round(), 2);
    }

    /// Player Core, "Step 1: Start Your Turn".
    #[test]
    fn a_dead_first_combatant_starts_the_other_in_round_1() {
        let (mut encounter, _) = begin(vec![dead(pc(1, 0, 20, 10)), pc(2, 0, 20, 10)], &[10, 5]);
        assert_eq!(encounter.start_turn().expect("start"), id(2));
        assert_eq!(encounter.round(), 1);
    }

    /// Player Core, "Step 2: Act".
    #[test]
    fn a_round_at_the_u32_maximum_does_not_advance() {
        let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10), pc(2, 0, 20, 10)], &[10, 8]);
        let order = encounter.order().to_vec();
        encounter.start_turn().expect("one");
        encounter.end_turn().expect("end one");
        assert_eq!(encounter.start_turn().expect("two"), id(2));
        encounter.set_round_for_test(u32::MAX);
        assert_eq!(
            encounter.end_turn().expect_err("overflow"),
            EncounterError::RoundOverflow
        );
        assert_eq!(encounter.round(), u32::MAX);
        assert_eq!(encounter.order(), order.as_slice());
        assert_eq!(encounter.turn_open(), Some(id(2)));
    }

    /// Player Core, "Step 1: Start Your Turn".
    #[test]
    fn a_dead_combatant_in_the_middle_is_skipped() {
        let (mut encounter, _) = begin(
            vec![pc(1, 0, 20, 10), dead(pc(2, 0, 20, 10)), pc(3, 0, 20, 10)],
            &[12, 11, 10],
        );
        assert_eq!(encounter.order(), order_of(&[1, 2, 3]).as_slice());
        assert_eq!(encounter.start_turn().expect("first"), id(1));
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("next"), id(3));
        assert_eq!(encounter.round(), 1);
        assert_eq!(actions(&encounter, 2), 0);
    }

    /// Player Core, "Step 1: Start Your Turn".
    ///
    /// QA probe p07. Order is dead, alive, dead, dead. After the living turn, the next start
    /// skips two dead combatants and the dead combatant at the wrap, and opens that same
    /// living turn again. A skip that stops after one dead returns no living combatant.
    #[test]
    fn two_dead_combatants_in_a_row_wrap_back_to_the_living_one() {
        let (mut encounter, _) = begin(
            vec![
                dead(pc(1, 0, 20, 10)),
                pc(2, 0, 20, 10),
                dead(pc(3, 0, 20, 10)),
                dead(pc(4, 0, 20, 10)),
            ],
            &[12, 11, 10, 9],
        );
        assert_eq!(encounter.order(), order_of(&[1, 2, 3, 4]).as_slice());
        assert_eq!(encounter.start_turn().expect("living"), id(2));
        assert_eq!(encounter.round(), 1);
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("wrap"), id(2));
        assert_eq!(encounter.round(), 2);
        assert_eq!(actions(&encounter, 1), 0);
        assert_eq!(actions(&encounter, 3), 0);
        assert_eq!(actions(&encounter, 4), 0);
    }

    /// Player Core, "Step 1: Start Your Turn".
    #[test]
    fn the_only_living_combatant_starts_the_next_round() {
        let (mut encounter, _) = begin(vec![pc(1, 0, 20, 10), dead(pc(2, 0, 20, 10))], &[10, 8]);
        assert_eq!(encounter.order(), order_of(&[1, 2]).as_slice());
        assert_eq!(encounter.start_turn().expect("first"), id(1));
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("again"), id(1));
        assert_eq!(encounter.round(), 2);
    }

    /// Player Core, "Step 1: Start Your Turn".
    #[test]
    fn no_living_combatant_returns_an_error() {
        let (mut encounter, _) = begin(
            vec![dead(pc(1, 0, 20, 10)), dead(pc(2, 0, 20, 10))],
            &[10, 8],
        );
        let order = encounter.order().to_vec();
        assert_eq!(encounter.round(), 1);
        assert_eq!(
            encounter.start_turn().expect_err("none"),
            EncounterError::NoLivingCombatant
        );
        assert_eq!(encounter.round(), 1);
        assert_eq!(encounter.order(), order.as_slice());
    }

    /// Player Core, Strike.
    ///
    /// Reach substitute: rule, a melee Strike targets a creature within reach (Player Core pg. 418,
    /// Strike); why not, reach, flanking, and prone are add-positioning; substitute, the caller
    /// names the target and this module stores no squares; intent, Strike is testable without the grid.
    #[test]
    fn the_first_strike_totals_the_d20_plus_the_attack_bonus() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 10]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        let outcome = encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("strike");
        assert_eq!(outcome.map, 0);
        assert_eq!(outcome.total, 18);
        assert_eq!(outcome.degree, DegreeOfSuccess::Success);
        assert_eq!(actions(&encounter, 1), 2);
        assert_eq!(encounter.turn_open(), Some(id(1)));
    }

    /// Player Core, Strike.
    ///
    /// The baked Armor Class result below is replaced by wire-dying-into-turns (DEV-36).
    #[test]
    fn a_not_alive_target_keeps_baked_armor_class() {
        let (mut encounter, mut dice) = begin(
            vec![
                combatant(1, Side::Pc, 0, 20, 18, false),
                combatant(2, Side::Pc, 0, 9, 18, true),
            ],
            &[10, 8, 15, 8, 10],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter
            .strike(
                id(1),
                id(2),
                &profile(10, Agility::NotAgile, 1, 8, 1),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("knockout");
        assert_ne!(vital(&encounter, 2), Vital::Alive);
        assert_ne!(vital(&encounter, 2), Vital::Dead);
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("empty"), id(2));
        encounter.end_turn().expect("close empty");
        assert_eq!(encounter.start_turn().expect("again"), id(1));
        let outcome = encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("second");
        assert_eq!(outcome.total, 18);
        assert_eq!(outcome.degree, DegreeOfSuccess::Success);
    }

    /// Player Core, Strike.
    #[test]
    fn strike_before_the_turn_starts_rolls_nothing() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 10]);
        assert_eq!(
            encounter
                .strike(
                    id(1),
                    id(2),
                    &profile(8, Agility::NotAgile, 0, 8, 0),
                    &[],
                    ShieldBlock::Declined,
                    &mut dice,
                )
                .expect_err("early"),
            EncounterError::TurnNotOpen
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Strike.
    #[test]
    fn a_strike_off_the_active_turn_rolls_nothing() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 10]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        assert_eq!(
            encounter
                .strike(
                    id(2),
                    id(1),
                    &profile(8, Agility::NotAgile, 0, 8, 0),
                    &[],
                    ShieldBlock::Declined,
                    &mut dice,
                )
                .expect_err("off"),
            EncounterError::NotYourTurn
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Strike.
    #[test]
    fn an_unknown_target_rolls_nothing() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 10]);
        encounter.start_turn().expect("start");
        assert_eq!(
            encounter
                .strike(
                    id(1),
                    id(99),
                    &profile(0, Agility::NotAgile, 0, 8, 0),
                    &[],
                    ShieldBlock::Declined,
                    &mut dice,
                )
                .expect_err("missing"),
            EncounterError::UnknownCombatant
        );
        assert_eq!(actions(&encounter, 1), 3);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Strike.
    #[test]
    fn striking_oneself_rolls_nothing() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 10]);
        encounter.start_turn().expect("start");
        assert_eq!(
            encounter
                .strike(
                    id(1),
                    id(1),
                    &profile(0, Agility::NotAgile, 0, 8, 0),
                    &[],
                    ShieldBlock::Declined,
                    &mut dice,
                )
                .expect_err("self"),
            EncounterError::SelfTarget
        );
        assert_eq!(actions(&encounter, 1), 3);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Strike.
    #[test]
    fn a_strike_on_a_dead_target_spends_nothing() {
        let (mut encounter, mut dice) = begin(
            vec![pc(1, 0, 20, 18), dead(pc(2, 0, 20, 18))],
            &[14, 10, 10],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        assert_eq!(
            encounter
                .strike(
                    id(1),
                    id(2),
                    &profile(0, Agility::NotAgile, 0, 8, 0),
                    &[],
                    ShieldBlock::Declined,
                    &mut dice,
                )
                .expect_err("dead"),
            EncounterError::TargetDead
        );
        assert_eq!(actions(&encounter, 1), 3);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Strike.
    #[test]
    fn a_strike_with_no_actions_rolls_nothing() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 10]);
        encounter.start_turn().expect("start");
        encounter.spend_actions(id(1), 3).expect("spend");
        assert_eq!(
            encounter
                .strike(
                    id(1),
                    id(2),
                    &profile(0, Agility::NotAgile, 0, 8, 0),
                    &[],
                    ShieldBlock::Declined,
                    &mut dice,
                )
                .expect_err("empty"),
            EncounterError::InsufficientActions
        );
        assert_eq!(dice.remaining(), 1);
        assert_eq!(encounter.turn_open(), Some(id(1)));
    }

    /// Player Core, Strike.
    #[test]
    fn armor_class_in_the_caller_list_does_not_change_the_result() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 12)], &[14, 10, 10]);
        encounter.start_turn().expect("start");
        let modifier = penalty(PenaltyType::Circumstance, -2, 1, Statistic::ArmorClass);
        let outcome = encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 0, 8, 0),
                &[modifier],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("strike");
        assert_eq!(outcome.total, 10);
        assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
    }

    /// Player Core, Penalties.
    #[test]
    fn the_second_strike_includes_map_and_a_caller_circumstance_penalty() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 16), pc(2, 0, 20, 16)], &[14, 10, 2, 12]);
        encounter.start_turn().expect("start");
        encounter
            .strike(
                id(1),
                id(2),
                &profile(9, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("first");
        let modifier = penalty(PenaltyType::Circumstance, -2, 1, Statistic::Attack);
        let outcome = encounter
            .strike(
                id(1),
                id(2),
                &profile(9, Agility::NotAgile, 0, 8, 0),
                &[modifier],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("second");
        assert_eq!(outcome.map, -5);
        assert_eq!(outcome.total, 14);
        assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
    }

    /// Player Core, Penalties.
    #[test]
    fn two_caller_circumstance_bonuses_keep_the_higher_on_a_strike() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 10), pc(2, 0, 20, 10)], &[14, 10, 10]);
        encounter.start_turn().expect("start");
        let modifiers = [
            bonus(BonusType::Circumstance, 4, 1, Statistic::Attack),
            bonus(BonusType::Circumstance, 1, 2, Statistic::Attack),
        ];
        let outcome = encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 0, 8, 0),
                &modifiers,
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("strike");
        assert_eq!(outcome.total, 14);
        assert_ne!(outcome.total, 15);
    }

    /// Player Core, Penalties. Player Core, "Step 4: Degree of Success".
    #[test]
    fn a_circumstance_bonus_and_penalty_on_the_attack_both_apply() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 10), pc(2, 0, 20, 10)], &[14, 10, 10]);
        encounter.start_turn().expect("start");
        let modifiers = [
            bonus(BonusType::Circumstance, 2, 1, Statistic::Attack),
            penalty(PenaltyType::Circumstance, -2, 2, Statistic::Attack),
        ];
        let outcome = encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 0, 8, 0),
                &modifiers,
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("strike");
        assert_eq!(outcome.total, 10);
        assert_eq!(outcome.degree, DegreeOfSuccess::Success);
        assert_ne!(outcome.total, 12);
        assert_ne!(outcome.total, 8);
    }

    /// Player Core, Penalties.
    #[test]
    fn a_damage_penalty_does_not_change_the_attack_total() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 12]);
        encounter.start_turn().expect("start");
        let modifiers = [
            penalty(PenaltyType::Circumstance, -2, 1, Statistic::Attack),
            penalty(PenaltyType::Untyped, -3, 2, Statistic::Damage),
        ];
        let outcome = encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &modifiers,
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("strike");
        assert_eq!(outcome.total, 18);
        assert_eq!(outcome.degree, DegreeOfSuccess::Success);
    }

    /// Player Core, Penalties. Player Core, "Step 4: Degree of Success".
    #[test]
    fn a_caller_untyped_penalty_stacks_with_map() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 11), pc(2, 0, 20, 11)], &[14, 10, 10, 12]);
        encounter.start_turn().expect("start");
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("first");
        let modifier = penalty(PenaltyType::Untyped, -5, 1, Statistic::Attack);
        let outcome = encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 0, 8, 0),
                &[modifier],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("second");
        assert_eq!(outcome.map, -5);
        assert_eq!(outcome.total, 2);
        assert_ne!(outcome.total, 7);
        assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
    }

    /// Player Core, "Multiple Attack Penalty".
    #[test]
    fn the_second_agile_strike_takes_minus_4() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 2, 14]);
        encounter.start_turn().expect("start");
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::Agile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("first");
        let outcome = encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::Agile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("second");
        assert_eq!(outcome.map, -4);
        assert_eq!(outcome.total, 18);
        assert_eq!(outcome.degree, DegreeOfSuccess::Success);
    }

    /// Player Core, "Multiple Attack Penalty".
    #[test]
    fn the_third_non_agile_strike_takes_minus_10() {
        let (mut encounter, mut dice) = begin(
            vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)],
            &[14, 10, 2, 2, 15],
        );
        encounter.start_turn().expect("start");
        let weapon = profile(8, Agility::NotAgile, 0, 8, 0);
        encounter
            .strike(id(1), id(2), &weapon, &[], ShieldBlock::Declined, &mut dice)
            .expect("first");
        encounter
            .strike(id(1), id(2), &weapon, &[], ShieldBlock::Declined, &mut dice)
            .expect("second");
        let outcome = encounter
            .strike(id(1), id(2), &weapon, &[], ShieldBlock::Declined, &mut dice)
            .expect("third");
        assert_eq!(outcome.map, -10);
        assert_eq!(outcome.total, 13);
        assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
        assert_eq!(actions(&encounter, 1), 0);
        assert_eq!(encounter.turn_open(), Some(id(1)));
    }

    /// Player Core, "Multiple Attack Penalty".
    #[test]
    fn a_rejected_strike_does_not_count() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[12, 8, 10]);
        assert_eq!(
            encounter
                .strike(
                    id(1),
                    id(2),
                    &profile(8, Agility::NotAgile, 0, 8, 0),
                    &[],
                    ShieldBlock::Declined,
                    &mut dice,
                )
                .expect_err("early"),
            EncounterError::TurnNotOpen
        );
        assert_eq!(dice.remaining(), 1);
        encounter.start_turn().expect("start");
        let outcome = encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("strike");
        assert_eq!(outcome.map, 0);
        assert_eq!(outcome.total, 18);
    }

    /// Player Core, "Multiple Attack Penalty".
    #[test]
    fn an_attack_count_at_the_u8_maximum_does_not_increment() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 10]);
        encounter.start_turn().expect("start");
        encounter.set_attack_count_for_test(id(1), 255);
        assert_eq!(
            encounter
                .strike(
                    id(1),
                    id(2),
                    &profile(8, Agility::NotAgile, 0, 8, 0),
                    &[],
                    ShieldBlock::Declined,
                    &mut dice,
                )
                .expect_err("count"),
            EncounterError::AttackCountOverflow
        );
        assert_eq!(actions(&encounter, 1), 3);
        assert_eq!(encounter.attack_count(id(1)).expect("count"), 255);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Multiple Attack Penalty". Player Core pg. 417, Ready.
    #[test]
    fn a_reaction_after_two_strikes_keeps_penalty_0() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 18), pc(2, 0, 20, 18)], &[14, 10, 2, 2, 8]);
        encounter.start_turn().expect("start");
        let weapon = profile(8, Agility::NotAgile, 0, 8, 0);
        encounter
            .strike(id(1), id(2), &weapon, &[], ShieldBlock::Declined, &mut dice)
            .expect("first");
        encounter
            .strike(id(1), id(2), &weapon, &[], ShieldBlock::Declined, &mut dice)
            .expect("second");
        encounter.end_turn().expect("end");
        assert_eq!(
            encounter
                .attack_trait_penalty(id(1), Agility::NotAgile)
                .expect("off turn"),
            0
        );
        assert_eq!(dice.remaining(), 1);
        encounter.start_turn().expect("other");
        encounter.end_turn().expect("end other");
        assert_eq!(encounter.start_turn().expect("next"), id(1));
        assert_eq!(
            encounter
                .attack_trait_penalty(id(1), Agility::NotAgile)
                .expect("next turn"),
            0
        );
    }

    fn strike_damage(
        attack: i32,
        armor_class: i32,
        dice_count: u8,
        face: u8,
        bonus: i32,
        rolls: &[u8],
        modifiers: &[Modifier],
    ) -> (StrikeOutcome, usize) {
        let mut prefix = vec![14, 10];
        prefix.extend_from_slice(rolls);
        let (mut encounter, mut dice) = begin(
            vec![pc(1, 0, 40, armor_class), pc(2, 0, 40, armor_class)],
            &prefix,
        );
        encounter.start_turn().expect("start");
        let outcome = encounter
            .strike(
                id(1),
                id(2),
                &profile(attack, Agility::NotAgile, dice_count, face, bonus),
                modifiers,
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("strike");
        (outcome, dice.remaining())
    }

    /// Player Core, Strike.
    #[test]
    fn a_success_deals_the_damage_roll_once() {
        let (outcome, remaining) = strike_damage(8, 18, 1, 8, 4, &[10, 3], &[]);
        assert_eq!(outcome.damage, 7);
        assert_eq!(remaining, 0);
    }

    /// Player Core, Strike.
    #[test]
    fn a_critical_success_doubles_one_damage_roll() {
        let (outcome, remaining) = strike_damage(10, 15, 1, 8, 4, &[15, 3, 6], &[]);
        assert_eq!(outcome.damage, 14);
        assert_eq!(remaining, 1);
    }

    /// Player Core, Strike.
    #[test]
    fn a_failure_rolls_no_damage_die() {
        let (outcome, remaining) = strike_damage(0, 18, 1, 8, 0, &[9, 8], &[]);
        assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
        assert_eq!(outcome.damage, 0);
        assert_eq!(remaining, 1);
    }

    /// Player Core, Strike.
    #[test]
    fn a_critical_failure_rolls_no_damage_die() {
        let (outcome, remaining) = strike_damage(0, 18, 1, 8, 0, &[5, 8], &[]);
        assert_eq!(outcome.degree, DegreeOfSuccess::CriticalFailure);
        assert_eq!(outcome.damage, 0);
        assert_eq!(remaining, 1);
    }

    /// Player Core, Strike. Player Core, "Natural 1 and Natural 20".
    #[test]
    fn a_natural_20_that_becomes_a_success_deals_damage_once() {
        let (outcome, _) = strike_damage(0, 25, 1, 8, 4, &[20, 3], &[]);
        assert_eq!(outcome.degree, DegreeOfSuccess::Success);
        assert_eq!(outcome.damage, 7);
    }

    /// Player Core, Strike. Player Core, "Natural 1 and Natural 20".
    #[test]
    fn a_natural_20_that_stays_a_failure_deals_no_damage() {
        let (outcome, remaining) = strike_damage(0, 30, 1, 8, 0, &[20, 8], &[]);
        assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
        assert_eq!(outcome.damage, 0);
        assert_eq!(remaining, 1);
    }

    /// Player Core, Strike. Player Core, "Natural 1 and Natural 20".
    #[test]
    fn a_natural_1_that_becomes_a_failure_deals_no_damage() {
        let (outcome, remaining) = strike_damage(17, 15, 1, 8, 0, &[1, 8], &[]);
        assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
        assert_eq!(outcome.damage, 0);
        assert_eq!(remaining, 1);
    }

    /// Player Core, Strike. Player Core, "Natural 1 and Natural 20".
    #[test]
    fn a_natural_1_that_becomes_a_success_deals_damage_once() {
        let (outcome, _) = strike_damage(24, 15, 1, 8, 0, &[1, 4], &[]);
        assert_eq!(outcome.degree, DegreeOfSuccess::Success);
        assert_eq!(outcome.damage, 4);
    }

    /// Player Core, Strike.
    ///
    /// A Failure spends the action and deals no damage. `HpError::NotAHit` stays off this path.
    #[test]
    fn a_failure_short_circuits_with_no_damage() {
        let (mut encounter, mut dice) = begin(
            vec![pc(1, 0, 20, 18), with_temp(pc(2, 0, 20, 18), 5)],
            &[14, 10, 9, 8],
        );
        encounter.start_turn().expect("start");
        let result = encounter.strike(
            id(1),
            id(2),
            &profile(0, Agility::NotAgile, 1, 8, 4),
            &[],
            ShieldBlock::Declined,
            &mut dice,
        );
        assert_ne!(result, Err(EncounterError::HitPoints(HpError::NotAHit)));
        let outcome = result.expect("short-circuit");
        assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
        assert_eq!(outcome.damage, 0);
        assert_eq!(actions(&encounter, 1), 2);
        assert_eq!(dice.remaining(), 1);
        assert_eq!(hit_points(&encounter, 2), 20);
        assert_eq!(temp_hp(&encounter, 2), 5);
        assert_eq!(wounded(&encounter, 2), 0);
        assert_eq!(vital(&encounter, 2), Vital::Alive);
    }

    /// Player Core, Strike.
    ///
    /// A CriticalFailure spends the action and deals no damage. `HpError::NotAHit` stays off this path.
    #[test]
    fn a_critical_failure_short_circuits_with_no_damage() {
        let (mut encounter, mut dice) = begin(
            vec![pc(1, 0, 20, 18), with_temp(pc(2, 0, 20, 18), 5)],
            &[14, 10, 5, 8],
        );
        encounter.start_turn().expect("start");
        let result = encounter.strike(
            id(1),
            id(2),
            &profile(0, Agility::NotAgile, 1, 8, 4),
            &[],
            ShieldBlock::Declined,
            &mut dice,
        );
        assert_ne!(result, Err(EncounterError::HitPoints(HpError::NotAHit)));
        let outcome = result.expect("short-circuit");
        assert_eq!(outcome.degree, DegreeOfSuccess::CriticalFailure);
        assert_eq!(outcome.damage, 0);
        assert_eq!(actions(&encounter, 1), 2);
        assert_eq!(dice.remaining(), 1);
        assert_eq!(hit_points(&encounter, 2), 20);
        assert_eq!(temp_hp(&encounter, 2), 5);
        assert_eq!(wounded(&encounter, 2), 0);
        assert_eq!(vital(&encounter, 2), Vital::Alive);
    }

    /// Player Core, "Damage Rolls". Ruling R-07. DEV-14.
    #[test]
    fn penalties_reduce_a_hit_to_1_damage() {
        let modifier = penalty(PenaltyType::Untyped, -6, 1, Statistic::Damage);
        let (outcome, _) = strike_damage(0, 11, 1, 8, 0, &[11, 4], &[modifier]);
        assert_eq!(outcome.damage, 1);
    }

    /// Player Core, "Damage Rolls". Player Core, "Doubling and Halving". Ruling R-07. DEV-14.
    #[test]
    fn the_same_penalties_on_a_crit_deal_2() {
        let modifier = penalty(PenaltyType::Untyped, -6, 1, Statistic::Damage);
        let (outcome, _) = strike_damage(10, 11, 1, 8, 0, &[11, 4], &[modifier]);
        assert_eq!(outcome.damage, 2);
    }

    /// Player Core, "Damage Rolls". Player Core, "Doubling and Halving". Ruling R-07. DEV-14.
    #[test]
    fn a_pre_penalty_2_with_a_minus_2_penalty_deals_1_or_2_on_a_crit() {
        let modifier = penalty(PenaltyType::Untyped, -2, 1, Statistic::Damage);
        let (hit, _) = strike_damage(0, 11, 1, 6, 0, &[11, 2], &[modifier]);
        let (crit, _) = strike_damage(10, 11, 1, 6, 0, &[11, 2], &[modifier]);
        assert_eq!(hit.damage, 1);
        assert_eq!(crit.damage, 2);
    }

    /// Player Core, "Damage Rolls". Player Core, "Doubling and Halving". Ruling R-07. DEV-14.
    #[test]
    fn a_clamped_total_stays_0_on_a_hit_and_on_a_crit() {
        let (hit, _) = strike_damage(0, 11, 1, 4, -5, &[11, 1], &[]);
        let (crit, _) = strike_damage(10, 11, 1, 4, -5, &[11, 1], &[]);
        assert_eq!(hit.damage, 0);
        assert_eq!(crit.damage, 0);
    }

    /// Player Core, "Damage Rolls". Player Core, "Doubling and Halving". Ruling R-07. DEV-14.
    #[test]
    fn a_negative_modifier_plus_a_penalty_stays_0_on_a_crit() {
        let modifier = penalty(PenaltyType::Untyped, -1, 1, Statistic::Damage);
        let (outcome, _) = strike_damage(10, 11, 1, 4, -5, &[11, 1], &[modifier]);
        assert_eq!(outcome.damage, 0);
    }

    /// Player Core, "Damage Rolls". Player Core, "Doubling and Halving". Ruling R-07. DEV-14.
    #[test]
    fn two_damage_dice_are_doubled_once() {
        let (outcome, remaining) = strike_damage(10, 11, 2, 6, 1, &[11, 3, 4], &[]);
        assert_eq!(outcome.damage, 16);
        assert_eq!(remaining, 0);
    }

    /// Player Core, "Damage Rolls". Ruling R-07. DEV-14.
    #[test]
    fn zero_damage_dice_deal_the_bonus() {
        let (outcome, remaining) = strike_damage(0, 11, 0, 8, 4, &[11, 8], &[]);
        assert_eq!(outcome.damage, 4);
        assert_eq!(remaining, 1);
    }

    /// Player Core, "Damage Rolls". Ruling R-07. DEV-14.
    #[test]
    fn two_circumstance_bonuses_to_damage_keep_the_higher() {
        let modifiers = [
            bonus(BonusType::Circumstance, 4, 1, Statistic::Damage),
            bonus(BonusType::Circumstance, 1, 2, Statistic::Damage),
        ];
        let (outcome, _) = strike_damage(0, 11, 1, 4, 0, &[11, 1], &modifiers);
        assert_eq!(outcome.damage, 5);
    }

    /// Player Core, "Temporary Hit Points". Ruling R-06.
    #[test]
    fn damage_removes_temporary_hit_points_then_hit_points() {
        let mut target = with_temp(pc(2, 0, 20, 11), 5);
        target.state.temp_rounds = 3;
        let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 11), target], &[14, 10, 11, 3]);
        encounter.start_turn().expect("start");
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 1, 8, 4),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("strike");
        assert_eq!(temp_hp(&encounter, 2), 0);
        assert_eq!(encounter.state(id(2)).expect("state").temp_rounds, 0);
        assert_eq!(hit_points(&encounter, 2), 18);
    }

    /// Player Core, "Temporary Hit Points". Ruling R-06.
    #[test]
    fn damage_within_temporary_hit_points_leaves_hit_points() {
        let (mut encounter, mut dice) = begin(
            vec![pc(1, 0, 20, 11), with_temp(pc(2, 0, 20, 11), 5)],
            &[14, 10, 11, 3],
        );
        encounter.start_turn().expect("start");
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 1, 4, 0),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("strike");
        assert_eq!(temp_hp(&encounter, 2), 2);
        assert_eq!(hit_points(&encounter, 2), 20);
    }

    /// Player Core, "Temporary Hit Points". Ruling R-06.
    #[test]
    fn zero_damage_leaves_temporary_hit_points() {
        let (mut encounter, mut dice) = begin(
            vec![pc(1, 0, 20, 11), with_temp(pc(2, 0, 20, 11), 5)],
            &[14, 10, 11, 1],
        );
        let before = hit_points(&encounter, 2);
        encounter.start_turn().expect("start");
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 1, 4, -5),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("strike");
        assert_eq!(temp_hp(&encounter, 2), 5);
        assert_eq!(hit_points(&encounter, 2), before);
    }

    /// Player Core, "Getting Knocked Out".
    #[test]
    fn without_dying_rules_0_hit_points_is_death() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 11), pc(2, 0, 9, 11)], &[14, 10, 11, 8]);
        encounter.start_turn().expect("start");
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 1, 8, 1),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("strike");
        assert_eq!(hit_points(&encounter, 2), 0);
        assert_eq!(vital(&encounter, 2), Vital::Dead);
        assert_eq!(wounded(&encounter, 2), 0);
        assert_eq!(encounter.creature(id(2)).expect("creature").hit_points, 9);
    }

    /// Player Core, "Getting Knocked Out".
    #[test]
    fn a_strike_does_not_change_the_shield() {
        let shield = ShieldTemplate {
            hardness: 0,
            hit_points: 20,
            broken_threshold: 0,
            // An AC bonus of 0 is InvalidShield. The strike still leaves these Hit Points.
            shield_ac_bonus: 1,
        };
        let target = participant(2, Side::Pc, creature(20, 11, 0, false, Some(shield)));
        let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 11), target], &[14, 10, 11, 4]);
        encounter.start_turn().expect("start");
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 1, 4, 0),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("strike");
        let state = encounter.state(id(2)).expect("state");
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(!state.shield_raised);
    }

    /// Player Core, "Damage Rolls".
    #[test]
    fn an_overflowing_attack_total_leaves_the_turn() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 11), pc(2, 0, 9, 11)], &[14, 10, 11, 8]);
        encounter.start_turn().expect("start");
        let modifiers = [
            penalty(PenaltyType::Untyped, i32::MIN, 1, Statistic::Attack),
            penalty(PenaltyType::Untyped, i32::MIN, 2, Statistic::Attack),
        ];
        assert_eq!(
            encounter
                .strike(
                    id(1),
                    id(2),
                    &profile(0, Agility::NotAgile, 0, 8, 0),
                    &modifiers,
                    ShieldBlock::Declined,
                    &mut dice,
                )
                .expect_err("overflow"),
            EncounterError::AttackOverflow(ModifierError::Overflow)
        );
        assert_eq!(dice.remaining(), 1);
        assert_eq!(actions(&encounter, 1), 3);
        assert_eq!(
            encounter
                .attack_trait_penalty(id(1), Agility::NotAgile)
                .expect("penalty"),
            0
        );
    }

    /// Player Core, "Damage Rolls".
    #[test]
    fn an_overflowing_damage_total_leaves_the_turn() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 11), pc(2, 0, 9, 11)], &[14, 10, 11, 1]);
        encounter.start_turn().expect("start");
        assert_eq!(temp_hp(&encounter, 2), 0);
        assert_eq!(
            encounter
                .strike(
                    id(1),
                    id(2),
                    &profile(0, Agility::NotAgile, 1, 4, 2_147_483_647),
                    &[],
                    ShieldBlock::Declined,
                    &mut dice,
                )
                .expect_err("overflow"),
            EncounterError::DamageOverflow(ModifierError::Overflow)
        );
        assert_eq!(dice.remaining(), 0);
        assert_eq!(actions(&encounter, 1), 3);
        assert_eq!(hit_points(&encounter, 2), 9);
        assert_eq!(
            encounter
                .attack_trait_penalty(id(1), Agility::NotAgile)
                .expect("penalty"),
            0
        );
    }

    /// Player Core, "Step 1: Start Your Turn".
    #[test]
    fn a_copied_state_does_not_change_the_encounter() {
        let (encounter, _) = begin(vec![pc(1, 0, 20, 10)], &[10]);
        assert_eq!(actions(&encounter, 1), 0);
        let mut copied = encounter.state(id(1)).expect("state").clone();
        copied.actions_remaining = 9;
        assert_eq!(actions(&encounter, 1), 0);
    }

    /// Player Core, "Damage Rolls".
    ///
    /// A negative amount is not damage and is not an overflow.
    #[test]
    fn a_negative_amount_is_not_overflow() {
        let mut participant = pc(1, 0, 20, 18);
        let creature = participant.creature.clone();
        let result = apply_resolved_blow(
            &mut participant.state,
            &creature,
            -1,
            DegreeOfSuccess::Success,
        );
        assert_eq!(result, Err(HpError::NegativeAmount));
        assert_ne!(result, Err(HpError::Overflow));
        assert_eq!(participant.state.hit_points, 20);
        assert_eq!(participant.state.temp_hit_points, 0);
        assert_eq!(participant.state.wounded, 0);
        assert_eq!(participant.state.vital, Vital::Alive);
    }

    fn take_report(encounter: &mut Encounter, value: u32) -> Option<InitiativeMove> {
        let member = encounter
            .members
            .iter_mut()
            .find(|member| member.participant.id == id(value))
            .expect("member");
        take_initiative_move(&mut member.participant.state)
    }

    /// Player Core pg. 410, "Getting Knocked Out".
    #[test]
    fn with_dying_rules_a_strike_to_0_sets_dying1_and_a_pending_report() {
        let (mut encounter, mut dice) = begin(
            vec![
                combatant(1, Side::Pc, 0, 20, 11, false),
                combatant(2, Side::Pc, 0, 9, 11, true),
            ],
            &[14, 10, 11, 8],
        );
        let order = encounter.order().to_vec();
        encounter.start_turn().expect("start");
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 1, 8, 1),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("strike");
        assert_eq!(hit_points(&encounter, 2), 0);
        assert_eq!(
            vital(&encounter, 2),
            Vital::Dying(DyingValue::new(1).expect("dying 1"))
        );
        assert_eq!(wounded(&encounter, 2), 0);
        assert_eq!(encounter.order(), order.as_slice());
        assert_eq!(
            take_report(&mut encounter, 2),
            Some(InitiativeMove::Pending)
        );

        let (mut crit_encounter, mut crit_dice) = begin(
            vec![
                combatant(1, Side::Pc, 0, 20, 11, false),
                combatant(2, Side::Pc, 0, 9, 11, true),
            ],
            &[14, 10, 20, 4, 20, 1],
        );
        crit_encounter.start_turn().expect("crit start");
        let crit = crit_encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 1, 8, 1),
                &[],
                ShieldBlock::Declined,
                &mut crit_dice,
            )
            .expect("crit strike");
        assert_eq!(crit.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(
            vital(&crit_encounter, 2),
            Vital::Dying(DyingValue::new(2).expect("dying 2"))
        );
        assert_eq!(crit_dice.remaining(), 2);

        let (mut inconsistent, mut inconsistent_dice) = begin(
            vec![
                combatant(1, Side::Pc, 0, 20, 11, false),
                combatant(2, Side::Pc, 0, 9, 11, true),
            ],
            &[14, 10, 11, 8],
        );
        {
            let target = inconsistent
                .members
                .iter_mut()
                .find(|member| member.participant.id == id(2))
                .expect("target");
            target.participant.state.vital = Vital::Dying(DyingValue::new(1).expect("dying 1"));
            target.participant.state.hit_points = 4;
        }
        inconsistent.start_turn().expect("inconsistent start");
        let attacker_actions = actions(&inconsistent, 1);
        let target_actions = actions(&inconsistent, 2);
        assert_eq!(
            inconsistent.strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 1, 8, 1),
                &[],
                ShieldBlock::Declined,
                &mut inconsistent_dice,
            ),
            Err(EncounterError::HitPoints(HpError::InconsistentVital))
        );
        assert_eq!(actions(&inconsistent, 1), attacker_actions);
        assert_eq!(actions(&inconsistent, 2), target_actions);
        assert_eq!(
            vital(&inconsistent, 2),
            Vital::Dying(DyingValue::new(1).expect("dying 1"))
        );
        assert_eq!(hit_points(&inconsistent, 2), 4);
    }

    /// Player Core pg. 412, Massive Damage.
    #[test]
    fn a_massive_strike_leaves_no_dying_value() {
        let (mut encounter, mut dice) = begin(
            vec![
                combatant(1, Side::Pc, 0, 20, 11, false),
                combatant(2, Side::Pc, 0, 10, 11, true),
            ],
            &[14, 10, 11, 8],
        );
        encounter.start_turn().expect("start");
        encounter
            .strike(
                id(1),
                id(2),
                &profile(0, Agility::NotAgile, 1, 8, 12),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("strike");
        assert_eq!(vital(&encounter, 2), Vital::Dead);
        assert!(!matches!(vital(&encounter, 2), Vital::Dying(_)));
        assert_eq!(take_report(&mut encounter, 2), None);
    }

    /// Player Core, "Raise a Shield". Player Core, "Step 2: Act".
    ///
    /// QA probe p11. `Encounter::raise_shield` spends the action.
    #[test]
    fn a_raise_through_the_encounter_leaves_2_actions() {
        let shield = ShieldTemplate {
            hardness: 5,
            hit_points: 20,
            broken_threshold: 10,
            shield_ac_bonus: 2,
        };
        let (mut encounter, _) = begin(
            vec![participant(
                1,
                Side::Pc,
                creature(20, 11, 0, false, Some(shield)),
            )],
            &[10],
        );
        encounter.start_turn().expect("start");
        assert_eq!(actions(&encounter, 1), 3);
        encounter.raise_shield(id(1)).expect("raise");
        let state = encounter.state(id(1)).expect("state");
        assert_eq!(state.actions_remaining, 2);
        assert!(state.shield_raised);
    }

    /// Player Core, "Raise a Shield". Player Core, "Step 2: Act".
    #[test]
    fn a_second_raise_only_spends_an_action() {
        let shield = ShieldTemplate {
            hardness: 5,
            hit_points: 20,
            broken_threshold: 10,
            shield_ac_bonus: 2,
        };
        let (mut encounter, _) = begin(
            vec![participant(
                1,
                Side::Pc,
                creature(20, 11, 0, false, Some(shield)),
            )],
            &[10],
        );
        encounter.start_turn().expect("start");
        {
            let member = encounter
                .members
                .iter_mut()
                .find(|member| member.participant.id == id(1))
                .expect("member");
            raise_combatant_shield(&mut member.participant.state, &member.participant.creature)
                .expect("already raised");
        }
        let before = encounter.state(id(1)).expect("state");
        assert!(before.shield_raised);
        assert_eq!(before.actions_remaining, 3);
        encounter.raise_shield(id(1)).expect("raise");
        let state = encounter.state(id(1)).expect("state");
        let defender = encounter.creature(id(1)).expect("creature");
        assert!(state.shield_raised);
        assert_eq!(state.actions_remaining, 2);
        let modifier = crate::shield::shield_ac_modifier(state, defender).expect("bonus");
        assert_eq!(modifier.value(), 2);
        let total = crate::positioning::armor_class(
            None,
            None,
            crate::positioning::OffGuard::No,
            crate::positioning::AttackKind::Melee,
            &[modifier],
        )
        .expect("armor class");
        assert_eq!(total.get(Statistic::ArmorClass), 2);
        assert_ne!(total.get(Statistic::ArmorClass), 4);
    }

    /// Player Core, "Raise a Shield". Player Core, "Step 2: Act".
    #[test]
    fn another_combatant_cannot_raise_on_this_turn() {
        let shield = ShieldTemplate {
            hardness: 5,
            hit_points: 20,
            broken_threshold: 10,
            shield_ac_bonus: 2,
        };
        let (mut encounter, dice) = begin(
            vec![
                pc(1, 0, 20, 11),
                participant(2, Side::Pc, creature(20, 11, 0, false, Some(shield))),
            ],
            &[10, 8],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        {
            let member = encounter
                .members
                .iter_mut()
                .find(|member| member.participant.id == id(2))
                .expect("member");
            member.participant.state.actions_remaining = 3;
            member.participant.state.shield_raised = false;
        }
        let error = encounter.raise_shield(id(2)).expect_err("other turn");
        assert_eq!(error, EncounterError::NotYourTurn);
        let state = encounter.state(id(2)).expect("state");
        assert_eq!(state.actions_remaining, 3);
        assert!(!state.shield_raised);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, Strike. Player Core, "Shield Block".
    #[test]
    fn a_legal_shield_block_changes_shield_hit_points() {
        let shield = ShieldTemplate {
            hardness: 5,
            hit_points: 20,
            broken_threshold: 10,
            shield_ac_bonus: 2,
        };
        let (mut encounter, mut dice) = begin(
            vec![
                participant(1, Side::Pc, creature(30, 11, 0, false, Some(shield))),
                participant(2, Side::Pc, creature(20, 11, 0, false, None)),
            ],
            &[10, 8, 11, 8],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.raise_shield(id(1)).expect("raise");
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("attacker"), id(2));
        let outcome = encounter
            .strike(
                id(2),
                id(1),
                &profile(0, Agility::NotAgile, 1, 8, 4),
                &[],
                ShieldBlock::Requested,
                &mut dice,
            )
            .expect("strike");
        assert_eq!(outcome.degree, DegreeOfSuccess::Success);
        assert_eq!(hit_points(&encounter, 1), 23);
        assert_eq!(
            encounter.state(id(1)).expect("state").shield_hit_points,
            Some(13)
        );
        assert_eq!(dice.remaining(), 0);
    }

    fn aided(
        value: u32,
        side: Side,
        perception: i32,
        modifier: Option<i32>,
        rank: Option<AidRank>,
    ) -> Participant {
        let parts = BuildParts {
            hit_points: 20,
            armor_class: 11,
            perception,
            speed_feet: 25,
            strike: MeleeStrike {
                attack_bonus: 0,
                damage_dice: DieCount::new(1).expect("dice"),
                damage_faces: faces(8),
                damage_bonus: 0,
                reach_feet: ReachFeet::new(5).expect("reach"),
                agile: false,
            },
            shield: None,
            stealth_dc: None,
            aid_modifier: modifier,
            aid_rank: rank,
            uses_dying_rules: Some(false),
        };
        participant(
            value,
            side,
            derive(&CharacterBuild::try_new(parts).expect("build")),
        )
    }

    fn preparation(encounter: &Encounter, value: u32) -> AidPreparation {
        encounter.state(id(value)).expect("state").aid_preparation
    }

    // `prepare_to_aid` spends an action and refuses a missing modifier, so those prepared states cannot be reached through it.
    fn set_prepared(encounter: &mut Encounter, aider: u32, ally: u32) {
        let member = encounter
            .members
            .iter_mut()
            .find(|member| member.participant.id == id(aider))
            .expect("member");
        member.participant.state.aid_preparation = AidPreparation::Prepared { ally: id(ally) };
    }

    fn dying_value() -> DyingValue {
        DyingValue::new(1).expect("dying 1")
    }

    /// Player Core, Aid.
    #[test]
    fn the_same_aid_twice_matches() {
        let once = || {
            let (mut encounter, mut dice) = begin(
                vec![
                    aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                    aided(2, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                ],
                &[10, 8, 10],
            );
            assert_eq!(encounter.start_turn().expect("start"), id(1));
            encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
            encounter.end_turn().expect("end");
            assert_eq!(encounter.start_turn().expect("ally"), id(2));
            let outcome = encounter.aid(id(1), id(2), &mut dice).expect("aid");
            assert_eq!(dice.remaining(), 0);
            outcome
        };
        let first = once();
        let second = once();
        assert_eq!(first, second);
        assert_eq!(first.total, 17);
        assert_eq!(first.degree, DegreeOfSuccess::Success);
        assert_eq!(first.circumstance, AidCircumstance::Plus1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 2: Act".
    #[test]
    fn preparing_spends_one_action() {
        let (mut encounter, dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                pc(2, 0, 20, 11),
            ],
            &[10, 8, 10],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        assert_eq!(actions(&encounter, 1), 2);
        assert_eq!(
            preparation(&encounter, 1),
            AidPreparation::Prepared { ally: id(2) }
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 2: Act".
    #[test]
    fn preparing_again_only_spends_an_action() {
        let (mut encounter, dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                pc(2, 0, 20, 11),
            ],
            &[10, 8, 8],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        set_prepared(&mut encounter, 1, 2);
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        assert_eq!(actions(&encounter, 1), 2);
        assert_eq!(
            preparation(&encounter, 1),
            AidPreparation::Prepared { ally: id(2) }
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 2: Act".
    #[test]
    fn preparing_again_for_a_new_ally_replaces_the_name() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                pc(2, 0, 20, 11),
                pc(3, 0, 20, 11),
            ],
            &[10, 8, 6, 10],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        set_prepared(&mut encounter, 1, 2);
        assert!(reaction(&encounter, 1));
        encounter.prepare_to_aid(id(1), id(3)).expect("prepare");
        let error = encounter
            .aid(id(1), id(2), &mut dice)
            .expect_err("old ally");
        assert_eq!(actions(&encounter, 1), 2);
        assert_eq!(
            preparation(&encounter, 1),
            AidPreparation::Prepared { ally: id(3) }
        );
        assert_eq!(error, EncounterError::NotThePreparedAlly);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 2: Act".
    #[test]
    fn preparing_off_turn_is_notyourturn() {
        let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 11), pc(2, 0, 20, 11)], &[10, 9, 8]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.set_vital_for_test(id(2), Vital::Unconscious);
        assert_eq!(preparation(&encounter, 2), AidPreparation::NotPrepared);
        assert!(reaction(&encounter, 2));
        let error = encounter
            .prepare_to_aid(id(2), id(1))
            .expect_err("off turn");
        assert_eq!(error, EncounterError::NotYourTurn);
        assert_eq!(preparation(&encounter, 2), AidPreparation::NotPrepared);
        assert!(reaction(&encounter, 2));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 2: Act".
    #[test]
    fn a_dying_preparer_on_their_turn_returns_notconscious() {
        let (mut encounter, dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                pc(2, 0, 20, 11),
            ],
            &[10, 8, 8],
        );
        encounter.set_vital_for_test(id(1), Vital::Dying(dying_value()));
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        let error = encounter.prepare_to_aid(id(1), id(2)).expect_err("dying");
        assert_eq!(error, EncounterError::NotConscious);
        assert_eq!(actions(&encounter, 1), 0);
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 2: Act".
    #[test]
    fn no_actions_does_not_prepare() {
        let (mut encounter, dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                pc(2, 0, 20, 11),
            ],
            &[10, 8, 8],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.spend_actions(id(1), 3).expect("spend");
        let error = encounter
            .prepare_to_aid(id(1), id(2))
            .expect_err("no actions");
        assert_eq!(error, EncounterError::InsufficientActions);
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    #[test]
    fn preparing_with_no_aid_modifier_does_not_spend_the_action() {
        let (mut encounter, dice) = begin(vec![pc(1, 0, 20, 11), pc(2, 0, 20, 11)], &[10, 8, 8]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        let error = encounter
            .prepare_to_aid(id(1), id(2))
            .expect_err("no modifier");
        assert_eq!(error, EncounterError::NoAidModifier);
        assert_eq!(actions(&encounter, 1), 3);
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    #[test]
    fn preparing_for_an_enemy_is_notanally() {
        let (mut encounter, dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                enemy(2, 0, 20, 11),
            ],
            &[10, 8, 10],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        let error = encounter.prepare_to_aid(id(1), id(2)).expect_err("enemy");
        assert_eq!(error, EncounterError::NotAnAlly);
        assert_eq!(actions(&encounter, 1), 3);
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    #[test]
    fn preparing_for_an_unknown_ally_is_unknowncombatant() {
        let (mut encounter, dice) = begin(
            vec![aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster))],
            &[10, 10],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        let error = encounter.prepare_to_aid(id(1), id(9)).expect_err("unknown");
        assert_eq!(error, EncounterError::UnknownCombatant);
        assert_eq!(actions(&encounter, 1), 3);
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    #[test]
    fn preparing_for_yourself_is_selftarget() {
        let (mut encounter, dice) = begin(
            vec![aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster))],
            &[10, 10],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        let error = encounter.prepare_to_aid(id(1), id(1)).expect_err("self");
        assert_eq!(error, EncounterError::SelfTarget);
        assert_eq!(actions(&encounter, 1), 3);
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 1: Start Your Turn".
    #[test]
    fn a_participant_starts_not_prepared() {
        let mut starter = pc(1, 0, 20, 11);
        starter.state.aid_preparation = AidPreparation::Prepared { ally: id(2) };
        let (encounter, dice) = begin(vec![starter], &[10]);
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        assert!(reaction(&encounter, 1));
        assert_eq!(actions(&encounter, 1), 0);
        assert_eq!(initiative(&encounter, 1), 10);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 1: Start Your Turn".
    #[test]
    fn another_turn_does_not_clear_preparation() {
        let (mut encounter, dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                aided(2, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            ],
            &[10, 8, 8],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("ally"), id(2));
        assert_eq!(
            preparation(&encounter, 1),
            AidPreparation::Prepared { ally: id(2) }
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 1: Start Your Turn".
    #[test]
    fn your_next_turn_clears_preparation() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                aided(2, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            ],
            &[10, 8, 10],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("ally"), id(2));
        encounter.end_turn().expect("end ally");
        assert_eq!(encounter.start_turn().expect("next"), id(1));
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        assert_eq!(actions(&encounter, 1), 3);
        assert!(reaction(&encounter, 1));
        let error = encounter.aid(id(1), id(2), &mut dice).expect_err("cleared");
        assert_eq!(error, EncounterError::NotPrepared);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 1: Start Your Turn".
    #[test]
    fn an_unused_preparation_expires_at_the_next_start() {
        let (mut encounter, dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                aided(2, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
            ],
            &[10, 8, 8],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("ally"), id(2));
        encounter.end_turn().expect("end ally");
        assert_eq!(encounter.start_turn().expect("next"), id(1));
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 1: Start Your Turn".
    #[test]
    fn a_successful_aid_clears_preparation() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                pc(2, 0, 20, 11),
            ],
            &[10, 8, 10, 8],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("ally"), id(2));
        let first = encounter.aid(id(1), id(2), &mut dice).expect("aid");
        assert_eq!(first.total, 17);
        assert_eq!(first.degree, DegreeOfSuccess::Success);
        assert_eq!(first.circumstance, AidCircumstance::Plus1);
        assert!(!reaction(&encounter, 1));
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        let error = encounter.aid(id(1), id(2), &mut dice).expect_err("again");
        assert_eq!(error, EncounterError::NotPrepared);
        assert_ne!(error, EncounterError::NoReaction);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 1: Start Your Turn".
    #[test]
    fn aiding_a_different_ally_is_notthepreparedally() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                pc(2, 0, 20, 11),
                pc(3, 0, 20, 11),
            ],
            &[10, 8, 6, 10],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        let error = encounter.aid(id(1), id(3), &mut dice).expect_err("other");
        assert_eq!(error, EncounterError::NotThePreparedAlly);
        assert_eq!(
            preparation(&encounter, 1),
            AidPreparation::Prepared { ally: id(2) }
        );
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 1: Start Your Turn".
    #[test]
    fn round_overflow_keeps_preparation() {
        let (mut encounter, dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                pc(2, 0, 20, 11),
            ],
            &[10, 8, 8],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("ally"), id(2));
        encounter.round = 4_294_967_295;
        let error = encounter.end_turn().expect_err("overflow");
        assert_eq!(error, EncounterError::RoundOverflow);
        assert_eq!(
            preparation(&encounter, 1),
            AidPreparation::Prepared { ally: id(2) }
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Reactions in Encounters".
    #[test]
    fn a_success_returns_plus_1() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::Master)),
                pc(2, 0, 20, 11),
            ],
            &[10, 8, 10],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("ally"), id(2));
        let hp = hit_points(&encounter, 2);
        assert_eq!(actions(&encounter, 1), 0);
        assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
        assert!(reaction(&encounter, 1));
        let outcome = encounter.aid(id(1), id(2), &mut dice).expect("aid");
        assert_eq!(outcome.total, 17);
        assert_eq!(outcome.degree, DegreeOfSuccess::Success);
        assert_eq!(outcome.circumstance, AidCircumstance::Plus1);
        assert_eq!(actions(&encounter, 1), 0);
        assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
        assert!(!reaction(&encounter, 1));
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        assert_eq!(actions(&encounter, 2), 3);
        assert_eq!(hit_points(&encounter, 2), hp);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, Aid.
    /// Player Core, "Reactions in Encounters".
    #[test]
    fn a_master_critical_success_returns_plus_3() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::Master)),
                pc(2, 0, 20, 11),
            ],
            &[10, 8, 18],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("ally"), id(2));
        let outcome = encounter.aid(id(1), id(2), &mut dice).expect("aid");
        assert_eq!(outcome.total, 25);
        assert_eq!(outcome.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(outcome.circumstance, AidCircumstance::Plus3);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, Aid.
    /// Player Core, "Reactions in Encounters".
    #[test]
    fn a_legendary_critical_success_returns_plus_4() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::Legendary)),
                pc(2, 0, 20, 11),
            ],
            &[10, 8, 18],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("ally"), id(2));
        let outcome = encounter.aid(id(1), id(2), &mut dice).expect("aid");
        assert_eq!(outcome.total, 25);
        assert_eq!(outcome.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(outcome.circumstance, AidCircumstance::Plus4);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, Aid.
    /// Player Core, "Reactions in Encounters".
    #[test]
    fn a_failure_returns_zero() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                pc(2, 0, 20, 11),
            ],
            &[10, 8, 4],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("ally"), id(2));
        let outcome = encounter.aid(id(1), id(2), &mut dice).expect("aid");
        assert_eq!(outcome.total, 11);
        assert_eq!(outcome.degree, DegreeOfSuccess::Failure);
        assert_eq!(outcome.circumstance, AidCircumstance::Zero);
        assert!(!reaction(&encounter, 1));
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, Aid.
    /// Player Core, "Reactions in Encounters".
    #[test]
    fn a_critical_failure_returns_minus_1() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(4), Some(AidRank::Master)),
                pc(2, 0, 20, 11),
            ],
            &[10, 8, 1],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("ally"), id(2));
        let outcome = encounter.aid(id(1), id(2), &mut dice).expect("aid");
        assert_eq!(outcome.total, 5);
        assert_eq!(outcome.degree, DegreeOfSuccess::CriticalFailure);
        assert_eq!(outcome.circumstance, AidCircumstance::Minus1);
        assert!(!reaction(&encounter, 1));
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, Aid.
    /// Player Core, "Reactions in Encounters".
    #[test]
    fn a_dead_ally_can_be_aided() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                dead(pc(2, 0, 20, 11)),
                pc(3, 0, 20, 11),
            ],
            &[10, 8, 6, 10],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("third"), id(3));
        assert_eq!(vital(&encounter, 2), Vital::Dead);
        let outcome = encounter.aid(id(1), id(2), &mut dice).expect("aid");
        assert_eq!(outcome.total, 17);
        assert_eq!(outcome.degree, DegreeOfSuccess::Success);
        assert_eq!(outcome.circumstance, AidCircumstance::Plus1);
        assert!(!reaction(&encounter, 1));
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, Aid.
    /// Player Core, "Reactions in Encounters".
    #[test]
    fn aid_with_the_reaction_already_spent_is_noreaction() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                pc(2, 0, 20, 11),
            ],
            &[10, 8, 10],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.spend_reaction(id(1)).expect("spend reaction");
        let error = encounter.aid(id(1), id(2), &mut dice).expect_err("spent");
        assert_eq!(error, EncounterError::NoReaction);
        assert_eq!(
            preparation(&encounter, 1),
            AidPreparation::Prepared { ally: id(2) }
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    #[test]
    fn aid_does_not_change_the_attack_count() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                pc(2, 0, 20, 11),
            ],
            &[10, 8, 10],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("ally"), id(2));
        assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
        encounter.aid(id(1), id(2), &mut dice).expect("aid");
        assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, Aid.
    #[test]
    fn an_enemy_may_aid_an_enemy() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Enemy, 0, Some(7), Some(AidRank::BelowMaster)),
                aided(2, Side::Enemy, 0, Some(7), Some(AidRank::BelowMaster)),
            ],
            &[10, 8, 10],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        let outcome = encounter.aid(id(1), id(2), &mut dice).expect("aid");
        assert_eq!(outcome.total, 17);
        assert_eq!(outcome.degree, DegreeOfSuccess::Success);
        assert_eq!(outcome.circumstance, AidCircumstance::Plus1);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 2: Act".
    /// Player Core, Unconscious.
    #[test]
    fn an_unknown_aider_rolls_nothing() {
        let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 11)], &[1, 8]);
        assert!(encounter.turn_open().is_none());
        let error = encounter.aid(id(9), id(1), &mut dice).expect_err("unknown");
        assert_eq!(error, EncounterError::UnknownCombatant);
        assert_ne!(error, EncounterError::TurnNotOpen);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 2: Act".
    /// Player Core, Unconscious.
    #[test]
    fn an_unknown_ally_rolls_nothing() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                pc(2, 0, 20, 11),
            ],
            &[1, 1, 8],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.end_turn().expect("end");
        assert!(reaction(&encounter, 1));
        assert!(encounter.turn_open().is_none());
        let error = encounter.aid(id(1), id(9), &mut dice).expect_err("unknown");
        assert_eq!(error, EncounterError::UnknownCombatant);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 2: Act".
    /// Player Core, Unconscious.
    #[test]
    fn aid_with_no_open_turn_is_turnnotopen() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                pc(2, 0, 20, 11),
            ],
            &[1, 1, 10],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.end_turn().expect("end");
        assert!(reaction(&encounter, 1));
        assert!(encounter.turn_open().is_none());
        let error = encounter.aid(id(1), id(2), &mut dice).expect_err("closed");
        assert_eq!(error, EncounterError::TurnNotOpen);
        assert!(reaction(&encounter, 1));
        assert_eq!(
            preparation(&encounter, 1),
            AidPreparation::Prepared { ally: id(2) }
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 2: Act".
    /// Player Core, Unconscious.
    #[test]
    fn turnnotopen_wins_over_notconscious() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                pc(2, 0, 20, 11),
            ],
            &[1, 1, 8],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.set_vital_for_test(id(1), Vital::Unconscious);
        encounter.end_turn().expect("end");
        assert!(encounter.turn_open().is_none());
        let error = encounter.aid(id(1), id(2), &mut dice).expect_err("closed");
        assert_eq!(error, EncounterError::TurnNotOpen);
        assert_ne!(error, EncounterError::NotConscious);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 2: Act".
    /// Player Core, Unconscious.
    #[test]
    fn notconscious_wins_over_later_errors() {
        let (mut encounter, mut dice) = begin(vec![pc(1, 0, 20, 11)], &[1, 8]);
        encounter.set_vital_for_test(id(1), Vital::Unconscious);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        assert!(!reaction(&encounter, 1));
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        let error = encounter.aid(id(1), id(1), &mut dice).expect_err("down");
        assert_eq!(error, EncounterError::NotConscious);
        assert!(!reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 2: Act".
    /// Player Core, Unconscious.
    #[test]
    fn a_dying_aider_returns_notconscious() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                pc(2, 0, 20, 11),
            ],
            &[10, 8, 18],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.set_vital_for_test(id(1), Vital::Dying(dying_value()));
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("ally"), id(2));
        assert!(reaction(&encounter, 1));
        let error = encounter.aid(id(1), id(2), &mut dice).expect_err("dying");
        assert_eq!(error, EncounterError::NotConscious);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 2: Act".
    /// Player Core, Unconscious.
    #[test]
    fn a_dead_aider_returns_notconscious() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster)),
                pc(2, 0, 20, 11),
            ],
            &[10, 8, 10],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.set_vital_for_test(id(1), Vital::Dead);
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("ally"), id(2));
        assert!(reaction(&encounter, 1));
        let error = encounter.aid(id(1), id(2), &mut dice).expect_err("dead");
        assert_eq!(error, EncounterError::NotConscious);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    #[test]
    fn aiding_yourself_is_selftarget() {
        let (mut encounter, mut dice) = begin(
            vec![aided(1, Side::Pc, 0, Some(7), Some(AidRank::BelowMaster))],
            &[10, 10],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        assert!(reaction(&encounter, 1));
        let error = encounter.aid(id(1), id(1), &mut dice).expect_err("self");
        assert_eq!(error, EncounterError::SelfTarget);
        assert_ne!(error, EncounterError::NotPrepared);
        assert!(reaction(&encounter, 1));
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    #[test]
    fn a_different_side_is_notanally() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 11), enemy(2, 0, 20, 11)], &[10, 8, 10]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.spend_reaction(id(1)).expect("spend reaction");
        assert!(!reaction(&encounter, 1));
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        let error = encounter.aid(id(1), id(2), &mut dice).expect_err("side");
        assert_eq!(error, EncounterError::NotAnAlly);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 2: Act".
    /// Player Core, Unconscious.
    #[test]
    fn an_unprepared_aider_does_not_roll() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 11), pc(2, 0, 20, 11)], &[10, 8, 10]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.spend_reaction(id(1)).expect("spend reaction");
        assert_eq!(preparation(&encounter, 1), AidPreparation::NotPrepared);
        let error = encounter
            .aid(id(1), id(2), &mut dice)
            .expect_err("unprepared");
        assert_eq!(error, EncounterError::NotPrepared);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 2: Act".
    /// Player Core, Unconscious.
    #[test]
    fn no_aid_modifier_does_not_spend_the_reaction() {
        let (mut encounter, mut dice) =
            begin(vec![pc(1, 0, 20, 11), pc(2, 0, 20, 11)], &[10, 8, 10]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        set_prepared(&mut encounter, 1, 2);
        encounter.spend_reaction(id(1)).expect("spend reaction");
        assert!(!reaction(&encounter, 1));
        let error = encounter
            .aid(id(1), id(2), &mut dice)
            .expect_err("modifier");
        assert_eq!(error, EncounterError::NoAidModifier);
        assert!(!reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 2: Act".
    /// Player Core, Unconscious.
    #[test]
    fn an_aid_modifier_at_the_d20_limit_is_rolled() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(
                    1,
                    Side::Pc,
                    0,
                    Some(2_147_483_627),
                    Some(AidRank::BelowMaster),
                ),
                pc(2, 0, 20, 11),
            ],
            &[10, 8, 20],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        let outcome = encounter.aid(id(1), id(2), &mut dice).expect("aid");
        assert_eq!(outcome.total, 2_147_483_647);
        assert_eq!(outcome.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(outcome.circumstance, AidCircumstance::Plus2);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, Aid.
    /// Player Core, "Step 2: Act".
    /// Player Core, Unconscious.
    #[test]
    fn the_next_aid_modifier_above_the_d20_limit_is_aidoverflow() {
        let (mut encounter, mut dice) = begin(
            vec![
                aided(
                    1,
                    Side::Pc,
                    0,
                    Some(2_147_483_628),
                    Some(AidRank::BelowMaster),
                ),
                pc(2, 0, 20, 11),
            ],
            &[10, 8, 20],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        assert!(reaction(&encounter, 1));
        let error = encounter
            .aid(id(1), id(2), &mut dice)
            .expect_err("overflow");
        assert_eq!(
            error,
            EncounterError::AidOverflow(ArithmeticOverflow::SumDoesNotFit)
        );
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid. Player Core, "Shield Block".
    #[test]
    fn a_reaction_spent_by_shield_block_still_reports_aidoverflow() {
        let shield = ShieldTemplate {
            hardness: 5,
            hit_points: 20,
            broken_threshold: 10,
            shield_ac_bonus: 2,
        };
        let parts = BuildParts {
            hit_points: 30,
            armor_class: 11,
            perception: 0,
            speed_feet: 25,
            strike: MeleeStrike {
                attack_bonus: 0,
                damage_dice: DieCount::new(1).expect("dice"),
                damage_faces: faces(8),
                damage_bonus: 0,
                reach_feet: ReachFeet::new(5).expect("reach"),
                agile: false,
            },
            shield: Some(shield),
            stealth_dc: None,
            aid_modifier: Some(i32::MAX - 19),
            aid_rank: Some(AidRank::BelowMaster),
            uses_dying_rules: Some(false),
        };
        let aider = participant(
            1,
            Side::Pc,
            derive(&CharacterBuild::try_new(parts).expect("build")),
        );
        let (mut encounter, mut dice) = begin(
            vec![aider, pc(2, 0, 20, 11), enemy(3, 0, 20, 11)],
            &[10, 8, 6, 12, 8, 20],
        );
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.prepare_to_aid(id(1), id(2)).expect("prepare");
        encounter.raise_shield(id(1)).expect("raise");
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("ally"), id(2));
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("attacker"), id(3));
        encounter
            .strike(
                id(3),
                id(1),
                &profile(0, Agility::NotAgile, 1, 8, 4),
                &[],
                ShieldBlock::Requested,
                &mut dice,
            )
            .expect("strike");
        assert!(!reaction(&encounter, 1));
        let error = encounter
            .aid(id(1), id(2), &mut dice)
            .expect_err("overflow");
        assert_eq!(
            error,
            EncounterError::AidOverflow(ArithmeticOverflow::SumDoesNotFit)
        );
        assert_ne!(error, EncounterError::NoReaction);
        assert_eq!(dice.remaining(), 1);
    }

    const MANIPULATE: &[ActionTrait] = &[ActionTrait::Manipulate];
    const MOVE_ONLY: &[ActionTrait] = &[ActionTrait::Move];
    const RANGED_ATTACK: &[ActionTrait] = &[ActionTrait::Ranged];
    const MOVE_AND_MANIPULATE: &[ActionTrait] = &[ActionTrait::Move, ActionTrait::Manipulate];
    const NO_TRIGGER: &[ActionTrait] = &[];

    fn one_action(traits: &'static [ActionTrait]) -> ActionCost {
        ActionCost::Actions { count: 1, traits }
    }

    fn free_action(traits: &'static [ActionTrait]) -> ActionCost {
        ActionCost::Free { traits }
    }

    fn sq(x: i32, y: i32) -> Square {
        Square { x, y }
    }

    fn with_reach(mut participant: Participant, reach: i32) -> Participant {
        participant.creature.strike.reach_feet = ReachFeet::new(reach).expect("reach");
        participant
    }

    fn with_current(mut participant: Participant, hp: i32) -> Participant {
        participant.state.hit_points = hp;
        participant
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "forwards reactive_strike's arguments so each scenario stays one call"
    )]
    fn react(
        encounter: &mut Encounter,
        dice: &mut ScriptedDice,
        reactor: u32,
        actor: u32,
        cost: ActionCost,
        identity: ActionIdentity,
        move_action: Option<MoveAction>,
        availability: ReactiveStrike,
        from: (i32, i32),
        to: (i32, i32),
        from_size: Size,
        to_size: Size,
        attack: StrikeProfile,
        modifiers: &[Modifier],
    ) -> Result<ReactiveOutcome, ReactiveError> {
        encounter.reactive_strike(
            id(reactor),
            id(actor),
            cost,
            identity,
            move_action,
            availability,
            sq(from.0, from.1),
            sq(to.0, to.1),
            from_size,
            to_size,
            &attack,
            modifiers,
            dice,
        )
    }

    fn standard_pair(
        reach: i32,
        actor_hp: i32,
        actor_ac: i32,
        actor_side: Side,
    ) -> Vec<Participant> {
        vec![
            with_reach(pc(1, 0, 20, actor_ac), reach),
            combatant(2, actor_side, 0, actor_hp, actor_ac, false),
        ]
    }

    /// Player Core, "Reactive Strike".
    #[test]
    fn the_same_reactive_strike_twice_matches() {
        let run = || {
            let (mut encounter, mut dice) =
                begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 3]);
            react(
                &mut encounter,
                &mut dice,
                1,
                2,
                one_action(MANIPULATE),
                ActionIdentity::Other,
                None,
                ReactiveStrike::Available,
                (0, 0),
                (-1, 0),
                Size::Medium,
                Size::Medium,
                profile(8, Agility::NotAgile, 1, 8, 4),
                &[],
            )
            .expect("reactive strike")
        };
        let first = run();
        let second = run();
        assert_eq!(first, second);
        assert_eq!(first.strike.natural, 10);
        assert_eq!(first.strike.total, 18);
        assert_eq!(first.strike.degree, DegreeOfSuccess::Success);
        assert_eq!(first.strike.map, 0);
        assert_eq!(first.strike.damage, 7);
        assert_eq!(first.disruption, Disruption::NotDisrupted);
    }

    /// Player Core, "Reactive Strike". Player Core, "Interact". Player Core, "Point Out".
    ///
    /// Interact is one action with manipulate. Point Out is manipulate when the caller passes it.
    #[test]
    fn a_manipulate_action_in_reach_triggers() {
        let (mut encounter, mut dice) =
            begin(standard_pair(5, 20, 18, Side::Enemy), &[10, 8, 10, 3]);
        assert_eq!(encounter.turn_open(), None);
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (-1, 0),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("manipulate");
        assert_eq!(outcome.strike.degree, DegreeOfSuccess::Success);
        assert_eq!(outcome.strike.total, 18);
        assert_eq!(outcome.strike.map, 0);
        assert_eq!(outcome.strike.damage, 7);
        assert_eq!(outcome.disruption, Disruption::NotDisrupted);
        assert_eq!(hit_points(&encounter, 2), 13);
        assert!(!reaction(&encounter, 1));
        assert_eq!(actions(&encounter, 1), 0);
        assert_eq!(encounter.turn_open(), None);
    }

    /// Player Core, "Reactive Strike". Player Core, "Drop Prone". Player Core, "Stand".
    ///
    /// Drop Prone, Stand, Stride, and Crawl are Move. Stand is Actions 2303 (Player Core pg. 418).
    /// This call passes the move trait and does not resolve those actions.
    #[test]
    fn a_move_action_in_reach_triggers() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 3]);
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MOVE_ONLY),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (-1, 0),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("move");
        assert_eq!(outcome.strike.degree, DegreeOfSuccess::Success);
        assert_eq!(outcome.strike.map, 0);
        assert_eq!(outcome.strike.damage, 7);
        assert_eq!(outcome.disruption, Disruption::NotDisrupted);
        assert!(!reaction(&encounter, 1));
        assert_eq!(actions(&encounter, 1), 0);
    }

    /// Player Core, "Reactive Strike".
    ///
    /// `Ranged` means this action is a ranged attack. It is not a Player Core action trait.
    #[test]
    fn a_ranged_attack_in_reach_triggers() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 3]);
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(RANGED_ATTACK),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (-1, 0),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("ranged");
        assert_eq!(outcome.strike.degree, DegreeOfSuccess::Success);
        assert_eq!(outcome.strike.map, 0);
        assert_eq!(outcome.strike.damage, 7);
        assert_eq!(outcome.disruption, Disruption::NotDisrupted);
        assert!(!reaction(&encounter, 1));
        assert_eq!(actions(&encounter, 1), 0);
    }

    /// Player Core, "Reactive Strike".
    ///
    /// Both creatures are Pc. Side is not read.
    #[test]
    fn the_same_side_still_triggers() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 3]);
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (-1, 0),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("same side");
        assert_eq!(outcome.strike.degree, DegreeOfSuccess::Success);
        assert_eq!(outcome.strike.map, 0);
        assert_eq!(outcome.strike.damage, 7);
        assert_eq!(outcome.disruption, Disruption::NotDisrupted);
        assert_eq!(hit_points(&encounter, 2), 13);
        assert!(!reaction(&encounter, 1));
        assert_eq!(actions(&encounter, 1), 0);
    }

    /// Player Core, "Reactive Strike".
    #[test]
    fn move_and_manipulate_on_one_cost_trigger_once() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 3]);
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MOVE_AND_MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("one outcome");
        assert_eq!(outcome.strike.map, 0);
        assert_eq!(outcome.strike.damage, 7);
        assert_eq!(outcome.disruption, Disruption::NotDisrupted);
        assert_eq!(hit_points(&encounter, 2), 13);
        assert!(!reaction(&encounter, 1));
        assert_eq!(actions(&encounter, 1), 0);
    }

    /// Player Core, "Reactive Strike". Player Core, "Release".
    #[test]
    fn a_free_manipulate_action_other_than_release_triggers() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 3]);
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            free_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("free manipulate");
        assert_eq!(outcome.strike.degree, DegreeOfSuccess::Success);
        assert_eq!(outcome.strike.map, 0);
        assert_eq!(outcome.strike.damage, 7);
        assert_eq!(outcome.disruption, Disruption::NotDisrupted);
        assert!(!reaction(&encounter, 1));
        assert_eq!(actions(&encounter, 1), 0);
    }

    /// Player Core, "Step".
    #[test]
    fn step_does_not_trigger() {
        let (mut encounter, mut dice) = begin(
            vec![
                with_reach(pc(1, 0, 20, 18), 7),
                combatant(2, Side::Pc, 0, 20, 18, false),
            ],
            &[10, 8, 10],
        );
        assert!(reaction(&encounter, 1));
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MOVE_ONLY),
            ActionIdentity::Other,
            Some(MoveAction::Step),
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Large,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("step");
        assert_eq!(error, ReactiveError::NotATrigger);
        assert_ne!(error, ReactiveError::UnsupportedSize);
        assert_ne!(error, ReactiveError::InvalidReach);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Strike.
    #[test]
    fn a_melee_strike_does_not_trigger() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(NO_TRIGGER),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("melee");
        assert_eq!(error, ReactiveError::NotATrigger);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Release".
    #[test]
    fn release_does_not_trigger() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            free_action(MANIPULATE),
            ActionIdentity::Release,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("release");
        assert_eq!(error, ReactiveError::NotATrigger);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Seek".
    #[test]
    fn seek_does_not_trigger() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(NO_TRIGGER),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("seek");
        assert_eq!(error, ReactiveError::NotATrigger);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Range and Reach". Player Core, "Reactive Strike".
    #[test]
    fn out_of_reach_does_not_trigger() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 2),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("reach");
        assert_eq!(error, ReactiveError::OutOfReach);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Range and Reach".
    #[test]
    fn reach_0_does_not_trigger() {
        let (mut encounter, mut dice) = begin(
            vec![
                with_reach(pc(1, 0, 20, 18), 0),
                combatant(2, Side::Pc, 0, 20, 18, false),
            ],
            &[10, 8, 10],
        );
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("reach 0");
        assert_eq!(error, ReactiveError::OutOfReach);
        assert_ne!(error, ReactiveError::InvalidReach);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Range and Reach".
    #[test]
    fn the_same_square_is_not_in_reach() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 0),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("same square");
        assert_eq!(error, ReactiveError::OutOfReach);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Range and Reach". Player Core, "Size, Space, and Reach".
    #[test]
    fn a_reactive_reach_that_is_not_a_multiple_of_5_is_rejected() {
        let (mut encounter, mut dice) = begin(
            vec![
                with_reach(pc(1, 0, 20, 18), 7),
                combatant(2, Side::Pc, 0, 20, 18, false),
            ],
            &[10, 8, 10],
        );
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("reach 7");
        assert_eq!(error, ReactiveError::InvalidReach);
        assert_ne!(error, ReactiveError::OutOfReach);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Size, Space, and Reach".
    #[test]
    fn a_large_creature_is_unsupported() {
        let (mut encounter, mut dice) = begin(
            vec![
                with_reach(pc(1, 0, 20, 18), 7),
                combatant(2, Side::Pc, 0, 20, 18, false),
            ],
            &[10, 8, 10],
        );
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Large,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("large");
        assert_eq!(error, ReactiveError::UnsupportedSize);
        assert_ne!(error, ReactiveError::InvalidReach);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Range and Reach".
    #[test]
    fn a_distance_that_does_not_fit_is_overflow() {
        let (mut encounter, mut dice) = begin(
            vec![
                with_reach(pc(1, 0, 20, 18), 7),
                combatant(2, Side::Pc, 0, 20, 18, false),
            ],
            &[10, 8, 10],
        );
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (i32::MIN, 0),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("overflow");
        assert_eq!(error, ReactiveError::Overflow);
        assert_ne!(error, ReactiveError::InvalidReach);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Range and Reach".
    #[test]
    fn a_10_ft_reach_diagonal_is_in_reach() {
        let (mut encounter, mut dice) = begin(
            vec![
                with_reach(pc(1, 0, 20, 18), 10),
                combatant(2, Side::Pc, 0, 20, 18, false),
            ],
            &[10, 8, 10, 3],
        );
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (2, 2),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("diagonal");
        assert_eq!(outcome.strike.degree, DegreeOfSuccess::Success);
        assert_eq!(outcome.strike.damage, 7);
        assert!(!reaction(&encounter, 1));
    }

    /// Player Core, "Range and Reach".
    #[test]
    fn a_10_ft_reach_does_not_reach_three_squares_straight() {
        let (mut encounter, mut dice) = begin(
            vec![
                with_reach(pc(1, 0, 20, 18), 10),
                combatant(2, Side::Pc, 0, 20, 18, false),
            ],
            &[10, 8, 10],
        );
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (3, 0),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("three squares");
        assert_eq!(error, ReactiveError::OutOfReach);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Reactive Strike".
    ///
    /// Proposed DEV-45: only the actor's square is tested. No path is read, and this test
    /// does not read a SquareEvent.
    #[test]
    fn a_move_that_starts_outside_reach_does_not_trigger() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MOVE_ONLY),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 2),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("outside");
        assert_eq!(error, ReactiveError::OutOfReach);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Reactions in Encounters".
    #[test]
    fn a_second_reactive_strike_is_noreaction() {
        let (mut encounter, mut dice) =
            begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 3, 4]);
        let first = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (-1, 0),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("first");
        assert_eq!(first.strike.damage, 7);
        assert_eq!(hit_points(&encounter, 2), 13);
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (-1, 0),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("second");
        assert_eq!(error, ReactiveError::NoReaction);
        assert_eq!(dice.remaining(), 1);
        assert_eq!(hit_points(&encounter, 2), 13);
    }

    /// Player Core, "Reactions in Encounters".
    ///
    /// The reaction was spent through Encounter::spend_reaction, as Aid would.
    #[test]
    fn a_reaction_already_spent_is_noreaction() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
        encounter.spend_reaction(id(1)).expect("spend");
        assert!(!reaction(&encounter, 1));
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("spent");
        assert_eq!(error, ReactiveError::NoReaction);
        assert_eq!(dice.remaining(), 1);
        assert!(!reaction(&encounter, 1));
    }

    /// Player Core, "Reactions in Encounters".
    #[test]
    fn actor_actions_stay_unchanged() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[8, 10, 10, 3]);
        assert_eq!(encounter.start_turn().expect("start"), id(2));
        assert_eq!(actions(&encounter, 2), 3);
        react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("spend");
        assert_eq!(actions(&encounter, 2), 3);
        assert_eq!(encounter.turn_open(), Some(id(2)));
        assert_eq!(actions(&encounter, 1), 0);
        assert!(!reaction(&encounter, 1));
    }

    /// Player Core, "Reactions in Encounters".
    #[test]
    fn a_reactive_strike_on_your_turn_spends_no_action() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 3]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        assert_eq!(actions(&encounter, 1), 3);
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("own turn");
        assert_eq!(actions(&encounter, 1), 3);
        assert_eq!(encounter.turn_open(), Some(id(1)));
        assert!(!reaction(&encounter, 1));
        assert_eq!(outcome.strike.map, 0);
        assert_eq!(outcome.strike.damage, 7);
        assert_eq!(hit_points(&encounter, 2), 13);
    }

    /// Player Core, Strike. Player Core, "Multiple Attack Penalty".
    #[test]
    fn a_reactive_strike_off_your_turn_spends_no_action() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[8, 10, 10, 3]);
        assert_eq!(encounter.start_turn().expect("start"), id(2));
        assert_eq!(actions(&encounter, 1), 0);
        assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("off turn");
        assert_eq!(actions(&encounter, 1), 0);
        assert_eq!(outcome.strike.map, 0);
        assert_eq!(encounter.attack_count(id(1)).expect("count"), 0);
        assert!(!reaction(&encounter, 1));
        assert_eq!(encounter.turn_open(), Some(id(2)));
    }

    /// Player Core, Strike. Player Core, "Multiple Attack Penalty".
    #[test]
    fn a_reactive_strike_off_your_turn_leaves_the_count() {
        let (mut encounter, mut dice) =
            begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 2, 2, 10, 3]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        let miss = profile(8, Agility::NotAgile, 0, 8, 0);
        encounter
            .strike(id(1), id(2), &miss, &[], ShieldBlock::Declined, &mut dice)
            .expect("first");
        encounter
            .strike(id(1), id(2), &miss, &[], ShieldBlock::Declined, &mut dice)
            .expect("second");
        encounter.end_turn().expect("end");
        assert_eq!(encounter.start_turn().expect("next"), id(2));
        assert_eq!(encounter.attack_count(id(1)).expect("count"), 2);
        assert_eq!(actions(&encounter, 1), 0);
        assert!(reaction(&encounter, 1));
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("off turn");
        assert_eq!(outcome.strike.map, 0);
        assert_eq!(encounter.attack_count(id(1)).expect("count"), 2);
        assert_eq!(actions(&encounter, 1), 0);
        assert_eq!(encounter.turn_open(), Some(id(2)));
    }

    /// Player Core, "Reactive Strike". Player Core, "Multiple Attack Penalty".
    #[test]
    fn reactive_strike_does_not_use_map() {
        let (mut encounter, mut dice) =
            begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 2, 2, 18, 3]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        let miss = profile(8, Agility::NotAgile, 0, 8, 0);
        encounter
            .strike(id(1), id(2), &miss, &[], ShieldBlock::Declined, &mut dice)
            .expect("first");
        encounter
            .strike(id(1), id(2), &miss, &[], ShieldBlock::Declined, &mut dice)
            .expect("second");
        assert_eq!(encounter.attack_count(id(1)).expect("count"), 2);
        assert_eq!(actions(&encounter, 1), 1);
        let modifiers = [penalty(PenaltyType::Circumstance, -2, 1, Statistic::Attack)];
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::Agile, 1, 8, 4),
            &modifiers,
        )
        .expect("agile reaction");
        assert_eq!(outcome.strike.map, 0);
        assert_eq!(outcome.strike.total, 24);
        assert_ne!(outcome.strike.total, 16);
        assert_ne!(outcome.strike.total, 14);
        assert_eq!(outcome.strike.degree, DegreeOfSuccess::Success);
        assert_eq!(outcome.strike.damage, 7);
        assert_eq!(encounter.attack_count(id(1)).expect("count"), 2);
        assert_eq!(actions(&encounter, 1), 1);
    }

    /// Player Core, "Reactive Strike". Player Core, "Multiple Attack Penalty".
    #[test]
    fn reactive_strike_does_not_count_toward_map() {
        let (mut encounter, mut dice) =
            begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 2, 10, 3, 12]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("first");
        assert_eq!(encounter.attack_count(id(1)).expect("count"), 1);
        assert_eq!(actions(&encounter, 1), 2);
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("reaction");
        assert_eq!(outcome.strike.map, 0);
        assert_eq!(outcome.strike.damage, 7);
        assert_eq!(encounter.attack_count(id(1)).expect("count"), 1);
        let follow = encounter
            .strike(
                id(1),
                id(2),
                &profile(8, Agility::NotAgile, 0, 8, 0),
                &[],
                ShieldBlock::Declined,
                &mut dice,
            )
            .expect("follow");
        assert_eq!(follow.map, -5);
        assert_eq!(follow.total, 15);
        assert_ne!(follow.map, -10);
        assert_ne!(follow.total, 10);
    }

    /// Player Core, "Reactive Strike". Player Core, "Disrupting Actions".
    ///
    /// The result is Disrupted. The caller voids the manipulate action's effects.
    /// This function leaves the cost spent.
    #[test]
    fn a_critical_hit_disrupts_a_manipulate_action() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 15, Side::Pc), &[8, 10, 15, 3]);
        assert_eq!(encounter.start_turn().expect("start"), id(2));
        encounter.spend_actions(id(2), 1).expect("spent cost");
        assert_eq!(actions(&encounter, 2), 2);
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(10, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("crit");
        assert_eq!(outcome.strike.total, 25);
        assert_eq!(outcome.strike.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(outcome.strike.damage, 14);
        assert_eq!(outcome.disruption, Disruption::Disrupted);
        assert_eq!(hit_points(&encounter, 2), 6);
        assert_eq!(actions(&encounter, 2), 2);
        assert_eq!(encounter.turn_open(), Some(id(2)));
        assert!(!reaction(&encounter, 1));
    }

    /// Player Core, "Reactive Strike". Player Core, "Disrupting Actions".
    #[test]
    fn a_critical_hit_does_not_disrupt_a_move() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 15, Side::Pc), &[10, 8, 15, 3]);
        assert_eq!(encounter.turn_open(), None);
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MOVE_ONLY),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(10, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("move crit");
        assert_eq!(outcome.strike.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(outcome.strike.damage, 14);
        assert_eq!(outcome.disruption, Disruption::NotDisrupted);
        assert!(!reaction(&encounter, 1));
    }

    /// Player Core, "Reactive Strike". Player Core, "Disrupting Actions".
    #[test]
    fn a_critical_hit_does_not_disrupt_a_ranged_attack() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 15, Side::Pc), &[10, 8, 15, 3]);
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(RANGED_ATTACK),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(10, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("ranged crit");
        assert_eq!(outcome.strike.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(outcome.strike.damage, 14);
        assert_eq!(outcome.disruption, Disruption::NotDisrupted);
    }

    /// Player Core, "Reactive Strike". Player Core, "Disrupting Actions".
    #[test]
    fn a_natural_20_that_is_only_a_success_does_not_disrupt() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 25, Side::Pc), &[10, 8, 20, 3]);
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(0, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("natural 20");
        assert_eq!(outcome.strike.total, 20);
        assert_eq!(outcome.strike.degree, DegreeOfSuccess::Success);
        assert_eq!(outcome.strike.damage, 7);
        assert_ne!(outcome.strike.damage, 14);
        assert_eq!(outcome.disruption, Disruption::NotDisrupted);
        assert_eq!(hit_points(&encounter, 2), 13);
        assert_eq!(vital(&encounter, 1), Vital::Alive);
        assert_eq!(vital(&encounter, 2), Vital::Alive);
    }

    /// Player Core, "Reactive Strike". Player Core, "Disrupting Actions".
    #[test]
    fn actor_actions_are_unchanged_on_disrupted() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 15, Side::Pc), &[8, 10, 15, 3]);
        assert_eq!(encounter.start_turn().expect("start"), id(2));
        encounter.spend_actions(id(2), 2).expect("two spent");
        assert_eq!(actions(&encounter, 2), 1);
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(10, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("disrupted");
        assert_eq!(outcome.disruption, Disruption::Disrupted);
        assert_eq!(actions(&encounter, 2), 1);
        assert_ne!(actions(&encounter, 2), 2);
        assert_ne!(actions(&encounter, 2), 0);
        assert_eq!(encounter.turn_open(), Some(id(2)));
    }

    /// Player Core, "Step 2: Act".
    #[test]
    fn a_dying_creature_cannot_react() {
        let reactor = with_current(with_reach(pc(1, 0, 20, 18), 5), 0);
        let (mut encounter, mut dice) = begin(
            vec![reactor, combatant(2, Side::Pc, 0, 20, 18, false)],
            &[10, 8, 10],
        );
        encounter.set_vital_for_test(id(1), Vital::Dying(DyingValue::new(1).expect("dying")));
        assert!(reaction(&encounter, 1));
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("dying");
        assert_eq!(error, ReactiveError::NotConscious);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Unconscious". Player Core, "Step 2: Act".
    #[test]
    fn an_unconscious_creature_cannot_react() {
        let reactor = with_current(with_reach(pc(1, 0, 20, 18), 5), 0);
        let (mut encounter, mut dice) = begin(
            vec![reactor, combatant(2, Side::Pc, 0, 20, 18, false)],
            &[10, 8, 10],
        );
        encounter.set_vital_for_test(id(1), Vital::Unconscious);
        assert!(reaction(&encounter, 1));
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("unconscious");
        assert_eq!(error, ReactiveError::NotConscious);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Step 2: Act".
    #[test]
    fn a_dead_creature_cannot_react() {
        let (mut encounter, mut dice) = begin(
            vec![
                with_reach(pc(1, 0, 20, 18), 5),
                combatant(2, Side::Pc, 0, 20, 18, false),
            ],
            &[10, 8, 10],
        );
        assert!(reaction(&encounter, 1));
        encounter.set_vital_for_test(id(1), Vital::Dead);
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("dead");
        assert_eq!(error, ReactiveError::NotConscious);
        assert_ne!(error, ReactiveError::NoReaction);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Step 2: Act". Player Core, "Reactive Strike".
    #[test]
    fn not_conscious_wins_over_unavailable() {
        let reactor = with_current(with_reach(pc(1, 0, 20, 18), 5), 0);
        let (mut encounter, mut dice) = begin(
            vec![reactor, combatant(2, Side::Pc, 0, 20, 18, false)],
            &[10, 8, 10],
        );
        encounter.set_vital_for_test(id(1), Vital::Dying(DyingValue::new(1).expect("dying")));
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Unavailable,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("dying");
        assert_eq!(error, ReactiveError::NotConscious);
        assert_ne!(error, ReactiveError::Unavailable);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Step 2: Act".
    #[test]
    fn a_dying_self_target_is_not_conscious() {
        let reactor = with_current(with_reach(pc(1, 0, 20, 18), 5), 0);
        let (mut encounter, mut dice) = begin(vec![reactor], &[10, 10]);
        encounter.set_vital_for_test(id(1), Vital::Dying(DyingValue::new(1).expect("dying")));
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            1,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 0),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("self");
        assert_eq!(error, ReactiveError::NotConscious);
        assert_ne!(error, ReactiveError::SelfTarget);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Reactive Strike".
    ///
    /// Proposed DEV-44: the caller passes Unavailable. There is no class and no feat.
    #[test]
    fn unavailable_spends_nothing() {
        let (mut encounter, mut dice) = begin(
            vec![
                with_reach(pc(1, 0, 20, 18), 7),
                combatant(2, Side::Pc, 0, 20, 18, false),
            ],
            &[10, 8, 10],
        );
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MOVE_ONLY),
            ActionIdentity::Other,
            Some(MoveAction::Step),
            ReactiveStrike::Unavailable,
            (0, 0),
            (0, 1),
            Size::Large,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("unavailable");
        assert_eq!(error, ReactiveError::Unavailable);
        assert_ne!(error, ReactiveError::NotATrigger);
        assert_ne!(error, ReactiveError::InvalidReach);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Strike. Player Core, "Temporary Hit Points".
    #[test]
    fn temporary_hit_points_are_spent_first() {
        let actor = with_temp(combatant(2, Side::Pc, 0, 20, 18, false), 5);
        let (mut encounter, mut dice) = begin(
            vec![with_reach(pc(1, 0, 20, 18), 5), actor],
            &[10, 8, 10, 3],
        );
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("temp");
        assert_eq!(outcome.strike.damage, 7);
        assert_eq!(temp_hp(&encounter, 2), 0);
        assert_eq!(hit_points(&encounter, 2), 18);
        assert_eq!(wounded(&encounter, 2), 0);
        assert_eq!(vital(&encounter, 2), Vital::Alive);
    }

    /// Player Core, "Temporary Hit Points".
    ///
    /// Strike's `damage_removes_temporary_hit_points_then_hit_points` clears `temp_rounds`
    /// when the same 5 temporary Hit Points take 7 damage. This reaction does the same.
    #[test]
    fn a_reactive_strike_clears_temp_rounds_when_temporary_hit_points_are_spent() {
        let mut actor = with_temp(combatant(2, Side::Pc, 0, 20, 18, false), 5);
        actor.state.temp_rounds = 3;
        let (mut encounter, mut dice) = begin(
            vec![with_reach(pc(1, 0, 20, 18), 5), actor],
            &[10, 8, 10, 3],
        );
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("temp rounds");
        assert_eq!(outcome.strike.damage, 7);
        assert_eq!(temp_hp(&encounter, 2), 0);
        assert_eq!(hit_points(&encounter, 2), 18);
        assert_eq!(encounter.state(id(2)).expect("state").temp_rounds, 0);
    }

    /// Player Core, Strike.
    #[test]
    fn a_failure_spends_the_reaction_and_deals_no_damage() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 9, 8]);
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(0, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("miss");
        assert_eq!(outcome.strike.total, 9);
        assert_eq!(outcome.strike.degree, DegreeOfSuccess::Failure);
        assert_eq!(outcome.strike.damage, 0);
        assert_eq!(outcome.disruption, Disruption::NotDisrupted);
        assert_eq!(dice.remaining(), 1);
        assert_eq!(hit_points(&encounter, 2), 20);
        assert!(!reaction(&encounter, 1));
        assert_eq!(outcome.strike.map, 0);
    }

    /// Player Core, Strike.
    #[test]
    fn a_critical_failure_spends_the_reaction_and_deals_no_damage() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 5, 8]);
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(0, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("crit miss");
        assert_eq!(outcome.strike.degree, DegreeOfSuccess::CriticalFailure);
        assert_eq!(outcome.strike.damage, 0);
        assert_eq!(outcome.disruption, Disruption::NotDisrupted);
        assert_eq!(dice.remaining(), 1);
        assert_eq!(hit_points(&encounter, 2), 20);
        assert!(!reaction(&encounter, 1));
    }

    /// Player Core, "Getting Knocked Out".
    #[test]
    fn a_critical_hit_drops_a_dying_rules_target_to_dying_2() {
        let actor = with_current(combatant(2, Side::Pc, 0, 20, 15, true), 5);
        let (mut encounter, mut dice) = begin(
            vec![with_reach(pc(1, 0, 20, 15), 5), actor],
            &[10, 8, 15, 3],
        );
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(10, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("knockout");
        assert_eq!(outcome.strike.damage, 14);
        assert_eq!(hit_points(&encounter, 2), 0);
        assert_eq!(
            vital(&encounter, 2),
            Vital::Dying(DyingValue::new(2).expect("dying 2"))
        );
        assert_eq!(wounded(&encounter, 2), 0);
        assert_eq!(
            take_report(&mut encounter, 2),
            Some(InitiativeMove::Pending)
        );
        assert!(!reaction(&encounter, 1));
    }

    /// Player Core, "Shield Block".
    #[test]
    fn a_reactive_strike_does_not_reduce_a_raised_shield() {
        let shield = ShieldTemplate {
            hardness: 5,
            hit_points: 20,
            broken_threshold: 10,
            shield_ac_bonus: 2,
        };
        let mut actor = participant(2, Side::Pc, creature(20, 18, 0, false, Some(shield)));
        shield::raise_shield(&mut actor.state, &actor.creature).expect("raise");
        let (mut encounter, mut dice) = begin(
            vec![with_reach(pc(1, 0, 20, 18), 5), actor],
            &[10, 8, 10, 3],
        );
        assert!(reaction(&encounter, 2));
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect("shield");
        assert_eq!(outcome.strike.damage, 7);
        assert_eq!(hit_points(&encounter, 2), 13);
        assert_eq!(
            encounter.state(id(2)).expect("state").shield_hit_points,
            Some(20)
        );
        assert!(reaction(&encounter, 2));
    }

    /// Player Core, "Reactive Strike".
    #[test]
    fn an_unconscious_target_returns_targetnotconscious() {
        let actor = with_current(combatant(2, Side::Pc, 0, 20, 18, false), 0);
        let (mut encounter, mut dice) =
            begin(vec![with_reach(pc(1, 0, 20, 18), 5), actor], &[10, 8, 10]);
        encounter.set_vital_for_test(id(2), Vital::Unconscious);
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("unconscious target");
        assert_eq!(error, ReactiveError::TargetNotConscious);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Reactive Strike".
    #[test]
    fn a_dying_target_in_reach_is_targetnotconscious() {
        let actor = with_current(combatant(2, Side::Pc, 0, 20, 18, true), 0);
        let (mut encounter, mut dice) =
            begin(vec![with_reach(pc(1, 0, 20, 18), 5), actor], &[10, 8, 10]);
        encounter.set_vital_for_test(id(2), Vital::Dying(DyingValue::new(1).expect("dying")));
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("dying target");
        assert_eq!(error, ReactiveError::TargetNotConscious);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Reactive Strike".
    #[test]
    fn a_dead_target_in_reach_is_targetnotconscious() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
        encounter.spend_reaction(id(1)).expect("spend");
        encounter.set_vital_for_test(id(2), Vital::Dead);
        assert!(!reaction(&encounter, 1));
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("dead target");
        assert_eq!(error, ReactiveError::TargetNotConscious);
        assert_ne!(error, ReactiveError::NoReaction);
        assert!(!reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Reactive Strike".
    #[test]
    fn a_reactive_strike_on_oneself_rolls_nothing() {
        let (mut encounter, mut dice) = begin(vec![with_reach(pc(1, 0, 20, 18), 5)], &[10, 10]);
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            1,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 0),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("self");
        assert_eq!(error, ReactiveError::SelfTarget);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Reactive Strike".
    #[test]
    fn an_unknown_reactor_rolls_nothing() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
        let ready = reaction(&encounter, 2);
        let error = react(
            &mut encounter,
            &mut dice,
            9,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("unknown reactor");
        assert_eq!(error, ReactiveError::UnknownCombatant);
        assert_eq!(dice.remaining(), 1);
        assert_eq!(reaction(&encounter, 2), ready);
    }

    /// Player Core, "Reactive Strike".
    #[test]
    fn an_unknown_actor_rolls_nothing() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            9,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("unknown actor");
        assert_eq!(error, ReactiveError::UnknownCombatant);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Reactive Strike". Player Core, "Range and Reach".
    #[test]
    fn a_dead_actor_out_of_reach_is_out_of_reach() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
        encounter.set_vital_for_test(id(2), Vital::Dead);
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 2),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("out of reach");
        assert_eq!(error, ReactiveError::OutOfReach);
        assert_ne!(error, ReactiveError::TargetNotConscious);
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Reactive Strike". Player Core, Penalties.
    #[test]
    fn an_attack_bonus_that_does_not_fit_spends_nothing() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10]);
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(2_147_483_647, Agility::NotAgile, 1, 8, 4),
            &[],
        )
        .expect_err("overflow");
        assert_eq!(
            error,
            ReactiveError::AttackOverflow(ModifierError::Overflow)
        );
        assert!(reaction(&encounter, 1));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Reactive Strike".
    #[test]
    fn an_attack_bonus_of_2147483627_and_a_natural_20_fit() {
        let (mut encounter, mut dice) = begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 20]);
        let outcome = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(2_147_483_627, Agility::NotAgile, 0, 8, 0),
            &[],
        )
        .expect("fits");
        assert_eq!(outcome.strike.total, 2_147_483_647);
        assert_eq!(outcome.strike.degree, DegreeOfSuccess::CriticalSuccess);
        assert!(!reaction(&encounter, 1));
    }

    /// Player Core, "Reactive Strike". Player Core, "Damage Rolls".
    #[test]
    fn a_damage_total_that_does_not_fit_spends_nothing() {
        let (mut encounter, mut dice) =
            begin(standard_pair(5, 20, 18, Side::Pc), &[10, 8, 10, 1, 4]);
        let error = react(
            &mut encounter,
            &mut dice,
            1,
            2,
            one_action(MANIPULATE),
            ActionIdentity::Other,
            None,
            ReactiveStrike::Available,
            (0, 0),
            (0, 1),
            Size::Medium,
            Size::Medium,
            profile(8, Agility::NotAgile, 1, 8, 2_147_483_647),
            &[],
        )
        .expect_err("damage overflow");
        assert_eq!(
            error,
            ReactiveError::DamageOverflow(ModifierError::Overflow)
        );
        assert_eq!(dice.remaining(), 1);
        assert!(reaction(&encounter, 1));
        assert_eq!(hit_points(&encounter, 2), 20);
    }

    mod delay_ready_tests;
}
