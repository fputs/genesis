//! The sixteen Player Core skills, their modifiers, and a skill DC.
//!
//! Player Core pg. 405, Skill Checks. Player Core pg. 225, Key Attribute.

use crate::check::{ArithmeticOverflow, CheckModifier, DegreeOfSuccess};
use crate::modifiers::{Modifier, Statistic, total, with_skill_conditions};
use crate::save::{Level, Proficiency, proficiency_bonus};

/// One of the sixteen Player Core skills. Lore is not a skill in this change.
///
/// Player Core pg. 225, Key Attribute. Player Core pg. 240, Lore.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skill {
    /// Dexterity. Player Core pg. 233, Acrobatics.
    Acrobatics,
    /// Intelligence. Player Core pg. 234, Arcana.
    Arcana,
    /// Strength. Player Core pg. 234, Athletics.
    Athletics,
    /// Intelligence. Player Core pg. 236, Crafting.
    Crafting,
    /// Charisma. Player Core pg. 237, Deception.
    Deception,
    /// Charisma. Player Core pg. 239, Diplomacy.
    Diplomacy,
    /// Charisma. Player Core pg. 240, Intimidation.
    Intimidation,
    /// Wisdom. Player Core pg. 241, Medicine.
    Medicine,
    /// Wisdom. Player Core pg. 242, Nature.
    Nature,
    /// Intelligence. Player Core pg. 243, Occultism.
    Occultism,
    /// Charisma. Player Core pg. 243, Performance.
    Performance,
    /// Wisdom. Player Core pg. 244, Religion.
    Religion,
    /// Intelligence. Player Core pg. 244, Society.
    Society,
    /// Dexterity. Player Core pg. 244, Stealth.
    Stealth,
    /// Wisdom. Player Core pg. 246, Survival.
    Survival,
    /// Dexterity. Player Core pg. 246, Thievery.
    Thievery,
}

/// The sixteen skills in book order. Tests walk this list; production matches on [`Skill`].
///
/// Player Core pg. 225, Key Attribute.
#[cfg(test)]
pub(crate) const SKILLS: [Skill; 16] = [
    Skill::Acrobatics,
    Skill::Arcana,
    Skill::Athletics,
    Skill::Crafting,
    Skill::Deception,
    Skill::Diplomacy,
    Skill::Intimidation,
    Skill::Medicine,
    Skill::Nature,
    Skill::Occultism,
    Skill::Performance,
    Skill::Religion,
    Skill::Society,
    Skill::Stealth,
    Skill::Survival,
    Skill::Thievery,
];

impl Skill {
    /// The statistic whose modifiers apply to this skill.
    ///
    /// Player Core pg. 405, Skill Checks.
    pub fn statistic(self) -> Statistic {
        match self {
            Self::Acrobatics => Statistic::Acrobatics,
            Self::Arcana => Statistic::Arcana,
            Self::Athletics => Statistic::Athletics,
            Self::Crafting => Statistic::Crafting,
            Self::Deception => Statistic::Deception,
            Self::Diplomacy => Statistic::Diplomacy,
            Self::Intimidation => Statistic::Intimidation,
            Self::Medicine => Statistic::Medicine,
            Self::Nature => Statistic::Nature,
            Self::Occultism => Statistic::Occultism,
            Self::Performance => Statistic::Performance,
            Self::Religion => Statistic::Religion,
            Self::Society => Statistic::Society,
            Self::Stealth => Statistic::Stealth,
            Self::Survival => Statistic::Survival,
            Self::Thievery => Statistic::Thievery,
        }
    }

    /// The key attribute modifier stored on `skills`.
    ///
    /// Player Core pg. 225, Key Attribute.
    pub fn attribute(self, skills: &Skills) -> i32 {
        match self {
            Self::Athletics => skills.strength,
            Self::Acrobatics | Self::Stealth | Self::Thievery => skills.dexterity,
            Self::Arcana | Self::Crafting | Self::Occultism | Self::Society => skills.intelligence,
            Self::Medicine | Self::Nature | Self::Religion | Self::Survival => skills.wisdom,
            Self::Deception | Self::Diplomacy | Self::Intimidation | Self::Performance => {
                skills.charisma
            }
        }
    }

    fn rank(self, skills: &Skills) -> Proficiency {
        match self {
            Self::Acrobatics => skills.acrobatics,
            Self::Arcana => skills.arcana,
            Self::Athletics => skills.athletics,
            Self::Crafting => skills.crafting,
            Self::Deception => skills.deception,
            Self::Diplomacy => skills.diplomacy,
            Self::Intimidation => skills.intimidation,
            Self::Medicine => skills.medicine,
            Self::Nature => skills.nature,
            Self::Occultism => skills.occultism,
            Self::Performance => skills.performance,
            Self::Religion => skills.religion,
            Self::Society => skills.society,
            Self::Stealth => skills.stealth,
            Self::Survival => skills.survival,
            Self::Thievery => skills.thievery,
        }
    }
}

/// Attribute modifiers and one proficiency rank for each skill.
///
/// The attributes are modifiers, not scores. Constitution is not stored.
/// They are not read from [`crate::save::SavingThrows`] (DEV-59).
///
/// Player Core pg. 225, Key Attribute. Player Core pg. 405, Skill Checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Skills {
    /// Level used for every proficiency bonus. Player Core pg. 405, Skill Checks.
    pub level: Level,
    /// Strength modifier. Player Core pg. 234, Athletics.
    pub strength: i32,
    /// Dexterity modifier. Player Core pg. 233, Acrobatics.
    pub dexterity: i32,
    /// Intelligence modifier. Player Core pg. 234, Arcana.
    pub intelligence: i32,
    /// Wisdom modifier. Player Core pg. 241, Medicine.
    pub wisdom: i32,
    /// Charisma modifier. Player Core pg. 239, Diplomacy.
    pub charisma: i32,
    /// Acrobatics rank. Player Core pg. 233, Acrobatics.
    pub acrobatics: Proficiency,
    /// Arcana rank. Player Core pg. 234, Arcana.
    pub arcana: Proficiency,
    /// Athletics rank. Player Core pg. 234, Athletics.
    pub athletics: Proficiency,
    /// Crafting rank. Player Core pg. 236, Crafting.
    pub crafting: Proficiency,
    /// Deception rank. Player Core pg. 237, Deception.
    pub deception: Proficiency,
    /// Diplomacy rank. Player Core pg. 239, Diplomacy.
    pub diplomacy: Proficiency,
    /// Intimidation rank. Player Core pg. 240, Intimidation.
    pub intimidation: Proficiency,
    /// Medicine rank. Player Core pg. 241, Medicine.
    pub medicine: Proficiency,
    /// Nature rank. Player Core pg. 242, Nature.
    pub nature: Proficiency,
    /// Occultism rank. Player Core pg. 243, Occultism.
    pub occultism: Proficiency,
    /// Performance rank. Player Core pg. 243, Performance.
    pub performance: Proficiency,
    /// Religion rank. Player Core pg. 244, Religion.
    pub religion: Proficiency,
    /// Society rank. Player Core pg. 244, Society.
    pub society: Proficiency,
    /// Stealth rank. Player Core pg. 244, Stealth.
    pub stealth: Proficiency,
    /// Survival rank. Player Core pg. 246, Survival.
    pub survival: Proficiency,
    /// Thievery rank. Player Core pg. 246, Thievery.
    pub thievery: Proficiency,
}

/// Why a skill modifier was rejected.
///
/// Player Core pg. 405, Skill Checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SkillError {
    /// The attribute, proficiency, and modifiers did not fit a check modifier.
    ///
    /// Player Core pg. 405, Skill Checks.
    ModifierOverflow(ArithmeticOverflow),
}

/// One skill check. The degree is the whole result. No fact is stored.
///
/// Player Core pg. 405, Skill Checks. Player Core pg. 231, Recall Knowledge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkillRoll {
    /// The d20. Player Core, Checks.
    pub natural: u8,
    /// Natural roll plus the skill modifier. Player Core pg. 405, Skill Checks.
    pub total: i32,
    /// Degree after the natural 20 or natural 1 shift. Player Core pg. 401, Natural 1 and Natural 20.
    pub degree: DegreeOfSuccess,
}

/// Key attribute, plus proficiency, plus modifiers on that skill, plus frightened and sickened.
///
/// A modifier on another skill is ignored. Proficiency is not a bonus type.
/// Frightened and sickened penalties apply even when the caller's entry names another
/// statistic. Unconscious adds nothing.
///
/// Player Core pg. 405, Skill Checks. Player Core pg. 444, Frightened.
/// Player Core pg. 446, Sickened.
pub(crate) fn skill_modifier(
    training: &Skills,
    skill: Skill,
    modifiers: &[Modifier],
) -> Result<CheckModifier, SkillError> {
    let expanded = with_skill_conditions(modifiers);
    let applicable: Vec<Modifier> = expanded
        .into_iter()
        .filter(|modifier| modifier.statistic() == skill.statistic())
        .collect();
    let stacked = total(&applicable)
        .map_err(|_| SkillError::ModifierOverflow(ArithmeticOverflow::SumDoesNotFit))?;
    let combined = skill
        .attribute(training)
        .checked_add(proficiency_bonus(training.level, skill.rank(training)))
        .and_then(|sum| sum.checked_add(stacked.get(skill.statistic())))
        .ok_or(SkillError::ModifierOverflow(
            ArithmeticOverflow::SumDoesNotFit,
        ))?;
    CheckModifier::new(combined).map_err(SkillError::ModifierOverflow)
}

/// Skill DC: 10 plus the skill modifier.
///
/// A [`CheckModifier`] keeps `modifier + 255` inside `i32`, so `modifier + 10` fits.
///
/// Player Core pg. 405, Skill Checks. Player Core pg. 401, Getting a DC from a Modifier.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "encounter checks take a caller DC; this is the skill DC procedure the tests call"
    )
)]
pub(crate) fn skill_dc(modifier: CheckModifier) -> i32 {
    modifier.get() + 10
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modifiers::{ConditionId, PenaltyType, SourceId};
    use crate::save::LevelError;

    fn level(value: i32) -> Level {
        Level::new(value).expect("level")
    }

    fn untrained() -> Proficiency {
        Proficiency::Untrained
    }

    fn athletics(level_value: i32, strength: i32, rank: Proficiency) -> Skills {
        Skills {
            level: level(level_value),
            strength,
            dexterity: 0,
            intelligence: 0,
            wisdom: 0,
            charisma: 0,
            acrobatics: untrained(),
            arcana: untrained(),
            athletics: rank,
            crafting: untrained(),
            deception: untrained(),
            diplomacy: untrained(),
            intimidation: untrained(),
            medicine: untrained(),
            nature: untrained(),
            occultism: untrained(),
            performance: untrained(),
            religion: untrained(),
            society: untrained(),
            stealth: untrained(),
            survival: untrained(),
            thievery: untrained(),
        }
    }

    fn modifier(training: &Skills, skill: Skill, modifiers: &[Modifier]) -> i32 {
        let first = skill_modifier(training, skill, modifiers).expect("modifier");
        let second = skill_modifier(training, skill, modifiers).expect("modifier");
        assert_eq!(first, second);
        first.get()
    }

    fn frightened(rank: u32, statistic: Statistic) -> Modifier {
        Modifier::penalty(
            PenaltyType::Status,
            -i32::try_from(rank).expect("rank"),
            SourceId(9),
            1,
            None,
            statistic,
        )
        .expect("penalty")
        .with_condition(ConditionId::FRIGHTENED, Some(rank))
        .expect("frightened")
    }

    fn sickened(rank: u32, statistic: Statistic) -> Modifier {
        Modifier::penalty(
            PenaltyType::Status,
            -i32::try_from(rank).expect("rank"),
            SourceId(8),
            2,
            None,
            statistic,
        )
        .expect("penalty")
        .with_condition(ConditionId::SICKENED, Some(rank))
        .expect("sickened")
    }

    /// Player Core pg. 405, Skill Checks.
    #[test]
    fn trained_at_level_2_with_strength_0_is_plus_4() {
        let training = athletics(2, 0, Proficiency::Trained);
        assert_eq!(modifier(&training, Skill::Athletics, &[]), 4);
    }

    /// Player Core pg. 405, Skill Checks.
    #[test]
    fn legendary_athletics_at_level_25_is_plus_33() {
        let training = athletics(25, 0, Proficiency::Legendary);
        assert_eq!(modifier(&training, Skill::Athletics, &[]), 33);
    }

    /// Player Core pg. 405, Skill Checks.
    #[test]
    fn a_skill_level_of_0_cannot_be_built() {
        assert_eq!(Level::new(0), Err(LevelError::OutOfRange));
    }

    /// Player Core pg. 405, Skill Checks.
    #[test]
    fn a_skill_level_of_26_cannot_be_built() {
        assert_eq!(Level::new(26), Err(LevelError::OutOfRange));
    }

    /// Player Core pg. 405, Skill Checks.
    #[test]
    fn a_skill_level_of_25_can_be_built() {
        let training = athletics(25, 0, untrained());
        assert_eq!(training.level.get(), 25);
    }

    /// Player Core pg. 405, Skill Checks. Player Core pg. 401, Getting a DC from a Modifier.
    #[test]
    fn a_skill_modifier_of_plus_7_has_dc_17() {
        let training = athletics(1, 4, Proficiency::Trained);
        let value = skill_modifier(&training, Skill::Athletics, &[]).expect("modifier");
        assert_eq!(value.get(), 7);
        assert_eq!(skill_dc(value), 17);
    }

    /// Player Core pg. 225, Key Attribute.
    #[test]
    fn each_skill_uses_its_key_attribute() {
        let training = Skills {
            level: level(1),
            strength: 1,
            dexterity: 1,
            intelligence: 1,
            wisdom: 1,
            charisma: 1,
            acrobatics: untrained(),
            arcana: untrained(),
            athletics: untrained(),
            crafting: untrained(),
            deception: untrained(),
            diplomacy: untrained(),
            intimidation: untrained(),
            medicine: untrained(),
            nature: untrained(),
            occultism: untrained(),
            performance: untrained(),
            religion: untrained(),
            society: untrained(),
            stealth: untrained(),
            survival: untrained(),
            thievery: untrained(),
        };
        assert_eq!(SKILLS.len(), 16);
        for skill in SKILLS {
            assert_eq!(modifier(&training, skill, &[]), 1);
        }
        assert_eq!(Skill::Acrobatics.attribute(&training), training.dexterity);
        assert_eq!(Skill::Arcana.attribute(&training), training.intelligence);
        assert_eq!(Skill::Athletics.attribute(&training), training.strength);
        assert_eq!(Skill::Crafting.attribute(&training), training.intelligence);
        assert_eq!(Skill::Deception.attribute(&training), training.charisma);
        assert_eq!(Skill::Diplomacy.attribute(&training), training.charisma);
        assert_eq!(Skill::Intimidation.attribute(&training), training.charisma);
        assert_eq!(Skill::Medicine.attribute(&training), training.wisdom);
        assert_eq!(Skill::Nature.attribute(&training), training.wisdom);
        assert_eq!(Skill::Occultism.attribute(&training), training.intelligence);
        assert_eq!(Skill::Performance.attribute(&training), training.charisma);
        assert_eq!(Skill::Religion.attribute(&training), training.wisdom);
        assert_eq!(Skill::Society.attribute(&training), training.intelligence);
        assert_eq!(Skill::Stealth.attribute(&training), training.dexterity);
        assert_eq!(Skill::Survival.attribute(&training), training.wisdom);
        assert_eq!(Skill::Thievery.attribute(&training), training.dexterity);
        assert!(!format!("{SKILLS:?}").contains("Lore"));
    }

    /// Player Core pg. 444, Frightened. Player Core pg. 405, Skill Checks.
    #[test]
    fn frightened_2_on_trained_athletics_is_plus_1_and_dc_11() {
        let training = athletics(1, 0, Proficiency::Trained);
        let modifiers = [frightened(2, Statistic::Attack)];
        let value = skill_modifier(&training, Skill::Athletics, &modifiers).expect("modifier");
        assert_eq!(value.get(), 1);
        assert_eq!(skill_dc(value), 11);
    }

    /// Player Core pg. 444, Frightened. Player Core pg. 446, Sickened.
    /// Player Core pg. 400, Penalties.
    #[test]
    fn frightened_2_and_sickened_1_give_minus_2() {
        let training = athletics(1, 0, Proficiency::Trained);
        let modifiers = [
            frightened(2, Statistic::Attack),
            sickened(1, Statistic::Fortitude),
        ];
        let expanded = with_skill_conditions(&modifiers);
        let stacked = total(&expanded).expect("total");
        assert_eq!(stacked.get(Statistic::Athletics), -2);
        let value = skill_modifier(&training, Skill::Athletics, &modifiers).expect("modifier");
        assert_eq!(value.get(), 1);
    }
}
