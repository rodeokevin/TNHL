/// Keyboard input handling
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Actions that can be triggered by keyboard input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Move selection up in the focused pane.
    MoveUp,
    /// Move selection down in the focused pane.
    MoveDown,
    /// Focus the previous (left) pane.
    FocusLeft,
    /// Focus the next (right) pane.
    FocusRight,
    /// Cycle focus to the next pane.
    CycleFocus,
    /// Force refresh all data sources.
    Refresh,
    /// Quit the application.
    Quit,
}

/// Map a key event to an application action.
///
/// Only handles `Press` and `Repeat` events — `Release` events are ignored
/// to prevent double-firing actions.
pub fn map_key(event: KeyEvent) -> Option<Action> {
    // Only handle Press and Repeat events, ignore Release
    if !matches!(event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return None;
    }

    // Ctrl+C always quits
    if event.modifiers.contains(KeyModifiers::CONTROL) && event.code == KeyCode::Char('c') {
        return Some(Action::Quit);
    }

    match event.code {
        // Navigation
        KeyCode::Up | KeyCode::Char('k') => Some(Action::MoveUp),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::MoveDown),
        KeyCode::Left | KeyCode::Char('h') => Some(Action::FocusLeft),
        KeyCode::Right | KeyCode::Char('l') => Some(Action::FocusRight),
        KeyCode::Tab => Some(Action::CycleFocus),

        // Actions
        KeyCode::Char('r') => Some(Action::Refresh),
        KeyCode::Char('q') | KeyCode::Esc => Some(Action::Quit),

        _ => None,
    }
}
