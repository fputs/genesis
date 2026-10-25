//! 5-foot grid measurement for Stride, Step, reach, and range.
//!
//! Callers pass squares, Speed, terrain, and occupants in. The diagonal counter is turn
//! state: it is not a field of a creature or of combatant state.
//!
//! Player Core, "Grid Movement".

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "callers are this crate's tests; the facade keeps the procedure and drops the root export"
    )
)]

/// One square on the grid. Coordinates increase east and south.
///
/// Player Core, "Grid Movement". A square is 5 feet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Square {
    /// Column. Negative values are off the map the caller supplied.
    pub x: i32,
    /// Row. Negative values are off the map the caller supplied.
    pub y: i32,
}

/// Terrain that changes the cost of entering a square.
///
/// Player Core, "Difficult Terrain".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Terrain {
    /// Entering costs the base only. Player Core, "Grid Movement".
    Open,
    /// Entering costs 5 extra feet. Player Core, "Difficult Terrain".
    Difficult,
    /// Entering costs 10 extra feet. Player Core, "Difficult Terrain".
    GreaterDifficult,
}

/// Creature size. Movement accepts only Small and Medium.
///
/// Player Core, "Size, Space, and Reach".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    /// Space under 5 feet. Player Core, "Size, Space, and Reach".
    Tiny,
    /// 5-foot space. Player Core, "Size, Space, and Reach".
    Small,
    /// 5-foot space. Player Core, "Size, Space, and Reach".
    Medium,
    /// 10-foot space. Player Core, "Size, Space, and Reach".
    Large,
    /// 15-foot space. Player Core, "Size, Space, and Reach".
    Huge,
    /// At least 20 feet. Player Core, "Size, Space, and Reach".
    Gargantuan,
}

/// How much space a size occupies.
///
/// Player Core, "Size, Space, and Reach".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Space {
    /// Less than 5 feet. Player Core, "Size, Space, and Reach".
    UnderFive,
    /// Exactly this many feet. Player Core, "Size, Space, and Reach".
    Exact(u32),
    /// At least this many feet. Player Core, "Size, Space, and Reach".
    AtLeast(u32),
}

/// Space and the two reaches for one size.
///
/// Player Core, "Size, Space, and Reach". Tall reach and long reach are separate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SizeData {
    /// Space the size occupies. Player Core, "Size, Space, and Reach".
    pub space: Space,
    /// Reach in feet when the creature is tall. Player Core, "Size, Space, and Reach".
    pub reach_tall_feet: u32,
    /// Reach in feet when the creature is long. Player Core, "Size, Space, and Reach".
    pub reach_long_feet: u32,
}

/// Phase 1 stand-in for a willing creature. Equal ids may be crossed.
///
/// Player Core, "Moving Through a Creature's Space". Willing is same faction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FactionId(pub u32);

/// Another creature already on a square. The mover is not in this list.
///
/// Player Core, "Moving Through a Creature's Space".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Occupant {
    /// Square that creature occupies. Player Core, "Moving Through a Creature's Space".
    pub square: Square,
    /// Same id as the mover may be crossed. Player Core, "Moving Through a Creature's Space".
    pub faction: FactionId,
}

/// Map size in squares. The caller supplies both dimensions.
///
/// Player Core, "Grid Movement".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapBounds {
    /// Squares from the origin along x. A square's x must be strictly below this.
    pub width: u32,
    /// Squares from the origin along y. A square's y must be strictly below this.
    pub height: u32,
}

/// Diagonal squares entered during one creature's turn.
///
/// Not stored on a creature. Orthogonal movement does not change it.
/// `fresh` makes the next diagonal cost 5 feet. `end_turn` returns that same start.
///
/// Player Core, "Diagonal Movement".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiagonalCounter {
    diagonals: u32,
}

impl DiagonalCounter {
    /// The next diagonal costs 5 feet.
    ///
    /// Player Core, "Diagonal Movement".
    pub const fn fresh() -> Self {
        Self { diagonals: 0 }
    }

    /// Resets the count so the next diagonal costs 5 feet.
    ///
    /// Player Core, "Diagonal Movement". The count resets at end of turn.
    pub const fn end_turn(self) -> Self {
        Self::fresh()
    }

    /// How many diagonal squares this counter has recorded.
    ///
    /// Player Core, "Diagonal Movement".
    pub const fn diagonals(self) -> u32 {
        self.diagonals
    }

    fn after_diagonal(self) -> Result<Self, GridError> {
        match self.diagonals.checked_add(1) {
            Some(diagonals) => Ok(Self { diagonals }),
            None => Err(GridError::Overflow),
        }
    }
}

/// One square a successful move left or entered.
///
/// Player Core, "Stride". The events follow the caller's path in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SquareEvent {
    /// The creature left this square. Player Core, "Stride".
    Left {
        /// The square the creature occupied before the step.
        square: Square,
    },
    /// The creature entered this square. Player Core, "Stride".
    Entered {
        /// The square the creature entered.
        square: Square,
    },
}

/// A Stride or Step that was accepted.
///
/// `triggers` covers only this result. Stand, Drop Prone, and Crawl also trigger
/// Reactive Strike elsewhere (FIX-06). No reaction is resolved here.
///
/// Player Core, "Stride" and "Step".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoveOutcome {
    /// Feet this action spent moving. Player Core, "Stride". Step is always 5.
    pub cost_feet: u32,
    /// Square occupied after the move. Player Core, "Grid Movement".
    pub end: Square,
    /// Leave and enter events in path order. Player Core, "Stride".
    pub events: Vec<SquareEvent>,
    /// Counter after this action. Player Core, "Diagonal Movement".
    pub counter: DiagonalCounter,
    /// True for Stride. False for Step. Player Core, "Step" triggers no reactions.
    pub triggers: bool,
}

/// Everything a Stride or Step needs. The path is the caller's ordered squares.
///
/// Player Core, "Stride". Nothing in this module searches for a route.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MoveInput<'a> {
    /// Square the creature occupies before the action. Player Core, "Grid Movement".
    pub start: Square,
    /// Squares to enter, in order. Empty is a Stride that does not move.
    pub path: &'a [Square],
    /// Speed in feet. Copied in by the caller. Player Core, "Speed".
    pub speed_feet: i32,
    /// Creature size. Only Small and Medium can move. Player Core, "Size, Space, and Reach".
    pub size: Size,
    /// When true, Stride and Step are illegal. Player Core, "Immobilized".
    pub immobilized: bool,
    /// The mover's faction. Player Core, "Moving Through a Creature's Space".
    pub faction: FactionId,
    /// Map width and height in squares. Player Core, "Grid Movement".
    pub bounds: MapBounds,
    /// Terrain listings. The first listing of a square wins. Unlisted squares are open.
    pub terrain: &'a [(Square, Terrain)],
    /// Other creatures. The mover is not included. Player Core, "Moving Through a Creature's Space".
    pub occupants: &'a [Occupant],
    /// This turn's diagonal count, taken by value and returned on success.
    pub counter: DiagonalCounter,
}

/// Increment count and its untyped penalty. Not applied to an attack roll.
///
/// Player Core, "Range Penalty".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RangeBand {
    /// How many increments the distance fills, from 1 through 6. Player Core, "Range Penalty".
    pub increments: u32,
    /// Untyped penalty, −2 per increment after the first. Player Core, "Range Penalty".
    pub penalty: i32,
}

/// Why a grid measurement was rejected. The caller keeps its start square and counter.
///
/// Player Core, "Grid Movement".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridError {
    /// The path costs more than Speed. Player Core, "Stride".
    ExceedsSpeed,
    /// A non-negative Speed was not a multiple of 5. Defensive guard for validation (DEV-21).
    SpeedNotMultipleOfFive,
    /// Speed was below 0. Player Core, "Speed".
    NegativeSpeed,
    /// The path ends on a same-faction creature. Player Core, "Moving Through a Creature's Space".
    OccupiedEnd,
    /// The path enters another faction's square. Player Core, "Moving Through a Creature's Space".
    EnemySquare,
    /// Step would enter difficult or greater difficult terrain. Player Core, "Step".
    StepIntoDifficultTerrain,
    /// Step requires Speed of at least 10 feet. Player Core, "Step".
    StepRequiresTenFeet,
    /// Step's path was not exactly one square. Player Core, "Step".
    StepNotOneSquare,
    /// A path square was not adjacent to the previous one. Player Core, "Grid Movement".
    PathNotAdjacent,
    /// A square was outside the caller-supplied map. Player Core, "Grid Movement".
    OffGrid,
    /// A coordinate or a cost did not fit. Player Core, "Grid Movement".
    Overflow,
    /// The immobilized flag blocks a move action. Player Core, "Immobilized".
    Immobilized,
    /// The size is not Small or Medium. Player Core, "Size, Space, and Reach".
    UnsupportedSize,
    /// The range increment was 0 or negative. Player Core, "Range Penalty".
    InvalidRangeIncrement,
    /// The distance was negative. Player Core, "Range Penalty".
    NegativeDistance,
    /// The distance is past the sixth range increment. Player Core, "Range Penalty".
    BeyondRange,
}

const SQUARE_FEET: u32 = 5;
const LONG_DIAGONAL_FEET: u32 = 10;
const DIFFICULT_EXTRA_FEET: u32 = 5;
const GREATER_DIFFICULT_EXTRA_FEET: u32 = 10;
const STEP_DISTANCE_FEET: u32 = 5;
const STEP_MIN_SPEED_FEET: i32 = 10;
const MAX_RANGE_INCREMENTS: u32 = 6;
const RANGE_PENALTY_STEP: i32 = -2;

/// Space and reach for every size. Does not decide whether that size can move.
///
/// Player Core, "Size, Space, and Reach".
pub(crate) fn size_data(size: Size) -> SizeData {
    match size {
        Size::Tiny => SizeData {
            space: Space::UnderFive,
            reach_tall_feet: 0,
            reach_long_feet: 0,
        },
        Size::Small | Size::Medium => SizeData {
            space: Space::Exact(5),
            reach_tall_feet: 5,
            reach_long_feet: 5,
        },
        Size::Large => SizeData {
            space: Space::Exact(10),
            reach_tall_feet: 10,
            reach_long_feet: 5,
        },
        Size::Huge => SizeData {
            space: Space::Exact(15),
            reach_tall_feet: 15,
            reach_long_feet: 10,
        },
        Size::Gargantuan => SizeData {
            space: Space::AtLeast(20),
            reach_tall_feet: 20,
            reach_long_feet: 15,
        },
    }
}

/// Accepts 0 and every positive multiple of 5. A negative Speed is [`GridError::NegativeSpeed`].
///
/// add-modifiers validates Creature Speed at load and at fold. This guard does not change that.
/// Rule: Player Core has no multiple-of-5 rule. validation (DEV-21). Rules 2347 (Player Core
/// pg. 420) and Rules 2356 (5-ft squares).
pub(crate) fn validate_speed(speed_feet: i32) -> Result<(), GridError> {
    if speed_feet < 0 {
        return Err(GridError::NegativeSpeed);
    }
    if speed_feet % 5 != 0 {
        return Err(GridError::SpeedNotMultipleOfFive);
    }
    Ok(())
}

/// `Ok(())` when a move-trait action is legal. [`GridError::Immobilized`] when the flag is set.
///
/// The check takes no path, so a later move action can call it. Crawl, Stand, and Drop Prone
/// are not implemented.
///
/// Player Core, "Immobilized".
pub(crate) fn move_trait_allowed(immobilized: bool) -> Result<(), GridError> {
    if immobilized {
        Err(GridError::Immobilized)
    } else {
        Ok(())
    }
}

/// Stride along an explicit path, up to Speed.
///
/// On error the caller keeps the start square and the counter it passed in.
/// An empty path costs 0 feet, emits no events, and ends on the start square.
///
/// Player Core, "Stride".
pub(crate) fn stride(input: MoveInput<'_>) -> Result<MoveOutcome, GridError> {
    resolve(input, Action::Stride)
}

/// Step into exactly one adjacent square that is not difficult terrain.
///
/// The distance is 5 feet even when the diagonal count would charge 10, and that 5 is not
/// compared with Speed. A diagonal Step still advances the counter. Speed must be at least 10.
///
/// Player Core, "Step".
pub(crate) fn step(input: MoveInput<'_>) -> Result<MoveOutcome, GridError> {
    resolve(input, Action::Step)
}

/// Grid cost between two Small or Medium squares, alternating from 5 feet.
///
/// Ignores terrain, other creatures, and the turn's diagonal counter. The same square is 0.
/// The two directions match.
///
/// Player Core, "Areas".
pub(crate) fn distance(
    from: Square,
    to: Square,
    from_size: Size,
    to_size: Size,
) -> Result<u32, GridError> {
    ensure_supported(from_size)?;
    ensure_supported(to_size)?;
    let delta = absolute_delta(from, to)?;
    grid_feet(delta)
}

/// Squares that share an edge or a corner. The same square is not adjacent.
///
/// Player Core, "Range and Reach".
pub(crate) fn adjacent(
    from: Square,
    to: Square,
    from_size: Size,
    to_size: Size,
) -> Result<bool, GridError> {
    ensure_supported(from_size)?;
    ensure_supported(to_size)?;
    let delta = absolute_delta(from, to)?;
    Ok(is_adjacent(delta))
}

/// Whether `to` is inside the reach of a creature on `from`.
///
/// The same square is never in reach. Reach 5 is adjacency only. Reach 10 also includes a
/// creature exactly two diagonal squares away. Reach above 10 is the grid distance only.
///
/// Player Core, "Range and Reach".
pub(crate) fn in_reach(
    from: Square,
    to: Square,
    from_size: Size,
    to_size: Size,
    reach_feet: u32,
) -> Result<bool, GridError> {
    ensure_supported(from_size)?;
    ensure_supported(to_size)?;
    if from == to {
        return Ok(false);
    }
    let delta = absolute_delta(from, to)?;
    let feet = grid_feet(delta)?;
    if reach_feet == SQUARE_FEET {
        return Ok(is_adjacent(delta));
    }
    if reach_feet == LONG_DIAGONAL_FEET {
        let two_diagonals = delta.ax == 2 && delta.ay == 2;
        return Ok(feet <= LONG_DIAGONAL_FEET || two_diagonals);
    }
    if reach_feet > LONG_DIAGONAL_FEET {
        return Ok(feet <= reach_feet);
    }
    Ok(false)
}

/// Increment count and untyped penalty for a distance. Does not roll an attack.
///
/// The count is the distance divided by the increment, rounded up, and at least 1.
/// An exact multiple stays in that count. A count above 6 is [`GridError::BeyondRange`].
///
/// Player Core, "Range Penalty".
pub(crate) fn range_band(distance_feet: i32, increment_feet: i32) -> Result<RangeBand, GridError> {
    if distance_feet < 0 {
        return Err(GridError::NegativeDistance);
    }
    if increment_feet <= 0 {
        return Err(GridError::InvalidRangeIncrement);
    }
    let distance = u32::try_from(distance_feet).map_err(|_| GridError::Overflow)?;
    let increment = u32::try_from(increment_feet).map_err(|_| GridError::Overflow)?;
    let increments = if distance == 0 {
        1
    } else {
        distance.div_ceil(increment)
    };
    if increments > MAX_RANGE_INCREMENTS {
        return Err(GridError::BeyondRange);
    }
    let extras = increments.checked_sub(1).ok_or(GridError::Overflow)?;
    let extras = i32::try_from(extras).map_err(|_| GridError::Overflow)?;
    let penalty = extras
        .checked_mul(RANGE_PENALTY_STEP)
        .ok_or(GridError::Overflow)?;
    Ok(RangeBand {
        increments,
        penalty,
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    Stride,
    Step,
}

#[derive(Clone, Copy)]
struct Measured {
    square: Square,
    ax: i32,
    ay: i32,
}

#[derive(Clone, Copy)]
struct AbsDelta {
    ax: i32,
    ay: i32,
}

fn resolve(input: MoveInput<'_>, action: Action) -> Result<MoveOutcome, GridError> {
    ensure_supported(input.size)?;
    move_trait_allowed(input.immobilized)?;
    if input.speed_feet < 0 {
        return Err(GridError::NegativeSpeed);
    }
    validate_speed(input.speed_feet)?;

    if action == Action::Step {
        if input.speed_feet < STEP_MIN_SPEED_FEET {
            return Err(GridError::StepRequiresTenFeet);
        }
        if input.path.len() != 1 {
            return Err(GridError::StepNotOneSquare);
        }
    }

    ensure_on_grid(input.start, input.path, input.bounds)?;
    let measured = measure(input.start, input.path)?;
    ensure_adjacent(&measured)?;
    ensure_no_enemy(&measured, input.occupants, input.faction)?;

    if action == Action::Step {
        ensure_step_terrain(&measured, input.terrain)?;
    }

    ensure_end_clear(&measured, input.occupants, input.faction)?;

    let (cost_feet, counter) = match action {
        Action::Stride => stride_cost(&measured, input.terrain, input.counter)?,
        Action::Step => step_cost(&measured, input.counter)?,
    };

    if action == Action::Stride {
        within_speed(cost_feet, input.speed_feet)?;
    }

    let end = match measured.last() {
        Some(step) => step.square,
        None => input.start,
    };

    Ok(MoveOutcome {
        cost_feet,
        end,
        events: leave_and_enter(input.start, &measured),
        counter,
        triggers: action == Action::Stride,
    })
}

fn ensure_supported(size: Size) -> Result<(), GridError> {
    match size {
        Size::Small | Size::Medium => Ok(()),
        Size::Tiny | Size::Large | Size::Huge | Size::Gargantuan => Err(GridError::UnsupportedSize),
    }
}

fn ensure_on_grid(start: Square, path: &[Square], bounds: MapBounds) -> Result<(), GridError> {
    if !inside(start, bounds) {
        return Err(GridError::OffGrid);
    }
    for square in path {
        if !inside(*square, bounds) {
            return Err(GridError::OffGrid);
        }
    }
    Ok(())
}

fn inside(square: Square, bounds: MapBounds) -> bool {
    match (u32::try_from(square.x), u32::try_from(square.y)) {
        (Ok(x), Ok(y)) => x < bounds.width && y < bounds.height,
        _ => false,
    }
}

fn measure(start: Square, path: &[Square]) -> Result<Vec<Measured>, GridError> {
    let mut steps = Vec::new();
    let mut previous = start;
    for square in path {
        let dx = square
            .x
            .checked_sub(previous.x)
            .ok_or(GridError::Overflow)?;
        let dy = square
            .y
            .checked_sub(previous.y)
            .ok_or(GridError::Overflow)?;
        // `i32::MIN` has no absolute value in `i32`.
        let ax = dx.checked_abs().ok_or(GridError::Overflow)?;
        let ay = dy.checked_abs().ok_or(GridError::Overflow)?;
        steps.push(Measured {
            square: *square,
            ax,
            ay,
        });
        previous = *square;
    }
    Ok(steps)
}

fn ensure_adjacent(steps: &[Measured]) -> Result<(), GridError> {
    for step in steps {
        if !is_adjacent(AbsDelta {
            ax: step.ax,
            ay: step.ay,
        }) {
            return Err(GridError::PathNotAdjacent);
        }
    }
    Ok(())
}

fn is_adjacent(delta: AbsDelta) -> bool {
    delta.ax.max(delta.ay) == 1
}

fn is_diagonal(step: Measured) -> bool {
    step.ax == 1 && step.ay == 1
}

fn ensure_no_enemy(
    steps: &[Measured],
    occupants: &[Occupant],
    faction: FactionId,
) -> Result<(), GridError> {
    for step in steps {
        if presence(occupants, step.square, faction) == Presence::Enemy {
            return Err(GridError::EnemySquare);
        }
    }
    Ok(())
}

fn ensure_step_terrain(steps: &[Measured], terrain: &[(Square, Terrain)]) -> Result<(), GridError> {
    let Some(destination) = steps.first() else {
        return Err(GridError::StepNotOneSquare);
    };
    match terrain_at(terrain, destination.square) {
        Terrain::Open => Ok(()),
        Terrain::Difficult | Terrain::GreaterDifficult => Err(GridError::StepIntoDifficultTerrain),
    }
}

fn ensure_end_clear(
    steps: &[Measured],
    occupants: &[Occupant],
    faction: FactionId,
) -> Result<(), GridError> {
    let Some(last) = steps.last() else {
        return Ok(());
    };
    if presence(occupants, last.square, faction) == Presence::Ally {
        Err(GridError::OccupiedEnd)
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Presence {
    Empty,
    Ally,
    Enemy,
}

fn presence(occupants: &[Occupant], square: Square, faction: FactionId) -> Presence {
    let mut ally = false;
    for occupant in occupants {
        if occupant.square != square {
            continue;
        }
        if occupant.faction != faction {
            return Presence::Enemy;
        }
        ally = true;
    }
    if ally {
        Presence::Ally
    } else {
        Presence::Empty
    }
}

fn terrain_at(listings: &[(Square, Terrain)], square: Square) -> Terrain {
    for (listed, terrain) in listings {
        if *listed == square {
            return *terrain;
        }
    }
    Terrain::Open
}

fn stride_cost(
    steps: &[Measured],
    terrain: &[(Square, Terrain)],
    mut counter: DiagonalCounter,
) -> Result<(u32, DiagonalCounter), GridError> {
    let mut total = 0u32;
    for step in steps {
        let base = if is_diagonal(*step) {
            let base = if counter.diagonals().is_multiple_of(2) {
                SQUARE_FEET
            } else {
                LONG_DIAGONAL_FEET
            };
            counter = counter.after_diagonal()?;
            base
        } else {
            SQUARE_FEET
        };
        let extra = match terrain_at(terrain, step.square) {
            Terrain::Open => 0,
            Terrain::Difficult => DIFFICULT_EXTRA_FEET,
            Terrain::GreaterDifficult => GREATER_DIFFICULT_EXTRA_FEET,
        };
        let cost = base.checked_add(extra).ok_or(GridError::Overflow)?;
        total = total.checked_add(cost).ok_or(GridError::Overflow)?;
    }
    Ok((total, counter))
}

fn step_cost(
    steps: &[Measured],
    counter: DiagonalCounter,
) -> Result<(u32, DiagonalCounter), GridError> {
    let Some(destination) = steps.first() else {
        return Err(GridError::StepNotOneSquare);
    };
    let counter = if is_diagonal(*destination) {
        counter.after_diagonal()?
    } else {
        counter
    };
    Ok((STEP_DISTANCE_FEET, counter))
}

fn within_speed(cost_feet: u32, speed_feet: i32) -> Result<(), GridError> {
    let speed = u32::try_from(speed_feet).map_err(|_| GridError::Overflow)?;
    if cost_feet > speed {
        Err(GridError::ExceedsSpeed)
    } else {
        Ok(())
    }
}

fn leave_and_enter(start: Square, steps: &[Measured]) -> Vec<SquareEvent> {
    let mut events = Vec::new();
    let mut current = start;
    for step in steps {
        events.push(SquareEvent::Left { square: current });
        events.push(SquareEvent::Entered {
            square: step.square,
        });
        current = step.square;
    }
    events
}

fn absolute_delta(from: Square, to: Square) -> Result<AbsDelta, GridError> {
    let dx = to.x.checked_sub(from.x).ok_or(GridError::Overflow)?;
    let dy = to.y.checked_sub(from.y).ok_or(GridError::Overflow)?;
    // `i32::MIN` has no absolute value in `i32`.
    let ax = dx.checked_abs().ok_or(GridError::Overflow)?;
    let ay = dy.checked_abs().ok_or(GridError::Overflow)?;
    Ok(AbsDelta { ax, ay })
}

fn grid_feet(delta: AbsDelta) -> Result<u32, GridError> {
    let diagonals = delta.ax.min(delta.ay);
    let orthogonals = delta
        .ax
        .max(delta.ay)
        .checked_sub(diagonals)
        .ok_or(GridError::Overflow)?;
    let diagonals = u32::try_from(diagonals).map_err(|_| GridError::Overflow)?;
    let orthogonals = u32::try_from(orthogonals).map_err(|_| GridError::Overflow)?;
    let pairs = diagonals / 2;
    let odd = diagonals % 2;
    let diagonal_cost = pairs.checked_mul(15).ok_or(GridError::Overflow)?;
    let odd_cost = odd.checked_mul(SQUARE_FEET).ok_or(GridError::Overflow)?;
    let orthogonal_cost = orthogonals
        .checked_mul(SQUARE_FEET)
        .ok_or(GridError::Overflow)?;
    diagonal_cost
        .checked_add(odd_cost)
        .ok_or(GridError::Overflow)?
        .checked_add(orthogonal_cost)
        .ok_or(GridError::Overflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BuildParts, CharacterBuild, CombatantId, DieCount, Faces, MeleeStrike, ReachFeet,
        ScriptedDice, derive, new_combatant_state,
    };

    const ALLY: FactionId = FactionId(1);
    const FOE: FactionId = FactionId(2);

    fn at(x: i32, y: i32) -> Square {
        Square { x, y }
    }

    fn input<'a>(
        start: Square,
        path: &'a [Square],
        speed_feet: i32,
        size: Size,
        counter: DiagonalCounter,
    ) -> MoveInput<'a> {
        MoveInput {
            start,
            path,
            speed_feet,
            size,
            immobilized: false,
            faction: ALLY,
            bounds: MapBounds {
                width: 10,
                height: 10,
            },
            terrain: &[],
            occupants: &[],
            counter,
        }
    }

    fn line(from_x: i32, count: i32) -> Vec<Square> {
        (1..=count).map(|step| at(from_x + step, 0)).collect()
    }

    fn creature() -> crate::Creature {
        let parts = BuildParts {
            hit_points: 20,
            armor_class: 18,
            perception: 5,
            speed_feet: 25,
            strike: MeleeStrike {
                attack_bonus: 7,
                damage_dice: DieCount::new(1).expect("one die"),
                damage_faces: Faces::new(8).expect("d8"),
                damage_bonus: 4,
                reach_feet: ReachFeet::new(5).expect("reach 5"),
                agile: false,
            },
            shield: None,
            stealth_dc: None,
            aid_modifier: None,
            aid_rank: None,
            uses_dying_rules: None,
            saves: None,
        };
        derive(&CharacterBuild::try_new(parts).expect("valid build"))
    }

    /// Player Core, "Grid Movement".
    #[test]
    fn the_same_stride_twice_matches() {
        let dice = ScriptedDice::new([8]);
        let start = at(0, 0);
        let path = [at(1, 0)];
        let move_input = input(start, &path, 25, Size::Medium, DiagonalCounter::fresh());
        let first = stride(move_input).expect("first stride");
        let second = stride(move_input).expect("second stride");
        assert_eq!(first, second);
        assert_eq!(first.cost_feet, 5);
        assert_eq!(dice.remaining(), 1);
    }

    /// Player Core, "Size, Space, and Reach".
    #[test]
    fn the_size_table_keeps_every_size() {
        assert_eq!(
            size_data(Size::Tiny),
            SizeData {
                space: Space::UnderFive,
                reach_tall_feet: 0,
                reach_long_feet: 0,
            }
        );
        assert_eq!(
            size_data(Size::Small),
            SizeData {
                space: Space::Exact(5),
                reach_tall_feet: 5,
                reach_long_feet: 5,
            }
        );
        assert_eq!(
            size_data(Size::Medium),
            SizeData {
                space: Space::Exact(5),
                reach_tall_feet: 5,
                reach_long_feet: 5,
            }
        );
        assert_eq!(
            size_data(Size::Large),
            SizeData {
                space: Space::Exact(10),
                reach_tall_feet: 10,
                reach_long_feet: 5,
            }
        );
        assert_eq!(
            size_data(Size::Huge),
            SizeData {
                space: Space::Exact(15),
                reach_tall_feet: 15,
                reach_long_feet: 10,
            }
        );
        assert_eq!(
            size_data(Size::Gargantuan),
            SizeData {
                space: Space::AtLeast(20),
                reach_tall_feet: 20,
                reach_long_feet: 15,
            }
        );
    }

    /// Player Core, "Size, Space, and Reach".
    #[test]
    fn only_small_and_medium_can_move() {
        let start = at(0, 0);
        let path = [at(1, 0)];
        let counter = DiagonalCounter::fresh();
        for size in [Size::Large, Size::Tiny] {
            let result = stride(input(start, &path, 25, size, counter));
            assert_eq!(result, Err(GridError::UnsupportedSize));
            assert_eq!(start, at(0, 0));
            assert_eq!(counter.diagonals(), 0);
        }
        for size in [Size::Small, Size::Medium] {
            let outcome = stride(input(start, &path, 25, size, counter)).expect("one square");
            assert_eq!(outcome.cost_feet, 5);
            assert_eq!(outcome.end, at(1, 0));
        }
    }

    /// Player Core, "Diagonal Movement".
    #[test]
    fn diagonal_parity_across_actions() {
        let first = stride(input(
            at(0, 0),
            &[at(1, 1)],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("first diagonal");
        assert_eq!(first.cost_feet, 5);
        assert_eq!(first.counter.diagonals(), 1);
        let second = stride(input(
            at(1, 1),
            &[at(2, 2)],
            25,
            Size::Medium,
            first.counter,
        ))
        .expect("second diagonal");
        assert_eq!(second.cost_feet, 10);
        assert_eq!(second.counter.diagonals(), 2);
    }

    /// Player Core, "Diagonal Movement".
    #[test]
    fn odd_diagonal_at_turn_end() {
        let ended = stride(input(
            at(0, 0),
            &[at(1, 1)],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("diagonal before the turn ends");
        assert_eq!(ended.counter.diagonals(), 1);
        let reset = ended.counter.end_turn();
        assert_eq!(reset.diagonals(), 0);
        let next =
            stride(input(at(1, 1), &[at(2, 2)], 25, Size::Medium, reset)).expect("next turn");
        assert_eq!(next.cost_feet, 5);
    }

    /// Player Core, "Diagonal Movement".
    #[test]
    fn orthogonal_does_not_flip_parity() {
        let diagonal = stride(input(
            at(0, 0),
            &[at(1, 1)],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("diagonal");
        assert_eq!(diagonal.cost_feet, 5);
        let orthogonal = stride(input(
            at(1, 1),
            &[at(1, 2)],
            25,
            Size::Medium,
            diagonal.counter,
        ))
        .expect("orthogonal");
        assert_eq!(orthogonal.cost_feet, 5);
        assert_eq!(orthogonal.counter.diagonals(), 1);
        let again = stride(input(
            at(1, 2),
            &[at(2, 3)],
            25,
            Size::Medium,
            orthogonal.counter,
        ))
        .expect("second diagonal");
        assert_eq!(again.cost_feet, 10);
        assert_eq!(
            diagonal.cost_feet + orthogonal.cost_feet + again.cost_feet,
            20
        );
    }

    /// Player Core, "Diagonal Movement".
    #[test]
    fn the_counter_is_not_on_the_creature() {
        let creature = creature();
        let before = creature.clone();
        let state = new_combatant_state(CombatantId(1), &creature);
        let outcome = stride(input(
            at(0, 0),
            &[at(1, 1)],
            creature.speed_feet,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("diagonal stride");
        assert_eq!(creature, before);
        assert_eq!(state, new_combatant_state(CombatantId(1), &creature));
        assert_eq!(outcome.counter.diagonals(), 1);
        assert_ne!(outcome.counter, DiagonalCounter::fresh());
    }

    /// Player Core, "Diagonal Movement".
    ///
    /// DEV-15 substitute: encounter-core passes [`DiagonalCounter::fresh`] for movement off
    /// the creature's turn and discards the returned counter. The stored turn counter stays put.
    #[test]
    fn off_turn_movement_uses_a_fresh_counter() {
        let stored = stride(input(
            at(0, 0),
            &[at(1, 1)],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("stored turn diagonal")
        .counter;
        assert_eq!(stored.diagonals(), 1);

        let one = stride(input(
            at(0, 0),
            &[at(1, 1)],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("off-turn diagonal");
        assert_eq!(one.cost_feet, 5);
        let _discarded = one.counter;

        let two = stride(input(
            at(0, 0),
            &[at(1, 1), at(2, 2)],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("off-turn two diagonals");
        assert_eq!(two.cost_feet, 15);
        let _discarded = two.counter;
        assert_eq!(stored.diagonals(), 1);
    }

    /// Player Core, "Diagonal Movement".
    #[test]
    fn shared_diagonal_counter_includes_step() {
        let diagonal = stride(input(
            at(0, 0),
            &[at(1, 1)],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("diagonal stride");
        assert_eq!(diagonal.cost_feet, 5);
        let stepped = step(input(
            at(1, 1),
            &[at(1, 2)],
            25,
            Size::Medium,
            diagonal.counter,
        ))
        .expect("orthogonal step");
        assert_eq!(stepped.cost_feet, 5);
        assert_eq!(stepped.counter.diagonals(), 1);
        let again = stride(input(
            at(1, 2),
            &[at(2, 3)],
            25,
            Size::Medium,
            stepped.counter,
        ))
        .expect("second diagonal");
        assert_eq!(again.cost_feet, 10);
    }

    /// Player Core, "Diagonal Movement", "Crawl", and "Leap".
    ///
    /// Crawl and Leap share this counter when they are added. This change does not implement them.
    #[test]
    fn step_and_crawl_share_the_diagonal_counter() {
        let stepped = step(input(
            at(0, 0),
            &[at(1, 1)],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("diagonal step");
        assert_eq!(stepped.counter.diagonals(), 1);
        let followed = stride(input(
            at(1, 1),
            &[at(2, 2)],
            25,
            Size::Medium,
            stepped.counter,
        ))
        .expect("diagonal stride");
        assert_eq!(followed.cost_feet, 10);
    }

    /// Player Core, "Difficult Terrain".
    #[test]
    fn difficult_on_a_diagonal() {
        let first = stride(input(
            at(0, 0),
            &[at(1, 1)],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("open diagonal");
        assert_eq!(first.cost_feet, 5);
        let terrain = [(at(2, 2), Terrain::Difficult)];
        let path = [at(2, 2)];
        let mut second = input(at(1, 1), &path, 25, Size::Medium, first.counter);
        second.terrain = &terrain;
        let second = stride(second).expect("difficult diagonal");
        assert_eq!(second.cost_feet, 15);
        assert_ne!(second.cost_feet, 20);
    }

    /// Player Core, "Difficult Terrain".
    #[test]
    fn greater_difficult_on_a_diagonal() {
        let greater = [(at(1, 1), Terrain::GreaterDifficult)];
        let first_path = [at(1, 1)];
        let mut first = input(
            at(0, 0),
            &first_path,
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        );
        first.terrain = &greater;
        let first = stride(first).expect("first greater diagonal");
        assert_eq!(first.cost_feet, 15);

        let open = stride(input(
            at(0, 0),
            &[at(1, 1)],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("open first diagonal");
        let second_square = [(at(2, 2), Terrain::GreaterDifficult)];
        let second_path = [at(2, 2)];
        let mut second = input(at(1, 1), &second_path, 25, Size::Medium, open.counter);
        second.terrain = &second_square;
        let second = stride(second).expect("second greater diagonal");
        assert_eq!(second.cost_feet, 20);
    }

    /// Player Core, "Difficult Terrain".
    #[test]
    fn leaving_difficult_terrain() {
        let terrain = [(at(0, 0), Terrain::Difficult)];
        let path = [at(1, 0)];
        let mut move_input = input(at(0, 0), &path, 25, Size::Medium, DiagonalCounter::fresh());
        move_input.terrain = &terrain;
        let outcome = stride(move_input).expect("leave difficult terrain");
        assert_eq!(outcome.cost_feet, 5);
    }

    /// Player Core, "Stride".
    #[test]
    fn exact_speed() {
        let five = line(0, 5);
        let succeeded = stride(input(
            at(0, 0),
            &five,
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("five squares");
        assert_eq!(succeeded.cost_feet, 25);
        assert_eq!(succeeded.end, at(5, 0));

        let start = at(0, 0);
        let six = line(0, 6);
        let counter = DiagonalCounter::fresh();
        let failed = stride(input(start, &six, 25, Size::Medium, counter));
        assert_eq!(failed, Err(GridError::ExceedsSpeed));
        assert_eq!(start, at(0, 0));
        assert_eq!(counter.diagonals(), 0);
    }

    /// Player Core, "Stride".
    #[test]
    fn terrain_cost_exceeds_the_remainder() {
        let path = line(0, 5);
        let terrain = [(at(5, 0), Terrain::Difficult)];
        let start = at(0, 0);
        let mut move_input = input(start, &path, 25, Size::Medium, DiagonalCounter::fresh());
        move_input.terrain = &terrain;
        assert_eq!(stride(move_input), Err(GridError::ExceedsSpeed));
        assert_eq!(start, at(0, 0));
    }

    /// Player Core, "Stride".
    #[test]
    fn leftover_movement_is_not_banked() {
        let first = stride(input(
            at(0, 0),
            &[at(1, 0)],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("one square");
        assert_eq!(first.cost_feet, 5);
        let five = line(1, 5);
        let second = stride(input(at(1, 0), &five, 25, Size::Medium, first.counter))
            .expect("five more squares");
        assert_eq!(second.cost_feet, 25);
        assert_ne!(second.cost_feet, 20);
    }

    /// Player Core, "Speed" and "Step".
    #[test]
    fn speed_0() {
        let start = at(0, 0);
        let counter = DiagonalCounter::fresh();
        let stayed = stride(input(start, &[], 0, Size::Medium, counter)).expect("empty stride");
        assert_eq!(stayed.cost_feet, 0);
        assert_eq!(stayed.end, start);

        let path = [at(1, 0)];
        assert_eq!(
            stride(input(start, &path, 0, Size::Medium, counter)),
            Err(GridError::ExceedsSpeed)
        );
        assert_eq!(
            step(input(start, &path, 0, Size::Medium, counter)),
            Err(GridError::StepRequiresTenFeet)
        );
        assert_eq!(start, at(0, 0));
        assert_eq!(counter.diagonals(), 0);
    }

    /// validation (DEV-21). Rules 2347 (Player Core pg. 420) and Rules 2356 (5-ft squares).
    ///
    /// Player Core has no multiple-of-5 rule. Movement keeps this rejection as a defensive guard.
    #[test]
    fn speed_not_a_multiple_of_five() {
        assert_eq!(validate_speed(7), Err(GridError::SpeedNotMultipleOfFive));
        assert_ne!(validate_speed(7), Err(GridError::NegativeSpeed));
        let start = at(0, 0);
        let result = stride(input(start, &[], 7, Size::Medium, DiagonalCounter::fresh()));
        assert_eq!(result, Err(GridError::SpeedNotMultipleOfFive));
        assert_ne!(result, Err(GridError::NegativeSpeed));
        assert_eq!(start, at(0, 0));
    }

    /// validation (DEV-21). Rules 2347 (Player Core pg. 420) and Rules 2356 (5-ft squares).
    ///
    /// Player Core has no multiple-of-5 rule.
    #[test]
    fn multiples_of_five_pass_speed_validation() {
        for speed in [0, 5, 10, 25] {
            assert_eq!(validate_speed(speed), Ok(()));
        }
    }

    /// validation (DEV-21). Rules 2347 (Player Core pg. 420) and Rules 2356 (5-ft squares).
    ///
    /// Player Core has no multiple-of-5 rule.
    #[test]
    fn a_step_uses_the_speed_validation() {
        let start = at(0, 0);
        let result = step(input(
            start,
            &[at(1, 0)],
            7,
            Size::Medium,
            DiagonalCounter::fresh(),
        ));
        assert_eq!(result, Err(GridError::SpeedNotMultipleOfFive));
        assert_eq!(start, at(0, 0));
    }

    /// Player Core, "Grid Movement".
    #[test]
    fn a_gap_in_the_path_is_rejected() {
        let start = at(0, 0);
        let counter = DiagonalCounter::fresh();
        let result = stride(input(start, &[at(2, 0)], 25, Size::Medium, counter));
        assert_eq!(result, Err(GridError::PathNotAdjacent));
        assert_eq!(start, at(0, 0));
        assert_eq!(counter.diagonals(), 0);
    }

    /// Player Core, "Stride".
    #[test]
    fn an_empty_path_does_not_search() {
        let start = at(0, 0);
        let destination = at(3, 0);
        let outcome = stride(input(
            start,
            &[],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("empty");
        assert_eq!(outcome.cost_feet, 0);
        assert!(outcome.events.is_empty());
        assert_eq!(outcome.end, start);
        assert_ne!(outcome.end, destination);
    }

    /// Player Core, "Grid Movement".
    #[test]
    fn off_grid() {
        let start = at(0, 0);
        let result = stride(input(
            start,
            &[at(10, 0)],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ));
        assert_eq!(result, Err(GridError::OffGrid));
        assert_eq!(start, at(0, 0));
    }

    /// Player Core, "Stride".
    #[test]
    fn stride_emits_left_and_entered_events() {
        let origin = at(0, 0);
        let orthogonal = at(1, 0);
        let diagonal = at(2, 1);
        let outcome = stride(input(
            origin,
            &[orthogonal, diagonal],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("two squares");
        assert_eq!(outcome.cost_feet, 10);
        assert!(outcome.triggers);
        assert_eq!(
            outcome.events,
            vec![
                SquareEvent::Left { square: origin },
                SquareEvent::Entered { square: orthogonal },
                SquareEvent::Left { square: orthogonal },
                SquareEvent::Entered { square: diagonal },
            ]
        );
    }

    /// Player Core, "Stride".
    #[test]
    fn a_stride_does_not_spend_actions() {
        let creature = creature();
        let mut state = new_combatant_state(CombatantId(1), &creature);
        state.actions_remaining = 2;
        state.reaction_ready = false;
        let outcome = stride(input(
            at(0, 0),
            &[at(1, 0)],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("legal stride");
        assert_eq!(state.actions_remaining, 2);
        assert!(!state.reaction_ready);
        let MoveOutcome {
            cost_feet,
            end,
            events,
            counter,
            triggers,
        } = outcome;
        assert_eq!(cost_feet, 5);
        assert_eq!(end, at(1, 0));
        assert_eq!(events.len(), 2);
        assert_eq!(counter.diagonals(), 0);
        assert!(triggers);
    }

    /// Player Core, "Grid Movement".
    ///
    /// Future ruling, not applied: a diagonal is blocked only when both orthogonal neighbours
    /// block. This change has no wall input and does not apply that ruling.
    #[test]
    fn a_diagonal_needs_no_wall_check() {
        let outcome = stride(input(
            at(0, 0),
            &[at(1, 1)],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("open diagonal");
        assert_eq!(outcome.cost_feet, 5);
        assert_eq!(outcome.end, at(1, 1));
    }

    /// Player Core, "Step".
    #[test]
    fn step_into_difficult_terrain() {
        let start = at(0, 0);
        let path = [at(1, 0)];
        let counter = DiagonalCounter::fresh();
        for terrain_kind in [Terrain::Difficult, Terrain::GreaterDifficult] {
            let terrain = [(at(1, 0), terrain_kind)];
            let mut move_input = input(start, &path, 25, Size::Medium, counter);
            move_input.terrain = &terrain;
            assert_eq!(step(move_input), Err(GridError::StepIntoDifficultTerrain));
            assert_eq!(start, at(0, 0));
            assert_eq!(counter.diagonals(), 0);
        }
    }

    /// Player Core, "Step".
    #[test]
    fn step_with_speed_5() {
        let start = at(0, 0);
        let result = step(input(
            start,
            &[at(1, 0)],
            5,
            Size::Medium,
            DiagonalCounter::fresh(),
        ));
        assert_eq!(result, Err(GridError::StepRequiresTenFeet));
        assert_eq!(start, at(0, 0));
    }

    /// Player Core, "Step".
    ///
    /// D6 substitute: the Step costs 5 feet, does not spend Speed, and still advances the counter.
    #[test]
    fn step_advances_parity() {
        let stepped = step(input(
            at(0, 0),
            &[at(1, 1)],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("diagonal step");
        assert_eq!(stepped.cost_feet, 5);
        let followed = stride(input(
            at(1, 1),
            &[at(2, 2)],
            25,
            Size::Medium,
            stepped.counter,
        ))
        .expect("following stride");
        assert_eq!(followed.cost_feet, 10);
    }

    /// Player Core, "Step".
    #[test]
    fn step_on_a_10_ft_diagonal_costs_5() {
        let counted = stride(input(
            at(0, 0),
            &[at(1, 1)],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("one diagonal already counted");
        assert_eq!(counted.counter.diagonals(), 1);
        let stepped = step(input(
            at(1, 1),
            &[at(2, 2)],
            25,
            Size::Medium,
            counted.counter,
        ))
        .expect("step on the 10-foot diagonal");
        assert_eq!(stepped.cost_feet, 5);
        assert_ne!(stepped.cost_feet, 10);
        assert_eq!(stepped.counter.diagonals(), 2);
        let followed = stride(input(
            at(2, 2),
            &[at(3, 3)],
            25,
            Size::Medium,
            stepped.counter,
        ))
        .expect("next diagonal");
        assert_eq!(followed.cost_feet, 5);
    }

    /// Player Core, "Step".
    #[test]
    fn a_step_is_one_square() {
        let start = at(0, 0);
        let two = [at(1, 0), at(2, 0)];
        assert_eq!(
            step(input(
                start,
                &two,
                25,
                Size::Medium,
                DiagonalCounter::fresh()
            )),
            Err(GridError::StepNotOneSquare)
        );
        assert_eq!(
            step(input(
                start,
                &[],
                25,
                Size::Medium,
                DiagonalCounter::fresh()
            )),
            Err(GridError::StepNotOneSquare)
        );
        assert_eq!(start, at(0, 0));
    }

    /// Player Core, "Step".
    #[test]
    fn step_events_are_non_triggering() {
        let start = at(0, 0);
        let destination = at(1, 1);
        let outcome = step(input(
            start,
            &[destination],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("diagonal step");
        assert_eq!(outcome.cost_feet, 5);
        assert!(!outcome.triggers);
        assert_eq!(
            outcome.events,
            vec![
                SquareEvent::Left { square: start },
                SquareEvent::Entered {
                    square: destination
                },
            ]
        );
    }

    /// Player Core, "Moving Through a Creature's Space".
    ///
    /// D7 substitute: same faction may be passed through. Willing is same faction until Tumble Through exists.
    #[test]
    fn through_an_ally() {
        let occupants = [Occupant {
            square: at(1, 0),
            faction: ALLY,
        }];
        let path = [at(1, 0), at(2, 0)];
        let mut move_input = input(at(0, 0), &path, 25, Size::Medium, DiagonalCounter::fresh());
        move_input.occupants = &occupants;
        let outcome = stride(move_input).expect("through an ally");
        assert_eq!(outcome.end, at(2, 0));
        assert!(
            outcome
                .events
                .contains(&SquareEvent::Entered { square: at(1, 0) })
        );
        assert_ne!(outcome.end, at(1, 0));
    }

    /// Player Core, "Moving Through a Creature's Space".
    #[test]
    fn through_an_enemy() {
        let occupants = [Occupant {
            square: at(1, 0),
            faction: FOE,
        }];
        let start = at(0, 0);
        for path in [vec![at(1, 0), at(2, 0)], vec![at(1, 0)]] {
            let mut move_input = input(start, &path, 25, Size::Medium, DiagonalCounter::fresh());
            move_input.occupants = &occupants;
            assert_eq!(stride(move_input), Err(GridError::EnemySquare));
            assert_eq!(start, at(0, 0));
        }
    }

    /// Player Core, "Moving Through a Creature's Space".
    ///
    /// D8 substitute: ending on any other creature is [`GridError::OccupiedEnd`]. The book's
    /// next-action move out of a willing creature's square is encounter-core.
    #[test]
    fn ending_in_an_ally_square() {
        let occupants = [Occupant {
            square: at(1, 0),
            faction: ALLY,
        }];
        let start = at(0, 0);
        let path = [at(1, 0)];
        let mut move_input = input(start, &path, 25, Size::Medium, DiagonalCounter::fresh());
        move_input.occupants = &occupants;
        assert_eq!(stride(move_input), Err(GridError::OccupiedEnd));
        assert_eq!(start, at(0, 0));
    }

    /// Player Core, "Sharing a Space".
    ///
    /// Prone, dead, willing, and Tiny sharing are out of scope. The occupant has no exception,
    /// so ending on a same-faction creature stays [`GridError::OccupiedEnd`].
    #[test]
    fn prone_dead_willing_and_tiny_sharing_is_out_of_scope() {
        let occupants = [Occupant {
            square: at(1, 0),
            faction: ALLY,
        }];
        let start = at(0, 0);
        let path = [at(1, 0)];
        let mut move_input = input(start, &path, 25, Size::Medium, DiagonalCounter::fresh());
        move_input.occupants = &occupants;
        assert_eq!(stride(move_input), Err(GridError::OccupiedEnd));
        assert_eq!(start, at(0, 0));
    }

    /// Player Core, "Areas".
    #[test]
    fn distance_0_and_adjacency() {
        let here = at(0, 0);
        let there = at(1, 1);
        let _difficult_between_other_routes = (at(1, 0), Terrain::Difficult);
        let _other_creature = Occupant {
            square: at(0, 1),
            faction: FOE,
        };
        let counter = stride(input(
            at(4, 4),
            &[at(5, 5)],
            25,
            Size::Medium,
            DiagonalCounter::fresh(),
        ))
        .expect("a counted diagonal")
        .counter;
        assert_eq!(counter.diagonals(), 1);

        assert_eq!(distance(here, there, Size::Medium, Size::Medium), Ok(5));
        assert_eq!(adjacent(here, there, Size::Medium, Size::Medium), Ok(true));
        assert_eq!(distance(there, here, Size::Medium, Size::Medium), Ok(5));
        assert_eq!(adjacent(there, here, Size::Medium, Size::Medium), Ok(true));
        assert_eq!(distance(here, here, Size::Medium, Size::Medium), Ok(0));
        assert_eq!(adjacent(here, here, Size::Medium, Size::Medium), Ok(false));
        assert_eq!(counter.diagonals(), 1);
    }

    /// Player Core, "Range and Reach".
    #[test]
    fn reach_5_is_only_adjacency() {
        let self_square = at(0, 0);
        assert_eq!(
            in_reach(self_square, at(1, 1), Size::Medium, Size::Medium, 5),
            Ok(true)
        );
        assert_eq!(
            in_reach(self_square, at(2, 0), Size::Medium, Size::Medium, 5),
            Ok(false)
        );
        assert_eq!(
            in_reach(self_square, self_square, Size::Medium, Size::Medium, 5),
            Ok(false)
        );
    }

    /// Player Core, "Range and Reach".
    #[test]
    fn reach_10_diagonal_exception() {
        let from = at(0, 0);
        assert_eq!(distance(from, at(2, 2), Size::Medium, Size::Medium), Ok(15));
        assert_eq!(
            in_reach(from, at(2, 2), Size::Medium, Size::Medium, 10),
            Ok(true)
        );
        assert_eq!(distance(from, at(2, 1), Size::Medium, Size::Medium), Ok(10));
        assert_eq!(
            in_reach(from, at(2, 1), Size::Medium, Size::Medium, 10),
            Ok(true)
        );
        assert_eq!(distance(from, at(3, 1), Size::Medium, Size::Medium), Ok(15));
        assert_eq!(
            in_reach(from, at(3, 1), Size::Medium, Size::Medium, 10),
            Ok(false)
        );
        assert_eq!(distance(from, at(3, 3), Size::Medium, Size::Medium), Ok(20));
        assert_eq!(
            in_reach(from, at(3, 3), Size::Medium, Size::Medium, 20),
            Ok(true)
        );
    }

    /// Player Core, "Range Penalty".
    #[test]
    fn range_boundary() {
        assert_eq!(
            range_band(0, 60),
            Ok(RangeBand {
                increments: 1,
                penalty: 0,
            })
        );
        assert_eq!(
            range_band(60, 60),
            Ok(RangeBand {
                increments: 1,
                penalty: 0,
            })
        );
        assert_eq!(
            range_band(65, 60),
            Ok(RangeBand {
                increments: 2,
                penalty: -2,
            })
        );
        assert_eq!(
            range_band(360, 60),
            Ok(RangeBand {
                increments: 6,
                penalty: -10,
            })
        );
        assert_eq!(range_band(365, 60), Err(GridError::BeyondRange));
        assert_ne!(
            range_band(365, 60),
            Ok(RangeBand {
                increments: 7,
                penalty: -12,
            })
        );
    }

    /// Player Core, "Range Penalty".
    #[test]
    fn a_boundary_uses_the_lower_increment() {
        assert_eq!(
            range_band(120, 60),
            Ok(RangeBand {
                increments: 2,
                penalty: -2,
            })
        );
        assert_eq!(
            range_band(121, 60),
            Ok(RangeBand {
                increments: 3,
                penalty: -4,
            })
        );
    }

    /// Player Core, "Range Penalty" and "Penalties".
    ///
    /// The summed −9 with the multiple attack penalty is encounter-core. This result is the
    /// count and the untyped penalty only.
    #[test]
    fn range_penalty_is_plain_data() {
        assert_eq!(
            range_band(130, 60),
            Ok(RangeBand {
                increments: 3,
                penalty: -4,
            })
        );
    }

    /// Player Core, "Speed" and "Range Penalty".
    #[test]
    fn negative_or_zero_inputs() {
        let start = at(0, 0);
        let speed = stride(input(
            start,
            &[],
            -5,
            Size::Medium,
            DiagonalCounter::fresh(),
        ));
        assert_eq!(speed, Err(GridError::NegativeSpeed));
        assert_ne!(
            speed,
            Ok(MoveOutcome {
                cost_feet: 0,
                end: start,
                events: Vec::new(),
                counter: DiagonalCounter::fresh(),
                triggers: true,
            })
        );
        assert_eq!(range_band(60, 0), Err(GridError::InvalidRangeIncrement));
        assert_ne!(
            range_band(60, 0),
            Ok(RangeBand {
                increments: 1,
                penalty: 0,
            })
        );
        assert_eq!(range_band(-1, 30), Err(GridError::NegativeDistance));
    }

    /// Player Core, "Grid Movement".
    #[test]
    fn coordinate_overflow_is_an_error() {
        let positive = at(i32::MAX, i32::MAX);
        let negative = at(i32::MIN, i32::MIN);
        assert_eq!(
            distance(positive, negative, Size::Medium, Size::Medium),
            Err(GridError::Overflow)
        );
    }

    /// Player Core, "Immobilized".
    #[test]
    fn immobilized() {
        let start = at(0, 0);
        let path = [at(1, 0)];
        let counter = DiagonalCounter::fresh();
        let mut move_input = input(start, &path, 25, Size::Medium, counter);
        move_input.immobilized = true;
        assert_eq!(stride(move_input), Err(GridError::Immobilized));
        assert_eq!(step(move_input), Err(GridError::Immobilized));
        assert_eq!(start, at(0, 0));
        assert_eq!(counter.diagonals(), 0);
    }

    /// Player Core, "Immobilized".
    #[test]
    fn the_immobilized_check_needs_no_path() {
        assert_eq!(move_trait_allowed(true), Err(GridError::Immobilized));
        assert_eq!(move_trait_allowed(false), Ok(()));
    }
}
