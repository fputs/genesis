#[cfg(test)]
mod tests {
    use super::{
        Level, LevelError, Proficiency, Save, SavingThrows, basic_save_amount, save_dc,
        save_modifier,
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

    /// Player Core pg. 404, Saving Throws.
    #[test]
    fn a_level_past_the_legendary_bonus_cannot_be_built() {
        assert_eq!(Level::new(2_147_483_640), Err(LevelError::OutOfRange));
    }
}
