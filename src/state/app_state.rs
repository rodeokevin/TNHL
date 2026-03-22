use crate::input::{Action, map_key};
use crate::sources::AppEvent;
use crate::state::standings_state::{StandingsState, StandingsFocus, ConferenceFocus, DivisionFocus};
use ratatui::widgets::{TableState};
// use crate::state::date_input::DateInput;
use crate::models::games::GamesResponse;
use crate::models::standings::StandingsResponse;

const LEAGUE_NUM_TEAMS: usize = 32;
const CONFERENCE_NUM_TEAMS: usize = 16;
const DIVISION_NUM_TEAMS: usize = 8;
const WILDCARD_NUM_TEAMS: usize = 16;

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

pub trait Cycle {
    fn next(self) -> Self;
    fn prev(self) -> Self;
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
    pub selected_menu: MenuFocus,
    pub standings: StandingsState,
    pub league_data: Option<StandingsResponse>,

    pub games_data: Option<GamesResponse>,
    pub selected_game_index: usize,
    pub sweeping_status_offset: usize, // For the --- under the time remaining

    pub focus: PaneFocus,
    pub should_quit: bool,
}

impl Default for AppState {
    fn default() -> Self {
        fn table() -> TableState {
            let mut t = TableState::default();
            t.select(Some(0));
            t
        }

        Self {
            selected_menu: MenuFocus::default(),
            standings: StandingsState::default(),
            league_data: None,

            games_data: None,
            selected_game_index: 0,
            sweeping_status_offset: 0,

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
                        self.selected_game_index =
                            change_index(self.selected_game_index, delta, len);
                    }
                    MenuFocus::Standings => {
                        self.standings.focus = if delta == 1 {
                            self.standings.focus.next()
                        } else {
                            self.standings.focus.prev()
                        };
                    }
                    MenuFocus::Teams => {}
                }
            }
            Action::CycleFocus => self.focus = self.focus.switch(),
            Action::NextStandings | Action::PrevStandings if self.focus == PaneFocus::Content => {
                let next = matches!(action, Action::NextStandings);
                match self.standings.focus {
                    StandingsFocus::Conference => {
                        self.standings.selected_conference = self.standings.selected_conference.toggle();
                    }
                    StandingsFocus::Division => {
                        self.standings.selected_division = if next {
                            self.standings.selected_division.next()
                        } else {
                            self.standings.selected_division.prev()
                        };
                    }
                    StandingsFocus::WildCard => {
                        self.standings.selected_wildcard = self.standings.selected_wildcard.toggle();
                    }
                    StandingsFocus::League => {}
                }
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
                MenuFocus::Standings => {
                    if let Some((table, len)) = self.current_table_state_mut() {
                        move_standings_selection(table, delta, len);
                    }
                }
                _ => {}
            },
        }
    }

    fn current_table_state_mut(&mut self) -> Option<(&mut TableState, usize)> {
        match self.standings.focus {
            StandingsFocus::League => Some((&mut self.standings.league_table_state, LEAGUE_NUM_TEAMS)),
            StandingsFocus::Conference => match self.standings.selected_conference {
                ConferenceFocus::Eastern => {
                    Some((&mut self.standings.eastern_table_state, CONFERENCE_NUM_TEAMS))
                }
                ConferenceFocus::Western => {
                    Some((&mut self.standings.western_table_state, CONFERENCE_NUM_TEAMS))
                }
            },
            StandingsFocus::Division => match self.standings.selected_division {
                DivisionFocus::Atlantic => {
                    Some((&mut self.standings.atlantic_table_state, DIVISION_NUM_TEAMS))
                }
                DivisionFocus::Metropolitan => {
                    Some((&mut self.standings.metropolitan_table_state, DIVISION_NUM_TEAMS))
                }
                DivisionFocus::Central => Some((&mut self.standings.central_table_state, DIVISION_NUM_TEAMS)),
                DivisionFocus::Pacific => Some((&mut self.standings.pacific_table_state, DIVISION_NUM_TEAMS)),
            },
            StandingsFocus::WildCard => match self.standings.selected_wildcard {
                ConferenceFocus::Eastern => {
                    Some((&mut self.standings.eastern_wildcard_table_state, WILDCARD_NUM_TEAMS))
                }
                ConferenceFocus::Western => {
                    Some((&mut self.standings.western_wildcard_table_state, WILDCARD_NUM_TEAMS))
                }
            },
        }
    }
}

fn move_standings_selection(table_state: &mut TableState, delta: i32, num_teams: usize) {
    let current = table_state.selected().unwrap_or(0);
    let new_index = change_index(current, delta, num_teams);
    table_state.select(Some(new_index));
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
