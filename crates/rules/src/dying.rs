//! Dying, recovery, temporary Hit Points, the initiative report, and victory.
//!
//! Strike and turn order stay in the encounter module. This module does not reorder combatants.
//!
//! Player Core pg. 410, "Getting Knocked Out".

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "callers are this crate's tests; the facade keeps the procedure and drops the root export"
    )
)]

use crate::check::{DegreeOfSuccess, FlatRoll, flat_check};
use crate::combatant::{
    CombatantId, CombatantState, ConditionSlot, Creature, DyingValue, HpError, Vital,
};
use crate::damage::DamageType;
use crate::dice::Dice;
use crate::encounter::Side;

/// A report that a blow newly stored a dying value. The caller takes it once.
///
/// Player Core pg. 410, "Getting Knocked Out".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InitiativeMove {
    /// The combatant gained a dying value and has not been moved yet.
    ///
    /// Player Core pg. 410, "Getting Knocked Out".
    Pending,
}

/// Which side, if any, has won. The query does not store a winner.
///
/// Player Core pg. 435, "End the Encounter".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EncounterOutcome {
    /// Fewer than two sides are present, or both still have an alive combatant.
    ///
    /// Player Core pg. 435, "End the Encounter".
    Ongoing,
    /// Exactly one side still has an alive combatant.
    ///
    /// Player Core pg. 435, "End the Encounter".
    Victory {
        /// The side that still has an alive combatant. Player Core pg. 435, "End the Encounter".
        side: Side,
    },
    /// Both sides are present and neither has an alive combatant.
    ///
    /// Player Core pg. 435, "End the Encounter".
    MutualDefeat,
}

/// One combatant in a victory query.
///
/// Player Core pg. 435, "End the Encounter".
pub(crate) struct RosterEntry<'a> {
    /// Combatant id. Player Core, Encounter Mode.
    #[cfg_attr(
        test,
        expect(
            dead_code,
            reason = "the roster value keeps the id; nothing in this crate reads it"
        )
    )]
    pub id: CombatantId,
    /// Side from encounter turns. Player Core pg. 435, "End the Encounter".
    pub side: Side,
    /// State the query reads. It is not written. Player Core pg. 435, "End the Encounter".
    pub state: &'a CombatantState,
}

/// Alive at 0 Hit Points, or dying or unconscious at any other total.
///
/// Player Core pg. 410, "Getting Knocked Out".
pub(crate) fn inconsistent_vital(state: &CombatantState) -> bool {
    match state.vital {
        Vital::Alive => state.hit_points == 0,
        Vital::Dying(_) | Vital::Unconscious => state.hit_points != 0,
        Vital::Dead => false,
    }
}

/// Resolves one already-calculated blow.
///
/// [`DegreeOfSuccess::CriticalSuccess`] is a critical blow. [`DegreeOfSuccess::Success`] is an
/// ordinary blow. A miss returns [`HpError::NotAHit`] and writes nothing. A negative amount
/// returns [`HpError::NegativeAmount`] and writes nothing. This function does not roll.
///
/// Player Core pg. 410, "Getting Knocked Out". Player Core pg. 412, Massive Damage.
pub(crate) fn apply_resolved_blow(
    state: &mut CombatantState,
    creature: &Creature,
    amount: i32,
    degree: DegreeOfSuccess,
) -> Result<(), HpError> {
    if matches!(
        degree,
        DegreeOfSuccess::Failure | DegreeOfSuccess::CriticalFailure
    ) {
        return Err(HpError::NotAHit);
    }
    if amount < 0 {
        return Err(HpError::NegativeAmount);
    }
    if inconsistent_vital(state) {
        return Err(HpError::InconsistentVital);
    }
    if state.vital == Vital::Dead {
        return Ok(());
    }
    if massive(amount, creature.hit_points) {
        // Judged on the blow, before temporary Hit Points are spent.
        die(state);
        return Ok(());
    }
    let before = state.vital;
    spend(state, amount);
    if amount == 0 {
        return Ok(());
    }
    if !creature.uses_dying_rules {
        if state.hit_points == 0 {
            die(state);
        }
        return Ok(());
    }
    let critical = degree == DegreeOfSuccess::CriticalSuccess;
    let step = if critical { 2 } else { 1 };
    match before {
        Vital::Alive => {
            if state.hit_points == 0 {
                gain_dying(state, step);
            }
        }
        Vital::Unconscious => gain_dying(state, step),
        Vital::Dying(value) => increase_dying(state, value, step),
        Vital::Dead => {}
    }
    Ok(())
}

/// Heals `amount` Hit Points. A non-positive amount, the dead, an inconsistent vital, or an
/// overflowing sum leaves the state unchanged.
///
/// Player Core pg. 410, Healing. Player Core pg. 411, "Losing the Dying Condition".
pub(crate) fn apply_healing(
    state: &mut CombatantState,
    creature: &Creature,
    amount: i32,
) -> Result<(), HpError> {
    if amount <= 0 {
        return Err(HpError::NonPositiveHealing);
    }
    if state.vital == Vital::Dead {
        return Err(HpError::Dead);
    }
    if inconsistent_vital(state) {
        return Err(HpError::InconsistentVital);
    }
    let added = state
        .hit_points
        .checked_add(amount)
        .ok_or(HpError::Overflow)?;
    let maximum = creature.hit_points.max(0);
    let hit_points = if added < 0 {
        0
    } else if added > maximum {
        maximum
    } else {
        added
    };
    let was_dying = matches!(state.vital, Vital::Dying(_));
    let wounded = if was_dying {
        state.wounded.checked_add(1).ok_or(HpError::Overflow)?
    } else {
        state.wounded
    };
    state.hit_points = hit_points;
    state.wounded = wounded;
    state.vital = Vital::Alive;
    state.initiative_move = None;
    if hit_points == maximum {
        state.conditions.retain(|slot| {
            !matches!(
                slot,
                ConditionSlot::PersistentDamage {
                    damage_type: DamageType::Bleed,
                    ..
                }
            )
        });
    }
    Ok(())
}

/// Ticks temporary Hit Points, then rolls a recovery check while the creature is dying.
///
/// `None` means no recovery check was rolled. A second call ticks again. Actions and the
/// reaction are left as the caller set them.
///
/// Player Core pg. 411, Recovery Checks. Player Core pg. 435, "Start Your Turn".
pub(crate) fn start_of_turn(
    state: &mut CombatantState,
    _creature: &Creature,
    dice: &mut impl Dice,
) -> Result<Option<FlatRoll>, HpError> {
    if inconsistent_vital(state) {
        return Err(HpError::InconsistentVital);
    }
    let snapshot = state.clone();
    tick_temporary(state);
    let Vital::Dying(value) = state.vital else {
        return Ok(None);
    };
    let Some(dc) = 10i32.checked_add(value.get()) else {
        *state = snapshot;
        return Err(HpError::Overflow);
    };
    let roll = flat_check(dice, dc);
    if let Err(error) = apply_recovery(state, value, roll.degree) {
        *state = snapshot;
        return Err(error);
    }
    Ok(Some(roll))
}

/// Grants temporary Hit Points. A strictly greater amount replaces the pool and its duration.
///
/// Player Core pg. 410, Temporary Hit Points. Ruling R-06.
pub(crate) fn grant_temporary_hit_points(
    state: &mut CombatantState,
    amount: i32,
    rounds: u32,
) -> Result<(), HpError> {
    if inconsistent_vital(state) {
        return Err(HpError::InconsistentVital);
    }
    if state.vital == Vital::Dead {
        return Err(HpError::Dead);
    }
    if amount < 0 {
        return Err(HpError::NegativeTemporaryHitPoints);
    }
    if amount == 0 {
        return Ok(());
    }
    if rounds == 0 {
        return Err(HpError::ZeroTemporaryDuration);
    }
    if amount > state.temp_hit_points {
        state.temp_hit_points = amount;
        state.temp_rounds = rounds;
    }
    Ok(())
}

/// Returns the initiative report and clears it.
///
/// Player Core pg. 410, "Getting Knocked Out".
pub(crate) fn take_initiative_move(state: &mut CombatantState) -> Option<InitiativeMove> {
    state.initiative_move.take()
}

/// Reads which side is still fighting. The roster is not written.
///
/// A side is present when it has a combatant, and defeated when none of those combatants is
/// alive. Fewer than two present sides is [`EncounterOutcome::Ongoing`].
///
/// Player Core pg. 435, "End the Encounter".
pub(crate) fn encounter_outcome(roster: &[RosterEntry<'_>]) -> EncounterOutcome {
    let mut pc_present = false;
    let mut pc_alive = false;
    let mut enemy_present = false;
    let mut enemy_alive = false;
    for entry in roster {
        let (present, alive) = match entry.side {
            Side::Pc => (&mut pc_present, &mut pc_alive),
            Side::Enemy => (&mut enemy_present, &mut enemy_alive),
        };
        *present = true;
        if entry.state.vital == Vital::Alive {
            *alive = true;
        }
    }
    let sides = i32::from(pc_present) + i32::from(enemy_present);
    if sides < 2 {
        return EncounterOutcome::Ongoing;
    }
    match (pc_alive, enemy_alive) {
        (true, false) => EncounterOutcome::Victory { side: Side::Pc },
        (false, true) => EncounterOutcome::Victory { side: Side::Enemy },
        (false, false) => EncounterOutcome::MutualDefeat,
        (true, true) => EncounterOutcome::Ongoing,
    }
}

fn massive(amount: i32, maximum: i32) -> bool {
    let Some(threshold) = i64::from(maximum).checked_mul(2) else {
        return false;
    };
    i64::from(amount) >= threshold
}

fn die(state: &mut CombatantState) {
    state.hit_points = 0;
    state.temp_hit_points = 0;
    state.temp_rounds = 0;
    state.vital = Vital::Dead;
    state.initiative_move = None;
    // Death clears every condition slot. add-conditions-with-durations uses this
    // same full clear, so do not narrow it to persistent damage.
    state.conditions.clear();
}

fn spend(state: &mut CombatantState, amount: i32) {
    if amount <= 0 {
        return;
    }
    let mut rest = amount;
    if state.temp_hit_points > 0 {
        match state.temp_hit_points.checked_sub(rest) {
            Some(left) if left >= 0 => {
                state.temp_hit_points = left;
                rest = 0;
            }
            _ => {
                rest = rest.checked_sub(state.temp_hit_points).unwrap_or(0);
                state.temp_hit_points = 0;
            }
        }
        if state.temp_hit_points == 0 {
            state.temp_rounds = 0;
        }
    }
    if rest <= 0 {
        return;
    }
    state.hit_points = match state.hit_points.checked_sub(rest) {
        Some(next) if next > 0 => next,
        _ => 0,
    };
}

fn gain_dying(state: &mut CombatantState, base: i32) {
    let Some(sum) = base.checked_add(state.wounded) else {
        die(state);
        return;
    };
    match DyingValue::new(sum) {
        Ok(value) => {
            state.hit_points = 0;
            state.vital = Vital::Dying(value);
            state.initiative_move = Some(InitiativeMove::Pending);
        }
        Err(crate::combatant::DyingValueError::OutOfRange) => die(state),
    }
}

fn increase_dying(state: &mut CombatantState, current: DyingValue, delta: i32) {
    let Some(sum) = current.get().checked_add(delta) else {
        die(state);
        return;
    };
    match DyingValue::new(sum) {
        Ok(value) => {
            state.hit_points = 0;
            state.vital = Vital::Dying(value);
        }
        Err(crate::combatant::DyingValueError::OutOfRange) => die(state),
    }
}

fn tick_temporary(state: &mut CombatantState) {
    let Some(next) = state.temp_rounds.checked_sub(1) else {
        return;
    };
    state.temp_rounds = next;
    if next == 0 {
        state.temp_hit_points = 0;
    }
}

fn apply_recovery(
    state: &mut CombatantState,
    value: DyingValue,
    degree: DegreeOfSuccess,
) -> Result<(), HpError> {
    let delta = match degree {
        DegreeOfSuccess::CriticalSuccess => -2,
        DegreeOfSuccess::Success => -1,
        DegreeOfSuccess::Failure => 1,
        DegreeOfSuccess::CriticalFailure => 2,
    };
    let next = value.get().checked_add(delta).ok_or(HpError::Overflow)?;
    if next <= 0 {
        let wounded = state.wounded.checked_add(1).ok_or(HpError::Overflow)?;
        state.wounded = wounded;
        state.hit_points = 0;
        state.vital = Vital::Unconscious;
        return Ok(());
    }
    if next >= 4 {
        die(state);
        return Ok(());
    }
    match DyingValue::new(next) {
        Ok(dying) => {
            state.vital = Vital::Dying(dying);
            Ok(())
        }
        Err(crate::combatant::DyingValueError::OutOfRange) => Err(HpError::Overflow),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::DegreeOfSuccess;
    use crate::combatant::{
        BuildParts, CharacterBuild, CombatantId, CombatantState, Creature, DieCount, DyingValue,
        HpError, MeleeStrike, ReachFeet, Vital, apply_dice_damage, apply_healing, damage_dealt,
        derive, new_combatant_state,
    };
    use crate::dice::{Faces, ScriptedDice};
    use crate::encounter::Side;
    use crate::modifiers::{
        ConditionId, Modifier, ModifierTotal, PenaltyType, SourceId, Statistic, ac_modifiers,
        perception_modifiers, reflex_modifiers, total,
    };

    fn faces(count: u8) -> Faces {
        Faces::new(count).expect("faces")
    }

    fn creature(max_hp: i32, rules: bool) -> Creature {
        let parts = BuildParts {
            hit_points: max_hp,
            armor_class: 11,
            perception: 0,
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
            shield: None,
            stealth_dc: None,
            aid_modifier: None,
            aid_rank: None,
            uses_dying_rules: Some(rules),
            saves: None,
            skills: None,
            defenses: crate::damage::DamageDefenses::empty(),
        };
        derive(&CharacterBuild::try_new(parts).expect("build"))
    }

    fn opened(max_hp: i32, rules: bool) -> (Creature, CombatantState) {
        let creature = creature(max_hp, rules);
        let state = new_combatant_state(CombatantId(1), &creature);
        (creature, state)
    }

    fn dying(value: i32) -> DyingValue {
        DyingValue::new(value).expect("dying value")
    }

    fn lay(state: &mut CombatantState, vital: Vital, hp: i32, wounded: i32) {
        state.vital = vital;
        state.hit_points = hp;
        state.wounded = wounded;
    }

    fn pool(state: &mut CombatantState, amount: i32, rounds: u32) {
        state.temp_hit_points = amount;
        state.temp_rounds = rounds;
    }

    fn hit(
        state: &mut CombatantState,
        creature: &Creature,
        face: u8,
        roll: u8,
        bonus: i32,
        modifiers: &[Modifier],
        critical: bool,
    ) -> Result<(), HpError> {
        apply_dice_damage(
            state,
            creature,
            &mut ScriptedDice::new([roll]),
            faces(face),
            bonus,
            modifiers,
            critical,
        )
    }

    fn recover(
        state: &mut CombatantState,
        creature: &Creature,
        natural: u8,
    ) -> Result<Option<crate::check::FlatRoll>, HpError> {
        start_of_turn(state, creature, &mut ScriptedDice::new([natural]))
    }

    fn entry<'a>(id: u32, side: Side, state: &'a CombatantState) -> RosterEntry<'a> {
        RosterEntry {
            id: CombatantId(id),
            side,
            state,
        }
    }

    fn damage_penalty(value: i32) -> Modifier {
        Modifier::penalty(
            PenaltyType::Untyped,
            value,
            SourceId(1),
            1,
            None,
            Statistic::Damage,
        )
        .expect("penalty")
    }

    fn downed(vital: Vital) -> bool {
        matches!(vital, Vital::Dying(_) | Vital::Unconscious)
    }

    fn folded(modifiers: &[Modifier], statistic: Statistic) -> ModifierTotal {
        let stacked = total(modifiers).expect("total");
        assert!(
            stacked
                .applied()
                .iter()
                .all(|modifier| modifier.statistic() == statistic)
        );
        stacked
    }

    fn unconscious_totals(vital: Vital) -> (ModifierTotal, ModifierTotal, ModifierTotal) {
        let unconscious = downed(vital);
        (
            folded(
                &ac_modifiers(None, unconscious, unconscious).expect("armor class"),
                Statistic::ArmorClass,
            ),
            folded(
                &perception_modifiers(unconscious).expect("perception"),
                Statistic::Perception,
            ),
            folded(
                &reflex_modifiers(unconscious).expect("reflex"),
                Statistic::Reflex,
            ),
        )
    }

    fn has_unconscious(total: &ModifierTotal) -> bool {
        total
            .applied()
            .iter()
            .any(|modifier| modifier.condition_id() == Some(ConditionId::UNCONSCIOUS))
    }

    /// Player Core pg. 410, "Getting Knocked Out". DEV-29.
    #[test]
    fn a_non_critical_blow_to_0_sets_dying_1() {
        let (creature, mut state) = opened(9, true);
        let before = creature.clone();
        hit(&mut state, &creature, 8, 8, 1, &[], false).expect("blow");
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.vital, Vital::Dying(dying(1)));
        assert_eq!(state.wounded, 0);
        assert_ne!(state.vital, Vital::Dead);
        assert_eq!(creature, before);
    }

    /// Player Core pg. 410, "Getting Knocked Out". DEV-29.
    #[test]
    fn a_critical_blow_to_0_sets_dying_2() {
        let (creature, mut state) = opened(16, true);
        hit(&mut state, &creature, 8, 8, 0, &[], true).expect("blow");
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.vital, Vital::Dying(dying(2)));
        assert_eq!(state.wounded, 0);
        assert_ne!(state.vital, Vital::Dead);
    }

    /// Player Core pg. 410, "Getting Knocked Out". DEV-29.
    #[test]
    fn the_dropping_blow_does_not_also_increase_dying() {
        let (creature, mut state) = opened(16, true);
        hit(&mut state, &creature, 8, 8, 0, &[], true).expect("blow");
        assert_eq!(state.vital, Vital::Dying(dying(2)));
        assert_ne!(state.vital, Vital::Dead);
    }

    /// Player Core pg. 410, "Getting Knocked Out". DEV-29.
    #[test]
    fn damage_above_0_does_not_start_dying() {
        let (creature, mut state) = opened(9, true);
        hit(&mut state, &creature, 8, 1, 1, &[], false).expect("blow");
        assert_eq!(state.hit_points, 7);
        assert_eq!(state.vital, Vital::Alive);
        assert!(!matches!(state.vital, Vital::Dying(_)));
    }

    /// Player Core pg. 410, "Getting Knocked Out". DEV-29.
    #[test]
    fn damage_past_0_sets_dying_1() {
        let (creature, mut state) = opened(10, true);
        state.hit_points = 3;
        hit(&mut state, &creature, 8, 7, 8, &[], false).expect("blow");
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.vital, Vital::Dying(dying(1)));
        assert_eq!(state.wounded, 0);
    }

    /// Player Core pg. 410, "Getting Knocked Out".
    #[test]
    fn an_enemy_at_0_hit_points_does_not_gain_wounded() {
        let (creature, mut state) = opened(9, false);
        hit(&mut state, &creature, 8, 8, 1, &[], false).expect("blow");
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.vital, Vital::Dead);
        assert_eq!(state.wounded, 0);
        assert!(!matches!(state.vital, Vital::Dying(_)));
    }

    /// Player Core pg. 446, Unconscious. Player Core pg. 411, "Losing the Dying Condition".
    #[test]
    fn positive_healing_wakes_unconscious_without_changing_wounded() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Unconscious, 0, 1);
        apply_healing(&mut state, &creature, 4).expect("heal");
        assert_eq!(state.vital, Vital::Alive);
        assert_eq!(state.hit_points, 4);
        assert_eq!(state.wounded, 1);
    }

    /// Player Core pg. 410, "Getting Knocked Out".
    #[test]
    fn dying_at_nonzero_hit_points_is_rejected() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 4, 0);
        let mut dice = ScriptedDice::new([1]);
        let result = apply_dice_damage(&mut state, &creature, &mut dice, faces(8), 0, &[], false);
        assert_eq!(result, Err(HpError::InconsistentVital));
        assert_eq!(dice.remaining(), 1);
        assert_eq!(state.vital, Vital::Dying(dying(1)));
        assert_eq!(state.hit_points, 4);
        let mut recovery = ScriptedDice::new([20]);
        assert_eq!(
            start_of_turn(&mut state, &creature, &mut recovery),
            Err(HpError::InconsistentVital)
        );
        assert_eq!(recovery.remaining(), 1);
        assert_eq!(state.vital, Vital::Dying(dying(1)));
        assert_eq!(state.hit_points, 4);
    }

    /// Player Core pg. 410, "Getting Knocked Out".
    #[test]
    fn unconscious_at_nonzero_hit_points_is_rejected() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Unconscious, 4, 0);
        let mut dice = ScriptedDice::new([1]);
        let result = apply_dice_damage(&mut state, &creature, &mut dice, faces(8), 0, &[], false);
        assert_eq!(result, Err(HpError::InconsistentVital));
        assert_eq!(dice.remaining(), 1);
        assert_eq!(state.vital, Vital::Unconscious);
        assert_eq!(state.hit_points, 4);
    }

    /// Player Core pg. 410, "Getting Knocked Out".
    #[test]
    fn alive_at_0_hit_points_is_rejected() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Alive, 0, 0);
        let mut dice = ScriptedDice::new([1]);
        let result = apply_dice_damage(&mut state, &creature, &mut dice, faces(8), 0, &[], false);
        assert_eq!(result, Err(HpError::InconsistentVital));
        assert_eq!(dice.remaining(), 1);
        assert_eq!(state.vital, Vital::Alive);
        assert_eq!(state.hit_points, 0);
    }

    /// Player Core pg. 446, Unconscious.
    #[test]
    fn a_dying_creature_takes_minus_4_on_three_statistics() {
        let (_creature, state) = opened(9, true);
        let mut dying_state = state;
        lay(&mut dying_state, Vital::Dying(dying(1)), 0, 0);
        let (armor_class, perception, reflex) = unconscious_totals(dying_state.vital);
        for total in [&armor_class, &perception, &reflex] {
            assert!(total.applied().iter().any(|modifier| {
                modifier.penalty_type() == Some(PenaltyType::Status) && modifier.value() == -4
            }));
        }
        assert_eq!(armor_class.get(Statistic::ArmorClass), -6);
        assert_eq!(perception.get(Statistic::Perception), -4);
        assert_eq!(reflex.get(Statistic::Reflex), -4);
        assert_eq!(armor_class.get(Statistic::Perception), 0);
        assert_eq!(reflex.get(Statistic::ArmorClass), 0);
    }

    /// Player Core pg. 446, Unconscious.
    #[test]
    fn a_conscious_creature_takes_no_unconscious_penalty() {
        let (armor_class, perception, reflex) = unconscious_totals(Vital::Alive);
        assert!(!has_unconscious(&armor_class));
        assert!(!has_unconscious(&perception));
        assert!(!has_unconscious(&reflex));
    }

    /// Player Core pg. 446, Unconscious.
    #[test]
    fn a_dead_creature_takes_no_unconscious_penalty() {
        let (armor_class, perception, reflex) = unconscious_totals(Vital::Dead);
        assert!(!has_unconscious(&armor_class));
        assert!(!has_unconscious(&perception));
        assert!(!has_unconscious(&reflex));
    }

    /// Player Core pg. 446, Unconscious.
    #[test]
    fn waking_removes_the_unconscious_penalty() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Unconscious, 0, 1);
        apply_healing(&mut state, &creature, 4).expect("heal");
        let (armor_class, perception, reflex) = unconscious_totals(state.vital);
        assert_eq!(state.vital, Vital::Alive);
        assert_eq!(state.hit_points, 4);
        assert_eq!(state.wounded, 1);
        assert!(!has_unconscious(&armor_class));
        assert!(!has_unconscious(&perception));
        assert!(!has_unconscious(&reflex));
    }

    /// Player Core pg. 446, Unconscious. DEV-31.
    #[test]
    fn unconscious_adds_off_guard() {
        for vital in [Vital::Dying(dying(1)), Vital::Unconscious] {
            let (creature, mut state) = opened(9, true);
            lay(&mut state, vital, 0, 0);
            let (armor_class, perception, reflex) = unconscious_totals(vital);
            assert_eq!(armor_class.get(Statistic::ArmorClass), -6);
            assert!(armor_class.applied().iter().any(|modifier| {
                modifier.penalty_type() == Some(PenaltyType::Status) && modifier.value() == -4
            }));
            assert!(armor_class.applied().iter().any(|modifier| {
                modifier.penalty_type() == Some(PenaltyType::Circumstance)
                    && modifier.value() == -2
                    && modifier.condition_id() == Some(ConditionId(1))
            }));
            assert_eq!(perception.get(Statistic::Perception), -4);
            assert_eq!(reflex.get(Statistic::Reflex), -4);
            assert!(
                perception
                    .applied()
                    .iter()
                    .all(|modifier| { modifier.penalty_type() != Some(PenaltyType::Circumstance) })
            );
            assert!(
                reflex
                    .applied()
                    .iter()
                    .all(|modifier| { modifier.penalty_type() != Some(PenaltyType::Circumstance) })
            );
            assert!(state.conditions.is_empty());
            assert_eq!(creature.hit_points, 9);
        }
    }

    /// Player Core pg. 410, "Getting Knocked Out". Ruling R-08.
    #[test]
    fn wounded_1_and_1_damage_sets_dying_2() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Unconscious, 0, 1);
        hit(&mut state, &creature, 8, 1, 0, &[], false).expect("blow");
        assert_eq!(state.vital, Vital::Dying(dying(2)));
        assert_eq!(state.wounded, 1);
        assert_eq!(state.hit_points, 0);
        assert_eq!(
            take_initiative_move(&mut state),
            Some(InitiativeMove::Pending)
        );
    }

    /// Player Core pg. 411, Recovery Checks. Player Core pg. 400, Flat Checks.
    #[test]
    fn recovery_critical_success_reduces_dying_by_2() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(3)), 0, 0);
        let roll = recover(&mut state, &creature, 20)
            .expect("recovery")
            .expect("rolled");
        assert_eq!(roll.dc, 13);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(state.vital, Vital::Dying(dying(1)));
    }

    /// Player Core pg. 411, Recovery Checks. Player Core pg. 400, Flat Checks.
    #[test]
    fn recovery_success_reduces_dying_by_1() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(2)), 0, 0);
        let roll = recover(&mut state, &creature, 12)
            .expect("recovery")
            .expect("rolled");
        assert_eq!(roll.dc, 12);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(state.vital, Vital::Dying(dying(1)));
    }

    /// Player Core pg. 411, Recovery Checks. Player Core pg. 400, Flat Checks.
    #[test]
    fn recovery_failure_increases_dying_by_1() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        let roll = recover(&mut state, &creature, 2)
            .expect("recovery")
            .expect("rolled");
        assert_eq!(roll.dc, 11);
        assert_eq!(roll.degree, DegreeOfSuccess::Failure);
        assert_eq!(state.vital, Vital::Dying(dying(2)));
    }

    /// Player Core pg. 411, Recovery Checks. Player Core pg. 400, Flat Checks.
    #[test]
    fn recovery_critical_failure_increases_dying_by_2() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        let roll = recover(&mut state, &creature, 1)
            .expect("recovery")
            .expect("rolled");
        assert_eq!(roll.dc, 11);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalFailure);
        assert_eq!(state.vital, Vital::Dying(dying(3)));
    }

    /// Player Core pg. 411, Recovery Checks.
    #[test]
    fn a_conscious_creature_does_not_roll_recovery() {
        let (creature, mut state) = opened(9, true);
        let mut dice = ScriptedDice::new([20]);
        assert_eq!(
            start_of_turn(&mut state, &creature, &mut dice).expect("turn"),
            None
        );
        assert_eq!(dice.remaining(), 1);
        assert_eq!(state.vital, Vital::Alive);
    }

    /// Player Core pg. 411, Recovery Checks.
    #[test]
    fn a_dead_creature_does_not_roll_recovery() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dead, 0, 0);
        let mut dice = ScriptedDice::new([20]);
        assert_eq!(
            start_of_turn(&mut state, &creature, &mut dice).expect("turn"),
            None
        );
        assert_eq!(dice.remaining(), 1);
        assert_eq!(state.vital, Vital::Dead);
    }

    /// Player Core pg. 411, Recovery Checks.
    #[test]
    fn the_same_recovery_twice_matches() {
        let run = || {
            let (creature, mut state) = opened(9, true);
            lay(&mut state, Vital::Dying(dying(2)), 0, 0);
            let mut dice = ScriptedDice::new([12]);
            let result = start_of_turn(&mut state, &creature, &mut dice).expect("recovery");
            (state, dice.remaining(), result)
        };
        assert_eq!(run(), run());
    }

    /// Player Core pg. 411, "Losing the Dying Condition".
    #[test]
    fn recovery_to_0_stays_unconscious_at_0_hit_points() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        pool(&mut state, 4, 2);
        let roll = recover(&mut state, &creature, 11)
            .expect("recovery")
            .expect("rolled");
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(state.vital, Vital::Unconscious);
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.wounded, 1);
        assert_eq!(state.temp_hit_points, 4);
        assert_eq!(state.temp_rounds, 1);
    }

    /// Player Core pg. 411, "Losing the Dying Condition".
    #[test]
    fn a_recovery_that_stays_dying_does_not_change_hit_points() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(2)), 0, 0);
        recover(&mut state, &creature, 12).expect("recovery");
        assert_eq!(state.vital, Vital::Dying(dying(1)));
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.wounded, 0);
        assert_ne!(state.vital, Vital::Alive);
    }

    /// Player Core pg. 411, "Losing the Dying Condition".
    #[test]
    fn a_natural_20_at_dying_1_stays_unconscious() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        let roll = recover(&mut state, &creature, 20)
            .expect("recovery")
            .expect("rolled");
        assert_eq!(roll.dc, 11);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalSuccess);
        assert_eq!(state.vital, Vital::Unconscious);
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.wounded, 1);
    }

    /// Player Core pg. 411, "Losing the Dying Condition".
    #[test]
    fn a_natural_19_at_dying_3_reduces_dying_to_2() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(3)), 0, 0);
        let roll = recover(&mut state, &creature, 19)
            .expect("recovery")
            .expect("rolled");
        assert_eq!(roll.dc, 13);
        assert_eq!(roll.degree, DegreeOfSuccess::Success);
        assert_eq!(state.vital, Vital::Dying(dying(2)));
    }

    /// Player Core pg. 411, "Losing the Dying Condition".
    #[test]
    fn a_natural_10_at_dying_1_increases_dying_to_2() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        let roll = recover(&mut state, &creature, 10)
            .expect("recovery")
            .expect("rolled");
        assert_eq!(roll.dc, 11);
        assert_eq!(roll.degree, DegreeOfSuccess::Failure);
        assert_eq!(state.vital, Vital::Dying(dying(2)));
        assert_eq!(take_initiative_move(&mut state), None);
    }

    /// Player Core pg. 411, "Losing the Dying Condition". Player Core pg. 447, Wounded.
    #[test]
    fn losing_dying_increases_wounded_by_1() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        recover(&mut state, &creature, 11).expect("recovery");
        assert_eq!(state.wounded, 1);
        assert_eq!(state.vital, Vital::Unconscious);
    }

    /// Player Core pg. 447, Wounded.
    #[test]
    fn gaining_dying_does_not_increase_wounded() {
        let (creature, mut state) = opened(9, true);
        hit(&mut state, &creature, 8, 8, 1, &[], false).expect("blow");
        assert_eq!(state.vital, Vital::Dying(dying(1)));
        assert_eq!(state.wounded, 0);
    }

    /// Player Core pg. 447, Wounded.
    #[test]
    fn damage_while_dying_does_not_increase_wounded() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 2);
        hit(&mut state, &creature, 8, 1, 0, &[], false).expect("blow");
        assert_eq!(state.vital, Vital::Dying(dying(2)));
        assert_eq!(state.wounded, 2);
    }

    /// Player Core pg. 447, Wounded.
    #[test]
    fn death_does_not_increase_wounded() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(3)), 0, 1);
        hit(&mut state, &creature, 8, 1, 0, &[], false).expect("blow");
        assert_eq!(state.vital, Vital::Dead);
        assert_eq!(state.wounded, 1);
    }

    /// Player Core pg. 447, Wounded.
    #[test]
    fn full_hit_points_do_not_clear_wounded() {
        let (creature, mut state) = opened(9, true);
        state.wounded = 2;
        apply_healing(&mut state, &creature, 4).expect("heal");
        assert_eq!(state.hit_points, 9);
        assert_eq!(state.wounded, 2);
    }

    /// Player Core pg. 411, "Losing the Dying Condition". Player Core pg. 447, Wounded.
    #[test]
    fn maximum_wounded_dies_on_the_knockout() {
        let (creature, mut state) = opened(9, true);
        state.wounded = 2_147_483_647;
        hit(&mut state, &creature, 8, 8, 1, &[], false).expect("blow");
        assert_eq!(state.vital, Vital::Dead);
        assert!(!matches!(state.vital, Vital::Dying(_)));
        assert_eq!(state.wounded, 2_147_483_647);
    }

    /// Player Core pg. 411, "Losing the Dying Condition". Player Core pg. 447, Wounded.
    #[test]
    fn healing_at_maximum_wounded_returns_overflow() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 2_147_483_647);
        assert_eq!(
            apply_healing(&mut state, &creature, 1),
            Err(HpError::Overflow)
        );
        assert_eq!(state.vital, Vital::Dying(dying(1)));
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.wounded, 2_147_483_647);
    }

    /// Player Core pg. 411, "Losing the Dying Condition". Player Core pg. 447, Wounded.
    #[test]
    fn recovery_at_maximum_wounded_returns_overflow() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 2_147_483_647);
        pool(&mut state, 4, 2);
        let before = state.clone();
        let mut dice = ScriptedDice::new([11]);
        assert_eq!(
            start_of_turn(&mut state, &creature, &mut dice),
            Err(HpError::Overflow)
        );
        assert_eq!(dice.remaining(), 0);
        assert_eq!(state, before);
    }

    /// Player Core pg. 410, "Getting Knocked Out". Player Core pg. 447, Wounded.
    #[test]
    fn wounded_increases_the_next_dying_value() {
        let (creature, mut state) = opened(9, true);
        state.wounded = 1;
        hit(&mut state, &creature, 8, 8, 1, &[], false).expect("blow");
        assert_eq!(state.vital, Vital::Dying(dying(2)));
        assert_eq!(state.wounded, 1);
    }

    /// Player Core pg. 410, "Getting Knocked Out". Player Core pg. 447, Wounded.
    #[test]
    fn a_critical_blow_adds_wounded_to_2() {
        let (creature, mut state) = opened(16, true);
        state.wounded = 1;
        hit(&mut state, &creature, 8, 8, 0, &[], true).expect("blow");
        assert_eq!(state.vital, Vital::Dying(dying(3)));
        assert_eq!(state.wounded, 1);
    }

    /// Player Core pg. 411, "Taking Damage while Dying".
    #[test]
    fn damage_while_dying_increases_dying_by_1() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        hit(&mut state, &creature, 8, 1, 0, &[], false).expect("blow");
        assert_eq!(state.vital, Vital::Dying(dying(2)));
        assert_eq!(state.hit_points, 0);
    }

    /// Player Core pg. 411, "Taking Damage while Dying".
    #[test]
    fn a_critical_blow_while_dying_increases_dying_by_2() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        hit(&mut state, &creature, 8, 1, 0, &[], true).expect("blow");
        assert_eq!(state.vital, Vital::Dying(dying(3)));
    }

    /// Player Core pg. 411, "Taking Damage while Dying".
    #[test]
    fn a_resolved_blow_of_0_does_not_increase_dying() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        let dice = ScriptedDice::new([8]);
        apply_resolved_blow(&mut state, &creature, 0, DegreeOfSuccess::Success).expect("zero");
        assert_eq!(dice.remaining(), 1);
        assert_eq!(state.vital, Vital::Dying(dying(1)));
    }

    /// Player Core pg. 406, "Damage Rolls". Rules 2302.
    #[test]
    fn a_damage_penalty_floors_to_1_and_increases_dying() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        let penalty = damage_penalty(-5);
        assert_eq!(damage_dealt(1, &[penalty], false).expect("floor"), 1);
        hit(&mut state, &creature, 4, 1, 0, &[penalty], false).expect("blow");
        assert_eq!(state.vital, Vital::Dying(dying(2)));
    }

    /// Player Core pg. 406, "Damage Rolls".
    #[test]
    fn a_baked_negative_bonus_deals_0_while_dying() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        assert_eq!(damage_dealt(-4, &[], false).expect("clamp"), 0);
        hit(&mut state, &creature, 4, 1, -5, &[], false).expect("blow");
        assert_eq!(state.vital, Vital::Dying(dying(1)));
    }

    /// Player Core pg. 411, "Taking Damage while Dying". Ruling R-10.
    #[test]
    fn temporary_hit_points_do_not_block_the_dying_increase() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        pool(&mut state, 5, 3);
        hit(&mut state, &creature, 4, 3, 0, &[], false).expect("blow");
        assert_eq!(state.temp_hit_points, 2);
        assert_eq!(state.temp_rounds, 3);
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.vital, Vital::Dying(dying(2)));
    }

    /// Player Core pg. 410, "Getting Knocked Out".
    #[test]
    fn a_blow_absorbed_down_to_1_hit_point_does_not_start_dying() {
        let (creature, mut state) = opened(9, true);
        state.hit_points = 3;
        pool(&mut state, 5, 3);
        hit(&mut state, &creature, 8, 7, 0, &[], false).expect("blow");
        assert_eq!(state.temp_hit_points, 0);
        assert_eq!(state.temp_rounds, 0);
        assert_eq!(state.hit_points, 1);
        assert_eq!(state.vital, Vital::Alive);
        assert!(!matches!(state.vital, Vital::Dying(_)));
    }

    /// Player Core pg. 411, Dying. Player Core pg. 443, Dying. Ruling R-10. DEV-28.
    #[test]
    fn dying_3_and_one_damage_kills() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(3)), 0, 0);
        pool(&mut state, 4, 2);
        hit(&mut state, &creature, 8, 1, 0, &[], false).expect("blow");
        assert_eq!(state.vital, Vital::Dead);
        assert!(!matches!(state.vital, Vital::Dying(_)));
        assert_eq!(state.wounded, 0);
        assert_eq!(state.temp_hit_points, 0);
        assert_eq!(state.temp_rounds, 0);
    }

    /// Player Core pg. 411, Dying.
    #[test]
    fn a_critical_blow_at_dying_3_kills() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(3)), 0, 0);
        hit(&mut state, &creature, 8, 1, 0, &[], true).expect("blow");
        assert_eq!(state.vital, Vital::Dead);
    }

    /// Player Core pg. 411, Dying. Ruling R-10.
    #[test]
    fn wounded_3_dies_on_the_knockout() {
        let (creature, mut state) = opened(9, true);
        state.hit_points = 5;
        state.wounded = 3;
        pool(&mut state, 4, 2);
        hit(&mut state, &creature, 8, 8, 1, &[], false).expect("blow");
        assert_eq!(state.vital, Vital::Dead);
        assert!(!matches!(state.vital, Vital::Dying(_)));
        assert_eq!(state.wounded, 3);
        assert_eq!(state.temp_hit_points, 0);
        assert_eq!(state.temp_rounds, 0);
    }

    /// Player Core pg. 411, Dying. DEV-28.
    #[test]
    fn a_recovery_critical_failure_at_dying_2_kills() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(2)), 0, 0);
        pool(&mut state, 4, 2);
        let roll = recover(&mut state, &creature, 1)
            .expect("recovery")
            .expect("rolled");
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalFailure);
        assert_eq!(state.vital, Vital::Dead);
        assert_eq!(state.temp_hit_points, 0);
        assert_eq!(state.temp_rounds, 0);
    }

    /// Player Core pg. 411, Dying.
    #[test]
    fn a_natural_2_at_dying_3_kills() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(3)), 0, 0);
        let roll = recover(&mut state, &creature, 2)
            .expect("recovery")
            .expect("rolled");
        assert_eq!(roll.dc, 13);
        assert_eq!(roll.degree, DegreeOfSuccess::CriticalFailure);
        assert_eq!(state.vital, Vital::Dead);
    }

    /// Player Core pg. 411, Dying.
    #[test]
    fn a_natural_12_at_dying_3_kills() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(3)), 0, 0);
        let roll = recover(&mut state, &creature, 12)
            .expect("recovery")
            .expect("rolled");
        assert_eq!(roll.dc, 13);
        assert_eq!(roll.degree, DegreeOfSuccess::Failure);
        assert_eq!(state.vital, Vital::Dead);
    }

    /// Player Core pg. 411, Dying.
    #[test]
    fn a_dead_creature_is_not_conscious() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(3)), 0, 0);
        hit(&mut state, &creature, 8, 1, 0, &[], false).expect("blow");
        assert_eq!(state.vital, Vital::Dead);
        assert_ne!(state.vital, Vital::Alive);
    }

    /// Player Core pg. 412, Massive Damage.
    #[test]
    fn damage_equal_to_twice_maximum_hit_points_kills() {
        let (creature, mut state) = opened(10, true);
        hit(&mut state, &creature, 8, 8, 12, &[], false).expect("blow");
        assert_eq!(state.vital, Vital::Dead);
        assert!(!matches!(state.vital, Vital::Dying(_)));
        assert_eq!(state.wounded, 0);
        assert_eq!(state.hit_points, 0);
    }

    /// Player Core pg. 412, Massive Damage.
    #[test]
    fn damage_one_below_twice_maximum_does_not_kill() {
        let (creature, mut state) = opened(10, true);
        hit(&mut state, &creature, 8, 8, 11, &[], false).expect("blow");
        assert_eq!(state.vital, Vital::Dying(dying(1)));
        assert_ne!(state.vital, Vital::Dead);
    }

    /// Player Core pg. 412, Massive Damage. Ruling R-06.
    #[test]
    fn massive_damage_ignores_temporary_hit_points() {
        let (creature, mut state) = opened(10, true);
        pool(&mut state, 30, 4);
        hit(&mut state, &creature, 8, 8, 12, &[], false).expect("blow");
        assert_eq!(state.vital, Vital::Dead);
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.temp_hit_points, 0);
        assert_eq!(state.temp_rounds, 0);
    }

    /// Player Core pg. 412, Massive Damage. Player Core pg. 407, "Doubling and Halving".
    #[test]
    fn a_critical_blow_doubles_before_the_massive_check() {
        let (creature, mut state) = opened(10, true);
        assert_eq!(damage_dealt(10, &[], true).expect("doubled"), 20);
        hit(&mut state, &creature, 8, 8, 2, &[], true).expect("blow");
        assert_eq!(state.vital, Vital::Dead);
    }

    /// Player Core pg. 412, Massive Damage.
    #[test]
    fn the_same_blow_without_doubling_does_not_kill() {
        let (creature, mut state) = opened(10, true);
        assert_eq!(damage_dealt(10, &[], false).expect("plain"), 10);
        hit(&mut state, &creature, 8, 8, 2, &[], false).expect("blow");
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.vital, Vital::Dying(dying(1)));
    }

    /// Player Core pg. 412, Massive Damage.
    #[test]
    fn massive_damage_while_dying_kills_without_another_increase() {
        let (creature, mut state) = opened(10, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        hit(&mut state, &creature, 8, 8, 12, &[], false).expect("blow");
        assert_eq!(state.vital, Vital::Dead);
        assert_ne!(state.vital, Vital::Dying(dying(2)));
    }

    /// Player Core pg. 412, Massive Damage. Player Core pg. 410, Temporary Hit Points.
    #[test]
    fn massive_damage_is_judged_before_temporary_hit_points() {
        let (creature, mut state) = opened(10, false);
        pool(&mut state, 30, 4);
        hit(&mut state, &creature, 8, 8, 12, &[], false).expect("blow");
        assert_eq!(state.vital, Vital::Dead);
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.temp_hit_points, 0);
        assert_eq!(state.temp_rounds, 0);
        assert_ne!(state.vital, Vital::Alive);
        assert_ne!(state.hit_points, 10);
    }

    /// Player Core pg. 411, "Losing the Dying Condition". Player Core pg. 410, Healing.
    #[test]
    fn healing_removes_dying_and_sets_the_healed_hit_points() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        pool(&mut state, 4, 0);
        apply_healing(&mut state, &creature, 4).expect("heal");
        assert_eq!(state.hit_points, 4);
        assert_eq!(state.temp_hit_points, 4);
        assert_eq!(state.wounded, 1);
        assert!(!matches!(state.vital, Vital::Dying(_)));
        assert_eq!(state.vital, Vital::Alive);
    }

    /// Player Core pg. 411, "Losing the Dying Condition".
    #[test]
    fn healing_is_capped_at_maximum_hit_points() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        apply_healing(&mut state, &creature, 100).expect("heal");
        assert_eq!(state.hit_points, 9);
        assert_eq!(state.wounded, 1);
        assert_eq!(state.vital, Vital::Alive);
    }

    /// Player Core pg. 411, "Losing the Dying Condition".
    #[test]
    fn healing_to_the_hit_point_cap_leaves_the_combatant_alive() {
        let (creature, mut state) = opened(2_147_483_647, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        apply_healing(&mut state, &creature, 2_147_483_647).expect("heal");
        assert_eq!(state.hit_points, 2_147_483_647);
        assert_eq!(state.wounded, 1);
        assert_eq!(state.vital, Vital::Alive);
    }

    /// Player Core pg. 410, Healing. Player Core pg. 411, Death.
    #[test]
    fn healing_the_dead_returns_an_error() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dead, 0, 1);
        let before = state.clone();
        assert_eq!(apply_healing(&mut state, &creature, 5), Err(HpError::Dead));
        assert_eq!(state, before);
    }

    /// Player Core pg. 410, Healing.
    #[test]
    fn a_zero_heal_does_not_remove_dying() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        assert_eq!(
            apply_healing(&mut state, &creature, 0),
            Err(HpError::NonPositiveHealing)
        );
        assert_eq!(state.vital, Vital::Dying(dying(1)));
    }

    /// Player Core pg. 410, Healing.
    #[test]
    fn a_negative_heal_does_not_change_hit_points() {
        let (creature, mut state) = opened(9, true);
        assert_eq!(
            apply_healing(&mut state, &creature, -1),
            Err(HpError::NonPositiveHealing)
        );
        assert_eq!(state.hit_points, 9);
    }

    /// Player Core pg. 410, Temporary Hit Points. Ruling R-06.
    #[test]
    fn a_smaller_pool_keeps_the_higher_amount_and_its_duration() {
        let (_creature, mut state) = opened(9, true);
        pool(&mut state, 5, 3);
        grant_temporary_hit_points(&mut state, 3, 10).expect("grant");
        assert_eq!(state.temp_hit_points, 5);
        assert_eq!(state.temp_rounds, 3);
    }

    /// Player Core pg. 410, Temporary Hit Points. Ruling R-06.
    #[test]
    fn a_tied_pool_keeps_the_existing_one() {
        let (_creature, mut state) = opened(9, true);
        pool(&mut state, 5, 2);
        grant_temporary_hit_points(&mut state, 5, 8).expect("grant");
        assert_eq!(state.temp_hit_points, 5);
        assert_eq!(state.temp_rounds, 2);
    }

    /// Player Core pg. 410, Temporary Hit Points. Ruling R-06.
    #[test]
    fn a_larger_pool_replaces_the_amount_and_its_duration() {
        let (_creature, mut state) = opened(9, true);
        pool(&mut state, 5, 2);
        grant_temporary_hit_points(&mut state, 8, 1).expect("grant");
        assert_eq!(state.temp_hit_points, 8);
        assert_eq!(state.temp_rounds, 1);
    }

    /// Player Core pg. 410, Temporary Hit Points. Ruling R-06.
    #[test]
    fn the_same_temporary_grant_twice_matches() {
        let run = || {
            let (_creature, mut state) = opened(9, true);
            pool(&mut state, 5, 3);
            let dice = ScriptedDice::new([8]);
            let result = grant_temporary_hit_points(&mut state, 8, 1);
            (state, dice.remaining(), result)
        };
        assert_eq!(run(), run());
    }

    /// Player Core pg. 410, Temporary Hit Points. Ruling R-06.
    #[test]
    fn seven_damage_removes_5_temporary_hit_points_and_2_hit_points() {
        let (creature, mut state) = opened(9, true);
        pool(&mut state, 5, 3);
        hit(&mut state, &creature, 8, 7, 0, &[], false).expect("blow");
        assert_eq!(state.temp_hit_points, 0);
        assert_eq!(state.temp_rounds, 0);
        assert_eq!(state.hit_points, 7);
    }

    /// Player Core pg. 410, Temporary Hit Points. Ruling R-06.
    #[test]
    fn damage_less_than_temporary_hit_points_leaves_hit_points() {
        let (creature, mut state) = opened(9, true);
        pool(&mut state, 5, 3);
        hit(&mut state, &creature, 4, 3, 0, &[], false).expect("blow");
        assert_eq!(state.temp_hit_points, 2);
        assert_eq!(state.temp_rounds, 3);
        assert_eq!(state.hit_points, 9);
    }

    /// Player Core pg. 410, Temporary Hit Points. Ruling R-06.
    #[test]
    fn damage_equal_to_temporary_hit_points_clears_the_pool() {
        let (creature, mut state) = opened(9, true);
        pool(&mut state, 5, 3);
        hit(&mut state, &creature, 8, 5, 0, &[], false).expect("blow");
        assert_eq!(state.temp_hit_points, 0);
        assert_eq!(state.temp_rounds, 0);
        assert_eq!(state.hit_points, 9);
    }

    /// Player Core pg. 410, Temporary Hit Points. Ruling R-06.
    #[test]
    fn healing_does_not_restore_temporary_hit_points() {
        let (creature, mut state) = opened(9, true);
        state.hit_points = 4;
        pool(&mut state, 2, 1);
        apply_healing(&mut state, &creature, 3).expect("heal");
        assert_eq!(state.hit_points, 7);
        assert_eq!(state.temp_hit_points, 2);
    }

    /// Player Core pg. 410, Temporary Hit Points. Ruling R-06.
    #[test]
    fn temporary_hit_points_do_not_raise_the_maximum() {
        let (creature, mut state) = opened(9, true);
        pool(&mut state, 5, 1);
        apply_healing(&mut state, &creature, 4).expect("heal");
        assert_eq!(state.hit_points, 9);
        assert_eq!(state.temp_hit_points, 5);
    }

    /// Player Core pg. 410, Temporary Hit Points. DEV-32.
    #[test]
    fn unused_temporary_hit_points_vanish_on_expiry() {
        let (creature, mut state) = opened(9, true);
        pool(&mut state, 4, 1);
        let mut dice = ScriptedDice::new([20]);
        assert_eq!(
            start_of_turn(&mut state, &creature, &mut dice).expect("turn"),
            None
        );
        assert_eq!(state.temp_hit_points, 0);
        assert_eq!(state.temp_rounds, 0);
        assert_eq!(state.hit_points, 9);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core pg. 410, Temporary Hit Points. DEV-32.
    #[test]
    fn a_longer_duration_ticks_once() {
        let (creature, mut state) = opened(9, true);
        pool(&mut state, 4, 2);
        assert_eq!(
            start_of_turn(&mut state, &creature, &mut ScriptedDice::new([])).expect("turn"),
            None
        );
        assert_eq!(state.temp_hit_points, 4);
        assert_eq!(state.temp_rounds, 1);
    }

    /// Player Core pg. 410, Temporary Hit Points. DEV-32.
    #[test]
    fn expiry_happens_before_recovery() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        pool(&mut state, 4, 1);
        recover(&mut state, &creature, 11).expect("recovery");
        assert_eq!(state.temp_hit_points, 0);
        assert_eq!(state.temp_rounds, 0);
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.wounded, 1);
        assert_eq!(state.vital, Vital::Unconscious);
    }

    /// Player Core pg. 435, "Start Your Turn".
    #[test]
    fn a_second_start_of_turn_ticks_again() {
        let (creature, mut state) = opened(9, true);
        pool(&mut state, 4, 2);
        let mut dice = ScriptedDice::new([20]);
        let first = start_of_turn(&mut state, &creature, &mut dice).expect("first");
        let after_one = state.clone();
        assert_eq!(first, None);
        assert_eq!(state.temp_rounds, 1);
        assert_eq!(dice.remaining(), 1);
        let second = start_of_turn(&mut state, &creature, &mut dice).expect("second");
        assert_eq!(second, None);
        assert_eq!(state.temp_hit_points, 0);
        assert_eq!(state.temp_rounds, 0);
        assert_ne!(state, after_one);
    }

    /// Player Core pg. 410, Temporary Hit Points.
    #[test]
    fn fresh_temporary_hit_points_are_none() {
        let (_creature, state) = opened(9, true);
        assert_eq!(state.temp_hit_points, 0);
        assert_eq!(state.temp_rounds, 0);
    }

    /// Player Core pg. 410, Temporary Hit Points.
    #[test]
    fn a_zero_grant_does_not_replace_a_pool() {
        let (_creature, mut state) = opened(9, true);
        pool(&mut state, 5, 3);
        grant_temporary_hit_points(&mut state, 0, 9).expect("grant");
        assert_eq!(state.temp_hit_points, 5);
        assert_eq!(state.temp_rounds, 3);
    }

    /// Player Core pg. 410, Temporary Hit Points.
    #[test]
    fn a_grant_after_the_pool_reaches_0_is_stored() {
        let (creature, mut state) = opened(9, true);
        pool(&mut state, 5, 3);
        hit(&mut state, &creature, 8, 5, 0, &[], false).expect("blow");
        assert_eq!(state.temp_hit_points, 0);
        assert_eq!(state.temp_rounds, 0);
        grant_temporary_hit_points(&mut state, 4, 2).expect("grant");
        assert_eq!(state.temp_hit_points, 4);
        assert_eq!(state.temp_rounds, 2);
    }

    /// Player Core pg. 410, Temporary Hit Points.
    #[test]
    fn a_negative_temporary_amount_is_rejected() {
        let (_creature, mut state) = opened(9, true);
        pool(&mut state, 5, 1);
        assert_eq!(
            grant_temporary_hit_points(&mut state, -1, 1),
            Err(HpError::NegativeTemporaryHitPoints)
        );
        assert_eq!(state.temp_hit_points, 5);
    }

    /// Player Core pg. 407, "Doubling and Halving".
    #[test]
    fn a_failure_degree_is_not_a_blow() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        assert_eq!(
            apply_resolved_blow(&mut state, &creature, 1, DegreeOfSuccess::Failure),
            Err(HpError::NotAHit)
        );
        assert_eq!(take_initiative_move(&mut state), None);
        assert_eq!(state.vital, Vital::Dying(dying(1)));
    }

    /// Player Core pg. 407, "Doubling and Halving".
    #[test]
    fn a_critical_failure_degree_is_not_a_blow() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        assert_eq!(
            apply_resolved_blow(&mut state, &creature, 1, DegreeOfSuccess::CriticalFailure),
            Err(HpError::NotAHit)
        );
        assert_eq!(take_initiative_move(&mut state), None);
        assert_eq!(state.vital, Vital::Dying(dying(1)));
    }

    /// Player Core pg. 411, Death.
    #[test]
    fn a_grant_to_the_dead_leaves_the_pool_empty() {
        let (_creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dead, 0, 0);
        assert_eq!(
            grant_temporary_hit_points(&mut state, 5, 2),
            Err(HpError::Dead)
        );
        assert_eq!(state.temp_hit_points, 0);
        assert_eq!(state.temp_rounds, 0);
    }

    /// Player Core pg. 410, Temporary Hit Points.
    #[test]
    fn a_zero_duration_does_not_replace_a_pool() {
        let (_creature, mut state) = opened(9, true);
        pool(&mut state, 5, 3);
        assert_eq!(
            grant_temporary_hit_points(&mut state, 8, 0),
            Err(HpError::ZeroTemporaryDuration)
        );
        assert_eq!(state.temp_hit_points, 5);
        assert_eq!(state.temp_rounds, 3);
    }

    /// Player Core pg. 410, Temporary Hit Points.
    #[test]
    fn granting_temporary_hit_points_does_not_remove_dying() {
        let (_creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        grant_temporary_hit_points(&mut state, 5, 2).expect("grant");
        assert_eq!(state.vital, Vital::Dying(dying(1)));
        assert_eq!(state.wounded, 0);
    }

    /// Player Core pg. 411, Death.
    #[test]
    fn a_blow_against_the_dead_leaves_the_state_unchanged() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dead, 0, 0);
        let mut dice = ScriptedDice::new([8]);
        apply_dice_damage(&mut state, &creature, &mut dice, faces(8), 1, &[], false).expect("blow");
        assert_eq!(dice.remaining(), 0);
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.vital, Vital::Dead);
    }

    /// Player Core pg. 411, Death.
    #[test]
    fn massive_damage_against_the_dead_leaves_the_state_unchanged() {
        let (creature, mut state) = opened(10, true);
        lay(&mut state, Vital::Dead, 0, 0);
        pool(&mut state, 4, 2);
        let before = state.clone();
        let mut dice = ScriptedDice::new([8]);
        apply_dice_damage(&mut state, &creature, &mut dice, faces(8), 12, &[], false)
            .expect("blow");
        assert_eq!(dice.remaining(), 0);
        assert_eq!(state, before);
    }

    /// Player Core pg. 411, Death.
    #[test]
    fn an_overflowing_blow_leaves_a_dying_combatant_unchanged() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        let before = state.clone();
        let mut dice = ScriptedDice::new([1]);
        assert_eq!(
            apply_dice_damage(
                &mut state,
                &creature,
                &mut dice,
                faces(8),
                2_147_483_647,
                &[],
                false,
            ),
            Err(HpError::Overflow)
        );
        assert_eq!(state, before);
    }

    /// Player Core pg. 411, Death. Player Core pg. 412, Massive Damage.
    #[test]
    fn maximum_hit_points_and_a_maximum_blow_are_not_massive() {
        let (creature, mut state) = opened(2_147_483_647, true);
        let dice = ScriptedDice::new([20]);
        apply_resolved_blow(
            &mut state,
            &creature,
            2_147_483_647,
            DegreeOfSuccess::Success,
        )
        .expect("blow");
        assert_eq!(dice.remaining(), 1);
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.vital, Vital::Dying(dying(1)));
    }

    /// Player Core pg. 435, "End the Encounter". DEV-30.
    #[test]
    fn one_side_with_no_conscious_combatant_loses() {
        let creature = creature(9, true);
        let pc = new_combatant_state(CombatantId(1), &creature);
        let mut dead = new_combatant_state(CombatantId(2), &creature);
        let mut dying_enemy = new_combatant_state(CombatantId(3), &creature);
        lay(&mut dead, Vital::Dead, 0, 0);
        lay(&mut dying_enemy, Vital::Dying(dying(1)), 0, 0);
        let roster = [
            entry(1, Side::Pc, &pc),
            entry(2, Side::Enemy, &dead),
            entry(3, Side::Enemy, &dying_enemy),
        ];
        assert_eq!(
            encounter_outcome(&roster),
            EncounterOutcome::Victory { side: Side::Pc }
        );
    }

    /// Player Core pg. 435, "End the Encounter". DEV-30.
    #[test]
    fn a_dying_side_loses_to_an_alive_side() {
        let creature = creature(9, true);
        let mut pc = new_combatant_state(CombatantId(1), &creature);
        let enemy = new_combatant_state(CombatantId(2), &creature);
        lay(&mut pc, Vital::Dying(dying(1)), 0, 0);
        assert_eq!(
            encounter_outcome(&[entry(1, Side::Pc, &pc), entry(2, Side::Enemy, &enemy)]),
            EncounterOutcome::Victory { side: Side::Enemy }
        );
    }

    /// Player Core pg. 435, "End the Encounter". DEV-30.
    #[test]
    fn both_sides_conscious_is_not_victory() {
        let creature = creature(9, true);
        let pc = new_combatant_state(CombatantId(1), &creature);
        let enemy = new_combatant_state(CombatantId(2), &creature);
        assert_eq!(
            encounter_outcome(&[entry(1, Side::Pc, &pc), entry(2, Side::Enemy, &enemy)]),
            EncounterOutcome::Ongoing
        );
    }

    /// Player Core pg. 435, "End the Encounter". DEV-30.
    #[test]
    fn no_conscious_combatant_is_a_mutual_defeat() {
        let creature = creature(9, true);
        let mut pc = new_combatant_state(CombatantId(1), &creature);
        let mut enemy = new_combatant_state(CombatantId(2), &creature);
        lay(&mut pc, Vital::Dead, 0, 0);
        lay(&mut enemy, Vital::Dying(dying(2)), 0, 0);
        assert_eq!(
            encounter_outcome(&[entry(1, Side::Pc, &pc), entry(2, Side::Enemy, &enemy)]),
            EncounterOutcome::MutualDefeat
        );
    }

    /// Player Core pg. 435, "End the Encounter". DEV-30.
    #[test]
    fn a_single_side_still_fighting_is_not_victory() {
        let creature = creature(9, true);
        let pc = new_combatant_state(CombatantId(1), &creature);
        assert_eq!(
            encounter_outcome(&[entry(1, Side::Pc, &pc)]),
            EncounterOutcome::Ongoing
        );
    }

    /// Player Core pg. 435, "End the Encounter". DEV-30.
    #[test]
    fn a_single_wiped_side_is_not_a_mutual_defeat() {
        let creature = creature(9, true);
        let mut pc = new_combatant_state(CombatantId(1), &creature);
        lay(&mut pc, Vital::Dead, 0, 0);
        assert_eq!(
            encounter_outcome(&[entry(1, Side::Pc, &pc)]),
            EncounterOutcome::Ongoing
        );
    }

    /// Player Core pg. 435, "End the Encounter". DEV-30.
    #[test]
    fn an_empty_roster_is_not_victory() {
        assert_eq!(encounter_outcome(&[]), EncounterOutcome::Ongoing);
    }

    /// Player Core pg. 435, "End the Encounter". DEV-30.
    #[test]
    fn roster_order_does_not_change_victory() {
        let creature = creature(9, true);
        let pc = new_combatant_state(CombatantId(1), &creature);
        let mut enemy = new_combatant_state(CombatantId(2), &creature);
        lay(&mut enemy, Vital::Dead, 0, 0);
        let forward = [entry(1, Side::Pc, &pc), entry(2, Side::Enemy, &enemy)];
        let reverse = [entry(2, Side::Enemy, &enemy), entry(1, Side::Pc, &pc)];
        let victory = EncounterOutcome::Victory { side: Side::Pc };
        assert_eq!(encounter_outcome(&forward), victory);
        assert_eq!(encounter_outcome(&reverse), victory);
    }

    /// Player Core pg. 435, "End the Encounter". DEV-30.
    #[test]
    fn the_same_roster_twice_matches() {
        let creature = creature(9, true);
        let pc = new_combatant_state(CombatantId(1), &creature);
        let enemy = new_combatant_state(CombatantId(2), &creature);
        let before_pc = pc.clone();
        let before_enemy = enemy.clone();
        let roster = [entry(1, Side::Pc, &pc), entry(2, Side::Enemy, &enemy)];
        assert_eq!(encounter_outcome(&roster), encounter_outcome(&roster));
        assert_eq!(pc, before_pc);
        assert_eq!(enemy, before_enemy);
    }

    /// Player Core pg. 435, "End the Encounter". DEV-30.
    #[test]
    fn blows_in_either_order_end_in_mutual_defeat() {
        let run = |pc_first: bool| {
            let creature = creature(9, true);
            let mut pc = new_combatant_state(CombatantId(1), &creature);
            let mut enemy = new_combatant_state(CombatantId(2), &creature);
            pc.hit_points = 5;
            enemy.hit_points = 5;
            let strike = |state: &mut CombatantState| {
                hit(state, &creature, 8, 5, 0, &[], false).expect("blow");
            };
            if pc_first {
                strike(&mut pc);
                assert_eq!(
                    encounter_outcome(&[entry(1, Side::Pc, &pc), entry(2, Side::Enemy, &enemy)]),
                    EncounterOutcome::Victory { side: Side::Enemy }
                );
                strike(&mut enemy);
            } else {
                strike(&mut enemy);
                assert_eq!(
                    encounter_outcome(&[entry(1, Side::Pc, &pc), entry(2, Side::Enemy, &enemy)]),
                    EncounterOutcome::Victory { side: Side::Pc }
                );
                strike(&mut pc);
            }
            encounter_outcome(&[entry(1, Side::Pc, &pc), entry(2, Side::Enemy, &enemy)])
        };
        assert_eq!(run(true), EncounterOutcome::MutualDefeat);
        assert_eq!(run(false), EncounterOutcome::MutualDefeat);
    }

    /// Player Core pg. 435, "End the Encounter". DEV-30.
    #[test]
    fn recovery_to_unconscious_is_a_mutual_defeat() {
        let creature = creature(9, true);
        let mut pc = new_combatant_state(CombatantId(1), &creature);
        let mut enemy = new_combatant_state(CombatantId(2), &creature);
        lay(&mut pc, Vital::Dying(dying(1)), 0, 0);
        lay(&mut enemy, Vital::Dying(dying(1)), 0, 0);
        recover(&mut pc, &creature, 11).expect("recovery");
        assert_eq!(pc.vital, Vital::Unconscious);
        assert_eq!(pc.hit_points, 0);
        assert_eq!(enemy.vital, Vital::Dying(dying(1)));
        assert_eq!(
            encounter_outcome(&[entry(1, Side::Pc, &pc), entry(2, Side::Enemy, &enemy)]),
            EncounterOutcome::MutualDefeat
        );
    }

    /// Player Core pg. 435, "End the Encounter". DEV-30.
    #[test]
    fn healing_a_downed_creature_returns_the_fight_to_ongoing() {
        let creature = creature(9, true);
        let mut pc = new_combatant_state(CombatantId(1), &creature);
        let enemy = new_combatant_state(CombatantId(2), &creature);
        lay(&mut pc, Vital::Dying(dying(1)), 0, 0);
        assert_eq!(
            encounter_outcome(&[entry(1, Side::Pc, &pc), entry(2, Side::Enemy, &enemy)]),
            EncounterOutcome::Victory { side: Side::Enemy }
        );
        apply_healing(&mut pc, &creature, 4).expect("heal");
        assert_eq!(pc.vital, Vital::Alive);
        assert_eq!(
            encounter_outcome(&[entry(1, Side::Pc, &pc), entry(2, Side::Enemy, &enemy)]),
            EncounterOutcome::Ongoing
        );
    }

    /// Player Core pg. 410, "Getting Knocked Out".
    #[test]
    fn gaining_dying_reports_an_initiative_move() {
        let (creature, mut state) = opened(9, true);
        hit(&mut state, &creature, 8, 8, 1, &[], false).expect("blow");
        assert_eq!(
            take_initiative_move(&mut state),
            Some(InitiativeMove::Pending)
        );
        assert_eq!(take_initiative_move(&mut state), None);
    }

    /// Player Core pg. 410, "Getting Knocked Out".
    #[test]
    fn massive_damage_does_not_report_an_initiative_move() {
        let (creature, mut state) = opened(10, true);
        hit(&mut state, &creature, 8, 8, 12, &[], false).expect("blow");
        assert_eq!(take_initiative_move(&mut state), None);
    }

    /// Player Core pg. 410, "Getting Knocked Out".
    #[test]
    fn death_at_0_without_dying_rules_does_not_report_an_initiative_move() {
        let (creature, mut state) = opened(9, false);
        let mut dice = ScriptedDice::new([8]);
        apply_dice_damage(&mut state, &creature, &mut dice, faces(8), 1, &[], false).expect("blow");
        assert_eq!(dice.remaining(), 0);
        assert_eq!(state.vital, Vital::Dead);
        assert_eq!(take_initiative_move(&mut state), None);
    }

    /// Player Core pg. 410, "Getting Knocked Out".
    #[test]
    fn damage_while_dying_does_not_create_an_initiative_report() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        hit(&mut state, &creature, 8, 1, 0, &[], false).expect("blow");
        assert_eq!(state.vital, Vital::Dying(dying(2)));
        assert_eq!(take_initiative_move(&mut state), None);
    }

    /// Player Core pg. 410, "Getting Knocked Out".
    #[test]
    fn two_dropped_combatants_each_report_once() {
        let creature = creature(9, true);
        let mut first = new_combatant_state(CombatantId(1), &creature);
        let mut second = new_combatant_state(CombatantId(2), &creature);
        hit(&mut first, &creature, 8, 8, 1, &[], false).expect("first");
        hit(&mut second, &creature, 8, 8, 1, &[], false).expect("second");
        assert_eq!(
            take_initiative_move(&mut first),
            Some(InitiativeMove::Pending)
        );
        assert_eq!(
            take_initiative_move(&mut second),
            Some(InitiativeMove::Pending)
        );
        assert_eq!(take_initiative_move(&mut first), None);
        assert_eq!(take_initiative_move(&mut second), None);
    }

    /// Player Core pg. 410, "Getting Knocked Out".
    #[test]
    fn healing_to_alive_clears_the_initiative_report() {
        let (creature, mut state) = opened(10, true);
        hit(&mut state, &creature, 8, 8, 2, &[], false).expect("blow");
        apply_healing(&mut state, &creature, 4).expect("heal");
        assert_eq!(state.vital, Vital::Alive);
        assert_eq!(state.hit_points, 4);
        assert_eq!(take_initiative_move(&mut state), None);
    }

    /// Player Core pg. 410, "Getting Knocked Out".
    #[test]
    fn death_clears_a_stale_initiative_report() {
        let (creature, mut state) = opened(10, true);
        hit(&mut state, &creature, 8, 8, 2, &[], false).expect("drop");
        hit(&mut state, &creature, 8, 8, 12, &[], false).expect("massive");
        assert_eq!(state.vital, Vital::Dead);
        assert_eq!(take_initiative_move(&mut state), None);
    }

    /// Player Core pg. 435, "Start Your Turn".
    #[test]
    fn start_of_turn_does_not_grant_actions() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Dying(dying(1)), 0, 0);
        state.actions_remaining = 2;
        state.reaction_ready = true;
        recover(&mut state, &creature, 11).expect("recovery");
        assert_eq!(state.actions_remaining, 2);
        assert!(state.reaction_ready);
        assert_eq!(state.hit_points, 0);
        assert_eq!(state.wounded, 1);
        assert_eq!(state.vital, Vital::Unconscious);
    }

    /// Player Core pg. 435, "Start Your Turn".
    #[test]
    fn a_knockout_does_not_spend_actions() {
        let (creature, mut state) = opened(9, true);
        state.actions_remaining = 2;
        state.reaction_ready = false;
        hit(&mut state, &creature, 8, 8, 1, &[], false).expect("blow");
        assert_eq!(state.vital, Vital::Dying(dying(1)));
        assert_eq!(state.actions_remaining, 2);
        assert!(!state.reaction_ready);
    }

    /// Player Core pg. 435, "Start Your Turn".
    #[test]
    fn healing_does_not_change_actions() {
        let (creature, mut state) = opened(9, true);
        lay(&mut state, Vital::Unconscious, 0, 1);
        state.actions_remaining = 2;
        state.reaction_ready = true;
        apply_healing(&mut state, &creature, 4).expect("heal");
        assert_eq!(state.actions_remaining, 2);
        assert!(state.reaction_ready);
    }
}
