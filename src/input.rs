use crate::state::app_state::{MenuFocus, PaneFocus};
/// Keyboard input handling
use crossterm::event::{KeyCode, KeyCode::Char, KeyEvent, KeyEventKind, KeyModifiers};

/// Actions that can be triggered by keyboard input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    SwitchPaneFocus,
    EnterDatePicker,
    InputChar(char),

    MenuUp,
    MenuDown,

    GamesScrollUp,
    GamesScrollDown,
    PrevGame,
    NextGame,

    StandingsUp,
    StandingsDown,
    StandingsLeft,
    StandingsRight,
    PrevStandingsType,
    NextStandingsType,

    DateLeft,
    DateRight,
    DateBackspace,
    ExitDatePicker,
    UpdateDate,

    None,
}

/// Map a key event to an application action.
///
/// Only handles `Press` and `Repeat` events — `Release` events are ignored
/// to prevent double-firing actions.
pub fn map_key(key_event: KeyEvent, focus: PaneFocus, menu: MenuFocus) -> Action {
    // Only handle Press and Repeat events, ignore Release
    if !matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return Action::None;
    }
    match (focus, menu, key_event.code, key_event.modifiers) {
        // Ctrl + c quits no matter what
        (_, _, Char('c'), KeyModifiers::CONTROL) => Action::Quit,
        // q also quits no matter what
        (_, _, Char('q'), _) => Action::Quit,

        // In DatePicker, capture all input
        (PaneFocus::DatePicker, _, Char(c), _) => Action::InputChar(c),

        // Tab switches focus if not in DatePicker
        (PaneFocus::Content | PaneFocus::Menu, _, KeyCode::Tab, _) => Action::SwitchPaneFocus,
        (PaneFocus::Content | PaneFocus::Menu, _, KeyCode::Char(':'), _) => Action::EnterDatePicker,

        // (_, _, KeyCode::Char('r), _) => self.refresh(),

        // In menu pane
        (PaneFocus::Menu, _, KeyCode::Up | KeyCode::Char('k'), _) => Action::MenuUp,
        (PaneFocus::Menu, _, KeyCode::Down | KeyCode::Char('j'), _) => Action::MenuDown,

        // In Games content pane
        (PaneFocus::Content, MenuFocus::Games, KeyCode::Up | KeyCode::Char('k'), _) => {
            Action::GamesScrollUp
        }
        (PaneFocus::Content, MenuFocus::Games, KeyCode::Down | KeyCode::Char('j'), _) => {
            Action::GamesScrollDown
        }
        (PaneFocus::Content, MenuFocus::Games, KeyCode::Left | KeyCode::Char('h'), _) => {
            Action::PrevGame
        }
        (PaneFocus::Content, MenuFocus::Games, KeyCode::Right | KeyCode::Char('l'), _) => {
            Action::NextGame
        }

        // Toggle box score and overview
        // (PaneFocus::Content, MenuFocus::Games, KeyCode::Char(','), _) => {
        //     todo!()
        // }
        // (PaneFocus::Content, MenuFocus::Games, KeyCode::Char('.'), _) => {
        //     todo!()
        // }
        // // Change dates quickly
        // (PaneFocus::Content, MenuFocus::Games, KeyCode::Char('t'), _) => {
        //     todo!()
        // }
        // (PaneFocus::Content, MenuFocus::Games, KeyCode::Char('y'), _) => {
        //     todo!()
        // }

        // In standings content pane
        (PaneFocus::Content, MenuFocus::Standings, KeyCode::Up | KeyCode::Char('k'), _) => {
            Action::StandingsUp
        }
        (PaneFocus::Content, MenuFocus::Standings, KeyCode::Down | KeyCode::Char('j'), _) => {
            Action::StandingsDown
        }
        (PaneFocus::Content, MenuFocus::Standings, KeyCode::Left | KeyCode::Char('h'), _) => {
            Action::StandingsLeft
        }
        (PaneFocus::Content, MenuFocus::Standings, KeyCode::Right | KeyCode::Char('l'), _) => {
            Action::StandingsRight
        }
        (PaneFocus::Content, MenuFocus::Standings, KeyCode::Char(','), _) => {
            Action::PrevStandingsType
        }
        (PaneFocus::Content, MenuFocus::Standings, KeyCode::Char('.'), _) => {
            Action::NextStandingsType
        }

        // In teams content pane
        (PaneFocus::DatePicker, _, KeyCode::Enter, _) => Action::UpdateDate,
        (PaneFocus::DatePicker, _, KeyCode::Left, _) => Action::DateLeft,
        (PaneFocus::DatePicker, _, KeyCode::Right, _) => Action::DateRight,
        (PaneFocus::DatePicker, _, KeyCode::Backspace, _) => Action::DateBackspace,
        (PaneFocus::DatePicker, _, KeyCode::Esc, _) => Action::ExitDatePicker,

        _ => Action::None,
    }
}
