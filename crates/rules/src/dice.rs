use std::collections::VecDeque;

/// Source of die rolls. Callers pass this into every roll. No roll reads a process-wide generator.
pub trait Dice {
    /// Uniform integer in `1..=faces`. Player Core, Checks.
    fn roll(&mut self, faces: u8) -> u8;

    /// Rolls a d20. Player Core, Checks.
    fn d20(&mut self) -> u8 {
        self.roll(20)
    }
}

/// Deterministic xorshift stream. The same seed yields the same rolls.
#[derive(Clone, Debug)]
pub struct SeededDice {
    state: u64,
}

impl SeededDice {
    /// Same seed, same sequence. Seed 0 is replaced so the stream is not stuck.
    ///
    /// Player Core, Checks. Callers inject this source.
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0xA5A5_5A5A_1234_5678
            } else {
                seed
            },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

impl Dice for SeededDice {
    fn roll(&mut self, faces: u8) -> u8 {
        assert!(faces >= 1, "a die has at least one face");
        (self.next_u64() % u64::from(faces)) as u8 + 1
    }
}

/// Yields a fixed sequence. Panics when the sequence runs out or a value is not on the die.
#[derive(Clone, Debug)]
pub struct ScriptedDice {
    rolls: VecDeque<u8>,
}

impl ScriptedDice {
    /// Queues `rolls` in the order they will be drawn.
    ///
    /// Player Core, Checks. Tests use this so a roll is exact.
    pub fn new(rolls: impl IntoIterator<Item = u8>) -> Self {
        Self {
            rolls: rolls.into_iter().collect(),
        }
    }

    /// How many scripted faces have not been drawn.
    pub fn remaining(&self) -> usize {
        self.rolls.len()
    }
}

impl Dice for ScriptedDice {
    fn roll(&mut self, faces: u8) -> u8 {
        let value = self.rolls.pop_front().expect("scripted dice exhausted");
        assert!(
            faces >= 1 && value >= 1 && value <= faces,
            "scripted roll {value} is not a legal d{faces}"
        );
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Player Core, Checks.
    #[test]
    fn seeded_dice_repeats_the_sequence_for_a_seed() {
        let mut first = SeededDice::new(42);
        let mut second = SeededDice::new(42);
        let sequence = |dice: &mut SeededDice| [dice.d20(), dice.roll(8), dice.roll(6)];
        assert_eq!(sequence(&mut first), sequence(&mut second));
    }

    /// Player Core, Checks.
    #[test]
    #[should_panic(expected = "scripted dice exhausted")]
    fn scripted_dice_panics_when_exhausted() {
        let mut dice = ScriptedDice::new([8]);
        assert_eq!(dice.d20(), 8);
        dice.d20();
    }

    /// Player Core, Checks.
    #[test]
    #[should_panic(expected = "scripted roll 6 is not a legal d4")]
    fn scripted_dice_panics_when_out_of_range() {
        for (value, faces) in [(0, 20), (21, 20)] {
            let mut dice = ScriptedDice::new([value]);
            let caught =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| dice.roll(faces)));
            assert!(caught.is_err(), "d{faces} value {value} should panic");
        }

        let mut legal = ScriptedDice::new([1, 20]);
        assert_eq!(legal.d20(), 1);
        assert_eq!(legal.d20(), 20);

        ScriptedDice::new([6]).roll(4);
    }

    /// Player Core, Checks.
    ///
    /// The dice spec requires two seeded sources for this scenario, so this test does not use scripted dice.
    #[test]
    fn the_same_seed_repeats() {
        let mut first = SeededDice::new(42);
        let mut second = SeededDice::new(42);
        assert_eq!(
            [first.d20(), first.roll(8), first.roll(6)],
            [second.d20(), second.roll(8), second.roll(6)]
        );
    }

    /// Player Core, Checks.
    #[test]
    #[should_panic(expected = "scripted dice exhausted")]
    fn scripted_dice_panic_when_exhausted() {
        let mut dice = ScriptedDice::new([8]);
        assert_eq!(dice.d20(), 8);
        dice.d20();
    }

    /// Player Core, Checks.
    #[test]
    fn scripted_dice_panic_out_of_range() {
        for (value, faces) in [(0, 20), (21, 20), (6, 4)] {
            let mut dice = ScriptedDice::new([value]);
            let caught =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| dice.roll(faces)));
            assert!(caught.is_err(), "d{faces} value {value} should panic");
        }
    }

    /// Player Core, Checks.
    #[test]
    fn scripted_bounds_are_legal() {
        let mut dice = ScriptedDice::new([1, 20]);
        assert_eq!(dice.d20(), 1);
        assert_eq!(dice.d20(), 20);
    }
}
