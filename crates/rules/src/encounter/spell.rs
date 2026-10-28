//! Breathe Fire and Telekinetic Projectile.
//!
//! Neither cast calls `resolve_reactions` or spends a slot or a focus point.
//!
//! Player Core pg. 299, Cast a Spell. Player Core pg. 319, Breathe Fire.
//! Player Core pg. 363, Telekinetic Projectile.

use crate::check::degree_of_success;
use crate::combatant::{CombatantId, Vital};
use crate::detection::{Detections, resolve_targeting};
use crate::dice::Dice;
use crate::grid::{Size, Square, distance};
use crate::modifiers::{Modifier, ModifierError};
use crate::save::Save;
use crate::spell::{
    Aim, AreaCreature, GeometryError, breathe_fire_cost, cone_membership, projectile_cost,
    roll_2d6, spell_attack_modifier, spell_dc,
};

use super::strike::{FlatGate, flat_gate, misses};
use super::{Encounter, EncounterError};

/// One hit or miss with a spell attack.
///
/// Player Core pg. 363, Telekinetic Projectile. Player Core pg. 401, Degree of Success.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpellAttackHit {
    /// Two actions with attack, cantrip, concentrate, and manipulate.
    ///
    /// Player Core pg. 363, Telekinetic Projectile.
    pub cost: crate::positioning::ActionCost,
    /// The attack d20. Player Core pg. 403, Spell Attack Rolls.
    pub natural: u8,
    /// Natural roll plus the spell attack modifier and the multiple attack penalty.
    ///
    /// Player Core pg. 403, Spell Attack Rolls. Player Core pg. 402, Multiple Attack Penalty.
    pub total: i32,
    /// Degree against baked Armor Class. Player Core pg. 401, Degree of Success.
    pub degree: crate::check::DegreeOfSuccess,
    /// Multiple attack penalty on this roll. It is not agile.
    ///
    /// Player Core pg. 402, Multiple Attack Penalty.
    pub map: i32,
    /// Spell DC from the same modifier, without the multiple attack penalty.
    ///
    /// Player Core pg. 403, Spell DC.
    pub spell_dc: i32,
    /// Damage dealt, or 0 on a miss. A critical success doubles the 2d6 once.
    ///
    /// Player Core pg. 363, Telekinetic Projectile. Player Core pg. 407, Doubling and Halving Damage.
    pub damage: i32,
}

/// A spell attack that rolled, or a flat check that did not affect the target.
///
/// Player Core pg. 363, Telekinetic Projectile. Player Core pg. 442, Concealed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpellAttackOutcome {
    /// The flat check spent the cast and rolled no attack d20.
    ///
    /// Player Core pg. 442, Concealed.
    Lost(super::LostStrike),
    /// The attack d20 was rolled. Player Core pg. 403, Spell Attack Rolls.
    Rolled(SpellAttackHit),
}

/// One Breathe Fire cast.
///
/// Player Core pg. 319, Breathe Fire.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BreatheFireOutcome {
    /// Two actions with concentrate, fire, and manipulate. Player Core pg. 319, Breathe Fire.
    pub cost: crate::positioning::ActionCost,
    /// The one 2d6, absent when nobody living is in the cone.
    ///
    /// Player Core pg. 406, Damage Rolls.
    pub rolled_damage: Option<i32>,
    /// One save per living creature in the cone, in ascending id order.
    ///
    /// Player Core pg. 404, Basic Saving Throws.
    pub saves: Vec<crate::save::SaveRoll>,
}

const PROJECTILE_RANGE_FEET: u32 = 30;

impl Encounter {
    /// Casts Breathe Fire, a 15-foot cone and one basic Reflex save per living creature.
    ///
    /// The 2d6 is rolled once, before the saves. A creature outside the cone is unchanged.
    /// The caster's square is not in the cone. Dead creatures in the cone draw no save.
    /// Standard cover is not added. The cast does not call `resolve_reactions`.
    ///
    /// Player Core pg. 319, Breathe Fire. Player Core pg. 404, Basic Saving Throws.
    /// Player Core pg. 428, Cone. Ruling R-13.
    #[expect(
        clippy::too_many_arguments,
        reason = "the cast names the caster, origin, size, aim, creatures, modifiers, and dice separately"
    )]
    pub fn breathe_fire(
        &mut self,
        caster: CombatantId,
        origin: Square,
        caster_size: Size,
        aim: Aim,
        creatures: &[AreaCreature],
        modifiers: &[Modifier],
        dice: &mut impl Dice,
    ) -> Result<BreatheFireOutcome, EncounterError> {
        self.prepare_caster(caster)?;
        reject_duplicates(creatures)?;
        for creature in creatures {
            self.member_index(creature.id)?;
        }
        let squares = cone_membership(origin, caster_size, aim, 15, creatures).map_err(geometry)?;
        let inside = |creature: &AreaCreature| squares.contains(&creature.square);
        for creature in creatures.iter().filter(|creature| inside(creature)) {
            if self.state(creature.id)?.vital != Vital::Dead
                && self.creature(creature.id)?.saves.is_none()
            {
                return Err(EncounterError::NoSavingThrows);
            }
        }
        let mut living: Vec<CombatantId> = creatures
            .iter()
            .filter(|creature| inside(creature))
            .map(|creature| creature.id)
            .filter(|id| {
                self.state(*id)
                    .map(|state| state.vital != Vital::Dead)
                    .unwrap_or(false)
            })
            .collect();
        living.sort();
        living.dedup();
        self.finish_cast(caster, false)?;
        if living.is_empty() {
            return Ok(BreatheFireOutcome {
                cost: breathe_fire_cost(),
                rolled_damage: None,
                saves: Vec::new(),
            });
        }
        let dc = self.caster_dc(caster, modifiers)?;
        let rolled = roll_2d6(dice);
        let mut saves = Vec::with_capacity(living.len());
        for id in living {
            saves.push(self.basic_save(id, Save::Reflex, dc, rolled, modifiers, dice)?);
        }
        Ok(BreatheFireOutcome {
            cost: breathe_fire_cost(),
            rolled_damage: Some(rolled),
            saves,
        })
    }

    /// Casts Telekinetic Projectile at one creature within 30 feet.
    ///
    /// Targets: 1 creature. An ally and the caster's own square are legal, and side is not
    /// read. The attack trait applies the multiple attack penalty. The penalty is not agile
    /// and is not part of the spell DC. A hit rolls 2d6 with no attribute modifier. A critical
    /// hit doubles that total once. There is no range increment and no line-of-effect test.
    ///
    /// Player Core pg. 363, Telekinetic Projectile. Player Core pg. 403, Spell Attack Rolls.
    /// Player Core pg. 402, Multiple Attack Penalty.
    #[expect(
        clippy::too_many_arguments,
        reason = "the cast names the caster, target, both squares, both sizes, modifiers, detection, and dice separately"
    )]
    pub fn telekinetic_projectile(
        &mut self,
        caster: CombatantId,
        target: CombatantId,
        caster_square: Square,
        target_square: Square,
        caster_size: Size,
        target_size: Size,
        modifiers: &[Modifier],
        detections: &Detections,
        dice: &mut impl Dice,
    ) -> Result<SpellAttackOutcome, EncounterError> {
        self.prepare_caster(caster)?;
        let target_state = self.state(target)?;
        if target_state.vital == Vital::Dead {
            return Err(EncounterError::TargetDead);
        }
        let feet = distance(caster_square, target_square, caster_size, target_size)
            .map_err(EncounterError::Grid)?;
        if feet > PROJECTILE_RANGE_FEET {
            return Err(EncounterError::OutOfSpellRange);
        }
        let casting = self
            .creature(caster)?
            .spellcasting
            .ok_or(EncounterError::NoSpellcasting)?;
        let spell_modifier = spell_attack_modifier(&casting, modifiers).map_err(overflow)?;
        let spell_dc = spell_dc(&casting, modifiers).map_err(overflow)?;
        let map = self.attack_trait_penalty(caster, super::Agility::NotAgile)?;
        let _ = self
            .attack_count(caster)?
            .checked_add(1)
            .ok_or(EncounterError::AttackCountOverflow)?;
        match flat_gate(resolve_targeting(detections, caster, target, false, dice)) {
            FlatGate::CannotTarget => return Err(EncounterError::CannotTarget),
            FlatGate::Lost(lost) => {
                self.finish_cast(caster, true)?;
                return Ok(SpellAttackOutcome::Lost(lost));
            }
            FlatGate::Roll => {}
        }
        let natural = dice.d20();
        let total = i32::from(natural)
            .checked_add(spell_modifier)
            .and_then(|sum| sum.checked_add(map))
            .ok_or_else(|| overflow(ModifierError::Overflow))?;
        let armor_class = self.creature(target)?.armor_class;
        let degree = degree_of_success(natural, total, armor_class);
        let damage = if misses(degree) {
            0
        } else {
            let rolled = roll_2d6(dice);
            let dealt = if degree == crate::check::DegreeOfSuccess::CriticalSuccess {
                rolled
                    .checked_mul(2)
                    .ok_or_else(|| overflow(ModifierError::Overflow))?
            } else {
                rolled
            };
            if dealt > 0 {
                let blow = if degree == crate::check::DegreeOfSuccess::CriticalSuccess {
                    crate::check::DegreeOfSuccess::CriticalSuccess
                } else {
                    crate::check::DegreeOfSuccess::Success
                };
                self.apply_damage(target, dealt, blow)?;
            }
            dealt
        };
        self.finish_cast(caster, true)?;
        Ok(SpellAttackOutcome::Rolled(SpellAttackHit {
            cost: projectile_cost(),
            natural,
            total,
            degree,
            map,
            spell_dc,
            damage,
        }))
    }

    fn prepare_caster(&self, id: CombatantId) -> Result<(), EncounterError> {
        self.member_index(id)?;
        self.require_active_alive(id)?;
        if self.creature(id)?.spellcasting.is_none() {
            return Err(EncounterError::NoSpellcasting);
        }
        if self.state(id)?.actions_remaining < 2 {
            return Err(EncounterError::InsufficientActions);
        }
        Ok(())
    }

    fn caster_dc(
        &self,
        caster: CombatantId,
        modifiers: &[Modifier],
    ) -> Result<i32, EncounterError> {
        let casting = self
            .creature(caster)?
            .spellcasting
            .ok_or(EncounterError::NoSpellcasting)?;
        spell_dc(&casting, modifiers).map_err(overflow)
    }

    fn finish_cast(&mut self, id: CombatantId, attack: bool) -> Result<(), EncounterError> {
        self.spend_actions(id, 2)?;
        if !attack {
            return Ok(());
        }
        let index = self.member_index(id)?;
        let member = self
            .members
            .get_mut(index)
            .ok_or(EncounterError::UnknownCombatant)?;
        member.attack_count = member
            .attack_count
            .checked_add(1)
            .ok_or(EncounterError::AttackCountOverflow)?;
        Ok(())
    }
}

fn reject_duplicates(creatures: &[AreaCreature]) -> Result<(), EncounterError> {
    for (index, creature) in creatures.iter().enumerate() {
        if creatures[..index]
            .iter()
            .any(|earlier| earlier.id == creature.id)
        {
            return Err(EncounterError::DuplicateAreaTarget);
        }
    }
    Ok(())
}

fn geometry(error: GeometryError) -> EncounterError {
    match error {
        GeometryError::UnsupportedCone => EncounterError::UnsupportedCone,
        GeometryError::Grid(error) => EncounterError::Grid(error),
        GeometryError::InvalidRadius | GeometryError::InvalidLength => {
            EncounterError::UnsupportedCone
        }
    }
}

fn overflow(error: ModifierError) -> EncounterError {
    EncounterError::AttackOverflow(error)
}
