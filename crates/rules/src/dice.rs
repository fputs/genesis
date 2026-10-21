use std::collections::VecDeque;

/// Source of die rolls. Callers pass this into every roll. No roll reads a process-wide generator.
pub trait Dice {
    /// Uniform integer in `1..=faces`.
    fn roll(&mut self, faces: u8) -> u8;

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
    pub fn new(rolls: impl IntoIterator<Item = u8>) -> Self {
        Self {
            rolls: rolls.into_iter().collect(),
        }
    }

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

    #[test]
    fn seeded_dice_repeats_the_sequence_for_a_seed() {
        let mut first = SeededDice::new(42);
        let mut second = SeededDice::new(42);
        let sequence = |dice: &mut SeededDice| [dice.d20(), dice.roll(8), dice.roll(6)];
        assert_eq!(sequence(&mut first), sequence(&mut second));
    }

    #[test]
    #[should_panic(expected = "scripted dice exhausted")]
    fn scripted_dice_panics_when_exhausted() {
        let mut dice = ScriptedDice::new([8]);
        assert_eq!(dice.d20(), 8);
        dice.d20();
    }

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
}
