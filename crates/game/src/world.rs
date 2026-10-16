//! Room projection, formation, and the snap onto 5-foot squares.

use std::collections::HashSet;

use content::Room;

pub const TILE_W: f32 = 64.0;
pub const TILE_H: f32 = 32.0;

/// Diamond projection. +x on the grid runs down-right, +y runs down-left.
pub fn cell_to_world(x: i32, y: i32) -> (f32, f32) {
    let wx = (x - y) as f32 * (TILE_W * 0.5);
    let wy = -((x + y) as f32) * (TILE_H * 0.5);
    (wx, wy)
}

pub fn world_to_cell(wx: f32, wy: f32) -> (i32, i32) {
    let a = wx / (TILE_W * 0.5);
    let b = -wy / (TILE_H * 0.5);
    let gx = ((a + b) * 0.5).round() as i32;
    let gy = ((b - a) * 0.5).round() as i32;
    (gx, gy)
}

/// Offsets from the selected leader, in world units. Slot `leader` stays on the leader point.
pub fn formation_offset(slot: usize, leader: usize) -> (f32, f32) {
    const OFFSETS: [(f32, f32); 4] = [(0.0, 0.0), (-22.0, 12.0), (22.0, 12.0), (0.0, 24.0)];
    OFFSETS[(slot + 4 - (leader % 4)) % 4]
}

pub fn member_world(leader_world: (f32, f32), slot: usize, leader: usize) -> (f32, f32) {
    let (ox, oy) = formation_offset(slot, leader);
    (leader_world.0 + ox, leader_world.1 + oy)
}

/// Chebyshev distance in feet. Orthogonal and diagonal neighbors are both 5 feet.
pub fn chebyshev_feet(ax: i32, ay: i32, bx: i32, by: i32) -> u32 {
    let dx = (ax - bx).unsigned_abs();
    let dy = (ay - by).unsigned_abs();
    dx.max(dy) * 5
}

pub fn cell_in_trigger(x: i32, y: i32, trigger_x: i32, trigger_y: i32, radius_feet: u32) -> bool {
    chebyshev_feet(x, y, trigger_x, trigger_y) <= radius_feet
}

pub fn nearest_walkable(
    room: &Room,
    x: i32,
    y: i32,
    taken: &HashSet<(i32, i32)>,
) -> Option<(i32, i32)> {
    if room.walkable(x, y) && !taken.contains(&(x, y)) {
        return Some((x, y));
    }
    for radius in 1i32..64 {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs() != radius && dy.abs() != radius {
                    continue;
                }
                let next = (x + dx, y + dy);
                if room.walkable(next.0, next.1) && !taken.contains(&next) {
                    return Some(next);
                }
            }
        }
    }
    None
}

/// Places the enemy first, then the four party slots, on distinct walkable squares.
pub fn snap_to_grid(
    room: &Room,
    party: [(i32, i32); 4],
    enemy: (i32, i32),
) -> Result<([(i32, i32); 4], (i32, i32)), String> {
    let mut taken = HashSet::new();
    let enemy = nearest_walkable(room, enemy.0, enemy.1, &taken)
        .ok_or_else(|| "no square for the enemy".to_string())?;
    taken.insert(enemy);
    let mut snapped = [(0, 0); 4];
    for (slot, cell) in party.iter().enumerate() {
        let pos = nearest_walkable(room, cell.0, cell.1, &taken)
            .ok_or_else(|| format!("no square for party slot {slot}"))?;
        taken.insert(pos);
        snapped[slot] = pos;
    }
    Ok((snapped, enemy))
}

#[cfg(test)]
mod tests {
    use super::*;
    use content::load_encounter;
    use std::path::PathBuf;

    fn room_one() -> content::EncounterFixture {
        load_encounter(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/encounters/room_one.json"),
        )
        .unwrap()
    }

    #[test]
    fn cell_centers_round_trip() {
        for y in 0..8 {
            for x in 0..12 {
                let (wx, wy) = cell_to_world(x, y);
                assert_eq!(world_to_cell(wx, wy), (x, y));
            }
        }
    }

    #[test]
    fn opening_formation_is_outside_the_trigger() {
        let fixture = room_one();
        let leader = (fixture.party[0].x, fixture.party[0].y);
        let origin = cell_to_world(leader.0, leader.1);
        for slot in 0..4 {
            let (wx, wy) = member_world(origin, slot, 0);
            let (cx, cy) = world_to_cell(wx, wy);
            assert!(
                fixture.room.walkable(cx, cy),
                "slot {slot} landed on ({cx}, {cy})"
            );
            assert!(
                !cell_in_trigger(
                    cx,
                    cy,
                    fixture.trigger.x,
                    fixture.trigger.y,
                    fixture.trigger.radius_feet
                ),
                "slot {slot} starts inside the trigger"
            );
        }
    }
}
