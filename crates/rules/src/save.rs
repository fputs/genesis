//! Fortitude, Reflex, and Will.
//!
//! Player Core pg. 404, Saving Throws.

use std::ops::RangeInclusive;

use crate::check::{
    ArithmeticOverflow, CheckModifier, DcModifier, DegreeOfSuccess, dc_from_modifier,
};
use crate::modifiers::{Modifier, Statistic, reflex_modifiers, total};

/// Inclusive levels a save may use. Currently `1` through `25`.
///
/// The minimum is 1. Player characters are levels 1 through 20. Level −1 and level 0 are
/// out of scope. GM Core creature-building tables start at −1 and top out at 24. The
/// maximum is 25 because Treerazer is the highest published creature. This is the only
/// span `Level::new` reads.
///
/// Player Core pg. 91. GM Core pg. 112. Monster Core pg. 328.
pub const LEVEL_RANGE: RangeInclusive<i32> = 1..=25;

/// `level + 8` fits in `i32` for every value in [`LEVEL_RANGE`].
const _: () = assert!(*LEVEL_RANGE.end() <= i32::MAX - 8);

/// How much training a creature has in one save.
///
/// Untrained is +0 (DEV-56). The other ranks add the level and then 2, 4, 6, or 8.
///
/// Player Core pg. 404, Saving Throws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Proficiency {
    /// No proficiency bonus. Player Core pg. 404, Saving Throws.
    Untrained,
    /// Level + 2. Player Core pg. 404, Saving Throws.
    Trained,
    /// Level + 4. Player Core pg. 404, Saving Throws.
    Expert,
    /// Level + 6. Player Core pg. 404, Saving Throws.
    Master,
    /// Level + 8. Player Core pg. 404, Saving Throws.
    Legendary,
}

/// A level inside [`LEVEL_RANGE`].
///
/// Player Core pg. 404, Saving Throws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Level(i32);

/// Why a level was rejected.
///
/// Player Core pg. 404, Saving Throws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LevelError {
    /// The integer is outside [`LEVEL_RANGE`]. Player Core pg. 404, Saving Throws.
    OutOfRange,
}

impl Level {
    /// Accepts only [`LEVEL_RANGE`].
    ///
    /// Player Core pg. 404, Saving Throws.
    pub fn new(level: i32) -> Result<Self, LevelError> {
        if LEVEL_RANGE.contains(&level) {
            Ok(Self(level))
        } else {
            Err(LevelError::OutOfRange)
        }
    }

    /// The stored level.
    ///
    /// Player Core pg. 404, Saving Throws.
    pub fn get(self) -> i32 {
        self.0
    }
}

/// Attribute modifiers and proficiency for the three saves.
///
/// The attributes are modifiers, not scores. Player Core pg. 404, Saving Throws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SavingThrows {
    /// Level used for every proficiency bonus. Player Core pg. 404, Saving Throws.
    pub level: Level,
    /// Constitution modifier. Player Core pg. 404, Saving Throws.
    pub constitution: i32,
    /// Dexterity modifier. Player Core pg. 404, Saving Throws.
    pub dexterity: i32,
    /// Wisdom modifier. Player Core pg. 404, Saving Throws.
    pub wisdom: i32,
    /// Fortitude rank. Player Core pg. 404, Saving Throws.
    pub fortitude: Proficiency,
    /// Reflex rank. Player Core pg. 404, Saving Throws.
    pub reflex: Proficiency,
    /// Will rank. Player Core pg. 404, Saving Throws.
    pub will: Proficiency,
}

/// Which save is being rolled. Attack, Armor Class, Perception, and Damage are not saves.
///
/// Player Core pg. 404, Saving Throws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Save {
    /// Constitution save. Player Core pg. 404, Saving Throws.
    Fortitude,
    /// Dexterity save. Player Core pg. 404, Saving Throws.
    Reflex,
    /// Wisdom save. Player Core pg. 404, Saving Throws.
    Will,
}

impl Save {
    /// The statistic whose modifiers apply to this save.
    ///
    /// Player Core pg. 404, Saving Throws.
    pub fn statistic(self) -> Statistic {
        match self {
            Self::Fortitude => Statistic::Fortitude,
            Self::Reflex => Statistic::Reflex,
            Self::Will => Statistic::Will,
        }
    }
}

/// Why a save modifier or a basic-save amount was rejected.
///
/// Player Core pg. 404, Saving Throws. Player Core pg. 407, Doubling and Halving Damage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveError {
    /// The listed amount was negative. Player Core pg. 404, Basic Saving Throws.
    NegativeAmount,
    /// The attribute, proficiency, and modifiers did not fit a check modifier.
    ///
    /// Player Core pg. 404, Saving Throws.
    ModifierOverflow(ArithmeticOverflow),
    /// Doubling the listed amount did not fit in `i32`.
    ///
    /// Player Core pg. 407, Doubling and Halving Damage.
    DoubleOverflow(ArithmeticOverflow),
}

/// One basic save after the die and the multiplier.
///
/// Player Core pg. 404, Basic Saving Throws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SaveRoll {
    /// The d20. Player Core, Checks.
    pub natural: u8,
    /// Natural roll plus the save modifier. Player Core pg. 404, Saving Throws.
    pub total: i32,
    /// Degree after the natural 20 or natural 1 shift. Player Core pg. 401, Natural 1 and Natural 20.
    pub degree: DegreeOfSuccess,
    /// Damage after the basic-save multiplier. Player Core pg. 404, Basic Saving Throws.
    pub damage: i32,
}

/// Proficiency bonus for `rank` at `level`. Untrained is 0.
///
/// [`LEVEL_RANGE`]'s maximum plus 8 fits in `i32`, so the missing sum is not a rules result.
///
/// Player Core pg. 404, Saving Throws.
pub fn proficiency_bonus(level: Level, rank: Proficiency) -> i32 {
    let added = match rank {
        Proficiency::Untrained => return 0,
        Proficiency::Trained => 2,
        Proficiency::Expert => 4,
        Proficiency::Master => 6,
        Proficiency::Legendary => 8,
    };
    match level.get().checked_add(added) {
        Some(bonus) => bonus,
        // The const assert on `LEVEL_RANGE` keeps `level + 8` inside `i32`.
        None => level.get(),
    }
}

/// Attribute modifier, plus proficiency, plus modifiers on `save` only.
///
/// `unconscious` is true for dying and unconscious. Only Reflex then includes the
/// existing status penalty of −4. A modifier on another statistic is ignored.
/// Proficiency is not a bonus type.
///
/// Player Core pg. 404, Saving Throws. Player Core pg. 446, Unconscious.
pub fn save_modifier(
    training: &SavingThrows,
    save: Save,
    unconscious: bool,
    modifiers: &[Modifier],
) -> Result<CheckModifier, SaveError> {
    let (attribute, rank) = match save {
        Save::Fortitude => (training.constitution, training.fortitude),
        Save::Reflex => (training.dexterity, training.reflex),
        Save::Will => (training.wisdom, training.will),
    };
    let mut applicable: Vec<Modifier> = modifiers
        .iter()
        .copied()
        .filter(|modifier| modifier.statistic() == save.statistic())
        .collect();
    if unconscious && matches!(save, Save::Reflex) {
        let reflex = reflex_modifiers(true)
            .map_err(|_| SaveError::ModifierOverflow(ArithmeticOverflow::SumDoesNotFit))?;
        applicable.extend(reflex);
    }
    let stacked = total(&applicable)
        .map_err(|_| SaveError::ModifierOverflow(ArithmeticOverflow::SumDoesNotFit))?;
    let combined = attribute
        .checked_add(proficiency_bonus(training.level, rank))
        .and_then(|sum| sum.checked_add(stacked.get(save.statistic())))
        .ok_or(SaveError::ModifierOverflow(
            ArithmeticOverflow::SumDoesNotFit,
        ))?;
    CheckModifier::new(combined).map_err(SaveError::ModifierOverflow)
}

/// Saving throw DC: 10 plus the save modifier.
///
/// A [`CheckModifier`] always fits a DC modifier, so this does not fail.
///
/// Player Core pg. 404, Saving Throws. Player Core pg. 401, Getting a DC from a Modifier.
pub fn save_dc(modifier: CheckModifier) -> i32 {
    match DcModifier::new(modifier.get()) {
        Ok(modifier) => dc_from_modifier(modifier),
        // `CheckModifier` keeps `modifier + 255` inside `i32`, so `modifier + 10` fits too.
        Err(_) => modifier.get(),
    }
}

/// Basic-save damage for `degree`. Half of 1 stays 1. A result of 0 is not raised to 1.
///
/// Player Core pg. 404, Basic Saving Throws. Player Core pg. 407, Doubling and Halving Damage.
pub fn basic_save_amount(amount: i32, degree: DegreeOfSuccess) -> Result<i32, SaveError> {
    if amount < 0 {
        return Err(SaveError::NegativeAmount);
    }
    match degree {
        DegreeOfSuccess::CriticalSuccess => Ok(0),
        DegreeOfSuccess::Success => {
            if amount == 0 {
                Ok(0)
            } else {
                Ok((amount / 2).max(1))
            }
        }
        DegreeOfSuccess::Failure => Ok(amount),
        DegreeOfSuccess::CriticalFailure => amount
            .checked_mul(2)
            .ok_or(SaveError::DoubleOverflow(ArithmeticOverflow::SumDoesNotFit)),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Level, LevelError, Proficiency, Save, SavingThrows, basic_save_amount, proficiency_bonus,
        save_dc, save_modifier,
    };
    use crate::check::{DegreeOfSuccess, resolve_check};
    use crate::dice::ScriptedDice;
    use crate::modifiers::{BonusType, Modifier, SourceId, Statistic};

    fn level(value: i32) -> Level {
        Level::new(value).expect("level")
    }

    fn training(
        level_value: i32,
        constitution: i32,
        dexterity: i32,
        wisdom: i32,
        fortitude: Proficiency,
        reflex: Proficiency,
        will: Proficiency,
    ) -> SavingThrows {
        SavingThrows {
            level: level(level_value),
            constitution,
            dexterity,
            wisdom,
            fortitude,
            reflex,
            will,
        }
    }

    fn fortitude(level_value: i32, constitution: i32, rank: Proficiency) -> SavingThrows {
        training(
            level_value,
            constitution,
            0,
            0,
            rank,
            Proficiency::Untrained,
            Proficiency::Untrained,
        )
    }

    fn amount(listed: i32, degree: DegreeOfSuccess) -> i32 {
        basic_save_amount(listed, degree).expect("amount")
    }

    /// Player Core pg. 404, Basic Saving Throws. Player Core pg. 407, Doubling and Halving Damage.
    #[test]
    fn critical_success_deals_no_damage() {
        assert_eq!(amount(5, DegreeOfSuccess::CriticalSuccess), 0);
    }

    /// Player Core pg. 404, Basic Saving Throws. Player Core pg. 407, Doubling and Halving Damage.
    #[test]
    fn success_halves_5_to_2() {
        assert_eq!(amount(5, DegreeOfSuccess::Success), 2);
    }

    /// Player Core pg. 404, Basic Saving Throws. Player Core pg. 407, Doubling and Halving Damage.
    #[test]
    fn success_halves_7_to_3() {
        assert_eq!(amount(7, DegreeOfSuccess::Success), 3);
    }

    /// Player Core pg. 404, Basic Saving Throws. Player Core pg. 407, Doubling and Halving Damage.
    #[test]
    fn success_on_1_damage_stays_1() {
        assert_eq!(amount(1, DegreeOfSuccess::Success), 1);
    }

    /// Player Core pg. 404, Basic Saving Throws. Player Core pg. 407, Doubling and Halving Damage.
    #[test]
    fn success_on_0_damage_stays_0() {
        assert_eq!(amount(0, DegreeOfSuccess::Success), 0);
    }

    /// Player Core pg. 404, Basic Saving Throws.
    #[test]
    fn failure_deals_the_full_amount() {
        assert_eq!(amount(5, DegreeOfSuccess::Failure), 5);
    }

    /// Player Core pg. 404, Basic Saving Throws. Player Core pg. 407, Doubling and Halving Damage.
    #[test]
    fn critical_failure_doubles_once() {
        assert_eq!(amount(5, DegreeOfSuccess::CriticalFailure), 10);
    }

    /// Player Core pg. 404, Basic Saving Throws. Player Core pg. 407, Doubling and Halving Damage.
    #[test]
    fn critical_success_on_1_damage_deals_0() {
        assert_eq!(amount(1, DegreeOfSuccess::CriticalSuccess), 0);
    }

    /// Player Core pg. 404, Basic Saving Throws. Player Core pg. 407, Doubling and Halving Damage.
    #[test]
    fn critical_failure_on_0_damage_stays_0() {
        assert_eq!(amount(0, DegreeOfSuccess::CriticalFailure), 0);
    }

    /// Player Core pg. 404, Saving Throws.
    #[test]
    fn the_same_modifier_twice_matches() {
        let training = fortitude(1, 2, Proficiency::Trained);
        let first = save_modifier(&training, Save::Fortitude, false, &[]).expect("modifier");
        let second = save_modifier(&training, Save::Fortitude, false, &[]).expect("modifier");
        assert_eq!(first.get(), 5);
        assert_eq!(second.get(), 5);
    }

    /// Player Core pg. 404, Saving Throws. Player Core pg. 401, Getting a DC from a Modifier.
    #[test]
    fn a_modifier_of_plus_7_has_dc_17() {
        let training = fortitude(1, 2, Proficiency::Trained);
        let item = Modifier::bonus(
            BonusType::Item,
            1,
            SourceId(1),
            1,
            None,
            Statistic::Fortitude,
        )
        .expect("item");
        let circumstance = Modifier::bonus(
            BonusType::Circumstance,
            1,
            SourceId(2),
            2,
            None,
            Statistic::Fortitude,
        )
        .expect("circumstance");
        let modifier =
            save_modifier(&training, Save::Fortitude, false, &[item, circumstance]).expect("save");
        assert_eq!(modifier.get(), 7);
        assert_eq!(save_dc(modifier), 17);
        let mut dice = ScriptedDice::new([10]);
        let roll = resolve_check(&mut dice, modifier, save_dc(modifier));
        assert_eq!(roll.total, 17);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core pg. 404, Saving Throws.
    #[test]
    fn a_level_of_0_cannot_be_built() {
        assert_eq!(Level::new(0), Err(LevelError::OutOfRange));
    }

    /// Player Core pg. 91. GM Core pg. 112. Monster Core pg. 328.
    #[test]
    fn a_level_of_26_cannot_be_built() {
        assert_eq!(Level::new(26), Err(LevelError::OutOfRange));
    }

    /// Player Core pg. 91. GM Core pg. 112. Monster Core pg. 328.
    #[test]
    fn a_level_of_25_can_be_built() {
        let level = Level::new(25).expect("level");
        assert_eq!(level.get(), 25);
    }

    /// Player Core pg. 404, Saving Throws.
    #[test]
    fn legendary_at_level_25_is_plus_33_before_ability() {
        let level = Level::new(25).expect("level");
        assert_eq!(proficiency_bonus(level, Proficiency::Legendary), 33);
    }
}
