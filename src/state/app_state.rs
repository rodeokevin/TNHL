use crate::input::{Action, map_key};
use crate::sources::AppEvent;
use crate::state::standings_state::StandingsState;
use crate::state::date_input::DateInput;
use crate::models::games::GamesResponse;
use crate::models::standings::StandingsResponse;

/// Which pane currently has keyboard focus.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PaneFocus {
    #[default]
    Menu,
    Content,
}

impl PaneFocus {
    pub fn switch(self) -> Self {
        match self {
            PaneFocus::Menu => PaneFocus::Content,
            PaneFocus::Content => PaneFocus::Menu,
        }
    }
}

/// Which menu item is currently selected.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MenuFocus {
    #[default]
    Games,
    Standings,
    Teams,
}

impl MenuFocus {
    pub fn next(self) -> Self {
        match self {
            MenuFocus::Games => MenuFocus::Standings,
            MenuFocus::Standings => MenuFocus::Teams,
            MenuFocus::Teams => MenuFocus::Teams,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            MenuFocus::Standings => MenuFocus::Games,
            MenuFocus::Teams => MenuFocus::Standings,
            MenuFocus::Games => MenuFocus::Games,
        }
    }
    pub fn index(&self) -> usize {
        match self {
            MenuFocus::Games => 0,
            MenuFocus::Standings => 1,
            MenuFocus::Teams => 2,
        }
    }
}

pub struct AppState {
    pub date_input: DateInput,

    pub selected_menu: MenuFocus,
    pub standings: StandingsState,
    pub league_data: Option<StandingsResponse>,

    pub games_data: Option<GamesResponse>,
    pub selected_game_index: usize,
    pub sweeping_status_offset: usize, // For the --- under the time remaining
    pub scoring_scroll_offset: usize,
    pub max_scoring_scroll: usize,

    pub focus: PaneFocus,
    pub should_quit: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            date_input: DateInput::default(),

            selected_menu: MenuFocus::default(),
            standings: StandingsState::default(),
            league_data: None,

            games_data: None,
            selected_game_index: 0,
            sweeping_status_offset: 0,
            scoring_scroll_offset: 0,
            max_scoring_scroll: 0,

            focus: PaneFocus::default(),
            should_quit: false,
        }
    }
}

impl AppState {
    // Handle an incoming event and update state accordingly
    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::StandingsUpdate(data) => {
                log::info!("Updating standings data");
                match StandingsResponse::from_json(&data) {
                    Ok(parsed_standings) => self.league_data = Some(parsed_standings),
                    Err(e) => log::error!("Failed to parse standings: {}", e),
                }
            }
            AppEvent::GamesUpdate(data) => {
                log::info!("Updating games data");
                match GamesResponse::from_json(&data) {
                    Ok(parsed_games) => {
                        self.games_data = Some(parsed_games);
                    }
                    Err(e) => log::error!("Failed to parse games: {}", e),
                }
            }
            AppEvent::Input(key_event) => {
                log::info!("Key event detected: {:?}", key_event);
                if let Some(action) = map_key(key_event) {
                    self.handle_action(action);
                }
            }
            AppEvent::Tick => {
                self.sweeping_status_offset = self.sweeping_status_offset.wrapping_add(1);
            }
        }
    }

    /// Handle a mapped action.
    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::MoveUp => self.move_selection(-1),
            Action::MoveDown => self.move_selection(1),
            Action::FocusLeft | Action::FocusRight if self.focus == PaneFocus::Content => {
                let delta = if matches!(action, Action::FocusRight) {
                    1
                } else {
                    -1
                };
                match self.selected_menu {
                    MenuFocus::Games => {
                        let len = self.games_data.as_ref().map_or(0, |d| d.games.len());
                        let prev = self.selected_game_index;
                        self.selected_game_index =
                            change_index(self.selected_game_index, delta, len);
                        if self.selected_game_index != prev {
                            self.scoring_scroll_offset = 0;
                            self.max_scoring_scroll = 0;
                        }
                    }
                    MenuFocus::Standings => self.standings.shift_standings_type(delta == 1),
                    MenuFocus::Teams => {}
                }
            }
            Action::CycleFocus => self.focus = self.focus.switch(),
            Action::NextStandings | Action::PrevStandings if self.focus == PaneFocus::Content => {
                self.standings
                    .cycle_focus(matches!(action, Action::NextStandings));
            }
            Action::Quit => self.should_quit = true,
            Action::Refresh | _ => {}
        }
    }

    /// Move the selection in the focused pane by `delta` (+1 = down, -1 = up).
    fn move_selection(&mut self, delta: i32) {
        match self.focus {
            PaneFocus::Menu => {
                self.selected_menu = if delta == 1 {
                    self.selected_menu.next()
                } else {
                    self.selected_menu.prev()
                };
            }
            PaneFocus::Content => match self.selected_menu {
                MenuFocus::Standings => self.standings.move_selection(delta),
                MenuFocus::Games => {
                    self.scoring_scroll_offset = if delta == 1 {
                        self.scoring_scroll_offset
                            .saturating_add(1)
                            .min(self.max_scoring_scroll)
                    } else {
                        self.scoring_scroll_offset.saturating_sub(1)
                    };
                }
                _ => {}
            },
        }
    }
}

/// Change an index by delta within [0, len), capping at boundaries.
fn change_index(current: usize, delta: i32, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let new = current as i32 + delta;
    if new < 0 {
        current
    } else if new >= len as i32 {
        current
    } else {
        new as usize
    }
}
