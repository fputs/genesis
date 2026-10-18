use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// One side of a combat square, in feet.
pub const FEET_PER_SQUARE: u32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Ground in one square. Missing entries are [`Terrain::Normal`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Terrain {
    #[default]
    Normal,
    Difficult,
    Greater,
}

impl Terrain {
    /// Extra feet to enter this square, on top of its diagonal or orthogonal cost.
    pub fn extra_feet(self) -> u32 {
        match self {
            Terrain::Normal => 0,
            Terrain::Difficult => 5,
            Terrain::Greater => 10,
        }
    }
}

/// Walkable 5-foot squares. Blocked cells are walls. Terrain defaults to normal.
#[derive(Clone, Debug)]
pub struct Grid {
    width: i32,
    height: i32,
    blocked: HashSet<GridPos>,
    terrain: HashMap<GridPos, Terrain>,
}

impl Grid {
    pub fn new(width: i32, height: i32, blocked: impl IntoIterator<Item = GridPos>) -> Self {
        Self {
            width,
            height,
            blocked: blocked.into_iter().collect(),
            terrain: HashMap::new(),
        }
    }

    pub fn terrain(&self, pos: GridPos) -> Terrain {
        self.terrain.get(&pos).copied().unwrap_or_default()
    }

    /// Marks one square. [`Terrain::Normal`] clears a previous mark.
    pub fn set_terrain(&mut self, pos: GridPos, terrain: Terrain) {
        if terrain == Terrain::Normal {
            self.terrain.remove(&pos);
        } else {
            self.terrain.insert(pos, terrain);
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn in_bounds(&self, pos: GridPos) -> bool {
        pos.x >= 0 && pos.y >= 0 && pos.x < self.width && pos.y < self.height
    }

    pub fn walkable(&self, pos: GridPos) -> bool {
        self.in_bounds(pos) && !self.blocked.contains(&pos)
    }
}

const NEIGHBORS: [(i32, i32); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

fn step_cost(diagonals_before: u32, from: GridPos, to: GridPos) -> (u32, u32) {
    let diagonal = from.x != to.x && from.y != to.y;
    if diagonal {
        let diagonals = diagonals_before + 1;
        let feet = if diagonals % 2 == 1 { 5 } else { 10 };
        (diagonals, feet)
    } else {
        (diagonals_before, 5)
    }
}

/// Minimum feet between two squares on an open plane.
///
/// Every second diagonal step costs 10 feet; orthogonal steps cost 5.
pub fn minimum_feet(from: GridPos, to: GridPos) -> u32 {
    let dx = (to.x - from.x).unsigned_abs();
    let dy = (to.y - from.y).unsigned_abs();
    let diagonals = dx.min(dy);
    let orthogonal = dx.max(dy) - diagonals;
    let mut cost = orthogonal * FEET_PER_SQUARE;
    for index in 1..=diagonals {
        cost += if index % 2 == 1 { 5 } else { 10 };
    }
    cost
}

/// Squares whose open-plane distance from `origin` is in `1..=feet`.
pub fn squares_within_feet(origin: GridPos, feet: u32, width: i32, height: i32) -> Vec<GridPos> {
    if feet == 0 {
        return Vec::new();
    }
    let span = (feet / FEET_PER_SQUARE) as i32 + 1;
    let mut cells = Vec::new();
    for y in (origin.y - span)..=(origin.y + span) {
        for x in (origin.x - span)..=(origin.x + span) {
            let pos = GridPos::new(x, y);
            if pos.x < 0 || pos.y < 0 || pos.x >= width || pos.y >= height || pos == origin {
                continue;
            }
            if minimum_feet(origin, pos) <= feet {
                cells.push(pos);
            }
        }
    }
    cells.sort();
    cells
}

struct StrideSearch {
    /// Cheapest arrival at a square that can be a destination: `(feet, diagonal count)`.
    arrived: HashMap<GridPos, (u32, u32)>,
    best: HashMap<(GridPos, u32), u32>,
    parent: HashMap<(GridPos, u32), (GridPos, u32)>,
}

/// One cost table for Stride destinations and for the squares a route leaves.
fn stride_search(
    grid: &Grid,
    start: GridPos,
    speed_feet: u32,
    cannot_enter: &HashSet<GridPos>,
    cannot_end: &HashSet<GridPos>,
) -> StrideSearch {
    let mut best: HashMap<(GridPos, u32), u32> = HashMap::new();
    let mut parent: HashMap<(GridPos, u32), (GridPos, u32)> = HashMap::new();
    let mut heap: BinaryHeap<(Reverse<u32>, GridPos, u32)> = BinaryHeap::new();
    best.insert((start, 0), 0);
    heap.push((Reverse(0), start, 0));

    let mut arrived: HashMap<GridPos, (u32, u32)> = HashMap::new();

    while let Some((Reverse(cost), pos, diagonals)) = heap.pop() {
        if best.get(&(pos, diagonals)).copied().unwrap_or(u32::MAX) < cost {
            continue;
        }
        if pos != start && !cannot_end.contains(&pos) {
            match arrived.get_mut(&pos) {
                Some((known, known_diagonals)) => {
                    if cost < *known {
                        *known = cost;
                        *known_diagonals = diagonals;
                    }
                }
                None => {
                    arrived.insert(pos, (cost, diagonals));
                }
            }
        }
        for (dx, dy) in NEIGHBORS {
            let next = GridPos::new(pos.x + dx, pos.y + dy);
            if !grid.walkable(next) || cannot_enter.contains(&next) {
                continue;
            }
            let (next_diagonals, step) = step_cost(diagonals, pos, next);
            let next_cost = cost + step + grid.terrain(next).extra_feet();
            if next_cost > speed_feet {
                continue;
            }
            let key = (next, next_diagonals);
            if best.get(&key).copied().unwrap_or(u32::MAX) <= next_cost {
                continue;
            }
            best.insert(key, next_cost);
            parent.insert(key, (pos, diagonals));
            heap.push((Reverse(next_cost), next, next_diagonals));
        }
    }

    StrideSearch {
        arrived,
        best,
        parent,
    }
}

/// Destinations reachable by a Stride of at most `speed_feet`.
///
/// `cannot_enter` blocks the path (enemies, walls are already non-walkable).
/// `cannot_end` forbids stopping (other living creatures, including allies).
/// The start square is not a destination. Each result is `(square, feet)`.
pub fn reachable_destinations(
    grid: &Grid,
    start: GridPos,
    speed_feet: u32,
    cannot_enter: &HashSet<GridPos>,
    cannot_end: &HashSet<GridPos>,
) -> Vec<(GridPos, u32)> {
    let search = stride_search(grid, start, speed_feet, cannot_enter, cannot_end);
    let mut destinations: Vec<_> = search
        .arrived
        .iter()
        .map(|(pos, (cost, _))| (*pos, *cost))
        .collect();
    destinations.sort_by_key(|(pos, _)| (pos.x, pos.y));
    destinations
}

/// Squares a Stride to `destination` leaves, in walk order, from the same search as
/// [`reachable_destinations`].
///
/// Each pair is `(square, feet already spent to stand there)`. The returned cost is the
/// destination cost from that search. `None` when `destination` is not reachable.
pub(crate) fn stride_leaves(
    grid: &Grid,
    start: GridPos,
    destination: GridPos,
    speed_feet: u32,
    cannot_enter: &HashSet<GridPos>,
    cannot_end: &HashSet<GridPos>,
) -> Option<(u32, Vec<(GridPos, u32)>)> {
    let search = stride_search(grid, start, speed_feet, cannot_enter, cannot_end);
    let (feet, mut diagonals) = search.arrived.get(&destination).copied()?;
    let mut pos = destination;
    let mut chain = Vec::new();
    loop {
        let cost = if pos == start {
            0
        } else {
            search.best.get(&(pos, diagonals)).copied()?
        };
        chain.push((pos, cost));
        if pos == start {
            break;
        }
        let prev = search.parent.get(&(pos, diagonals)).copied()?;
        pos = prev.0;
        diagonals = prev.1;
    }
    chain.reverse();
    chain.pop();
    Some((feet, chain))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    /// Player Core, "Movement": the first diagonal in a move costs 5 feet and every second diagonal costs 10.
    #[test]
    fn diagonal_steps_alternate_five_and_ten() {
        let origin = GridPos::new(0, 0);
        assert_eq!(minimum_feet(origin, GridPos::new(1, 0)), 5);
        assert_eq!(minimum_feet(origin, GridPos::new(1, 1)), 5);
        assert_eq!(minimum_feet(origin, GridPos::new(2, 2)), 15);
        assert_eq!(minimum_feet(origin, GridPos::new(3, 3)), 20);
        assert_eq!(minimum_feet(origin, GridPos::new(4, 4)), 30);
        assert_eq!(minimum_feet(origin, GridPos::new(5, 0)), 25);
        assert_eq!(minimum_feet(origin, GridPos::new(3, 1)), 15);
    }

    #[test]
    fn squares_start_normal_until_marked() {
        let mut grid = Grid::new(3, 3, []);
        assert_eq!(grid.terrain(GridPos::new(1, 1)), Terrain::Normal);
        grid.set_terrain(GridPos::new(1, 1), Terrain::Difficult);
        grid.set_terrain(GridPos::new(2, 2), Terrain::Greater);
        assert_eq!(grid.terrain(GridPos::new(1, 1)), Terrain::Difficult);
        assert_eq!(grid.terrain(GridPos::new(2, 2)), Terrain::Greater);
        assert_eq!(grid.terrain(GridPos::new(0, 0)), Terrain::Normal);
        assert_eq!(Terrain::Difficult.extra_feet(), 5);
        assert_eq!(Terrain::Greater.extra_feet(), 10);
    }

    #[test]
    fn five_foot_reach_is_the_eight_neighbors() {
        let origin = GridPos::new(4, 4);
        let cells = squares_within_feet(origin, 5, 10, 10);
        assert_eq!(cells.len(), 8);
        assert!(cells.contains(&GridPos::new(5, 4)));
        assert!(cells.contains(&GridPos::new(5, 5)));
        assert!(!cells.contains(&GridPos::new(6, 4)));
    }

    #[test]
    fn stride_leaves_the_only_route_at_the_reachable_cost() {
        let blocked: Vec<GridPos> = (0..5)
            .flat_map(|x| [GridPos::new(x, 0), GridPos::new(x, 2)])
            .collect();
        let grid = Grid::new(5, 3, blocked);
        let start = GridPos::new(0, 1);
        let destination = GridPos::new(3, 1);
        let empty = HashSet::new();
        let reached = reachable_destinations(&grid, start, 25, &empty, &empty);
        let feet = reached
            .iter()
            .find(|(pos, _)| *pos == destination)
            .unwrap()
            .1;
        assert_eq!(feet, 15);
        let (route_feet, left) =
            stride_leaves(&grid, start, destination, 25, &empty, &empty).unwrap();
        assert_eq!(route_feet, feet);
        assert_eq!(
            left,
            vec![
                (GridPos::new(0, 1), 0),
                (GridPos::new(1, 1), 5),
                (GridPos::new(2, 1), 10),
            ]
        );
    }

    #[test]
    fn every_reachable_stride_uses_one_search_cost() {
        let grid = Grid::new(8, 8, [GridPos::new(2, 2), GridPos::new(3, 1)]);
        let start = GridPos::new(1, 1);
        let cannot_enter = HashSet::from([GridPos::new(4, 4)]);
        let cannot_end = HashSet::from([GridPos::new(1, 2)]);
        let reached = reachable_destinations(&grid, start, 30, &cannot_enter, &cannot_end);
        assert!(!reached.is_empty());
        for (destination, feet) in reached {
            let (route_feet, left) =
                stride_leaves(&grid, start, destination, 30, &cannot_enter, &cannot_end).unwrap();
            assert_eq!(route_feet, feet);
            assert_eq!(left[0], (start, 0));
            let mut previous = start;
            let mut previous_cost = 0;
            for (pos, cost) in left.iter().skip(1).copied() {
                assert!(neighbors(previous, pos));
                assert!(cost > previous_cost);
                previous = pos;
                previous_cost = cost;
            }
            assert!(neighbors(previous, destination));
            assert!(previous_cost < feet);
        }
    }

    fn neighbors(from: GridPos, to: GridPos) -> bool {
        let dx = (from.x - to.x).abs();
        let dy = (from.y - to.y).abs();
        dx <= 1 && dy <= 1 && dx + dy > 0
    }
}
