use std::collections::{BTreeMap, HashSet};
use std::fmt;

use crate::check::{degree_of_success, multiple_attack_penalty, Degree};
use crate::combatant::{
    prepare_combatant, ClassFeature, CreatureTemplate, DamageFormula, PreparedStrike, Side,
};
use crate::dice::Dice;
use crate::grid::{
    minimum_feet, reachable_destinations, squares_within_feet, stride_leaves, Grid, GridPos,
};

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
    pub reaction_ready: bool,
    pub features: Vec<ClassFeature>,
    pub prone: bool,
    /// Dying value while the creature is on the track. `0` when they are not dying. Death clears it.
    pub dying: u8,
    pub wounded: u32,
    pub dead: bool,
}

impl Combatant {
    /// Hit points above 0, and not dead.
    pub fn is_conscious(&self) -> bool {
        self.hit_points > 0 && !self.dead
    }

    /// 0 hit points, and not dead.
    pub fn is_unconscious(&self) -> bool {
        self.hit_points <= 0 && !self.dead
    }
}

/// Closed set of combat actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Stride { destination: GridPos },
    Step { destination: GridPos },
    Strike { target: CombatantId },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Every enemy is dead.
    Victory,
    /// Every party slot is dead.
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
    Stepped {
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
    /// A Reactive Strike resolved before the mover entered the next square.
    ReactiveStrike {
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
    },
    CombatantDowned {
        id: CombatantId,
    },
    CombatantDied {
        id: CombatantId,
    },
    /// Flat recovery check at the start of a dying creature's turn.
    RecoveryCheck {
        id: CombatantId,
        natural: u8,
        dc: i32,
        degree: Degree,
        dying: u8,
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
    NoReaction,
}

impl fmt::Display for Illegal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Illegal::EncounterOver => write!(f, "the encounter is over"),
            Illegal::NotYourTurn => write!(f, "that combatant is not the active one"),
            Illegal::NotLegal => write!(f, "that action is not legal"),
            Illegal::UnknownCombatant => write!(f, "unknown combatant"),
            Illegal::NoReaction => write!(f, "that reaction is already spent"),
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

    /// Four party slots, including any unconscious or dead member.
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

    /// Whether `id` still has a reaction. `None` when `id` is not in this encounter.
    pub fn reaction_ready(&self, id: CombatantId) -> Option<bool> {
        self.combatants
            .get(&id)
            .map(|combatant| combatant.reaction_ready)
    }

    /// Spends `id`'s reaction. A second spend before that creature's next turn returns [`Illegal::NoReaction`].
    pub fn spend_reaction(&mut self, id: CombatantId) -> Result<(), Illegal> {
        let Some(combatant) = self.combatants.get_mut(&id) else {
            return Err(Illegal::UnknownCombatant);
        };
        if !combatant.reaction_ready {
            return Err(Illegal::NoReaction);
        }
        combatant.reaction_ready = false;
        Ok(())
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
        let party_living = self.party.iter().any(|id| !self.combatants[id].dead);
        let enemy_living = self
            .combatants
            .values()
            .any(|combatant| combatant.side == Side::Enemy && !combatant.dead);
        if !enemy_living {
            Some(Outcome::Victory)
        } else if !party_living {
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
            reaction_ready: true,
            features: prepared.features,
            prone: false,
            dying: 0,
            wounded: 0,
            dead: false,
        },
    );
    Ok(())
}

/// Legal Stride, Step, and Strike actions for `id` on their turn.
///
/// Stride lists every square up to Speed that can be reached. Step lists each walkable square
/// whose open-plane distance is 5 feet, and only when Speed is at least 10. Allies can be crossed
/// and not ended on. Conscious enemies cannot be entered. Strike lists each living opponent
/// in reach, including an unconscious one. A dead creature is not a target. Unconscious and dead
/// creatures have no legal Stride, Step, or Strike.
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
        actions.push(Action::Stride { destination });
    }

    if actor.speed_feet >= 10 {
        for destination in squares_within_feet(
            actor.position,
            5,
            encounter.grid.width(),
            encounter.grid.height(),
        ) {
            if minimum_feet(actor.position, destination) != 5 {
                continue;
            }
            if !encounter.grid.walkable(destination)
                || cannot_enter.contains(&destination)
                || cannot_end.contains(&destination)
            {
                continue;
            }
            actions.push(Action::Step { destination });
        }
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
        .filter(|other| other.side != actor.side && !other.dead && reach.contains(&other.position))
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
        Action::Stride { destination } => {
            resolve_move(encounter, id, destination, rng, &mut events)
        }
        Action::Step { destination } => resolve_step(encounter, id, destination, &mut events),
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
    rng: &mut impl Dice,
    events: &mut Vec<Event>,
) {
    let actor = &encounter.combatants[&id];
    let from = actor.position;
    let speed = actor.speed_feet;
    let (cannot_enter, cannot_end) = blockers(encounter, id);
    let (feet, left) = stride_leaves(
        &encounter.grid,
        from,
        destination,
        speed,
        &cannot_enter,
        &cannot_end,
    )
    .expect("legal destination has a stride cost");

    let mut squares: Vec<GridPos> = left.into_iter().map(|(pos, _)| pos).collect();
    squares.push(destination);
    for index in 0..squares.len() - 1 {
        let leaving = squares[index];
        let entering = squares[index + 1];
        for reactor in reactors_for_left_square(encounter, id, leaving) {
            if resolve_reactive_strike(encounter, reactor, id, rng, events) {
                encounter.combatants.get_mut(&id).unwrap().position = leaving;
                encounter.actions_remaining -= 1;
                return;
            }
        }
        encounter.combatants.get_mut(&id).unwrap().position = entering;
    }

    encounter.actions_remaining -= 1;
    events.push(Event::Moved {
        id,
        from,
        to: destination,
        feet,
        actions_remaining: encounter.actions_remaining,
    });
}

/// Opponents who can Reactive Strike a creature leaving `square`.
///
/// Each has [`ClassFeature::ReactiveStrike`], a ready reaction, and `square` inside their reach.
/// The ids are in [`CombatantId`] order.
pub fn reactors_for_left_square(
    encounter: &Encounter,
    mover: CombatantId,
    square: GridPos,
) -> Vec<CombatantId> {
    let Some(mover) = encounter.combatants.get(&mover) else {
        return Vec::new();
    };
    let mut reactors: Vec<CombatantId> = encounter
        .combatants
        .values()
        .filter(|other| {
            other.id != mover.id
                && other.side != mover.side
                && other.is_conscious()
                && other.reaction_ready
                && other.features.contains(&ClassFeature::ReactiveStrike)
                && other.position != square
                && minimum_feet(other.position, square) <= other.strike.reach_feet
        })
        .map(|other| other.id)
        .collect();
    reactors.sort();
    reactors
}

fn resolve_reactive_strike(
    encounter: &mut Encounter,
    reactor: CombatantId,
    mover: CombatantId,
    rng: &mut impl Dice,
    events: &mut Vec<Event>,
) -> bool {
    encounter
        .spend_reaction(reactor)
        .expect("reactor has a ready reaction");
    let attack = resolve_attack(encounter, reactor, mover, 0, rng);
    events.push(Event::ReactiveStrike {
        attacker: reactor,
        target: mover,
        natural: attack.natural,
        attack_bonus: attack.attack_bonus,
        map: attack.map,
        total: attack.total,
        armor_class: attack.armor_class,
        degree: attack.degree,
        damage: attack.damage,
        target_hit_points: attack.target_hit_points,
    });
    push_harm(events, mover, attack.harm);
    !encounter.combatants[&mover].is_conscious()
}

fn resolve_step(
    encounter: &mut Encounter,
    id: CombatantId,
    destination: GridPos,
    events: &mut Vec<Event>,
) {
    let from = encounter.combatants[&id].position;
    encounter.combatants.get_mut(&id).unwrap().position = destination;
    encounter.actions_remaining -= 1;
    events.push(Event::Stepped {
        id,
        from,
        to: destination,
        feet: 5,
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
    let attack = resolve_attack(encounter, id, target, map, rng);
    encounter.strikes_this_turn += 1;
    encounter.actions_remaining -= 1;
    let remaining = encounter.actions_remaining;
    events.push(Event::StrikeResolved {
        attacker: id,
        target,
        natural: attack.natural,
        attack_bonus: attack.attack_bonus,
        map: attack.map,
        total: attack.total,
        armor_class: attack.armor_class,
        degree: attack.degree,
        damage: attack.damage,
        target_hit_points: attack.target_hit_points,
        actions_remaining: remaining,
    });
    push_harm(events, target, attack.harm);
}

struct ResolvedAttack {
    natural: u8,
    attack_bonus: i32,
    map: i32,
    total: i32,
    armor_class: i32,
    degree: Degree,
    damage: i32,
    target_hit_points: i32,
    harm: Harm,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Harm {
    None,
    NowDying,
    Dead,
}

fn resolve_attack(
    encounter: &mut Encounter,
    attacker: CombatantId,
    target: CombatantId,
    map: i32,
    rng: &mut impl Dice,
) -> ResolvedAttack {
    let armor_class = defense_ac(encounter, attacker, target);
    let (attack_bonus, formula, unconscious) = {
        let actor = &encounter.combatants[&attacker];
        let defender = &encounter.combatants[&target];
        (
            actor.strike.attack_bonus,
            actor.strike.damage.clone(),
            defender.is_unconscious(),
        )
    };
    let natural = rng.d20();
    let total = i32::from(natural) + attack_bonus + map;
    let mut degree = degree_of_success(natural, total, armor_class);
    if unconscious && degree == Degree::Success {
        degree = Degree::CriticalSuccess;
    }
    let damage = match degree {
        Degree::Success => roll_damage(rng, &formula),
        Degree::CriticalSuccess => roll_damage(rng, &formula).saturating_mul(2),
        Degree::Failure | Degree::CriticalFailure => 0,
    };
    let (target_hit_points, harm) = {
        let defender = encounter.combatants.get_mut(&target).unwrap();
        let harm = apply_harm(defender, degree, damage);
        (defender.hit_points, harm)
    };
    ResolvedAttack {
        natural,
        attack_bonus,
        map,
        total,
        armor_class,
        degree,
        damage,
        target_hit_points,
        harm,
    }
}

fn defense_ac(encounter: &Encounter, attacker: CombatantId, target: CombatantId) -> i32 {
    let base = encounter.combatants[&target].armor_class;
    if off_guard(encounter, attacker, target) {
        base - 2
    } else {
        base
    }
}

/// Prone, unconscious, or flanked by this attacker. The circumstance penalty applies once.
fn off_guard(encounter: &Encounter, attacker: CombatantId, target: CombatantId) -> bool {
    let defender = &encounter.combatants[&target];
    defender.prone || defender.is_unconscious() || flanked_by(encounter, attacker, target)
}

fn flanked_by(encounter: &Encounter, attacker: CombatantId, target: CombatantId) -> bool {
    let Some(attacker) = encounter.combatants.get(&attacker) else {
        return false;
    };
    let Some(target) = encounter.combatants.get(&target) else {
        return false;
    };
    if !attacker.is_conscious() || attacker.side == target.side {
        return false;
    }
    if minimum_feet(attacker.position, target.position) > attacker.strike.reach_feet {
        return false;
    }
    let attacker_id = attacker.id;
    let attacker_side = attacker.side;
    let attacker_pos = attacker.position;
    let target_id = target.id;
    let target_pos = target.position;
    encounter.combatants.values().any(|ally| {
        ally.id != attacker_id
            && ally.id != target_id
            && ally.side == attacker_side
            && ally.is_conscious()
            && minimum_feet(ally.position, target_pos) <= ally.strike.reach_feet
            && centers_cross_opposite(attacker_pos, target_pos, ally.position)
    })
}

/// Line between square centers crosses opposite sides or opposite corners of `target`.
///
/// Centers are stored on odd coordinates so the square's edges stay on even ones.
fn centers_cross_opposite(attacker: GridPos, target: GridPos, ally: GridPos) -> bool {
    let ax = attacker.x * 2 + 1;
    let ay = attacker.y * 2 + 1;
    let bx = ally.x * 2 + 1;
    let by = ally.y * 2 + 1;
    let left = target.x * 2;
    let right = left + 2;
    let bottom = target.y * 2;
    let top = bottom + 2;
    crosses_opposite(ax, ay, bx, by, left, right, bottom, top)
        || crosses_opposite(ay, ax, by, bx, bottom, top, left, right)
}

fn crosses_opposite(
    ax: i32,
    ay: i32,
    bx: i32,
    by: i32,
    near: i32,
    far: i32,
    low: i32,
    high: i32,
) -> bool {
    segment_hits_edge(ax, ay, bx, by, near, low, high)
        && segment_hits_edge(ax, ay, bx, by, far, low, high)
}

fn segment_hits_edge(ax: i32, ay: i32, bx: i32, by: i32, edge: i32, low: i32, high: i32) -> bool {
    let vx = bx - ax;
    let vy = by - ay;
    if vx == 0 {
        return false;
    }
    let dx = edge - ax;
    if dx.signum() != vx.signum() || dx.abs() >= vx.abs() {
        return false;
    }
    let y_num = ay * vx + dx * vy;
    if vx > 0 {
        y_num >= low * vx && y_num <= high * vx
    } else {
        y_num <= low * vx && y_num >= high * vx
    }
}

fn push_harm(events: &mut Vec<Event>, id: CombatantId, harm: Harm) {
    match harm {
        Harm::NowDying => events.push(Event::CombatantDowned { id }),
        Harm::Dead => events.push(Event::CombatantDied { id }),
        Harm::None => {}
    }
}

fn apply_harm(defender: &mut Combatant, degree: Degree, damage: i32) -> Harm {
    if damage <= 0 || defender.dead {
        return Harm::None;
    }
    if defender.hit_points > 0 && damage >= defender.max_hit_points.saturating_mul(2) {
        defender.hit_points = 0;
        defender.prone = true;
        defender.dying = 0;
        defender.dead = true;
        return Harm::Dead;
    }
    if defender.hit_points > 0 {
        let remaining = defender.hit_points - damage;
        if remaining > 0 {
            defender.hit_points = remaining;
            return Harm::None;
        }
        defender.hit_points = 0;
        return gain_dying(defender, degree);
    }
    if defender.dying > 0 {
        let gain = if degree == Degree::CriticalSuccess {
            2
        } else {
            1
        };
        let dying = u32::from(defender.dying) + gain;
        if dying >= 4 {
            defender.dying = 0;
            defender.dead = true;
            defender.prone = true;
            return Harm::Dead;
        }
        defender.dying = dying as u8;
        return Harm::None;
    }
    gain_dying(defender, degree)
}

fn gain_dying(defender: &mut Combatant, degree: Degree) -> Harm {
    let base = if degree == Degree::CriticalSuccess {
        2
    } else {
        1
    };
    let dying = base + defender.wounded;
    defender.wounded = defender.wounded.saturating_add(1);
    defender.hit_points = 0;
    defender.prone = true;
    if dying >= 4 {
        defender.dying = 0;
        defender.dead = true;
        Harm::Dead
    } else {
        defender.dying = dying as u8;
        Harm::NowDying
    }
}

fn recovery_check(
    encounter: &mut Encounter,
    id: CombatantId,
    rng: &mut impl Dice,
    events: &mut Vec<Event>,
) {
    let dying = encounter.combatants[&id].dying;
    let dc = 10 + i32::from(dying);
    let natural = rng.d20();
    let degree = degree_of_success(natural, i32::from(natural), dc);
    let delta: i32 = match degree {
        Degree::CriticalSuccess => -2,
        Degree::Success => -1,
        Degree::Failure => 1,
        Degree::CriticalFailure => 2,
    };
    let next = i32::from(dying) + delta;
    let combatant = encounter.combatants.get_mut(&id).unwrap();
    if next <= 0 {
        combatant.dying = 0;
    } else if next >= 4 {
        combatant.dying = 0;
        combatant.dead = true;
        combatant.hit_points = 0;
        combatant.prone = true;
    } else {
        combatant.dying = next as u8;
    }
    let dying_after = encounter.combatants[&id].dying;
    events.push(Event::RecoveryCheck {
        id,
        natural,
        dc,
        degree,
        dying: dying_after,
    });
    if encounter.combatants[&id].dead {
        events.push(Event::CombatantDied { id });
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
/// A dying creature's turn starts with a recovery check, then passes when they still cannot act.
/// Dead creatures and unconscious creatures who are not dying are skipped. A party member stays
/// in the party array. A wrap past the end of the order starts the next round.
pub fn advance_if_turn_done(encounter: &mut Encounter, rng: &mut impl Dice) -> Vec<Event> {
    let mut events = Vec::new();
    if finish_if_over(encounter, &mut events) {
        return events;
    }

    let guard = encounter.initiative.len().max(1);
    for _ in 0..guard {
        let active = encounter.initiative[encounter.current];
        let actor = &encounter.combatants[&active];
        let done = encounter.actions_remaining == 0
            || !actor.is_conscious()
            || legal_actions(encounter, active).is_empty();
        if !done {
            break;
        }
        if !advance_to_next(encounter, rng, &mut events) {
            break;
        }
        if finish_if_over(encounter, &mut events) {
            break;
        }
    }
    events
}

/// Sets actions to zero and advances, spending the rest of the active turn.
pub fn end_turn(encounter: &mut Encounter, rng: &mut impl Dice) -> Result<Vec<Event>, Illegal> {
    if encounter.outcome.is_some() {
        return Err(Illegal::EncounterOver);
    }
    if encounter.active().is_none() {
        return Err(Illegal::EncounterOver);
    }
    encounter.actions_remaining = 0;
    Ok(advance_if_turn_done(encounter, rng))
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

fn advance_to_next(
    encounter: &mut Encounter,
    rng: &mut impl Dice,
    events: &mut Vec<Event>,
) -> bool {
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
        let combatant = &encounter.combatants[&id];
        if combatant.dead || (combatant.is_unconscious() && combatant.dying == 0) {
            continue;
        }
        encounter.current = idx;
        if wrapped {
            encounter.round += 1;
            wrapped = false;
        }
        encounter.strikes_this_turn = 0;
        encounter.combatants.get_mut(&id).unwrap().reaction_ready = true;
        let conscious = encounter.combatants[&id].is_conscious();
        let actions = if conscious { 3 } else { 0 };
        encounter.actions_remaining = actions;
        events.push(Event::TurnStarted {
            id,
            round: encounter.round,
            actions,
        });
        if encounter.combatants[&id].dying > 0 {
            recovery_check(encounter, id, rng, events);
        }
        if finish_if_over(encounter, events) {
            return false;
        }
        if encounter.combatants[&id].is_conscious() {
            encounter.actions_remaining = 3;
            return true;
        }
        encounter.actions_remaining = 0;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flank_lines_cross_opposite_sides_or_corners_only() {
        let target = GridPos::new(2, 2);
        assert!(centers_cross_opposite(
            GridPos::new(1, 2),
            target,
            GridPos::new(3, 2)
        ));
        assert!(centers_cross_opposite(
            GridPos::new(2, 4),
            target,
            GridPos::new(2, 0)
        ));
        assert!(centers_cross_opposite(
            GridPos::new(3, 3),
            target,
            GridPos::new(1, 1)
        ));
        assert!(centers_cross_opposite(
            GridPos::new(1, 3),
            target,
            GridPos::new(3, 1)
        ));
        assert!(!centers_cross_opposite(
            GridPos::new(1, 2),
            target,
            GridPos::new(2, 1)
        ));
        assert!(!centers_cross_opposite(
            GridPos::new(1, 2),
            target,
            GridPos::new(1, 3)
        ));
        assert!(!centers_cross_opposite(
            GridPos::new(1, 1),
            target,
            GridPos::new(3, 2)
        ));
    }
}
