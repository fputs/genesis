//! Cast a Spell: the spell attack modifier, the spell DC, and area membership.
//!
//! The encounter methods spend the actions. This module does not read a clock.
//!
//! Player Core pg. 299, Cast a Spell. Player Core pg. 403, Spell Attack Rolls.
//! Player Core pg. 428, Areas.

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "area helpers and the projectile cost are called by the encounter and by this crate's tests"
    )
)]

use crate::check::{DcModifier, dc_from_modifier};
use crate::combatant::CombatantId;
use crate::grid::{AbsDelta, GridError, Size, Square, distance, grid_feet};
use crate::modifiers::{Modifier, ModifierError, Statistic, total};
use crate::positioning::{ActionCost, ActionTrait};
use crate::save::{Level, Proficiency, proficiency_bonus};

/// One spellcasting entry. The attribute is a modifier, not a score.
///
/// Player Core pg. 403, Spell Attack Rolls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spellcasting {
    /// Level used for the proficiency bonus. Player Core pg. 403, Spell Attack Rolls.
    pub level: Level,
    /// Attribute modifier added to the spell attack and the spell DC.
    ///
    /// Player Core pg. 403, Spell Attack Rolls.
    pub attribute: i32,
    /// Proficiency rank. Untrained is +0. Player Core pg. 403, Spell Attack Rolls.
    pub proficiency: Proficiency,
}

/// Which way a cone or a line points. East increases `x` and south increases `y`.
///
/// Player Core pg. 428, Areas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aim {
    /// Increasing `x`. Player Core pg. 428, Areas.
    East,
    /// Decreasing `y`. Player Core pg. 428, Areas.
    North,
    /// Increasing `y`. Player Core pg. 428, Areas.
    South,
    /// Decreasing `x`. Player Core pg. 428, Areas.
    West,
    /// Increasing `x` and decreasing `y`. Player Core pg. 428, Cone.
    Northeast,
    /// Increasing `x` and increasing `y`. Player Core pg. 428, Cone.
    Southeast,
    /// Decreasing `x` and increasing `y`. Player Core pg. 428, Cone.
    Southwest,
    /// Decreasing `x` and decreasing `y`. Player Core pg. 428, Cone.
    Northwest,
}

/// A creature the caller places on one square.
///
/// Player Core pg. 428, Areas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AreaCreature {
    /// Combatant id. Player Core, Encounter Mode.
    pub id: CombatantId,
    /// Square that creature occupies. Player Core pg. 428, Areas.
    pub square: Square,
    /// Space used for the size check. Player Core pg. 421, Size, Space, and Reach.
    pub size: Size,
}

/// Why an area query was rejected. A rejection returns no squares.
///
/// Player Core pg. 428, Areas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GeometryError {
    /// The radius was not a positive multiple of 5. Player Core pg. 428, Burst.
    InvalidRadius,
    /// The length was not a positive multiple of 5. Player Core pg. 428, Line.
    InvalidLength,
    /// The cone is not the 15-foot template. Player Core pg. 428, Cone.
    UnsupportedCone,
    /// The grid rejected the measurement. Player Core pg. 421, Size, Space, and Reach.
    Grid(GridError),
}

const BREATHE_FIRE_TRAITS: &[ActionTrait] = &[
    ActionTrait::Concentrate,
    ActionTrait::Fire,
    ActionTrait::Manipulate,
];

const PROJECTILE_TRAITS: &[ActionTrait] = &[
    ActionTrait::Attack,
    ActionTrait::Cantrip,
    ActionTrait::Concentrate,
    ActionTrait::Manipulate,
];

const D6: crate::dice::Faces = {
    match crate::dice::Faces::new(6) {
        Ok(faces) => faces,
        Err(_) => crate::dice::Faces::twenty(),
    }
};

/// Breathe Fire's 2-action cost. It has no attack trait.
///
/// Player Core pg. 319, Breathe Fire.
pub(crate) fn breathe_fire_cost() -> ActionCost {
    ActionCost::Actions {
        count: 2,
        traits: BREATHE_FIRE_TRAITS,
    }
}

/// Telekinetic Projectile's 2-action cost. The attack trait carries the multiple attack penalty.
///
/// Targets: 1 creature. Player Core pg. 363, Telekinetic Projectile.
pub(crate) fn projectile_cost() -> ActionCost {
    ActionCost::Actions {
        count: 2,
        traits: PROJECTILE_TRAITS,
    }
}

/// Attribute modifier, plus proficiency, plus `SpellAttack` modifiers only.
///
/// A modifier on `Attack` is not read. Untrained is +0. The other ranks are level plus
/// 2, 4, 6, or 8. A sum that does not fit is [`ModifierError::Overflow`].
///
/// Player Core pg. 403, Spell Attack Rolls. Player Core pg. 400, Bonuses.
pub fn spell_attack_modifier(
    casting: &Spellcasting,
    modifiers: &[Modifier],
) -> Result<i32, ModifierError> {
    let stacked = total(modifiers)?;
    let bonus = proficiency_bonus(casting.level, casting.proficiency);
    casting
        .attribute
        .checked_add(bonus)
        .and_then(|sum| sum.checked_add(stacked.get(Statistic::SpellAttack)))
        .ok_or(ModifierError::Overflow)
}

/// Spell DC: 10 plus [`spell_attack_modifier`].
///
/// Player Core pg. 403, Spell DC.
pub fn spell_dc(casting: &Spellcasting, modifiers: &[Modifier]) -> Result<i32, ModifierError> {
    let modifier = spell_attack_modifier(casting, modifiers)?;
    let dc = DcModifier::new(modifier).map_err(|_| ModifierError::Overflow)?;
    Ok(dc_from_modifier(dc))
}

/// Rolls 2d6 once. The attribute modifier is not added.
///
/// Player Core pg. 319, Breathe Fire. Player Core pg. 363, Telekinetic Projectile.
pub(crate) fn roll_2d6(dice: &mut impl crate::dice::Dice) -> i32 {
    i32::from(dice.roll(D6)) + i32::from(dice.roll(D6))
}

/// Squares of a burst measured from the northwest corner of `origin`.
///
/// A square is inside when [`grid_feet`] from that corner to the square's farthest corner
/// is at most `radius_feet`. The radius must be a positive multiple of 5. Terrain and the
/// turn's diagonal counter are not read.
///
/// Player Core pg. 428, Burst. Ruling R-13 is the cone; this is the same corner measurement.
pub(crate) fn burst(origin: Square, radius_feet: i32) -> Result<Vec<Square>, GeometryError> {
    let radius = positive_multiple(radius_feet).ok_or(GeometryError::InvalidRadius)?;
    let span = (radius_feet / 5)
        .checked_add(3)
        .ok_or(GeometryError::Grid(GridError::Overflow))?;
    let mut squares = Vec::new();
    for x in -span..=span {
        for y in -span..=span {
            let square = Square {
                x: origin
                    .x
                    .checked_add(x)
                    .ok_or(GeometryError::Grid(GridError::Overflow))?,
                y: origin
                    .y
                    .checked_add(y)
                    .ok_or(GeometryError::Grid(GridError::Overflow))?,
            };
            if farthest_corner(origin, square)? <= radius {
                squares.push(square);
            }
        }
    }
    sort_squares(&mut squares);
    Ok(squares)
}

/// The 15-foot cone template. Any other length is [`GeometryError::UnsupportedCone`].
///
/// Cardinal aims use the seven-square diagram. Diagonal aims use one 15-foot burst
/// quadrant from the chosen corner. Ruling R-13.
///
/// Player Core pg. 428, Cone.
pub(crate) fn cone(
    origin: Square,
    aim: Aim,
    length_feet: i32,
) -> Result<Vec<Square>, GeometryError> {
    if length_feet != 15 {
        return Err(GeometryError::UnsupportedCone);
    }
    let mut squares = match aim {
        Aim::East | Aim::North | Aim::South | Aim::West => cardinal_cone(origin, aim)?,
        Aim::Northeast | Aim::Southeast | Aim::Southwest | Aim::Northwest => {
            diagonal_cone(origin, aim)?
        }
    };
    sort_squares(&mut squares);
    Ok(squares)
}

/// Cone squares after a size check. An unsupported size returns no squares.
///
/// Player Core pg. 421, Size, Space, and Reach. Player Core pg. 428, Cone.
pub(crate) fn cone_membership(
    origin: Square,
    caster_size: Size,
    aim: Aim,
    length_feet: i32,
    creatures: &[AreaCreature],
) -> Result<Vec<Square>, GeometryError> {
    validate_size(caster_size)?;
    for creature in creatures {
        validate_size(creature.size)?;
    }
    cone(origin, aim, length_feet)
}

/// Squares within `radius_feet` of `origin`, using the existing square distance.
///
/// The origin square is included only when `include_origin` is set.
///
/// Player Core pg. 428, Emanation.
pub(crate) fn emanation(
    origin: Square,
    radius_feet: i32,
    include_origin: bool,
) -> Result<Vec<Square>, GeometryError> {
    let radius = positive_multiple(radius_feet).ok_or(GeometryError::InvalidRadius)?;
    let span = (radius_feet / 5)
        .checked_add(2)
        .ok_or(GeometryError::Grid(GridError::Overflow))?;
    let mut squares = Vec::new();
    for x in -span..=span {
        for y in -span..=span {
            let square = Square {
                x: origin
                    .x
                    .checked_add(x)
                    .ok_or(GeometryError::Grid(GridError::Overflow))?,
                y: origin
                    .y
                    .checked_add(y)
                    .ok_or(GeometryError::Grid(GridError::Overflow))?,
            };
            if square == origin && !include_origin {
                continue;
            }
            let feet = distance(origin, square, Size::Medium, Size::Medium)
                .map_err(GeometryError::Grid)?;
            if feet <= radius {
                squares.push(square);
            }
        }
    }
    sort_squares(&mut squares);
    Ok(squares)
}

/// One-square-wide line. A diagonal line uses the 5-10-5-10 count.
///
/// Four squares at 30 feet are that rule. The origin square is not included.
///
/// Player Core pg. 428, Line. Player Core pg. 421, Diagonal Movement.
pub(crate) fn line(
    origin: Square,
    aim: Aim,
    length_feet: i32,
) -> Result<Vec<Square>, GeometryError> {
    let length = positive_multiple(length_feet).ok_or(GeometryError::InvalidLength)?;
    let steps = length_feet / 5;
    let mut squares = Vec::new();
    for step in 1..=steps {
        let square = stepped(origin, aim, step)?;
        let feet =
            distance(origin, square, Size::Medium, Size::Medium).map_err(GeometryError::Grid)?;
        if feet <= length {
            squares.push(square);
        }
    }
    sort_squares(&mut squares);
    Ok(squares)
}

fn positive_multiple(feet: i32) -> Option<u32> {
    if feet > 0 && feet % 5 == 0 {
        u32::try_from(feet).ok()
    } else {
        None
    }
}

fn validate_size(size: Size) -> Result<(), GeometryError> {
    distance(
        Square { x: 0, y: 0 },
        Square { x: 0, y: 0 },
        size,
        Size::Medium,
    )
    .map(|_| ())
    .map_err(GeometryError::Grid)
}

fn farthest_corner(origin: Square, square: Square) -> Result<u32, GeometryError> {
    let mut best = 0u32;
    for (x, y) in corner_points(square)? {
        let dx = x
            .checked_sub(origin.x)
            .ok_or(GeometryError::Grid(GridError::Overflow))?;
        let dy = y
            .checked_sub(origin.y)
            .ok_or(GeometryError::Grid(GridError::Overflow))?;
        let feet = grid_feet(AbsDelta {
            ax: dx
                .checked_abs()
                .ok_or(GeometryError::Grid(GridError::Overflow))?,
            ay: dy
                .checked_abs()
                .ok_or(GeometryError::Grid(GridError::Overflow))?,
        })
        .map_err(GeometryError::Grid)?;
        best = best.max(feet);
    }
    Ok(best)
}

fn corner_points(square: Square) -> Result<[(i32, i32); 4], GeometryError> {
    let east = square
        .x
        .checked_add(1)
        .ok_or(GeometryError::Grid(GridError::Overflow))?;
    let south = square
        .y
        .checked_add(1)
        .ok_or(GeometryError::Grid(GridError::Overflow))?;
    Ok([
        (square.x, square.y),
        (east, square.y),
        (square.x, south),
        (east, south),
    ])
}

fn cardinal_cone(origin: Square, aim: Aim) -> Result<Vec<Square>, GeometryError> {
    const EAST: [(i32, i32); 7] = [(1, 0), (2, -1), (2, 0), (2, 1), (3, -1), (3, 0), (3, 1)];
    let mut squares = Vec::with_capacity(EAST.len());
    for (x, y) in EAST {
        let (x, y) = turn(x, y, aim);
        squares.push(offset(origin, x, y)?);
    }
    Ok(squares)
}

fn turn(x: i32, y: i32, aim: Aim) -> (i32, i32) {
    match aim {
        Aim::East | Aim::Northeast => (x, y),
        Aim::North | Aim::Northwest => (y, -x),
        Aim::West | Aim::Southwest => (-x, -y),
        Aim::South | Aim::Southeast => (-y, x),
    }
}

fn diagonal_cone(origin: Square, aim: Aim) -> Result<Vec<Square>, GeometryError> {
    let (corner_x, corner_y) = match aim {
        Aim::Northeast => (1, 0),
        Aim::Southeast => (1, 1),
        Aim::Southwest => (0, 1),
        Aim::Northwest => (0, 0),
        Aim::East | Aim::North | Aim::South | Aim::West => {
            return Err(GeometryError::UnsupportedCone);
        }
    };
    let corner = offset(origin, corner_x, corner_y)?;
    let burst = burst(corner, 15)?;
    Ok(burst
        .into_iter()
        .filter(|square| in_quadrant(origin, *square, aim))
        .collect())
}

fn in_quadrant(origin: Square, square: Square, aim: Aim) -> bool {
    match aim {
        Aim::Northeast => square.x > origin.x && square.y < origin.y,
        Aim::Southeast => square.x > origin.x && square.y > origin.y,
        Aim::Southwest => square.x < origin.x && square.y > origin.y,
        Aim::Northwest => square.x < origin.x && square.y < origin.y,
        Aim::East | Aim::North | Aim::South | Aim::West => false,
    }
}

fn stepped(origin: Square, aim: Aim, step: i32) -> Result<Square, GeometryError> {
    let (x, y) = match aim {
        Aim::East => (step, 0),
        Aim::West => (-step, 0),
        Aim::South => (0, step),
        Aim::North => (0, -step),
        Aim::Northeast => (step, -step),
        Aim::Southeast => (step, step),
        Aim::Southwest => (-step, step),
        Aim::Northwest => (-step, -step),
    };
    offset(origin, x, y)
}

fn offset(origin: Square, x: i32, y: i32) -> Result<Square, GeometryError> {
    Ok(Square {
        x: origin
            .x
            .checked_add(x)
            .ok_or(GeometryError::Grid(GridError::Overflow))?,
        y: origin
            .y
            .checked_add(y)
            .ok_or(GeometryError::Grid(GridError::Overflow))?,
    })
}

fn sort_squares(squares: &mut [Square]) {
    squares.sort_by_key(|square| (square.y, square.x));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice::{Dice, ScriptedDice};
    use crate::modifiers::{BonusType, Modifier, SourceId};
    use crate::save::Level;

    fn at(x: i32, y: i32) -> Square {
        Square { x, y }
    }

    fn points(squares: &[Square]) -> std::collections::BTreeSet<(i32, i32)> {
        squares.iter().map(|square| (square.x, square.y)).collect()
    }

    fn casting(level: i32, attribute: i32, proficiency: Proficiency) -> Spellcasting {
        Spellcasting {
            level: Level::new(level).expect("level"),
            attribute,
            proficiency,
        }
    }

    fn circumstance(value: i32, source: u32, statistic: Statistic) -> Modifier {
        Modifier::bonus(
            BonusType::Circumstance,
            value,
            SourceId(source),
            source,
            None,
            statistic,
        )
        .expect("bonus")
    }

    fn status(value: i32, statistic: Statistic) -> Modifier {
        Modifier::bonus(BonusType::Status, value, SourceId(1), 1, None, statistic).expect("bonus")
    }

    /// Player Core pg. 428, Area.
    #[test]
    fn the_same_burst_twice_matches() {
        let mut dice = ScriptedDice::new([8]);
        let first = burst(at(0, 0), 10).expect("burst");
        let second = burst(at(0, 0), 10).expect("burst");
        assert_eq!(first, second);
        assert_eq!(dice.remaining(), 1);
        assert_eq!(dice.d20(), 8);
    }

    /// Player Core pg. 428, Burst.
    #[test]
    fn a_5_foot_burst_is_the_four_touching_squares() {
        let squares = burst(at(0, 0), 5).expect("burst");
        assert_eq!(
            points(&squares),
            points(&[at(-1, -1), at(-1, 0), at(0, -1), at(0, 0)])
        );
    }

    /// Player Core pg. 428, Burst.
    #[test]
    fn a_10_foot_burst_is_twelve_squares() {
        let squares = burst(at(0, 0), 10).expect("burst");
        assert_eq!(squares.len(), 12);
        assert_eq!(
            points(&squares),
            points(&[
                at(-2, -1),
                at(-2, 0),
                at(-1, -2),
                at(-1, -1),
                at(-1, 0),
                at(-1, 1),
                at(0, -2),
                at(0, -1),
                at(0, 0),
                at(0, 1),
                at(1, -1),
                at(1, 0),
            ])
        );
    }

    /// Player Core pg. 428, Burst.
    #[test]
    fn a_20_foot_burst_reaches_the_far_corner_boundary() {
        let squares = burst(at(0, 0), 20).expect("burst");
        let found = points(&squares);
        assert_eq!(squares.len(), 44);
        assert!(found.contains(&(3, 0)));
        assert!(found.contains(&(2, 2)));
        assert!(!found.contains(&(4, 0)));
        assert!(!found.contains(&(3, 1)));
    }

    /// Player Core pg. 428, Burst.
    #[test]
    fn a_radius_of_0_4_or_negative_5_is_rejected() {
        for radius in [0, 4, -5] {
            assert_eq!(burst(at(0, 0), radius), Err(GeometryError::InvalidRadius));
        }
    }

    /// Player Core pg. 428, Cone. Ruling R-13.
    #[test]
    fn east_is_the_seven_squares() {
        let squares = cone(at(0, 0), Aim::East, 15).expect("cone");
        assert_eq!(
            points(&squares),
            points(&[
                at(1, 0),
                at(2, -1),
                at(2, 0),
                at(2, 1),
                at(3, -1),
                at(3, 0),
                at(3, 1),
            ])
        );
        assert!(!points(&squares).contains(&(0, 0)));
        assert!(!points(&squares).contains(&(1, -1)));
        assert!(!points(&squares).contains(&(1, 1)));
    }

    /// Player Core pg. 428, Cone. Ruling R-13.
    #[test]
    fn north_is_that_template_turned() {
        let squares = cone(at(0, 0), Aim::North, 15).expect("cone");
        assert_eq!(
            points(&squares),
            points(&[
                at(0, -1),
                at(-1, -2),
                at(0, -2),
                at(1, -2),
                at(-1, -3),
                at(0, -3),
                at(1, -3),
            ])
        );
    }

    /// Player Core pg. 428, Cone. Ruling R-13.
    #[test]
    fn south_is_that_template_turned() {
        let squares = cone(at(0, 0), Aim::South, 15).expect("cone");
        assert_eq!(
            points(&squares),
            points(&[
                at(0, 1),
                at(-1, 2),
                at(0, 2),
                at(1, 2),
                at(-1, 3),
                at(0, 3),
                at(1, 3),
            ])
        );
    }

    /// Player Core pg. 428, Cone. Ruling R-13.
    #[test]
    fn west_is_that_template_turned() {
        let squares = cone(at(0, 0), Aim::West, 15).expect("cone");
        assert_eq!(
            points(&squares),
            points(&[
                at(-1, 0),
                at(-2, -1),
                at(-2, 0),
                at(-2, 1),
                at(-3, -1),
                at(-3, 0),
                at(-3, 1),
            ])
        );
    }

    /// Player Core pg. 428, Cone.
    #[test]
    fn a_30_foot_cone_is_rejected() {
        assert_eq!(
            cone(at(0, 0), Aim::East, 30),
            Err(GeometryError::UnsupportedCone)
        );
    }

    /// Player Core pg. 428, Cone. Ruling R-13.
    #[test]
    fn northeast_is_the_six_squares() {
        let squares = cone(at(0, 0), Aim::Northeast, 15).expect("cone");
        assert_eq!(
            points(&squares),
            points(&[
                at(1, -1),
                at(2, -1),
                at(3, -1),
                at(1, -2),
                at(2, -2),
                at(1, -3),
            ])
        );
        assert!(!points(&squares).contains(&(0, 0)));
    }

    /// Player Core pg. 428, Cone. Ruling R-13.
    #[test]
    fn southeast_is_the_six_squares() {
        let squares = cone(at(0, 0), Aim::Southeast, 15).expect("cone");
        assert_eq!(
            points(&squares),
            points(&[at(1, 1), at(1, 2), at(1, 3), at(2, 1), at(2, 2), at(3, 1)])
        );
    }

    /// Player Core pg. 428, Cone. Ruling R-13.
    #[test]
    fn southwest_is_the_six_squares() {
        let squares = cone(at(0, 0), Aim::Southwest, 15).expect("cone");
        assert_eq!(
            points(&squares),
            points(&[
                at(-3, 1),
                at(-2, 1),
                at(-2, 2),
                at(-1, 1),
                at(-1, 2),
                at(-1, 3),
            ])
        );
    }

    /// Player Core pg. 428, Cone. Ruling R-13.
    #[test]
    fn northwest_is_the_six_squares() {
        let squares = cone(at(0, 0), Aim::Northwest, 15).expect("cone");
        assert_eq!(
            points(&squares),
            points(&[
                at(-3, -1),
                at(-2, -2),
                at(-2, -1),
                at(-1, -3),
                at(-1, -2),
                at(-1, -1),
            ])
        );
    }

    /// Player Core pg. 428, Emanation.
    #[test]
    fn a_5_foot_emanation_is_the_eight_adjacent_squares() {
        let squares = emanation(at(0, 0), 5, false).expect("emanation");
        assert_eq!(squares.len(), 8);
        assert!(!points(&squares).contains(&(0, 0)));
        assert!(points(&squares).contains(&(1, 0)));
        assert!(points(&squares).contains(&(1, 1)));
    }

    /// Player Core pg. 428, Emanation.
    #[test]
    fn the_caller_can_include_the_origin() {
        let squares = emanation(at(0, 0), 5, true).expect("emanation");
        assert!(points(&squares).contains(&(0, 0)));
        assert_eq!(squares.len(), 9);
    }

    /// Player Core pg. 428, Emanation.
    #[test]
    fn a_10_foot_emanation_includes_2_1_and_excludes_2_2() {
        let squares = emanation(at(0, 0), 10, false).expect("emanation");
        let found = points(&squares);
        assert_eq!(squares.len(), 20);
        assert!(found.contains(&(2, 1)));
        assert!(!found.contains(&(2, 2)));
        assert!(!found.contains(&(0, 0)));
    }

    /// Player Core pg. 428, Line.
    #[test]
    fn a_30_foot_east_line_is_six_squares() {
        let squares = line(at(0, 0), Aim::East, 30).expect("line");
        assert_eq!(
            points(&squares),
            points(&[at(1, 0), at(2, 0), at(3, 0), at(4, 0), at(5, 0), at(6, 0)])
        );
        assert!(!points(&squares).contains(&(0, 0)));
    }

    /// Player Core pg. 421, Diagonal Movement. Player Core pg. 428, Line.
    #[test]
    fn a_30_foot_northeast_line_is_four_squares() {
        let squares = line(at(0, 0), Aim::Northeast, 30).expect("line");
        assert_eq!(
            points(&squares),
            points(&[at(1, -1), at(2, -2), at(3, -3), at(4, -4)])
        );
        assert!(!points(&squares).contains(&(5, -5)));
    }

    /// Player Core pg. 428, Line.
    #[test]
    fn a_length_of_0_is_rejected() {
        assert_eq!(
            line(at(0, 0), Aim::East, 0),
            Err(GeometryError::InvalidLength)
        );
    }

    /// Player Core pg. 421, Size, Space, and Reach.
    #[test]
    fn a_large_creature_is_rejected() {
        let creature = AreaCreature {
            id: CombatantId(2),
            square: at(1, 0),
            size: Size::Large,
        };
        assert_eq!(
            cone_membership(at(0, 0), Size::Medium, Aim::East, 15, &[creature]),
            Err(GeometryError::Grid(GridError::UnsupportedSize))
        );
    }

    /// Player Core pg. 403, Spell Attack Rolls. Player Core pg. 403, Spell DC.
    #[test]
    fn trained_at_level_1_with_plus_4_is_plus_7_and_dc_17() {
        let casting = casting(1, 4, Proficiency::Trained);
        assert_eq!(spell_attack_modifier(&casting, &[]).expect("modifier"), 7);
        assert_eq!(spell_dc(&casting, &[]).expect("dc"), 17);
        assert_eq!(spell_attack_modifier(&casting, &[]).expect("again"), 7);
    }

    /// Player Core pg. 403, Spell Attack Rolls.
    #[test]
    fn expert_at_level_5_with_plus_3_is_plus_12_and_dc_22() {
        let casting = casting(5, 3, Proficiency::Expert);
        assert_eq!(spell_attack_modifier(&casting, &[]).expect("modifier"), 12);
        assert_eq!(spell_dc(&casting, &[]).expect("dc"), 22);
    }

    /// Player Core pg. 403, Spell Attack Rolls.
    #[test]
    fn untrained_at_level_20_with_plus_4_is_plus_4_and_dc_14() {
        let casting = casting(20, 4, Proficiency::Untrained);
        assert_eq!(spell_attack_modifier(&casting, &[]).expect("modifier"), 4);
        assert_eq!(spell_dc(&casting, &[]).expect("dc"), 14);
    }

    /// Player Core pg. 403, Spell DC.
    #[test]
    fn a_negative_attribute_lowers_the_dc() {
        let casting = casting(1, -1, Proficiency::Trained);
        assert_eq!(spell_attack_modifier(&casting, &[]).expect("modifier"), 2);
        assert_eq!(spell_dc(&casting, &[]).expect("dc"), 12);
    }

    /// Player Core pg. 403, Spell Attack Rolls.
    #[test]
    fn an_attack_bonus_does_not_raise_the_spell_dc() {
        let casting = casting(1, 4, Proficiency::Trained);
        let attack = circumstance(2, 1, Statistic::Attack);
        assert_eq!(
            spell_attack_modifier(&casting, &[attack]).expect("modifier"),
            7
        );
        assert_eq!(spell_dc(&casting, &[attack]).expect("dc"), 17);
    }

    /// Player Core pg. 403, Spell Attack Rolls.
    #[test]
    fn a_spellattack_status_bonus_raises_the_roll_and_the_dc() {
        let casting = casting(1, 4, Proficiency::Trained);
        let bonus = status(1, Statistic::SpellAttack);
        assert_eq!(
            spell_attack_modifier(&casting, &[bonus]).expect("modifier"),
            8
        );
        assert_eq!(spell_dc(&casting, &[bonus]).expect("dc"), 18);
    }

    /// Player Core pg. 400, Bonuses.
    #[test]
    fn two_circumstance_bonuses_keep_the_highest() {
        let casting = casting(1, 4, Proficiency::Trained);
        let modifiers = [
            circumstance(2, 1, Statistic::SpellAttack),
            circumstance(1, 2, Statistic::SpellAttack),
        ];
        assert_eq!(
            spell_attack_modifier(&casting, &modifiers).expect("modifier"),
            9
        );
        assert_eq!(spell_dc(&casting, &modifiers).expect("dc"), 19);
    }

    /// Player Core pg. 403, Spell Attack Rolls.
    #[test]
    fn a_modifier_that_does_not_fit_draws_nothing() {
        let dice = ScriptedDice::new([10]);
        let casting = casting(1, 2_147_483_647, Proficiency::Trained);
        assert_eq!(
            spell_attack_modifier(&casting, &[]),
            Err(ModifierError::Overflow)
        );
        assert_eq!(dice.remaining(), 1);
    }
}
