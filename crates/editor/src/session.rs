use std::path::PathBuf;

use content::{load_room, save_room, Floor, Room};

/// What the status line is reporting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notice {
    Ready,
    Saved,
    Failed,
}

/// The open room, the selected floor, and the last load or save result.
pub struct EditorSession {
    pub path: PathBuf,
    pub room: Option<Room>,
    pub brush: Floor,
    pub notice: Notice,
    pub detail: String,
    pub generation: u32,
}

impl EditorSession {
    pub fn open(path: impl Into<PathBuf>) -> Self {
        let mut session = Self {
            path: path.into(),
            room: None,
            brush: Floor::Stone,
            notice: Notice::Ready,
            detail: String::new(),
            generation: 0,
        };
        session.reload();
        session
    }

    /// Reads the room file again. A failed read keeps the room that is already open.
    pub fn reload(&mut self) -> bool {
        match load_room(&self.path) {
            Ok(room) => {
                self.room = Some(room);
                self.notice = Notice::Ready;
                self.detail.clear();
                self.generation = self.generation.wrapping_add(1);
                true
            }
            Err(error) => {
                self.notice = Notice::Failed;
                self.detail = error.to_string();
                false
            }
        }
    }

    /// Paints the selected floor onto one square.
    pub fn paint(&mut self, x: i32, y: i32) -> bool {
        let Some(room) = self.room.as_mut() else {
            return false;
        };
        let before = room.floor(x, y);
        if !room.set_floor(x, y, self.brush) {
            return false;
        }
        if before != Some(self.brush) {
            self.notice = Notice::Ready;
            self.detail.clear();
        }
        true
    }

    /// Writes the open room back to its file.
    pub fn save(&mut self) {
        let Some(room) = self.room.clone() else {
            self.notice = Notice::Failed;
            self.detail = "no map to save".into();
            return;
        };
        match save_room(&self.path, &room) {
            Ok(()) => {
                self.notice = Notice::Saved;
                self.detail.clear();
            }
            Err(error) => {
                self.notice = Notice::Failed;
                self.detail = error.to_string();
            }
        }
    }

    pub fn status(&self) -> String {
        match self.notice {
            Notice::Failed => self.detail.clone(),
            Notice::Saved => match &self.room {
                Some(room) => format!(
                    "Saved · {} · {}×{} · {} ft",
                    room.id, room.width, room.height, room.cell_feet
                ),
                None => "Saved".into(),
            },
            Notice::Ready => status_line(self.room.as_ref()),
        }
    }

    pub fn map_name(&self) -> String {
        self.room
            .as_ref()
            .map(|room| room.id.clone())
            .unwrap_or_else(|| "—".into())
    }

    pub fn cell_size_label(&self) -> String {
        self.room
            .as_ref()
            .map(|room| format!("{} ft", room.cell_feet))
            .unwrap_or_else(|| "—".into())
    }
}

pub fn status_line(room: Option<&Room>) -> String {
    match room {
        Some(room) => format!(
            "Paint floor · {} · {}×{} · {} ft",
            room.id, room.width, room.height, room.cell_feet
        ),
        None => "Paint floor · no map".into(),
    }
}
