use std::collections::{BTreeMap, HashSet};
use std::fmt;

use crate::check::{degree_of_success, multiple_attack_penalty, Degree};
use crate::combatant::{prepare_combatant, CreatureTemplate, DamageFormula, PreparedStrike, Side};
use crate::dice::Dice;
use crate::grid::{reachable_destinations, squares_within_feet, Grid, GridPos};

/// Stable id for a creature in one encounter. Rules never use engine entity ids.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CombatantId(pub u32);

impl fmt::Display for CombatantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A creature placed on the encounter grid.
#[derive(Clone, Debug)]
pub struct Combatant {
    pub id: CombatantId,
    pub name: String,
    pub side: Side,
    pub level: u8,
    pub max_hit_points: i32,
    pub hit_points: i32,
    pub armor_class: i32,
    pub perception: i32,
    pub speed_feet: u32,
    pub strike: PreparedStrike,
    pub position: GridPos,
    pub initiative_roll: u8,
    pub initiative_total: i32,
}

impl Combatant {
    pub fn is_conscious(&self) -> bool {
        self.hit_points > 0
    }
}

/// Closed set of combat actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Move { destination: GridPos },
    Strike { target: CombatantId },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Every enemy is at 0 HP.
    Victory,
    /// Every party slot is at 0 HP.
    Defeat,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Moved {
        id: CombatantId,
        from: GridPos,
        to: GridPos,
        feet: u32,
        actions_remaining: u8,
    },
    StrikeResolved {
        attacker: CombatantId,
        target: CombatantId,
        natural: u8,
        attack_bonus: i32,
        map: i32,
        total: i32,
        armor_class: i32,
        degree: Degree,
        damage: i32,
        target_hit_points: i32,
        actions_remaining: u8,
    },
    CombatantDowned {
        id: CombatantId,
    },
    TurnStarted {
        id: CombatantId,
        round: u32,
        actions: u8,
    },
    EncounterEnded {
        outcome: Outcome,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartError {
    PositionCount,
    Unwalkable { pos: GridPos },
    Overlap { pos: GridPos },
    InvalidHitPoints { name: String },
}

impl fmt::Display for StartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StartError::PositionCount => write!(f, "spawn count does not match combatants"),
            StartError::Unwalkable { pos } => {
                write!(f, "spawn ({}, {}) is not a walkable square", pos.x, pos.y)
            }
            StartError::Overlap { pos } => {
                write!(f, "spawn ({}, {}) is already occupied", pos.x, pos.y)
            }
            StartError::InvalidHitPoints { name } => {
                write!(f, "{name} must start with hit points above 0")
            }
        }
    }
}

impl std::error::Error for StartError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Illegal {
    EncounterOver,
    NotYourTurn,
    NotLegal,
    UnknownCombatant,
}

impl fmt::Display for Illegal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Illegal::EncounterOver => write!(f, "the encounter is over"),
            Illegal::NotYourTurn => write!(f, "that combatant is not the active one"),
            Illegal::NotLegal => write!(f, "that action is not legal"),
            Illegal::UnknownCombatant => write!(f, "unknown combatant"),
        }
    }
}

impl std::error::Error for Illegal {}

/// One fight: initiative order, the current index, combatants, and the 5-foot grid.
#[derive(Clone, Debug)]
pub struct Encounter {
    combatants: BTreeMap<CombatantId, Combatant>,
    party: [CombatantId; 4],
    initiative: Vec<CombatantId>,
    current: usize,
    round: u32,
    grid: Grid,
    actions_remaining: u8,
    strikes_this_turn: u8,
    outcome: Option<Outcome>,
}

impl Encounter {
    pub fn combatant(&self, id: CombatantId) -> Option<&Combatant> {
        self.combatants.get(&id)
    }

    pub fn combatants(&self) -> impl Iterator<Item = &Combatant> {
        self.combatants.values()
    }

    /// Four party slots, including any at 0 HP.
    pub fn party(&self) -> [CombatantId; 4] {
        self.party
    }

    pub fn initiative(&self) -> &[CombatantId] {
        &self.initiative
    }

    pub fn current_index(&self) -> usize {
        self.current
    }

    pub fn round(&self) -> u32 {
        self.round
    }

    pub fn actions_remaining(&self) -> u8 {
        self.actions_remaining
    }

    pub fn strikes_this_turn(&self) -> u8 {
        self.strikes_this_turn
    }

    pub fn outcome(&self) -> Option<Outcome> {
        self.outcome
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    /// `None` once the encounter has an outcome.
    pub fn active(&self) -> Option<CombatantId> {
        if self.outcome.is_some() {
            None
        } else {
            self.initiative.get(self.current).copied()
        }
    }

    fn detect_outcome(&self) -> Option<Outcome> {
        let party_up = self
            .party
            .iter()
            .any(|id| self.combatants[id].is_conscious());
        let enemy_up = self
            .combatants
            .values()
            .any(|combatant| combatant.side == Side::Enemy && combatant.is_conscious());
        if !enemy_up {
            Some(Outcome::Victory)
        } else if !party_up {
            Some(Outcome::Defeat)
        } else {
            None
        }
    }
}

/// Prepares the party and enemies, places them, and rolls Perception initiative.
///
/// Party length is fixed at four. Initiative dice are rolled in party-slot order, then enemy order.
/// Ties break by higher Perception modifier, then lower [`CombatantId`].
pub fn start_encounter(
    party: &[CreatureTemplate; 4],
    enemies: &[CreatureTemplate],
    party_positions: &[GridPos; 4],
    enemy_positions: &[GridPos],
    grid: Grid,
    rng: &mut impl Dice,
) -> Result<Encounter, StartError> {
    if enemy_positions.len() != enemies.len() {
        return Err(StartError::PositionCount);
    }

    let mut combatants = BTreeMap::new();
    let mut occupied = HashSet::new();
    let mut party_ids = [CombatantId(0); 4];

    for (slot, (template, pos)) in party.iter().zip(party_positions.iter()).enumerate() {
        let id = CombatantId(slot as u32);
        party_ids[slot] = id;
        insert_combatant(
            &mut combatants,
            &mut occupied,
            &grid,
            id,
            template,
            *pos,
            rng,
        )?;
    }
    for (index, (template, pos)) in enemies.iter().zip(enemy_positions.iter()).enumerate() {
        let id = CombatantId(4 + index as u32);
        insert_combatant(
            &mut combatants,
            &mut occupied,
            &grid,
            id,
            template,
            *pos,
            rng,
        )?;
    }

    let mut initiative: Vec<CombatantId> = combatants.keys().copied().collect();
    initiative.sort_by(|left, right| {
        let a = &combatants[left];
        let b = &combatants[right];
        b.initiative_total
            .cmp(&a.initiative_total)
            .then(b.perception.cmp(&a.perception))
            .then(left.cmp(right))
    });

    let mut encounter = Encounter {
        combatants,
        party: party_ids,
        initiative,
        current: 0,
        round: 1,
        grid,
        actions_remaining: 3,
        strikes_this_turn: 0,
        outcome: None,
    };
    if let Some(outcome) = encounter.detect_outcome() {
        encounter.outcome = Some(outcome);
    }
    Ok(encounter)
}

fn insert_combatant(
    combatants: &mut BTreeMap<CombatantId, Combatant>,
    occupied: &mut HashSet<GridPos>,
    grid: &Grid,
    id: CombatantId,
    template: &CreatureTemplate,
    pos: GridPos,
    rng: &mut impl Dice,
) -> Result<(), StartError> {
    if template.hit_points <= 0 {
        return Err(StartError::InvalidHitPoints {
            name: template.name.clone(),
        });
    }
    if !grid.walkable(pos) {
        return Err(StartError::Unwalkable { pos });
    }
    if !occupied.insert(pos) {
        return Err(StartError::Overlap { pos });
    }
    let prepared = prepare_combatant(template);
    let natural = rng.d20();
    let total = i32::from(natural) + prepared.perception;
    combatants.insert(
        id,
        Combatant {
            id,
            name: prepared.name,
            side: prepared.side,
            level: prepared.level,
            max_hit_points: prepared.hit_points,
            hit_points: prepared.hit_points,
            armor_class: prepared.armor_class,
            perception: prepared.perception,
            speed_feet: prepared.speed_feet,
            strike: prepared.strike,
            position: pos,
            initiative_roll: natural,
            initiative_total: total,
        },
    );
    Ok(())
}

/// Legal Move and Strike actions for `id` on their turn.
///
/// Move lists every square a Stride of up to Speed can reach. Allies can be crossed and not
/// ended on. Conscious enemies cannot be entered. Strike lists each conscious opponent in reach.
pub fn legal_actions(encounter: &Encounter, id: CombatantId) -> Vec<Action> {
    if encounter.outcome.is_some() || encounter.actions_remaining == 0 {
        return Vec::new();
    }
    if encounter.active() != Some(id) {
        return Vec::new();
    }
    let Some(actor) = encounter.combatants.get(&id) else {
        return Vec::new();
    };
    if !actor.is_conscious() {
        return Vec::new();
    }

    let mut cannot_enter = HashSet::new();
    let mut cannot_end = HashSet::new();
    for other in encounter.combatants.values() {
        if other.id == id || !other.is_conscious() {
            continue;
        }
        cannot_end.insert(other.position);
        if other.side != actor.side {
            cannot_enter.insert(other.position);
        }
    }

    let mut actions = Vec::new();
    for (destination, _) in reachable_destinations(
        &encounter.grid,
        actor.position,
        actor.speed_feet,
        &cannot_enter,
        &cannot_end,
    ) {
        actions.push(Action::Move { destination });
    }

    let reach = squares_within_feet(
        actor.position,
        actor.strike.reach_feet,
        encounter.grid.width(),
        encounter.grid.height(),
    );
    let mut targets: Vec<CombatantId> = encounter
        .combatants
        .values()
        .filter(|other| {
            other.side != actor.side && other.is_conscious() && reach.contains(&other.position)
        })
        .map(|other| other.id)
        .collect();
    targets.sort();
    for target in targets {
        actions.push(Action::Strike { target });
    }
    actions
}

/// Resolves one legal action for the active combatant.
///
/// Strike rolls the d20 first. Damage dice are rolled only when the degree deals damage.
pub fn declare_action(
    encounter: &mut Encounter,
    id: CombatantId,
    action: Action,
    rng: &mut impl Dice,
) -> Result<Vec<Event>, Illegal> {
    if encounter.outcome.is_some() {
        return Err(Illegal::EncounterOver);
    }
    if !encounter.combatants.contains_key(&id) {
        return Err(Illegal::UnknownCombatant);
    }
    if encounter.active() != Some(id) {
        return Err(Illegal::NotYourTurn);
    }
    if !legal_actions(encounter, id).contains(&action) {
        return Err(Illegal::NotLegal);
    }

    let mut events = Vec::new();
    match action {
        Action::Move { destination } => resolve_move(encounter, id, destination, &mut events),
        Action::Strike { target } => resolve_strike(encounter, id, target, rng, &mut events),
    }
    if let Some(outcome) = encounter.detect_outcome() {
        encounter.outcome = Some(outcome);
        events.push(Event::EncounterEnded { outcome });
    }
    Ok(events)
}

fn resolve_move(
    encounter: &mut Encounter,
    id: CombatantId,
    destination: GridPos,
    events: &mut Vec<Event>,
) {
    let actor = &encounter.combatants[&id];
    let from = actor.position;
    let (cannot_enter, cannot_end) = blockers(encounter, id);
    let feet = reachable_destinations(
        &encounter.grid,
        from,
        actor.speed_feet,
        &cannot_enter,
        &cannot_end,
    )
    .into_iter()
    .find(|(pos, _)| *pos == destination)
    .map(|(_, feet)| feet)
    .expect("legal destination has a stride cost");

    encounter.combatants.get_mut(&id).unwrap().position = destination;
    encounter.actions_remaining -= 1;
    events.push(Event::Moved {
        id,
        from,
        to: destination,
        feet,
        actions_remaining: encounter.actions_remaining,
    });
}

fn resolve_strike(
    encounter: &mut Encounter,
    id: CombatantId,
    target: CombatantId,
    rng: &mut impl Dice,
    events: &mut Vec<Event>,
) {
    let map = multiple_attack_penalty(encounter.strikes_this_turn);
    let (attack_bonus, armor_class, formula) = {
        let actor = &encounter.combatants[&id];
        let defender = &encounter.combatants[&target];
        (
            actor.strike.attack_bonus,
            defender.armor_class,
            actor.strike.damage.clone(),
        )
    };
    let natural = rng.d20();
    let total = i32::from(natural) + attack_bonus + map;
    let degree = degree_of_success(natural, total, armor_class);
    let damage = match degree {
        Degree::Success => roll_damage(rng, &formula),
        Degree::CriticalSuccess => roll_damage(rng, &formula).saturating_mul(2),
        Degree::Failure | Degree::CriticalFailure => 0,
    };

    encounter.strikes_this_turn += 1;
    encounter.actions_remaining -= 1;
    let remaining = encounter.actions_remaining;

    let target_hp = {
        let defender = encounter.combatants.get_mut(&target).unwrap();
        if damage > 0 {
            defender.hit_points = (defender.hit_points - damage).max(0);
        }
        defender.hit_points
    };
    events.push(Event::StrikeResolved {
        attacker: id,
        target,
        natural,
        attack_bonus,
        map,
        total,
        armor_class,
        degree,
        damage,
        target_hit_points: target_hp,
        actions_remaining: remaining,
    });
    if target_hp == 0 {
        events.push(Event::CombatantDowned { id: target });
    }
}

fn roll_damage(rng: &mut impl Dice, formula: &DamageFormula) -> i32 {
    let mut total = formula.bonus;
    for _ in 0..formula.dice {
        total += i32::from(rng.roll(formula.faces));
    }
    total.max(0)
}

fn blockers(encounter: &Encounter, id: CombatantId) -> (HashSet<GridPos>, HashSet<GridPos>) {
    let side = encounter.combatants[&id].side;
    let mut cannot_enter = HashSet::new();
    let mut cannot_end = HashSet::new();
    for other in encounter.combatants.values() {
        if other.id == id || !other.is_conscious() {
            continue;
        }
        cannot_end.insert(other.position);
        if other.side != side {
            cannot_enter.insert(other.position);
        }
    }
    (cannot_enter, cannot_end)
}

/// Ends the active turn when no actions remain or none of the remaining actions are legal.
///
/// Conscious combatants with no legal action are skipped. Downed combatants are skipped and stay
/// in the party array. A wrap past the end of the order starts the next round.
pub fn advance_if_turn_done(encounter: &mut Encounter) -> Vec<Event> {
    let mut events = Vec::new();
    if finish_if_over(encounter, &mut events) {
        return events;
    }

    let guard = encounter.initiative.len().max(1);
    for _ in 0..guard {
        let active = encounter.initiative[encounter.current];
        let done = encounter.actions_remaining == 0
            || !encounter.combatants[&active].is_conscious()
            || legal_actions(encounter, active).is_empty();
        if !done {
            break;
        }
        if !advance_to_next(encounter, &mut events) {
            break;
        }
        if finish_if_over(encounter, &mut events) {
            break;
        }
    }
    events
}

/// Sets actions to zero and advances, spending the rest of the active turn.
pub fn end_turn(encounter: &mut Encounter) -> Result<Vec<Event>, Illegal> {
    if encounter.outcome.is_some() {
        return Err(Illegal::EncounterOver);
    }
    if encounter.active().is_none() {
        return Err(Illegal::EncounterOver);
    }
    encounter.actions_remaining = 0;
    Ok(advance_if_turn_done(encounter))
}

fn finish_if_over(encounter: &mut Encounter, events: &mut Vec<Event>) -> bool {
    if encounter.outcome.is_some() {
        return true;
    }
    if let Some(outcome) = encounter.detect_outcome() {
        encounter.outcome = Some(outcome);
        events.push(Event::EncounterEnded { outcome });
        return true;
    }
    false
}

fn advance_to_next(encounter: &mut Encounter, events: &mut Vec<Event>) -> bool {
    let len = encounter.initiative.len();
    let mut idx = encounter.current;
    let mut wrapped = false;
    for _ in 0..len {
        idx += 1;
        if idx >= len {
            idx = 0;
            wrapped = true;
        }
        let id = encounter.initiative[idx];
        if encounter.combatants[&id].is_conscious() {
            encounter.current = idx;
            if wrapped {
                encounter.round += 1;
            }
            encounter.actions_remaining = 3;
            encounter.strikes_this_turn = 0;
            events.push(Event::TurnStarted {
                id,
                round: encounter.round,
                actions: 3,
            });
            return true;
        }
    }
    false
}
