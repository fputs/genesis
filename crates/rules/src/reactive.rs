//! Reactive Strike: one melee reaction inside the reactor's reach.
//!
//! The encounter spends the reaction. This module names the trigger, the reach errors,
//! and whether a critical hit disrupts a manipulate action.
//!
//! Player Core, "Reactive Strike".

use crate::check::{ArithmeticOverflow, AttackTotalTerm, DegreeOfSuccess, attack_check_total};
use crate::grid::{GridError, Size, Square, in_reach};
use crate::modifiers::{Modifier, ModifierError, Statistic, total};
use crate::positioning::{ActionCost, ActionTrait, MoveAction};

/// Which free action with the manipulate trait is being used.
///
/// Only [`ActionIdentity::Release`] is exempt. Any other manipulate action triggers.
///
/// Player Core, "Release". Player Core, "Reactive Strike".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionIdentity {
    /// Release, a free action with the manipulate trait that does not trigger.
    ///
    /// Player Core, "Release".
    Release,
    /// Any action that is not Release. Player Core, "Reactive Strike".
    Other,
}

/// Whether the caller grants this creature the reaction.
///
/// The class that grants it is proposed DEV-44. There is no class and no feat.
///
/// Player Core, "Reactive Strike".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReactiveStrike {
    /// The caller grants the reaction. Player Core, "Reactive Strike".
    Available,
    /// The caller does not grant the reaction. Nothing is spent. Player Core, "Reactive Strike".
    Unavailable,
}

/// Whether a critical hit disrupts the triggering manipulate action.
///
/// The caller voids that action's effects. This result does not refund its cost.
///
/// Player Core, "Reactive Strike". Player Core, "Disrupting Actions".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disruption {
    /// A critical hit on a cost that includes manipulate. Player Core, "Disrupting Actions".
    Disrupted,
    /// Every other result, including a critical hit on a move or a ranged attack.
    ///
    /// Player Core, "Reactive Strike".
    NotDisrupted,
}

/// One resolved Reactive Strike.
///
/// Player Core, "Reactive Strike".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReactiveOutcome {
    /// The melee Strike. The multiple attack penalty on it is 0.
    ///
    /// Player Core, "Reactive Strike". Player Core, Strike.
    pub strike: crate::encounter::StrikeDelivery,
    /// Whether the triggering manipulate action is disrupted.
    ///
    /// Player Core, "Disrupting Actions".
    pub disruption: Disruption,
}

/// Why a Reactive Strike was rejected.
///
/// [`ReactiveError::UnsupportedSize`], [`ReactiveError::Overflow`], and
/// [`ReactiveError::InvalidReach`] use positioning's names on purpose.
/// There is no `TargetDead` variant.
///
/// Player Core, "Reactive Strike".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReactiveError {
    /// The reactor or the actor is not in the encounter. Player Core, Encounter Mode.
    UnknownCombatant,
    /// The reactor's vital is not Alive. Player Core, "Step 2: Act".
    NotConscious,
    /// The reactor and the actor are the same creature. Player Core, "Reactive Strike".
    SelfTarget,
    /// The caller passed [`ReactiveStrike::Unavailable`]. Player Core, "Reactive Strike".
    Unavailable,
    /// Step, Release, or a cost with none of move, manipulate, or ranged.
    ///
    /// Player Core, "Step". Player Core, "Release". Player Core, "Seek".
    NotATrigger,
    /// A creature is not Small or Medium. Player Core, "Size, Space, and Reach".
    UnsupportedSize,
    /// A distance did not fit in `i32`, including when the reach is also not a multiple of 5.
    ///
    /// Player Core, "Range and Reach".
    Overflow,
    /// Reach is not a multiple of 5. Player Core, "Range and Reach".
    InvalidReach,
    /// The actor's square is outside the reactor's melee reach. Player Core, "Range and Reach".
    OutOfReach,
    /// The actor's vital is not Alive. Player Core, "Reactive Strike".
    TargetNotConscious,
    /// The reactor's reaction is not ready. Player Core, "Reactions in Encounters".
    NoReaction,
    /// The reactor has Delayed and cannot use a reaction yet.
    ///
    /// Checked after [`ReactiveError::NotConscious`] and before a reaction is spent.
    ///
    /// Player Core, Delay.
    Delayed,
    /// The attack bonus, the caller modifiers, and 20 do not fit in `i32`. This check is before the d20.
    ///
    /// Player Core, Penalties.
    AttackOverflow(ModifierError),
    /// The damage total did not fit. The flat-check die, if any, and both the attack die and the
    /// damage die were consumed. The reaction stays ready.
    ///
    /// Player Core, "Damage Rolls". Player Core pg. 442, Concealed.
    DamageOverflow(ModifierError),
    /// The actor is undetected, so it cannot be chosen. The reaction stays ready and no die is drawn.
    ///
    /// Player Core pg. 447, Undetected. Player Core, "Reactive Strike".
    CannotTarget,
    /// Applying the blow failed. Player Core, Hit Points.
    HitPoints(crate::combatant::HpError),
}

/// Whether the cost can trigger Reactive Strike.
///
/// A bool would hide Step, Release, and a cost with none of the trigger traits
/// behind one value. Callers match the variant.
///
/// Player Core, "Step". Player Core, "Release". Player Core, "Reactive Strike".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TriggerClass {
    /// Step, Release, or a cost that lacks move, manipulate, and ranged.
    NotATrigger,
    /// A move, manipulate, or ranged cost other than Step and Release.
    Triggers,
}

/// Step, Release, or a cost that lacks move, manipulate, and ranged.
///
/// Step is rejected before the trait slice is read. Release is the only manipulate
/// identity that is exempt.
///
/// Player Core, "Step". Player Core, "Release". Player Core, "Reactive Strike".
pub(crate) fn not_a_trigger(
    cost: ActionCost,
    identity: ActionIdentity,
    move_action: Option<MoveAction>,
) -> TriggerClass {
    if matches!(move_action, Some(MoveAction::Step)) {
        return TriggerClass::NotATrigger;
    }
    if matches!(identity, ActionIdentity::Release) {
        return TriggerClass::NotATrigger;
    }
    if traits(cost).iter().any(|name| {
        matches!(
            name,
            ActionTrait::Move | ActionTrait::Manipulate | ActionTrait::Ranged
        )
    }) {
        TriggerClass::Triggers
    } else {
        TriggerClass::NotATrigger
    }
}

/// Traits carried by an action cost, including a free action.
///
/// Player Core, "Reactive Strike".
pub(crate) fn traits(cost: ActionCost) -> &'static [ActionTrait] {
    match cost {
        ActionCost::Actions { traits, .. } | ActionCost::Free { traits } => traits,
    }
}

/// Reach errors in positioning's order, then the `in_reach` boolean.
///
/// `in_reach` is the only distance call. It reports an unsupported size and a distance
/// that does not fit before this function rejects a reach that is not a multiple of 5.
/// Reach 0 is a multiple of 5 and then fails the boolean.
///
/// Player Core, "Range and Reach". Player Core, "Size, Space, and Reach".
pub(crate) fn outside_reach(
    from: Square,
    to: Square,
    from_size: Size,
    to_size: Size,
    reach_feet: u32,
) -> Result<(), ReactiveError> {
    let reached = in_reach(from, to, from_size, to_size, reach_feet).map_err(from_grid)?;
    if !reach_feet.is_multiple_of(5) {
        return Err(ReactiveError::InvalidReach);
    }
    if !reached {
        return Err(ReactiveError::OutOfReach);
    }
    Ok(())
}

/// `true` when a critical hit disrupts a cost that includes manipulate.
///
/// Player Core, "Disrupting Actions".
pub(crate) fn disruption(degree: DegreeOfSuccess, cost: ActionCost) -> Disruption {
    let manipulates = traits(cost).contains(&ActionTrait::Manipulate);
    if degree == DegreeOfSuccess::CriticalSuccess && manipulates {
        Disruption::Disrupted
    } else {
        Disruption::NotDisrupted
    }
}

/// Whether `attack_bonus` plus the attack-statistic modifiers plus 20 fits in `i32`.
///
/// This is the check before the d20. A modifier fold that does not fit is the same error.
///
/// Player Core, Penalties.
pub(crate) fn attack_bonus_fits(
    attack_bonus: i32,
    modifiers: &[Modifier],
) -> Result<(), ModifierError> {
    let folded = i64::from(total(&attack_statistic(modifiers))?.get(Statistic::Attack));
    let sum = i64::from(attack_bonus) + folded + 20;
    i32::try_from(sum)
        .map(|_| ())
        .map_err(|_| ModifierError::Overflow)
}

/// Attack total with a multiple attack penalty of 0.
///
/// The bonus is an [`AttackTotalTerm`] when that term accepts it. A bonus such as
/// 2147483627 is too large for that term and still fits in `i32` once, so it is added
/// after [`attack_check_total`] folds the natural roll and the caller modifiers.
/// Agile is not an argument.
///
/// Player Core, "Reactive Strike". Player Core, "Multiple Attack Penalty".
pub(crate) fn reactive_attack_total(
    natural: u8,
    attack_bonus: i32,
    modifiers: &[Modifier],
) -> Result<i32, ModifierError> {
    let modifiers = attack_statistic(modifiers);
    let zero = match AttackTotalTerm::new(0) {
        Ok(term) => term,
        Err(ArithmeticOverflow::SumDoesNotFit) => return Err(ModifierError::Overflow),
    };
    match AttackTotalTerm::new(attack_bonus) {
        Ok(bonus) => attack_check_total(natural, bonus, zero, zero, zero, &modifiers),
        Err(ArithmeticOverflow::SumDoesNotFit) => {
            let base = attack_check_total(natural, zero, zero, zero, zero, &modifiers)?;
            i64::from(base)
                .checked_add(i64::from(attack_bonus))
                .and_then(|sum| i32::try_from(sum).ok())
                .ok_or(ModifierError::Overflow)
        }
    }
}

fn attack_statistic(modifiers: &[Modifier]) -> Vec<Modifier> {
    modifiers
        .iter()
        .copied()
        .filter(|modifier| modifier.statistic() == Statistic::Attack)
        .collect()
}

/// `in_reach` returns unsupported size or overflow. Each grid error has its own arm so a
/// new variant fails this match.
#[expect(
    clippy::match_same_arms,
    reason = "each GridError variant is named so a new one cannot collapse into Overflow"
)]
fn from_grid(error: GridError) -> ReactiveError {
    match error {
        GridError::UnsupportedSize => ReactiveError::UnsupportedSize,
        GridError::Overflow => ReactiveError::Overflow,
        GridError::ExceedsSpeed => ReactiveError::Overflow,
        GridError::SpeedNotMultipleOfFive => ReactiveError::Overflow,
        GridError::NegativeSpeed => ReactiveError::Overflow,
        GridError::OccupiedEnd => ReactiveError::Overflow,
        GridError::EnemySquare => ReactiveError::Overflow,
        GridError::StepIntoDifficultTerrain => ReactiveError::Overflow,
        GridError::StepRequiresTenFeet => ReactiveError::Overflow,
        GridError::StepNotOneSquare => ReactiveError::Overflow,
        GridError::PathNotAdjacent => ReactiveError::Overflow,
        GridError::OffGrid => ReactiveError::Overflow,
        GridError::Immobilized => ReactiveError::Overflow,
        GridError::InvalidRangeIncrement => ReactiveError::Overflow,
        GridError::NegativeDistance => ReactiveError::Overflow,
        GridError::BeyondRange => ReactiveError::Overflow,
    }
}

#[cfg(test)]
mod tests {
    use super::attack_bonus_fits;
    use crate::modifiers::ModifierError;

    /// The pre-roll gate is the bonus plus 20. `i32::MAX - 20` is the last value that fits.
    ///
    /// Player Core, Penalties.
    #[test]
    fn attack_bonus_plus_20_fits_only_through_i32_max_minus_20() {
        assert_eq!(
            attack_bonus_fits(i32::MAX - 19, &[]),
            Err(ModifierError::Overflow)
        );
        assert_eq!(attack_bonus_fits(i32::MAX - 20, &[]), Ok(()));
    }
}
