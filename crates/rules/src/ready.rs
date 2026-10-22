//! Ready: two actions, one trigger, one action resolved later as a reaction.
//!
//! The encounter stores the ready and spends the reaction. This module names the
//! trigger, the action, and the errors. It does not roll.
//!
//! Player Core, "Ready".

use crate::check::DegreeOfSuccess;
use crate::combatant::{CombatantId, HpError};
use crate::encounter::{Agility, StrikeProfile};
use crate::grid::{GridError, Size, Square, in_reach};
use crate::modifiers::Modifier;
use crate::positioning::{ActionCost, ActionTrait, PositioningError};
use crate::reactive::{ReactiveError, ReactiveStrike};

const CONCENTRATE: &[ActionTrait] = &[ActionTrait::Concentrate];

/// Where a readied Strike measures range.
///
/// Player Core, "Ready". Player Core, "Range and Reach".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrikeRange {
    /// Melee reach, checked with `in_reach`. Player Core, "Range and Reach".
    Melee,
    /// A ranged weapon's range increment in feet. Player Core, "Range Penalty".
    Ranged {
        /// Feet in one range increment. Player Core, "Range Penalty".
        increment_feet: i32,
    },
}

/// The one action Ready stores.
///
/// Delay and Return are named so a triggered free action is rejected as its own error.
///
/// Player Core, "Ready".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadyActivity {
    /// A Strike whose target is the trigger subject. Player Core, Strike.
    Strike {
        /// The trigger subject. Player Core, "Ready".
        target: CombatantId,
        /// Attack bonus, agile, and damage dice. Player Core, Strike.
        profile: StrikeProfile,
        /// Melee reach or a range increment. Player Core, "Range and Reach".
        range: StrikeRange,
    },
    /// A Stride along a path the caller supplies when the trigger is offered.
    ///
    /// Player Core, "Stride".
    Stride,
    /// A Step to one square the caller supplies. Player Core, "Step".
    Step,
    /// Dismiss a stored cover benefit. Player Core, "Take Cover".
    Dismiss,
    /// Delay already has the trigger "your turn begins". Player Core, Delay.
    Delay,
    /// Return from Delay already has a trigger. Player Core, Delay.
    ReturnFromDelay,
}

/// What the caller observed. `Other` is Seek or any action this set does not name.
///
/// Player Core, "Ready".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservedAction {
    /// The subject begins a Strike. Player Core, Strike.
    Strike,
    /// The subject begins a Stride. Player Core, "Stride".
    Stride,
    /// The subject begins a Step. Player Core, "Step".
    Step,
    /// The subject begins Drop Prone. Player Core, "Drop Prone".
    DropProne,
    /// The subject begins Stand. Player Core, "Stand".
    Stand,
    /// Seek, or any other unnamed action. Player Core, Seek.
    Other,
}

/// The trigger Ready stores.
///
/// Player Core, "Ready".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadyTrigger {
    /// The subject enters one square inside reach from a square outside it.
    ///
    /// Player Core, "Ready". Player Core, "Range and Reach".
    MovesWithinReach {
        /// The creature who moves. Player Core, "Ready".
        subject: CombatantId,
    },
    /// The subject begins one named action. `None` is Seek or any unnamed action.
    ///
    /// Player Core, "Ready".
    Begins {
        /// The creature who acts. Player Core, "Ready".
        subject: CombatantId,
        /// The action that fires this ready. `None` does not fire. Player Core, "Ready".
        action: Option<ObservedAction>,
    },
}

impl ReadyTrigger {
    /// The creature this trigger names.
    ///
    /// Player Core, "Ready".
    pub fn subject(self) -> CombatantId {
        match self {
            Self::MovesWithinReach { subject } | Self::Begins { subject, .. } => subject,
        }
    }
}

/// One stored ready: the trigger, the action, and the attack count at resolution.
///
/// Player Core, "Ready". Player Core, "Multiple Attack Penalty".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoredReady {
    /// When the action fires. Player Core, "Ready".
    pub trigger: ReadyTrigger,
    /// The action that fires. Player Core, "Ready".
    pub activity: ReadyActivity,
    /// Attack-trait checks already made when Ready resolved. Player Core, "Multiple Attack Penalty".
    pub frozen_attack_count: u8,
}

/// Numbers from a readied Strike. The attack count is not changed.
///
/// Player Core, "Ready". Player Core, "Multiple Attack Penalty".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadyStrike {
    /// Multiple attack penalty from the frozen count. Player Core, "Multiple Attack Penalty".
    pub map: i32,
    /// Untyped range penalty. 0 for melee. Player Core, "Range Penalty".
    pub range_penalty: i32,
    /// Natural roll plus attack bonus, penalty, and range penalty. Player Core, Strike.
    pub total: i32,
    /// Degree against the target's baked Armor Class. Player Core, "Degree of Success".
    pub degree: DegreeOfSuccess,
    /// Damage dealt, or 0 on a miss. Player Core, "Damage Rolls".
    pub damage: i32,
}

/// What a fired ready did. A move reports the distance the fresh counter produced.
///
/// Player Core, "Ready".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadySuccess {
    /// The Strike resolved. Player Core, Strike.
    Strike(ReadyStrike),
    /// The creature left its square. Player Core, "Stride".
    Move {
        /// Square occupied after the move. Player Core, "Grid Movement".
        end: Square,
        /// Feet the move cost. Player Core, "Stride".
        distance_feet: u32,
    },
    /// The stored cover benefit was dismissed. Player Core, "Take Cover".
    Dismiss,
}

/// One reaction `resolve_reactions` ran, in initiative order.
///
/// Player Core, "Ready". Player Core, "Reactive Strike".
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedReaction {
    /// A stored ready. Player Core, "Ready".
    Ready {
        /// The creature who readied. Player Core, "Ready".
        id: CombatantId,
        /// The fired action, or why it did not resolve. Player Core, "Ready".
        result: Result<ReadySuccess, ReadyError>,
    },
    /// A caller-supplied Reactive Strike. Nothing in the input is stored.
    ///
    /// Player Core, "Reactive Strike".
    ReactiveStrike {
        /// The reactor. Player Core, "Reactive Strike".
        id: CombatantId,
        /// Why the reaction was rejected, or that it resolved. Player Core, "Reactive Strike".
        result: Result<(), ReactiveError>,
    },
}

/// Caller-supplied Reactive Strike for one creature. The encounter does not store it.
///
/// Player Core, "Reactive Strike".
#[derive(Clone, Copy, Debug)]
pub struct ReactiveStrikeInput<'a> {
    /// The reactor. Player Core, "Reactive Strike".
    pub id: CombatantId,
    /// Whether the caller grants the reaction. Player Core, "Reactive Strike".
    pub availability: ReactiveStrike,
    /// The melee Strike. Player Core, Strike.
    pub profile: StrikeProfile,
    /// Attack-statistic modifiers. Player Core, Penalties.
    pub modifiers: &'a [Modifier],
    /// The reactor's size. Player Core, "Size, Space, and Reach".
    pub reactor_size: Size,
    /// The actor's size. Player Core, "Size, Space, and Reach".
    pub actor_size: Size,
    /// The reactor's square. The encounter does not keep it. Player Core, "Grid Movement".
    pub square: Square,
}

/// Why Ready or a readied action was rejected. Nothing is written.
///
/// Player Core, "Ready".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadyError {
    /// That id is not in the encounter. Player Core, Encounter Mode.
    UnknownCombatant,
    /// No turn is open. Player Core, "Step 2: Act".
    TurnNotOpen,
    /// The actor is not the active combatant. Player Core, "Step 2: Act".
    NotYourTurn,
    /// The combatant is not Alive. Player Core, "Step 2: Act".
    NotConscious,
    /// The readied cost is not one action, or Dismiss was not free. Player Core, "Ready".
    NotASingleAction,
    /// Delay and Return already have triggers. Player Core, "Ready".
    TriggeredFreeAction,
    /// The trigger names the readier. Player Core, "Ready".
    TriggerSubjectIsSelf,
    /// The Strike names someone other than the trigger subject. Player Core, "Ready".
    NotTheTriggerSubject,
    /// Fewer than 2 actions remain. Player Core, "Ready".
    InsufficientActions,
    /// Ending this turn would pass the maximum round. Player Core, "Step 2: Act".
    RoundOverflow,
    /// Nothing is stored. Player Core, "Ready".
    NotReadied,
    /// The stored action is not the one being checked. Player Core, "Ready".
    WrongAction,
    /// The readier has Delayed. Player Core, Delay.
    Delayed,
    /// The offer is not the stored trigger. Player Core, "Ready".
    TriggerMismatch,
    /// A melee Strike is outside reach. Player Core, "Range and Reach".
    OutOfReach,
    /// The Strike's target is dead. Player Core, Strike.
    TargetDead,
    /// The reaction is not ready. Player Core, "Reactions in Encounters".
    NoReaction,
    /// The path or the range measurement was rejected. Player Core, "Grid Movement".
    Grid(GridError),
    /// Dismiss returned a positioning error. Player Core, "Take Cover".
    Positioning(PositioningError),
    /// The attack total did not fit. Player Core, Penalties.
    AttackOverflow(crate::modifiers::ModifierError),
    /// The damage total did not fit. Player Core, "Damage Rolls".
    DamageOverflow(crate::modifiers::ModifierError),
    /// Applying Hit Points failed. Player Core, Hit Points.
    HitPoints(HpError),
}

/// Which stored action a fire check is comparing.
///
/// Player Core, "Ready".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ActivityKind {
    /// A Strike. Player Core, Strike.
    Strike,
    /// A Stride. Player Core, "Stride".
    Stride,
    /// A Step. Player Core, "Step".
    Step,
    /// Dismiss. Player Core, "Take Cover".
    Dismiss,
}

impl ReadyActivity {
    pub(crate) fn kind(self) -> Option<ActivityKind> {
        match self {
            Self::Strike { .. } => Some(ActivityKind::Strike),
            Self::Stride => Some(ActivityKind::Stride),
            Self::Step => Some(ActivityKind::Step),
            Self::Dismiss => Some(ActivityKind::Dismiss),
            Self::Delay | Self::ReturnFromDelay => None,
        }
    }
}

/// Ready costs 2 actions and has the concentrate trait.
///
/// Player Core, "Ready".
pub fn ready_cost() -> ActionCost {
    ActionCost::Actions {
        count: 2,
        traits: CONCENTRATE,
    }
}

/// Whether this cost is the one action Ready may store.
///
/// Dismiss is the free action. Every other stored action costs 1.
///
/// Player Core, "Ready".
pub(crate) fn single_action(activity: ReadyActivity, cost: ActionCost) -> Result<(), ReadyError> {
    if matches!(
        activity,
        ReadyActivity::Delay | ReadyActivity::ReturnFromDelay
    ) {
        return Err(ReadyError::TriggeredFreeAction);
    }
    let dismiss = matches!(activity, ReadyActivity::Dismiss);
    let fits = match cost {
        ActionCost::Free { .. } => dismiss,
        ActionCost::Actions { count, .. } => count == 1 && !dismiss,
    };
    if fits {
        Ok(())
    } else {
        Err(ReadyError::NotASingleAction)
    }
}

/// `true` when the entered square is inside reach and the square left was not.
///
/// A grid error is returned. A move that stays inside reach is `Ok(false)`.
///
/// Player Core, "Ready". Player Core, "Range and Reach".
#[expect(
    clippy::too_many_arguments,
    reason = "reach compares the trigger, the actor, the observed action, both squares, both sizes, and the reach"
)]
pub(crate) fn entered_reach(
    trigger: ReadyTrigger,
    actor: CombatantId,
    action: ObservedAction,
    from: Square,
    to: Square,
    readier: Square,
    readier_size: Size,
    actor_size: Size,
    reach_feet: u32,
) -> Result<bool, ReadyError> {
    if trigger.subject() != actor {
        return Ok(false);
    }
    match trigger {
        ReadyTrigger::MovesWithinReach { .. } => {
            let entered = in_reach(readier, to, readier_size, actor_size, reach_feet)
                .map_err(ReadyError::Grid)?;
            let left = in_reach(readier, from, readier_size, actor_size, reach_feet)
                .map_err(ReadyError::Grid)?;
            Ok(entered && !left)
        }
        ReadyTrigger::Begins { action: stored, .. } => Ok(stored == Some(action)),
    }
}

/// Agile from the stored Strike, for the readied penalty.
///
/// Player Core, "Multiple Attack Penalty".
pub(crate) fn stored_agility(activity: ReadyActivity) -> Option<Agility> {
    match activity {
        ReadyActivity::Strike { profile, .. } => Some(profile.agility),
        ReadyActivity::Stride
        | ReadyActivity::Step
        | ReadyActivity::Dismiss
        | ReadyActivity::Delay
        | ReadyActivity::ReturnFromDelay => None,
    }
}
