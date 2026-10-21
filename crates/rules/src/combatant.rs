use std::fmt;

use crate::check::{CheckRoll, DegreeOfSuccess, resolve_check};
use crate::dice::{Dice, Faces};

/// Stable id for a creature in one encounter. Rules never use engine entity ids.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CombatantId(
    /// Numeric id. Not a roster slot.
    pub u32,
);

impl fmt::Display for CombatantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Number of damage dice. The constructor rejects a count that does not fit.
///
/// Player Core, Damage Rolls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DieCount(u8);

/// Why a die count was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DieCountError {
    /// The count was negative or did not fit in a `u8`.
    OutOfRange,
}

impl DieCount {
    /// Accepts `0..=u8::MAX`.
    ///
    /// Player Core, Damage Rolls.
    pub fn new(count: i32) -> Result<Self, DieCountError> {
        u8::try_from(count)
            .map(Self)
            .map_err(|_| DieCountError::OutOfRange)
    }

    /// The die count.
    pub const fn get(self) -> u8 {
        self.0
    }

    /// The die count as a contribution. Always fits in `i32`.
    pub fn to_i32(self) -> i32 {
        i32::from(self.0)
    }
}

/// Reach in feet. The constructor rejects a negative reach.
///
/// Player Core, Strike.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReachFeet(i32);

/// Why a reach was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReachError {
    /// The reach was negative.
    Negative,
}

impl ReachFeet {
    /// Accepts a non-negative reach.
    ///
    /// Player Core, Strike.
    pub fn new(feet: i32) -> Result<Self, ReachError> {
        if feet < 0 {
            return Err(ReachError::Negative);
        }
        Ok(Self(feet))
    }

    /// Reach in feet. `new` rejects a negative value, so this fits in `u32`.
    pub fn get(self) -> u32 {
        u32::try_from(self.0).unwrap_or_default()
    }

    /// The reach as a contribution.
    pub const fn to_i32(self) -> i32 {
        self.0
    }
}

/// Why a fold could not build a creature without wrapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoldError {
    /// A summed `i32` overflowed.
    Overflow,
    /// The summed die count does not fit in a [`DieCount`].
    DieCountOutOfRange,
    /// The summed face count is zero or does not fit in [`Faces`].
    FaceCountOutOfRange,
    /// The summed reach is negative.
    ReachOutOfRange,
}

/// One melee strike on a creature.
///
/// Player Core, Strike.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MeleeStrike {
    /// Attack bonus added to the d20. Player Core, Strike.
    pub attack_bonus: i32,
    /// Number of damage dice. Player Core, Damage Rolls.
    pub damage_dice: DieCount,
    /// Faces on each damage die. Zero is not a die. Player Core, Damage Rolls.
    pub damage_faces: Faces,
    /// Flat damage added after the dice. Player Core, Damage Rolls.
    pub damage_bonus: i32,
    /// Reach in feet. Player Core, Strike.
    pub reach_feet: ReachFeet,
    /// Agile lowers the multiple attack penalty for this weapon. Player Core, "Multiple Attack Penalty".
    pub agile: bool,
}

/// Shield numbers stored on the build. The AC bonus is this shield's own value.
///
/// Player Core, "Shield Statistics".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShieldTemplate {
    /// Hardness. Player Core, "Shield Statistics".
    pub hardness: i32,
    /// Maximum Hit Points. Player Core, "Shield Statistics".
    pub hit_points: i32,
    /// Broken Threshold. Player Core, "Shield Statistics".
    pub broken_threshold: i32,
    /// This shield's AC bonus, not a flat +2. Player Core, "Shield Statistics".
    pub shield_ac_bonus: i32,
}

/// Phase 1 inputs for one character. `uses_dying_rules: None` stores false.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildParts {
    /// Maximum Hit Points. Player Core, Hit Points.
    pub hit_points: i32,
    /// Armor Class. Player Core, Hit Points.
    pub armor_class: i32,
    /// Perception modifier. Player Core, "Getting a DC from a Modifier".
    pub perception: i32,
    /// Speed in feet. Player Core, Speed.
    pub speed_feet: i32,
    /// The one melee strike. Player Core, Strike.
    pub strike: MeleeStrike,
    /// Optional shield. Player Core, "Shield Statistics".
    pub shield: Option<ShieldTemplate>,
    /// Optional Stealth DC. Player Core, Seek.
    pub stealth_dc: Option<i32>,
    /// Optional Aid modifier. Player Core, Aid.
    pub aid_modifier: Option<i32>,
    /// None stores false. Player Core, "Getting Knocked Out".
    pub uses_dying_rules: Option<bool>,
}

/// Immutable inputs for one character. No setters, and no combatant id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterBuild {
    parts: BuildParts,
}

/// Why a character build was rejected. A rejected build is not derived.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplateError {
    /// Hit Points were 0 or less. Player Core, "Getting Knocked Out".
    NonPositiveHitPoints,
    /// Speed was negative. Player Core, Speed.
    NegativeSpeed,
}

impl CharacterBuild {
    /// Rejects non-positive Hit Points and a negative Speed. Speed 0 is stored.
    ///
    /// Player Core, "Getting Knocked Out" and Speed.
    pub fn try_new(parts: BuildParts) -> Result<Self, TemplateError> {
        if parts.hit_points <= 0 {
            return Err(TemplateError::NonPositiveHitPoints);
        }
        if parts.speed_feet < 0 {
            return Err(TemplateError::NegativeSpeed);
        }
        Ok(Self { parts })
    }
}

/// One derivation layer. Numerics are added. `Option` fields replace when `Some`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Contribution {
    /// Hit Points added by this layer. Player Core, Hit Points.
    pub hit_points: i32,
    /// Armor Class added by this layer. Player Core, Hit Points.
    pub armor_class: i32,
    /// Perception modifier added by this layer. Player Core, "Getting a DC from a Modifier".
    pub perception: i32,
    /// Speed in feet added by this layer. Player Core, Speed.
    pub speed_feet: i32,
    /// Attack bonus added by this layer. Player Core, Strike.
    pub attack_bonus: i32,
    /// Damage dice added by this layer. Player Core, Damage Rolls.
    pub damage_dice: i32,
    /// Damage die faces added by this layer. Player Core, Damage Rolls.
    pub damage_faces: i32,
    /// Damage bonus added by this layer. Player Core, Damage Rolls.
    pub damage_bonus: i32,
    /// Reach in feet added by this layer. Player Core, Strike.
    pub reach_feet: i32,
    /// Replaces the agile flag when `Some`. Player Core, "Multiple Attack Penalty".
    pub agile: Option<bool>,
    /// Replaces the shield when `Some`. Player Core, "Shield Statistics".
    pub shield: Option<ShieldTemplate>,
    /// Replaces the Stealth DC when `Some`. Player Core, Seek.
    pub stealth_dc: Option<i32>,
    /// Replaces the Aid modifier when `Some`. Player Core, Aid.
    pub aid_modifier: Option<i32>,
    /// Replaces the dying-rules flag when `Some`. Player Core, "Getting Knocked Out".
    pub uses_dying_rules: Option<bool>,
}

/// Copies the authored numbers as the first layer.
///
/// Player Core, Hit Points.
pub fn base_contribution(build: &CharacterBuild) -> Contribution {
    let parts = &build.parts;
    Contribution {
        hit_points: parts.hit_points,
        armor_class: parts.armor_class,
        perception: parts.perception,
        speed_feet: parts.speed_feet,
        attack_bonus: parts.strike.attack_bonus,
        damage_dice: parts.strike.damage_dice.to_i32(),
        damage_faces: parts.strike.damage_faces.to_i32(),
        damage_bonus: parts.strike.damage_bonus,
        reach_feet: parts.strike.reach_feet.to_i32(),
        agile: Some(parts.strike.agile),
        shield: parts.shield.clone(),
        stealth_dc: parts.stealth_dc,
        aid_modifier: parts.aid_modifier,
        uses_dying_rules: parts.uses_dying_rules,
    }
}

/// Phase 1 supplies only the base contribution.
///
/// Player Core, Hit Points.
pub fn contributions(build: &CharacterBuild) -> Vec<Contribution> {
    vec![base_contribution(build)]
}

/// Folds layers in order, starting from zeros and `None`.
///
/// Sums use checked addition. Die count, face count, and reach are built with their
/// constructors, so a value that does not fit is an error instead of a wrapped number.
///
/// Player Core, Hit Points.
pub fn fold_contributions(ordered: &[Contribution]) -> Result<Creature, FoldError> {
    let mut hit_points: i32 = 0;
    let mut armor_class: i32 = 0;
    let mut perception: i32 = 0;
    let mut speed_feet: i32 = 0;
    let mut attack_bonus: i32 = 0;
    let mut damage_dice: i32 = 0;
    let mut damage_faces: i32 = 0;
    let mut damage_bonus: i32 = 0;
    let mut reach_feet: i32 = 0;
    let mut agile = None;
    let mut shield = None;
    let mut stealth_dc = None;
    let mut aid_modifier = None;
    let mut uses_dying_rules = None;

    for layer in ordered {
        hit_points = hit_points
            .checked_add(layer.hit_points)
            .ok_or(FoldError::Overflow)?;
        armor_class = armor_class
            .checked_add(layer.armor_class)
            .ok_or(FoldError::Overflow)?;
        perception = perception
            .checked_add(layer.perception)
            .ok_or(FoldError::Overflow)?;
        speed_feet = speed_feet
            .checked_add(layer.speed_feet)
            .ok_or(FoldError::Overflow)?;
        attack_bonus = attack_bonus
            .checked_add(layer.attack_bonus)
            .ok_or(FoldError::Overflow)?;
        damage_dice = damage_dice
            .checked_add(layer.damage_dice)
            .ok_or(FoldError::Overflow)?;
        damage_faces = damage_faces
            .checked_add(layer.damage_faces)
            .ok_or(FoldError::Overflow)?;
        damage_bonus = damage_bonus
            .checked_add(layer.damage_bonus)
            .ok_or(FoldError::Overflow)?;
        reach_feet = reach_feet
            .checked_add(layer.reach_feet)
            .ok_or(FoldError::Overflow)?;
        if let Some(value) = layer.agile {
            agile = Some(value);
        }
        if layer.shield.is_some() {
            shield.clone_from(&layer.shield);
        }
        if let Some(value) = layer.stealth_dc {
            stealth_dc = Some(value);
        }
        if let Some(value) = layer.aid_modifier {
            aid_modifier = Some(value);
        }
        if let Some(value) = layer.uses_dying_rules {
            uses_dying_rules = Some(value);
        }
    }

    let damage_dice = DieCount::new(damage_dice).map_err(|_| FoldError::DieCountOutOfRange)?;
    let damage_faces =
        Faces::try_from_i32(damage_faces).map_err(|_| FoldError::FaceCountOutOfRange)?;
    let reach_feet = ReachFeet::new(reach_feet).map_err(|_| FoldError::ReachOutOfRange)?;

    Ok(Creature {
        hit_points,
        armor_class,
        perception,
        speed_feet,
        strike: MeleeStrike {
            attack_bonus,
            damage_dice,
            damage_faces,
            damage_bonus,
            reach_feet,
            agile: agile.unwrap_or(false),
        },
        shield,
        stealth_dc,
        aid_modifier,
        uses_dying_rules: uses_dying_rules.unwrap_or(false),
    })
}

fn creature_from_validated(build: &CharacterBuild) -> Creature {
    let parts = &build.parts;
    Creature {
        hit_points: parts.hit_points,
        armor_class: parts.armor_class,
        perception: parts.perception,
        speed_feet: parts.speed_feet,
        strike: parts.strike.clone(),
        shield: parts.shield.clone(),
        stealth_dc: parts.stealth_dc,
        aid_modifier: parts.aid_modifier,
        uses_dying_rules: parts.uses_dying_rules.unwrap_or(false),
    }
}

/// Pure derivation. Does not roll dice, do I/O, or read global state.
///
/// Phase 1 folds the base contribution. Those numbers were stored with the validated
/// constructors, so a fold error copies that strike instead of wrapping.
///
/// Player Core, Hit Points are fixed by the creature, not by a roll.
pub fn derive(build: &CharacterBuild) -> Creature {
    match fold_contributions(&contributions(build)) {
        Ok(creature) => creature,
        Err(_) => creature_from_validated(build),
    }
}

/// Read-only result of derivation. Hit Points and shield Hit Points are maximums.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Creature {
    /// Maximum Hit Points. Player Core, Hit Points.
    pub hit_points: i32,
    /// Armor Class. Player Core, Hit Points.
    pub armor_class: i32,
    /// Perception modifier. Player Core, "Getting a DC from a Modifier".
    pub perception: i32,
    /// Speed in feet. Player Core, Speed.
    pub speed_feet: i32,
    /// The one melee strike. Player Core, Strike.
    pub strike: MeleeStrike,
    /// Optional shield, with maximum Hit Points. Player Core, "Shield Statistics".
    pub shield: Option<ShieldTemplate>,
    /// Optional Stealth DC. Player Core, Seek.
    pub stealth_dc: Option<i32>,
    /// Optional Aid modifier. Player Core, Aid.
    pub aid_modifier: Option<i32>,
    /// False when the build omitted it. Player Core, "Getting Knocked Out".
    pub uses_dying_rules: bool,
}

/// What 0 current Hit Points means for this combatant. Encounter-core owns the dying value.
///
/// Player Core, "Getting Knocked Out".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vital {
    /// Current Hit Points are above 0. Player Core, "Getting Knocked Out".
    Alive,
    /// 0 Hit Points and `uses_dying_rules` is false. Player Core, "Getting Knocked Out".
    Dead,
    /// 0 Hit Points and dying applies. No dying value is chosen. Player Core, "Getting Knocked Out".
    EnterDying,
}

/// Empty in Phase 1. The vec is the slot later conditions will fill.
///
/// Player Core, Encounter Mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConditionSlot;

/// Runtime combat data, keyed by [`CombatantId`]. Never written back onto the build or the creature.
///
/// Player Core, Encounter Mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombatantState {
    /// Key for this combatant. Player Core, Encounter Mode.
    pub id: CombatantId,
    /// Current Hit Points. Player Core, "Getting Knocked Out".
    pub hit_points: i32,
    /// Temporary Hit Points. Player Core, Hit Points.
    pub temp_hit_points: i32,
    /// Condition slot. Empty in Phase 1. Player Core, Encounter Mode.
    pub conditions: Vec<ConditionSlot>,
    /// True when 0 Hit Points entered dying. Not a dying value. Player Core, "Getting Knocked Out".
    pub dying_entered: bool,
    /// Wounded value. This change does not increase it. Player Core, "Getting Knocked Out".
    pub wounded: i32,
    /// Current shield Hit Points, if the creature has a shield. Player Core, "Shield Statistics".
    pub shield_hit_points: Option<i32>,
    /// Raised shield. Starts false. Player Core, "Shield Statistics".
    pub shield_raised: bool,
    /// Actions left. Encounter-core sets the turn's 3. Player Core, Encounter Mode.
    pub actions_remaining: u8,
    /// Reaction ready. Encounter-core readies it. Player Core, Encounter Mode.
    pub reaction_ready: bool,
    /// Alive, dead, or entering dying. Player Core, "Getting Knocked Out".
    pub vital: Vital,
}

/// Initializes state from a creature. Actions and the reaction stay unset for encounter-core.
///
/// Player Core, Encounter Mode.
pub fn new_combatant_state(id: CombatantId, creature: &Creature) -> CombatantState {
    CombatantState {
        id,
        hit_points: creature.hit_points,
        temp_hit_points: 0,
        conditions: Vec::new(),
        dying_entered: false,
        wounded: 0,
        shield_hit_points: creature.shield.as_ref().map(|shield| shield.hit_points),
        shield_raised: false,
        actions_remaining: 0,
        reaction_ready: false,
        vital: Vital::Alive,
    }
}

/// Rolls `faces` once, adds `bonus`, and subtracts that from current Hit Points, floored at 0.
///
/// A negative total is clamped to 0, so this never raises Hit Points. The addition and the
/// subtraction are checked, so an overflowing `i32` does not panic. A sum above `i32::MAX` drops
/// Hit Points to 0.
///
/// At 0 Hit Points, `uses_dying_rules` false marks the state dead. True records that dying applies
/// and does not choose a dying value. The creature and the build are not written.
///
/// Player Core, "Getting Knocked Out".
pub fn apply_dice_damage(
    state: &mut CombatantState,
    creature: &Creature,
    dice: &mut impl Dice,
    faces: Faces,
    bonus: i32,
) {
    let rolled = i32::from(dice.roll(faces));
    let amount = match rolled.checked_add(bonus) {
        Some(total) if total > 0 => total,
        // Zero and negative totals deal no damage, so Hit Points are not raised.
        Some(_) => 0,
        // The sum is above `i32::MAX`, so Hit Points fall to 0.
        None => {
            state.hit_points = 0;
            note_hit_points(state, creature);
            return;
        }
    };
    state.hit_points = match state.hit_points.checked_sub(amount) {
        Some(next) => next.max(0),
        // `amount` is non-negative, so a missing difference is below 0.
        None => 0,
    };
    note_hit_points(state, creature);
}

fn note_hit_points(state: &mut CombatantState, creature: &Creature) {
    if state.hit_points > 0 {
        state.vital = Vital::Alive;
    } else if creature.uses_dying_rules {
        state.vital = Vital::EnterDying;
        state.dying_entered = true;
    } else {
        state.vital = Vital::Dead;
    }
}

/// Stealth DC placed on a build: 10 plus the Stealth modifier. Not used inside [`derive`].
///
/// Player Core, "Getting a DC from a Modifier" and Seek.
pub fn stealth_dc_from_modifier(stealth_modifier: crate::check::DcModifier) -> i32 {
    crate::check::dc_from_modifier(stealth_modifier)
}

/// Circumstance bonus from an Aid degree. Critical success is the fixed +2.
///
/// Player Core, Aid. Master and legendary are not chosen: the baked modifier has no rank.
pub fn aid_circumstance_bonus(degree: DegreeOfSuccess) -> i32 {
    match degree {
        DegreeOfSuccess::CriticalSuccess => 2,
        DegreeOfSuccess::Success => 1,
        DegreeOfSuccess::Failure => 0,
        DegreeOfSuccess::CriticalFailure => -1,
    }
}

/// Aid check against DC 15 using the creature's Aid modifier. `None` when the creature has none,
/// or when that modifier cannot be added to a die face inside `i32`.
///
/// Player Core, Aid.
pub fn aid_check(dice: &mut impl Dice, creature: &Creature) -> Option<(CheckRoll, i32)> {
    let modifier = crate::check::CheckModifier::new(creature.aid_modifier?).ok()?;
    let roll = resolve_check(dice, modifier, 15);
    let bonus = aid_circumstance_bonus(roll.degree);
    Some((roll, bonus))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::{
        CheckModifier, DcModifier, DegreeOfSuccess, degree_of_success, resolve_check,
    };
    use crate::dice::{Faces, ScriptedDice};

    fn d8() -> Faces {
        Faces::new(8).expect("d8")
    }

    fn dc_modifier(value: i32) -> DcModifier {
        DcModifier::new(value).expect("scenario DC modifier fits")
    }

    fn check_modifier(value: i32) -> CheckModifier {
        CheckModifier::new(value).expect("scenario modifier fits")
    }

    fn strike(
        attack: i32,
        dice: u8,
        faces: u8,
        bonus: i32,
        reach: u32,
        agile: bool,
    ) -> MeleeStrike {
        MeleeStrike {
            attack_bonus: attack,
            damage_dice: DieCount::new(i32::from(dice)).expect("dice"),
            damage_faces: Faces::new(faces).expect("faces"),
            damage_bonus: bonus,
            reach_feet: ReachFeet::new(i32::try_from(reach).expect("reach")).expect("reach"),
            agile,
        }
    }

    fn fighter_parts() -> BuildParts {
        BuildParts {
            hit_points: 20,
            armor_class: 18,
            perception: 5,
            speed_feet: 25,
            strike: strike(7, 1, 8, 4, 5, false),
            shield: None,
            stealth_dc: None,
            aid_modifier: None,
            uses_dying_rules: None,
        }
    }

    fn build(parts: BuildParts) -> CharacterBuild {
        CharacterBuild::try_new(parts).expect("valid build")
    }

    /// Player Core, Hit Points are fixed by the creature, not by a roll.
    #[test]
    fn derive_is_deterministic() {
        let parts = fighter_parts();
        let original = build(parts.clone());
        let dice = ScriptedDice::new([8]);
        let first = derive(&original);
        let second = derive(&original);
        assert_eq!(first, second);
        assert_eq!(derive(&original.clone()), first);
        let rebuilt = build(parts);
        assert_eq!(original, rebuilt);
        assert_eq!(derive(&rebuilt), first);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Hit Points.
    #[test]
    fn later_contribution_folds_after_the_base() {
        let mut parts = fighter_parts();
        parts.stealth_dc = Some(17);
        let original = build(parts);
        let layers = contributions(&original);
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0], base_contribution(&original));
        assert_eq!(derive(&original).armor_class, 18);

        let mut ordered = layers;
        // Stealth is replaced, not added. A reversed fold would keep the base DC of 17.
        ordered.push(Contribution {
            armor_class: 1,
            stealth_dc: Some(4),
            ..Contribution::default()
        });
        assert_eq!(ordered[0].armor_class, 18);
        assert_eq!(ordered[1].armor_class, 1);
        let folded = fold_contributions(&ordered).expect("folded numbers fit");
        assert_eq!(folded.armor_class, 19);
        assert_eq!(folded.stealth_dc, Some(4));
        assert_eq!(derive(&original).armor_class, 18);
    }

    /// Player Core, Hit Points.
    /// Player Core, Strike.
    /// Player Core, Speed.
    #[test]
    fn baked_statistics_stay_as_authored() {
        let original = build(fighter_parts());
        let before = original.clone();
        let creature = derive(&original);
        assert_eq!(original, before);
        assert_eq!(creature.hit_points, 20);
        assert_eq!(creature.armor_class, 18);
        assert_eq!(creature.perception, 5);
        assert_eq!(creature.speed_feet, 25);
        assert_eq!(creature.strike.attack_bonus, 7);
        assert_eq!(creature.strike.damage_dice.get(), 1);
        assert_eq!(creature.strike.damage_faces.get(), 8);
        assert_eq!(creature.strike.damage_bonus, 4);
        assert_eq!(creature.strike.reach_feet.get(), 5);
        assert!(!creature.strike.agile);
        assert!(creature.shield.is_none());
        assert_eq!(creature.stealth_dc, None);
        assert_eq!(creature.aid_modifier, None);
        assert!(!creature.uses_dying_rules);

        let mut agile_parts = fighter_parts();
        agile_parts.strike = strike(6, 1, 6, 2, 5, true);
        let agile = derive(&build(agile_parts));
        assert!(agile.strike.agile);
        assert_eq!(agile.strike.attack_bonus, 6);
        assert_eq!(agile.strike.damage_dice.get(), 1);
        assert_eq!(agile.strike.damage_faces.get(), 6);
        assert_eq!(agile.strike.damage_bonus, 2);
        assert_eq!(agile.strike.reach_feet.get(), 5);
    }

    /// Player Core, "Shield Statistics". A buckler grants +1.
    #[test]
    fn shield_ac_bonus_is_not_rewritten_to_plus_two() {
        let buckler = ShieldTemplate {
            hardness: 3,
            hit_points: 8,
            broken_threshold: 4,
            shield_ac_bonus: 1,
        };
        let mut buckler_parts = fighter_parts();
        buckler_parts.shield = Some(buckler.clone());
        let creature = derive(&build(buckler_parts));
        assert_eq!(creature.shield, Some(buckler));

        let steel = ShieldTemplate {
            hardness: 5,
            hit_points: 20,
            broken_threshold: 10,
            shield_ac_bonus: 2,
        };
        let mut steel_parts = fighter_parts();
        steel_parts.shield = Some(steel.clone());
        assert_eq!(derive(&build(steel_parts)).shield, Some(steel));

        assert!(derive(&build(fighter_parts())).shield.is_none());
    }

    /// Player Core, "Seek".
    /// Player Core, "Getting a DC from a Modifier".
    #[test]
    fn stealth_dc_from_modifier_is_ten_plus() {
        assert_eq!(stealth_dc_from_modifier(dc_modifier(7)), 17);

        let mut parts = fighter_parts();
        parts.stealth_dc = Some(stealth_dc_from_modifier(dc_modifier(7)));
        let creature = derive(&build(parts));
        assert_eq!(creature.stealth_dc, Some(17));

        let before = creature.clone();
        let mut dice = ScriptedDice::new([12]);
        let seek = resolve_check(
            &mut dice,
            check_modifier(creature.perception),
            creature.stealth_dc.unwrap(),
        );
        assert_eq!(seek.total, 17);
        assert_eq!(seek.degree, DegreeOfSuccess::Success);
        assert_eq!(creature, before);
        assert_eq!(dice.remaining(), 0);

        assert_eq!(derive(&build(fighter_parts())).stealth_dc, None);
    }

    /// Player Core, "Aid".
    ///
    /// Deviation:
    /// - Rule: critical success grants +2 circumstance, +3 at master, and +4 at legendary.
    /// - Why: the baked modifier has no proficiency rank, so master and legendary cannot be chosen.
    /// - Substitute: critical success on this Aid check always grants +2 circumstance.
    /// - Intent: a critical success still helps more than a success.
    #[test]
    fn aid_check_degrees_use_the_fixed_critical_bonus() {
        let mut parts = fighter_parts();
        parts.aid_modifier = Some(7);
        let creature = derive(&build(parts));
        let before = creature.clone();

        let (success, success_bonus) = aid_check(&mut ScriptedDice::new([10]), &creature).unwrap();
        assert_eq!(success.total, 17);
        assert_eq!(success.degree, DegreeOfSuccess::Success);
        assert_eq!(success_bonus, 1);
        assert_eq!(creature, before);

        let (critical, critical_bonus) =
            aid_check(&mut ScriptedDice::new([18]), &creature).unwrap();
        assert_eq!(critical.total, 25);
        assert_eq!(critical.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(critical_bonus, 2);

        let (failure, failure_bonus) = aid_check(&mut ScriptedDice::new([4]), &creature).unwrap();
        assert_eq!(failure.total, 11);
        assert_eq!(failure.degree, DegreeOfSuccess::Failure);
        assert_eq!(failure_bonus, 0);

        let mut crit_fail_parts = fighter_parts();
        crit_fail_parts.aid_modifier = Some(4);
        let crit_fail_creature = derive(&build(crit_fail_parts));
        let (crit_fail, crit_fail_bonus) =
            aid_check(&mut ScriptedDice::new([1]), &crit_fail_creature).unwrap();
        assert_eq!(crit_fail.total, 5);
        assert_eq!(crit_fail.degree, DegreeOfSuccess::CriticalFailure);
        assert_eq!(crit_fail_bonus, -1);
        assert_eq!(
            degree_of_success(10, 5, 15),
            DegreeOfSuccess::CriticalFailure
        );

        let mut nat_20_parts = fighter_parts();
        nat_20_parts.aid_modifier = Some(0);
        let nat_20_creature = derive(&build(nat_20_parts));
        let (nat_20, nat_20_bonus) =
            aid_check(&mut ScriptedDice::new([20]), &nat_20_creature).unwrap();
        assert_eq!(nat_20.total, 20);
        assert_eq!(degree_of_success(10, 20, 15), DegreeOfSuccess::Success);
        assert_eq!(nat_20.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(nat_20_bonus, 2);
        assert_eq!(nat_20_creature, derive(&build(fighter_with_aid(0))));

        assert!(
            aid_check(
                &mut ScriptedDice::new([10]),
                &derive(&build(fighter_parts()))
            )
            .is_none()
        );
    }

    fn fighter_with_aid(modifier: i32) -> BuildParts {
        let mut parts = fighter_parts();
        parts.aid_modifier = Some(modifier);
        parts
    }

    /// Player Core, Aid.
    #[test]
    fn template_does_not_store_aid_preparation() {
        let creature = derive(&build(fighter_with_aid(6)));
        assert_eq!(creature.aid_modifier, Some(6));
    }

    /// Player Core, Encounter Mode.
    #[test]
    fn fresh_state_matches_and_mutation_does_not_change_creature() {
        let mut parts = fighter_parts();
        parts.shield = Some(ShieldTemplate {
            hardness: 3,
            hit_points: 8,
            broken_threshold: 4,
            shield_ac_bonus: 1,
        });
        let original = build(parts);
        let before_build = original.clone();
        let creature = derive(&original);
        let before_creature = creature.clone();

        let first = new_combatant_state(CombatantId(7), &creature);
        let second = new_combatant_state(CombatantId(7), &creature);
        assert_eq!(first, second);
        assert_eq!(first.hit_points, creature.hit_points);
        assert_eq!(first.temp_hit_points, 0);
        assert!(!first.shield_raised);
        assert_eq!(first.shield_hit_points, Some(8));
        assert_eq!(first.actions_remaining, 0);
        assert!(!first.reaction_ready);
        assert_eq!(first.vital, Vital::Alive);
        assert!(first.conditions.is_empty());
        assert_eq!(CombatantId(7).to_string(), "7");

        let mut state = new_combatant_state(CombatantId(0), &creature);
        state.hit_points = 12;
        assert_eq!(creature, before_creature);
        assert_eq!(creature.hit_points, 20);
        assert_eq!(original, before_build);
    }

    /// Player Core, Encounter Mode.
    #[test]
    fn any_roster_length_is_valid() {
        let original = build(fighter_parts());
        let creature = derive(&original);
        let before = creature.clone();
        let roster: Vec<_> = (0..6)
            .map(|id| new_combatant_state(CombatantId(id), &creature))
            .collect();
        assert_eq!(roster.len(), 6);
        assert_eq!(roster[0].id, CombatantId(0));
        assert_eq!(roster[5].id, CombatantId(5));

        let wide = new_combatant_state(CombatantId(4_000_000_000), &creature);
        let also = new_combatant_state(CombatantId(0), &creature);
        assert_eq!(wide.id, CombatantId(4_000_000_000));
        assert_eq!(also.id, CombatantId(0));
        assert_eq!(wide.id.to_string(), "4000000000");
        assert_eq!(creature, before);
    }

    /// Player Core, "Getting Knocked Out".
    /// Player Core, "Movement".
    #[test]
    fn validation_rejects_non_positive_hp_and_negative_speed() {
        let mut zero_hp = fighter_parts();
        zero_hp.hit_points = 0;
        assert_eq!(
            CharacterBuild::try_new(zero_hp).unwrap_err(),
            TemplateError::NonPositiveHitPoints
        );

        let mut negative_hp = fighter_parts();
        negative_hp.hit_points = -1;
        assert_eq!(
            CharacterBuild::try_new(negative_hp).unwrap_err(),
            TemplateError::NonPositiveHitPoints
        );

        let mut negative_speed = fighter_parts();
        negative_speed.speed_feet = -5;
        negative_speed.hit_points = 12;
        assert_eq!(
            CharacterBuild::try_new(negative_speed).unwrap_err(),
            TemplateError::NegativeSpeed
        );

        let mut speed_zero = fighter_parts();
        speed_zero.hit_points = 12;
        speed_zero.speed_feet = 0;
        let creature = derive(&build(speed_zero));
        assert_eq!(creature.speed_feet, 0);
        assert_eq!(creature.hit_points, 12);
    }

    /// Player Core, "Getting Knocked Out".
    #[test]
    fn zero_hp_without_dying_rules_is_dead() {
        let original = build(fighter_with_hp(9, None));
        let creature = derive(&original);
        assert!(!creature.uses_dying_rules);
        assert_eq!(creature.hit_points, 9);
        let before = creature.clone();

        let mut state = new_combatant_state(CombatantId(1), &creature);
        apply_dice_damage(&mut state, &creature, &mut ScriptedDice::new([8]), d8(), 1);
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.vital, Vital::Dead);
        assert!(!state.dying_entered);
        assert!(state.conditions.is_empty());
        assert_eq!(state.wounded, 0);
        assert_eq!(creature, before);

        let mut grazed = new_combatant_state(CombatantId(1), &creature);
        apply_dice_damage(&mut grazed, &creature, &mut ScriptedDice::new([1]), d8(), 1);
        assert_eq!(grazed.hit_points, 7);
        assert_eq!(grazed.vital, Vital::Alive);
        assert!(!grazed.dying_entered);
        assert_eq!(creature.hit_points, 9);
    }

    /// Player Core, "Getting Knocked Out".
    #[test]
    fn zero_hp_with_dying_rules_enters_dying() {
        let original = build(fighter_with_hp(9, Some(true)));
        let creature = derive(&original);
        assert!(creature.uses_dying_rules);
        let before = creature.clone();

        let mut state = new_combatant_state(CombatantId(1), &creature);
        apply_dice_damage(&mut state, &creature, &mut ScriptedDice::new([8]), d8(), 1);
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.vital, Vital::EnterDying);
        assert!(state.dying_entered);
        assert_ne!(state.vital, Vital::Dead);
        assert!(state.conditions.is_empty());
        assert_eq!(state.wounded, 0);
        assert_eq!(creature, before);
        assert_eq!(creature.hit_points, 9);
    }

    fn fighter_with_hp(hit_points: i32, uses_dying_rules: Option<bool>) -> BuildParts {
        let mut parts = fighter_parts();
        parts.hit_points = hit_points;
        parts.uses_dying_rules = uses_dying_rules;
        parts
    }

    fn buckler() -> ShieldTemplate {
        ShieldTemplate {
            hardness: 3,
            hit_points: 8,
            broken_threshold: 4,
            shield_ac_bonus: 1,
        }
    }

    /// Player Core, Hit Points.
    #[test]
    fn the_build_is_unchanged_by_derive() {
        let dice = ScriptedDice::new([8]);
        let original = build(fighter_parts());
        let before = original.clone();
        let _ = derive(&original);
        assert_eq!(original, before);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Hit Points.
    #[test]
    fn combat_does_not_write_the_build() {
        let dice = ScriptedDice::new([8]);
        let original = build(fighter_parts());
        let before = original.clone();
        let creature = derive(&original);
        let mut state = new_combatant_state(CombatantId(0), &creature);
        state.hit_points = 12;
        assert_eq!(original, before);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Hit Points.
    #[test]
    fn calling_derive_twice_matches() {
        let original = build(fighter_parts());
        let dice = ScriptedDice::new([8]);
        assert_eq!(derive(&original), derive(&original));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Hit Points.
    #[test]
    fn a_clone_and_an_equal_build_match() {
        let dice = ScriptedDice::new([8]);
        let parts = fighter_parts();
        let original = build(parts.clone());
        let cloned = original.clone();
        let rebuilt = build(parts);
        assert_eq!(original, rebuilt);
        assert_eq!(derive(&original), derive(&cloned));
        assert_eq!(derive(&cloned), derive(&rebuilt));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Hit Points.
    #[test]
    fn phase_1_folds_only_the_base() {
        let dice = ScriptedDice::new([8]);
        let original = build(fighter_parts());
        let layers = contributions(&original);
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].armor_class, 18);
        assert_eq!(derive(&original).armor_class, 18);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Hit Points.
    #[test]
    fn a_later_contribution_is_folded_after_the_base() {
        let dice = ScriptedDice::new([8]);
        let mut parts = fighter_parts();
        parts.stealth_dc = Some(17);
        let original = build(parts);
        let mut ordered = contributions(&original);
        ordered.push(Contribution {
            armor_class: 1,
            stealth_dc: Some(4),
            ..Contribution::default()
        });
        let folded = fold_contributions(&ordered).expect("folded numbers fit");
        assert_eq!(ordered[0].armor_class, 18);
        assert_eq!(folded.armor_class, 19);
        assert_eq!(folded.stealth_dc, Some(4));
        assert_eq!(derive(&original).armor_class, 18);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Hit Points.
    #[test]
    fn authored_numbers_are_kept() {
        let dice = ScriptedDice::new([8]);
        let creature = derive(&build(fighter_parts()));
        assert_eq!(creature.hit_points, 20);
        assert_eq!(creature.armor_class, 18);
        assert_eq!(creature.perception, 5);
        assert_eq!(creature.speed_feet, 25);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Strike.
    #[test]
    fn strike_fields_are_kept() {
        let dice = ScriptedDice::new([8]);
        let strike = derive(&build(fighter_parts())).strike;
        assert_eq!(strike.attack_bonus, 7);
        assert_eq!(strike.damage_dice.get(), 1);
        assert_eq!(strike.damage_faces.get(), 8);
        assert_eq!(strike.damage_bonus, 4);
        assert_eq!(strike.reach_feet.get(), 5);
        assert!(!strike.agile);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Multiple Attack Penalty".
    #[test]
    fn the_agile_flag_is_kept() {
        let dice = ScriptedDice::new([8]);
        let mut parts = fighter_parts();
        parts.strike = strike(6, 1, 6, 2, 5, true);
        let kept = derive(&build(parts)).strike;
        assert!(kept.agile);
        assert_eq!(kept.attack_bonus, 6);
        assert_eq!(kept.damage_dice.get(), 1);
        assert_eq!(kept.damage_faces.get(), 6);
        assert_eq!(kept.damage_bonus, 2);
        assert_eq!(kept.reach_feet.get(), 5);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Shield Statistics".
    #[test]
    fn a_buckler_bonus_stays_plus_1() {
        let dice = ScriptedDice::new([8]);
        let shield = buckler();
        let mut parts = fighter_parts();
        parts.shield = Some(shield.clone());
        assert_eq!(derive(&build(parts)).shield, Some(shield));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Shield Statistics".
    #[test]
    fn a_second_shield_keeps_its_own_bonus() {
        let dice = ScriptedDice::new([8]);
        let mut first_parts = fighter_parts();
        first_parts.shield = Some(buckler());
        let steel = ShieldTemplate {
            hardness: 5,
            hit_points: 20,
            broken_threshold: 10,
            shield_ac_bonus: 2,
        };
        let mut second_parts = fighter_parts();
        second_parts.shield = Some(steel);
        assert_eq!(
            derive(&build(first_parts)).shield.unwrap().shield_ac_bonus,
            1
        );
        assert_eq!(
            derive(&build(second_parts)).shield.unwrap().shield_ac_bonus,
            2
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Shield Statistics".
    #[test]
    fn no_shield_is_omitted() {
        let dice = ScriptedDice::new([8]);
        assert!(derive(&build(fighter_parts())).shield.is_none());
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Getting a DC from a Modifier".
    #[test]
    fn stealth_modifier_plus_10() {
        let dice = ScriptedDice::new([8]);
        let mut parts = fighter_parts();
        parts.stealth_dc = Some(stealth_dc_from_modifier(dc_modifier(7)));
        assert_eq!(derive(&build(parts)).stealth_dc, Some(17));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Seek.
    #[test]
    fn seek_uses_the_derived_dc() {
        let mut parts = fighter_parts();
        parts.stealth_dc = Some(17);
        let creature = derive(&build(parts));
        let before = creature.clone();
        let mut dice = ScriptedDice::new([12]);
        let seek = resolve_check(&mut dice, check_modifier(5), creature.stealth_dc.unwrap());
        assert_eq!(seek.total, 17);
        assert_eq!(seek.degree, DegreeOfSuccess::Success);
        assert_eq!(dice.remaining(), 0);
        assert_eq!(creature, before);
    }

    /// Player Core, Seek.
    #[test]
    fn omitted_stealth_dc_stores_nothing() {
        let dice = ScriptedDice::new([8]);
        assert_eq!(derive(&build(fighter_parts())).stealth_dc, None);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    ///
    /// Deviation: critical success is +2, not +3 at master or +4 at legendary, because the baked modifier has no rank. A critical success still helps more than a success.
    #[test]
    fn success_grants_plus_1() {
        let creature = derive(&build(fighter_with_aid(7)));
        let before = creature.clone();
        let (roll, bonus) = aid_check(&mut ScriptedDice::new([10]), &creature).unwrap();
        assert_eq!(roll.total, 17);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(bonus, 1);
        assert_eq!(creature, before);
    }

    /// Player Core, Aid.
    ///
    /// Deviation: critical success is always +2. The baked modifier has no rank, so master +3 and legendary +4 are not chosen. A critical success still helps more than a success.
    #[test]
    fn critical_success_grants_plus_2() {
        let creature = derive(&build(fighter_with_aid(7)));
        let (roll, bonus) = aid_check(&mut ScriptedDice::new([18]), &creature).unwrap();
        assert_eq!(roll.total, 25);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(bonus, 2);
    }

    /// Player Core, Aid.
    #[test]
    fn failure_grants_0() {
        let creature = derive(&build(fighter_with_aid(7)));
        let (roll, bonus) = aid_check(&mut ScriptedDice::new([4]), &creature).unwrap();
        assert_eq!(roll.total, 11);
        assert_eq!(roll.degree, DegreeOfSuccess::Failure);
        assert_eq!(bonus, 0);
    }

    /// Player Core, Aid.
    #[test]
    fn critical_failure_grants_minus_1() {
        let creature = derive(&build(fighter_with_aid(4)));
        let (roll, bonus) = aid_check(&mut ScriptedDice::new([1]), &creature).unwrap();
        assert_eq!(roll.total, 5);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalFailure);
        assert_eq!(bonus, -1);
    }

    /// Player Core, Aid.
    ///
    /// Deviation: the natural 20 critical success grants +2, not the master or legendary value. The baked modifier has no rank. A critical success still helps more than a success.
    #[test]
    fn a_natural_20_uses_the_fixed_critical_bonus() {
        let creature = derive(&build(fighter_with_aid(0)));
        let (roll, bonus) = aid_check(&mut ScriptedDice::new([20]), &creature).unwrap();
        assert_eq!(roll.total, 20);
        assert_eq!(degree_of_success(10, 20, 15), DegreeOfSuccess::Success);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(bonus, 2);
    }

    /// Player Core, Aid.
    #[test]
    fn no_prepared_flag_is_required() {
        let dice = ScriptedDice::new([8]);
        let creature = derive(&build(fighter_with_aid(6)));
        assert_eq!(creature.aid_modifier, Some(6));
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Aid.
    #[test]
    fn an_aid_modifier_past_i32_is_not_rolled() {
        let creature = derive(&build(fighter_with_aid(i32::MAX)));
        let mut dice = ScriptedDice::new([10]);
        assert!(aid_check(&mut dice, &creature).is_none());
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Getting Knocked Out".
    #[test]
    fn zero_hit_points_is_rejected() {
        let dice = ScriptedDice::new([8]);
        let mut parts = fighter_parts();
        parts.hit_points = 0;
        assert_eq!(
            CharacterBuild::try_new(parts).unwrap_err(),
            TemplateError::NonPositiveHitPoints
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Getting Knocked Out".
    #[test]
    fn negative_hit_points_are_rejected() {
        let dice = ScriptedDice::new([8]);
        let mut parts = fighter_parts();
        parts.hit_points = -1;
        assert_eq!(
            CharacterBuild::try_new(parts).unwrap_err(),
            TemplateError::NonPositiveHitPoints
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Speed.
    #[test]
    fn negative_speed_is_rejected() {
        let dice = ScriptedDice::new([8]);
        let mut parts = fighter_parts();
        parts.hit_points = 12;
        parts.speed_feet = -5;
        assert_eq!(
            CharacterBuild::try_new(parts).unwrap_err(),
            TemplateError::NegativeSpeed
        );
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Speed.
    #[test]
    fn speed_zero_is_accepted() {
        let dice = ScriptedDice::new([8]);
        let mut parts = fighter_parts();
        parts.hit_points = 12;
        parts.speed_feet = 0;
        assert_eq!(derive(&build(parts)).speed_feet, 0);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Encounter Mode.
    #[test]
    fn fresh_states_from_one_creature_match() {
        let dice = ScriptedDice::new([8]);
        let mut parts = fighter_parts();
        parts.shield = Some(buckler());
        let creature = derive(&build(parts));
        let first = new_combatant_state(CombatantId(7), &creature);
        let second = new_combatant_state(CombatantId(7), &creature);
        assert_eq!(first, second);
        assert_eq!(first.hit_points, creature.hit_points);
        assert_eq!(first.temp_hit_points, 0);
        assert!(!first.shield_raised);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Getting Knocked Out".
    #[test]
    fn mutating_state_leaves_the_creature_unchanged() {
        let dice = ScriptedDice::new([8]);
        let creature = derive(&build(fighter_parts()));
        let before = creature.clone();
        let mut state = new_combatant_state(CombatantId(7), &creature);
        state.hit_points = 12;
        assert_eq!(creature.hit_points, 20);
        assert_eq!(creature, before);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Encounter Mode.
    #[test]
    fn two_ids_can_share_one_creature() {
        let dice = ScriptedDice::new([8]);
        let creature = derive(&build(fighter_parts()));
        let before = creature.clone();
        let low = new_combatant_state(CombatantId(0), &creature);
        let high = new_combatant_state(CombatantId(4_000_000_000), &creature);
        assert_eq!(low.id, CombatantId(0));
        assert_eq!(high.id, CombatantId(4_000_000_000));
        assert_eq!(creature, before);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, Encounter Mode.
    #[test]
    fn more_than_four_states_are_valid() {
        let dice = ScriptedDice::new([8]);
        let creature = derive(&build(fighter_parts()));
        let roster: Vec<_> = (0..6)
            .map(|id| new_combatant_state(CombatantId(id), &creature))
            .collect();
        assert_eq!(roster.len(), 6);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Getting Knocked Out".
    #[test]
    fn omitting_the_flag_defaults_to_false() {
        let dice = ScriptedDice::new([8]);
        assert!(!derive(&build(fighter_parts())).uses_dying_rules);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Getting Knocked Out".
    #[test]
    fn without_the_flag_0_hit_points_is_death_on_the_state() {
        let creature = derive(&build(fighter_with_hp(9, None)));
        let before = creature.clone();
        let mut state = new_combatant_state(CombatantId(1), &creature);
        apply_dice_damage(&mut state, &creature, &mut ScriptedDice::new([8]), d8(), 1);
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.vital, Vital::Dead);
        assert!(!state.dying_entered);
        assert!(state.conditions.is_empty());
        assert_eq!(creature, before);
    }

    /// Player Core, "Getting Knocked Out".
    #[test]
    fn with_the_flag_0_hit_points_enters_dying_on_the_state() {
        let creature = derive(&build(fighter_with_hp(9, Some(true))));
        let before = creature.clone();
        let mut state = new_combatant_state(CombatantId(1), &creature);
        apply_dice_damage(&mut state, &creature, &mut ScriptedDice::new([8]), d8(), 1);
        assert_eq!(state.hit_points, 0);
        assert_ne!(state.vital, Vital::Dead);
        assert_eq!(state.vital, Vital::EnterDying);
        assert!(state.dying_entered);
        assert_eq!(creature, before);
    }

    /// Player Core, "Getting Knocked Out".
    #[test]
    fn damage_above_0_does_not_kill() {
        let creature = derive(&build(fighter_with_hp(9, None)));
        let mut state = new_combatant_state(CombatantId(1), &creature);
        apply_dice_damage(&mut state, &creature, &mut ScriptedDice::new([1]), d8(), 1);
        assert_eq!(state.hit_points, 7);
        assert_eq!(state.vital, Vital::Alive);
        assert_eq!(creature.hit_points, 9);
    }

    /// A die count, face count, or reach that does not fit is an error. Player Core, Damage Rolls.
    #[test]
    fn folded_dice_and_reach_reject_values_that_do_not_fit() {
        assert_eq!(DieCount::new(-1), Err(DieCountError::OutOfRange));
        assert_eq!(DieCount::new(256), Err(DieCountError::OutOfRange));
        assert_eq!(ReachFeet::new(-1), Err(ReachError::Negative));

        let too_many_dice = Contribution {
            damage_dice: 256,
            damage_faces: 8,
            reach_feet: 5,
            ..Contribution::default()
        };
        assert_eq!(
            fold_contributions(&[too_many_dice]),
            Err(FoldError::DieCountOutOfRange)
        );

        let zero_faces = Contribution {
            damage_dice: 1,
            damage_faces: 0,
            reach_feet: 5,
            ..Contribution::default()
        };
        assert_eq!(
            fold_contributions(&[zero_faces]),
            Err(FoldError::FaceCountOutOfRange)
        );

        let negative_reach = Contribution {
            damage_dice: 1,
            damage_faces: 8,
            reach_feet: -5,
            ..Contribution::default()
        };
        assert_eq!(
            fold_contributions(&[negative_reach]),
            Err(FoldError::ReachOutOfRange)
        );

        let original = build(fighter_parts());
        let folded = fold_contributions(&contributions(&original)).expect("base fits");
        assert_eq!(derive(&original), folded);
    }

    /// Player Core, "Getting Knocked Out".
    #[test]
    fn damage_past_i32_floors_hit_points_at_0() {
        let creature = derive(&build(fighter_with_hp(9, None)));
        let mut state = new_combatant_state(CombatantId(1), &creature);
        apply_dice_damage(
            &mut state,
            &creature,
            &mut ScriptedDice::new([1]),
            d8(),
            i32::MAX,
        );
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.vital, Vital::Dead);
    }

    /// Player Core, "Getting Knocked Out".
    #[test]
    fn damage_from_i32_min_floors_hit_points_at_0() {
        let creature = derive(&build(fighter_with_hp(1, Some(true))));
        let mut state = new_combatant_state(CombatantId(1), &creature);
        state.hit_points = i32::MIN;
        apply_dice_damage(&mut state, &creature, &mut ScriptedDice::new([1]), d8(), 0);
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.vital, Vital::EnterDying);
        assert!(state.dying_entered);
    }

    /// Player Core, "Getting Knocked Out".
    #[test]
    fn a_negative_total_does_not_raise_hit_points() {
        let creature = derive(&build(fighter_with_hp(9, None)));
        assert_eq!(creature.hit_points, 9);
        let mut state = new_combatant_state(CombatantId(1), &creature);
        let mut dice = ScriptedDice::new([1]);
        apply_dice_damage(
            &mut state,
            &creature,
            &mut dice,
            Faces::new(4).expect("d4"),
            -5,
        );
        assert_eq!(state.hit_points, 9);
        assert_eq!(creature.hit_points, 9);
        assert_eq!(state.temp_hit_points, 0);
        assert_eq!(state.vital, Vital::Alive);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, "Getting Knocked Out".
    ///
    /// `1 + i32::MIN` is negative. Clamping that total to 0 leaves Hit Points unchanged.
    #[test]
    fn healing_past_i32_max_leaves_hit_points_unchanged() {
        let total = 1i32.checked_add(i32::MIN).expect("1 + i32::MIN fits");
        assert_eq!(total.max(0), 0);

        let creature = derive(&build(fighter_with_hp(1, None)));
        let mut state = new_combatant_state(CombatantId(1), &creature);
        let mut dice = ScriptedDice::new([1]);
        apply_dice_damage(&mut state, &creature, &mut dice, d8(), i32::MIN);
        assert_eq!(dice.remaining(), 0);
        assert_eq!(state.hit_points, 1);
        assert_eq!(state.vital, Vital::Alive);
    }
}
