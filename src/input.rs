use crate::state::{
    app_state::{AppState, MenuFocus, PaneFocus},
    games_state::GamesFocus,
    playoffs_state::PlayoffsFocus,
};
/// Keyboard input handling
use crossterm::event::{KeyCode, KeyCode::Char, KeyEvent, KeyEventKind, KeyModifiers};

/// Actions that can be triggered by keyboard input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,

    ToggleDisplayMenu,

    SelectMenu(usize),

    PrevGame,
    NextGame,
    PrevGamesDisplay,
    NextGamesDisplay,
    GamesScrollUp,
    GamesScrollDown,
    GamesPageUp,
    GamesPageDown,
    BoxscoreUp,
    BoxscoreDown,
    BoxscorePageUp,
    BoxscorePageDown,
    BoxscoreForwards,
    BoxscoreDefensemen,
    BoxscoreGoalies,
    BoxscoreToggleTeam,

    /// Move up a row in standings table
    StandingsUp,
    /// Move down a row in standings table
    StandingsDown,
    /// Page up in standings table
    StandingsPageUp,
    /// Page down in standings table
    StandingsPageDown,
    /// Select previous (if possible) standings type
    StandingsLeft,
    /// Select next (if possible) standings type
    StandingsRight,
    /// Select previous (if possible) within a standings type
    PrevStandingsDisplay,
    /// Select next (if possible) standings type
    NextStandingsDisplay,

    // Scrolling in playoff bracket
    PlayoffsScrollUp,
    PlayoffsScrollDown,
    // Only bracket will have scroll left and right
    PlayoffsScrollLeft,
    PlayoffsScrollRight,
    // Page scrolling in playoffs page
    PlayoffsPageUp,
    PlayoffsPageDown,
    // Only bracket will have page left and right
    PlayoffsPageLeft,
    PlayoffsPageRight,

    /// Move up a row in team stats table
    TeamStatsUp,
    /// Move down a row in team stats table
    TeamStatsDown,
    /// Page up in team stats table
    TeamStatsPageUp,
    /// Page down in team stats table
    TeamStatsPageDown,
    /// Toggle between skaters and goalies
    ToggleTeamStats,

    DatePickerInputChar(char),
    EnterDatePicker,
    DateLeft,
    DateRight,
    DateBackspace,
    UpdateDate,
    UpdateYear,
    YearLeft,
    YearRight,
    ExitDatePicker,

    TeamPickerInputChar(char),
    EnterTeamPicker,
    TeamBackspace,
    UpdateTeam,
    ExitTeamPicker,

    SelectSeries(char),
    ExitSeries,

    EnterHelp,
    HelpScrollUp,
    HelpScrollDown,
    ExitHelp,

    None,
}

/// Map a key event to an application action.
///
/// Only handles `Press` and `Repeat` events — `Release` events are ignored
/// to prevent double-firing actions.
pub fn map_key(key_event: KeyEvent, state: &mut AppState) -> Action {
    // Only handle Press and Repeat events, ignore Release
    if !matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return Action::None;
    }
    match (
        &state.focus,
        &state.selected_menu,
        key_event.code,
        key_event.modifiers,
    ) {
        // Ctrl + c quits no matter what
        (_, _, Char('c'), KeyModifiers::CONTROL) => Action::Quit,

        // In DatePicker or TeamPicker, capture all input
        (PaneFocus::DatePicker, _, Char(c), _) => Action::DatePickerInputChar(c),
        (PaneFocus::TeamPicker, _, Char(c), _) => Action::TeamPickerInputChar(c),

        // q quits if we are not in any picker widgets
        (_, _, Char('q'), _) => Action::Quit,

        (PaneFocus::Content, _, KeyCode::Char('m'), _) => Action::ToggleDisplayMenu,
        (PaneFocus::Content, MenuFocus::TeamStats, KeyCode::Char('t'), _) => {
            Action::EnterTeamPicker
        }
        (PaneFocus::Content, _, KeyCode::Char(':'), _) => Action::EnterDatePicker,
        (PaneFocus::Content, _, KeyCode::Char('?'), _) => Action::EnterHelp,

        // (_, _, KeyCode::Char('r), _) => self.refresh(),

        // Selecting a menu
        (PaneFocus::Content, _, KeyCode::Char('1'), _) => Action::SelectMenu(1),
        (PaneFocus::Content, _, KeyCode::Char('2'), _) => Action::SelectMenu(2),
        (PaneFocus::Content, _, KeyCode::Char('3'), _) => Action::SelectMenu(3),
        (PaneFocus::Content, _, KeyCode::Char('4'), _) => Action::SelectMenu(4),

        // In games content pane
        (PaneFocus::Content, MenuFocus::Games, _, _) => {
            match (&state.games.focus, key_event.code, key_event.modifiers) {
                (_, KeyCode::Left | KeyCode::Char('h'), _) => Action::PrevGame,
                (_, KeyCode::Right | KeyCode::Char('l'), _) => Action::NextGame,
                (_, KeyCode::Char('<'), _) => Action::PrevGamesDisplay,
                (_, KeyCode::Char('>'), _) => Action::NextGamesDisplay,
                // Scoring or stats page actions
                (
                    GamesFocus::Scoring | GamesFocus::Stats,
                    KeyCode::Up | KeyCode::Char('K'),
                    KeyModifiers::SHIFT,
                ) => Action::GamesPageUp,
                (
                    GamesFocus::Scoring | GamesFocus::Stats,
                    KeyCode::Down | KeyCode::Char('J'),
                    KeyModifiers::SHIFT,
                ) => Action::GamesPageDown,
                (GamesFocus::Scoring | GamesFocus::Stats, KeyCode::Up | KeyCode::Char('k'), _) => {
                    Action::GamesScrollUp
                }
                (
                    GamesFocus::Scoring | GamesFocus::Stats,
                    KeyCode::Down | KeyCode::Char('j'),
                    _,
                ) => Action::GamesScrollDown,
                // Boxscore actions
                (GamesFocus::Boxscore, KeyCode::Up | KeyCode::Char('K'), KeyModifiers::SHIFT) => {
                    Action::BoxscorePageUp
                }
                (GamesFocus::Boxscore, KeyCode::Down | KeyCode::Char('J'), KeyModifiers::SHIFT) => {
                    Action::BoxscorePageDown
                }
                (GamesFocus::Boxscore, KeyCode::Up | KeyCode::Char('k'), _) => Action::BoxscoreUp,
                (GamesFocus::Boxscore, KeyCode::Down | KeyCode::Char('j'), _) => {
                    Action::BoxscoreDown
                }
                (GamesFocus::Boxscore, KeyCode::Char('f'), _) => Action::BoxscoreForwards,
                (GamesFocus::Boxscore, KeyCode::Char('d'), _) => Action::BoxscoreDefensemen,
                (GamesFocus::Boxscore, KeyCode::Char('g'), _) => Action::BoxscoreGoalies,
                (GamesFocus::Boxscore, KeyCode::Char('t'), _) => Action::BoxscoreToggleTeam,
                (_, _, _) => Action::None,
            }
        }

        // In standings content pane
        (
            PaneFocus::Content,
            MenuFocus::Standings,
            KeyCode::Up | KeyCode::Char('K'),
            KeyModifiers::SHIFT,
        ) => Action::StandingsPageUp,
        (
            PaneFocus::Content,
            MenuFocus::Standings,
            KeyCode::Down | KeyCode::Char('J'),
            KeyModifiers::SHIFT,
        ) => Action::StandingsPageDown,
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
        (PaneFocus::Content, MenuFocus::Standings, KeyCode::Char('<'), _) => {
            Action::PrevStandingsDisplay
        }
        (PaneFocus::Content, MenuFocus::Standings, KeyCode::Char('>'), _) => {
            Action::NextStandingsDisplay
        }

        // In team stats content page
        (
            PaneFocus::Content,
            MenuFocus::TeamStats,
            KeyCode::Up | KeyCode::Char('K'),
            KeyModifiers::SHIFT,
        ) => Action::TeamStatsPageUp,
        (
            PaneFocus::Content,
            MenuFocus::TeamStats,
            KeyCode::Down | KeyCode::Char('J'),
            KeyModifiers::SHIFT,
        ) => Action::TeamStatsPageDown,
        (PaneFocus::Content, MenuFocus::TeamStats, KeyCode::Up | KeyCode::Char('k'), _) => {
            Action::TeamStatsUp
        }
        (PaneFocus::Content, MenuFocus::TeamStats, KeyCode::Down | KeyCode::Char('j'), _) => {
            Action::TeamStatsDown
        }
        (PaneFocus::Content, MenuFocus::TeamStats, KeyCode::Char('>') | KeyCode::Char('<'), _) => {
            Action::ToggleTeamStats
        }

        // In playoffs content page
        (
            PaneFocus::Content,
            MenuFocus::Playoffs,
            KeyCode::Up | KeyCode::Char('K'),
            KeyModifiers::SHIFT,
        ) => Action::PlayoffsPageUp,
        (
            PaneFocus::Content,
            MenuFocus::Playoffs,
            KeyCode::Down | KeyCode::Char('J'),
            KeyModifiers::SHIFT,
        ) => Action::PlayoffsPageDown,
        (
            PaneFocus::Content,
            MenuFocus::Playoffs,
            KeyCode::Right | KeyCode::Char('L'),
            KeyModifiers::SHIFT,
        ) => Action::PlayoffsPageRight,
        (
            PaneFocus::Content,
            MenuFocus::Playoffs,
            KeyCode::Left | KeyCode::Char('H'),
            KeyModifiers::SHIFT,
        ) => Action::PlayoffsPageLeft,
        (PaneFocus::Content, MenuFocus::Playoffs, Char(c), _) => {
            // Can only choose series from bracket page
            if matches!(&state.playoffs.focus, PlayoffsFocus::Bracket) {
                Action::SelectSeries(c)
            } else {
                Action::None
            }
        }
        // No hjkl for scrolling since it's used to select series
        (PaneFocus::Content, MenuFocus::Playoffs, KeyCode::Up, _) => Action::PlayoffsScrollUp,
        (PaneFocus::Content, MenuFocus::Playoffs, KeyCode::Down, _) => Action::PlayoffsScrollDown,
        (PaneFocus::Content, MenuFocus::Playoffs, KeyCode::Right, _) => Action::PlayoffsScrollRight,
        (PaneFocus::Content, MenuFocus::Playoffs, KeyCode::Left, _) => Action::PlayoffsScrollLeft,
        (PaneFocus::Content, MenuFocus::Playoffs, KeyCode::Esc, _) => {
            // Go back to bracket page from series page
            if matches!(&state.playoffs.focus, PlayoffsFocus::Series) {
                Action::ExitSeries
            } else {
                Action::None
            }
        }

        // In date picker
        (PaneFocus::DatePicker, MenuFocus::Games | MenuFocus::Standings, KeyCode::Enter, _) => {
            Action::UpdateDate
        }
        (PaneFocus::DatePicker, MenuFocus::Games | MenuFocus::Standings, KeyCode::Left, _) => {
            Action::DateLeft
        }
        (PaneFocus::DatePicker, MenuFocus::Games | MenuFocus::Standings, KeyCode::Right, _) => {
            Action::DateRight
        }
        // In year picker
        (PaneFocus::DatePicker, MenuFocus::Playoffs | MenuFocus::TeamStats, KeyCode::Enter, _) => {
            Action::UpdateYear
        }
        (PaneFocus::DatePicker, MenuFocus::Playoffs | MenuFocus::TeamStats, KeyCode::Left, _) => {
            Action::YearLeft
        }
        (PaneFocus::DatePicker, MenuFocus::Playoffs | MenuFocus::TeamStats, KeyCode::Right, _) => {
            Action::YearRight
        }
        // Actions for both date and year picker
        (PaneFocus::DatePicker, _, KeyCode::Backspace, _) => Action::DateBackspace,
        (PaneFocus::DatePicker, _, KeyCode::Esc, _) => Action::ExitDatePicker,

        // In team picker
        (PaneFocus::TeamPicker, _, KeyCode::Enter, _) => Action::UpdateTeam,
        (PaneFocus::TeamPicker, _, KeyCode::Backspace, _) => Action::TeamBackspace,
        (PaneFocus::TeamPicker, _, KeyCode::Esc, _) => Action::ExitTeamPicker,

        // In help page
        (PaneFocus::Help, _, KeyCode::Up | KeyCode::Char('k'), _) => Action::HelpScrollUp,
        (PaneFocus::Help, _, KeyCode::Down | KeyCode::Char('j'), _) => Action::HelpScrollDown,
        (PaneFocus::Help, _, KeyCode::Esc, _) => Action::ExitHelp,

        _ => Action::None,
    }
}
