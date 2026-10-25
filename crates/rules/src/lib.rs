//! Pathfinder Remaster dice, checks, modifiers, grid movement, positioning, detection, creature templates, encounters, dying, shields, reactive strikes, Aid, Delay, Ready, and saving throws.
//!
//! Callers inject [`Dice`]. [`derive`] does not roll.
//! `ScriptedDice` is compiled for tests and the `test-support` feature.

//!
//! An external crate starts and ends one turn.
//!
//! ```
//! use rules::{
//!     begin_encounter, derive, new_combatant_state, BuildParts, CharacterBuild, CombatantId,
//!     DieCount, Faces, MeleeStrike, Participant, ReachFeet, SeededDice, Side,
//! };
//!
//! let build = CharacterBuild::try_new(BuildParts {
//!     hit_points: 30,
//!     armor_class: 11,
//!     perception: 0,
//!     speed_feet: 25,
//!     strike: MeleeStrike {
//!         attack_bonus: 0,
//!         damage_dice: DieCount::new(1).expect("dice"),
//!         damage_faces: Faces::new(8).expect("faces"),
//!         damage_bonus: 0,
//!         reach_feet: ReachFeet::new(5).expect("reach"),
//!         agile: false,
//!     },
//!     shield: None,
//!     stealth_dc: None,
//!     aid_modifier: None,
//!     aid_rank: None,
//!     uses_dying_rules: None,
//!     saves: None,
//! })
//! .expect("build");
//! let creature = derive(&build);
//! let id = CombatantId(1);
//! let state = new_combatant_state(id, &creature);
//! let mut dice = SeededDice::new(1);
//! let mut encounter = begin_encounter(
//!     vec![Participant {
//!         id,
//!         side: Side::Pc,
//!         creature,
//!         state,
//!     }],
//!     &mut dice,
//! )
//! .expect("encounter");
//! assert_eq!(encounter.round(), 1);
//! assert_eq!(encounter.order(), &[id]);
//! assert_eq!(encounter.start_turn().expect("start"), id);
//! encounter.end_turn().expect("end");
//! ```
//!
//! The public names are this set.
//!
//! ```
//! #![allow(unused_imports)]
//! use rules::{
//!     ActionCost,
//!     ActionIdentity,
//!     ActionTrait,
//!     Agility,
//!     AidCircumstance,
//!     AidOutcome,
//!     AidPreparation,
//!     AidRank,
//!     ArithmeticOverflow,
//!     BonusType,
//!     BuildParts,
//!     CharacterBuild,
//!     CombatantId,
//!     CombatantState,
//!     Concealment,
//!     ConditionId,
//!     ConditionSlot,
//!     CoverBenefit,
//!     Creature,
//!     Detection,
//!     DetectionError,
//!     Detections,
//!     DiagonalCounter,
//!     Dice,
//!     DieCount,
//!     DieCountError,
//!     DegreeOfSuccess,
//!     Disruption,
//!     DyingValue,
//!     DyingValueError,
//!     Encounter,
//!     EncounterError,
//!     FaceCountError,
//!     Faces,
//!     FactionId,
//!     GridError,
//!     HpError,
//!     InitiativeMove,
//!     LEVEL_RANGE,
//!     Level,
//!     LevelError,
//!     LostStrike,
//!     MapBounds,
//!     MeleeStrike,
//!     Mobility,
//!     Modifier,
//!     ModifierError,
//!     MoveAction,
//!     MoveOutcome,
//!     Occupant,
//!     OccupantPresence,
//!     ObservedAction,
//!     Participant,
//!     PenaltyType,
//!     PositioningError,
//!     Posture,
//!     PostureChange,
//!     Proficiency,
//!     ReachError,
//!     ReachFeet,
//!     ReactionField,
//!     ReactionPlace,
//!     ReactiveError,
//!     ReactiveOutcome,
//!     ReactiveStrike,
//!     ReactiveStrikeInput,
//!     ReadyActivity,
//!     ReadyEncounter,
//!     ReadyError,
//!     ReadyStrike,
//!     ReadySuccess,
//!     ReadyTrigger,
//!     Relation,
//!     ResolvedReaction,
//!     Save,
//!     SaveRoll,
//!     SavingThrows,
//!     SeededDice,
//!     ShieldBlock,
//!     ShieldTemplate,
//!     Side,
//!     Size,
//!     SourceId,
//!     Square,
//!     SquareEvent,
//!     Statistic,
//!     StoredReady,
//!     StrikeDelivery,
//!     StrikeOutcome,
//!     StrikeProfile,
//!     StrikeRange,
//!     TemplateError,
//!     Terrain,
//!     TurnSlot,
//!     Vital,
//!     begin_encounter,
//!     derive,
//!     new_combatant_state,
//! };
//! ```
//!
//! The public method set is unchanged.
//!
//! ```
//! let _ = rules::Encounter::round;
//! let _ = rules::Encounter::order;
//! let _ = rules::Encounter::turn_open;
//! let _ = rules::Encounter::state;
//! let _ = rules::Encounter::creature;
//! let _ = rules::Encounter::start_turn;
//! let _ = rules::Encounter::end_turn;
//! let _ = rules::Encounter::spend_actions;
//! let _ = rules::Encounter::spend_free_action;
//! let _ = rules::Encounter::spend_reaction;
//! let _ = rules::Encounter::prepare_to_aid;
//! let _: fn(
//!     &mut rules::Encounter,
//!     rules::CombatantId,
//!     rules::CombatantId,
//!     &mut rules::SeededDice,
//! ) -> Result<rules::AidOutcome, rules::EncounterError> = rules::Encounter::aid;
//! let _ = rules::Encounter::attack_trait_penalty;
//! let _ = rules::Encounter::raise_shield;
//! let _ = rules::Encounter::shield_block;
//! let _: fn(
//!     &mut rules::Encounter,
//!     rules::CombatantId,
//!     rules::CombatantId,
//!     &rules::StrikeProfile,
//!     &[rules::Modifier],
//!     rules::ShieldBlock,
//!     &rules::Detections,
//!     &mut rules::SeededDice,
//! ) -> Result<rules::StrikeDelivery, rules::EncounterError> = rules::Encounter::strike;
//! let _: fn(
//!     &mut rules::Encounter,
//!     rules::CombatantId,
//!     rules::CombatantId,
//!     rules::ActionCost,
//!     rules::ActionIdentity,
//!     Option<rules::MoveAction>,
//!     rules::ReactiveStrike,
//!     rules::Square,
//!     rules::Square,
//!     rules::Size,
//!     rules::Size,
//!     &rules::StrikeProfile,
//!     &[rules::Modifier],
//!     &rules::Detections,
//!     &mut rules::SeededDice,
//! ) -> Result<rules::ReactiveOutcome, rules::ReactiveError> = rules::Encounter::reactive_strike;
//! let _ = rules::Encounter::initiative_total;
//! let _ = rules::Encounter::turn_slot;
//! let _ = rules::Encounter::next_start;
//! let _ = rules::Encounter::stored_ready;
//! let _ = rules::Encounter::frozen_attack_count;
//! let _ = rules::Encounter::attack_count;
//! let _: fn(
//!     &mut rules::Encounter,
//!     rules::CombatantId,
//!     rules::Save,
//!     i32,
//!     i32,
//!     &[rules::Modifier],
//!     &mut rules::SeededDice,
//! ) -> Result<rules::SaveRoll, rules::EncounterError> = rules::Encounter::basic_save;
//! let _ = rules::Encounter::delay;
//! let _ = rules::Encounter::return_from_delay;
//! let _ = rules::Encounter::ready;
//! let _ = rules::Encounter::readied_attack_penalty;
//! let _: fn(
//!     &mut rules::Encounter,
//!     rules::CombatantId,
//!     rules::ObservedAction,
//!     rules::ActionCost,
//!     rules::Square,
//!     rules::Square,
//!     &[rules::Square],
//!     &rules::ReactionField<'_>,
//!     &[rules::ReactiveStrikeInput<'_>],
//!     &rules::Detections,
//!     &mut rules::SeededDice,
//! ) -> Vec<rules::ResolvedReaction> = rules::Encounter::resolve_reactions;
//! let _ = rules::Encounter::stride;
//! let _ = rules::Encounter::step;
//! let _ = rules::Encounter::drop_prone;
//! let _ = rules::Encounter::stand;
//! ```
//!
//! Hit points and a condition id stay readable.
//!
//! ```
//! #![allow(dead_code)]
//! use rules::{CombatantState, ConditionId};
//!
//! fn read(state: &CombatantState) -> i32 {
//!     let _frightened = ConditionId::FRIGHTENED;
//!     state.hit_points
//! }
//! ```
//!
//! The encounter Stride method stays callable.
//!
//! ```
//! let _stride = rules::Encounter::stride;
//! ```
//!
//! A module path is private.
//!
//! ```compile_fail,E0603
//! use rules::check::degree_of_success;
//! ```
//!
//! Encounter is not reached through a module path.
//!
//! ```compile_fail,E0603
//! use rules::encounter::Encounter;
//! ```
//!
//! ```
//! #![allow(unused_imports)]
//! use rules::Encounter;
//! ```
//!
//! `Area` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::Area;
//! ```
//!
//! `AttackBlowError` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::AttackBlowError;
//! ```
//!
//! `AttackKind` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::AttackKind;
//! ```
//!
//! `AttackTotalTerm` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::AttackTotalTerm;
//! ```
//!
//! `CheckModifier` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::CheckModifier;
//! ```
//!
//! `CheckRoll` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::CheckRoll;
//! ```
//!
//! `CombatantPos` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::CombatantPos;
//! ```
//!
//! `Contribution` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::Contribution;
//! ```
//!
//! `Cover` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::Cover;
//! ```
//!
//! `DcModifier` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::DcModifier;
//! ```
//!
//! `DetectionCost` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::DetectionCost;
//! ```
//!
//! `EncounterOutcome` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::EncounterOutcome;
//! ```
//!
//! `EndReason` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::EndReason;
//! ```
//!
//! `FlatRoll` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::FlatRoll;
//! ```
//!
//! `FoldError` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::FoldError;
//! ```
//!
//! `HideObserver` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::HideObserver;
//! ```
//!
//! `HideResult` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::HideResult;
//! ```
//!
//! `MapInput` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::MapInput;
//! ```
//!
//! `ModifierTotal` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::ModifierTotal;
//! ```
//!
//! `MoveInput` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::MoveInput;
//! ```
//!
//! `NearbyFeature` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::NearbyFeature;
//! ```
//!
//! `OffGuard` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::OffGuard;
//! ```
//!
//! `OpenRoll` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::OpenRoll;
//! ```
//!
//! `RangeBand` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::RangeBand;
//! ```
//!
//! `Readiness` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::Readiness;
//! ```
//!
//! `RosterEntry` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::RosterEntry;
//! ```
//!
//! `SaveError` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::SaveError;
//! ```
//!
//! `SeekResult` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::SeekResult;
//! ```
//!
//! `ShieldError` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::ShieldError;
//! ```
//!
//! `SizeData` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::SizeData;
//! ```
//!
//! `Space` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::Space;
//! ```
//!
//! `StrikeNotice` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::StrikeNotice;
//! ```
//!
//! `TakeCoverResult` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::TakeCoverResult;
//! ```
//!
//! `TargetingResult` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::TargetingResult;
//! ```
//!
//! `ac_modifiers` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::ac_modifiers;
//! ```
//!
//! `adjacent` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::adjacent;
//! ```
//!
//! `aid_check` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::aid_check;
//! ```
//!
//! `aid_circumstance_bonus` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::aid_circumstance_bonus;
//! ```
//!
//! `apply_attack_blow` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::apply_attack_blow;
//! ```
//!
//! `apply_dice_damage` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::apply_dice_damage;
//! ```
//!
//! `apply_healing` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::apply_healing;
//! ```
//!
//! `apply_resolved_blow` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::apply_resolved_blow;
//! ```
//!
//! `armor_class` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::armor_class;
//! ```
//!
//! `attack_check_total` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::attack_check_total;
//! ```
//!
//! `attack_modifiers` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::attack_modifiers;
//! ```
//!
//! `attack_penalty` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::attack_penalty;
//! ```
//!
//! `base_contribution` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::base_contribution;
//! ```
//!
//! `basic_save_amount` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::basic_save_amount;
//! ```
//!
//! `contributions` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::contributions;
//! ```
//!
//! `damage_dealt` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::damage_dealt;
//! ```
//!
//! `dc_from_modifier` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::dc_from_modifier;
//! ```
//!
//! `dc_from_total` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::dc_from_total;
//! ```
//!
//! `degree_of_success` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::degree_of_success;
//! ```
//!
//! `dismiss` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::dismiss;
//! ```
//!
//! `dismiss_cost` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::dismiss_cost;
//! ```
//!
//! `distance` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::distance;
//! ```
//!
//! `drop_prone` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::drop_prone;
//! ```
//!
//! `encounter_outcome` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::encounter_outcome;
//! ```
//!
//! `end_benefit` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::end_benefit;
//! ```
//!
//! `fallback_unlisted_critical` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::fallback_unlisted_critical;
//! ```
//!
//! `flanks` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::flanks;
//! ```
//!
//! `flat_check` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::flat_check;
//! ```
//!
//! `flat_checks_for_same_thing` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::flat_checks_for_same_thing;
//! ```
//!
//! `fold_contributions` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::fold_contributions;
//! ```
//!
//! `grant_temporary_hit_points` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::grant_temporary_hit_points;
//! ```
//!
//! `hide` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::hide;
//! ```
//!
//! `in_reach` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::in_reach;
//! ```
//!
//! `leap_while_prone` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::leap_while_prone;
//! ```
//!
//! `lower_raised_shield` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::lower_raised_shield;
//! ```
//!
//! `move_trait_allowed` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::move_trait_allowed;
//! ```
//!
//! `multiple_attack_penalty` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::multiple_attack_penalty;
//! ```
//!
//! `note_action` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::note_action;
//! ```
//!
//! `note_strike` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::note_strike;
//! ```
//!
//! `off_guard` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::off_guard;
//! ```
//!
//! `off_guard_against` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::off_guard_against;
//! ```
//!
//! `perception_modifiers` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::perception_modifiers;
//! ```
//!
//! `proficiency_bonus` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::proficiency_bonus;
//! ```
//!
//! `prone_move` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::prone_move;
//! ```
//!
//! `raise_shield` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::raise_shield;
//! ```
//!
//! `range_band` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::range_band;
//! ```
//!
//! `ready_cost` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::ready_cost;
//! ```
//!
//! `reflex_modifiers` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::reflex_modifiers;
//! ```
//!
//! `refresh` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::refresh;
//! ```
//!
//! `resolve_check` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::resolve_check;
//! ```
//!
//! `resolve_targeting` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::resolve_targeting;
//! ```
//!
//! `save_dc` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::save_dc;
//! ```
//!
//! `save_modifier` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::save_modifier;
//! ```
//!
//! `seek` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::seek;
//! ```
//!
//! `shield_ac_modifier` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::shield_ac_modifier;
//! ```
//!
//! `size_data` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::size_data;
//! ```
//!
//! `stand` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::stand;
//! ```
//!
//! `start_of_turn` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::start_of_turn;
//! ```
//!
//! `stealth_dc_from_modifier` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::stealth_dc_from_modifier;
//! ```
//!
//! `step` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::step;
//! ```
//!
//! `stride` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::stride;
//! ```
//!
//! `take_cover` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::take_cover;
//! ```
//!
//! `take_initiative_move` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::take_initiative_move;
//! ```
//!
//! `total` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::total;
//! ```
//!
//! `validate_speed` is not a crate-root name.
//!
//! ```compile_fail,E0432
//! use rules::validate_speed;
//! ```

#![forbid(unsafe_code)]

mod aid;
mod check;
mod combatant;
mod detection;
mod dice;
mod dying;
mod encounter;
mod grid;
mod modifiers;
mod positioning;
mod reactive;
mod ready;
mod save;
mod shield;

pub use aid::{AidCircumstance, AidOutcome, AidPreparation, AidRank};
pub use check::{ArithmeticOverflow, DegreeOfSuccess};
pub use combatant::{
    BuildParts, CharacterBuild, CombatantId, CombatantState, ConditionSlot, Creature, DieCount,
    DieCountError, DyingValue, DyingValueError, HpError, MeleeStrike, ReachError, ReachFeet,
    ShieldTemplate, TemplateError, Vital, derive, new_combatant_state,
};
pub use detection::{Concealment, Detection, DetectionError, Detections, Relation};
pub use dice::{Dice, FaceCountError, Faces, SeededDice};
pub use dying::InitiativeMove;
pub use encounter::{
    Agility, Encounter, EncounterError, LostStrike, Participant, Side, StrikeDelivery,
    StrikeOutcome, StrikeProfile, TurnSlot, begin_encounter,
};
pub use modifiers::{
    BonusType, ConditionId, Modifier, ModifierError, PenaltyType, SourceId, Statistic,
};

#[cfg(any(test, feature = "test-support"))]
pub use dice::ScriptedDice;
pub use grid::{
    DiagonalCounter, FactionId, GridError, MapBounds, MoveOutcome, Occupant, Size, Square,
    SquareEvent, Terrain,
};
pub use positioning::{
    ActionCost, ActionTrait, CoverBenefit, Mobility, MoveAction, OccupantPresence,
    PositioningError, Posture, PostureChange,
};
pub use reactive::{ActionIdentity, Disruption, ReactiveError, ReactiveOutcome, ReactiveStrike};
pub use ready::{
    ObservedAction, ReactionField, ReactionPlace, ReactiveStrikeInput, ReadyActivity,
    ReadyEncounter, ReadyError, ReadyStrike, ReadySuccess, ReadyTrigger, ResolvedReaction,
    StoredReady, StrikeRange,
};
pub use save::{LEVEL_RANGE, Level, LevelError, Proficiency, Save, SaveRoll, SavingThrows};
pub use shield::ShieldBlock;

#[cfg(test)]
mod facade_check;
