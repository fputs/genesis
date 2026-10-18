use std::collections::{BTreeMap, HashSet};
use std::fmt;

use crate::check::{degree_of_success, multiple_attack_penalty, Degree};
use crate::combatant::{
    prepare_combatant, ClassFeature, CreatureTemplate, DamageFormula, PreparedStrike, Shield, Side,
};
use crate::dice::Dice;
use crate::grid::{
    minimum_feet, reachable_destinations, squares_within_feet, stride_leaves, Grid, GridPos,
    Terrain,
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
    pub shield: Option<Shield>,
    pub holding_weapon: bool,
    pub shield_block: bool,
    pub prone: bool,
    /// Absent means this creature cannot be Hidden or Undetected.
    pub stealth_dc: Option<i32>,
    /// Set by Take Cover. The bonus is derived per attack and is not stored.
    pub taking_cover: bool,
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

/// The weapon Strike uses, or the wielded shield.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gear {
    Weapon,
    Shield,
}

/// Closed set of combat actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Stride {
        destination: GridPos,
    },
    Step {
        destination: GridPos,
    },
    Strike {
        target: CombatantId,
    },
    /// One action. Legal only with a shield that is not broken and not destroyed.
    RaiseShield,
    /// One action. Sets prone. Illegal when already prone, unconscious, or dead.
    DropProne,
    /// One action. Clears prone. Illegal unless the creature is prone and conscious.
    Stand,
    /// One action. One square that costs 5 feet, and only while prone.
    Crawl {
        destination: GridPos,
    },
    /// One action. Horizontal leap. Illegal below Speed 15 and while prone.
    Leap {
        destination: GridPos,
    },
    /// One action. Perception against each Hidden or Undetected enemy who has a Stealth DC.
    Seek,
    /// One action. Allies who had `target` Undetected now have them Hidden.
    PointOut {
        target: CombatantId,
    },
    /// One action. Sets `taking_cover` when prone, beside a wall, or already in cover.
    TakeCover,
    /// Free action. Drops a held weapon or shield in the actor's square.
    Release {
        item: Gear,
    },
    /// One action. Picks up one dropped object in the actor's square. A shield comes first.
    Interact,
}

/// How one observer detects one subject. Observed is not stored; a missing pair means Observed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Detection {
    Concealed,
    Hidden,
    Undetected,
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
    DroppedProne {
        id: CombatantId,
        actions_remaining: u8,
    },
    Stood {
        id: CombatantId,
        actions_remaining: u8,
    },
    Crawled {
        id: CombatantId,
        from: GridPos,
        to: GridPos,
        feet: u32,
        actions_remaining: u8,
    },
    Leapt {
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
    ShieldRaised {
        id: CombatantId,
        actions_remaining: u8,
    },
    /// Hardness prevented `prevented`. Creature and shield each take `damage`.
    ShieldBlock {
        defender: CombatantId,
        prevented: i32,
        damage: i32,
        shield_hit_points: i32,
        broken: bool,
        destroyed: bool,
    },
    EncounterEnded {
        outcome: Outcome,
    },
    Sought {
        id: CombatantId,
        rolls: Vec<SeekRoll>,
        actions_remaining: u8,
    },
    PointedOut {
        id: CombatantId,
        target: CombatantId,
        actions_remaining: u8,
    },
    TookCover {
        id: CombatantId,
        actions_remaining: u8,
    },
    /// The flat check missed. The Strike does not count toward the multiple attack penalty.
    FlatCheckFailed {
        attacker: CombatantId,
        target: CombatantId,
        natural: u8,
        dc: i32,
        actions_remaining: u8,
    },
    Released {
        id: CombatantId,
        item: Gear,
        actions_remaining: u8,
    },
    PickedUp {
        id: CombatantId,
        item: Gear,
        actions_remaining: u8,
    },
}

/// One Perception roll from Seek.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeekRoll {
    pub target: CombatantId,
    pub natural: u8,
    pub total: i32,
    pub dc: i32,
    pub degree: Degree,
}

/// A strike has rolled damage and is waiting on Shield Block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShieldBlockOffer {
    pub defender: CombatantId,
    pub attacker: CombatantId,
    pub damage: i32,
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
    pending: Option<PendingShieldBlock>,
    /// `(observer, subject)`. Missing means Observed.
    detection: BTreeMap<(CombatantId, CombatantId), Detection>,
    /// At most one dropped weapon and one dropped shield per square.
    dropped: BTreeMap<GridPos, Dropped>,
}

#[derive(Clone, Debug, Default)]
struct Dropped {
    weapon: bool,
    shield: Option<Shield>,
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

    /// Damage is rolled. The caller accepts or declines before it is applied.
    pub fn pending_shield_block(&self) -> Option<ShieldBlockOffer> {
        self.pending.as_ref().map(|pending| ShieldBlockOffer {
            defender: pending.defender,
            attacker: pending.attacker,
            damage: pending.rolled.damage,
        })
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    /// `None` means Observed. Observed is never stored.
    pub fn detection(&self, observer: CombatantId, subject: CombatantId) -> Option<Detection> {
        self.detection.get(&(observer, subject)).copied()
    }

    pub fn dropped_weapon(&self, pos: GridPos) -> bool {
        self.dropped.get(&pos).is_some_and(|dropped| dropped.weapon)
    }

    pub fn dropped_shield(&self, pos: GridPos) -> Option<&Shield> {
        self.dropped
            .get(&pos)
            .and_then(|dropped| dropped.shield.as_ref())
    }

    /// Stores Concealed, Hidden, or Undetected. `None` clears the pair back to Observed.
    ///
    /// Hidden and Undetected are refused when the subject has no Stealth DC.
    pub fn set_detection(
        &mut self,
        observer: CombatantId,
        subject: CombatantId,
        detection: Option<Detection>,
    ) -> Result<(), Illegal> {
        if !self.combatants.contains_key(&observer) || !self.combatants.contains_key(&subject) {
            return Err(Illegal::UnknownCombatant);
        }
        if observer == subject {
            return Err(Illegal::NotLegal);
        }
        match detection {
            Some(Detection::Hidden | Detection::Undetected)
                if self.combatants[&subject].stealth_dc.is_none() =>
            {
                Err(Illegal::NotLegal)
            }
            Some(state) => {
                self.detection.insert((observer, subject), state);
                Ok(())
            }
            None => {
                self.detection.remove(&(observer, subject));
                Ok(())
            }
        }
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
        pending: None,
        detection: BTreeMap::new(),
        dropped: BTreeMap::new(),
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
            shield: prepared.shield,
            holding_weapon: true,
            shield_block: prepared.shield_block,
            prone: false,
            stealth_dc: prepared.stealth_dc,
            taking_cover: false,
            dying: 0,
            wounded: 0,
            dead: false,
        },
    );
    Ok(())
}

/// Legal actions for `id` on their turn.
///
/// Stride lists every square up to Speed that can be reached. Step lists each walkable normal
/// square whose open-plane distance is 5 feet, and only when Speed is at least 10. Difficult and
/// greater squares are not Step destinations. Allies can be crossed and not ended on. Conscious
/// enemies cannot be entered. While prone, Stride, Step, and Leap are absent. Crawl lists one
/// square whose enter cost is at most 5 feet, and Stand is legal. Leap lists squares up to its
/// own distance within the foot-cost rule, and it does not enter another creature's square.
/// Strike lists each living opponent in reach, including an unconscious one, except a creature
/// who is Undetected by this attacker. A dead creature is not a target. Raise a Shield is legal
/// when the creature is wielding a shield that is not broken and not destroyed. Drop Prone is
/// legal when the creature is not already prone. Seek is legal. Point Out is legal for each
/// creature this attacker Observes when at least one ally has that creature Undetected. Take
/// Cover is legal when the creature is prone, adjacent to an unwalkable square, or would already
/// have cover against at least one enemy. Strike is absent without a held weapon. Interact is
/// legal when the square has a dropped object and the actor does not already hold that kind. A
/// shield in the square is that object even when a weapon is there too. Release is a free
/// action, legal while that item is held and the square does not already hold one of that kind,
/// including when no other actions remain. Unconscious and dead creatures have no legal actions.
/// None are legal while a Shield
/// Block offer is open.
pub fn legal_actions(encounter: &Encounter, id: CombatantId) -> Vec<Action> {
    if encounter.pending.is_some() || encounter.outcome.is_some() {
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
    if encounter.actions_remaining == 0 {
        return free_releases(encounter, id);
    }

    let (cannot_enter, cannot_end) = blockers(encounter, id);
    let mut actions = Vec::new();
    if actor.prone {
        for (destination, _) in reachable_destinations(
            &encounter.grid,
            actor.position,
            5,
            &cannot_enter,
            &cannot_end,
        ) {
            actions.push(Action::Crawl { destination });
        }
        actions.push(Action::Stand);
    } else {
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
                if encounter.grid.terrain(destination) != Terrain::Normal {
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

        if let Some(budget) = leap_feet(actor.speed_feet) {
            let occupied = creature_squares(encounter, id);
            for (destination, _) in reachable_destinations(
                &encounter.grid,
                actor.position,
                budget,
                &occupied,
                &occupied,
            ) {
                actions.push(Action::Leap { destination });
            }
        }
        actions.push(Action::DropProne);
    }

    if actor.holding_weapon {
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
                other.side != actor.side && !other.dead && reach.contains(&other.position)
            })
            .map(|other| other.id)
            .collect();
        targets.retain(|target| encounter.detection(id, *target) != Some(Detection::Undetected));
        targets.sort();
        for target in targets {
            actions.push(Action::Strike { target });
        }
    }
    if actor
        .shield
        .as_ref()
        .is_some_and(|shield| !shield.is_broken() && !shield.is_destroyed())
    {
        actions.push(Action::RaiseShield);
    }
    if take_cover_legal(encounter, id) {
        actions.push(Action::TakeCover);
    }
    actions.push(Action::Seek);
    for target in point_out_targets(encounter, id) {
        actions.push(Action::PointOut { target });
    }
    if interact_item(encounter, id).is_some() {
        actions.push(Action::Interact);
    }
    actions.extend(free_releases(encounter, id));
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
    if encounter.pending.is_some() {
        return Err(Illegal::NotLegal);
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
            let speed = encounter.combatants[&id].speed_feet;
            let (cannot_enter, cannot_end) = blockers(encounter, id);
            resolve_travel(
                encounter,
                id,
                destination,
                speed,
                &cannot_enter,
                &cannot_end,
                Travel::Stride,
                rng,
                &mut events,
            );
        }
        Action::Step { destination } => resolve_step(encounter, id, destination, &mut events),
        Action::Strike { target } => resolve_strike(encounter, id, target, rng, &mut events),
        Action::RaiseShield => resolve_raise_shield(encounter, id, &mut events),
        Action::DropProne => resolve_drop_prone(encounter, id, &mut events),
        Action::Stand => resolve_stand(encounter, id, &mut events),
        Action::Crawl { destination } => {
            let (cannot_enter, cannot_end) = blockers(encounter, id);
            resolve_travel(
                encounter,
                id,
                destination,
                5,
                &cannot_enter,
                &cannot_end,
                Travel::Crawl,
                rng,
                &mut events,
            );
        }
        Action::Leap { destination } => {
            let budget = leap_feet(encounter.combatants[&id].speed_feet)
                .expect("a legal leap has a distance");
            let occupied = creature_squares(encounter, id);
            resolve_travel(
                encounter,
                id,
                destination,
                budget,
                &occupied,
                &occupied,
                Travel::Leap,
                rng,
                &mut events,
            );
        }
        Action::Seek => resolve_seek(encounter, id, rng, &mut events),
        Action::PointOut { target } => resolve_point_out(encounter, id, target, &mut events),
        Action::TakeCover => resolve_take_cover(encounter, id, &mut events),
        Action::Release { item } => {
            resolve_manipulate(encounter, id, ManipKind::Release(item), rng, &mut events)
        }
        Action::Interact => {
            resolve_manipulate(encounter, id, ManipKind::Interact, rng, &mut events)
        }
    }
    if let Some(outcome) = encounter.detect_outcome() {
        encounter.outcome = Some(outcome);
        events.push(Event::EncounterEnded { outcome });
    }
    Ok(events)
}

#[derive(Clone, Copy, Debug)]
enum Travel {
    Stride,
    Crawl,
    Leap,
}

#[derive(Clone, Debug)]
struct MoveResume {
    mover: CombatantId,
    from: GridPos,
    destination: GridPos,
    feet: u32,
    squares: Vec<GridPos>,
    index: usize,
    travel: Travel,
}

/// Leap distance is its own cap. It does not spend the creature's Speed.
fn leap_feet(speed_feet: u32) -> Option<u32> {
    if speed_feet >= 30 {
        Some(15)
    } else if speed_feet >= 15 {
        Some(10)
    } else {
        None
    }
}

#[derive(Clone, Debug)]
struct RolledAttack {
    natural: u8,
    attack_bonus: i32,
    map: i32,
    total: i32,
    armor_class: i32,
    degree: Degree,
    damage: i32,
}

#[derive(Clone, Debug)]
enum PendingKind {
    Strike,
    Reactive(MoveResume),
    Manipulate(ManipulateResume),
}

#[derive(Clone, Debug)]
struct ManipulateResume {
    actor: CombatantId,
    kind: ManipKind,
    reactors: Vec<CombatantId>,
    next: usize,
    disrupted: bool,
}

#[derive(Clone, Copy, Debug)]
enum ManipKind {
    Release(Gear),
    Interact,
}

#[derive(Clone, Debug)]
struct PendingShieldBlock {
    rolled: RolledAttack,
    attacker: CombatantId,
    defender: CombatantId,
    kind: PendingKind,
}

fn resolve_travel(
    encounter: &mut Encounter,
    id: CombatantId,
    destination: GridPos,
    budget_feet: u32,
    cannot_enter: &HashSet<GridPos>,
    cannot_end: &HashSet<GridPos>,
    travel: Travel,
    rng: &mut impl Dice,
    events: &mut Vec<Event>,
) {
    release_cover(encounter, id);
    let from = encounter.combatants[&id].position;
    let (feet, left) = stride_leaves(
        &encounter.grid,
        from,
        destination,
        budget_feet,
        cannot_enter,
        cannot_end,
    )
    .expect("legal destination has a foot cost");

    let mut squares: Vec<GridPos> = left.into_iter().map(|(pos, _)| pos).collect();
    squares.push(destination);
    continue_move(
        encounter,
        MoveResume {
            mover: id,
            from,
            destination,
            feet,
            squares,
            index: 0,
            travel,
        },
        rng,
        events,
    );
}

fn resolve_drop_prone(encounter: &mut Encounter, id: CombatantId, events: &mut Vec<Event>) {
    let combatant = encounter.combatants.get_mut(&id).unwrap();
    combatant.taking_cover = false;
    combatant.prone = true;
    encounter.actions_remaining -= 1;
    events.push(Event::DroppedProne {
        id,
        actions_remaining: encounter.actions_remaining,
    });
}

fn resolve_stand(encounter: &mut Encounter, id: CombatantId, events: &mut Vec<Event>) {
    let combatant = encounter.combatants.get_mut(&id).unwrap();
    combatant.taking_cover = false;
    combatant.prone = false;
    encounter.actions_remaining -= 1;
    events.push(Event::Stood {
        id,
        actions_remaining: encounter.actions_remaining,
    });
}

fn continue_move(
    encounter: &mut Encounter,
    mut resume: MoveResume,
    rng: &mut impl Dice,
    events: &mut Vec<Event>,
) {
    while resume.index < resume.squares.len() - 1 {
        let leaving = resume.squares[resume.index];
        let entering = resume.squares[resume.index + 1];
        // Spent reactions drop out, including a reactor whose damage is waiting on Shield Block.
        let reactors = reactors_for_left_square(encounter, resume.mover, leaving);
        for reactor in reactors {
            if begin_reactive_strike(encounter, reactor, &resume, rng, events) {
                return;
            }
            if !encounter.combatants[&resume.mover].is_conscious() {
                encounter
                    .combatants
                    .get_mut(&resume.mover)
                    .unwrap()
                    .position = leaving;
                encounter.actions_remaining -= 1;
                return;
            }
        }
        encounter
            .combatants
            .get_mut(&resume.mover)
            .unwrap()
            .position = entering;
        resume.index += 1;
    }

    encounter.actions_remaining -= 1;
    let actions_remaining = encounter.actions_remaining;
    events.push(match resume.travel {
        Travel::Stride => Event::Moved {
            id: resume.mover,
            from: resume.from,
            to: resume.destination,
            feet: resume.feet,
            actions_remaining,
        },
        Travel::Crawl => Event::Crawled {
            id: resume.mover,
            from: resume.from,
            to: resume.destination,
            feet: resume.feet,
            actions_remaining,
        },
        Travel::Leap => Event::Leapt {
            id: resume.mover,
            from: resume.from,
            to: resume.destination,
            feet: resume.feet,
            actions_remaining,
        },
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
    let mover_id = mover.id;
    let mover_side = mover.side;
    let mut reactors: Vec<CombatantId> = encounter
        .combatants
        .values()
        .filter(|other| {
            other.id != mover_id
                && other.side != mover_side
                && other.is_conscious()
                && other.reaction_ready
                && other.features.contains(&ClassFeature::ReactiveStrike)
                && other.holding_weapon
                && other.position != square
                && minimum_feet(other.position, square) <= other.strike.reach_feet
        })
        .map(|other| other.id)
        .collect();
    reactors
        .retain(|reactor| encounter.detection(*reactor, mover_id) != Some(Detection::Undetected));
    reactors.sort();
    reactors
}

fn begin_reactive_strike(
    encounter: &mut Encounter,
    reactor: CombatantId,
    resume: &MoveResume,
    rng: &mut impl Dice,
    events: &mut Vec<Event>,
) -> bool {
    roll_reactive(
        encounter,
        reactor,
        resume.mover,
        rng,
        events,
        PendingKind::Reactive(resume.clone()),
    )
    .is_none()
}

/// `None` while Shield Block is waiting. A flat-check miss is a non-critical degree.
fn roll_reactive(
    encounter: &mut Encounter,
    reactor: CombatantId,
    target: CombatantId,
    rng: &mut impl Dice,
    events: &mut Vec<Event>,
    kind: PendingKind,
) -> Option<Degree> {
    encounter
        .spend_reaction(reactor)
        .expect("reactor has a ready reaction");
    release_cover(encounter, reactor);
    if let Some((natural, dc)) = failed_flat_check(encounter, reactor, target, rng) {
        events.push(Event::FlatCheckFailed {
            attacker: reactor,
            target,
            natural,
            dc,
            actions_remaining: encounter.actions_remaining,
        });
        return Some(Degree::Failure);
    }
    let rolled = roll_attack(encounter, reactor, target, 0, rng);
    if rolled.damage > 0 && shield_block_legal(encounter, target) {
        encounter.pending = Some(PendingShieldBlock {
            attacker: reactor,
            defender: target,
            rolled,
            kind,
        });
        return None;
    }
    let attack = commit_attack(encounter, reactor, target, rolled, false);
    let degree = attack.degree;
    publish_reactive(events, reactor, target, &attack);
    Some(degree)
}

fn resolve_step(
    encounter: &mut Encounter,
    id: CombatantId,
    destination: GridPos,
    events: &mut Vec<Event>,
) {
    release_cover(encounter, id);
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

fn resolve_raise_shield(encounter: &mut Encounter, id: CombatantId, events: &mut Vec<Event>) {
    if let Some(shield) = encounter.combatants.get_mut(&id).unwrap().shield.as_mut() {
        shield.raised = true;
    }
    encounter.actions_remaining -= 1;
    events.push(Event::ShieldRaised {
        id,
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
    release_cover(encounter, id);
    if let Some((natural, dc)) = failed_flat_check(encounter, id, target, rng) {
        encounter.actions_remaining -= 1;
        events.push(Event::FlatCheckFailed {
            attacker: id,
            target,
            natural,
            dc,
            actions_remaining: encounter.actions_remaining,
        });
        return;
    }
    let map = multiple_attack_penalty(encounter.strikes_this_turn);
    let rolled = roll_attack(encounter, id, target, map, rng);
    encounter.strikes_this_turn += 1;
    encounter.actions_remaining -= 1;
    if rolled.damage > 0 && shield_block_legal(encounter, target) {
        encounter.pending = Some(PendingShieldBlock {
            attacker: id,
            defender: target,
            rolled,
            kind: PendingKind::Strike,
        });
        return;
    }
    let attack = commit_attack(encounter, id, target, rolled, false);
    publish_strike(encounter, events, id, target, &attack);
}

/// Applies or declines the open Shield Block, then finishes the strike that was waiting.
pub fn resolve_pending_shield_block(
    encounter: &mut Encounter,
    accept: bool,
    rng: &mut impl Dice,
) -> Result<Vec<Event>, Illegal> {
    let Some(pending) = encounter.pending.clone() else {
        return Err(Illegal::NotLegal);
    };
    if accept && !shield_block_legal(encounter, pending.defender) {
        return Err(Illegal::NotLegal);
    }
    encounter.pending = None;
    let mut events = Vec::new();
    let attack = commit_attack(
        encounter,
        pending.attacker,
        pending.defender,
        pending.rolled,
        accept,
    );
    match pending.kind {
        PendingKind::Strike => {
            publish_strike(
                encounter,
                &mut events,
                pending.attacker,
                pending.defender,
                &attack,
            );
        }
        PendingKind::Reactive(resume) => {
            publish_reactive(&mut events, pending.attacker, pending.defender, &attack);
            if !encounter.combatants[&resume.mover].is_conscious() {
                let leaving = resume.squares[resume.index];
                encounter
                    .combatants
                    .get_mut(&resume.mover)
                    .unwrap()
                    .position = leaving;
                encounter.actions_remaining -= 1;
            } else {
                continue_move(encounter, resume, rng, &mut events);
            }
        }
        PendingKind::Manipulate(mut resume) => {
            publish_reactive(&mut events, pending.attacker, pending.defender, &attack);
            if attack.degree == Degree::CriticalSuccess
                || !encounter.combatants[&resume.actor].is_conscious()
            {
                resume.disrupted = true;
            }
            if encounter.combatants[&resume.actor].is_conscious() {
                continue_manipulate(encounter, resume, rng, &mut events);
            } else {
                apply_manipulate(encounter, &resume, &mut events);
            }
        }
    }
    if let Some(outcome) = encounter.detect_outcome() {
        if encounter.outcome.is_none() {
            encounter.outcome = Some(outcome);
            events.push(Event::EncounterEnded { outcome });
        }
    }
    Ok(events)
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
    shield: Option<ShieldBlockBlow>,
}

struct ShieldBlockBlow {
    prevented: i32,
    shield_hit_points: i32,
    broken: bool,
    destroyed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Harm {
    None,
    NowDying,
    Dead,
}

fn roll_attack(
    encounter: &Encounter,
    attacker: CombatantId,
    target: CombatantId,
    map: i32,
    rng: &mut impl Dice,
) -> RolledAttack {
    let armor_class = defense_ac(encounter, attacker, target);
    let (attack_bonus, formula, unconscious, prone) = {
        let actor = &encounter.combatants[&attacker];
        let defender = &encounter.combatants[&target];
        (
            actor.strike.attack_bonus,
            actor.strike.damage.clone(),
            defender.is_unconscious(),
            actor.prone,
        )
    };
    let natural = rng.d20();
    // Prone (−2) and the multiple attack penalty are circumstance penalties on this roll.
    // They do not stack. Off-guard stays on Armor Class.
    let map = if prone { map.min(-2) } else { map };
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
    RolledAttack {
        natural,
        attack_bonus,
        map,
        total,
        armor_class,
        degree,
        damage,
    }
}

fn commit_attack(
    encounter: &mut Encounter,
    _attacker: CombatantId,
    target: CombatantId,
    rolled: RolledAttack,
    accept_shield_block: bool,
) -> ResolvedAttack {
    let mut damage = rolled.damage;
    let mut shield = None;
    if accept_shield_block {
        encounter
            .spend_reaction(target)
            .expect("shield block was legal");
        let defender = encounter.combatants.get_mut(&target).unwrap();
        let item = defender.shield.as_mut().expect("shield block was legal");
        let remaining = (damage - item.hardness).max(0);
        let prevented = damage - remaining;
        item.hit_points = (item.hit_points - remaining).max(0);
        shield = Some(ShieldBlockBlow {
            prevented,
            shield_hit_points: item.hit_points,
            broken: item.is_broken(),
            destroyed: item.is_destroyed(),
        });
        damage = remaining;
    }
    let (target_hit_points, harm) = {
        let defender = encounter.combatants.get_mut(&target).unwrap();
        let harm = apply_harm(defender, rolled.degree, damage);
        (defender.hit_points, harm)
    };
    ResolvedAttack {
        natural: rolled.natural,
        attack_bonus: rolled.attack_bonus,
        map: rolled.map,
        total: rolled.total,
        armor_class: rolled.armor_class,
        degree: rolled.degree,
        damage,
        target_hit_points,
        harm,
        shield,
    }
}

fn publish_strike(
    encounter: &Encounter,
    events: &mut Vec<Event>,
    attacker: CombatantId,
    target: CombatantId,
    attack: &ResolvedAttack,
) {
    push_shield_block(events, target, attack);
    events.push(Event::StrikeResolved {
        attacker,
        target,
        natural: attack.natural,
        attack_bonus: attack.attack_bonus,
        map: attack.map,
        total: attack.total,
        armor_class: attack.armor_class,
        degree: attack.degree,
        damage: attack.damage,
        target_hit_points: attack.target_hit_points,
        actions_remaining: encounter.actions_remaining,
    });
    push_harm(events, target, attack.harm);
}

fn publish_reactive(
    events: &mut Vec<Event>,
    attacker: CombatantId,
    target: CombatantId,
    attack: &ResolvedAttack,
) {
    push_shield_block(events, target, attack);
    events.push(Event::ReactiveStrike {
        attacker,
        target,
        natural: attack.natural,
        attack_bonus: attack.attack_bonus,
        map: attack.map,
        total: attack.total,
        armor_class: attack.armor_class,
        degree: attack.degree,
        damage: attack.damage,
        target_hit_points: attack.target_hit_points,
    });
    push_harm(events, target, attack.harm);
}

fn push_shield_block(events: &mut Vec<Event>, defender: CombatantId, attack: &ResolvedAttack) {
    let Some(blow) = &attack.shield else {
        return;
    };
    events.push(Event::ShieldBlock {
        defender,
        prevented: blow.prevented,
        damage: attack.damage,
        shield_hit_points: blow.shield_hit_points,
        broken: blow.broken,
        destroyed: blow.destroyed,
    });
}

fn shield_block_legal(encounter: &Encounter, id: CombatantId) -> bool {
    let Some(combatant) = encounter.combatants.get(&id) else {
        return false;
    };
    if combatant.dead || !combatant.shield_block || !combatant.reaction_ready {
        return false;
    }
    combatant
        .shield
        .as_ref()
        .is_some_and(|shield| shield.raised && !shield.is_broken() && !shield.is_destroyed())
}

fn release_cover(encounter: &mut Encounter, id: CombatantId) {
    if let Some(combatant) = encounter.combatants.get_mut(&id) {
        combatant.taking_cover = false;
    }
}

/// `Some` when a Concealed or Hidden flat check fails. A pass, or no check, is `None`.
fn failed_flat_check(
    encounter: &Encounter,
    attacker: CombatantId,
    target: CombatantId,
    rng: &mut impl Dice,
) -> Option<(u8, i32)> {
    let dc = match encounter.detection(attacker, target) {
        Some(Detection::Concealed) => 5,
        Some(Detection::Hidden) => 11,
        Some(Detection::Undetected) | None => return None,
    };
    let natural = rng.d20();
    (i32::from(natural) < dc).then_some((natural, dc))
}

fn resolve_seek(
    encounter: &mut Encounter,
    id: CombatantId,
    rng: &mut impl Dice,
    events: &mut Vec<Event>,
) {
    let perception = encounter.combatants[&id].perception;
    let side = encounter.combatants[&id].side;
    let candidates: Vec<(CombatantId, i32)> = encounter
        .combatants
        .values()
        .filter(|other| other.side != side)
        .filter_map(|other| other.stealth_dc.map(|dc| (other.id, dc)))
        .collect();
    let targets: Vec<(CombatantId, i32)> = candidates
        .into_iter()
        .filter(|(target, _)| {
            matches!(
                encounter.detection(id, *target),
                Some(Detection::Hidden | Detection::Undetected)
            )
        })
        .collect();
    let mut rolls = Vec::new();
    for (target, dc) in targets {
        let natural = rng.d20();
        let total = i32::from(natural) + perception;
        let degree = degree_of_success(natural, total, dc);
        let state = encounter.detection(id, target);
        match degree {
            Degree::CriticalSuccess => {
                encounter.detection.remove(&(id, target));
            }
            Degree::Success => match state {
                Some(Detection::Undetected) => {
                    encounter.detection.insert((id, target), Detection::Hidden);
                }
                Some(Detection::Hidden) => {
                    encounter.detection.remove(&(id, target));
                }
                Some(Detection::Concealed) | None => {}
            },
            Degree::Failure | Degree::CriticalFailure => {}
        }
        rolls.push(SeekRoll {
            target,
            natural,
            total,
            dc,
            degree,
        });
    }
    encounter.actions_remaining -= 1;
    events.push(Event::Sought {
        id,
        rolls,
        actions_remaining: encounter.actions_remaining,
    });
}

fn resolve_point_out(
    encounter: &mut Encounter,
    id: CombatantId,
    target: CombatantId,
    events: &mut Vec<Event>,
) {
    let side = encounter.combatants[&id].side;
    let allies: Vec<CombatantId> = encounter
        .combatants
        .values()
        .filter(|other| other.id != id && other.side == side)
        .map(|other| other.id)
        .collect();
    let allies: Vec<CombatantId> = allies
        .into_iter()
        .filter(|ally| encounter.detection(*ally, target) == Some(Detection::Undetected))
        .collect();
    for ally in allies {
        encounter
            .detection
            .insert((ally, target), Detection::Hidden);
    }
    encounter.actions_remaining -= 1;
    events.push(Event::PointedOut {
        id,
        target,
        actions_remaining: encounter.actions_remaining,
    });
}

fn resolve_take_cover(encounter: &mut Encounter, id: CombatantId, events: &mut Vec<Event>) {
    encounter.combatants.get_mut(&id).unwrap().taking_cover = true;
    encounter.actions_remaining -= 1;
    events.push(Event::TookCover {
        id,
        actions_remaining: encounter.actions_remaining,
    });
}

fn resolve_manipulate(
    encounter: &mut Encounter,
    id: CombatantId,
    kind: ManipKind,
    rng: &mut impl Dice,
    events: &mut Vec<Event>,
) {
    // The same reactors as leaving this square. Release and Interact do not also move.
    let reactors = reactors_for_left_square(encounter, id, encounter.combatants[&id].position);
    continue_manipulate(
        encounter,
        ManipulateResume {
            actor: id,
            kind,
            reactors,
            next: 0,
            disrupted: false,
        },
        rng,
        events,
    );
}

fn continue_manipulate(
    encounter: &mut Encounter,
    mut resume: ManipulateResume,
    rng: &mut impl Dice,
    events: &mut Vec<Event>,
) {
    while resume.next < resume.reactors.len() {
        let reactor = resume.reactors[resume.next];
        resume.next += 1;
        let degree = roll_reactive(
            encounter,
            reactor,
            resume.actor,
            rng,
            events,
            PendingKind::Manipulate(resume.clone()),
        );
        let Some(degree) = degree else {
            return;
        };
        // A critical success disrupts the manipulate action. Reaching 0 HP does too.
        // The same reaction does not cancel Stride, Step, Crawl, or Leap.
        if degree == Degree::CriticalSuccess {
            resume.disrupted = true;
        }
        if !encounter.combatants[&resume.actor].is_conscious() {
            resume.disrupted = true;
            break;
        }
    }
    apply_manipulate(encounter, &resume, events);
}

fn apply_manipulate(encounter: &mut Encounter, resume: &ManipulateResume, events: &mut Vec<Event>) {
    let actor = resume.actor;
    let cancelled = resume.disrupted || !encounter.combatants[&actor].is_conscious();
    match resume.kind {
        ManipKind::Release(item) => {
            if cancelled {
                return;
            }
            drop_held(encounter, actor, item);
            events.push(Event::Released {
                id: actor,
                item,
                actions_remaining: encounter.actions_remaining,
            });
        }
        ManipKind::Interact => {
            let item = if cancelled {
                None
            } else {
                Some(take_dropped(encounter, actor))
            };
            // The action is spent even when a critical hit or 0 HP cancelled the pickup.
            encounter.actions_remaining -= 1;
            if let Some(item) = item {
                events.push(Event::PickedUp {
                    id: actor,
                    item,
                    actions_remaining: encounter.actions_remaining,
                });
            }
        }
    }
}

fn free_releases(encounter: &Encounter, id: CombatantId) -> Vec<Action> {
    let mut actions = Vec::new();
    for item in [Gear::Shield, Gear::Weapon] {
        if can_release(encounter, id, item) {
            actions.push(Action::Release { item });
        }
    }
    actions
}

fn can_release(encounter: &Encounter, id: CombatantId, item: Gear) -> bool {
    let Some(actor) = encounter.combatants.get(&id) else {
        return false;
    };
    let held = match item {
        Gear::Weapon => actor.holding_weapon,
        Gear::Shield => actor.shield.is_some(),
    };
    if !held {
        return false;
    }
    let dropped = encounter.dropped.get(&actor.position);
    match item {
        Gear::Weapon => !dropped.is_some_and(|dropped| dropped.weapon),
        Gear::Shield => !dropped.is_some_and(|dropped| dropped.shield.is_some()),
    }
}

/// Shield when one is in the square, otherwise the weapon. `None` when that kind is already held.
fn interact_item(encounter: &Encounter, id: CombatantId) -> Option<Gear> {
    let actor = encounter.combatants.get(&id)?;
    let dropped = encounter.dropped.get(&actor.position)?;
    if dropped.shield.is_some() {
        if actor.shield.is_some() {
            return None;
        }
        return Some(Gear::Shield);
    }
    if dropped.weapon && !actor.holding_weapon {
        return Some(Gear::Weapon);
    }
    None
}

fn drop_held(encounter: &mut Encounter, id: CombatantId, item: Gear) {
    let pos = encounter.combatants[&id].position;
    match item {
        Gear::Weapon => {
            encounter.combatants.get_mut(&id).unwrap().holding_weapon = false;
            encounter.dropped.entry(pos).or_default().weapon = true;
        }
        Gear::Shield => {
            let mut shield = encounter
                .combatants
                .get_mut(&id)
                .unwrap()
                .shield
                .take()
                .expect("release requires a held shield");
            shield.raised = false;
            encounter.dropped.entry(pos).or_default().shield = Some(shield);
        }
    }
}

fn take_dropped(encounter: &mut Encounter, id: CombatantId) -> Gear {
    let item = interact_item(encounter, id).expect("interact requires a dropped object");
    let pos = encounter.combatants[&id].position;
    match item {
        Gear::Shield => {
            let mut shield = encounter
                .dropped
                .get_mut(&pos)
                .unwrap()
                .shield
                .take()
                .expect("the square has a shield");
            shield.raised = false;
            encounter.combatants.get_mut(&id).unwrap().shield = Some(shield);
        }
        Gear::Weapon => {
            encounter.dropped.get_mut(&pos).unwrap().weapon = false;
            encounter.combatants.get_mut(&id).unwrap().holding_weapon = true;
        }
    }
    let empty = encounter
        .dropped
        .get(&pos)
        .is_some_and(|dropped| !dropped.weapon && dropped.shield.is_none());
    if empty {
        encounter.dropped.remove(&pos);
    }
    item
}

fn take_cover_legal(encounter: &Encounter, id: CombatantId) -> bool {
    let actor = &encounter.combatants[&id];
    let prone = actor.prone;
    let position = actor.position;
    let side = actor.side;
    if prone || beside_unwalkable(&encounter.grid, position) {
        return true;
    }
    let enemies: Vec<CombatantId> = encounter
        .combatants
        .values()
        .filter(|enemy| enemy.id != id && enemy.side != side && !enemy.dead)
        .map(|enemy| enemy.id)
        .collect();
    enemies
        .into_iter()
        .any(|enemy| derived_cover_bonus(encounter, enemy, id) > 0)
}

fn point_out_targets(encounter: &Encounter, actor: CombatantId) -> Vec<CombatantId> {
    let Some(side) = encounter
        .combatants
        .get(&actor)
        .map(|combatant| combatant.side)
    else {
        return Vec::new();
    };
    let ids: Vec<CombatantId> = encounter.combatants.keys().copied().collect();
    let mut targets: Vec<CombatantId> = ids
        .iter()
        .copied()
        .filter(|subject| {
            *subject != actor
                && encounter.detection(actor, *subject).is_none()
                && ids.iter().any(|ally| {
                    *ally != actor
                        && encounter.combatants[ally].side == side
                        && encounter.detection(*ally, *subject) == Some(Detection::Undetected)
                })
        })
        .collect();
    targets.sort();
    targets
}

fn beside_unwalkable(grid: &Grid, pos: GridPos) -> bool {
    squares_within_feet(pos, 5, grid.width(), grid.height())
        .into_iter()
        .any(|near| !grid.walkable(near))
}

/// Cover from `attacker` to `target`, after Take Cover's one-step upgrade.
fn cover_bonus(encounter: &Encounter, attacker: CombatantId, target: CombatantId) -> i32 {
    let derived = derived_cover_bonus(encounter, attacker, target);
    let defender = &encounter.combatants[&target];
    if !defender.taking_cover {
        return derived;
    }
    // Nothing becomes lesser only while prone or beside a wall. The flag does not record which.
    match derived {
        0 if defender.prone || beside_unwalkable(&encounter.grid, defender.position) => 1,
        0 => 0,
        1 => 2,
        _ => 4,
    }
}

fn derived_cover_bonus(encounter: &Encounter, attacker: CombatantId, target: CombatantId) -> i32 {
    let from = encounter.combatants[&attacker].position;
    let to = encounter.combatants[&target].position;
    let lesser = encounter.combatants.values().any(|other| {
        other.id != attacker && other.id != target && square_on_segment(from, to, other.position)
    });
    let standard = {
        let grid = &encounter.grid;
        let mut found = false;
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                let pos = GridPos::new(x, y);
                if !grid.walkable(pos) && square_on_segment(from, to, pos) {
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
        }
        found
    };
    if standard {
        2
    } else if lesser {
        1
    } else {
        0
    }
}

fn square_on_segment(from: GridPos, to: GridPos, square: GridPos) -> bool {
    if square == from || square == to {
        return false;
    }
    // Same center-line test as flanking, with the intervening square in the middle.
    centers_cross_opposite(from, square, to)
}

fn defense_ac(encounter: &Encounter, attacker: CombatantId, target: CombatantId) -> i32 {
    let defender = &encounter.combatants[&target];
    let mut armor_class = defender.armor_class;
    let shield_bonus = if defender
        .shield
        .as_ref()
        .is_some_and(|shield| shield.raised && !shield.is_broken() && !shield.is_destroyed())
    {
        2
    } else {
        0
    };
    // The shield and cover are both circumstance bonuses, so only the larger applies.
    armor_class += shield_bonus.max(cover_bonus(encounter, attacker, target));
    if off_guard(encounter, attacker, target) {
        armor_class -= 2;
    }
    armor_class
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

/// Conscious creatures other than `id`. A leap cannot enter or land on these squares.
fn creature_squares(encounter: &Encounter, id: CombatantId) -> HashSet<GridPos> {
    encounter
        .combatants
        .values()
        .filter(|other| other.id != id && other.is_conscious())
        .map(|other| other.position)
        .collect()
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
    if encounter.pending.is_some() {
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
    if encounter.pending.is_some() {
        return Err(Illegal::NotLegal);
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
        {
            let combatant = encounter.combatants.get_mut(&id).unwrap();
            combatant.reaction_ready = true;
            combatant.taking_cover = false;
            if let Some(shield) = combatant.shield.as_mut() {
                shield.raised = false;
            }
        }
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
