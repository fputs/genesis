#[cfg(any(test, feature = "test-support"))]
use std::collections::VecDeque;

use std::num::NonZeroU8;

/// A die's face count. Zero is not a die.
///
/// Player Core, Checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Faces(NonZeroU8);

/// Why a face count was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaceCountError {
    /// The count was zero.
    Zero,
    /// The count was negative or did not fit in a `u8`.
    OutOfRange,
}

impl Faces {
    /// Rejects zero.
    ///
    /// Player Core, Checks.
    pub const fn new(faces: u8) -> Result<Self, FaceCountError> {
        match NonZeroU8::new(faces) {
            Some(value) => Ok(Self(value)),
            None => Err(FaceCountError::Zero),
        }
    }

    /// Rejects zero, a negative count, and a count above `u8::MAX`.
    ///
    /// Player Core, Damage Rolls.
    pub fn try_from_i32(faces: i32) -> Result<Self, FaceCountError> {
        let Ok(faces) = u8::try_from(faces) else {
            return Err(FaceCountError::OutOfRange);
        };
        Self::new(faces).map_err(|_| FaceCountError::OutOfRange)
    }

    /// The face count. Always at least 1.
    pub const fn get(self) -> u8 {
        self.0.get()
    }

    /// The face count as a contribution. Always fits in `i32`.
    pub fn to_i32(self) -> i32 {
        i32::from(self.get())
    }

    /// A d20. Twenty is not zero.
    pub const fn twenty() -> Self {
        match Self::new(20) {
            Ok(faces) => faces,
            Err(FaceCountError::Zero | FaceCountError::OutOfRange) => Self(NonZeroU8::MIN),
        }
    }
}

/// Source of die rolls. Callers pass this into every roll. No roll reads a process-wide generator.
pub trait Dice {
    /// Uniform integer in `1..=faces`. Player Core, Checks.
    fn roll(&mut self, faces: Faces) -> u8;

    /// Rolls a d20. Player Core, Checks.
    fn d20(&mut self) -> u8 {
        self.roll(Faces::twenty())
    }
}

/// Deterministic xorshift stream. The same seed yields the same rolls.
///
/// `roll` takes [`Faces`], so a zero-face die cannot be requested. This source never panics.
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
    fn roll(&mut self, faces: Faces) -> u8 {
        let span = u64::from(faces.get());
        let zero_based = self.next_u64() % span;
        match u8::try_from(zero_based) {
            Ok(face) => face + 1,
            Err(_) => 1,
        }
    }
}

/// Fixed sequence of faces for tests.
///
/// Compiled for this crate's tests and for the `test-support` feature.
///
/// # Panics
///
/// [`ScriptedDice::roll`] panics if the script is exhausted, or if the queued value is
/// outside `1..=faces`. A face count of zero cannot be constructed.
#[cfg(any(test, feature = "test-support"))]
#[derive(Clone, Debug)]
pub struct ScriptedDice {
    rolls: VecDeque<u8>,
}

#[cfg(any(test, feature = "test-support"))]
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

#[cfg(any(test, feature = "test-support"))]
impl Dice for ScriptedDice {
    /// # Panics
    ///
    /// Panics with `scripted dice exhausted` when the queue is empty.
    /// Panics with `scripted roll {value} is not a legal d{faces}` when the queued value
    /// is outside `1..=faces`.
    fn roll(&mut self, faces: Faces) -> u8 {
        let value = self.rolls.pop_front().expect("scripted dice exhausted");
        let faces = faces.get();
        assert!(
            value >= 1 && value <= faces,
            "scripted roll {value} is not a legal d{faces}"
        );
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn die(count: u8) -> Faces {
        Faces::new(count).expect("test face count")
    }

    /// Player Core, Checks.
    #[test]
    fn seeded_dice_repeats_the_sequence_for_a_seed() {
        let mut first = SeededDice::new(42);
        let mut second = SeededDice::new(42);
        let sequence = |dice: &mut SeededDice| [dice.d20(), dice.roll(die(8)), dice.roll(die(6))];
        assert_eq!(sequence(&mut first), sequence(&mut second));
    }

    /// Zero is not a die. Player Core, Checks.
    #[test]
    fn faces_reject_zero_and_values_that_do_not_fit() {
        assert_eq!(Faces::new(0), Err(FaceCountError::Zero));
        assert_eq!(Faces::try_from_i32(0), Err(FaceCountError::OutOfRange));
        assert_eq!(Faces::try_from_i32(-1), Err(FaceCountError::OutOfRange));
        assert_eq!(Faces::try_from_i32(256), Err(FaceCountError::OutOfRange));
        assert_eq!(Faces::new(20).expect("d20").get(), 20);
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
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| dice.roll(die(faces))));
            assert!(caught.is_err(), "d{faces} value {value} should panic");
        }

        let mut legal = ScriptedDice::new([1, 20]);
        assert_eq!(legal.d20(), 1);
        assert_eq!(legal.d20(), 20);

        ScriptedDice::new([6]).roll(die(4));
    }

    /// Player Core, Checks.
    ///
    /// The dice spec requires two seeded sources for this scenario, so this test does not use scripted dice.
    #[test]
    fn the_same_seed_repeats() {
        let mut first = SeededDice::new(42);
        let mut second = SeededDice::new(42);
        assert_eq!(
            [first.d20(), first.roll(die(8)), first.roll(die(6))],
            [second.d20(), second.roll(die(8)), second.roll(die(6))]
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
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| dice.roll(die(faces))));
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
