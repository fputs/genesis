//! Initiative, the round, the three-action turn, Strike, and Reactive Strike.
//!
//! Player Core, "Step 1: Roll Initiative". Player Core, "Step 2: Act". Player Core, Strike.
//! Player Core, "Reactive Strike".

use crate::check::{
    AttackTotalTerm, DegreeOfSuccess, MapInput, attack_check_total, degree_of_success,
    multiple_attack_penalty,
};
use crate::combatant::{
    CombatantId, CombatantState, Creature, DieCount, HpError, Vital, damage_dealt,
};
use crate::dice::{Dice, Faces};
use crate::dying::apply_resolved_blow;
use crate::grid::{Size, Square};
use crate::modifiers::{Modifier, ModifierError, Statistic};
use crate::positioning::{ActionCost, MoveAction};
use crate::reactive::{
    ActionIdentity, ReactiveError, ReactiveOutcome, ReactiveStrike, attack_bonus_fits, disruption,
    not_a_trigger, outside_reach, reactive_attack_total,
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
}

/// Rolls initiative and builds the order.
///
/// Dice are drawn in ascending [`CombatantId`] order, including a dead combatant. The total is
/// the face plus the baked Perception modifier. A natural 20 does not add 10. Sort is higher
/// total, then [`Side::Enemy`] before [`Side::Pc`], then the lower id. Same-Side ties use that
/// lower id (DEV-05). A duplicate id, an empty roster, or a Perception modifier that cannot be
/// added to 20 is rejected before any die is drawn.
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
            Member {
                participant,
                attack_count: 0,
                initiative: 0,
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
    /// no reaction. A second start while a turn is open is [`EncounterError::TurnInProgress`].
    ///
    /// Player Core, "Step 1: Start Your Turn". Player Core, "Step 2: Act".
    pub fn start_turn(&mut self) -> Result<CombatantId, EncounterError> {
        if self.open.is_some() {
            return Err(EncounterError::TurnInProgress);
        }
        if self.no_living() {
            return Err(EncounterError::NoLivingCombatant);
        }
        let len = self.order.len();
        let mut index = self.cursor;
        let mut round = self.round;
        let mut steps = 0usize;
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
            if index + 1 >= len {
                round = round.checked_add(1).ok_or(EncounterError::RoundOverflow)?;
                index = 0;
            } else {
                index += 1;
            }
        };
        self.cursor = index;
        self.round = round;
        self.open_at(id);
        Ok(id)
    }

    /// Closes the open turn and advances once.
    ///
    /// The empty end step runs, actions drop to 0, and an unspent reaction stays available.
    /// Moving past the last combatant wraps the cursor and adds 1 to the round. A second call
    /// is [`EncounterError::TurnNotOpen`] and does not advance again.
    ///
    /// Player Core, "Step 3: End Your Turn".
    pub fn end_turn(&mut self) -> Result<(), EncounterError> {
        let Some(id) = self.open else {
            return Err(EncounterError::TurnNotOpen);
        };
        let len = self.order.len();
        let next = self.cursor.saturating_add(1);
        let new_round = if next >= len {
            Some(
                self.round
                    .checked_add(1)
                    .ok_or(EncounterError::RoundOverflow)?,
            )
        } else {
            None
        };
        self.end_of_turn_step(id);
        if let Ok(index) = self.member_index(id)
            && let Some(member) = self.members.get_mut(index)
        {
            member.participant.state.actions_remaining = 0;
        }
        self.open = None;
        if let Some(round) = new_round {
            self.round = round;
            self.cursor = 0;
        } else {
            self.cursor = next;
        }
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
        let index = self.member_index(id)?;
        let member = self
            .members
            .get_mut(index)
            .ok_or(EncounterError::UnknownCombatant)?;
        if member.participant.state.vital != Vital::Alive {
            return Err(EncounterError::NotConscious);
        }
        if !member.participant.state.reaction_ready {
            return Err(EncounterError::NoReaction);
        }
        member.participant.state.reaction_ready = false;
        Ok(())
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
    /// The reaction is spent only through [`Self::spend_reaction`], and only after the block
    /// is legal. This does not call [`crate::shield::apply_attack_blow`]. A blow that fails
    /// restores the state, so the reaction spend is not kept.
    ///
    /// Player Core, "Shield Block".
    pub fn shield_block(
        &mut self,
        defender: CombatantId,
        amount: i32,
        degree: DegreeOfSuccess,
    ) -> Result<(), EncounterError> {
        let index = self.member_index(defender)?;
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
        if let ShieldBlock::Requested = block {
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
        let (reactor_vital, reaction_ready) = {
            let state = self
                .state(reactor)
                .map_err(|_| ReactiveError::UnknownCombatant)?;
            (state.vital, state.reaction_ready)
        };
        let actor_vital = self
            .state(actor)
            .map_err(|_| ReactiveError::UnknownCombatant)?
            .vital;
        if reactor_vital != Vital::Alive {
            return Err(ReactiveError::NotConscious);
        }
        if reactor == actor {
            return Err(ReactiveError::SelfTarget);
        }
        if matches!(availability, ReactiveStrike::Unavailable) {
            return Err(ReactiveError::Unavailable);
        }
        if not_a_trigger(cost, identity, move_action) {
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
        if !reaction_ready {
            return Err(ReactiveError::NoReaction);
        }
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

    #[cfg(test)]
    fn initiative_total(&self, id: CombatantId) -> Option<i32> {
        self.members
            .iter()
            .find(|member| member.participant.id == id)
            .map(|member| member.initiative)
    }

    fn attack_count(&self, id: CombatantId) -> Result<u8, EncounterError> {
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
        self.start_of_turn_step(id);
        self.open = Some(id);
        let alive = self
            .state(id)
            .map(|state| state.vital == Vital::Alive)
            .unwrap_or(false);
        if let Ok(index) = self.member_index(id)
            && let Some(member) = self.members.get_mut(index)
        {
            if alive {
                member.participant.state.actions_remaining = 3;
                member.participant.state.reaction_ready = true;
                member.attack_count = 0;
            } else {
                member.participant.state.actions_remaining = 0;
                member.participant.state.reaction_ready = false;
            }
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

fn from_spend(error: EncounterError) -> ReactiveError {
    match error {
        EncounterError::UnknownCombatant => ReactiveError::UnknownCombatant,
        EncounterError::NotConscious => ReactiveError::NotConscious,
        EncounterError::NoReaction => ReactiveError::NoReaction,
        EncounterError::DuplicateCombatant
        | EncounterError::EmptyRoster
        | EncounterError::PerceptionOutOfRange
        | EncounterError::TurnInProgress
        | EncounterError::TurnNotOpen
        | EncounterError::NotYourTurn
        | EncounterError::InsufficientActions
        | EncounterError::InvalidActionCost
        | EncounterError::NoShield
        | EncounterError::NotRaised
        | EncounterError::Broken
        | EncounterError::Destroyed
        | EncounterError::NegativeAmount
        | EncounterError::TargetDead
        | EncounterError::TargetNotConscious
        | EncounterError::SelfTarget
        | EncounterError::NoLivingCombatant
        | EncounterError::AttackCountOverflow
        | EncounterError::RoundOverflow
        | EncounterError::AttackOverflow(_)
        | EncounterError::DamageOverflow(_)
        | EncounterError::HitPoints(_) => ReactiveError::NoReaction,
    }
}

fn as_damage_overflow(error: EncounterError) -> ReactiveError {
    match error {
        EncounterError::DamageOverflow(error) => ReactiveError::DamageOverflow(error),
        EncounterError::DuplicateCombatant
        | EncounterError::EmptyRoster
        | EncounterError::PerceptionOutOfRange
        | EncounterError::UnknownCombatant
        | EncounterError::TurnInProgress
        | EncounterError::TurnNotOpen
        | EncounterError::NotYourTurn
        | EncounterError::InsufficientActions
        | EncounterError::InvalidActionCost
        | EncounterError::NoReaction
        | EncounterError::NotConscious
        | EncounterError::TargetNotConscious
        | EncounterError::NoShield
        | EncounterError::NotRaised
        | EncounterError::Broken
        | EncounterError::Destroyed
        | EncounterError::NegativeAmount
        | EncounterError::TargetDead
        | EncounterError::SelfTarget
        | EncounterError::NoLivingCombatant
        | EncounterError::AttackCountOverflow
        | EncounterError::RoundOverflow
        | EncounterError::AttackOverflow(_)
        | EncounterError::HitPoints(_) => ReactiveError::DamageOverflow(ModifierError::Overflow),
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
    use crate::check::DegreeOfSuccess;
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
}
