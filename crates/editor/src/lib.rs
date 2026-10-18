//! Floor map editor. Loads a room, paints stone, grass, or dirt, and saves that room.

mod iso;
mod session;
mod view;

pub use session::{status_line, EditorSession, Notice};
pub use view::run;
