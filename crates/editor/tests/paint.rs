use std::fs;

use content::Floor;
use editor::{EditorSession, Notice};

#[test]
fn painting_one_square_survives_save_and_load() {
    let root = std::env::temp_dir().join(format!("genesis-editor-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let path = root.join("room.json");
    fs::write(
        &path,
        "{\n  \"id\": \"booth\",\n  \"cell_feet\": 5,\n  \"rows\": [\"##\", \"#.\"]\n}\n",
    )
    .unwrap();

    let mut session = EditorSession::open(&path);
    assert_eq!(session.notice, Notice::Ready);
    assert_eq!(session.status(), "Paint floor · booth · 2×2 · 5 ft");
    session.brush = Floor::Grass;
    assert!(session.paint(1, 1));
    assert!(!session.paint(-1, 0));
    let room = session.room.as_ref().unwrap();
    assert!(!room.walkable(0, 0));
    assert_eq!(room.floor(1, 1), Some(Floor::Grass));
    assert_eq!(room.floor(0, 0), Some(Floor::Stone));
    session.save();
    assert_eq!(session.notice, Notice::Saved);
    assert_eq!(session.status(), "Saved · booth · 2×2 · 5 ft");

    let loaded = EditorSession::open(&path);
    let room = loaded.room.unwrap();
    assert_eq!(room.floor(1, 1), Some(Floor::Grass));
    assert_eq!(room.floor(0, 1), Some(Floor::Stone));
    assert!(!room.walkable(0, 1));
    assert!(room.walkable(1, 1));

    let missing = EditorSession::open(root.join("missing.json"));
    assert_eq!(missing.notice, Notice::Failed);
    assert!(missing.room.is_none());
    assert!(missing.status().contains("missing.json"));

    let _ = fs::remove_dir_all(&root);
}
