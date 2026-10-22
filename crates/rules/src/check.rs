use crate::dice::Dice;
use crate::modifiers::{Modifier, ModifierError, Statistic, attack_check_parts, total};

/// Four degrees of success on a d20 check.
///
/// Player Core, "Step 4: Degree of Success".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DegreeOfSuccess {
    /// Missed the DC by 10 or more. Player Core, "Step 4: Degree of Success".
    CriticalFailure,
    /// Missed the DC by less than 10. Player Core, "Step 4: Degree of Success".
    Failure,
    /// Met the DC by less than 10. Player Core, "Step 4: Degree of Success".
    Success,
    /// Beat the DC by 10 or more. Player Core, "Step 4: Degree of Success".
    CriticalSuccess,
}

impl DegreeOfSuccess {
    /// One step toward critical success. A critical success stays put.
    ///
    /// Player Core, "Natural 1 and Natural 20".
    pub fn improve(self) -> Self {
        match self {
            DegreeOfSuccess::CriticalFailure => DegreeOfSuccess::Failure,
            DegreeOfSuccess::Failure => DegreeOfSuccess::Success,
            DegreeOfSuccess::Success => DegreeOfSuccess::CriticalSuccess,
            DegreeOfSuccess::CriticalSuccess => DegreeOfSuccess::CriticalSuccess,
        }
    }

    /// One step toward critical failure. A critical failure stays put.
    ///
    /// Player Core, "Natural 1 and Natural 20".
    pub fn worsen(self) -> Self {
        match self {
            DegreeOfSuccess::CriticalSuccess => DegreeOfSuccess::Success,
            DegreeOfSuccess::Success => DegreeOfSuccess::Failure,
            DegreeOfSuccess::Failure => DegreeOfSuccess::CriticalFailure,
            DegreeOfSuccess::CriticalFailure => DegreeOfSuccess::CriticalFailure,
        }
    }
}

/// Degree from `total` against `dc`, then a natural 20 improves it one step and a natural 1 worsens it one step.
///
/// The natural 20 or natural 1 is applied before any later effect calls [`DegreeOfSuccess::improve`] or [`DegreeOfSuccess::worsen`].
/// The 10-point bands use checked addition and subtraction. A DC at either end of `i32` has no
/// representable threshold on that side, so the band is not met.
///
/// Player Core, "Step 4: Degree of Success" and "Natural 1 and Natural 20".
pub fn degree_of_success(natural: u8, total: i32, dc: i32) -> DegreeOfSuccess {
    let ten_above = match dc.checked_add(10) {
        Some(threshold) => total >= threshold,
        None => false,
    };
    let ten_below = match dc.checked_sub(10) {
        Some(threshold) => total <= threshold,
        None => false,
    };
    let mut degree = if ten_above {
        DegreeOfSuccess::CriticalSuccess
    } else if total >= dc {
        DegreeOfSuccess::Success
    } else if ten_below {
        DegreeOfSuccess::CriticalFailure
    } else {
        DegreeOfSuccess::Failure
    };
    if natural == 20 {
        degree = degree.improve();
    } else if natural == 1 {
        degree = degree.worsen();
    }
    degree
}

/// Maps an unlisted critical success to success and an unlisted critical failure to failure.
///
/// Player Core, "Step 4: Degree of Success".
pub fn fallback_unlisted_critical(
    degree: DegreeOfSuccess,
    lists_critical_success: bool,
    lists_critical_failure: bool,
) -> DegreeOfSuccess {
    match degree {
        DegreeOfSuccess::CriticalSuccess if !lists_critical_success => DegreeOfSuccess::Success,
        DegreeOfSuccess::CriticalFailure if !lists_critical_failure => DegreeOfSuccess::Failure,
        other => other,
    }
}

/// A modifier for which `10 + modifier` fits in `i32`.
///
/// Player Core, "Getting a DC from a Modifier".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DcModifier(i32);

/// A modifier that can be added to any die face in `0..=255` without leaving `i32`.
///
/// Player Core, Checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckModifier(i32);

/// One addend of an attack total. Four values of this type, plus any `u8` natural roll, fit in `i32`.
///
/// Player Core, Penalties.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttackTotalTerm(i32);

/// The sum of the given numbers does not fit in `i32`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArithmeticOverflow {
    /// Checked addition has no `i32` result.
    SumDoesNotFit,
}

impl DcModifier {
    /// Rejects a modifier for which `10 + modifier` does not fit in `i32`.
    ///
    /// Player Core, "Getting a DC from a Modifier".
    pub fn new(modifier: i32) -> Result<Self, ArithmeticOverflow> {
        match 10i32.checked_add(modifier) {
            Some(_) => Ok(Self(modifier)),
            None => Err(ArithmeticOverflow::SumDoesNotFit),
        }
    }

    /// The modifier that was accepted.
    pub fn get(self) -> i32 {
        self.0
    }
}

impl CheckModifier {
    /// Rejects a modifier that cannot be added to every face in `0..=255`.
    ///
    /// Player Core, Checks.
    pub fn new(modifier: i32) -> Result<Self, ArithmeticOverflow> {
        match modifier.checked_add(i32::from(u8::MAX)) {
            Some(_) => Ok(Self(modifier)),
            None => Err(ArithmeticOverflow::SumDoesNotFit),
        }
    }

    /// The modifier that was accepted.
    pub fn get(self) -> i32 {
        self.0
    }
}

impl AttackTotalTerm {
    /// Rejects a term when four copies of it, plus a natural roll of 255, would leave `i32`.
    ///
    /// Player Core, Penalties.
    pub fn new(term: i32) -> Result<Self, ArithmeticOverflow> {
        let Some(quad) = term.checked_mul(4) else {
            return Err(ArithmeticOverflow::SumDoesNotFit);
        };
        match quad.checked_add(i32::from(u8::MAX)) {
            Some(_) => Ok(Self(term)),
            None => Err(ArithmeticOverflow::SumDoesNotFit),
        }
    }

    /// The term that was accepted.
    pub fn get(self) -> i32 {
        self.0
    }
}

/// DC taken from a modifier: 10 plus that modifier.
///
/// Player Core, "Getting a DC from a Modifier".
pub fn dc_from_modifier(modifier: DcModifier) -> i32 {
    match 10i32.checked_add(modifier.get()) {
        Some(dc) => dc,
        // `DcModifier::new` already rejected every modifier for which this sum does not fit.
        None => modifier.get(),
    }
}

/// One resolved d20 check. Player Core, Checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckRoll {
    /// Face rolled on the d20. Player Core, Checks.
    pub natural: u8,
    /// Natural roll plus the modifier. Player Core, Checks.
    pub total: i32,
    /// DC the total was compared with. Player Core, Checks.
    pub dc: i32,
    /// Degree after the natural 20 or natural 1 shift. Player Core, "Step 4: Degree of Success".
    pub degree: DegreeOfSuccess,
}

/// Rolls a d20, adds `modifier`, and applies [`degree_of_success`] (natural 20 and natural 1 included).
///
/// Player Core, Checks.
pub fn resolve_check(dice: &mut impl Dice, modifier: CheckModifier, dc: i32) -> CheckRoll {
    let natural = dice.d20();
    let total = match i32::from(natural).checked_add(modifier.get()) {
        Some(total) => total,
        // `CheckModifier` keeps every `u8` face plus the modifier inside `i32`.
        None => i32::from(natural),
    };
    let degree = degree_of_success(natural, total, dc);
    CheckRoll {
        natural,
        total,
        dc,
        degree,
    }
}

/// A flat check. `natural` is absent when the DC resolves without a roll.
///
/// Player Core, Flat Checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlatRoll {
    /// The d20, or nothing when the DC did not roll. Player Core, Flat Checks.
    pub natural: Option<u8>,
    /// DC that was compared. Player Core, Flat Checks.
    pub dc: i32,
    /// Degree of the flat check. Player Core, Flat Checks.
    pub degree: DegreeOfSuccess,
}

/// Rolls one bare d20 against `dc`. No modifier is added.
///
/// DC 1 or lower succeeds without a roll. DC 21 or higher fails without a roll.
/// Automatic results are an ordinary success or an ordinary failure.
///
/// Player Core, Flat Checks.
pub fn flat_check(dice: &mut impl Dice, dc: i32) -> FlatRoll {
    if dc <= 1 {
        return FlatRoll {
            natural: None,
            dc,
            degree: DegreeOfSuccess::Success,
        };
    }
    if dc >= 21 {
        return FlatRoll {
            natural: None,
            dc,
            degree: DegreeOfSuccess::Failure,
        };
    }
    let natural = dice.d20();
    FlatRoll {
        natural: Some(natural),
        dc,
        degree: degree_of_success(natural, i32::from(natural), dc),
    }
}

/// Several flat checks for the same thing roll once, at the highest DC.
///
/// `first_dc` plus `other_dcs` is a non-empty list. The highest DC is chosen with no empty-list result.
///
/// Player Core, Flat Checks.
pub fn flat_checks_for_same_thing(
    dice: &mut impl Dice,
    first_dc: i32,
    other_dcs: &[i32],
) -> FlatRoll {
    let dc = other_dcs.iter().copied().fold(first_dc, i32::max);
    flat_check(dice, dc)
}

/// Inputs for the untyped multiple attack penalty. The caller counts attack-trait checks.
///
/// Player Core, "Multiple Attack Penalty".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapInput {
    /// False on another creature's turn, including a reaction then. Player Core, "Multiple Attack Penalty".
    pub on_own_turn: bool,
    /// Attack-trait checks already made on this own turn. Player Core, "Multiple Attack Penalty".
    pub prior_attack_trait_checks_this_turn: u8,
    /// Agile on the weapon used for this check, not an earlier one. Player Core, "Multiple Attack Penalty".
    pub current_weapon_agile: bool,
    /// `Some(count)` is a readied attack. `count` is the attack-trait checks
    /// already made when Ready resolved, not a penalty. `None` is every other check.
    ///
    /// Player Core, "Multiple Attack Penalty". Player Core, "Ready".
    pub readied_attack_count: Option<u8>,
}

/// Untyped multiple attack penalty for this check.
///
/// On the creature's own turn the penalty is 0, then −5, then −10, or 0, then −4, then −8
/// when the current weapon is agile. A readied attack uses the attack count frozen when Ready
/// resolved, and that attack's weapon sets agile, including off the creature's own turn.
/// Any other check off turn has penalty 0. This does not increment the count.
///
/// Player Core, "Multiple Attack Penalty". Player Core, "Ready".
pub fn multiple_attack_penalty(input: MapInput) -> i32 {
    let count = match input.readied_attack_count {
        Some(count) => count,
        None if input.on_own_turn => input.prior_attack_trait_checks_this_turn,
        None => return 0,
    };
    let (second, later) = if input.current_weapon_agile {
        (-4, -8)
    } else {
        (-5, -10)
    };
    match count {
        0 => 0,
        1 => second,
        _ => later,
    }
}

/// Adds the natural roll, attack bonus, internal terms, and caller modifiers.
///
/// The internal terms and `caller` are folded together. A 0 internal argument is omitted,
/// because a modifier cannot be zero. The multiple attack penalty reuses its reserved source
/// and is applied once. It is not dropped by keeping only the worse of that penalty and the
/// circumstance penalty. The range penalty is the other untyped internal penalty. A positive
/// untyped internal term stays a raw addend. A reserved source does not skip same-type stacking.
/// An empty `caller` keeps the totals an attack check already had. A fold that does not fit in
/// `i32` is [`ModifierError::Overflow`], not the natural roll.
///
/// Player Core, Penalties. Player Core, "Multiple Attack Penalty". Player Core, "Range Penalty".
pub fn attack_check_total(
    natural: u8,
    attack_bonus: AttackTotalTerm,
    map: AttackTotalTerm,
    circumstance_penalty: AttackTotalTerm,
    range_penalty: AttackTotalTerm,
    caller: &[Modifier],
) -> Result<i32, ModifierError> {
    let parts = attack_check_parts(map.get(), circumstance_penalty.get(), range_penalty.get());
    let mut modifiers = parts.modifiers;
    modifiers.extend_from_slice(caller);
    let folded = i64::from(total(&modifiers)?.get(Statistic::Attack));
    let sum = i64::from(natural) + i64::from(attack_bonus.get()) + folded + parts.raw_extra;
    i32::try_from(sum).map_err(|_| ModifierError::Overflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::ScriptedDice;

    fn modifier(value: i32) -> CheckModifier {
        CheckModifier::new(value).expect("scenario modifier fits")
    }

    fn dc_modifier(value: i32) -> DcModifier {
        DcModifier::new(value).expect("scenario DC modifier fits")
    }

    fn term(value: i32) -> AttackTotalTerm {
        AttackTotalTerm::new(value).expect("scenario attack term fits")
    }

    /// Player Core, "Step 4: Degree of Success".
    /// Player Core, "Natural 1 and Natural 20".
    #[test]
    fn degrees_follow_dc_bands_then_natural_20_and_1() {
        assert_eq!(
            degree_of_success(10, 24, 14),
            DegreeOfSuccess::CriticalSuccess
        );
        assert_eq!(degree_of_success(11, 18, 18), DegreeOfSuccess::Success);
        assert_eq!(degree_of_success(8, 15, 18), DegreeOfSuccess::Failure);
        assert_eq!(
            degree_of_success(2, 4, 18),
            DegreeOfSuccess::CriticalFailure
        );

        // Total is a failure (above DC−10); the natural 20 improves it to a success.
        assert_eq!(degree_of_success(20, 31, 40), DegreeOfSuccess::Success);
        // Total is already a success; the natural 20 improves it to a critical success.
        assert_eq!(
            degree_of_success(20, 21, 18),
            DegreeOfSuccess::CriticalSuccess
        );
        // Total is a critical success; a natural 1 worsens it one step.
        assert_eq!(degree_of_success(1, 30, 18), DegreeOfSuccess::Success);
        // Total is a failure; a natural 1 worsens it to a critical failure.
        assert_eq!(
            degree_of_success(1, 12, 18),
            DegreeOfSuccess::CriticalFailure
        );

        let success = resolve_check(&mut ScriptedDice::new([11]), modifier(4), 15);
        assert_eq!(success.total, 15);
        assert_eq!(success.degree, DegreeOfSuccess::Success);

        let by_nine = resolve_check(&mut ScriptedDice::new([14]), modifier(10), 15);
        assert_eq!(by_nine.total, 24);
        assert_eq!(by_nine.degree, DegreeOfSuccess::Success);

        let by_ten = resolve_check(&mut ScriptedDice::new([15]), modifier(10), 15);
        assert_eq!(by_ten.total, 25);
        assert_eq!(by_ten.degree, DegreeOfSuccess::CriticalSuccess);

        let miss_nine = resolve_check(&mut ScriptedDice::new([6]), modifier(0), 15);
        assert_eq!(miss_nine.total, 6);
        assert_eq!(miss_nine.degree, DegreeOfSuccess::Failure);

        let miss_ten = resolve_check(&mut ScriptedDice::new([5]), modifier(0), 15);
        assert_eq!(miss_ten.total, 5);
        assert_eq!(miss_ten.degree, DegreeOfSuccess::CriticalFailure);

        let nat_20_failure = resolve_check(&mut ScriptedDice::new([20]), modifier(0), 25);
        assert_eq!(nat_20_failure.total, 20);
        assert_eq!(degree_of_success(10, 20, 25), DegreeOfSuccess::Failure);
        assert_eq!(nat_20_failure.degree, DegreeOfSuccess::Success);

        let nat_20_success = resolve_check(&mut ScriptedDice::new([20]), modifier(0), 18);
        assert_eq!(nat_20_success.total, 20);
        assert_eq!(nat_20_success.degree, DegreeOfSuccess::CriticalSuccess);

        let nat_20_already_crit = resolve_check(&mut ScriptedDice::new([20]), modifier(8), 18);
        assert_eq!(nat_20_already_crit.total, 28);
        assert_eq!(nat_20_already_crit.degree, DegreeOfSuccess::CriticalSuccess);

        let nat_20_from_crit_fail = resolve_check(&mut ScriptedDice::new([20]), modifier(0), 30);
        assert_eq!(nat_20_from_crit_fail.total, 20);
        assert_eq!(
            degree_of_success(10, 20, 30),
            DegreeOfSuccess::CriticalFailure
        );
        assert_eq!(nat_20_from_crit_fail.degree, DegreeOfSuccess::Failure);

        let nat_1_success = resolve_check(&mut ScriptedDice::new([1]), modifier(17), 15);
        assert_eq!(nat_1_success.total, 18);
        assert_eq!(degree_of_success(10, 18, 15), DegreeOfSuccess::Success);
        assert_eq!(nat_1_success.degree, DegreeOfSuccess::Failure);

        let nat_1_crit = resolve_check(&mut ScriptedDice::new([1]), modifier(24), 15);
        assert_eq!(nat_1_crit.total, 25);
        assert_eq!(nat_1_crit.degree, DegreeOfSuccess::Success);

        let nat_1_already_crit_fail = resolve_check(&mut ScriptedDice::new([1]), modifier(0), 15);
        assert_eq!(nat_1_already_crit_fail.total, 1);
        assert_eq!(
            nat_1_already_crit_fail.degree,
            DegreeOfSuccess::CriticalFailure
        );
    }

    /// Player Core, "Natural 1 and Natural 20". The shift is applied before a later upgrade.
    #[test]
    fn natural_20_is_applied_before_a_later_upgrade() {
        let nat_20 = resolve_check(&mut ScriptedDice::new([20]), modifier(0), 25);
        assert_eq!(
            degree_of_success(10, nat_20.total, 25),
            DegreeOfSuccess::Failure
        );
        assert_eq!(nat_20.degree, DegreeOfSuccess::Success);
        let upgraded = upgrade_success(nat_20.degree);
        assert_eq!(upgraded, DegreeOfSuccess::CriticalSuccess);

        let nat_1 = resolve_check(&mut ScriptedDice::new([1]), modifier(17), 15);
        assert_eq!(
            degree_of_success(10, nat_1.total, 15),
            DegreeOfSuccess::Success
        );
        assert_eq!(nat_1.degree, DegreeOfSuccess::Failure);
        assert_eq!(upgrade_success(nat_1.degree), DegreeOfSuccess::Failure);
    }

    /// A later effect that upgrades a success to a critical success, and leaves every other degree.
    fn upgrade_success(degree: DegreeOfSuccess) -> DegreeOfSuccess {
        if degree == DegreeOfSuccess::Success {
            degree.improve()
        } else {
            degree
        }
    }

    /// Player Core, "Step 4: Degree of Success".
    #[test]
    fn missing_critical_entry_falls_back() {
        let crit_success = resolve_check(&mut ScriptedDice::new([15]), modifier(10), 15);
        assert_eq!(crit_success.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(
            fallback_unlisted_critical(crit_success.degree, false, true),
            DegreeOfSuccess::Success
        );

        let crit_failure = resolve_check(&mut ScriptedDice::new([5]), modifier(0), 15);
        assert_eq!(crit_failure.degree, DegreeOfSuccess::CriticalFailure);
        assert_eq!(
            fallback_unlisted_critical(crit_failure.degree, true, false),
            DegreeOfSuccess::Failure
        );

        let listed_success = resolve_check(&mut ScriptedDice::new([15]), modifier(10), 15);
        let listed_failure = resolve_check(&mut ScriptedDice::new([5]), modifier(0), 15);
        assert_eq!(
            fallback_unlisted_critical(listed_success.degree, true, true),
            DegreeOfSuccess::CriticalSuccess
        );
        assert_eq!(
            fallback_unlisted_critical(listed_failure.degree, true, true),
            DegreeOfSuccess::CriticalFailure
        );
    }

    /// Player Core, "Getting a DC from a Modifier".
    #[test]
    fn dc_from_modifier_is_ten_plus() {
        assert_eq!(dc_from_modifier(dc_modifier(7)), 17);
        assert_eq!(dc_from_modifier(dc_modifier(0)), 10);
        assert_eq!(dc_from_modifier(dc_modifier(-3)), 7);

        let dc = dc_from_modifier(dc_modifier(7));
        let roll = resolve_check(&mut ScriptedDice::new([12]), modifier(5), dc);
        assert_eq!(roll.total, 17);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
    }

    /// Player Core, "Flat Checks".
    #[test]
    fn flat_check_ignores_modifiers() {
        let ignored_modifier = 9;
        let mut dice = ScriptedDice::new([10]);
        let roll = flat_check(&mut dice, 11);
        assert_eq!(roll.natural, Some(10));
        assert_eq!(roll.degree, DegreeOfSuccess::Failure);
        assert_eq!(
            degree_of_success(10, 10 + ignored_modifier, 11),
            DegreeOfSuccess::Success
        );
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, "Flat Checks".
    #[test]
    fn flat_checks_for_the_same_thing_roll_once() {
        let mut same = ScriptedDice::new([10, 20]);
        let combined = flat_checks_for_same_thing(&mut same, 5, &[15]);
        assert_eq!(combined.natural, Some(10));
        assert_eq!(combined.dc, 15);
        assert_eq!(combined.degree, DegreeOfSuccess::Failure);
        assert_eq!(same.remaining(), 1);
        assert_eq!(same.d20(), 20);

        let mut separate = ScriptedDice::new([10, 4]);
        let first = flat_check(&mut separate, 5);
        let second = flat_check(&mut separate, 11);
        assert_eq!(first.natural, Some(10));
        assert_eq!(first.degree, DegreeOfSuccess::Success);
        assert_eq!(second.natural, Some(4));
        assert_eq!(second.degree, DegreeOfSuccess::Failure);
        assert_eq!(separate.remaining(), 0);
    }

    /// Player Core, "Multiple Attack Penalty".
    #[test]
    fn map_is_zero_then_minus_five_then_minus_ten() {
        let penalty = |prior, agile, on_own_turn| {
            multiple_attack_penalty(MapInput {
                on_own_turn,
                prior_attack_trait_checks_this_turn: prior,
                current_weapon_agile: agile,
                readied_attack_count: None,
            })
        };

        assert_eq!(penalty(0, false, true), 0);
        assert_eq!(penalty(1, false, true), -5);
        assert_eq!(penalty(2, false, true), -10);
        assert_eq!(penalty(3, false, true), -10);

        assert_eq!(penalty(1, true, true), -4);
        assert_eq!(penalty(2, true, true), -8);
        assert_eq!(penalty(3, true, true), -8);
        assert_eq!(penalty(0, false, false), 0);
        assert_eq!(penalty(2, false, false), 0);

        let mut dice = ScriptedDice::new([10, 15, 15, 15, 14, 18, 12, 12, 10, 11, 15]);

        let first = attack_outcome(&mut dice, penalty(0, false, true), 8, 18);
        assert_eq!(first, (0, 18, DegreeOfSuccess::Success));

        let second = attack_outcome(&mut dice, penalty(1, false, true), 8, 18);
        assert_eq!(second, (-5, 18, DegreeOfSuccess::Success));

        let third = attack_outcome(&mut dice, penalty(2, false, true), 8, 18);
        assert_eq!(third, (-10, 13, DegreeOfSuccess::Failure));

        let fourth = attack_outcome(&mut dice, penalty(3, false, true), 8, 18);
        assert_eq!(fourth.0, -10);
        assert_eq!(fourth.1, 13);

        let second_agile = attack_outcome(&mut dice, penalty(1, true, true), 8, 18);
        assert_eq!(second_agile, (-4, 18, DegreeOfSuccess::Success));

        let third_agile = attack_outcome(&mut dice, penalty(2, true, true), 8, 18);
        assert_eq!(third_agile, (-8, 18, DegreeOfSuccess::Success));

        // One prior non-agile check; the current weapon is agile, so this check takes −4.
        let agile_now = attack_outcome(&mut dice, penalty(1, true, true), 8, 16);
        assert_eq!(agile_now, (-4, 16, DegreeOfSuccess::Success));

        // One prior agile check; the current weapon is not agile, so this check takes −5.
        let non_agile_now = attack_outcome(&mut dice, penalty(1, false, true), 8, 16);
        assert_eq!(non_agile_now, (-5, 15, DegreeOfSuccess::Failure));

        // A reaction on another creature's turn, after two own-turn checks, takes no penalty
        // and does not add to the count. The next own turn therefore starts at zero again.
        let reaction = attack_outcome(&mut dice, penalty(2, false, false), 8, 18);
        assert_eq!(reaction.0, 0);
        assert_eq!(reaction.1, 18);
        let next_own_turn = attack_outcome(&mut dice, penalty(0, false, true), 8, 19);
        assert_eq!(next_own_turn.0, 0);
        assert_eq!(next_own_turn.1, 19);

        // An attack-trait check that is not a Strike still counts, so the following Strike is the second.
        let after_non_strike = attack_outcome(&mut dice, penalty(1, false, true), 8, 19);
        assert_eq!(after_non_strike, (-5, 18, DegreeOfSuccess::Failure));
        assert_eq!(dice.remaining(), 0);
    }

    fn attack_outcome(
        dice: &mut ScriptedDice,
        map: i32,
        attack_bonus: i32,
        dc: i32,
    ) -> (i32, i32, DegreeOfSuccess) {
        let natural = dice.d20();
        let total = attack_check_total(
            natural,
            term(attack_bonus),
            term(map),
            term(0),
            term(0),
            &[],
        )
        .expect("empty caller list fits");
        (map, total, degree_of_success(natural, total, dc))
    }

    /// Player Core, "Penalties". MAP is untyped, so it is added to a circumstance penalty.
    #[test]
    fn map_stacks_with_a_circumstance_penalty() {
        let map = multiple_attack_penalty(MapInput {
            on_own_turn: true,
            prior_attack_trait_checks_this_turn: 1,
            current_weapon_agile: false,
            readied_attack_count: None,
        });
        assert_eq!(map, -5);
        let natural = ScriptedDice::new([12]).d20();
        let total = attack_check_total(natural, term(9), term(map), term(-2), term(0), &[])
            .expect("empty caller list fits");
        assert_eq!(total, 14);
        assert_eq!(
            degree_of_success(natural, total, 16),
            DegreeOfSuccess::Failure
        );
    }

    /// Player Core, "Penalties". MAP is added to every other untyped penalty.
    #[test]
    fn map_stacks_with_another_untyped_penalty() {
        let map = multiple_attack_penalty(MapInput {
            on_own_turn: true,
            prior_attack_trait_checks_this_turn: 1,
            current_weapon_agile: false,
            readied_attack_count: None,
        });
        assert_eq!(map, -5);
        let natural = ScriptedDice::new([12]).d20();
        let total = attack_check_total(natural, term(9), term(map), term(0), term(-2), &[])
            .expect("empty caller list fits");
        assert_eq!(total, 14);
        assert_eq!(
            degree_of_success(natural, total, 16),
            DegreeOfSuccess::Failure
        );
    }

    fn check(natural: u8, modifier_value: i32, dc: i32) -> CheckRoll {
        resolve_check(
            &mut ScriptedDice::new([natural]),
            modifier(modifier_value),
            dc,
        )
    }

    fn penalty(prior: u8, agile: bool, on_own_turn: bool) -> i32 {
        multiple_attack_penalty(MapInput {
            on_own_turn,
            prior_attack_trait_checks_this_turn: prior,
            current_weapon_agile: agile,
            readied_attack_count: None,
        })
    }

    fn strike_check(
        natural: u8,
        prior: u8,
        agile: bool,
        on_own_turn: bool,
        bonus: i32,
        dc: i32,
    ) -> (i32, i32, DegreeOfSuccess) {
        let mut dice = ScriptedDice::new([natural]);
        let rolled = dice.d20();
        let map = penalty(prior, agile, on_own_turn);
        let total = attack_check_total(rolled, term(bonus), term(map), term(0), term(0), &[])
            .expect("empty caller list fits");
        (map, total, degree_of_success(rolled, total, dc))
    }

    /// Player Core, "Step 4: Degree of Success".
    #[test]
    fn total_equal_to_the_dc_is_a_success() {
        let roll = check(11, 4, 15);
        assert_eq!(roll.total, 15);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
    }

    /// Player Core, "Step 4: Degree of Success".
    #[test]
    fn beating_the_dc_by_9_is_a_success() {
        let roll = check(14, 10, 15);
        assert_eq!(roll.total, 24);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
    }

    /// Player Core, "Step 4: Degree of Success".
    #[test]
    fn beating_the_dc_by_10_is_a_critical_success() {
        let roll = check(15, 10, 15);
        assert_eq!(roll.total, 25);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalSuccess);
    }

    /// Player Core, "Step 4: Degree of Success".
    #[test]
    fn missing_the_dc_by_9_is_a_failure() {
        let roll = check(6, 0, 15);
        assert_eq!(roll.total, 6);
        assert_eq!(roll.degree, DegreeOfSuccess::Failure);
    }

    /// Player Core, "Step 4: Degree of Success".
    #[test]
    fn missing_the_dc_by_10_is_a_critical_failure() {
        let roll = check(5, 0, 15);
        assert_eq!(roll.total, 5);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalFailure);
    }

    /// Player Core, "Natural 1 and Natural 20".
    #[test]
    fn natural_20_turns_a_failure_into_a_success() {
        let roll = check(20, 0, 25);
        assert_eq!(roll.total, 20);
        assert_eq!(degree_of_success(10, 20, 25), DegreeOfSuccess::Failure);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
    }

    /// Player Core, "Natural 1 and Natural 20".
    #[test]
    fn natural_20_turns_a_success_into_a_critical_success() {
        let roll = check(20, 0, 18);
        assert_eq!(roll.total, 20);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalSuccess);
    }

    /// Player Core, "Natural 1 and Natural 20".
    #[test]
    fn natural_20_does_not_raise_a_critical_success() {
        let roll = check(20, 8, 18);
        assert_eq!(roll.total, 28);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalSuccess);
    }

    /// Player Core, "Natural 1 and Natural 20".
    #[test]
    fn natural_20_turns_a_critical_failure_into_a_failure() {
        let roll = check(20, 0, 30);
        assert_eq!(roll.total, 20);
        assert_eq!(
            degree_of_success(10, 20, 30),
            DegreeOfSuccess::CriticalFailure
        );
        assert_eq!(roll.degree, DegreeOfSuccess::Failure);
    }

    /// Player Core, "Natural 1 and Natural 20".
    #[test]
    fn natural_1_turns_a_success_into_a_failure() {
        let roll = check(1, 17, 15);
        assert_eq!(roll.total, 18);
        assert_eq!(degree_of_success(10, 18, 15), DegreeOfSuccess::Success);
        assert_eq!(roll.degree, DegreeOfSuccess::Failure);
    }

    /// Player Core, "Natural 1 and Natural 20".
    #[test]
    fn natural_1_turns_a_critical_success_into_a_success() {
        let roll = check(1, 24, 15);
        assert_eq!(roll.total, 25);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
    }

    /// Player Core, "Natural 1 and Natural 20".
    #[test]
    fn natural_1_does_not_lower_a_critical_failure() {
        let roll = check(1, 0, 15);
        assert_eq!(roll.total, 1);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalFailure);
    }

    /// Player Core, "Natural 1 and Natural 20".
    #[test]
    fn a_later_upgrade_sees_the_degree_after_the_natural_20() {
        let roll = check(20, 0, 25);
        assert_eq!(
            degree_of_success(10, roll.total, 25),
            DegreeOfSuccess::Failure
        );
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(
            upgrade_success(roll.degree),
            DegreeOfSuccess::CriticalSuccess
        );
    }

    /// Player Core, "Natural 1 and Natural 20".
    #[test]
    fn a_later_upgrade_does_not_see_the_degree_from_before_the_natural_1() {
        let roll = check(1, 17, 15);
        assert_eq!(
            degree_of_success(10, roll.total, 15),
            DegreeOfSuccess::Success
        );
        assert_eq!(roll.degree, DegreeOfSuccess::Failure);
        assert_eq!(upgrade_success(roll.degree), DegreeOfSuccess::Failure);
    }

    /// Player Core, "Step 4: Degree of Success".
    #[test]
    fn no_critical_success_entry() {
        let roll = check(15, 10, 15);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(
            fallback_unlisted_critical(roll.degree, false, true),
            DegreeOfSuccess::Success
        );
    }

    /// Player Core, "Step 4: Degree of Success".
    #[test]
    fn no_critical_failure_entry() {
        let roll = check(5, 0, 15);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalFailure);
        assert_eq!(
            fallback_unlisted_critical(roll.degree, true, false),
            DegreeOfSuccess::Failure
        );
    }

    /// Player Core, "Step 4: Degree of Success".
    #[test]
    fn listed_critical_entries_stay() {
        let success = check(15, 10, 15);
        let failure = check(5, 0, 15);
        assert_eq!(
            fallback_unlisted_critical(success.degree, true, true),
            DegreeOfSuccess::CriticalSuccess
        );
        assert_eq!(
            fallback_unlisted_critical(failure.degree, true, true),
            DegreeOfSuccess::CriticalFailure
        );
    }

    /// Player Core, "Getting a DC from a Modifier".
    #[test]
    fn positive_modifier() {
        let dice = ScriptedDice::new([8]);
        assert_eq!(dc_from_modifier(dc_modifier(7)), 17);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Getting a DC from a Modifier".
    #[test]
    fn zero_modifier() {
        let dice = ScriptedDice::new([8]);
        assert_eq!(dc_from_modifier(dc_modifier(0)), 10);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Getting a DC from a Modifier".
    #[test]
    fn negative_modifier() {
        let dice = ScriptedDice::new([8]);
        assert_eq!(dc_from_modifier(dc_modifier(-3)), 7);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Getting a DC from a Modifier".
    #[test]
    fn a_check_uses_the_derived_dc() {
        let roll = check(12, 5, dc_from_modifier(dc_modifier(7)));
        assert_eq!(roll.total, 17);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
    }

    /// Player Core, "Flat Checks".
    #[test]
    fn modifiers_are_ignored() {
        let mut dice = ScriptedDice::new([10]);
        let roll = flat_check(&mut dice, 11);
        assert_eq!(roll.natural, Some(10));
        assert_eq!(roll.degree, DegreeOfSuccess::Failure);
        assert_eq!(degree_of_success(10, 19, 11), DegreeOfSuccess::Success);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, "Flat Checks".
    #[test]
    fn dc_1_succeeds_without_rolling() {
        let mut dice = ScriptedDice::new([4]);
        let roll = flat_check(&mut dice, 1);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(roll.natural, None);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Flat Checks".
    #[test]
    fn dc_0_succeeds_without_rolling() {
        let mut dice = ScriptedDice::new([4]);
        let roll = flat_check(&mut dice, 0);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(roll.natural, None);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Flat Checks".
    #[test]
    fn dc_21_fails_without_rolling() {
        let mut dice = ScriptedDice::new([20]);
        let roll = flat_check(&mut dice, 21);
        assert_eq!(roll.degree, DegreeOfSuccess::Failure);
        assert_eq!(roll.natural, None);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Flat Checks".
    ///
    /// Numeric total 20 meets DC 20. Natural 20 then improves that success.
    #[test]
    fn dc_20_still_rolls() {
        let mut dice = ScriptedDice::new([20]);
        let roll = flat_check(&mut dice, 20);
        assert_eq!(dice.remaining(), 0);
        assert_eq!(roll.natural, Some(20));
        assert_eq!(degree_of_success(10, 20, 20), DegreeOfSuccess::Success);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalSuccess);
    }

    /// Player Core, "Flat Checks" and "Natural 1 and Natural 20".
    #[test]
    fn dc_20_natural_1_is_a_critical_failure() {
        let mut dice = ScriptedDice::new([1]);
        let roll = flat_check(&mut dice, 20);
        assert_eq!(dice.remaining(), 0);
        assert_eq!(roll.natural, Some(1));
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalFailure);
    }

    /// Player Core, "Flat Checks".
    #[test]
    fn the_same_thing_rolls_once_at_the_highest_dc() {
        let mut dice = ScriptedDice::new([10, 20]);
        let roll = flat_checks_for_same_thing(&mut dice, 5, &[15]);
        assert_eq!(roll.natural, Some(10));
        assert_eq!(roll.dc, 15);
        assert_eq!(roll.degree, DegreeOfSuccess::Failure);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Flat Checks".
    #[test]
    fn different_things_roll_separately() {
        let mut dice = ScriptedDice::new([10, 4]);
        let first = flat_check(&mut dice, 5);
        let second = flat_check(&mut dice, 11);
        assert_eq!(first.natural, Some(10));
        assert_eq!(first.degree, DegreeOfSuccess::Success);
        assert_eq!(second.natural, Some(4));
        assert_eq!(second.degree, DegreeOfSuccess::Failure);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, "Multiple Attack Penalty".
    #[test]
    fn the_first_check_on_your_turn_has_no_penalty() {
        assert_eq!(
            strike_check(10, 0, false, true, 8, 18),
            (0, 18, DegreeOfSuccess::Success)
        );
    }

    /// Player Core, "Multiple Attack Penalty".
    #[test]
    fn the_second_non_agile_check_takes_minus_5() {
        assert_eq!(
            strike_check(15, 1, false, true, 8, 18),
            (-5, 18, DegreeOfSuccess::Success)
        );
    }

    /// Player Core, "Multiple Attack Penalty".
    #[test]
    fn the_third_and_later_non_agile_checks_take_minus_10() {
        assert_eq!(
            strike_check(15, 2, false, true, 8, 18),
            (-10, 13, DegreeOfSuccess::Failure)
        );
    }

    /// Player Core, "Multiple Attack Penalty".
    #[test]
    fn a_fourth_non_agile_check_stays_at_minus_10() {
        let (map, total, _) = strike_check(15, 3, false, true, 8, 18);
        assert_eq!(map, -10);
        assert_eq!(total, 13);
    }

    /// Player Core, "Multiple Attack Penalty".
    #[test]
    fn the_second_agile_check_takes_minus_4() {
        assert_eq!(
            strike_check(14, 1, true, true, 8, 18),
            (-4, 18, DegreeOfSuccess::Success)
        );
    }

    /// Player Core, "Multiple Attack Penalty".
    #[test]
    fn the_third_agile_check_takes_minus_8() {
        assert_eq!(
            strike_check(18, 2, true, true, 8, 18),
            (-8, 18, DegreeOfSuccess::Success)
        );
    }

    /// Player Core, "Multiple Attack Penalty".
    #[test]
    fn the_current_weapon_sets_the_penalty() {
        assert_eq!(
            strike_check(12, 1, true, true, 8, 16),
            (-4, 16, DegreeOfSuccess::Success)
        );
    }

    /// Player Core, "Multiple Attack Penalty".
    #[test]
    fn a_non_agile_weapon_after_an_agile_weapon_uses_minus_5() {
        assert_eq!(
            strike_check(12, 1, false, true, 8, 16),
            (-5, 15, DegreeOfSuccess::Failure)
        );
    }

    /// Player Core, "Multiple Attack Penalty".
    #[test]
    fn a_reaction_on_another_turn_takes_no_penalty_and_does_not_count() {
        let (map, total, _) = strike_check(10, 2, false, false, 8, 18);
        assert_eq!(map, 0);
        assert_eq!(total, 18);
        let (next_map, next_total, _) = strike_check(11, 0, false, true, 8, 19);
        assert_eq!(next_map, 0);
        assert_eq!(next_total, 19);
    }

    /// Player Core, "Multiple Attack Penalty".
    #[test]
    fn an_attack_trait_check_that_is_not_a_strike_still_counts() {
        assert_eq!(
            strike_check(15, 1, false, true, 8, 19),
            (-5, 18, DegreeOfSuccess::Failure)
        );
    }

    /// Player Core, "Multiple Attack Penalty".
    /// Player Core, "Ready".
    #[test]
    fn a_readied_non_agile_attack_after_one_check_takes_minus_5() {
        let input = MapInput {
            on_own_turn: false,
            prior_attack_trait_checks_this_turn: 1,
            current_weapon_agile: false,
            readied_attack_count: Some(1),
        };
        let penalty = multiple_attack_penalty(input);
        assert_eq!(penalty, multiple_attack_penalty(input));
        assert_ne!(penalty, -4);
        let mut dice = ScriptedDice::new([15]);
        assert_eq!(
            attack_outcome(&mut dice, penalty, 8, 18),
            (-5, 18, DegreeOfSuccess::Success)
        );
    }

    /// Player Core, "Multiple Attack Penalty".
    /// Player Core, "Ready".
    #[test]
    fn a_readied_attack_with_no_prior_check_takes_no_penalty() {
        let input = MapInput {
            on_own_turn: false,
            prior_attack_trait_checks_this_turn: 0,
            current_weapon_agile: false,
            readied_attack_count: Some(0),
        };
        let penalty = multiple_attack_penalty(input);
        assert_eq!(penalty, multiple_attack_penalty(input));
        let mut dice = ScriptedDice::new([10]);
        assert_eq!(
            attack_outcome(&mut dice, penalty, 8, 18),
            (0, 18, DegreeOfSuccess::Success)
        );
    }

    /// Player Core, "Multiple Attack Penalty".
    /// Player Core, "Ready".
    #[test]
    fn a_readied_agile_attack_after_two_checks_takes_minus_8() {
        let input = MapInput {
            on_own_turn: false,
            prior_attack_trait_checks_this_turn: 2,
            current_weapon_agile: true,
            readied_attack_count: Some(2),
        };
        let penalty = multiple_attack_penalty(input);
        assert_eq!(penalty, multiple_attack_penalty(input));
        assert_ne!(penalty, -10);
        let mut dice = ScriptedDice::new([18]);
        assert_eq!(
            attack_outcome(&mut dice, penalty, 8, 18),
            (-8, 18, DegreeOfSuccess::Success)
        );
    }

    /// Player Core, "Multiple Attack Penalty".
    /// Player Core, "Ready".
    #[test]
    fn a_readied_non_agile_attack_after_two_checks_takes_minus_10() {
        let input = MapInput {
            on_own_turn: false,
            prior_attack_trait_checks_this_turn: 2,
            current_weapon_agile: false,
            readied_attack_count: Some(2),
        };
        let penalty = multiple_attack_penalty(input);
        assert_eq!(penalty, multiple_attack_penalty(input));
        let mut dice = ScriptedDice::new([15]);
        assert_eq!(
            attack_outcome(&mut dice, penalty, 8, 18),
            (-10, 13, DegreeOfSuccess::Failure)
        );
    }

    /// Player Core, "Multiple Attack Penalty".
    /// Player Core, "Ready".
    #[test]
    fn a_readied_attack_leaves_the_next_own_turn_at_no_penalty() {
        let readied = MapInput {
            on_own_turn: false,
            prior_attack_trait_checks_this_turn: 1,
            current_weapon_agile: false,
            readied_attack_count: Some(1),
        };
        let next_own_turn = MapInput {
            on_own_turn: true,
            prior_attack_trait_checks_this_turn: 0,
            current_weapon_agile: false,
            readied_attack_count: None,
        };
        let readied_penalty = multiple_attack_penalty(readied);
        assert_eq!(readied_penalty, multiple_attack_penalty(readied));
        let next_penalty = multiple_attack_penalty(next_own_turn);
        assert_eq!(next_penalty, multiple_attack_penalty(next_own_turn));
        let mut dice = ScriptedDice::new([15, 10]);
        let (map, total, _) = attack_outcome(&mut dice, readied_penalty, 8, 18);
        assert_eq!(map, -5);
        assert_eq!(total, 18);
        let (map, total, _) = attack_outcome(&mut dice, next_penalty, 8, 18);
        assert_eq!(map, 0);
        assert_eq!(total, 18);
    }

    /// Player Core, "Multiple Attack Penalty".
    /// Player Core, "Ready".
    #[test]
    fn a_reactive_strike_on_the_same_round_takes_no_penalty() {
        let input = MapInput {
            on_own_turn: false,
            prior_attack_trait_checks_this_turn: 2,
            current_weapon_agile: false,
            readied_attack_count: None,
        };
        let penalty = multiple_attack_penalty(input);
        assert_eq!(penalty, multiple_attack_penalty(input));
        let mut dice = ScriptedDice::new([10]);
        let (map, total, _) = attack_outcome(&mut dice, penalty, 8, 18);
        assert_eq!(map, 0);
        assert_eq!(total, 18);
    }

    /// Player Core, "Multiple Attack Penalty".
    /// Player Core, "Ready".
    #[test]
    fn a_readied_non_agile_attack_at_a_frozen_count_of_3_takes_minus_10() {
        let input = MapInput {
            on_own_turn: false,
            prior_attack_trait_checks_this_turn: 3,
            current_weapon_agile: false,
            readied_attack_count: Some(3),
        };
        let penalty = multiple_attack_penalty(input);
        assert_eq!(penalty, multiple_attack_penalty(input));
        let mut dice = ScriptedDice::new([15]);
        assert_eq!(
            attack_outcome(&mut dice, penalty, 8, 18),
            (-10, 13, DegreeOfSuccess::Failure)
        );
    }

    /// Player Core, "Multiple Attack Penalty".
    /// Player Core, "Ready".
    #[test]
    fn a_readied_agile_attack_at_a_frozen_count_of_3_takes_minus_8() {
        let input = MapInput {
            on_own_turn: false,
            prior_attack_trait_checks_this_turn: 3,
            current_weapon_agile: true,
            readied_attack_count: Some(3),
        };
        let penalty = multiple_attack_penalty(input);
        assert_eq!(penalty, multiple_attack_penalty(input));
        let mut dice = ScriptedDice::new([18]);
        assert_eq!(
            attack_outcome(&mut dice, penalty, 8, 18),
            (-8, 18, DegreeOfSuccess::Success)
        );
    }

    /// Player Core, "Multiple Attack Penalty".
    /// Player Core, "Ready".
    #[test]
    fn a_readied_attack_at_a_frozen_count_of_255_stays_at_the_later_penalty() {
        let non_agile = MapInput {
            on_own_turn: false,
            prior_attack_trait_checks_this_turn: 0,
            current_weapon_agile: false,
            readied_attack_count: Some(255),
        };
        let agile = MapInput {
            on_own_turn: false,
            prior_attack_trait_checks_this_turn: 0,
            current_weapon_agile: true,
            readied_attack_count: Some(255),
        };
        let non_agile_penalty = multiple_attack_penalty(non_agile);
        assert_eq!(non_agile_penalty, multiple_attack_penalty(non_agile));
        let agile_penalty = multiple_attack_penalty(agile);
        assert_eq!(agile_penalty, multiple_attack_penalty(agile));
        let mut dice = ScriptedDice::new([15, 18]);
        assert_eq!(
            attack_outcome(&mut dice, non_agile_penalty, 8, 18),
            (-10, 13, DegreeOfSuccess::Failure)
        );
        assert_eq!(
            attack_outcome(&mut dice, agile_penalty, 8, 18),
            (-8, 18, DegreeOfSuccess::Success)
        );
    }

    /// Player Core, "Multiple Attack Penalty".
    /// Player Core, "Ready".
    #[test]
    fn a_readied_attack_on_your_own_turn_uses_the_frozen_count() {
        let input = MapInput {
            on_own_turn: true,
            prior_attack_trait_checks_this_turn: 0,
            current_weapon_agile: false,
            readied_attack_count: Some(2),
        };
        let penalty = multiple_attack_penalty(input);
        assert_eq!(penalty, multiple_attack_penalty(input));
        let mut dice = ScriptedDice::new([15]);
        assert_eq!(
            attack_outcome(&mut dice, penalty, 8, 18),
            (-10, 13, DegreeOfSuccess::Failure)
        );
    }

    /// Player Core, "Multiple Attack Penalty".
    /// Player Core, "Ready".
    #[test]
    fn a_readied_agile_attack_after_one_non_agile_check_takes_minus_4() {
        let input = MapInput {
            on_own_turn: false,
            prior_attack_trait_checks_this_turn: 1,
            current_weapon_agile: true,
            readied_attack_count: Some(1),
        };
        let penalty = multiple_attack_penalty(input);
        assert_eq!(penalty, multiple_attack_penalty(input));
        let mut dice = ScriptedDice::new([14]);
        assert_eq!(
            attack_outcome(&mut dice, penalty, 8, 18),
            (-4, 18, DegreeOfSuccess::Success)
        );
    }

    /// Player Core, "Multiple Attack Penalty".
    /// Player Core, "Ready".
    #[test]
    fn a_reaction_attack_on_your_own_turn_takes_the_multiple_attack_penalty() {
        let input = MapInput {
            on_own_turn: true,
            prior_attack_trait_checks_this_turn: 1,
            current_weapon_agile: false,
            readied_attack_count: None,
        };
        let penalty = multiple_attack_penalty(input);
        assert_eq!(penalty, multiple_attack_penalty(input));
        let mut dice = ScriptedDice::new([15]);
        assert_eq!(
            attack_outcome(&mut dice, penalty, 8, 18),
            (-5, 18, DegreeOfSuccess::Success)
        );
    }

    /// Player Core, Penalties.
    #[test]
    fn the_penalty_stacks_with_a_circumstance_penalty() {
        let map = penalty(1, false, true);
        let natural = ScriptedDice::new([12]).d20();
        let total = attack_check_total(natural, term(9), term(map), term(-2), term(0), &[])
            .expect("empty caller list fits");
        assert_eq!(total, 14);
        assert_eq!(
            degree_of_success(natural, total, 16),
            DegreeOfSuccess::Failure
        );
    }

    /// Player Core, Penalties.
    #[test]
    fn the_penalty_stacks_with_another_untyped_penalty() {
        let map = penalty(1, false, true);
        let natural = ScriptedDice::new([12]).d20();
        let total = attack_check_total(natural, term(9), term(map), term(0), term(-2), &[])
            .expect("empty caller list fits");
        assert_eq!(total, 14);
        assert_eq!(
            degree_of_success(natural, total, 16),
            DegreeOfSuccess::Failure
        );
    }

    /// Player Core, "Step 4: Degree of Success".
    #[test]
    fn ten_above_an_end_dc_does_not_overflow() {
        assert_eq!(
            degree_of_success(10, i32::MAX, i32::MAX),
            DegreeOfSuccess::Success
        );
        assert_eq!(
            degree_of_success(10, i32::MAX, i32::MAX - 10),
            DegreeOfSuccess::CriticalSuccess
        );
        assert_eq!(
            degree_of_success(20, i32::MAX, i32::MAX),
            DegreeOfSuccess::CriticalSuccess
        );
    }

    /// Player Core, "Step 4: Degree of Success".
    #[test]
    fn ten_below_an_end_dc_does_not_overflow() {
        assert_eq!(
            degree_of_success(10, i32::MIN, i32::MIN),
            DegreeOfSuccess::Success
        );
        assert_eq!(
            degree_of_success(10, i32::MIN, i32::MIN + 10),
            DegreeOfSuccess::CriticalFailure
        );
        assert_eq!(
            degree_of_success(1, i32::MIN, i32::MIN),
            DegreeOfSuccess::Failure
        );
    }

    /// Player Core, "Getting a DC from a Modifier".
    #[test]
    fn dc_modifier_rejects_a_sum_past_i32() {
        assert_eq!(
            DcModifier::new(i32::MAX),
            Err(ArithmeticOverflow::SumDoesNotFit)
        );
        assert_eq!(
            DcModifier::new(i32::MAX - 9),
            Err(ArithmeticOverflow::SumDoesNotFit)
        );
        let edge = DcModifier::new(i32::MAX - 10).expect("10 plus this modifier is i32::MAX");
        assert_eq!(dc_from_modifier(edge), i32::MAX);
        let low = DcModifier::new(i32::MIN).expect("10 plus i32::MIN fits");
        assert_eq!(dc_from_modifier(low), i32::MIN + 10);
    }

    /// Player Core, Checks.
    #[test]
    fn check_modifier_rejects_a_face_sum_past_i32() {
        assert_eq!(
            CheckModifier::new(i32::MAX),
            Err(ArithmeticOverflow::SumDoesNotFit)
        );
        assert_eq!(
            CheckModifier::new(i32::MAX - i32::from(u8::MAX) + 1),
            Err(ArithmeticOverflow::SumDoesNotFit)
        );
        let edge = CheckModifier::new(i32::MAX - i32::from(u8::MAX)).expect("255 plus this fits");
        let mut dice = ScriptedDice::new([20]);
        let roll = resolve_check(&mut dice, edge, 15);
        assert_eq!(roll.natural, 20);
        assert_eq!(roll.total, i32::MAX - i32::from(u8::MAX) + 20);
        assert_eq!(dice.remaining(), 0);
        assert!(CheckModifier::new(i32::MIN).is_ok());
    }

    /// Player Core, Penalties.
    #[test]
    fn attack_term_rejects_a_sum_past_i32() {
        assert_eq!(
            AttackTotalTerm::new(i32::MAX),
            Err(ArithmeticOverflow::SumDoesNotFit)
        );
        assert_eq!(
            AttackTotalTerm::new(i32::MIN),
            Err(ArithmeticOverflow::SumDoesNotFit)
        );
        let high = AttackTotalTerm::new((i32::MAX - i32::from(u8::MAX)) / 4).expect("high term");
        assert_eq!(
            attack_check_total(u8::MAX, high, high, high, high, &[]).expect("high terms fit"),
            i32::MAX
        );
        let low = AttackTotalTerm::new(i32::MIN / 4).expect("low term");
        assert_eq!(
            attack_check_total(0, low, low, low, low, &[]).expect("low terms fit"),
            i32::MIN
        );
    }
}
