//! Damage types and the immunity, weakness, and resistance step.
//!
//! The roll, the penalty floor, and critical doubling stay in [`crate::combatant::damage_dealt`].
//! This step runs after that roll and before Hit Points are reduced.
//!
//! Player Core pg. 407, Step 2: Damage Type. Player Core pg. 407, Step 3.
//! Player Core pg. 408, Immunity, Weakness, and Resistance.

use crate::modifiers::ModifierError;

/// One kind of damage an instance can carry.
///
/// The order is the order persistent flat checks use.
///
/// Player Core pg. 407, Damage Types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DamageType {
    /// A blunt physical blow. Player Core pg. 407, Physical Damage.
    Bludgeoning,
    /// A puncture. Player Core pg. 407, Physical Damage.
    Piercing,
    /// A cut. Player Core pg. 407, Physical Damage.
    Slashing,
    /// Bleeding. Player Core pg. 407, Bleed Damage.
    Bleed,
    /// Acid. Player Core pg. 407, Energy Damage.
    Acid,
    /// Cold. Player Core pg. 407, Energy Damage.
    Cold,
    /// Electricity. Player Core pg. 407, Energy Damage.
    Electricity,
    /// Fire. Player Core pg. 407, Energy Damage.
    Fire,
    /// Sonic. Player Core pg. 407, Energy Damage.
    Sonic,
    /// Force. Player Core pg. 407, Energy Damage.
    Force,
    /// Spirit. Player Core pg. 407, Spirit Damage.
    Spirit,
    /// Mental. Player Core pg. 407, Mental Damage.
    Mental,
    /// Poison. Player Core pg. 407, Poison Damage.
    Poison,
}

impl DamageType {
    /// Shield Block triggers on these three physical types only.
    ///
    /// Player Core pg. 262, Shield Block.
    pub(crate) const fn shield_block(self) -> bool {
        matches!(self, Self::Bludgeoning | Self::Piercing | Self::Slashing)
    }
}

/// Extra damage taken from one type. The value is at least 1.
///
/// Player Core pg. 408, Weakness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Weakness {
    /// The type that triggers this weakness. Player Core pg. 408, Weakness.
    pub damage_type: DamageType,
    /// Added once for this type. Player Core pg. 408, Weakness.
    pub value: i32,
}

/// Damage subtracted for one type. The value is at least 1.
///
/// Player Core pg. 408, Resistance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resistance {
    /// The type this resistance reduces. Player Core pg. 408, Resistance.
    pub damage_type: DamageType,
    /// Subtracted once for this type. Player Core pg. 408, Resistance.
    pub value: i32,
}

/// Immunities, weaknesses, and resistances copied onto a creature.
///
/// Player Core pg. 408, Immunity, Weakness, and Resistance.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DamageDefenses {
    /// Types this creature ignores. Player Core pg. 408, Immunity.
    pub immunities: Vec<DamageType>,
    /// Weaknesses. Two values of one type do not stack. Player Core pg. 408, Weakness.
    pub weaknesses: Vec<Weakness>,
    /// Resistances. Two values of one type do not stack. Player Core pg. 408, Resistance.
    pub resistances: Vec<Resistance>,
}

impl DamageDefenses {
    /// No immunity, weakness, or resistance.
    ///
    /// Player Core pg. 408, Immunity, Weakness, and Resistance.
    pub const fn empty() -> Self {
        Self {
            immunities: Vec::new(),
            weaknesses: Vec::new(),
            resistances: Vec::new(),
        }
    }

    /// Whether `damage_type` is listed. A duplicate listing is still immune once.
    ///
    /// Player Core pg. 408, Immunity.
    pub(crate) fn immune(&self, damage_type: DamageType) -> bool {
        self.immunities.contains(&damage_type)
    }
}

/// Immunity, then the highest weakness of that type, then the highest resistance, minimum 0.
///
/// A weakness or resistance of another type is ignored. Two values of one type are not summed.
/// The sum is checked in `i64`. A result that does not fit in `i32` is [`ModifierError::Overflow`].
/// This function does not roll.
///
/// Player Core pg. 407, Step 3. Player Core pg. 408, Weakness. Player Core pg. 408, Resistance.
pub(crate) fn adjust_damage(
    amount: i32,
    damage_type: DamageType,
    defenses: &DamageDefenses,
) -> Result<i32, ModifierError> {
    if defenses.immune(damage_type) {
        return Ok(0);
    }
    let weakness = highest(
        defenses
            .weaknesses
            .iter()
            .filter(|entry| entry.damage_type == damage_type)
            .map(|entry| entry.value),
    );
    let resistance = highest(
        defenses
            .resistances
            .iter()
            .filter(|entry| entry.damage_type == damage_type)
            .map(|entry| entry.value),
    );
    let total = i64::from(amount)
        .checked_add(weakness)
        .ok_or(ModifierError::Overflow)?;
    let total = total
        .checked_sub(resistance)
        .ok_or(ModifierError::Overflow)?;
    let total = total.max(0);
    i32::try_from(total).map_err(|_| ModifierError::Overflow)
}

/// Shield Block can trigger only for a positive physical amount the target is not immune to.
///
/// Player Core pg. 262, Shield Block. Player Core pg. 407, Step 3.
pub(crate) fn shield_trigger(damage_type: DamageType, defenses: &DamageDefenses) -> bool {
    damage_type.shield_block() && !defenses.immune(damage_type)
}

fn highest(values: impl Iterator<Item = i32>) -> i64 {
    values.map(i64::from).max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{DamageDefenses, DamageType, Resistance, Weakness, adjust_damage};
    use crate::combatant::damage_dealt;
    use crate::dice::ScriptedDice;
    use crate::modifiers::{Modifier, PenaltyType, SourceId, Statistic};

    fn defenses(
        immunities: &[DamageType],
        weaknesses: &[(DamageType, i32)],
        resistances: &[(DamageType, i32)],
    ) -> DamageDefenses {
        DamageDefenses {
            immunities: immunities.to_vec(),
            weaknesses: weaknesses
                .iter()
                .map(|(damage_type, value)| Weakness {
                    damage_type: *damage_type,
                    value: *value,
                })
                .collect(),
            resistances: resistances
                .iter()
                .map(|(damage_type, value)| Resistance {
                    damage_type: *damage_type,
                    value: *value,
                })
                .collect(),
        }
    }

    fn adjusted(amount: i32, damage_type: DamageType, defenses: &DamageDefenses) -> i32 {
        adjust_damage(amount, damage_type, defenses).expect("adjustment fits")
    }

    /// Player Core pg. 407, Damage Types.
    #[test]
    fn fire_and_slashing_are_different_instances() {
        let dice = ScriptedDice::new([8]);
        assert_ne!(DamageType::Fire, DamageType::Slashing);
        assert_eq!(adjusted(5, DamageType::Fire, &DamageDefenses::empty()), 5);
        assert_eq!(
            adjusted(5, DamageType::Slashing, &DamageDefenses::empty()),
            5
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 408, Immunity.
    #[test]
    fn immunity_to_fire_ignores_10_fire() {
        let defenses = defenses(&[DamageType::Fire], &[], &[]);
        assert_eq!(adjusted(10, DamageType::Fire, &defenses), 0);
    }

    /// Player Core pg. 407, Step 3. Player Core pg. 408, Immunity.
    #[test]
    fn immunity_beats_weakness_to_the_same_type() {
        let defenses = defenses(&[DamageType::Fire], &[(DamageType::Fire, 5)], &[]);
        assert_eq!(adjusted(10, DamageType::Fire, &defenses), 0);
    }

    /// Player Core pg. 408, Immunity.
    #[test]
    fn immunity_to_fire_does_not_affect_slashing() {
        let defenses = defenses(&[DamageType::Fire], &[], &[]);
        assert_eq!(adjusted(8, DamageType::Slashing, &defenses), 8);
    }

    /// Player Core pg. 408, Weakness.
    #[test]
    fn weakness_5_fire_raises_7_fire_to_12() {
        let defenses = defenses(&[], &[(DamageType::Fire, 5)], &[]);
        assert_eq!(adjusted(7, DamageType::Fire, &defenses), 12);
    }

    /// Player Core pg. 408, Weakness. DEV-62.
    #[test]
    fn two_fire_weaknesses_use_10_once() {
        let defenses = defenses(&[], &[(DamageType::Fire, 5), (DamageType::Fire, 10)], &[]);
        assert_eq!(adjusted(4, DamageType::Fire, &defenses), 14);
    }

    /// Player Core pg. 408, Weakness.
    #[test]
    fn a_slashing_weakness_does_not_raise_fire() {
        let defenses = defenses(&[], &[(DamageType::Slashing, 5)], &[]);
        assert_eq!(adjusted(7, DamageType::Fire, &defenses), 7);
    }

    /// Player Core pg. 408, Resistance. Player Core pg. 407, Step 3.
    #[test]
    fn weakness_then_resistance_leaves_3() {
        let defenses = defenses(&[], &[(DamageType::Fire, 5)], &[(DamageType::Fire, 10)]);
        assert_eq!(adjusted(8, DamageType::Fire, &defenses), 3);
    }

    /// Player Core pg. 408, Resistance.
    #[test]
    fn resistance_10_fire_reduces_8_fire_to_0() {
        let defenses = defenses(&[], &[], &[(DamageType::Fire, 10)]);
        assert_eq!(adjusted(8, DamageType::Fire, &defenses), 0);
    }

    /// Player Core pg. 408, Resistance. DEV-63.
    #[test]
    fn two_fire_resistances_use_10() {
        let defenses = defenses(&[], &[], &[(DamageType::Fire, 5), (DamageType::Fire, 10)]);
        assert_eq!(adjusted(12, DamageType::Fire, &defenses), 2);
    }

    /// Player Core pg. 408, Resistance.
    #[test]
    fn slashing_resistance_does_not_reduce_fire() {
        let defenses = defenses(&[], &[], &[(DamageType::Slashing, 10)]);
        assert_eq!(adjusted(8, DamageType::Fire, &defenses), 8);
    }

    /// Player Core pg. 406, Step 1: Roll Damage Dice. Player Core pg. 408, Resistance.
    #[test]
    fn a_floored_1_is_resisted_to_0() {
        let dice = ScriptedDice::new([8]);
        let penalty = Modifier::penalty(
            PenaltyType::Untyped,
            -6,
            SourceId(1),
            1,
            None,
            Statistic::Damage,
        )
        .expect("penalty");
        let rolled = damage_dealt(4, &[penalty], false).expect("floor");
        let defenses = defenses(&[], &[], &[(DamageType::Slashing, 5)]);
        assert_eq!(rolled, 1);
        assert_eq!(adjusted(rolled, DamageType::Slashing, &defenses), 0);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 407, Doubling and Halving. Player Core pg. 408, Weakness.
    #[test]
    fn a_critical_6_fire_with_weakness_5_deals_17() {
        let doubled = damage_dealt(6, &[], true).expect("double");
        let defenses = defenses(&[], &[(DamageType::Fire, 5)], &[]);
        assert_eq!(doubled, 12);
        assert_eq!(adjusted(doubled, DamageType::Fire, &defenses), 17);
    }

    /// Player Core pg. 407, Doubling and Halving. Player Core pg. 408, Resistance.
    #[test]
    fn a_critical_6_fire_with_resistance_5_deals_7() {
        let doubled = damage_dealt(6, &[], true).expect("double");
        let defenses = defenses(&[], &[], &[(DamageType::Fire, 5)]);
        assert_eq!(doubled, 12);
        assert_eq!(adjusted(doubled, DamageType::Fire, &defenses), 7);
    }

    /// Player Core pg. 407, Step 3.
    #[test]
    fn the_same_fire_adjustment_matches() {
        let dice = ScriptedDice::new([8]);
        let defenses = defenses(&[], &[(DamageType::Fire, 5)], &[(DamageType::Fire, 3)]);
        let first = adjusted(8, DamageType::Fire, &defenses);
        let second = adjusted(8, DamageType::Fire, &defenses);
        assert_eq!(first, 10);
        assert_eq!(second, 10);
        assert_eq!(dice.remaining(), 1);
    }
}
