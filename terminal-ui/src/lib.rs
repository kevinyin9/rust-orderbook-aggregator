pub mod ui;
pub mod events;

use orderbook_merger::orderbook_summary::Summary;
use crossterm::event;

pub enum InputEvent {
    Input(Key),
    Update(Summary),
}

// Represents an key.
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Key {
    Char(char),
    Ctrl(char),
    Unknown
}

// This impl allows us to convert a `KeyEvent` into a `Key` enum value.
impl From<event::KeyEvent> for Key {
    fn from(key_event: event::KeyEvent) -> Self {
        match key_event {
            event::KeyEvent {
                code: event::KeyCode::Char(c),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            } => Key::Ctrl(c),

            _ => Key::Unknown,
        }
    }
}
