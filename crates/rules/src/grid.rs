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

/// Walkable 5-foot squares. Blocked cells are walls.
#[derive(Clone, Debug)]
pub struct Grid {
    width: i32,
    height: i32,
    blocked: HashSet<GridPos>,
}

impl Grid {
    pub fn new(width: i32, height: i32, blocked: impl IntoIterator<Item = GridPos>) -> Self {
        Self {
            width,
            height,
            blocked: blocked.into_iter().collect(),
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
    let mut best: HashMap<(GridPos, u32), u32> = HashMap::new();
    let mut heap: BinaryHeap<(Reverse<u32>, GridPos, u32)> = BinaryHeap::new();
    best.insert((start, 0), 0);
    heap.push((Reverse(0), start, 0));

    let mut arrived: HashMap<GridPos, u32> = HashMap::new();

    while let Some((Reverse(cost), pos, diagonals)) = heap.pop() {
        if best.get(&(pos, diagonals)).copied().unwrap_or(u32::MAX) < cost {
            continue;
        }
        if pos != start && !cannot_end.contains(&pos) {
            arrived
                .entry(pos)
                .and_modify(|known| *known = (*known).min(cost))
                .or_insert(cost);
        }
        for (dx, dy) in NEIGHBORS {
            let next = GridPos::new(pos.x + dx, pos.y + dy);
            if !grid.walkable(next) || cannot_enter.contains(&next) {
                continue;
            }
            let (next_diagonals, step) = step_cost(diagonals, pos, next);
            let next_cost = cost + step;
            if next_cost > speed_feet {
                continue;
            }
            let key = (next, next_diagonals);
            if best.get(&key).copied().unwrap_or(u32::MAX) <= next_cost {
                continue;
            }
            best.insert(key, next_cost);
            heap.push((Reverse(next_cost), next, next_diagonals));
        }
    }

    let mut destinations: Vec<_> = arrived.into_iter().collect();
    destinations.sort_by_key(|(pos, _)| (pos.x, pos.y));
    destinations
}

#[cfg(test)]
mod tests {
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
    fn five_foot_reach_is_the_eight_neighbors() {
        let origin = GridPos::new(4, 4);
        let cells = squares_within_feet(origin, 5, 10, 10);
        assert_eq!(cells.len(), 8);
        assert!(cells.contains(&GridPos::new(5, 4)));
        assert!(cells.contains(&GridPos::new(5, 5)));
        assert!(!cells.contains(&GridPos::new(6, 4)));
    }
}
