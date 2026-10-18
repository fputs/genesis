use std::fs;
use std::path::PathBuf;

use content::{load_encounter, load_room, save_room, Floor, LoadError};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/encounters/room_one.json")
}

fn chebyshev_feet(ax: i32, ay: i32, bx: i32, by: i32) -> u32 {
    let dx = (ax - bx).unsigned_abs();
    let dy = (ay - by).unsigned_abs();
    dx.max(dy) * 5
}

#[test]
fn room_one_loads_four_fighters_and_one_guard() {
    let encounter = load_encounter(&fixture_path()).unwrap();
    assert_eq!(encounter.id, "room_one");
    assert_eq!(encounter.rng_seed, 115);
    assert_eq!(encounter.party.len(), 4);
    assert_eq!(encounter.enemies.len(), 1);
    assert_eq!(
        encounter.party.each_ref().map(|spawn| spawn.name.as_str()),
        ["Sera Voss", "Bram Holt", "Quill Ames", "Doss Kade"]
    );
    assert!(encounter
        .party
        .iter()
        .all(|spawn| spawn.creature.id == "fighter_l1" && spawn.creature.level == 1));
    let sera = &encounter.party[0].creature;
    assert_eq!(sera.armor_class, 18);
    assert_eq!(sera.hit_points, 20);
    assert_eq!(sera.perception, 5);
    assert_eq!(sera.speed_feet, 25);
    assert_eq!(sera.strike.name, "longsword");
    assert_eq!(sera.strike.attack_bonus, 7);
    assert_eq!(sera.strike.reach_feet, 5);
    assert_eq!(sera.strike.damage.dice, 1);
    assert_eq!(sera.strike.damage.faces, 8);
    assert_eq!(sera.strike.damage.bonus, 4);

    let guard = &encounter.enemies[0];
    assert_eq!(guard.name, "Hall Guard");
    assert_eq!(guard.creature.armor_class, 14);
    assert_eq!(guard.creature.hit_points, 18);
    assert_eq!(guard.x, 9);
    assert_eq!(guard.y, 4);

    assert_eq!(encounter.room.id, "small_room");
    assert_eq!(encounter.room.cell_feet, 5);
    assert_eq!(encounter.room.width, 12);
    assert_eq!(encounter.room.height, 8);
    assert_eq!(encounter.room.floors.len(), 12 * 8);
    assert!(encounter
        .room
        .floors
        .iter()
        .all(|floor| *floor == Floor::Stone));
    assert_eq!(encounter.room.floor(2, 2), Some(Floor::Stone));
    assert!(!encounter.room.walkable(0, 0));
    assert!(encounter.room.walkable(2, 2));
    assert!(encounter.room.walkable(6, 4));

    assert!(encounter.party.iter().all(|spawn| {
        chebyshev_feet(spawn.x, spawn.y, encounter.trigger.x, encounter.trigger.y)
            > encounter.trigger.radius_feet
    }));
    assert!(
        chebyshev_feet(guard.x, guard.y, encounter.trigger.x, encounter.trigger.y)
            > encounter.trigger.radius_feet
    );
}

#[test]
fn rejects_a_short_party_overlap_and_a_missing_field() {
    let root = std::env::temp_dir().join(format!("genesis-content-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("encounters")).unwrap();
    fs::create_dir_all(root.join("rooms")).unwrap();
    fs::create_dir_all(root.join("creatures")).unwrap();

    fs::write(
        root.join("rooms/small_room.json"),
        "{\n  \"id\": \"small_room\",\n  \"cell_feet\": 5,\n  \"rows\": [\"###\", \"#.#\", \"###\"]\n}\n",
    )
    .unwrap();
    fs::write(
        root.join("creatures/fighter_l1.json"),
        fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/creatures/fighter_l1.json"),
        )
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("creatures/broken.json"),
        r#"{
  "id": "broken",
  "name": "Broken",
  "level": 1,
  "hit_points": 10,
  "perception": 0,
  "speed_feet": 25,
  "strike": {
    "name": "fist",
    "attack_bonus": 0,
    "damage": { "dice": 1, "faces": 4, "bonus": 0 },
    "reach_feet": 5
  }
}"#,
    )
    .unwrap();

    let short = root.join("encounters/short.json");
    fs::write(
        &short,
        r#"{
  "id": "short",
  "room": "../rooms/small_room.json",
  "rng_seed": 1,
  "trigger": { "x": 1, "y": 1, "radius_feet": 5 },
  "party": [
    { "creature": "../creatures/fighter_l1.json", "name": "Sera Voss", "x": 1, "y": 1 },
    { "creature": "../creatures/fighter_l1.json", "name": "Bram Holt", "x": 1, "y": 1 },
    { "creature": "../creatures/fighter_l1.json", "name": "Quill Ames", "x": 1, "y": 1 }
  ],
  "enemies": [
    { "creature": "../creatures/fighter_l1.json", "name": "Hall Guard", "x": 1, "y": 1 }
  ]
}"#,
    )
    .unwrap();
    let error = load_encounter(&short).unwrap_err();
    let text = error.to_string();
    assert!(text.contains("4 slots"), "{text}");
    assert!(text.contains("overlaps"), "{text}");

    let broken = root.join("encounters/broken.json");
    fs::write(
        &broken,
        r#"{
  "id": "broken",
  "room": "../rooms/small_room.json",
  "rng_seed": 1,
  "trigger": { "x": 1, "y": 1, "radius_feet": 5 },
  "party": [
    { "creature": "../creatures/broken.json", "name": "Sera Voss", "x": 1, "y": 1 },
    { "creature": "../creatures/fighter_l1.json", "name": "Bram Holt", "x": 1, "y": 1 },
    { "creature": "../creatures/fighter_l1.json", "name": "Quill Ames", "x": 1, "y": 1 },
    { "creature": "../creatures/fighter_l1.json", "name": "Doss Kade", "x": 1, "y": 1 }
  ],
  "enemies": [
    { "creature": "../creatures/fighter_l1.json", "name": "Hall Guard", "x": 1, "y": 1 }
  ]
}"#,
    )
    .unwrap();
    let error = load_encounter(&broken).unwrap_err();
    assert!(matches!(error, LoadError::Invalid { .. }), "{error}");
    assert!(error.to_string().contains("armor_class"), "{error}");

    let missing = root.join("encounters/missing.json");
    let error = load_encounter(&missing).unwrap_err();
    assert!(matches!(error, LoadError::Io { .. }), "{error}");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn floors_default_to_stone_and_round_trip() {
    let root = std::env::temp_dir().join(format!("genesis-floors-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    let plain = root.join("plain.json");
    fs::write(
        &plain,
        "{\n  \"id\": \"plain\",\n  \"cell_feet\": 5,\n  \"rows\": [\"##\", \"#.\"]\n}\n",
    )
    .unwrap();
    let room = load_room(&plain).unwrap();
    assert_eq!(room.floors, vec![Floor::Stone; 4]);
    assert!(!room.walkable(0, 0));
    assert!(room.walkable(1, 1));

    let painted = root.join("painted.json");
    fs::write(
        &painted,
        "{\n  \"id\": \"painted\",\n  \"cell_feet\": 5,\n  \"rows\": [\"##\", \"#.\"],\n  \"floors\": [[\"stone\", \"grass\"], [\"dirt\", \"stone\"]]\n}\n",
    )
    .unwrap();
    let mut room = load_room(&painted).unwrap();
    assert_eq!(
        room.floors,
        vec![Floor::Stone, Floor::Grass, Floor::Dirt, Floor::Stone]
    );
    assert!(room.set_floor(1, 1, Floor::Grass));
    assert!(!room.set_floor(4, 4, Floor::Dirt));
    save_room(&painted, &room).unwrap();
    let again = load_room(&painted).unwrap();
    assert_eq!(again, room);
    assert_eq!(again.floor(1, 1), Some(Floor::Grass));
    assert!(!again.walkable(0, 1));

    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/rooms/small_room.json");
    let original = fs::read_to_string(&fixture).unwrap();
    let saved = root.join("small_room.json");
    save_room(&saved, &load_room(&fixture).unwrap()).unwrap();
    assert_eq!(fs::read_to_string(&saved).unwrap(), original);

    let short = root.join("short.json");
    fs::write(
        &short,
        "{\n  \"id\": \"short\",\n  \"cell_feet\": 5,\n  \"rows\": [\"##\", \"#.\"],\n  \"floors\": [[\"stone\"]]\n}\n",
    )
    .unwrap();
    let error = load_room(&short).unwrap_err();
    assert!(error.to_string().contains("floor rows"), "{error}");

    let lava = root.join("lava.json");
    fs::write(
        &lava,
        r#"{
  "id": "lava",
  "cell_feet": 5,
  "rows": ["."],
  "floors": [["lava"]]
}
"#,
    )
    .unwrap();
    let error = load_room(&lava).unwrap_err();
    assert!(matches!(error, LoadError::Json { .. }), "{error}");
    assert!(error.to_string().contains("lava"), "{error}");

    let _ = fs::remove_dir_all(&root);
}
