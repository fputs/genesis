use crate::dice::Dice;

/// Four degrees of success on a d20 check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Degree {
    CriticalFailure,
    Failure,
    Success,
    CriticalSuccess,
}

impl Degree {
    pub fn improve(self) -> Self {
        match self {
            Degree::CriticalFailure => Degree::Failure,
            Degree::Failure => Degree::Success,
            Degree::Success => Degree::CriticalSuccess,
            Degree::CriticalSuccess => Degree::CriticalSuccess,
        }
    }

    pub fn worsen(self) -> Self {
        match self {
            Degree::CriticalSuccess => Degree::Success,
            Degree::Success => Degree::Failure,
            Degree::Failure => Degree::CriticalFailure,
            Degree::CriticalFailure => Degree::CriticalFailure,
        }
    }
}

/// Degree from `total` against `dc`, then a natural 20 improves it one step and a natural 1 worsens it one step.
///
/// The natural 20 or natural 1 is applied before any later effect calls [`Degree::improve`] or [`Degree::worsen`].
pub fn degree_of_success(natural: u8, total: i32, dc: i32) -> Degree {
    let mut degree = if total >= dc + 10 {
        Degree::CriticalSuccess
    } else if total >= dc {
        Degree::Success
    } else if total <= dc - 10 {
        Degree::CriticalFailure
    } else {
        Degree::Failure
    };
    if natural == 20 {
        degree = degree.improve();
    } else if natural == 1 {
        degree = degree.worsen();
    }
    degree
}

/// Maps an unlisted critical success to success and an unlisted critical failure to failure.
pub fn fallback_unlisted_critical(
    degree: Degree,
    lists_critical_success: bool,
    lists_critical_failure: bool,
) -> Degree {
    match degree {
        Degree::CriticalSuccess if !lists_critical_success => Degree::Success,
        Degree::CriticalFailure if !lists_critical_failure => Degree::Failure,
        other => other,
    }
}

/// DC taken from a modifier: 10 plus that modifier.
pub fn dc_from_modifier(modifier: i32) -> i32 {
    10 + modifier
}

/// One resolved d20 check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckRoll {
    pub natural: u8,
    pub total: i32,
    pub dc: i32,
    pub degree: Degree,
}

/// Rolls a d20, adds `modifier`, and applies [`degree_of_success`] (natural 20 and natural 1 included).
pub fn resolve_check(dice: &mut impl Dice, modifier: i32, dc: i32) -> CheckRoll {
    let natural = dice.d20();
    let total = i32::from(natural) + modifier;
    let degree = degree_of_success(natural, total, dc);
    CheckRoll {
        natural,
        total,
        dc,
        degree,
    }
}

/// A flat check. `natural` is absent when the DC resolves without a roll.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlatRoll {
    pub natural: Option<u8>,
    pub dc: i32,
    pub degree: Degree,
}

/// Rolls one bare d20 against `dc`. No modifier is added.
///
/// DC 1 or lower succeeds without a roll. DC 21 or higher fails without a roll.
/// Automatic results are an ordinary success or an ordinary failure.
pub fn flat_check(dice: &mut impl Dice, dc: i32) -> FlatRoll {
    if dc <= 1 {
        return FlatRoll {
            natural: None,
            dc,
            degree: Degree::Success,
        };
    }
    if dc >= 21 {
        return FlatRoll {
            natural: None,
            dc,
            degree: Degree::Failure,
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
pub fn flat_checks_for_same_thing(dice: &mut impl Dice, dcs: &[i32]) -> FlatRoll {
    let dc = dcs
        .iter()
        .copied()
        .max()
        .expect("flat checks for the same thing name at least one DC");
    flat_check(dice, dc)
}

/// Inputs for the untyped multiple attack penalty. The caller counts attack-trait checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapInput {
    pub on_own_turn: bool,
    pub prior_attack_trait_checks_this_turn: u8,
    pub current_weapon_agile: bool,
}

/// Untyped multiple attack penalty for this check.
///
/// On the creature's own turn the penalty is 0, then −5, then −10, or 0, then −4, then −8
/// when the current weapon is agile. Off turn the penalty is 0. This does not increment the count.
pub fn multiple_attack_penalty(input: MapInput) -> i32 {
    if !input.on_own_turn {
        return 0;
    }
    let (second, later) = if input.current_weapon_agile {
        (-4, -8)
    } else {
        (-5, -10)
    };
    match input.prior_attack_trait_checks_this_turn {
        0 => 0,
        1 => second,
        _ => later,
    }
}

/// Adds the natural roll, attack bonus, untyped MAP, a circumstance penalty, and another untyped penalty.
///
/// Penalties are negative. MAP is not dropped by keeping only the worse of MAP and the circumstance penalty.
pub fn attack_check_total(
    natural: u8,
    attack_bonus: i32,
    map: i32,
    circumstance_penalty: i32,
    other_untyped_penalty: i32,
) -> i32 {
    i32::from(natural) + attack_bonus + map + circumstance_penalty + other_untyped_penalty
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::ScriptedDice;

    /// Player Core, "Step 4: Degree of Success".
    /// Player Core, "Natural 1 and Natural 20".
    #[test]
    fn degrees_follow_dc_bands_then_natural_20_and_1() {
        assert_eq!(degree_of_success(10, 24, 14), Degree::CriticalSuccess);
        assert_eq!(degree_of_success(11, 18, 18), Degree::Success);
        assert_eq!(degree_of_success(8, 15, 18), Degree::Failure);
        assert_eq!(degree_of_success(2, 4, 18), Degree::CriticalFailure);

        // Total is a failure (above DC−10); the natural 20 improves it to a success.
        assert_eq!(degree_of_success(20, 31, 40), Degree::Success);
        // Total is already a success; the natural 20 improves it to a critical success.
        assert_eq!(degree_of_success(20, 21, 18), Degree::CriticalSuccess);
        // Total is a critical success; a natural 1 worsens it one step.
        assert_eq!(degree_of_success(1, 30, 18), Degree::Success);
        // Total is a failure; a natural 1 worsens it to a critical failure.
        assert_eq!(degree_of_success(1, 12, 18), Degree::CriticalFailure);

        let success = resolve_check(&mut ScriptedDice::new([11]), 4, 15);
        assert_eq!(success.total, 15);
        assert_eq!(success.degree, Degree::Success);

        let by_nine = resolve_check(&mut ScriptedDice::new([14]), 10, 15);
        assert_eq!(by_nine.total, 24);
        assert_eq!(by_nine.degree, Degree::Success);

        let by_ten = resolve_check(&mut ScriptedDice::new([15]), 10, 15);
        assert_eq!(by_ten.total, 25);
        assert_eq!(by_ten.degree, Degree::CriticalSuccess);

        let miss_nine = resolve_check(&mut ScriptedDice::new([6]), 0, 15);
        assert_eq!(miss_nine.total, 6);
        assert_eq!(miss_nine.degree, Degree::Failure);

        let miss_ten = resolve_check(&mut ScriptedDice::new([5]), 0, 15);
        assert_eq!(miss_ten.total, 5);
        assert_eq!(miss_ten.degree, Degree::CriticalFailure);

        let nat_20_failure = resolve_check(&mut ScriptedDice::new([20]), 0, 25);
        assert_eq!(nat_20_failure.total, 20);
        assert_eq!(degree_of_success(10, 20, 25), Degree::Failure);
        assert_eq!(nat_20_failure.degree, Degree::Success);

        let nat_20_success = resolve_check(&mut ScriptedDice::new([20]), 0, 18);
        assert_eq!(nat_20_success.total, 20);
        assert_eq!(nat_20_success.degree, Degree::CriticalSuccess);

        let nat_20_already_crit = resolve_check(&mut ScriptedDice::new([20]), 8, 18);
        assert_eq!(nat_20_already_crit.total, 28);
        assert_eq!(nat_20_already_crit.degree, Degree::CriticalSuccess);

        let nat_20_from_crit_fail = resolve_check(&mut ScriptedDice::new([20]), 0, 30);
        assert_eq!(nat_20_from_crit_fail.total, 20);
        assert_eq!(degree_of_success(10, 20, 30), Degree::CriticalFailure);
        assert_eq!(nat_20_from_crit_fail.degree, Degree::Failure);

        let nat_1_success = resolve_check(&mut ScriptedDice::new([1]), 17, 15);
        assert_eq!(nat_1_success.total, 18);
        assert_eq!(degree_of_success(10, 18, 15), Degree::Success);
        assert_eq!(nat_1_success.degree, Degree::Failure);

        let nat_1_crit = resolve_check(&mut ScriptedDice::new([1]), 24, 15);
        assert_eq!(nat_1_crit.total, 25);
        assert_eq!(nat_1_crit.degree, Degree::Success);

        let nat_1_already_crit_fail = resolve_check(&mut ScriptedDice::new([1]), 0, 15);
        assert_eq!(nat_1_already_crit_fail.total, 1);
        assert_eq!(nat_1_already_crit_fail.degree, Degree::CriticalFailure);
    }

    /// Player Core, "Natural 1 and Natural 20". The shift is applied before a later upgrade.
    #[test]
    fn natural_20_is_applied_before_a_later_upgrade() {
        let nat_20 = resolve_check(&mut ScriptedDice::new([20]), 0, 25);
        assert_eq!(degree_of_success(10, nat_20.total, 25), Degree::Failure);
        assert_eq!(nat_20.degree, Degree::Success);
        let upgraded = upgrade_success(nat_20.degree);
        assert_eq!(upgraded, Degree::CriticalSuccess);

        let nat_1 = resolve_check(&mut ScriptedDice::new([1]), 17, 15);
        assert_eq!(degree_of_success(10, nat_1.total, 15), Degree::Success);
        assert_eq!(nat_1.degree, Degree::Failure);
        assert_eq!(upgrade_success(nat_1.degree), Degree::Failure);
    }

    /// A later effect that upgrades a success to a critical success, and leaves every other degree.
    fn upgrade_success(degree: Degree) -> Degree {
        if degree == Degree::Success {
            degree.improve()
        } else {
            degree
        }
    }

    /// Player Core, "Step 4: Degree of Success".
    #[test]
    fn missing_critical_entry_falls_back() {
        let crit_success = resolve_check(&mut ScriptedDice::new([15]), 10, 15);
        assert_eq!(crit_success.degree, Degree::CriticalSuccess);
        assert_eq!(
            fallback_unlisted_critical(crit_success.degree, false, true),
            Degree::Success
        );

        let crit_failure = resolve_check(&mut ScriptedDice::new([5]), 0, 15);
        assert_eq!(crit_failure.degree, Degree::CriticalFailure);
        assert_eq!(
            fallback_unlisted_critical(crit_failure.degree, true, false),
            Degree::Failure
        );

        let listed_success = resolve_check(&mut ScriptedDice::new([15]), 10, 15);
        let listed_failure = resolve_check(&mut ScriptedDice::new([5]), 0, 15);
        assert_eq!(
            fallback_unlisted_critical(listed_success.degree, true, true),
            Degree::CriticalSuccess
        );
        assert_eq!(
            fallback_unlisted_critical(listed_failure.degree, true, true),
            Degree::CriticalFailure
        );
    }

    /// Player Core, "Getting a DC from a Modifier".
    #[test]
    fn dc_from_modifier_is_ten_plus() {
        assert_eq!(dc_from_modifier(7), 17);
        assert_eq!(dc_from_modifier(0), 10);
        assert_eq!(dc_from_modifier(-3), 7);

        let dc = dc_from_modifier(7);
        let roll = resolve_check(&mut ScriptedDice::new([12]), 5, dc);
        assert_eq!(roll.total, 17);
        assert_eq!(roll.degree, Degree::Success);
    }

    /// Player Core, "Flat Checks".
    #[test]
    fn flat_check_ignores_modifiers() {
        let ignored_modifier = 9;
        let mut dice = ScriptedDice::new([10]);
        let roll = flat_check(&mut dice, 11);
        assert_eq!(roll.natural, Some(10));
        assert_eq!(roll.degree, Degree::Failure);
        assert_eq!(
            degree_of_success(10, 10 + ignored_modifier, 11),
            Degree::Success
        );
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, "Flat Checks".
    #[test]
    fn flat_check_auto_success_and_failure() {
        let mut dc_one = ScriptedDice::new([4]);
        let auto_one = flat_check(&mut dc_one, 1);
        assert_eq!(auto_one.degree, Degree::Success);
        assert_eq!(auto_one.natural, None);
        assert_eq!(dc_one.remaining(), 1);
        assert_eq!(dc_one.d20(), 4);

        let mut dc_zero = ScriptedDice::new([4]);
        let auto_zero = flat_check(&mut dc_zero, 0);
        assert_eq!(auto_zero.degree, Degree::Success);
        assert_eq!(auto_zero.natural, None);
        assert_eq!(dc_zero.remaining(), 1);

        let mut dc_twenty_one = ScriptedDice::new([20]);
        let auto_fail = flat_check(&mut dc_twenty_one, 21);
        assert_eq!(auto_fail.degree, Degree::Failure);
        assert_eq!(auto_fail.natural, None);
        assert_eq!(dc_twenty_one.remaining(), 1);

        let mut dc_twenty = ScriptedDice::new([20]);
        let rolled = flat_check(&mut dc_twenty, 20);
        assert_eq!(dc_twenty.remaining(), 0);
        assert_eq!(rolled.natural, Some(20));
        // Numeric total 20 meets DC 20 (a success). Natural 20 then improves that degree.
        assert_eq!(degree_of_success(10, 20, 20), Degree::Success);
        assert_eq!(rolled.degree, Degree::CriticalSuccess);
    }

    /// Player Core, "Flat Checks".
    #[test]
    fn flat_checks_for_the_same_thing_roll_once() {
        let mut same = ScriptedDice::new([10, 20]);
        let combined = flat_checks_for_same_thing(&mut same, &[5, 15]);
        assert_eq!(combined.natural, Some(10));
        assert_eq!(combined.dc, 15);
        assert_eq!(combined.degree, Degree::Failure);
        assert_eq!(same.remaining(), 1);
        assert_eq!(same.d20(), 20);

        let mut separate = ScriptedDice::new([10, 4]);
        let first = flat_check(&mut separate, 5);
        let second = flat_check(&mut separate, 11);
        assert_eq!(first.natural, Some(10));
        assert_eq!(first.degree, Degree::Success);
        assert_eq!(second.natural, Some(4));
        assert_eq!(second.degree, Degree::Failure);
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
        assert_eq!(first, (0, 18, Degree::Success));

        let second = attack_outcome(&mut dice, penalty(1, false, true), 8, 18);
        assert_eq!(second, (-5, 18, Degree::Success));

        let third = attack_outcome(&mut dice, penalty(2, false, true), 8, 18);
        assert_eq!(third, (-10, 13, Degree::Failure));

        let fourth = attack_outcome(&mut dice, penalty(3, false, true), 8, 18);
        assert_eq!(fourth.0, -10);
        assert_eq!(fourth.1, 13);

        let second_agile = attack_outcome(&mut dice, penalty(1, true, true), 8, 18);
        assert_eq!(second_agile, (-4, 18, Degree::Success));

        let third_agile = attack_outcome(&mut dice, penalty(2, true, true), 8, 18);
        assert_eq!(third_agile, (-8, 18, Degree::Success));

        // One prior non-agile check; the current weapon is agile, so this check takes −4.
        let agile_now = attack_outcome(&mut dice, penalty(1, true, true), 8, 16);
        assert_eq!(agile_now, (-4, 16, Degree::Success));

        // One prior agile check; the current weapon is not agile, so this check takes −5.
        let non_agile_now = attack_outcome(&mut dice, penalty(1, false, true), 8, 16);
        assert_eq!(non_agile_now, (-5, 15, Degree::Failure));

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
        assert_eq!(after_non_strike, (-5, 18, Degree::Failure));
        assert_eq!(dice.remaining(), 0);
    }

    fn attack_outcome(
        dice: &mut ScriptedDice,
        map: i32,
        attack_bonus: i32,
        dc: i32,
    ) -> (i32, i32, Degree) {
        let natural = dice.d20();
        let total = attack_check_total(natural, attack_bonus, map, 0, 0);
        (map, total, degree_of_success(natural, total, dc))
    }

    /// Player Core, "Penalties". MAP is untyped, so it is added to a circumstance penalty.
    #[test]
    fn map_stacks_with_a_circumstance_penalty() {
        let map = multiple_attack_penalty(MapInput {
            on_own_turn: true,
            prior_attack_trait_checks_this_turn: 1,
            current_weapon_agile: false,
        });
        assert_eq!(map, -5);
        let natural = ScriptedDice::new([12]).d20();
        let total = attack_check_total(natural, 9, map, -2, 0);
        assert_eq!(total, 14);
        assert_eq!(degree_of_success(natural, total, 16), Degree::Failure);
    }

    /// Player Core, "Penalties". MAP is added to every other untyped penalty.
    #[test]
    fn map_stacks_with_another_untyped_penalty() {
        let map = multiple_attack_penalty(MapInput {
            on_own_turn: true,
            prior_attack_trait_checks_this_turn: 1,
            current_weapon_agile: false,
        });
        assert_eq!(map, -5);
        let natural = ScriptedDice::new([12]).d20();
        let total = attack_check_total(natural, 9, map, 0, -2);
        assert_eq!(total, 14);
        assert_eq!(degree_of_success(natural, total, 16), Degree::Failure);
    }
}
