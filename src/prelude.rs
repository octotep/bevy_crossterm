pub use crate::{CrosstermPlugin, CrosstermWindow, CrosstermWindowSettings, Cursor, DefaultCrosstermPlugins};

pub use crate::components::{Color, Colors, Position, Sprite, SpriteBundle, Style, StyleMap, Visible};

// Re-export crossterm structs for easier access
pub use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent};
pub use crossterm::style::{Attribute, Attributes};
