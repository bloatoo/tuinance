use crossterm::event::{self, KeyEvent};

#[derive(Debug, Clone, Copy)]
pub enum Key {
    Char(char),
    Ctrl(char),
    Up,
    Right,
    Down,
    Left,
    Backspace,
    Esc,
    Enter,
    Unknown,
}

impl From<KeyEvent> for Key {
    fn from(key_event: KeyEvent) -> Key {
        match key_event {
            KeyEvent {
                code: event::KeyCode::Esc,
                ..
            } => Key::Esc,

            KeyEvent {
                code: event::KeyCode::Enter,
                ..
            } => Key::Enter,

            KeyEvent {
                code: event::KeyCode::Backspace,
                ..
            } => Key::Backspace,

            KeyEvent {
                code: event::KeyCode::Left,
                ..
            } => Key::Left,

            KeyEvent {
                code: event::KeyCode::Right,
                ..
            } => Key::Right,

            KeyEvent {
                code: event::KeyCode::Up,
                ..
            } => Key::Up,

            KeyEvent {
                code: event::KeyCode::Down,
                ..
            } => Key::Down,

            KeyEvent {
                code: event::KeyCode::Char(c),
                modifiers: event::KeyModifiers::CONTROL
            } => Key::Ctrl(c),

            KeyEvent {
                code: event::KeyCode::Char(c),
                ..
            } => Key::Char(c),

            _ => Key::Unknown,

        }
    }
}
