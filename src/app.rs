use crate::input::{Action, map_key};
use crate::sources::AppEvent;
use ratatui::widgets::TableState;

const MENU_LEN: usize = 3;
const NUM_TEAMS: usize = 32;

/// Which pane currently has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneFocus {
    Menu,
    Information,
}

impl PaneFocus {
    pub fn next(self) -> Self {
        match self {
            PaneFocus::Menu => PaneFocus::Information,
            PaneFocus::Information => PaneFocus::Menu,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            PaneFocus::Menu => PaneFocus::Information,
            PaneFocus::Information => PaneFocus::Menu,
        }
    }
}

pub struct App {
    /// Current data for league standings
    pub league_standings: Option<String>,
    /// State for league standings table
    pub league_standings_table_state: TableState,
    /// Whether we should quit the application
    pub should_quit: bool,

    /// Selected index in the menu
    pub menu_index: usize,

    /// Currently focused pane.
    pub focus: PaneFocus,
}

impl App {
    pub fn new() -> App {
        let mut table_state = TableState::default();
        table_state.select(Some(0));

        App {
            league_standings: None,
            league_standings_table_state: table_state,
            should_quit: false,
            menu_index: 0,
            focus: PaneFocus::Menu,
        }
    }

    // Handle an incoming event and update state accordingly
    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::StandingsUpdate(standings) => {
                log::info!("Updating standings");
                self.league_standings = Some(standings);
            }
            AppEvent::Input(key_event) => {
                log::info!("Key event detected: {:?}", key_event);
                if let Some(action) = map_key(key_event) {
                    self.handle_action(action);
                }
            }
            AppEvent::Tick => {
                // No-op for now; UI refresh is driven by render calls
            }
        }
    }

    /// Handle a mapped action.
    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::MoveUp => self.move_selection(-1),
            Action::MoveDown => self.move_selection(1),
            Action::FocusLeft => self.focus = self.focus.prev(),
            Action::FocusRight => self.focus = self.focus.next(),
            Action::CycleFocus => self.focus = self.focus.next(),
            Action::Refresh => {
                // Sources auto-refresh; this is a UI hint
            }
            Action::Quit => {
                self.should_quit = true;
            }
        }
    }

    /// Move the selection in the focused pane by `delta` (+1 = down, -1 = up).
    fn move_selection(&mut self, delta: i32) {
        match self.focus {
            PaneFocus::Menu => {
                self.menu_index = wrap_index(self.menu_index, delta, MENU_LEN);
            }
            PaneFocus::Information => {
                let current = self.league_standings_table_state.selected().unwrap_or(0);

                let new_index = wrap_index(current, delta, NUM_TEAMS);

                self.league_standings_table_state.select(Some(new_index));
            }
        }
    }
}

/// Wrap an index by delta within [0, len), wrapping around at boundaries.
fn wrap_index(current: usize, delta: i32, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let new = current as i32 + delta;
    if new < 0 {
        len - 1
    } else if new >= len as i32 {
        0
    } else {
        new as usize
    }
}
