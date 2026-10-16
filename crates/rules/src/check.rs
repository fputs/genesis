/// Four degrees of success on a d20 check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Degree {
    CriticalFailure,
    Failure,
    Success,
    CriticalSuccess,
}

impl Degree {
    fn improve(self) -> Self {
        match self {
            Degree::CriticalFailure => Degree::Failure,
            Degree::Failure => Degree::Success,
            Degree::Success => Degree::CriticalSuccess,
            Degree::CriticalSuccess => Degree::CriticalSuccess,
        }
    }

    fn worsen(self) -> Self {
        match self {
            Degree::CriticalSuccess => Degree::Success,
            Degree::Success => Degree::Failure,
            Degree::Failure => Degree::CriticalFailure,
            Degree::CriticalFailure => Degree::CriticalFailure,
        }
    }
}

/// Degree from `total` against `dc`, then a natural 20 improves it one step and a natural 1 worsens it one step.
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

/// Penalty applied to an attack roll after `prior_strikes` Strikes already made this turn.
pub fn multiple_attack_penalty(prior_strikes: u8) -> i32 {
    match prior_strikes {
        0 => 0,
        1 => -5,
        _ => -10,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Player Core, "Degrees of Success".
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
    }

    /// Player Core, "Multiple Attack Penalty": second attack −5, third and subsequent −10.
    #[test]
    fn map_is_zero_then_minus_five_then_minus_ten() {
        assert_eq!(multiple_attack_penalty(0), 0);
        assert_eq!(multiple_attack_penalty(1), -5);
        assert_eq!(multiple_attack_penalty(2), -10);
        assert_eq!(multiple_attack_penalty(3), -10);
    }
}
