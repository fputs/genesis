//! Core conditions, their durations, and the modifiers they contribute.
//!
//! Player Core pg. 442, Appendix entry, Redundant Conditions.
//! Player Core pg. 436, Step 3: End Your Turn.
//! Player Core pg. 435, Step 1: Start Your Turn.

use crate::combatant::{CombatantId, ConditionSlot};
use crate::modifiers::{
    ConditionId, Modifier, ModifierError, ModifierSource, PenaltyType, Statistic,
};

/// Why a condition could not be stored.
///
/// Player Core pg. 442, Condition Values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConditionError {
    /// A valued condition was applied with no value.
    ///
    /// Player Core pg. 442, Condition Values.
    ValueRequired,
    /// A valued condition was applied at 0.
    ///
    /// Player Core pg. 442, Condition Values.
    EndedValue,
    /// Off-guard or prone was given a value, or asked to be reduced.
    ///
    /// Player Core pg. 442, Condition Values.
    ValueForbidden,
    /// A round duration started at 0.
    ///
    /// Player Core pg. 302, Durations.
    EmptyDuration,
}

/// One tracked condition name.
///
/// Player Core pg. 442, Conditions Appendix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConditionKind {
    /// Player Core pg. 444, Frightened.
    Frightened,
    /// Player Core pg. 446, Sickened.
    Sickened,
    /// Player Core pg. 445, Off-Guard.
    OffGuard,
    /// Player Core pg. 445, Prone.
    Prone,
    /// Player Core pg. 442, Clumsy.
    Clumsy,
    /// Player Core pg. 443, Enfeebled.
    Enfeebled,
}

/// How long one instance lasts.
///
/// Player Core pg. 302, Durations. Player Core pg. 436, Step 3: End Your Turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConditionDuration {
    /// Stays until removed or reduced to 0. Player Core pg. 442, Redundant Conditions.
    UntilRemoved,
    /// Counts down on `on`'s start. `remaining` is 1 or more.
    ///
    /// Player Core pg. 302, Durations.
    Rounds {
        /// Rounds still left, including the next start of `on`.
        remaining: u32,
        /// The creature whose start decreases this duration.
        on: CombatantId,
    },
    /// Ends at the end of `on`'s next turn once `due` is set.
    ///
    /// Player Core pg. 436, Step 3: End Your Turn.
    UntilEndOfNextTurn {
        /// The creature whose end can remove this instance.
        on: CombatantId,
        /// True when that creature's turn is not the one now open.
        due: bool,
    },
}

/// Which ability a Strike's attack roll uses.
///
/// Player Core pg. 442, Clumsy. Player Core pg. 443, Enfeebled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackBasis {
    /// A melee attack roll that uses Strength.
    StrengthMelee,
    /// A melee attack roll that uses Dexterity. Damage may still use Strength.
    DexterityMelee,
    /// A ranged attack roll. Damage in this change is not Strength.
    Ranged,
}

/// One stored condition. Fields stay crate-private so a literal cannot skip [`ConditionSlot::try_new`].
///
/// Player Core pg. 442, Condition Values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instance {
    kind: ConditionKind,
    value: Option<u32>,
    duration: ConditionDuration,
}

/// Whether a Strike's damage adds Strength.
///
/// Player Core pg. 443, Enfeebled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageBasis {
    /// Damage adds Strength, including a Dexterity melee Strike.
    Strength,
    /// Damage does not add Strength.
    NotStrength,
}

impl ConditionKind {
    fn valued(self) -> bool {
        matches!(
            self,
            Self::Frightened | Self::Sickened | Self::Clumsy | Self::Enfeebled
        )
    }

    fn condition_id(self) -> ConditionId {
        match self {
            Self::Frightened => ConditionId::FRIGHTENED,
            Self::Sickened => ConditionId::SICKENED,
            Self::OffGuard | Self::Prone => ConditionId::OFF_GUARD,
            Self::Clumsy => ConditionId::CLUMSY,
            Self::Enfeebled => ConditionId::ENFEEBLED,
        }
    }
}

impl ConditionSlot {
    /// Stores one instance. Persistent damage is not built here.
    ///
    /// Player Core pg. 442, Condition Values. Player Core pg. 302, Durations.
    pub fn try_new(
        kind: ConditionKind,
        value: Option<u32>,
        duration: ConditionDuration,
    ) -> Result<Self, ConditionError> {
        if let ConditionDuration::Rounds { remaining: 0, .. } = duration {
            return Err(ConditionError::EmptyDuration);
        }
        match (kind.valued(), value) {
            (true, None) => return Err(ConditionError::ValueRequired),
            (true, Some(0)) => return Err(ConditionError::EndedValue),
            (false, Some(_)) => return Err(ConditionError::ValueForbidden),
            (true, Some(_)) | (false, None) => {}
        }
        Ok(Self::Instance(Instance {
            kind,
            value,
            duration,
        }))
    }

    /// The instance name. Persistent damage has none.
    ///
    /// Player Core pg. 442, Redundant Conditions.
    pub fn kind(&self) -> Option<ConditionKind> {
        match self {
            Self::Instance(slot) => Some(slot.kind),
            Self::PersistentDamage { .. } => None,
        }
    }

    /// The instance value, when it has one. Persistent damage has none.
    ///
    /// Player Core pg. 442, Condition Values.
    pub fn value(&self) -> Option<u32> {
        match self {
            Self::Instance(slot) => slot.value,
            Self::PersistentDamage { .. } => None,
        }
    }

    /// The instance duration. Persistent damage has none.
    ///
    /// Player Core pg. 302, Durations.
    pub fn duration(&self) -> Option<ConditionDuration> {
        match self {
            Self::Instance(slot) => Some(slot.duration),
            Self::PersistentDamage { .. } => None,
        }
    }

    /// The condition id used when this instance emits a modifier.
    ///
    /// Prone's Armor Class penalty uses off-guard. Persistent damage has none.
    ///
    /// Player Core pg. 442, Appendix entry, Redundant Conditions.
    pub fn condition_id(&self) -> Option<ConditionId> {
        self.kind().map(ConditionKind::condition_id)
    }
}

/// Appends one instance. Persistent damage is left where it is.
///
/// Player Core pg. 442, Appendix entry, Redundant Conditions.
pub(crate) fn apply(list: &mut Vec<ConditionSlot>, slot: ConditionSlot) {
    if slot.kind().is_some() {
        list.push(slot);
    }
}

/// Subtracts `by` from every instance of `kind`.
///
/// Off-guard and prone are [`ConditionError::ValueForbidden`], including a reduction of 0.
/// A valued reduction of 0 changes nothing. An instance at 0, or a subtraction that does not
/// fit, ends that instance.
///
/// Player Core pg. 442, Appendix entry, Redundant Conditions With Values.
pub(crate) fn reduce(
    list: &mut Vec<ConditionSlot>,
    kind: ConditionKind,
    by: u32,
) -> Result<(), ConditionError> {
    if !kind.valued() {
        return Err(ConditionError::ValueForbidden);
    }
    if by == 0 {
        return Ok(());
    }
    let mut kept = Vec::with_capacity(list.len());
    for slot in list.drain(..) {
        match slot {
            ConditionSlot::Instance(Instance {
                kind: slot_kind,
                value: Some(value),
                duration,
            }) if slot_kind == kind => match value.checked_sub(by) {
                Some(left) if left >= 1 => kept.push(ConditionSlot::Instance(Instance {
                    kind,
                    value: Some(left),
                    duration,
                })),
                _ => {}
            },
            other => kept.push(other),
        }
    }
    *list = kept;
    Ok(())
}

/// Drops every instance of `kind`. Persistent damage stays.
///
/// Player Core pg. 442, Appendix entry, Redundant Conditions.
pub(crate) fn remove(list: &mut Vec<ConditionSlot>, kind: ConditionKind) {
    list.retain(|slot| slot.kind() != Some(kind));
}

/// Decreases round durations clocked to `clock` by 1. Remaining 0 ends the instance.
///
/// Player Core pg. 435, Step 1: Start Your Turn. Player Core pg. 302, Durations.
pub(crate) fn start_tick(list: &mut Vec<ConditionSlot>, clock: CombatantId) {
    let mut kept = Vec::with_capacity(list.len());
    for slot in list.drain(..) {
        match slot {
            ConditionSlot::Instance(Instance {
                kind,
                value,
                duration: ConditionDuration::Rounds { remaining, on },
            }) if on == clock => {
                if let Some(left) = remaining.checked_sub(1)
                    && left >= 1
                {
                    kept.push(ConditionSlot::Instance(Instance {
                        kind,
                        value,
                        duration: ConditionDuration::Rounds {
                            remaining: left,
                            on,
                        },
                    }));
                }
            }
            other => kept.push(other),
        }
    }
    *list = kept;
}

/// Removes a due until-end instance clocked to `clock`, then decreases frightened when
/// `bearer` is that clock.
///
/// Rules 2430 allows expiries in any order. This tick removes the due instance first.
/// The persistent hook already ran.
///
/// Player Core pg. 436, Step 3: End Your Turn. Player Core pg. 444, Frightened.
pub(crate) fn end_tick(list: &mut Vec<ConditionSlot>, bearer: CombatantId, clock: CombatantId) {
    let mut kept = Vec::with_capacity(list.len());
    for slot in list.drain(..) {
        match slot {
            ConditionSlot::Instance(Instance {
                kind,
                value,
                duration: ConditionDuration::UntilEndOfNextTurn { on, due },
            }) if on == clock => {
                if !due {
                    kept.push(ConditionSlot::Instance(Instance {
                        kind,
                        value,
                        duration: ConditionDuration::UntilEndOfNextTurn { on, due: true },
                    }));
                }
            }
            other => kept.push(other),
        }
    }
    *list = kept;
    if bearer == clock {
        let _ = reduce(list, ConditionKind::Frightened, 1);
    }
}

/// Modifiers for one statistic from the active instances.
///
/// `attack_basis` is used for [`Statistic::Attack`]. `damage_basis` is used for
/// [`Statistic::Damage`]. Level is none. The caller folds the result through [`crate::modifiers::total`].
///
/// Player Core pg. 400, Penalties. Player Core pg. 442, Appendix entry, Redundant Conditions.
pub(crate) fn condition_modifiers(
    list: &[ConditionSlot],
    statistic: Statistic,
    attack_basis: Option<AttackBasis>,
    damage_basis: Option<DamageBasis>,
) -> Result<Vec<Modifier>, ModifierError> {
    let mut modifiers = Vec::new();
    if checks_and_dcs(statistic)
        && let Some(value) = active_value(list, ConditionKind::Frightened)
    {
        modifiers.push(status_penalty(
            statistic,
            value,
            ConditionId::FRIGHTENED,
            ModifierSource::ConditionFrightened,
            20,
        )?);
    }
    if checks_and_dcs(statistic)
        && let Some(value) = active_value(list, ConditionKind::Sickened)
    {
        modifiers.push(status_penalty(
            statistic,
            value,
            ConditionId::SICKENED,
            ModifierSource::ConditionSickened,
            21,
        )?);
    }
    if clumsy_applies(statistic, attack_basis)
        && let Some(value) = active_value(list, ConditionKind::Clumsy)
    {
        modifiers.push(status_penalty(
            statistic,
            value,
            ConditionId::CLUMSY,
            ModifierSource::ConditionClumsy,
            22,
        )?);
    }
    if enfeebled_applies(statistic, attack_basis, damage_basis)
        && let Some(value) = active_value(list, ConditionKind::Enfeebled)
    {
        modifiers.push(status_penalty(
            statistic,
            value,
            ConditionId::ENFEEBLED,
            ModifierSource::ConditionEnfeebled,
            23,
        )?);
    }
    if statistic == Statistic::ArmorClass
        && (present(list, ConditionKind::OffGuard) || present(list, ConditionKind::Prone))
    {
        modifiers.push(circumstance_penalty(
            statistic,
            ConditionId::OFF_GUARD,
            ModifierSource::ConditionOffGuard,
            24,
        )?);
    }
    // Prone is −2 circumstance to attack rolls. It is not a penalty to a spell DC.
    if statistic == Statistic::Attack && present(list, ConditionKind::Prone) {
        modifiers.push(circumstance_penalty(
            statistic,
            ConditionId::PRONE,
            ModifierSource::ConditionProne,
            25,
        )?);
    }
    Ok(modifiers)
}

/// Prone's −2 circumstance penalty for one spell attack roll.
///
/// [`condition_modifiers`] does not emit this for [`Statistic::SpellAttack`], so a spell
/// DC built from that list does not take it. The returned modifier is tagged
/// [`Statistic::SpellAttack`] so it folds with the attack's other circumstance penalties.
///
/// Player Core pg. 445, Prone.
pub(crate) fn prone_spell_attack_penalty(
    list: &[ConditionSlot],
) -> Result<Option<Modifier>, ModifierError> {
    if !present(list, ConditionKind::Prone) {
        return Ok(None);
    }
    Ok(Some(circumstance_penalty(
        Statistic::SpellAttack,
        ConditionId::PRONE,
        ModifierSource::ConditionProne,
        25,
    )?))
}

fn checks_and_dcs(statistic: Statistic) -> bool {
    matches!(
        statistic,
        Statistic::Attack
            | Statistic::ArmorClass
            | Statistic::Perception
            | Statistic::Fortitude
            | Statistic::Reflex
            | Statistic::Will
            | Statistic::SpellAttack
    )
}

fn clumsy_applies(statistic: Statistic, attack_basis: Option<AttackBasis>) -> bool {
    match statistic {
        Statistic::ArmorClass | Statistic::Reflex => true,
        Statistic::Attack => matches!(
            attack_basis,
            Some(AttackBasis::DexterityMelee | AttackBasis::Ranged)
        ),
        Statistic::Perception
        | Statistic::Damage
        | Statistic::Fortitude
        | Statistic::Will
        | Statistic::SpellAttack => false,
        Statistic::Acrobatics
        | Statistic::Arcana
        | Statistic::Athletics
        | Statistic::Crafting
        | Statistic::Deception
        | Statistic::Diplomacy
        | Statistic::Intimidation
        | Statistic::Medicine
        | Statistic::Nature
        | Statistic::Occultism
        | Statistic::Performance
        | Statistic::Religion
        | Statistic::Society
        | Statistic::Stealth
        | Statistic::Survival
        | Statistic::Thievery => false,
    }
}

fn enfeebled_applies(
    statistic: Statistic,
    attack_basis: Option<AttackBasis>,
    damage_basis: Option<DamageBasis>,
) -> bool {
    match statistic {
        Statistic::Attack => matches!(attack_basis, Some(AttackBasis::StrengthMelee)),
        Statistic::Damage => matches!(damage_basis, Some(DamageBasis::Strength)),
        Statistic::ArmorClass
        | Statistic::Perception
        | Statistic::Fortitude
        | Statistic::Reflex
        | Statistic::Will
        | Statistic::SpellAttack => false,
        Statistic::Acrobatics
        | Statistic::Arcana
        | Statistic::Athletics
        | Statistic::Crafting
        | Statistic::Deception
        | Statistic::Diplomacy
        | Statistic::Intimidation
        | Statistic::Medicine
        | Statistic::Nature
        | Statistic::Occultism
        | Statistic::Performance
        | Statistic::Religion
        | Statistic::Society
        | Statistic::Stealth
        | Statistic::Survival
        | Statistic::Thievery => false,
    }
}

fn active_value(list: &[ConditionSlot], kind: ConditionKind) -> Option<u32> {
    list.iter()
        .filter_map(|slot| match slot {
            ConditionSlot::Instance(Instance {
                kind: slot_kind,
                value,
                ..
            }) if *slot_kind == kind => *value,
            _ => None,
        })
        .max()
}

fn present(list: &[ConditionSlot], kind: ConditionKind) -> bool {
    list.iter().any(|slot| slot.kind() == Some(kind))
}

fn status_penalty(
    statistic: Statistic,
    value: u32,
    id: ConditionId,
    source: ModifierSource,
    sequence: u32,
) -> Result<Modifier, ModifierError> {
    let magnitude = i32::try_from(value).map_err(|_| ModifierError::Overflow)?;
    let signed = magnitude.checked_neg().ok_or(ModifierError::Overflow)?;
    Modifier::penalty_from(
        PenaltyType::Status,
        signed,
        source,
        sequence,
        None,
        statistic,
    )?
    .with_condition(id, Some(value))
}

fn circumstance_penalty(
    statistic: Statistic,
    id: ConditionId,
    source: ModifierSource,
    sequence: u32,
) -> Result<Modifier, ModifierError> {
    Modifier::penalty_from(
        PenaltyType::Circumstance,
        -2,
        source,
        sequence,
        None,
        statistic,
    )?
    .with_condition(id, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::ScriptedDice;
    use crate::modifiers::{BonusType, ModifierTotal, PenaltyType, SourceId, total};

    fn id() -> CombatantId {
        CombatantId(1)
    }

    fn other() -> CombatantId {
        CombatantId(2)
    }

    fn frightened(value: u32) -> ConditionSlot {
        ConditionSlot::try_new(
            ConditionKind::Frightened,
            Some(value),
            ConditionDuration::UntilRemoved,
        )
        .expect("frightened")
    }

    fn sickened(value: u32) -> ConditionSlot {
        ConditionSlot::try_new(
            ConditionKind::Sickened,
            Some(value),
            ConditionDuration::UntilRemoved,
        )
        .expect("sickened")
    }

    fn off_guard() -> ConditionSlot {
        ConditionSlot::try_new(
            ConditionKind::OffGuard,
            None,
            ConditionDuration::UntilRemoved,
        )
        .expect("off-guard")
    }

    fn prone() -> ConditionSlot {
        ConditionSlot::try_new(ConditionKind::Prone, None, ConditionDuration::UntilRemoved)
            .expect("prone")
    }

    fn clumsy(value: u32) -> ConditionSlot {
        ConditionSlot::try_new(
            ConditionKind::Clumsy,
            Some(value),
            ConditionDuration::UntilRemoved,
        )
        .expect("clumsy")
    }

    fn enfeebled(value: u32) -> ConditionSlot {
        ConditionSlot::try_new(
            ConditionKind::Enfeebled,
            Some(value),
            ConditionDuration::UntilRemoved,
        )
        .expect("enfeebled")
    }

    fn store(slot: ConditionSlot) -> Vec<ConditionSlot> {
        let mut list = Vec::new();
        apply(&mut list, slot);
        list
    }

    fn stacked(list: &[ConditionSlot], statistic: Statistic) -> ModifierTotal {
        stacked_basis(list, statistic, None, None)
    }

    fn stacked_basis(
        list: &[ConditionSlot],
        statistic: Statistic,
        attack_basis: Option<AttackBasis>,
        damage_basis: Option<DamageBasis>,
    ) -> ModifierTotal {
        let modifiers =
            condition_modifiers(list, statistic, attack_basis, damage_basis).expect("modifiers");
        total(&modifiers).expect("total")
    }

    fn status_values(total: &ModifierTotal, statistic: Statistic) -> Vec<i32> {
        total
            .applied()
            .iter()
            .filter(|modifier| {
                modifier.statistic() == statistic
                    && modifier.penalty_type() == Some(PenaltyType::Status)
            })
            .map(|modifier| modifier.value())
            .collect()
    }

    fn circumstance_values(total: &ModifierTotal, statistic: Statistic) -> Vec<i32> {
        total
            .applied()
            .iter()
            .filter(|modifier| {
                modifier.statistic() == statistic
                    && modifier.penalty_type() == Some(PenaltyType::Circumstance)
            })
            .map(|modifier| modifier.value())
            .collect()
    }

    /// Player Core pg. 442, Appendix entry, Redundant Conditions.
    #[test]
    fn the_same_conditions_twice_match() {
        let dice = ScriptedDice::new([8]);
        let list = store(frightened(2));
        let first = condition_modifiers(
            &list,
            Statistic::Attack,
            Some(AttackBasis::StrengthMelee),
            None,
        )
        .expect("first");
        let second = condition_modifiers(
            &list,
            Statistic::Attack,
            Some(AttackBasis::StrengthMelee),
            None,
        )
        .expect("second");
        assert_eq!(first, second);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 442, Appendix entry, Redundant Conditions.
    #[test]
    fn frightened_2_is_stored_at_value_2() {
        let dice = ScriptedDice::new([8]);
        let list = store(frightened(2));
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].kind(), Some(ConditionKind::Frightened));
        assert_eq!(list[0].value(), Some(2));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 442, Appendix entry, Redundant Conditions.
    #[test]
    fn a_second_frightened_instance_stays_stored() {
        let mut list = store(frightened(1));
        apply(&mut list, frightened(2));
        assert_eq!(list.len(), 2);
        assert_eq!(active_value(&list, ConditionKind::Frightened), Some(2));
    }

    /// Player Core pg. 442, Condition Values.
    #[test]
    fn frightened_without_a_value_is_rejected() {
        let error = ConditionSlot::try_new(
            ConditionKind::Frightened,
            None,
            ConditionDuration::UntilRemoved,
        )
        .expect_err("missing");
        assert_eq!(error, ConditionError::ValueRequired);
    }

    /// Player Core pg. 442, Condition Values.
    #[test]
    fn frightened_0_is_rejected() {
        let error = ConditionSlot::try_new(
            ConditionKind::Frightened,
            Some(0),
            ConditionDuration::UntilRemoved,
        )
        .expect_err("zero");
        assert_eq!(error, ConditionError::EndedValue);
    }

    /// Player Core pg. 442, Condition Values.
    #[test]
    fn off_guard_with_a_value_is_rejected() {
        let error = ConditionSlot::try_new(
            ConditionKind::OffGuard,
            Some(1),
            ConditionDuration::UntilRemoved,
        )
        .expect_err("value");
        assert_eq!(error, ConditionError::ValueForbidden);
    }

    /// Player Core pg. 442, Condition Values.
    #[test]
    fn a_round_duration_of_0_is_rejected() {
        let error = ConditionSlot::try_new(
            ConditionKind::Sickened,
            Some(1),
            ConditionDuration::Rounds {
                remaining: 0,
                on: id(),
            },
        )
        .expect_err("rounds");
        assert_eq!(error, ConditionError::EmptyDuration);
    }

    /// Player Core pg. 442, Appendix entry, Redundant Conditions With Values.
    #[test]
    fn the_active_value_is_the_higher_frightened() {
        let mut list = store(frightened(1));
        apply(&mut list, frightened(3));
        assert_eq!(active_value(&list, ConditionKind::Frightened), Some(3));
        assert_eq!(list.len(), 2);
    }

    /// Player Core pg. 302, Durations.
    #[test]
    fn the_higher_frightened_ending_leaves_the_lower_one() {
        let mut list = Vec::new();
        apply(
            &mut list,
            ConditionSlot::try_new(
                ConditionKind::Frightened,
                Some(3),
                ConditionDuration::Rounds {
                    remaining: 1,
                    on: id(),
                },
            )
            .expect("rounds"),
        );
        apply(&mut list, frightened(1));
        start_tick(&mut list, id());
        assert!(list.iter().all(|slot| slot.value() != Some(3)));
        assert_eq!(active_value(&list, ConditionKind::Frightened), Some(1));
    }

    /// Player Core pg. 442, Appendix entry, Redundant Conditions With Values.
    #[test]
    fn reduce_lowers_every_frightened_instance() {
        let mut list = store(frightened(3));
        apply(&mut list, frightened(1));
        reduce(&mut list, ConditionKind::Frightened, 1).expect("reduce");
        let values: Vec<_> = list.iter().filter_map(ConditionSlot::value).collect();
        assert_eq!(values, vec![2]);
        assert_eq!(active_value(&list, ConditionKind::Frightened), Some(2));
    }

    /// Player Core pg. 442, Appendix entry, Redundant Conditions With Values.
    #[test]
    fn frightened_1_reduced_by_1_ends() {
        let mut list = store(frightened(1));
        reduce(&mut list, ConditionKind::Frightened, 1).expect("reduce");
        assert!(list.is_empty());
    }

    /// Player Core pg. 442, Appendix entry, Redundant Conditions With Values.
    #[test]
    fn reduce_on_an_absent_condition_changes_nothing() {
        let dice = ScriptedDice::new([8]);
        let mut list = store(sickened(1));
        reduce(&mut list, ConditionKind::Frightened, 1).expect("reduce");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].kind(), Some(ConditionKind::Sickened));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 442, Appendix entry, Redundant Conditions With Values.
    #[test]
    fn reduce_by_0_changes_nothing() {
        let mut list = store(frightened(2));
        reduce(&mut list, ConditionKind::Frightened, 0).expect("reduce");
        assert_eq!(active_value(&list, ConditionKind::Frightened), Some(2));
    }

    /// Player Core pg. 445, Off-Guard.
    #[test]
    fn off_guard_rejects_a_reduction() {
        let mut list = store(off_guard());
        let error = reduce(&mut list, ConditionKind::OffGuard, 1).expect_err("reduce");
        assert_eq!(error, ConditionError::ValueForbidden);
        assert_eq!(list.len(), 1);
    }

    /// Player Core pg. 445, Prone.
    #[test]
    fn prone_rejects_a_reduction() {
        let mut list = store(prone());
        let error = reduce(&mut list, ConditionKind::Prone, 1).expect_err("reduce");
        assert_eq!(error, ConditionError::ValueForbidden);
        assert_eq!(list.len(), 1);
    }

    /// Player Core pg. 442, Appendix entry, Redundant Conditions.
    #[test]
    fn remove_clears_frightened_and_leaves_sickened() {
        let mut list = store(frightened(2));
        apply(&mut list, sickened(1));
        remove(&mut list, ConditionKind::Frightened);
        assert!(active_value(&list, ConditionKind::Frightened).is_none());
        assert_eq!(active_value(&list, ConditionKind::Sickened), Some(1));
    }

    /// Player Core pg. 442, Appendix entry, Redundant Conditions.
    #[test]
    fn remove_on_an_absent_condition_changes_nothing() {
        let mut list = store(off_guard());
        remove(&mut list, ConditionKind::Prone);
        assert!(present(&list, ConditionKind::OffGuard));
    }

    /// Player Core pg. 444, Frightened. Player Core pg. 400, Penalties.
    #[test]
    fn frightened_2_and_sickened_1_are_a_status_penalty_of_minus_2() {
        let mut list = store(frightened(2));
        apply(&mut list, sickened(1));
        let total = stacked(&list, Statistic::Attack);
        assert_eq!(status_values(&total, Statistic::Attack), vec![-2]);
        assert_eq!(total.get(Statistic::Attack), -2);
    }

    /// Player Core pg. 444, Frightened.
    #[test]
    fn frightened_does_not_penalize_damage() {
        let list = store(frightened(2));
        let total = stacked(&list, Statistic::Damage);
        assert!(total.applied().is_empty());
    }

    /// Player Core pg. 400, Penalties.
    #[test]
    fn tracked_frightened_2_and_a_caller_frightened_1_are_minus_2() {
        let list = store(frightened(2));
        let mut modifiers = condition_modifiers(
            &list,
            Statistic::Attack,
            Some(AttackBasis::StrengthMelee),
            None,
        )
        .expect("tracked");
        let caller = Modifier::penalty(
            PenaltyType::Status,
            -1,
            SourceId(1),
            1,
            None,
            Statistic::Attack,
        )
        .expect("caller")
        .with_condition(ConditionId::FRIGHTENED, Some(1))
        .expect("condition");
        modifiers.push(caller);
        let total = total(&modifiers).expect("total");
        assert_eq!(status_values(&total, Statistic::Attack), vec![-2]);
        assert_eq!(total.get(Statistic::Attack), -2);
    }

    /// Player Core pg. 400, Bonuses. Player Core pg. 400, Penalties.
    #[test]
    fn a_status_bonus_and_frightened_both_apply() {
        let list = store(frightened(2));
        let mut modifiers = condition_modifiers(
            &list,
            Statistic::Attack,
            Some(AttackBasis::StrengthMelee),
            None,
        )
        .expect("tracked");
        modifiers.push(
            Modifier::bonus(
                BonusType::Status,
                1,
                SourceId(2),
                2,
                None,
                Statistic::Attack,
            )
            .expect("bonus"),
        );
        let total = total(&modifiers).expect("total");
        assert!(total.applied().iter().any(|modifier| {
            modifier.bonus_type() == Some(BonusType::Status) && modifier.value() == 1
        }));
        assert_eq!(status_values(&total, Statistic::Attack), vec![-2]);
        assert_eq!(total.get(Statistic::Attack), -1);
    }

    /// Player Core pg. 400, Penalties.
    #[test]
    fn frightened_and_off_guard_both_apply_to_armor_class() {
        let mut list = store(frightened(2));
        apply(&mut list, off_guard());
        let total = stacked(&list, Statistic::ArmorClass);
        assert_eq!(status_values(&total, Statistic::ArmorClass), vec![-2]);
        assert_eq!(circumstance_values(&total, Statistic::ArmorClass), vec![-2]);
        assert_eq!(total.get(Statistic::ArmorClass), -4);
    }

    /// Player Core pg. 445, Off-Guard.
    #[test]
    fn off_guard_is_minus_2_circumstance_to_armor_class() {
        let list = store(off_guard());
        let armor = stacked(&list, Statistic::ArmorClass);
        let attack = stacked_basis(
            &list,
            Statistic::Attack,
            Some(AttackBasis::StrengthMelee),
            None,
        );
        assert_eq!(circumstance_values(&armor, Statistic::ArmorClass), vec![-2]);
        assert!(attack.applied().is_empty());
    }

    /// Player Core pg. 445, Prone.
    #[test]
    fn prone_penalizes_attacks_and_armor_class() {
        let list = store(prone());
        let armor = stacked(&list, Statistic::ArmorClass);
        let attack = stacked_basis(
            &list,
            Statistic::Attack,
            Some(AttackBasis::StrengthMelee),
            None,
        );
        assert_eq!(circumstance_values(&armor, Statistic::ArmorClass), vec![-2]);
        assert_eq!(circumstance_values(&attack, Statistic::Attack), vec![-2]);
    }

    /// Player Core pg. 445, Prone. Player Core pg. 400, Penalties.
    #[test]
    fn prone_and_off_guard_penalize_armor_class_once() {
        let mut list = store(prone());
        apply(&mut list, off_guard());
        let total = stacked(&list, Statistic::ArmorClass);
        assert_eq!(circumstance_values(&total, Statistic::ArmorClass), vec![-2]);
        assert_eq!(total.get(Statistic::ArmorClass), -2);
    }

    /// Player Core pg. 442, Clumsy.
    #[test]
    fn clumsy_2_penalizes_reflex_and_a_ranged_attack() {
        let list = store(clumsy(2));
        let reflex = stacked(&list, Statistic::Reflex);
        let attack = stacked_basis(&list, Statistic::Attack, Some(AttackBasis::Ranged), None);
        assert_eq!(status_values(&reflex, Statistic::Reflex), vec![-2]);
        assert_eq!(status_values(&attack, Statistic::Attack), vec![-2]);
    }

    /// Player Core pg. 442, Clumsy.
    #[test]
    fn clumsy_2_does_not_penalize_a_strength_melee_attack() {
        let list = store(clumsy(2));
        let fortitude = stacked(&list, Statistic::Fortitude);
        let attack = stacked_basis(
            &list,
            Statistic::Attack,
            Some(AttackBasis::StrengthMelee),
            None,
        );
        assert!(fortitude.applied().is_empty());
        assert!(attack.applied().is_empty());
    }

    /// Player Core pg. 443, Enfeebled.
    #[test]
    fn enfeebled_2_penalizes_a_strength_melee_attack_and_its_damage() {
        let list = store(enfeebled(2));
        let attack = stacked_basis(
            &list,
            Statistic::Attack,
            Some(AttackBasis::StrengthMelee),
            None,
        );
        let damage = stacked_basis(&list, Statistic::Damage, None, Some(DamageBasis::Strength));
        assert_eq!(status_values(&attack, Statistic::Attack), vec![-2]);
        assert_eq!(status_values(&damage, Statistic::Damage), vec![-2]);
    }

    /// Player Core pg. 443, Enfeebled.
    #[test]
    fn enfeebled_2_does_not_penalize_armor_class() {
        let list = store(enfeebled(2));
        let armor = stacked(&list, Statistic::ArmorClass);
        let attack = stacked_basis(&list, Statistic::Attack, Some(AttackBasis::Ranged), None);
        assert!(armor.applied().is_empty());
        assert!(attack.applied().is_empty());
    }

    /// Player Core pg. 443, Enfeebled.
    #[test]
    fn enfeebled_2_lowers_strength_damage_on_a_dexterity_melee_attack() {
        let list = store(enfeebled(2));
        let attack = stacked_basis(
            &list,
            Statistic::Attack,
            Some(AttackBasis::DexterityMelee),
            None,
        );
        let damage = stacked_basis(&list, Statistic::Damage, None, Some(DamageBasis::Strength));
        assert!(attack.applied().is_empty());
        assert_eq!(status_values(&damage, Statistic::Damage), vec![-2]);
    }

    /// Player Core pg. 436, Step 3: End Your Turn. Player Core pg. 444, Frightened.
    #[test]
    fn the_end_tick_lowers_frightened_and_not_sickened() {
        let mut list = store(frightened(2));
        apply(&mut list, sickened(1));
        let hit_points = 12;
        end_tick(&mut list, id(), id());
        assert_eq!(active_value(&list, ConditionKind::Frightened), Some(1));
        assert_eq!(active_value(&list, ConditionKind::Sickened), Some(1));
        assert_eq!(hit_points, 12);
    }

    /// Player Core pg. 444, Frightened.
    #[test]
    fn frightened_1_ends_on_the_end_tick() {
        let mut list = store(frightened(1));
        end_tick(&mut list, id(), id());
        assert!(active_value(&list, ConditionKind::Frightened).is_none());
    }

    /// Player Core pg. 436, Step 3: End Your Turn.
    #[test]
    fn a_due_frightened_3_leaves_the_other_frightened_at_1() {
        let mut list = Vec::new();
        apply(
            &mut list,
            ConditionSlot::try_new(
                ConditionKind::Frightened,
                Some(3),
                ConditionDuration::UntilEndOfNextTurn {
                    on: id(),
                    due: true,
                },
            )
            .expect("due"),
        );
        apply(&mut list, frightened(2));
        end_tick(&mut list, id(), id());
        assert_eq!(list.len(), 1);
        assert_eq!(active_value(&list, ConditionKind::Frightened), Some(1));
    }

    /// Player Core pg. 436, Step 3: End Your Turn.
    #[test]
    fn an_end_that_is_not_yet_due_survives_one_tick() {
        let mut list = Vec::new();
        apply(
            &mut list,
            ConditionSlot::try_new(
                ConditionKind::OffGuard,
                None,
                ConditionDuration::UntilEndOfNextTurn {
                    on: id(),
                    due: false,
                },
            )
            .expect("not due"),
        );
        end_tick(&mut list, id(), id());
        assert!(present(&list, ConditionKind::OffGuard));
        end_tick(&mut list, id(), id());
        assert!(!present(&list, ConditionKind::OffGuard));
    }

    /// Player Core pg. 436, Step 3: End Your Turn.
    #[test]
    fn the_end_tick_leaves_persistent_damage() {
        let mut list = vec![ConditionSlot::PersistentDamage {
            damage_type: crate::damage::DamageType::Fire,
            amount: 5,
        }];
        apply(&mut list, frightened(2));
        let hit_points = 12;
        end_tick(&mut list, id(), id());
        assert_eq!(active_value(&list, ConditionKind::Frightened), Some(1));
        assert!(list.iter().any(|slot| matches!(
            slot,
            ConditionSlot::PersistentDamage {
                damage_type: crate::damage::DamageType::Fire,
                amount: 5,
            }
        )));
        assert_eq!(hit_points, 12);
    }

    /// Player Core pg. 436, Step 3: End Your Turn.
    #[test]
    fn a_due_end_removes_the_condition() {
        let mut list = Vec::new();
        apply(
            &mut list,
            ConditionSlot::try_new(
                ConditionKind::OffGuard,
                None,
                ConditionDuration::UntilEndOfNextTurn {
                    on: id(),
                    due: true,
                },
            )
            .expect("due"),
        );
        end_tick(&mut list, id(), id());
        assert!(!present(&list, ConditionKind::OffGuard));
    }

    /// Player Core pg. 435, Step 1: Start Your Turn.
    #[test]
    fn one_start_tick_lowers_3_rounds_to_2() {
        let mut list = Vec::new();
        apply(
            &mut list,
            ConditionSlot::try_new(
                ConditionKind::Sickened,
                Some(1),
                ConditionDuration::Rounds {
                    remaining: 3,
                    on: id(),
                },
            )
            .expect("rounds"),
        );
        start_tick(&mut list, id());
        match list[0].duration() {
            Some(ConditionDuration::Rounds { remaining: 2, .. }) => {}
            other => panic!("remaining {other:?}"),
        }
        assert_eq!(active_value(&list, ConditionKind::Sickened), Some(1));
    }

    /// Player Core pg. 435, Step 1: Start Your Turn.
    #[test]
    fn the_third_start_tick_ends_a_3_round_duration() {
        let mut list = Vec::new();
        apply(
            &mut list,
            ConditionSlot::try_new(
                ConditionKind::Sickened,
                Some(1),
                ConditionDuration::Rounds {
                    remaining: 3,
                    on: id(),
                },
            )
            .expect("rounds"),
        );
        start_tick(&mut list, id());
        start_tick(&mut list, id());
        start_tick(&mut list, id());
        assert!(list.is_empty());
    }

    /// Player Core pg. 302, Durations.
    #[test]
    fn another_creature_s_start_does_not_lower_that_duration() {
        let mut list = Vec::new();
        apply(
            &mut list,
            ConditionSlot::try_new(
                ConditionKind::Sickened,
                Some(1),
                ConditionDuration::Rounds {
                    remaining: 3,
                    on: id(),
                },
            )
            .expect("rounds"),
        );
        start_tick(&mut list, other());
        match list[0].duration() {
            Some(ConditionDuration::Rounds { remaining: 3, .. }) => {}
            other => panic!("remaining {other:?}"),
        }
        assert_eq!(active_value(&list, ConditionKind::Sickened), Some(1));
    }

    /// Player Core pg. 444, Frightened.
    #[test]
    fn the_start_tick_does_not_lower_frightened() {
        let mut list = store(frightened(2));
        start_tick(&mut list, id());
        assert_eq!(active_value(&list, ConditionKind::Frightened), Some(2));
    }
}
