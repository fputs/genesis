use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// One square address in a fixture room.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
}

/// Surface of one square. Walls stay in the room rows; this is only the floor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Floor {
    Stone,
    Grass,
    Dirt,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DamageSpec {
    pub dice: u8,
    pub faces: u8,
    pub bonus: i32,
}

/// A class feature named in a creature fixture. Any other name is rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Feature {
    ReactiveStrike,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrikeSpec {
    pub name: String,
    pub attack_bonus: i32,
    pub damage: DamageSpec,
    pub reach_feet: u32,
}

/// A creature stat block as authored in a fixture file.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Creature {
    pub id: String,
    pub name: String,
    pub level: u8,
    pub hit_points: i32,
    pub armor_class: i32,
    pub perception: i32,
    pub speed_feet: u32,
    pub strike: StrikeSpec,
    /// Missing lists stay empty. The only accepted name is `reactive_strike`.
    #[serde(default)]
    pub features: Vec<Feature>,
}

/// One placed creature. `name` is the name used in the encounter, not the template name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spawn {
    pub creature: Creature,
    pub name: String,
    pub x: i32,
    pub y: i32,
}

/// Walkable map. `#` in the source rows becomes a blocked cell. `.` is open. Each cell is 5 feet.
/// `floors` is row-major: index `y * width + x`, one floor per square.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Room {
    pub id: String,
    pub cell_feet: u32,
    pub width: i32,
    pub height: i32,
    pub blocked: Vec<Cell>,
    pub floors: Vec<Floor>,
}

impl Room {
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width && y < self.height
    }

    pub fn walkable(&self, x: i32, y: i32) -> bool {
        self.in_bounds(x, y) && !self.blocked.iter().any(|cell| cell.x == x && cell.y == y)
    }

    /// Floor at `(x, y)`, or `None` when the square is outside the grid.
    pub fn floor(&self, x: i32, y: i32) -> Option<Floor> {
        let index = self.index(x, y)?;
        self.floors.get(index).copied()
    }

    /// Paints `(x, y)`. Returns false when the square is outside the grid.
    pub fn set_floor(&mut self, x: i32, y: i32, floor: Floor) -> bool {
        let Some(index) = self.index(x, y) else {
            return false;
        };
        if index >= self.floors.len() {
            return false;
        }
        self.floors[index] = floor;
        true
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if !self.in_bounds(x, y) {
            return None;
        }
        Some((y * self.width + x) as usize)
    }
}

/// Proximity volume. Distance is Chebyshev squares times 5 feet.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trigger {
    pub x: i32,
    pub y: i32,
    pub radius_feet: u32,
}

/// A loaded, validated encounter. The party array is always four slots.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncounterFixture {
    pub id: String,
    pub rng_seed: u64,
    pub room: Room,
    pub party: [Spawn; 4],
    pub enemies: Vec<Spawn>,
    pub trigger: Trigger,
}

#[derive(Debug)]
pub enum LoadError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    Invalid {
        reasons: Vec<String>,
    },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Io { path, source } => write!(f, "reading {}: {source}", path.display()),
            LoadError::Write { path, source } => write!(f, "writing {}: {source}", path.display()),
            LoadError::Json { path, source } => write!(f, "parsing {}: {source}", path.display()),
            LoadError::Invalid { reasons } => write!(f, "{}", reasons.join("; ")),
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoadError::Io { source, .. } | LoadError::Write { source, .. } => Some(source),
            LoadError::Json { source, .. } => Some(source),
            LoadError::Invalid { .. } => None,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EncounterFile {
    id: String,
    room: String,
    rng_seed: u64,
    trigger: Trigger,
    party: Vec<SpawnFile>,
    enemies: Vec<SpawnFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpawnFile {
    creature: String,
    name: String,
    x: i32,
    y: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RoomFile {
    id: String,
    cell_feet: u32,
    rows: Vec<String>,
    /// Omitted floors are stone. Present rows are parsed here, then stored row-major on [`Room`].
    #[serde(default)]
    floors: Option<Vec<Vec<Floor>>>,
}

/// Loads an encounter JSON file and the creature and room files it points at.
///
/// Paths inside the file are relative to the encounter file's directory.
/// Validation fails closed: a bad party size, overlap, or blocked spawn is an error.
pub fn load_encounter(path: &Path) -> Result<EncounterFixture, LoadError> {
    let file: EncounterFile = read_json(path)?;
    let base = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let mut reasons = Vec::new();
    if file.id.trim().is_empty() {
        reasons.push("encounter id is empty".into());
    }
    if file.party.len() != 4 {
        reasons.push(format!(
            "party has {} members; encounters use 4 slots",
            file.party.len()
        ));
    }
    if file.enemies.len() != 1 {
        reasons.push(format!(
            "encounter has {} enemies; this loader expects 1",
            file.enemies.len()
        ));
    }

    let room = match read_json::<RoomFile>(&base.join(&file.room)) {
        Ok(room_file) => match room_from_file(room_file) {
            Ok(room) => Some(room),
            Err(reason) => {
                reasons.push(reason);
                None
            }
        },
        Err(error) => {
            reasons.push(error.to_string());
            None
        }
    };

    let mut party = Vec::new();
    for spawn in &file.party {
        match load_spawn(&base, spawn) {
            Ok(spawn) => party.push(spawn),
            Err(error) => reasons.push(error.to_string()),
        }
    }
    let mut enemies = Vec::new();
    for spawn in &file.enemies {
        match load_spawn(&base, spawn) {
            Ok(spawn) => enemies.push(spawn),
            Err(error) => reasons.push(error.to_string()),
        }
    }

    if let Some(room) = room.as_ref() {
        validate_placement(room, &file.trigger, &party, &enemies, &mut reasons);
    }
    for spawn in party.iter().chain(enemies.iter()) {
        validate_creature(&spawn.creature, &spawn.name, &mut reasons);
    }
    for spawn in &party {
        if spawn.creature.level != 1 {
            reasons.push(format!(
                "{} is level {}, party fixtures are level 1",
                spawn.name, spawn.creature.level
            ));
        }
    }

    if !reasons.is_empty() {
        return Err(LoadError::Invalid { reasons });
    }

    let Ok(party) = <[Spawn; 4]>::try_from(party) else {
        return Err(LoadError::Invalid {
            reasons: vec!["party did not resolve to 4 slots".into()],
        });
    };
    Ok(EncounterFixture {
        id: file.id,
        rng_seed: file.rng_seed,
        room: room.expect("room present when reasons are empty"),
        party,
        enemies,
        trigger: file.trigger,
    })
}

fn load_spawn(base: &Path, file: &SpawnFile) -> Result<Spawn, LoadError> {
    let creature: Creature = read_json(&base.join(&file.creature))?;
    Ok(Spawn {
        creature,
        name: file.name.clone(),
        x: file.x,
        y: file.y,
    })
}

fn room_from_file(file: RoomFile) -> Result<Room, String> {
    if file.id.trim().is_empty() {
        return Err("room id is empty".into());
    }
    if file.cell_feet != 5 {
        return Err(format!(
            "room {} uses {}-foot cells; squares are 5 feet",
            file.id, file.cell_feet
        ));
    }
    if file.rows.is_empty() {
        return Err(format!("room {} has no rows", file.id));
    }
    let width = file.rows[0].chars().count();
    if width == 0 {
        return Err(format!("room {} has an empty row", file.id));
    }
    let mut blocked = Vec::new();
    for (y, row) in file.rows.iter().enumerate() {
        let cells: Vec<char> = row.chars().collect();
        if cells.len() != width {
            return Err(format!(
                "room {} row {y} has length {}, expected {width}",
                file.id,
                cells.len()
            ));
        }
        for (x, cell) in cells.into_iter().enumerate() {
            match cell {
                '#' => blocked.push(Cell {
                    x: x as i32,
                    y: y as i32,
                }),
                '.' => {}
                other => {
                    return Err(format!(
                        "room {} has '{other}' at ({x}, {y}); use '#' or '.'",
                        file.id
                    ));
                }
            }
        }
    }
    let height = file.rows.len();
    let floors = floors_from_file(&file.id, width, height, file.floors)?;
    Ok(Room {
        id: file.id,
        cell_feet: file.cell_feet,
        width: width as i32,
        height: height as i32,
        blocked,
        floors,
    })
}

fn floors_from_file(
    id: &str,
    width: usize,
    height: usize,
    floors: Option<Vec<Vec<Floor>>>,
) -> Result<Vec<Floor>, String> {
    let Some(rows) = floors else {
        return Ok(vec![Floor::Stone; width * height]);
    };
    if rows.len() != height {
        return Err(format!(
            "room {id} has {} floor rows; the map has {height}",
            rows.len()
        ));
    }
    let mut flat = Vec::with_capacity(width * height);
    for (y, row) in rows.into_iter().enumerate() {
        if row.len() != width {
            return Err(format!(
                "room {id} floor row {y} has length {}, expected {width}",
                row.len()
            ));
        }
        flat.extend(row);
    }
    Ok(flat)
}

/// Loads one room file. Paths are the file itself; encounter files are not read.
pub fn load_room(path: &Path) -> Result<Room, LoadError> {
    let file: RoomFile = read_json(path)?;
    room_from_file(file).map_err(|reason| LoadError::Invalid {
        reasons: vec![reason],
    })
}

/// Writes `room` as JSON. `rows` keep blocked squares. `floors` is one row per map row.
pub fn save_room(path: &Path, room: &Room) -> Result<(), LoadError> {
    let mut reasons = Vec::new();
    if room.id.trim().is_empty() {
        reasons.push("room id is empty".into());
    }
    if room.cell_feet != 5 {
        reasons.push(format!(
            "room {} uses {}-foot cells; squares are 5 feet",
            room.id, room.cell_feet
        ));
    }
    if room.width <= 0 || room.height <= 0 {
        reasons.push(format!(
            "room {} is {}×{}; both axes must be positive",
            room.id, room.width, room.height
        ));
    }
    let expected = (room.width.max(0) as usize).saturating_mul(room.height.max(0) as usize);
    if room.floors.len() != expected {
        reasons.push(format!(
            "room {} has {} floors; a {}×{} grid has {expected}",
            room.id,
            room.floors.len(),
            room.width,
            room.height
        ));
    }
    for cell in &room.blocked {
        if !room.in_bounds(cell.x, cell.y) {
            reasons.push(format!(
                "room {} has a blocked square ({}, {}) outside the grid",
                room.id, cell.x, cell.y
            ));
        }
    }
    if !reasons.is_empty() {
        return Err(LoadError::Invalid { reasons });
    }

    let text = room_json(room);
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, &text).map_err(|source| LoadError::Write {
        path: tmp.clone(),
        source,
    })?;
    fs::rename(&tmp, path).map_err(|source| LoadError::Write {
        path: path.to_path_buf(),
        source,
    })
}

fn room_json(room: &Room) -> String {
    let mut out = String::new();
    out.push_str("{\n  \"id\": ");
    out.push_str(&serde_json::to_string(&room.id).unwrap());
    out.push_str(",\n  \"cell_feet\": ");
    out.push_str(&room.cell_feet.to_string());
    out.push_str(",\n  \"rows\": [\n");
    let rows = rows_of(room);
    for (index, row) in rows.iter().enumerate() {
        out.push_str("    ");
        out.push_str(&serde_json::to_string(row).unwrap());
        if index + 1 != rows.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ],\n  \"floors\": [\n");
    let width = room.width as usize;
    let height = room.height as usize;
    for y in 0..height {
        out.push_str("    [");
        for x in 0..width {
            if x > 0 {
                out.push_str(", ");
            }
            out.push_str(&serde_json::to_string(&room.floors[y * width + x]).unwrap());
        }
        out.push(']');
        if y + 1 != height {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n}\n");
    out
}

fn rows_of(room: &Room) -> Vec<String> {
    let mut blocked = vec![false; (room.width as usize) * (room.height as usize)];
    for cell in &room.blocked {
        if let Some(index) = room.index(cell.x, cell.y) {
            if let Some(flag) = blocked.get_mut(index) {
                *flag = true;
            }
        }
    }
    (0..room.height)
        .map(|y| {
            (0..room.width)
                .map(|x| {
                    let index = (y * room.width + x) as usize;
                    if blocked[index] {
                        '#'
                    } else {
                        '.'
                    }
                })
                .collect()
        })
        .collect()
}

fn validate_placement(
    room: &Room,
    trigger: &Trigger,
    party: &[Spawn],
    enemies: &[Spawn],
    reasons: &mut Vec<String>,
) {
    if !room.walkable(trigger.x, trigger.y) {
        reasons.push(format!(
            "trigger ({}, {}) is not a walkable square",
            trigger.x, trigger.y
        ));
    }
    if trigger.radius_feet == 0 {
        reasons.push("trigger radius is 0".into());
    }
    let mut seen = Vec::new();
    for spawn in party.iter().chain(enemies.iter()) {
        if spawn.name.trim().is_empty() {
            reasons.push("a spawn is missing a name".into());
        }
        if !room.walkable(spawn.x, spawn.y) {
            reasons.push(format!(
                "{} spawns on a blocked or out-of-bounds square ({}, {})",
                spawn.name, spawn.x, spawn.y
            ));
        }
        if seen.iter().any(|(x, y)| *x == spawn.x && *y == spawn.y) {
            reasons.push(format!(
                "{} overlaps another spawn at ({}, {})",
                spawn.name, spawn.x, spawn.y
            ));
        }
        seen.push((spawn.x, spawn.y));
    }
    let mut names = Vec::new();
    for spawn in party.iter().chain(enemies.iter()) {
        if names.iter().any(|name| name == &spawn.name) {
            reasons.push(format!("duplicate spawn name {}", spawn.name));
        }
        names.push(spawn.name.clone());
    }
}

fn validate_creature(creature: &Creature, label: &str, reasons: &mut Vec<String>) {
    if creature.id.trim().is_empty() {
        reasons.push(format!("{label} has an empty creature id"));
    }
    if creature.hit_points <= 0 {
        reasons.push(format!("{label} has hit points {}", creature.hit_points));
    }
    if creature.armor_class <= 0 {
        reasons.push(format!("{label} has armor class {}", creature.armor_class));
    }
    if creature.speed_feet == 0 || creature.speed_feet % 5 != 0 {
        reasons.push(format!(
            "{label} has speed {} feet; speed is a positive multiple of 5",
            creature.speed_feet
        ));
    }
    if creature.strike.name.trim().is_empty() {
        reasons.push(format!("{label} has an empty strike name"));
    }
    if creature.strike.reach_feet == 0 || creature.strike.reach_feet % 5 != 0 {
        reasons.push(format!(
            "{label} has reach {} feet; reach is a positive multiple of 5",
            creature.strike.reach_feet
        ));
    }
    if creature.strike.damage.dice == 0 || creature.strike.damage.faces < 2 {
        reasons.push(format!(
            "{label} has damage {}d{}",
            creature.strike.damage.dice, creature.strike.damage.faces
        ));
    }
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, LoadError> {
    let text = fs::read_to_string(path).map_err(|source| LoadError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| LoadError::Json {
        path: path.to_path_buf(),
        source,
    })
}
