//! Aid rank, the DC 15 circumstance, and one check's bonus.
//!
//! Player Core, Aid. Player Core, "Step 4: Degree of Success".

use crate::check::{ArithmeticOverflow, CheckRoll, DegreeOfSuccess, degree_of_success};
use crate::combatant::CombatantId;
use crate::dice::Dice;

/// DC of an Aid check. Player Core, Aid.
const AID_DC: i32 = 15;

/// Highest face on the Aid die. The precheck is the modifier plus this face.
const AID_DIE: i32 = 20;

/// How much a critical success on Aid grants.
///
/// Below master is the book's +2. Master is +3. Legendary is +4. This is not a skill rank.
///
/// Player Core, Aid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AidRank {
    /// Critical success grants +2. Player Core, Aid.
    BelowMaster,
    /// Critical success grants +3. Player Core, Aid.
    Master,
    /// Critical success grants +4. Player Core, Aid.
    Legendary,
}

/// Circumstance modifier from one Aid check.
///
/// Positive values are circumstance bonuses. [`AidCircumstance::Minus1`] is a circumstance
/// penalty. [`AidCircumstance::Zero`] adds no modifier.
///
/// Player Core, Aid. Player Core, Bonuses. Player Core, Penalties.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AidCircumstance {
    /// +1 circumstance bonus. Player Core, Aid.
    Plus1,
    /// +2 circumstance bonus. Player Core, Aid.
    Plus2,
    /// +3 circumstance bonus. Player Core, Aid.
    Plus3,
    /// +4 circumstance bonus. Player Core, Aid.
    Plus4,
    /// No modifier. Player Core, Aid.
    Zero,
    /// −1 circumstance penalty. Player Core, Aid.
    Minus1,
}

impl AidCircumstance {
    /// The signed circumstance modifier: +1, +2, +3, +4, 0, or −1.
    ///
    /// Player Core, Aid. Player Core, Bonuses. Player Core, Penalties.
    pub const fn value(self) -> i32 {
        match self {
            Self::Plus1 => 1,
            Self::Plus2 => 2,
            Self::Plus3 => 3,
            Self::Plus4 => 4,
            Self::Zero => 0,
            Self::Minus1 => -1,
        }
    }
}

/// Whether a creature has prepared to Aid, and which ally that names.
///
/// `prepare_to_aid` names one ally. The preparation ends at that aider's next `start_turn`,
/// or when `aid` returns `Ok`, whichever comes first (DEV-50).
///
/// Player Core, Aid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AidPreparation {
    /// No ally is named. Player Core, Aid.
    NotPrepared,
    /// Prepared for one ally. Player Core, Aid.
    Prepared {
        /// The ally named by the last preparation. Player Core, Aid.
        ally: CombatantId,
    },
}

/// One resolved Aid check.
///
/// Player Core, Aid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AidOutcome {
    /// Natural roll plus the Aid modifier. Player Core, Aid.
    pub total: i32,
    /// Degree against DC 15, after a natural 20 or natural 1.
    ///
    /// Player Core, "Step 4: Degree of Success".
    pub degree: DegreeOfSuccess,
    /// Circumstance for this one check. Player Core, Aid.
    pub circumstance: AidCircumstance,
}

/// Circumstance from an Aid degree and rank.
///
/// Success is [`AidCircumstance::Plus1`], failure is [`AidCircumstance::Zero`], and a
/// critical failure is [`AidCircumstance::Minus1`]. Those three ignore `rank`. A critical
/// success is [`AidCircumstance::Plus2`] below master, [`AidCircumstance::Plus3`] at master,
/// and [`AidCircumstance::Plus4`] at legendary. This supersedes DEV-03.
///
/// Player Core, Aid. Player Core, "Step 4: Degree of Success".
pub(crate) fn aid_circumstance_bonus(degree: DegreeOfSuccess, rank: AidRank) -> AidCircumstance {
    match degree {
        DegreeOfSuccess::CriticalSuccess => match rank {
            AidRank::BelowMaster => AidCircumstance::Plus2,
            AidRank::Master => AidCircumstance::Plus3,
            AidRank::Legendary => AidCircumstance::Plus4,
        },
        DegreeOfSuccess::Success => AidCircumstance::Plus1,
        DegreeOfSuccess::Failure => AidCircumstance::Zero,
        DegreeOfSuccess::CriticalFailure => AidCircumstance::Minus1,
    }
}

/// An Aid modifier for which `modifier + 20` fits in `i32`.
///
/// [`FittingAidModifier::new`] is the only check of that bound.
///
/// Player Core, Aid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FittingAidModifier(i32);

impl FittingAidModifier {
    /// Rejects a modifier for which `modifier + 20` does not fit in `i32`.
    ///
    /// Player Core, Aid.
    pub(crate) fn new(modifier: i32) -> Result<Self, ArithmeticOverflow> {
        match modifier.checked_add(AID_DIE) {
            Some(_) => Ok(Self(modifier)),
            None => Err(ArithmeticOverflow::SumDoesNotFit),
        }
    }

    fn get(self) -> i32 {
        self.0
    }
}

/// Rolls one d20 Aid check for a modifier that fits with a face of 20.
///
/// A face outside `1..=20` whose sum does not fit is an error. It is not reported as that
/// face alone.
///
/// Player Core, Aid. Player Core, "Step 4: Degree of Success".
pub(crate) fn roll_aid(
    dice: &mut impl Dice,
    modifier: FittingAidModifier,
    rank: AidRank,
) -> Result<(CheckRoll, AidCircumstance), ArithmeticOverflow> {
    let natural = dice.d20();
    let total = match i32::from(natural).checked_add(modifier.get()) {
        Some(total) => total,
        None => return Err(ArithmeticOverflow::SumDoesNotFit),
    };
    let degree = degree_of_success(natural, total, AID_DC);
    let circumstance = aid_circumstance_bonus(degree, rank);
    Ok((
        CheckRoll {
            natural,
            total,
            dc: AID_DC,
            degree,
        },
        circumstance,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::{AttackTotalTerm, attack_check_total};
    use crate::combatant::{BuildParts, CharacterBuild, DieCount, MeleeStrike, ReachFeet, derive};
    use crate::dice::{Dice, Faces};
    use crate::modifiers::{
        BonusType, Modifier, PenaltyType, SourceId, Statistic, attack_modifiers,
    };

    fn ally() -> crate::combatant::Creature {
        let parts = BuildParts {
            hit_points: 20,
            armor_class: 18,
            perception: 0,
            speed_feet: 25,
            strike: MeleeStrike {
                attack_bonus: 0,
                damage_dice: DieCount::new(1).expect("dice"),
                damage_faces: Faces::new(8).expect("d8"),
                damage_bonus: 0,
                reach_feet: ReachFeet::new(5).expect("reach"),
                agile: false,
            },
            shield: None,
            stealth_dc: None,
            aid_modifier: None,
            aid_rank: None,
            uses_dying_rules: None,
            saves: None,
        };
        derive(&CharacterBuild::try_new(parts).expect("build"))
    }

    fn check_total(natural: u8, attack_bonus: i32, modifiers: &[Modifier]) -> i32 {
        attack_check_total(
            natural,
            AttackTotalTerm::new(attack_bonus).expect("bonus"),
            AttackTotalTerm::new(0).expect("map"),
            AttackTotalTerm::new(0).expect("circumstance"),
            AttackTotalTerm::new(0).expect("range"),
            modifiers,
        )
        .expect("total")
    }

    fn circumstance_bonus(value: i32, source: u32) -> Modifier {
        Modifier::bonus(
            BonusType::Circumstance,
            value,
            SourceId(source),
            source,
            None,
            Statistic::Attack,
        )
        .expect("bonus")
    }

    fn circumstance_penalty(value: i32, source: u32) -> Modifier {
        Modifier::penalty(
            PenaltyType::Circumstance,
            value,
            SourceId(source),
            source,
            None,
            Statistic::Attack,
        )
        .expect("penalty")
    }

    /// Player Core, Aid.
    /// Player Core, Bonuses.
    /// Player Core, Penalties.
    #[test]
    fn the_next_check_does_not_keep_the_bonus() {
        let ally = ally();
        let before = ally.clone();
        let aided = attack_modifiers(0, false, AidCircumstance::Plus1.value()).expect("aid");
        assert_eq!(check_total(10, ally.strike.attack_bonus, &aided), 11);
        let plain = attack_modifiers(0, false, 0).expect("no aid");
        assert_eq!(check_total(10, ally.strike.attack_bonus, &plain), 10);
        assert_eq!(ally.strike.attack_bonus, 0);
        assert_eq!(ally, before);
    }

    /// Player Core, Aid.
    /// Player Core, Bonuses.
    /// Player Core, Penalties.
    #[test]
    fn zero_adds_no_modifier() {
        let modifiers = attack_modifiers(0, false, AidCircumstance::Zero.value()).expect("aid");
        assert!(modifiers.is_empty());
        assert_eq!(check_total(10, 0, &modifiers), 10);
    }

    /// Player Core, Aid.
    /// Player Core, Bonuses.
    /// Player Core, Penalties.
    #[test]
    fn a_critical_failure_is_a_circumstance_penalty() {
        let modifiers = attack_modifiers(0, false, AidCircumstance::Minus1.value()).expect("aid");
        assert!(modifiers.iter().any(|modifier| {
            modifier.penalty_type() == Some(PenaltyType::Circumstance) && modifier.value() == -1
        }));
        assert_eq!(check_total(10, 0, &modifiers), 9);
    }

    /// Player Core, Aid.
    /// Player Core, Bonuses.
    /// Player Core, Penalties.
    #[test]
    fn the_worse_circumstance_penalty_wins() {
        let mut modifiers =
            attack_modifiers(0, false, AidCircumstance::Minus1.value()).expect("aid");
        modifiers.push(circumstance_penalty(-2, 21));
        assert_eq!(check_total(10, 0, &modifiers), 8);
    }

    /// Player Core, Aid.
    /// Player Core, Bonuses.
    /// Player Core, Penalties.
    #[test]
    fn plus_1_plus_a_caller_circumstance_plus_2_totals_12() {
        let mut modifiers =
            attack_modifiers(0, false, AidCircumstance::Plus1.value()).expect("aid");
        modifiers.push(circumstance_bonus(AidCircumstance::Plus2.value(), 21));
        assert_eq!(check_total(10, 0, &modifiers), 12);
    }

    /// Player Core, Aid.
    /// Player Core, Bonuses.
    /// Player Core, Penalties.
    #[test]
    fn plus_4_plus_a_caller_circumstance_plus_2_totals_14() {
        let mut modifiers =
            attack_modifiers(0, false, AidCircumstance::Plus4.value()).expect("aid");
        modifiers.push(circumstance_bonus(AidCircumstance::Plus2.value(), 21));
        assert_eq!(check_total(10, 0, &modifiers), 14);
    }

    /// Player Core, Aid.
    /// Player Core, Bonuses.
    /// Player Core, Penalties.
    #[test]
    fn two_aid_bonuses_keep_the_higher() {
        let mut modifiers =
            attack_modifiers(0, false, AidCircumstance::Plus2.value()).expect("aid");
        modifiers.push(circumstance_bonus(AidCircumstance::Plus1.value(), 21));
        assert_eq!(check_total(10, 0, &modifiers), 12);
    }

    /// Player Core, Aid.
    /// Player Core, Bonuses.
    /// Player Core, Penalties.
    #[test]
    fn a_circumstance_bonus_and_penalty_both_apply() {
        let mut modifiers =
            attack_modifiers(0, false, AidCircumstance::Plus2.value()).expect("aid");
        modifiers.push(circumstance_penalty(AidCircumstance::Minus1.value(), 21));
        assert_eq!(check_total(10, 0, &modifiers), 11);
    }

    /// A face above 20 is not a d20. Reporting that face as the total would be a different sum.
    #[test]
    fn a_face_past_20_is_an_error_not_the_natural() {
        struct WideDice;
        impl Dice for WideDice {
            fn roll(&mut self, _faces: Faces) -> u8 {
                u8::MAX
            }
        }
        let modifier = FittingAidModifier::new(i32::MAX - AID_DIE).expect("fits a d20");
        let error = roll_aid(&mut WideDice, modifier, AidRank::BelowMaster).expect_err("face");
        assert_eq!(error, ArithmeticOverflow::SumDoesNotFit);
    }
}
