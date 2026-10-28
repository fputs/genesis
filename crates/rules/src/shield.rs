//! Raise a Shield, the raised circumstance bonus, and Shield Block.
//!
//! Creature damage stays in [`apply_resolved_blow`]. This module subtracts hardness once
//! and passes the remainder through.
//!
//! Player Core, "Raise a Shield". Player Core, "Shield Block".

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "callers are this crate's tests; the facade keeps the procedure and drops the root export"
    )
)]

use crate::check::DegreeOfSuccess;
use crate::combatant::{CombatantState, Creature, HpError, Vital};
use crate::dying::apply_resolved_blow;
use crate::modifiers::{Modifier, raised_shield_modifier};

/// Whether the defender uses Shield Block on this attack blow.
///
/// Player Core, "Shield Block".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShieldBlock {
    /// Subtract hardness, then resolve the remainder. Player Core, "Shield Block".
    Requested,
    /// Pass the full amount through and leave the shield. Player Core, "Shield Block".
    Declined,
}

/// Why Raise a Shield was rejected. The state is unchanged.
///
/// Player Core, "Raise a Shield". Player Core, Broken.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShieldError {
    /// The creature has no shield. Player Core, "Raise a Shield".
    NoShield,
    /// Shield Hit Points are above 0 and at or below the broken threshold.
    ///
    /// Player Core, Broken.
    Broken,
    /// Shield Hit Points are 0. Player Core, "Shield Statistics".
    Destroyed,
    /// The combatant is not Alive. Player Core, "Raise a Shield".
    NotConscious,
}

/// Why an attack blow was not applied. Nothing is written.
///
/// Player Core, "Shield Block".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AttackBlowError {
    /// The resolved blow rejected the amount. Player Core, Hit Points.
    HitPoints(HpError),
    /// The combatant is not Alive. Player Core, "Shield Block".
    NotConscious,
    /// The creature has no shield. Player Core, "Shield Block".
    NoShield,
    /// The shield is not raised. Player Core, "Shield Block".
    NotRaised,
    /// The shield is broken or destroyed. Player Core, Broken.
    Broken,
    /// The reaction is not ready. Player Core, "Shield Block".
    NoReaction,
    /// The amount is negative. Player Core, "Shield Block".
    NegativeAmount,
}

/// What a planned blow will write. Planning itself writes nothing.
pub(crate) enum PlannedBlow {
    /// Pass this amount to [`apply_resolved_blow`]. Leave the shield and the reaction.
    Unblocked(i32),
    /// Amount 0 on a legal request. Write nothing.
    NoWrite,
    /// Pass this remainder through, then remove it from the shield and spend the reaction.
    Blocked(i32),
}

/// Raises the shield. Actions and the reaction stay as they are.
///
/// Checks run in order: no shield, not Alive, 0 Hit Points, then Hit Points at or below
/// the broken threshold. Raising an already raised shield is `Ok` and does not add a
/// second bonus.
///
/// Player Core, "Raise a Shield". Player Core, Broken.
pub(crate) fn raise_shield(
    state: &mut CombatantState,
    creature: &Creature,
) -> Result<(), ShieldError> {
    validate_raise(state, creature)?;
    state.shield_raised = true;
    Ok(())
}

/// Whether Raise a Shield can set the flag. Writes nothing.
///
/// Player Core, "Raise a Shield". Player Core, Broken.
pub(crate) fn validate_raise(
    state: &CombatantState,
    creature: &Creature,
) -> Result<(), ShieldError> {
    let Some(shield) = creature.shield.as_ref() else {
        return Err(ShieldError::NoShield);
    };
    let Some(hit_points) = state.shield_hit_points else {
        return Err(ShieldError::NoShield);
    };
    if state.vital != Vital::Alive {
        return Err(ShieldError::NotConscious);
    }
    if hit_points == 0 {
        return Err(ShieldError::Destroyed);
    }
    if hit_points <= shield.broken_threshold {
        return Err(ShieldError::Broken);
    }
    Ok(())
}

/// Lowers a raised shield. Shield Hit Points, actions, and the reaction stay as they are.
///
/// Player Core, "Raise a Shield". Player Core, "Start Your Turn".
pub(crate) fn lower_raised_shield(state: &mut CombatantState) {
    state.shield_raised = false;
}

/// The raised shield's circumstance bonus, when that bonus applies.
///
/// `Some` is the shield's listed bonus while it is raised and intact and the combatant
/// is not Dead. `None` is every other case, including a bonus that is not positive.
/// This is not a [`Result`] and it is not a [`ShieldError`].
///
/// Player Core, "Raise a Shield". Player Core, Bonuses. Player Core, Broken.
pub(crate) fn shield_ac_modifier(state: &CombatantState, creature: &Creature) -> Option<Modifier> {
    if state.vital == Vital::Dead || !state.shield_raised {
        return None;
    }
    let shield = creature.shield.as_ref()?;
    let hit_points = state.shield_hit_points?;
    if hit_points <= 0 || hit_points <= shield.broken_threshold {
        return None;
    }
    raised_shield_modifier(shield.shield_ac_bonus).ok()
}

/// Applies one already-calculated attack blow, then Shield Block when it was requested.
///
/// The amount is [`crate::combatant::damage_dealt`]'s result. This function does not roll
/// and does not double. On a legal block the remainder is `max(0, amount - hardness)`,
/// computed in `i64`, and that remainder is the only input to [`apply_resolved_blow`].
/// The shield then loses the same remainder, floored at 0, and the reaction is cleared.
///
/// Player Core, "Shield Block". Player Core, "Item Damage".
pub(crate) fn apply_attack_blow(
    state: &mut CombatantState,
    creature: &Creature,
    amount: i32,
    degree: DegreeOfSuccess,
    block: ShieldBlock,
) -> Result<(), AttackBlowError> {
    match plan_attack_blow(state, creature, amount, degree, block)? {
        PlannedBlow::NoWrite => Ok(()),
        PlannedBlow::Unblocked(amount) => {
            apply_resolved_blow(state, creature, amount, degree).map_err(AttackBlowError::HitPoints)
        }
        PlannedBlow::Blocked(remainder) => {
            apply_resolved_blow(state, creature, remainder, degree)
                .map_err(AttackBlowError::HitPoints)?;
            reduce_shield(state, remainder);
            state.reaction_ready = false;
            Ok(())
        }
    }
}

/// Plans an attack blow without writing. [`ShieldBlock::Declined`] skips the request checks.
///
/// Player Core, "Shield Block".
pub(crate) fn plan_attack_blow(
    state: &CombatantState,
    creature: &Creature,
    amount: i32,
    degree: DegreeOfSuccess,
    block: ShieldBlock,
) -> Result<PlannedBlow, AttackBlowError> {
    if matches!(
        degree,
        DegreeOfSuccess::Failure | DegreeOfSuccess::CriticalFailure
    ) {
        return Err(AttackBlowError::HitPoints(HpError::NotAHit));
    }
    if amount < 0 {
        return Err(AttackBlowError::NegativeAmount);
    }
    let ShieldBlock::Requested = block else {
        return Ok(PlannedBlow::Unblocked(amount));
    };
    if state.vital != Vital::Alive {
        return Err(AttackBlowError::NotConscious);
    }
    let Some(shield) = creature.shield.as_ref() else {
        return Err(AttackBlowError::NoShield);
    };
    let Some(hit_points) = state.shield_hit_points else {
        return Err(AttackBlowError::NoShield);
    };
    if !state.shield_raised {
        return Err(AttackBlowError::NotRaised);
    }
    if hit_points == 0 || hit_points <= shield.broken_threshold {
        return Err(AttackBlowError::Broken);
    }
    if !state.reaction_ready {
        return Err(AttackBlowError::NoReaction);
    }
    if amount == 0 {
        return Ok(PlannedBlow::NoWrite);
    }
    Ok(PlannedBlow::Blocked(remainder_after_hardness(
        amount,
        shield.hardness,
    )?))
}

/// Removes `loss` shield Hit Points, floored at 0.
///
/// Player Core, "Item Damage".
pub(crate) fn reduce_shield(state: &mut CombatantState, loss: i32) {
    if loss <= 0 {
        return;
    }
    let Some(hit_points) = state.shield_hit_points else {
        return;
    };
    let next = i64::from(hit_points) - i64::from(loss);
    state.shield_hit_points = Some(match i32::try_from(next) {
        Ok(value) if value > 0 => value,
        _ => 0,
    });
}

fn remainder_after_hardness(amount: i32, hardness: i32) -> Result<i32, AttackBlowError> {
    let rest = i64::from(amount) - i64::from(hardness);
    if rest <= 0 {
        Ok(0)
    } else {
        i32::try_from(rest).map_err(|_| AttackBlowError::HitPoints(HpError::Overflow))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::DegreeOfSuccess;
    use crate::combatant::{
        BuildParts, CharacterBuild, CombatantId, DieCount, DyingValue, MeleeStrike, ReachFeet,
        ShieldTemplate, Vital, derive, new_combatant_state,
    };
    use crate::detection::Detections;
    use crate::dice::{Faces, ScriptedDice};
    use crate::dying::start_of_turn;
    use crate::encounter::{
        Agility, Encounter, EncounterError, Participant, Side, StrikeDelivery, StrikeOutcome,
        StrikeProfile,
    };
    use crate::modifiers::{BonusType, Cover, ModifierTotal, Statistic};
    use crate::positioning::{AttackKind, CoverBenefit, OffGuard, armor_class};

    fn rolled(delivery: StrikeDelivery) -> StrikeOutcome {
        match delivery {
            StrikeDelivery::Rolled(outcome) => outcome,
            StrikeDelivery::Lost(lost) => panic!("expected a rolled strike, lost {lost:?}"),
        }
    }

    fn id(value: u32) -> CombatantId {
        CombatantId(value)
    }

    fn faces(count: u8) -> Faces {
        Faces::new(count).expect("faces")
    }

    fn steel() -> ShieldTemplate {
        ShieldTemplate {
            hardness: 5,
            hit_points: 20,
            broken_threshold: 10,
            shield_ac_bonus: 2,
        }
    }

    /// Book buckler. The authored build buckler is a different pair of numbers.
    fn buckler() -> ShieldTemplate {
        ShieldTemplate {
            hardness: 3,
            hit_points: 6,
            broken_threshold: 3,
            shield_ac_bonus: 1,
        }
    }

    fn creature(
        hit_points: i32,
        armor_class: i32,
        perception: i32,
        dying: bool,
        shield: Option<ShieldTemplate>,
    ) -> Creature {
        let parts = BuildParts {
            hit_points,
            armor_class,
            perception,
            speed_feet: 25,
            strike: MeleeStrike {
                attack_bonus: 0,
                damage_dice: DieCount::new(1).expect("dice"),
                damage_faces: faces(8),
                damage_bonus: 0,
                reach_feet: ReachFeet::new(5).expect("reach"),
                agile: false,
                damage_type: crate::damage::DamageType::Slashing,
            },
            shield,
            stealth_dc: None,
            aid_modifier: None,
            aid_rank: None,
            uses_dying_rules: Some(dying),
            saves: None,
            skills: None,
            defenses: crate::damage::DamageDefenses::empty(),
            spellcasting: None,
        };
        derive(&CharacterBuild::try_new(parts).expect("build"))
    }

    fn state_for(creature: &Creature) -> CombatantState {
        let mut state = new_combatant_state(id(1), creature);
        state.actions_remaining = 3;
        state.reaction_ready = true;
        state
    }

    fn raised(creature: &Creature, shield_hit_points: i32) -> CombatantState {
        let mut state = state_for(creature);
        state.shield_hit_points = Some(shield_hit_points);
        state.shield_raised = true;
        state
    }

    fn participant(value: u32, creature: Creature) -> Participant {
        let combatant = id(value);
        let state = new_combatant_state(combatant, &creature);
        Participant {
            id: combatant,
            side: Side::Pc,
            creature,
            state,
        }
    }

    fn begin(participants: Vec<Participant>, rolls: &[u8]) -> (Encounter, ScriptedDice) {
        let mut dice = ScriptedDice::new(rolls.iter().copied());
        let encounter =
            crate::encounter::begin_encounter(participants, &mut dice).expect("encounter");
        (encounter, dice)
    }

    fn profile(dice: u8, bonus: i32) -> StrikeProfile {
        StrikeProfile {
            attack_bonus: 0,
            agility: Agility::NotAgile,
            damage_dice: DieCount::new(1).expect("dice"),
            damage_faces: faces(dice),
            damage_bonus: bonus,
            damage_type: crate::damage::DamageType::Slashing,
            attack_basis: crate::conditions::AttackBasis::StrengthMelee,
            damage_basis: crate::conditions::DamageBasis::Strength,
        }
    }

    fn ac_total(
        state: &CombatantState,
        creature: &Creature,
        benefit: Option<CoverBenefit>,
        cover: Option<Cover>,
        off_guard: OffGuard,
        attack: AttackKind,
    ) -> ModifierTotal {
        let extra: Vec<Modifier> = shield_ac_modifier(state, creature).into_iter().collect();
        armor_class(benefit, cover, off_guard, attack, &extra).expect("armor class")
    }

    fn plain_total(state: &CombatantState, creature: &Creature) -> ModifierTotal {
        ac_total(state, creature, None, None, OffGuard::No, AttackKind::Melee)
    }

    fn circumstance_bonuses(total: &ModifierTotal) -> Vec<i32> {
        total
            .applied()
            .iter()
            .filter(|modifier| {
                modifier.statistic() == Statistic::ArmorClass
                    && modifier.bonus_type() == Some(BonusType::Circumstance)
            })
            .map(|modifier| modifier.value())
            .collect()
    }

    fn circumstance_penalty(total: &ModifierTotal) -> i32 {
        let penalties: Vec<i32> = total
            .applied()
            .iter()
            .filter(|modifier| {
                modifier.statistic() == Statistic::ArmorClass
                    && modifier.penalty_type() == Some(crate::modifiers::PenaltyType::Circumstance)
            })
            .map(|modifier| modifier.value())
            .collect();
        assert_eq!(penalties.len(), 1);
        penalties[0]
    }

    fn broken(hit_points: i32, threshold: i32) -> bool {
        hit_points > 0 && hit_points <= threshold
    }

    fn blow(
        state: &mut CombatantState,
        creature: &Creature,
        amount: i32,
        degree: DegreeOfSuccess,
        block: ShieldBlock,
    ) -> Result<(), AttackBlowError> {
        apply_attack_blow(state, creature, amount, degree, block)
    }

    /// Player Core, "Shield Statistics".
    ///
    /// (DEV-39, proposed). A shield on the Creature is wielded. No hands, no free-hand
    /// check, and no disarm.
    #[test]
    fn a_fresh_steel_shield_is_not_raised() {
        let creature = creature(20, 18, 0, false, Some(steel()));
        let state = new_combatant_state(id(1), &creature);
        let shield_hit_points = state.shield_hit_points.expect("shield");
        assert_eq!(shield_hit_points, 20);
        assert!(!state.shield_raised);
        assert!(!broken(shield_hit_points, 10));
        assert_ne!(shield_hit_points, 0);
        let shield = creature.shield.expect("template");
        assert_eq!(shield.hardness, 5);
        assert_eq!(shield.hit_points, 20);
        assert_eq!(shield.broken_threshold, 10);
        assert_eq!(shield.shield_ac_bonus, 2);
    }

    /// Player Core, "Raise a Shield".
    ///
    /// (DEV-39, proposed). A shield on the Creature is wielded. There is no hand check.
    #[test]
    fn no_shield_cannot_be_raised() {
        let creature = creature(20, 18, 0, false, None);
        let mut state = state_for(&creature);
        let error = raise_shield(&mut state, &creature).expect_err("no shield");
        assert_eq!(error, ShieldError::NoShield);
        assert_eq!(state.actions_remaining, 3);
        assert!(!state.shield_raised);
    }

    /// Player Core, Broken. Player Core, "Shield Statistics".
    ///
    /// (DEV-39, proposed). The shield is wielded, and a broken one still cannot be raised.
    #[test]
    fn a_broken_shield_cannot_be_raised() {
        let built = creature(20, 18, 0, false, Some(steel()));
        let mut state = state_for(&built);
        state.shield_hit_points = Some(10);
        let error = raise_shield(&mut state, &built).expect_err("broken");
        assert_eq!(error, ShieldError::Broken);
        assert_eq!(state.actions_remaining, 3);
        assert!(!state.shield_raised);

        let defender = participant(1, creature(30, 18, 0, false, Some(steel())));
        let (mut encounter, _) = begin(vec![defender], &[10]);
        encounter.start_turn().expect("start");
        encounter.raise_shield(id(1)).expect("raise");
        encounter
            .shield_block(id(1), 15, DegreeOfSuccess::Success)
            .expect("break the shield");
        encounter
            .end_turn(&mut crate::dice::ScriptedDice::new([]))
            .expect("end");
        encounter.start_turn().expect("next turn");
        let error = encounter.raise_shield(id(1)).expect_err("broken");
        assert_eq!(error, EncounterError::Broken);
        let encounter_state = encounter.state(id(1)).expect("state");
        assert_eq!(encounter_state.actions_remaining, 3);
        assert!(!encounter_state.shield_raised);
        assert_eq!(encounter_state.shield_hit_points, Some(10));
    }

    /// Player Core, Broken. Player Core, "Shield Statistics".
    ///
    /// (DEV-39, proposed). The shield is wielded, and a destroyed one still cannot be raised.
    #[test]
    fn a_destroyed_shield_cannot_be_raised() {
        let creature = creature(20, 18, 0, false, Some(steel()));
        let mut state = state_for(&creature);
        state.shield_hit_points = Some(0);
        let error = raise_shield(&mut state, &creature).expect_err("destroyed");
        assert_eq!(error, ShieldError::Destroyed);
        assert_eq!(state.actions_remaining, 3);
        assert!(!state.shield_raised);
    }

    /// Player Core, "Raise a Shield".
    ///
    /// (DEV-39, proposed). The shield is wielded. A dying creature still cannot raise it.
    #[test]
    fn a_dying_creature_cannot_raise_a_shield() {
        let creature = creature(20, 18, 0, true, Some(steel()));
        let mut state = state_for(&creature);
        state.hit_points = 0;
        state.vital = Vital::Dying(DyingValue::new(1).expect("dying 1"));
        state.shield_hit_points = Some(20);
        let error = raise_shield(&mut state, &creature).expect_err("dying");
        assert_eq!(error, ShieldError::NotConscious);
        assert_eq!(state.actions_remaining, 3);
        assert!(!state.shield_raised);
    }

    /// Player Core, "Raise a Shield".
    ///
    /// (DEV-39, proposed). The shield is wielded. An unconscious creature still cannot raise it.
    #[test]
    fn an_unconscious_creature_cannot_raise_a_shield() {
        let creature = creature(20, 18, 0, false, Some(steel()));
        let mut state = state_for(&creature);
        state.hit_points = 0;
        state.vital = Vital::Unconscious;
        state.shield_hit_points = Some(20);
        state.shield_raised = false;
        let error = raise_shield(&mut state, &creature).expect_err("unconscious");
        assert_eq!(error, ShieldError::NotConscious);
        assert_eq!(state.actions_remaining, 3);
        assert!(!state.shield_raised);
    }

    /// Player Core, "Raise a Shield". Player Core, "Step 2: Act".
    #[test]
    fn raising_a_steel_shield_leaves_2_actions() {
        let defender = participant(1, creature(30, 11, 0, false, Some(steel())));
        let (mut encounter, dice) = begin(vec![defender], &[10]);
        encounter.start_turn().expect("start");
        encounter.raise_shield(id(1)).expect("raise");
        let state = encounter.state(id(1)).expect("state");
        let defender = encounter.creature(id(1)).expect("creature");
        assert!(state.shield_raised);
        assert_eq!(state.actions_remaining, 2);
        assert!(state.reaction_ready);
        let total = plain_total(state, defender);
        assert_eq!(circumstance_bonuses(&total), vec![2]);
        assert_eq!(total.get(Statistic::ArmorClass), 2);
        assert_eq!(defender.shield.as_ref().expect("shield").hit_points, 20);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, "Raise a Shield". Player Core, "Step 2: Act".
    #[test]
    fn raising_again_does_not_stack_the_bonus() {
        let defender = participant(1, creature(30, 11, 0, false, Some(steel())));
        let (mut encounter, _) = begin(vec![defender], &[10]);
        encounter.start_turn().expect("start");
        encounter.raise_shield(id(1)).expect("first");
        encounter.raise_shield(id(1)).expect("second");
        let state = encounter.state(id(1)).expect("state");
        let defender = encounter.creature(id(1)).expect("creature");
        assert_eq!(state.actions_remaining, 1);
        assert!(state.shield_raised);
        let total = plain_total(state, defender);
        assert_eq!(circumstance_bonuses(&total), vec![2]);
        assert_ne!(total.get(Statistic::ArmorClass), 4);
    }

    /// Player Core, "Raise a Shield". Player Core, "Step 2: Act".
    #[test]
    fn no_actions_does_not_raise_the_shield() {
        let defender = participant(1, creature(30, 11, 0, false, Some(steel())));
        let (mut encounter, _) = begin(vec![defender], &[10]);
        encounter.start_turn().expect("start");
        encounter.spend_actions(id(1), 3).expect("spend");
        let error = encounter.raise_shield(id(1)).expect_err("no actions");
        assert_eq!(error, EncounterError::InsufficientActions);
        let state = encounter.state(id(1)).expect("state");
        assert!(!state.shield_raised);
        assert_eq!(state.actions_remaining, 0);
    }

    /// Player Core, "Raise a Shield". Player Core, "Step 2: Act".
    #[test]
    fn the_same_raise_twice_matches() {
        let run = || {
            let defender = participant(1, creature(20, 11, 0, false, Some(buckler())));
            let (mut encounter, _) = begin(vec![defender], &[10]);
            encounter.start_turn().expect("start");
            encounter.raise_shield(id(1)).expect("raise");
            let state = encounter.state(id(1)).expect("state").clone();
            let creature = encounter.creature(id(1)).expect("creature").clone();
            (state.clone(), plain_total(&state, &creature))
        };
        let (first_state, first_total) = run();
        let (second_state, second_total) = run();
        assert_eq!(first_state, second_state);
        assert!(first_state.shield_raised);
        assert!(second_state.shield_raised);
        assert_eq!(first_state.actions_remaining, 2);
        assert_eq!(second_state.actions_remaining, 2);
        assert_eq!(circumstance_bonuses(&first_total), vec![1]);
        assert_eq!(circumstance_bonuses(&second_total), vec![1]);
    }

    /// Player Core, "Raise a Shield". Player Core, "Start Your Turn".
    #[test]
    fn ending_the_turn_keeps_the_shield_raised() {
        let first = participant(1, creature(20, 11, 0, false, Some(steel())));
        let second = participant(2, creature(20, 11, 0, false, None));
        let (mut encounter, mut dice) = begin(vec![first, second], &[10, 8]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.raise_shield(id(1)).expect("raise");
        encounter.end_turn(&mut dice).expect("end");
        let state = encounter.state(id(1)).expect("state");
        let defender = encounter.creature(id(1)).expect("creature");
        assert!(state.shield_raised);
        assert_eq!(state.shield_hit_points, Some(20));
        assert_eq!(state.actions_remaining, 0);
        assert!(state.reaction_ready);
        assert_eq!(circumstance_bonuses(&plain_total(state, defender)), vec![2]);
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, "Raise a Shield". Player Core, "Start Your Turn".
    #[test]
    fn another_turn_does_not_lower_the_shield() {
        let first = participant(1, creature(20, 11, 0, false, Some(steel())));
        let second = participant(2, creature(20, 11, 0, false, None));
        let (mut encounter, _) = begin(vec![first, second], &[10, 8]);
        encounter.start_turn().expect("start");
        encounter.raise_shield(id(1)).expect("raise");
        encounter
            .end_turn(&mut crate::dice::ScriptedDice::new([]))
            .expect("end");
        assert_eq!(encounter.start_turn().expect("other"), id(2));
        encounter
            .end_turn(&mut crate::dice::ScriptedDice::new([]))
            .expect("end other");
        let state = encounter.state(id(1)).expect("state");
        let defender = encounter.creature(id(1)).expect("creature");
        assert!(state.shield_raised);
        assert_eq!(circumstance_bonuses(&plain_total(state, defender)), vec![2]);
    }

    /// Player Core, "Raise a Shield". Player Core, "Start Your Turn".
    #[test]
    fn the_next_turn_starts_with_the_shield_lowered() {
        let first = participant(1, creature(20, 11, 0, false, Some(steel())));
        let second = participant(2, creature(20, 11, 0, false, None));
        let (mut encounter, _) = begin(vec![first, second], &[10, 8]);
        encounter.start_turn().expect("start");
        encounter.raise_shield(id(1)).expect("raise");
        encounter
            .end_turn(&mut crate::dice::ScriptedDice::new([]))
            .expect("end");
        encounter.start_turn().expect("other");
        encounter
            .end_turn(&mut crate::dice::ScriptedDice::new([]))
            .expect("end other");
        assert_eq!(encounter.start_turn().expect("next"), id(1));
        let state = encounter.state(id(1)).expect("state");
        let defender = encounter.creature(id(1)).expect("creature");
        assert!(!state.shield_raised);
        assert_eq!(state.shield_hit_points, Some(20));
        assert_eq!(state.actions_remaining, 3);
        assert!(state.reaction_ready);
        assert_eq!(plain_total(state, defender).get(Statistic::ArmorClass), 0);
    }

    /// Player Core, "Raise a Shield". Player Core, "Start Your Turn".
    #[test]
    fn round_overflow_leaves_the_shield_raised() {
        let first = participant(1, creature(20, 11, 0, false, Some(steel())));
        let second = participant(2, creature(20, 11, 0, false, None));
        let (mut encounter, _) = begin(vec![first, second], &[10, 8]);
        encounter.start_turn().expect("start");
        encounter.raise_shield(id(1)).expect("raise");
        encounter
            .end_turn(&mut crate::dice::ScriptedDice::new([]))
            .expect("end");
        assert_eq!(encounter.start_turn().expect("other"), id(2));
        encounter.set_round_for_test(4_294_967_295);
        let error = encounter
            .end_turn(&mut crate::dice::ScriptedDice::new([]))
            .expect_err("overflow");
        assert_eq!(error, EncounterError::RoundOverflow);
        assert_eq!(encounter.round(), 4_294_967_295);
        assert_eq!(encounter.turn_open(), Some(id(2)));
        let state = encounter.state(id(1)).expect("state");
        assert!(state.shield_raised);
        assert_eq!(state.shield_hit_points, Some(20));
    }

    /// Player Core, "Raise a Shield". Player Core, "Start Your Turn".
    #[test]
    fn a_dead_shield_is_never_lowered() {
        let mut first = participant(1, creature(20, 11, 0, false, Some(steel())));
        first.state.hit_points = 0;
        first.state.vital = Vital::Dead;
        first.state.shield_raised = true;
        first.state.shield_hit_points = Some(20);
        let second = participant(2, creature(20, 11, 0, false, None));
        let (mut encounter, dice) = begin(vec![first, second], &[10, 8]);
        assert_eq!(encounter.start_turn().expect("start"), id(2));
        let state = encounter.state(id(1)).expect("state");
        let defender = encounter.creature(id(1)).expect("creature");
        assert!(state.shield_raised);
        assert_eq!(state.shield_hit_points, Some(20));
        assert_eq!(
            ac_total(state, defender, None, None, OffGuard::No, AttackKind::Melee)
                .get(Statistic::ArmorClass),
            0
        );
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, Bonuses. Player Core, "Raise a Shield".
    #[test]
    fn a_buckler_grants_plus_1() {
        let creature = creature(20, 11, 0, false, Some(buckler()));
        let state = raised(&creature, 6);
        let total = plain_total(&state, &creature);
        assert_eq!(circumstance_bonuses(&total), vec![1]);
        assert_eq!(total.get(Statistic::ArmorClass), 1);
        assert_ne!(total.get(Statistic::ArmorClass), 2);
    }

    /// Player Core, Bonuses. Player Core, "Raise a Shield".
    #[test]
    fn standard_cover_and_a_steel_shield_stay_plus_2() {
        let creature = creature(20, 11, 0, false, Some(steel()));
        let state = raised(&creature, 20);
        let total = ac_total(
            &state,
            &creature,
            None,
            Some(Cover::Standard),
            OffGuard::No,
            AttackKind::Melee,
        );
        assert_eq!(circumstance_bonuses(&total), vec![2]);
        assert_eq!(total.get(Statistic::ArmorClass), 2);
        assert_ne!(total.get(Statistic::ArmorClass), 4);
    }

    /// Player Core, Bonuses. Player Core, "Raise a Shield".
    #[test]
    fn greater_cover_beats_the_steel_shield() {
        let creature = creature(20, 11, 0, false, Some(steel()));
        let state = raised(&creature, 20);
        let total = ac_total(
            &state,
            &creature,
            None,
            Some(Cover::Greater),
            OffGuard::No,
            AttackKind::Melee,
        );
        assert_eq!(circumstance_bonuses(&total), vec![4]);
        assert_eq!(total.get(Statistic::ArmorClass), 4);
        assert!(!circumstance_bonuses(&total).contains(&2));
    }

    /// Player Core, Penalties. Player Core, "Off-Guard". Player Core, "Raise a Shield".
    #[test]
    fn a_steel_shield_and_off_guard_total_0() {
        let creature = creature(20, 11, 0, false, Some(steel()));
        let state = raised(&creature, 20);
        let total = ac_total(
            &state,
            &creature,
            None,
            None,
            OffGuard::Yes,
            AttackKind::Melee,
        );
        assert_eq!(circumstance_bonuses(&total), vec![2]);
        assert_eq!(circumstance_penalty(&total), -2);
        assert_eq!(total.get(Statistic::ArmorClass), 0);
    }

    /// Player Core, Bonuses. Player Core, "Raise a Shield".
    #[test]
    fn a_lowered_shield_adds_nothing_beside_cover() {
        let creature = creature(20, 11, 0, false, Some(steel()));
        let state = state_for(&creature);
        assert!(shield_ac_modifier(&state, &creature).is_none());
        let total = ac_total(
            &state,
            &creature,
            None,
            Some(Cover::Standard),
            OffGuard::No,
            AttackKind::Melee,
        );
        assert_eq!(circumstance_bonuses(&total), vec![2]);
        assert_eq!(total.get(Statistic::ArmorClass), 2);
        assert!(
            total
                .applied()
                .iter()
                .all(|modifier| modifier.sequence() != 11)
        );
    }

    /// Player Core, Bonuses. Player Core, "Raise a Shield".
    #[test]
    fn the_same_armor_class_total_twice_matches() {
        let run = || {
            let creature = creature(20, 11, 0, false, Some(steel()));
            let state = raised(&creature, 20);
            (
                state.clone(),
                ac_total(
                    &state,
                    &creature,
                    None,
                    Some(Cover::Standard),
                    OffGuard::No,
                    AttackKind::Melee,
                ),
            )
        };
        let (first_state, first_total) = run();
        let (second_state, second_total) = run();
        assert_eq!(first_total.get(Statistic::ArmorClass), 2);
        assert_eq!(second_total.get(Statistic::ArmorClass), 2);
        assert_eq!(first_state, second_state);
    }

    /// Player Core, Bonuses. Player Core, Penalties. Player Core, "Off-Guard".
    /// Player Core, "Raise a Shield".
    #[test]
    fn prone_with_greater_cover_against_ranged_totals_0_and_plus_2() {
        let creature = creature(20, 11, 0, false, Some(steel()));
        let state = raised(&creature, 20);
        let melee = ac_total(
            &state,
            &creature,
            Some(CoverBenefit::GreaterVsRanged),
            None,
            OffGuard::Yes,
            AttackKind::Melee,
        );
        let ranged = ac_total(
            &state,
            &creature,
            Some(CoverBenefit::GreaterVsRanged),
            None,
            OffGuard::Yes,
            AttackKind::Ranged,
        );
        assert_eq!(melee.get(Statistic::ArmorClass), 0);
        assert_eq!(ranged.get(Statistic::ArmorClass), 2);
        assert_eq!(circumstance_bonuses(&melee), vec![2]);
        assert_eq!(circumstance_bonuses(&ranged), vec![4]);
    }

    /// Player Core, Broken. Player Core, "Shield Statistics".
    #[test]
    fn a_broken_raised_shield_adds_no_bonus() {
        let creature = creature(20, 11, 0, false, Some(steel()));
        let state = raised(&creature, 10);
        let total = plain_total(&state, &creature);
        assert_eq!(total.get(Statistic::ArmorClass), 0);
        let value = total.get(Statistic::ArmorClass);
        assert_ne!(value, -1);
        assert_ne!(value, -2);
        assert_ne!(value, -3);
    }

    /// Player Core, "Shield Block". Player Core, Shields.
    ///
    /// (DEV-38, proposed). Any creature whose Creature has a shield may Shield Block while
    /// that shield is raised. The call takes no feat.
    #[test]
    fn no_feat_record_is_consulted() {
        let creature = creature(30, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 20);
        state.actions_remaining = 2;
        blow(
            &mut state,
            &creature,
            12,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect("block");
        assert_eq!(state.hit_points, 23);
        assert_eq!(state.shield_hit_points, Some(13));
        assert!(!state.reaction_ready);
    }

    /// Player Core, "Shield Block". Player Core, Shields. Player Core, "Shield Statistics".
    /// Player Core, "Item Damage".
    #[test]
    fn hardness_5_turns_12_damage_into_7() {
        let creature = creature(30, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 20);
        state.actions_remaining = 2;
        blow(
            &mut state,
            &creature,
            12,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect("block");
        assert_eq!(state.hit_points, 23);
        assert_eq!(state.shield_hit_points, Some(13));
        assert!(!broken(13, 10));
        assert!(!state.reaction_ready);
        assert_eq!(state.actions_remaining, 2);
        assert_eq!(creature.shield.expect("shield").hit_points, 20);
        assert_ne!(state.shield_hit_points, Some(18));
    }

    /// Player Core, "Shield Block".
    #[test]
    fn damage_equal_to_hardness_deals_0() {
        let creature = creature(30, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 20);
        blow(
            &mut state,
            &creature,
            5,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect("block");
        assert_eq!(state.hit_points, 30);
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(!state.reaction_ready);
    }

    /// Player Core, "Shield Block".
    #[test]
    fn damage_below_hardness_deals_0() {
        let creature = creature(30, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 20);
        blow(
            &mut state,
            &creature,
            4,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect("block");
        assert_eq!(state.hit_points, 30);
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(!state.reaction_ready);

        let mut one = raised(&creature, 20);
        blow(
            &mut one,
            &creature,
            1,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect("one damage");
        assert_eq!(one.hit_points, 30);
        assert_eq!(one.shield_hit_points, Some(20));
        assert!(!one.reaction_ready);
    }

    /// Player Core, "Shield Block". Player Core, Shields.
    #[test]
    fn a_buckler_turns_8_damage_into_5() {
        let creature = creature(30, 11, 0, false, Some(buckler()));
        let mut state = raised(&creature, 6);
        blow(
            &mut state,
            &creature,
            8,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect("block");
        assert_eq!(state.hit_points, 25);
        assert_eq!(state.shield_hit_points, Some(1));
        assert!(broken(1, 3));
        assert_ne!(state.shield_hit_points, Some(0));
        assert!(!state.reaction_ready);
    }

    /// Player Core, "Shield Statistics". Player Core, "Shield Block".
    #[test]
    fn the_broken_threshold_breaks_the_steel_shield() {
        let creature = creature(30, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 20);
        blow(
            &mut state,
            &creature,
            15,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect("block");
        assert_eq!(state.hit_points, 20);
        assert_eq!(state.shield_hit_points, Some(10));
        assert!(broken(10, 10));
        assert_ne!(state.shield_hit_points, Some(0));

        let mut boundary = raised(&creature, 11);
        blow(
            &mut boundary,
            &creature,
            6,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect("remainder 1");
        assert_eq!(boundary.hit_points, 29);
        assert_eq!(boundary.shield_hit_points, Some(10));
        assert!(broken(10, 10));
        assert!(!boundary.reaction_ready);
    }

    /// Player Core, "Shield Block". Player Core, "Shield Statistics".
    #[test]
    fn reducing_the_shield_to_0_destroys_it() {
        let creature = creature(40, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 20);
        blow(
            &mut state,
            &creature,
            25,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect("block");
        assert_eq!(state.hit_points, 20);
        assert_eq!(state.shield_hit_points, Some(0));
        assert_eq!(plain_total(&state, &creature).get(Statistic::ArmorClass), 0);
    }

    /// Player Core, "Shield Block". Player Core, "Item Damage".
    #[test]
    fn damage_past_0_hit_points_destroys_the_shield_at_0() {
        let creature = creature(40, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 20);
        blow(
            &mut state,
            &creature,
            30,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect("block");
        assert_eq!(state.hit_points, 15);
        assert_eq!(state.shield_hit_points, Some(0));
        assert_ne!(state.shield_hit_points, Some(-5));
    }

    /// Player Core, "Shield Block".
    #[test]
    fn declining_the_block_leaves_the_shield_intact() {
        let creature = creature(30, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 20);
        blow(
            &mut state,
            &creature,
            12,
            DegreeOfSuccess::Success,
            ShieldBlock::Declined,
        )
        .expect("decline");
        assert_eq!(state.hit_points, 18);
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(state.reaction_ready);
    }

    /// Player Core, "Shield Block".
    #[test]
    fn declining_with_no_shield_deals_the_blow() {
        let creature = creature(30, 11, 0, false, None);
        let mut state = state_for(&creature);
        let result = blow(
            &mut state,
            &creature,
            12,
            DegreeOfSuccess::Success,
            ShieldBlock::Declined,
        );
        assert_eq!(result, Ok(()));
        assert_eq!(state.hit_points, 18);
    }

    /// Player Core, "Shield Block".
    #[test]
    fn a_shield_that_is_not_raised_does_not_block() {
        let creature = creature(30, 11, 0, false, Some(steel()));
        let mut state = state_for(&creature);
        let error = blow(
            &mut state,
            &creature,
            12,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect_err("not raised");
        assert_eq!(error, AttackBlowError::NotRaised);
        assert_eq!(state.hit_points, 30);
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(state.reaction_ready);
    }

    /// Player Core, "Shield Block".
    #[test]
    fn a_creature_with_no_shield_does_not_block() {
        let creature = creature(30, 11, 0, false, None);
        let mut state = state_for(&creature);
        let error = blow(
            &mut state,
            &creature,
            12,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect_err("no shield");
        assert_eq!(error, AttackBlowError::NoShield);
        assert_eq!(state.hit_points, 30);
        assert!(state.reaction_ready);
    }

    /// Player Core, "Shield Block".
    #[test]
    fn a_spent_reaction_does_not_block() {
        let creature = creature(30, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 20);
        state.reaction_ready = false;
        let error = blow(
            &mut state,
            &creature,
            12,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect_err("no reaction");
        assert_eq!(error, AttackBlowError::NoReaction);
        assert_eq!(state.hit_points, 30);
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(!state.reaction_ready);
    }

    /// Player Core, "Shield Block".
    #[test]
    fn a_broken_shield_does_not_block() {
        let creature = creature(30, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 10);
        let error = blow(
            &mut state,
            &creature,
            12,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect_err("broken");
        assert_eq!(error, AttackBlowError::Broken);
        assert_eq!(state.hit_points, 30);
        assert_eq!(state.shield_hit_points, Some(10));
        assert!(state.reaction_ready);
    }

    /// Player Core, "Shield Block".
    #[test]
    fn a_destroyed_shield_does_not_block() {
        let creature = creature(30, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 0);
        let error = blow(
            &mut state,
            &creature,
            12,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect_err("destroyed");
        assert_eq!(error, AttackBlowError::Broken);
        assert_eq!(state.hit_points, 30);
        assert_eq!(state.shield_hit_points, Some(0));
        assert!(state.reaction_ready);
    }

    /// Player Core, "Shield Block".
    #[test]
    fn dying_on_the_direct_state_does_not_block() {
        let creature = creature(9, 11, 0, true, Some(steel()));
        let mut state = raised(&creature, 20);
        state.hit_points = 0;
        state.vital = Vital::Dying(DyingValue::new(1).expect("dying 1"));
        let error = blow(
            &mut state,
            &creature,
            8,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect_err("dying");
        assert_eq!(error, AttackBlowError::NotConscious);
        assert_eq!(
            state.vital,
            Vital::Dying(DyingValue::new(1).expect("dying 1"))
        );
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(state.reaction_ready);
    }

    /// Player Core, "Shield Block". Player Core, Strike.
    #[test]
    fn a_failure_is_not_a_blow() {
        let creature = creature(30, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 20);
        let error = blow(
            &mut state,
            &creature,
            12,
            DegreeOfSuccess::Failure,
            ShieldBlock::Requested,
        )
        .expect_err("miss");
        assert_eq!(error, AttackBlowError::HitPoints(HpError::NotAHit));
        assert_eq!(state.hit_points, 30);
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(state.reaction_ready);

        let negative = blow(
            &mut state,
            &creature,
            -1,
            DegreeOfSuccess::Failure,
            ShieldBlock::Requested,
        )
        .expect_err("negative miss");
        assert_eq!(negative, AttackBlowError::HitPoints(HpError::NotAHit));
        assert_eq!(state.hit_points, 30);
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(state.reaction_ready);
    }

    /// Player Core, "Shield Block". Player Core, Strike.
    #[test]
    fn a_critical_failure_is_not_a_blow() {
        let creature = creature(30, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 20);
        let error = blow(
            &mut state,
            &creature,
            12,
            DegreeOfSuccess::CriticalFailure,
            ShieldBlock::Requested,
        )
        .expect_err("miss");
        assert_eq!(error, AttackBlowError::HitPoints(HpError::NotAHit));
        assert_eq!(state.hit_points, 30);
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(state.reaction_ready);
    }

    /// Player Core, "Shield Block". Player Core, Strike.
    #[test]
    fn negative_hardness_on_a_failure_is_notahit() {
        let mut creature = creature(30, 11, 0, false, Some(steel()));
        creature.shield = Some(ShieldTemplate {
            hardness: -1,
            hit_points: 20,
            broken_threshold: 10,
            shield_ac_bonus: 2,
        });
        let mut state = raised(&creature, 20);
        let error = blow(
            &mut state,
            &creature,
            12,
            DegreeOfSuccess::Failure,
            ShieldBlock::Requested,
        )
        .expect_err("miss");
        assert_eq!(error, AttackBlowError::HitPoints(HpError::NotAHit));
        assert_eq!(state.hit_points, 30);
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(state.reaction_ready);
    }

    /// Player Core, "Shield Block". Player Core, Strike.
    #[test]
    fn a_broken_shield_on_a_failure_is_notahit() {
        let creature = creature(30, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 10);
        let error = blow(
            &mut state,
            &creature,
            12,
            DegreeOfSuccess::Failure,
            ShieldBlock::Requested,
        )
        .expect_err("miss");
        assert_eq!(error, AttackBlowError::HitPoints(HpError::NotAHit));
        assert_eq!(state.hit_points, 30);
        assert_eq!(state.shield_hit_points, Some(10));
        assert!(state.reaction_ready);
    }

    /// Player Core, "Shield Block".
    #[test]
    fn a_zero_amount_spends_nothing() {
        let creature = creature(30, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 20);
        let result = blow(
            &mut state,
            &creature,
            0,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        );
        assert_eq!(result, Ok(()));
        assert_eq!(state.hit_points, 30);
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(state.reaction_ready);
    }

    /// Player Core, "Shield Block".
    #[test]
    fn a_declined_negative_amount_writes_nothing() {
        let creature = creature(30, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 20);
        let error = blow(
            &mut state,
            &creature,
            -1,
            DegreeOfSuccess::Success,
            ShieldBlock::Declined,
        )
        .expect_err("negative");
        assert_eq!(error, AttackBlowError::NegativeAmount);
        assert_eq!(state.hit_points, 30);
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(state.reaction_ready);
    }

    /// Player Core, "Shield Block".
    #[test]
    fn a_negative_amount_writes_nothing() {
        let creature = creature(30, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 20);
        let error = blow(
            &mut state,
            &creature,
            -1,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect_err("negative");
        assert_eq!(error, AttackBlowError::NegativeAmount);
        assert_eq!(state.hit_points, 30);
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(state.reaction_ready);
    }

    /// Player Core, "Shield Block". Player Core, Shields.
    #[test]
    fn the_maximum_amount_does_not_wrap() {
        let creature = creature(20, 11, 0, true, Some(steel()));
        let mut state = raised(&creature, 20);
        let result = blow(
            &mut state,
            &creature,
            2_147_483_647,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        );
        assert_eq!(result, Ok(()));
        assert_eq!(state.vital, Vital::Dead);
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.shield_hit_points, Some(0));
    }

    /// Player Core, "Massive Damage".
    #[test]
    fn a_blocked_22_is_not_massive_damage() {
        let creature = creature(10, 11, 0, true, Some(steel()));
        let mut state = raised(&creature, 20);
        blow(
            &mut state,
            &creature,
            22,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect("block");
        assert_eq!(state.hit_points, 0);
        assert_eq!(
            state.vital,
            Vital::Dying(DyingValue::new(1).expect("dying 1"))
        );
        assert_ne!(state.vital, Vital::Dead);
        assert_eq!(state.shield_hit_points, Some(3));
        assert!(broken(3, 10));
        assert!(!state.reaction_ready);
    }

    /// Player Core, "Massive Damage".
    #[test]
    fn the_same_22_without_a_block_is_massive_damage() {
        let creature = creature(10, 11, 0, true, Some(steel()));
        let mut state = raised(&creature, 20);
        blow(
            &mut state,
            &creature,
            22,
            DegreeOfSuccess::Success,
            ShieldBlock::Declined,
        )
        .expect("decline");
        assert_eq!(state.vital, Vital::Dead);
        assert!(!matches!(state.vital, Vital::Dying(_)));
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(state.reaction_ready);
    }

    /// Player Core, "Temporary Hit Points".
    #[test]
    fn temporary_hit_points_are_spent_by_the_resolved_blow() {
        let creature = creature(9, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 20);
        state.hit_points = 9;
        state.temp_hit_points = 5;
        state.temp_rounds = 3;
        blow(
            &mut state,
            &creature,
            12,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect("block");
        assert_eq!(state.temp_hit_points, 0);
        assert_eq!(state.temp_rounds, 0);
        assert_eq!(state.hit_points, 7);
        assert_eq!(state.vital, Vital::Alive);
        assert_eq!(state.shield_hit_points, Some(13));
    }

    /// Player Core, "Massive Damage". Player Core, "Taking Damage while Dying".
    #[test]
    fn a_critical_blow_is_not_doubled_again() {
        let creature = creature(10, 11, 0, true, Some(steel()));
        let mut state = raised(&creature, 20);
        blow(
            &mut state,
            &creature,
            20,
            DegreeOfSuccess::CriticalSuccess,
            ShieldBlock::Requested,
        )
        .expect("block");
        assert_eq!(state.hit_points, 0);
        assert_eq!(
            state.vital,
            Vital::Dying(DyingValue::new(2).expect("dying 2"))
        );
        assert_ne!(state.vital, Vital::Dead);
        assert_eq!(state.shield_hit_points, Some(5));
        assert!(broken(5, 10));
    }

    /// Player Core, "Shield Block". Player Core, "Massive Damage".
    #[test]
    fn a_remainder_of_0_spends_the_reaction_and_stays_alive() {
        let creature = creature(30, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 20);
        blow(
            &mut state,
            &creature,
            4,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect("block");
        assert_eq!(state.hit_points, 30);
        assert_eq!(state.vital, Vital::Alive);
        assert!(!matches!(state.vital, Vital::Dying(_)));
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(!state.reaction_ready);
    }

    /// Player Core, "Temporary Hit Points". Player Core, "Taking Damage while Dying".
    #[test]
    fn temporary_hit_points_do_not_stop_the_dying_increase() {
        let creature = creature(9, 11, 0, true, Some(steel()));
        let mut state = raised(&creature, 20);
        state.hit_points = 0;
        state.vital = Vital::Dying(DyingValue::new(1).expect("dying 1"));
        state.temp_hit_points = 5;
        state.temp_rounds = 3;
        blow(
            &mut state,
            &creature,
            3,
            DegreeOfSuccess::Success,
            ShieldBlock::Declined,
        )
        .expect("decline");
        assert_eq!(state.temp_hit_points, 2);
        assert_eq!(state.temp_rounds, 3);
        assert_eq!(
            state.vital,
            Vital::Dying(DyingValue::new(2).expect("dying 2"))
        );
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(state.reaction_ready);
    }

    /// Player Core, "Massive Damage". Player Core, "Shield Block".
    #[test]
    fn a_direct_resolved_blow_does_not_touch_the_shield() {
        let creature = creature(30, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 20);
        apply_resolved_blow(&mut state, &creature, 12, DegreeOfSuccess::Success).expect("blow");
        assert_eq!(state.hit_points, 18);
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(state.shield_raised);
        assert!(state.reaction_ready);
    }

    /// Player Core, "Getting Knocked Out". Player Core, "Shield Block".
    #[test]
    fn an_inconsistent_vital_writes_nothing() {
        let built = creature(9, 11, 0, true, Some(steel()));
        let mut state = raised(&built, 20);
        state.hit_points = 0;
        state.vital = Vital::Alive;
        let error = blow(
            &mut state,
            &built,
            12,
            DegreeOfSuccess::Success,
            ShieldBlock::Requested,
        )
        .expect_err("inconsistent");
        assert_eq!(
            error,
            AttackBlowError::HitPoints(HpError::InconsistentVital)
        );
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(state.reaction_ready);

        let defender = participant(1, creature(9, 11, 0, true, Some(steel())));
        let attacker = participant(2, creature(20, 11, 0, false, None));
        let (mut encounter, mut dice) = begin(vec![defender, attacker], &[10, 8, 11, 8]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.raise_shield(id(1)).expect("raise");
        encounter.end_turn(&mut dice).expect("end");
        assert_eq!(encounter.start_turn().expect("attacker"), id(2));
        encounter
            .strike(
                id(2),
                id(1),
                &profile(8, 1),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect("drop to 0");
        encounter.set_vital_for_test(id(1), Vital::Alive);
        let before = encounter.state(id(1)).expect("state").clone();
        assert_eq!(before.hit_points, 0);
        assert_eq!(before.vital, Vital::Alive);
        assert!(before.shield_raised);
        assert_eq!(before.shield_hit_points, Some(20));
        assert!(before.reaction_ready);
        let rolled_back = encounter
            .shield_block(id(1), 12, DegreeOfSuccess::Success)
            .expect_err("rollback");
        assert_eq!(
            rolled_back,
            EncounterError::HitPoints(HpError::InconsistentVital)
        );
        assert_eq!(encounter.state(id(1)).expect("state"), &before);
    }

    /// Player Core, "Shield Block".
    #[test]
    fn the_same_block_twice_matches() {
        let run = || {
            let creature = creature(30, 11, 0, false, Some(steel()));
            let mut state = raised(&creature, 20);
            blow(
                &mut state,
                &creature,
                12,
                DegreeOfSuccess::Success,
                ShieldBlock::Requested,
            )
            .expect("block");
            state
        };
        let first = run();
        let second = run();
        assert_eq!(first, second);
        assert_eq!(first.hit_points, 23);
        assert_eq!(second.hit_points, 23);
        assert_eq!(first.shield_hit_points, Some(13));
        assert_eq!(second.shield_hit_points, Some(13));
    }

    /// Player Core, "Shield Block".
    #[test]
    fn persistent_damage_leaves_the_shield_and_the_reaction() {
        let creature = creature(30, 11, 0, false, Some(steel()));
        let mut state = raised(&creature, 20);
        apply_resolved_blow(&mut state, &creature, 5, DegreeOfSuccess::Success)
            .expect("persistent");
        assert_eq!(state.hit_points, 25);
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(state.reaction_ready);
    }

    /// Player Core, Strike. Player Core, "Shield Block".
    #[test]
    fn a_blocked_strike_still_hits_baked_armor_class() {
        let defender = participant(1, creature(30, 11, 0, false, Some(steel())));
        let attacker = participant(2, creature(20, 11, 0, false, None));
        let (mut encounter, mut dice) = begin(vec![defender, attacker], &[10, 8, 11, 8]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.raise_shield(id(1)).expect("raise");
        encounter.end_turn(&mut dice).expect("end");
        assert_eq!(encounter.start_turn().expect("attacker"), id(2));
        let outcome = rolled(
            encounter
                .strike(
                    id(2),
                    id(1),
                    &profile(8, 4),
                    &[],
                    ShieldBlock::Requested,
                    &Detections::new(),
                    &mut dice,
                )
                .expect("strike"),
        );
        assert_eq!(outcome.degree, DegreeOfSuccess::Success);
        let state = encounter.state(id(1)).expect("state");
        assert_eq!(state.hit_points, 23);
        assert_eq!(state.shield_hit_points, Some(13));
        assert!(!state.reaction_ready);
        assert_eq!(
            encounter.state(id(2)).expect("attacker").actions_remaining,
            2
        );
        assert_eq!(dice.remaining(), 0);
    }

    /// Player Core, Strike. Player Core, "Shield Block".
    #[test]
    fn a_strike_that_does_not_block_leaves_the_shield() {
        let defender = participant(1, creature(20, 11, 0, false, Some(steel())));
        let attacker = participant(2, creature(20, 11, 0, false, None));
        let (mut encounter, mut dice) = begin(vec![defender, attacker], &[10, 8, 11, 8]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.end_turn(&mut dice).expect("end");
        assert_eq!(encounter.start_turn().expect("attacker"), id(2));
        let outcome = rolled(
            encounter
                .strike(
                    id(2),
                    id(1),
                    &profile(8, 0),
                    &[],
                    ShieldBlock::Declined,
                    &Detections::new(),
                    &mut dice,
                )
                .expect("strike"),
        );
        assert_eq!(outcome.degree, DegreeOfSuccess::Success);
        let state = encounter.state(id(1)).expect("state");
        assert_eq!(state.hit_points, 12);
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(!state.shield_raised);
    }

    /// Player Core, Strike. Player Core, "Shield Block".
    #[test]
    fn a_strike_error_spends_nothing_and_draws_nothing() {
        let defender = participant(1, creature(30, 11, 0, false, Some(steel())));
        let attacker = participant(2, creature(20, 11, 0, false, None));
        let (mut encounter, mut dice) = begin(vec![defender, attacker], &[1, 10, 11, 8]);
        assert_eq!(encounter.start_turn().expect("attacker"), id(2));
        assert_eq!(
            encounter.state(id(2)).expect("attacker").actions_remaining,
            3
        );
        let error = encounter
            .strike(
                id(2),
                id(1),
                &profile(8, 0),
                &[],
                ShieldBlock::Requested,
                &Detections::new(),
                &mut dice,
            )
            .expect_err("not raised");
        assert_eq!(error, EncounterError::NotRaised);
        assert_eq!(
            encounter.state(id(2)).expect("attacker").actions_remaining,
            3
        );
        assert_eq!(dice.remaining(), 2);
        let state = encounter.state(id(1)).expect("state");
        assert_eq!(state.hit_points, 30);
        assert_eq!(state.shield_hit_points, Some(20));
    }

    /// Player Core, Strike. Player Core, "Shield Block".
    #[test]
    fn a_strike_against_a_dying_creature_does_not_block() {
        let defender = participant(1, creature(9, 11, 0, true, Some(steel())));
        let attacker = participant(2, creature(20, 11, 0, false, None));
        let (mut encounter, mut dice) = begin(vec![defender, attacker], &[10, 8, 11, 8, 11, 8]);
        assert_eq!(encounter.start_turn().expect("start"), id(1));
        encounter.raise_shield(id(1)).expect("raise");
        encounter.end_turn(&mut dice).expect("end");
        assert_eq!(encounter.start_turn().expect("attacker"), id(2));
        encounter
            .strike(
                id(2),
                id(1),
                &profile(8, 1),
                &[],
                ShieldBlock::Declined,
                &Detections::new(),
                &mut dice,
            )
            .expect("first");
        let error = encounter
            .strike(
                id(2),
                id(1),
                &profile(8, 1),
                &[],
                ShieldBlock::Requested,
                &Detections::new(),
                &mut dice,
            )
            .expect_err("dying");
        assert_eq!(error, EncounterError::TargetNotConscious);
        let state = encounter.state(id(1)).expect("state");
        assert_eq!(
            state.vital,
            Vital::Dying(DyingValue::new(1).expect("dying 1"))
        );
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.shield_hit_points, Some(20));
        assert!(state.reaction_ready);
        assert_eq!(
            encounter.state(id(2)).expect("attacker").actions_remaining,
            2
        );
        assert_eq!(dice.remaining(), 2);
    }

    /// Covers dying-and-victory scenarios "A non-critical blow to 0 sets dying 1"
    /// and "Recovery to 0 stays unconscious at 0 Hit Points".
    /// The raised shield stays on the combatant's state until that creature's next turn starts.
    /// Player Core pg. 410, "Getting Knocked Out". Player Core pg. 446, Unconscious.
    /// Player Core pg. 419, "Raise a Shield".
    #[test]
    fn falling_unconscious_keeps_the_held_strike_and_the_shield() {
        let creature = creature(9, 11, 0, true, Some(steel()));
        let mut state = state_for(&creature);
        raise_shield(&mut state, &creature).expect("raise");
        apply_resolved_blow(&mut state, &creature, 9, DegreeOfSuccess::Success).expect("knockout");
        assert_eq!(state.hit_points, 0);
        assert_eq!(
            state.vital,
            Vital::Dying(DyingValue::new(1).expect("dying 1"))
        );
        assert!(state.shield_raised);
        assert_eq!(state.shield_hit_points, Some(20));
        assert_eq!(
            shield_ac_modifier(&state, &creature)
                .expect("raised shield")
                .value(),
            2
        );

        let mut dice = ScriptedDice::new([11]);
        let roll = start_of_turn(&mut state, &creature, &mut dice)
            .expect("recovery")
            .expect("rolled");
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(state.vital, Vital::Unconscious);
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.wounded, 1);
        assert!(state.shield_raised);
        assert_eq!(state.shield_hit_points, Some(20));
        assert_eq!(
            shield_ac_modifier(&state, &creature)
                .expect("shield still raised")
                .value(),
            2
        );
        assert_eq!(dice.remaining(), 0);
    }
}
