//! Initiative, the round, the three-action turn, Strike, Aid, Reactive Strike, Delay, and Ready.
//!
//! Player Core, "Step 1: Roll Initiative". Player Core, "Step 2: Act". Player Core, Strike.
//! Player Core, Aid. Player Core, "Reactive Strike". Player Core, Delay. Player Core, "Ready".

use crate::check::{ArithmeticOverflow, DegreeOfSuccess};
use crate::combatant::{CombatantId, CombatantState, Creature, DieCount, HpError};
use crate::dice::Faces;
use crate::grid::GridError;
use crate::modifiers::ModifierError;
use crate::positioning::PositioningError;
use crate::ready::StoredReady;

/// Which team a combatant is on. Phase 1 has no other team type (DEV-26).
///
/// Player Core, "Step 1: Roll Initiative".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// A player character. Player Core, "Step 1: Roll Initiative".
    Pc,
    /// An enemy. Player Core, "Step 1: Roll Initiative".
    Enemy,
}

/// Whether the weapon used for this Strike is agile.
///
/// Player Core, "Multiple Attack Penalty".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Agility {
    /// The second and later penalties are −4 and −8. Player Core, "Multiple Attack Penalty".
    Agile,
    /// The second and later penalties are −5 and −10. Player Core, "Multiple Attack Penalty".
    NotAgile,
}

/// One combatant handed to [`begin_encounter`].
///
/// Player Core, Encounter Mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Participant {
    /// Identity in this encounter. Player Core, Encounter Mode.
    pub id: CombatantId,
    /// Team used for initiative ties. Player Core, "Step 1: Roll Initiative".
    pub side: Side,
    /// Baked statistics. Player Core, Hit Points.
    pub creature: Creature,
    /// Current Hit Points, actions, and vital. Player Core, Encounter Mode.
    pub state: CombatantState,
}

/// Numbers for one Strike. The encounter does not store squares or reach.
///
/// Player Core, Strike.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StrikeProfile {
    /// Added to the d20. Player Core, Strike.
    pub attack_bonus: i32,
    /// Agile or not, for this Strike only. Player Core, "Multiple Attack Penalty".
    pub agility: Agility,
    /// How many damage dice to roll. Player Core, "Damage Rolls".
    pub damage_dice: DieCount,
    /// Faces on each damage die. Player Core, "Damage Rolls".
    pub damage_faces: Faces,
    /// Flat damage added to the dice. Player Core, "Damage Rolls".
    pub damage_bonus: i32,
    /// The type of this Strike's damage. Player Core pg. 407, Damage Types.
    pub damage_type: crate::damage::DamageType,
    /// Which ability the attack roll uses. Player Core pg. 442, Clumsy.
    pub attack_basis: crate::conditions::AttackBasis,
    /// Whether damage adds Strength. Player Core pg. 443, Enfeebled.
    pub damage_basis: crate::conditions::DamageBasis,
}

/// Why an encounter action was rejected.
///
/// Player Core, Encounter Mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncounterError {
    /// Two participants share a [`CombatantId`]. Player Core, Encounter Mode.
    DuplicateCombatant,
    /// The roster had nobody to roll. Player Core, "Step 1: Roll Initiative".
    EmptyRoster,
    /// Perception cannot be added to a d20 of 20. Player Core, "Step 1: Roll Initiative".
    PerceptionOutOfRange,
    /// That id is not in the encounter. Player Core, Encounter Mode.
    UnknownCombatant,
    /// A turn is open, so another start was rejected. Player Core, "Step 1: Start Your Turn".
    TurnInProgress,
    /// No turn is open. Player Core, "Step 3: End Your Turn".
    TurnNotOpen,
    /// The actor is not the active combatant. Player Core, "Step 2: Act".
    NotYourTurn,
    /// The combatant cannot pay that many actions. Player Core, Activities.
    InsufficientActions,
    /// The cost was not 1, 2, or 3. Player Core, Activities.
    InvalidActionCost,
    /// The reaction was not available. Player Core, "Reactions in Encounters".
    NoReaction,
    /// The combatant is not Alive. Player Core, "Step 2: Act".
    NotConscious,
    /// The target is not Alive, so a requested Shield Block does not apply.
    ///
    /// This is the target's error. [`EncounterError::NotConscious`] stays the attacker's.
    ///
    /// Player Core, "Shield Block".
    TargetNotConscious,
    /// The creature has no shield. Player Core, "Raise a Shield".
    NoShield,
    /// The shield is not raised. Player Core, "Shield Block".
    NotRaised,
    /// The shield is broken, or a block was requested against a destroyed shield.
    ///
    /// Player Core, Broken.
    Broken,
    /// The shield is destroyed, so it cannot be raised. Player Core, "Shield Statistics".
    Destroyed,
    /// The blow amount is negative. Player Core, "Shield Block".
    NegativeAmount,
    /// The target is dead. Player Core, Strike.
    TargetDead,
    /// The target is the attacker. Player Core, Strike.
    SelfTarget,
    /// Every combatant is dead. Player Core, "Step 1: Start Your Turn".
    NoLivingCombatant,
    /// The attack count cannot increase past `u8`. Player Core, "Multiple Attack Penalty".
    AttackCountOverflow,
    /// The round cannot increase past `u32`. Player Core, "Step 2: Act".
    RoundOverflow,
    /// The attack total did not fit. Player Core, Penalties.
    AttackOverflow(ModifierError),
    /// The damage total did not fit. Player Core, "Damage Rolls".
    DamageOverflow(ModifierError),
    /// Applying Hit Points failed. Player Core, Hit Points.
    HitPoints(HpError),
    /// The aider has not prepared for an ally. Player Core, Aid.
    NotPrepared,
    /// The aider has no Aid modifier. Player Core, Aid.
    NoAidModifier,
    /// The other creature is not on the aider's side. Player Core, Aid.
    NotAnAlly,
    /// Preparation names a different ally. Player Core, Aid.
    NotThePreparedAlly,
    /// The named ally is dead.
    ///
    /// [`EncounterError::NotConscious`] stays the aider's error, including Unconscious and Dying.
    /// [`EncounterError::TargetDead`] stays the Strike target's error.
    ///
    /// Player Core, Aid. Player Core, Unconscious. Player Core, "Getting Knocked Out".
    AllyDead,
    /// The Aid modifier plus 20 does not fit in `i32`. Player Core, Aid.
    AidOverflow(ArithmeticOverflow),
    /// The roster has no creature. Player Core, Encounter Mode.
    NoCreature,
    /// Return was asked of a creature who is not delayed. Player Core, Delay.
    NotDelayed,
    /// Delay or Return was not at its trigger. Player Core, Delay.
    TriggerNotMet,
    /// The creature has Delayed and cannot use a reaction yet.
    ///
    /// Checked after [`EncounterError::NotConscious`] and before any action or die.
    ///
    /// Player Core, Delay.
    Delayed,
    /// A begun Stride or Step was rejected by the grid. Player Core, "Grid Movement".
    Grid(GridError),
    /// Stand or Drop Prone was rejected after the creature was conscious.
    ///
    /// Player Core, "Stand". Player Core, "Drop Prone".
    Positioning(PositioningError),
    /// The creature has no save training. Player Core pg. 404, Saving Throws.
    NoSavingThrows,
    /// The save modifier does not fit a check modifier. Player Core pg. 404, Saving Throws.
    SaveOverflow(ArithmeticOverflow),
    /// The creature has no skill training. Player Core pg. 405, Skill Checks.
    NoSkills,
    /// The skill modifier does not fit a check modifier. Player Core pg. 405, Skill Checks.
    SkillOverflow(ArithmeticOverflow),
    /// The target is undetected, so it cannot be chosen. Nothing is spent and no die is drawn.
    ///
    /// Player Core pg. 447, Undetected. Player Core pg. 418, Strike.
    CannotTarget,
    /// Persistent damage was gained with an amount below 1. Nothing is stored and no die is drawn.
    ///
    /// Player Core pg. 445, Persistent Damage.
    NonPositivePersistent,
    /// The creature has no spellcasting entry. Player Core pg. 403, Spell Attack Rolls.
    NoSpellcasting,
    /// The target is farther than the spell's range. Nothing is spent and no die is drawn.
    ///
    /// Player Core pg. 363, Telekinetic Projectile. Player Core pg. 426, Range and Reach.
    OutOfSpellRange,
    /// The cone length is not the 15-foot template. No squares are returned.
    ///
    /// Player Core pg. 428, Cone.
    UnsupportedCone,
    /// The same creature was listed twice. Nothing is spent and no die is drawn.
    ///
    /// Player Core pg. 428, Areas.
    DuplicateAreaTarget,
    /// A condition value or duration was rejected. Nothing is stored.
    ///
    /// Player Core pg. 442, Condition Values.
    Condition(crate::conditions::ConditionError),
}

/// Where the round sits between turns.
///
/// A blow's delayed target is [`TargetDelay`], not a variant here.
/// A pending wrap cannot be stored as a delayed target.
///
/// Player Core, Delay.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TurnState {
    /// The round is not waiting to wrap.
    Continuing,
    /// The last turn ended while someone is Delayed. The round has not advanced.
    PendingWrap,
}

/// Whether a blow was planned while the target's slot was Delayed.
///
/// Split from [`TurnState`] so the round's wrap and the blow's target
/// cannot occupy one value.
///
/// Player Core, Delay. Player Core, "Getting Knocked Out".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TargetDelay {
    /// The target's slot was Delayed when the blow was planned.
    Delayed,
    /// The target's slot was not Delayed when the blow was planned.
    NotDelayed,
}

/// Where a creature sits in the round.
///
/// There is no delayed flag. The slot is the only record.
///
/// Player Core, Delay. Player Core, "Step 1: Start Your Turn".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnSlot {
    /// An ordinary turn. `start_turn` sets this. Player Core, "Step 1: Start Your Turn".
    Normal,
    /// Out of the order until they return or the slot comes up again. Player Core, Delay.
    Delayed,
    /// The open turn a return just produced. Delay on it is [`EncounterError::TriggerNotMet`].
    ///
    /// Player Core, Delay.
    Returned,
    /// A later creature in the same return. The next start grants actions and skips the start step.
    ///
    /// Player Core, Delay.
    SkipStartHook,
}

/// One resolved Strike.
///
/// Player Core, Strike.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrikeOutcome {
    /// The d20. Player Core, Strike.
    pub natural: u8,
    /// Natural roll plus attack bonus, multiple attack penalty, and caller modifiers.
    ///
    /// Player Core, Strike.
    pub total: i32,
    /// Degree after a natural 20 or natural 1. Player Core, "Step 4: Degree of Success".
    pub degree: DegreeOfSuccess,
    /// Multiple attack penalty used on this Strike. Player Core, "Multiple Attack Penalty".
    pub map: i32,
    /// Damage dealt, or 0 on a miss. Player Core, "Damage Rolls".
    pub damage: i32,
}

/// A Strike the flat check did not affect. No attack d20 and no damage die were rolled.
///
/// Player Core pg. 442, Concealed. Player Core pg. 444, Hidden.
/// Player Core pg. 400, "Checks (Flat Checks)".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LostStrike {
    /// The flat-check face. Player Core pg. 400, "Checks (Flat Checks)".
    pub natural: u8,
    /// Concealed is DC 5. Hidden, including hidden and concealed, is DC 11.
    ///
    /// Player Core pg. 442, Concealed. Player Core pg. 444, Hidden.
    pub dc: i32,
    /// The flat check's degree, including a critical failure on a natural 1.
    ///
    /// Player Core pg. 400, "Checks (Flat Checks)".
    pub degree: DegreeOfSuccess,
}

/// A Strike that rolled against Armor Class, or a flat check that lost the Strike.
///
/// Player Core pg. 418, Strike. Player Core pg. 442, Concealed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StrikeDelivery {
    /// The attack d20 was rolled. Player Core pg. 418, Strike.
    Rolled(StrikeOutcome),
    /// The flat check did not affect the target. Player Core pg. 442, Concealed.
    Lost(LostStrike),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Member {
    participant: Participant,
    attack_count: u8,
    initiative: i32,
    slot: TurnSlot,
    ready: Option<StoredReady>,
}

/// One fight: initiative order, the round, the open turn, and Strike.
///
/// Player Core, Encounter Mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Encounter {
    members: Vec<Member>,
    order: Vec<CombatantId>,
    cursor: usize,
    round: u32,
    open: Option<CombatantId>,
    /// The creature whose turn just ended. Delay clears it. Return requires it.
    ///
    /// Player Core, Delay. Player Core, "Step 3: End Your Turn".
    last_ended: Option<CombatantId>,
    /// Whether the next start wraps the round. Delay does not advance it.
    ///
    /// Player Core, Delay.
    turn_state: TurnState,
}

mod delay_ready;
mod reactions;
mod skills;
mod spell;
mod strike;
mod turns;
mod vitals;

pub use spell::{BreatheFireOutcome, SpellAttackHit, SpellAttackOutcome};
pub use turns::begin_encounter;

#[cfg(test)]
mod tests;
