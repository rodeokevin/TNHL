use crate::input::{Action, map_key};
use crate::sources::AppEvent;
use ratatui::widgets::TableState;

use crate::models::games::GamesResponse;
use crate::models::standings::StandingsResponse;

const LEAGUE_NUM_TEAMS: usize = 32;
const CONFERENCE_NUM_TEAMS: usize = 16;
const DIVISION_NUM_TEAMS: usize = 8;
const WILDCARD_NUM_TEAMS: usize = 16;

/// Which pane currently has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneFocus {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuFocus {
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

// Which standings is selected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandingsFocus {
    WildCard,
    Division,
    Conference,
    League,
}

impl StandingsFocus {
    pub fn next(self) -> Self {
        match self {
            StandingsFocus::WildCard => StandingsFocus::Division,
            StandingsFocus::Division => StandingsFocus::Conference,
            StandingsFocus::Conference => StandingsFocus::League,
            StandingsFocus::League => StandingsFocus::League,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            StandingsFocus::WildCard => StandingsFocus::WildCard,
            StandingsFocus::Division => StandingsFocus::WildCard,
            StandingsFocus::Conference => StandingsFocus::Division,
            StandingsFocus::League => StandingsFocus::Conference,
        }
    }
}

// Which conference is selected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConferenceFocus {
    Eastern,
    Western,
}

impl ConferenceFocus {
    pub fn next(self) -> Self {
        match self {
            ConferenceFocus::Eastern => ConferenceFocus::Western,
            ConferenceFocus::Western => ConferenceFocus::Eastern,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ConferenceFocus::Eastern => ConferenceFocus::Western,
            ConferenceFocus::Western => ConferenceFocus::Eastern,
        }
    }
}

// Which division is selected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DivisionFocus {
    Atlantic,
    Metropolitan,
    Central,
    Pacific,
}

impl DivisionFocus {
    pub fn next(self) -> Self {
        match self {
            DivisionFocus::Atlantic => DivisionFocus::Metropolitan,
            DivisionFocus::Metropolitan => DivisionFocus::Central,
            DivisionFocus::Central => DivisionFocus::Pacific,
            DivisionFocus::Pacific => DivisionFocus::Atlantic,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            DivisionFocus::Atlantic => DivisionFocus::Pacific,
            DivisionFocus::Metropolitan => DivisionFocus::Atlantic,
            DivisionFocus::Central => DivisionFocus::Metropolitan,
            DivisionFocus::Pacific => DivisionFocus::Central,
        }
    }
}

pub struct App {
    /// Selected menu item
    pub selected_menu: MenuFocus,
    /// Selected standings
    pub standings_type: StandingsFocus,
    /// Current data for league standings
    pub league_data: Option<StandingsResponse>,
    /// States for standings tables
    /// Conference tables
    pub eastern_table_state: TableState,
    pub western_table_state: TableState,
    pub selected_conference: ConferenceFocus,
    /// Division tables
    pub atlantic_table_state: TableState,
    pub metropolitan_table_state: TableState,
    pub central_table_state: TableState,
    pub pacific_table_state: TableState,
    pub selected_division: DivisionFocus,
    /// Wildcard tables
    pub eastern_wildcard_table_state: TableState,
    pub western_wildcard_table_state: TableState,
    pub selected_wildcard: ConferenceFocus,
    /// League table
    pub league_table_state: TableState,

    /// Current data for games
    pub games_data: Option<GamesResponse>,
    pub selected_game_index: usize,

    /// Currently focused pane.
    pub focus: PaneFocus,

    /// Whether we should quit the application
    pub should_quit: bool,
}

impl App {
    pub fn new() -> App {
        let mut table_state = TableState::default();
        table_state.select(Some(0));
        // Default app states
        App {
            selected_menu: MenuFocus::Games,
            standings_type: StandingsFocus::WildCard,
            league_data: None,
            eastern_table_state: table_state.clone(),
            western_table_state: table_state.clone(),
            selected_conference: ConferenceFocus::Eastern,
            league_table_state: table_state.clone(),
            atlantic_table_state: table_state.clone(),
            metropolitan_table_state: table_state.clone(),
            central_table_state: table_state.clone(),
            pacific_table_state: table_state.clone(),
            selected_division: DivisionFocus::Atlantic,
            eastern_wildcard_table_state: table_state.clone(),
            western_wildcard_table_state: table_state.clone(),
            selected_wildcard: ConferenceFocus::Eastern,

            games_data: None,
            selected_game_index: 0,

            should_quit: false,
            focus: PaneFocus::Menu,
        }
    }

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
                    Ok(parsed_games) => self.games_data = Some(parsed_games),
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
                // No-op for now; UI refresh is driven by render calls
            }
        }
    }

    /// Handle a mapped action.
    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::MoveUp => self.move_selection(-1),
            Action::MoveDown => self.move_selection(1),
            // Left right for content pane only
            Action::FocusLeft if self.focus == PaneFocus::Content => match self.selected_menu {
                MenuFocus::Games => self.move_selection(-1),
                MenuFocus::Standings => self.standings_type = self.standings_type.prev(),
                MenuFocus::Teams => {}
            },
            Action::FocusRight if self.focus == PaneFocus::Content => match self.selected_menu {
                MenuFocus::Games => self.move_selection(1),
                MenuFocus::Standings => self.standings_type = self.standings_type.next(),
                MenuFocus::Teams => {}
            },
            // Use tab to switch between menu and content
            Action::CycleFocus => self.focus = self.focus.switch(),
            // Next standings
            Action::NextStandings if self.focus == PaneFocus::Content => {
                match self.standings_type {
                    StandingsFocus::Conference => {
                        self.selected_conference = self.selected_conference.next()
                    }
                    StandingsFocus::Division => {
                        self.selected_division = self.selected_division.next()
                    }
                    StandingsFocus::WildCard => {
                        self.selected_wildcard = self.selected_wildcard.next()
                    }
                    StandingsFocus::League => {}
                }
            }
            // Previous standings
            Action::PrevStandings if self.focus == PaneFocus::Content => {
                match self.standings_type {
                    StandingsFocus::Conference => {
                        self.selected_conference = self.selected_conference.prev()
                    }
                    StandingsFocus::Division => {
                        self.selected_division = self.selected_division.prev()
                    }
                    StandingsFocus::WildCard => {
                        self.selected_wildcard = self.selected_wildcard.prev()
                    }
                    StandingsFocus::League => {}
                }
            }
            Action::Refresh => {
                // Sources auto-refresh; this is a UI hint
            }
            Action::Quit => {
                self.should_quit = true;
            }
            _ => {}
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
                MenuFocus::Games => {
                    let current = self.selected_game_index;
                    let len = self.games_data.as_ref().map_or(0, |data| data.games.len());
                    let new_index = change_index(current, delta, len);
                    self.selected_game_index = new_index;
                }
                MenuFocus::Standings => match self.standings_type {
                    StandingsFocus::League => {
                        let current = self.league_table_state.selected().unwrap_or(0);
                        let new_index = change_index(current, delta, LEAGUE_NUM_TEAMS);
                        self.league_table_state.select(Some(new_index));
                    }
                    StandingsFocus::Conference => match self.selected_conference {
                        ConferenceFocus::Eastern => {
                            let current = self.eastern_table_state.selected().unwrap_or(0);
                            let new_index = change_index(current, delta, CONFERENCE_NUM_TEAMS);
                            self.eastern_table_state.select(Some(new_index));
                        }
                        ConferenceFocus::Western => {
                            let current = self.western_table_state.selected().unwrap_or(0);
                            let new_index = change_index(current, delta, CONFERENCE_NUM_TEAMS);
                            self.western_table_state.select(Some(new_index));
                        }
                    },
                    StandingsFocus::Division => match self.selected_division {
                        DivisionFocus::Atlantic => {
                            let current = self.atlantic_table_state.selected().unwrap_or(0);
                            let new_index = change_index(current, delta, DIVISION_NUM_TEAMS);
                            self.atlantic_table_state.select(Some(new_index));
                        }
                        DivisionFocus::Metropolitan => {
                            let current = self.metropolitan_table_state.selected().unwrap_or(0);
                            let new_index = change_index(current, delta, DIVISION_NUM_TEAMS);
                            self.metropolitan_table_state.select(Some(new_index));
                        }
                        DivisionFocus::Central => {
                            let current = self.central_table_state.selected().unwrap_or(0);
                            let new_index = change_index(current, delta, DIVISION_NUM_TEAMS);
                            self.central_table_state.select(Some(new_index));
                        }
                        DivisionFocus::Pacific => {
                            let current = self.pacific_table_state.selected().unwrap_or(0);
                            let new_index = change_index(current, delta, DIVISION_NUM_TEAMS);
                            self.pacific_table_state.select(Some(new_index));
                        }
                    },
                    StandingsFocus::WildCard => match self.selected_wildcard {
                        ConferenceFocus::Eastern => {
                            let current = self.eastern_wildcard_table_state.selected().unwrap_or(0);
                            let new_index = change_index(current, delta, WILDCARD_NUM_TEAMS);
                            self.eastern_wildcard_table_state.select(Some(new_index));
                        }
                        ConferenceFocus::Western => {
                            let current = self.western_wildcard_table_state.selected().unwrap_or(0);
                            let new_index = change_index(current, delta, WILDCARD_NUM_TEAMS);
                            self.western_wildcard_table_state.select(Some(new_index));
                        }
                    },
                },
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
