use crate::input::{Action, map_key};
use crate::sources::AppEvent;
use ratatui::widgets::TableState;

const MENU_LEN: usize = 3;
const LEAGUE_NUM_TEAMS: usize = 32;
const CONFERENCE_NUM_TEAMS: usize = 32;
const DIVISION_NUM_TEAMS: usize = 32;
const WILDCARD_NUM_TEAMS: usize = 32;

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
pub enum ConferenceType {
    Eastern,
    Western,
}

impl ConferenceType {
    pub fn next(self) -> Self {
        match self {
            ConferenceType::Eastern => ConferenceType::Western,
            ConferenceType::Western => ConferenceType::Eastern,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ConferenceType::Eastern => ConferenceType::Western,
            ConferenceType::Western => ConferenceType::Eastern,
        }
    }
}

pub struct App {
    /// Selected index in the menu
    pub menu_index: usize,
    /// Selected standings
    pub standings_type: StandingsFocus,

    /// Current data for league standings
    pub league_standings: Option<String>,
    /// States for standings tables
    pub wildcard_table_state: TableState,
    pub division_table_state: TableState,
    /// Conference tables
    pub eastern_table_state: TableState,
    pub western_table_state: TableState,
    pub selected_conference: ConferenceType,
    pub league_table_state: TableState,

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
            menu_index: 0,
            standings_type: StandingsFocus::WildCard,
            league_standings: None,
            wildcard_table_state: table_state.clone(),
            division_table_state: table_state.clone(),
            eastern_table_state: table_state.clone(),
            western_table_state: table_state.clone(),
            selected_conference: ConferenceType::Eastern,
            league_table_state: table_state.clone(),
            should_quit: false,
            focus: PaneFocus::Menu,
        }
    }

    // Handle an incoming event and update state accordingly
    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::StandingsUpdate(standings) => {
                log::info!("Updating standings data");
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
            // Left right for content pane only
            Action::FocusLeft if self.focus == PaneFocus::Content => self.standings_type = self.standings_type.prev(),
            Action::FocusRight if self.focus == PaneFocus::Content => self.standings_type = self.standings_type.next(),
            // Use tab to switch between menu and content
            Action::CycleFocus => self.focus = self.focus.switch(),
            // Next standings 
            Action::NextStandings if self.focus == PaneFocus::Content => {
                match self.standings_type {
                    StandingsFocus::Conference => self.selected_conference = self.selected_conference.next(), 
                    _ => {}
                }
            }
            // Previous standings
            Action::PrevStandings if self.focus == PaneFocus::Content => {
                match self.standings_type {
                    StandingsFocus::Conference => self.selected_conference = self.selected_conference.prev(), 
                    _ => {}
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
                self.menu_index = change_index(self.menu_index, delta, MENU_LEN);
            }
            PaneFocus::Content => {
                match self.standings_type {
                    StandingsFocus::League => {
                        let current = self.league_table_state.selected().unwrap_or(0);
                        let new_index = change_index(current, delta, LEAGUE_NUM_TEAMS);
                        self.league_table_state.select(Some(new_index));
                    }
                    StandingsFocus::Conference => {
                        if self.selected_conference == ConferenceType::Eastern {
                            let current = self.eastern_table_state.selected().unwrap_or(0);
                            let new_index = change_index(current, delta, CONFERENCE_NUM_TEAMS);
                            self.eastern_table_state.select(Some(new_index));
                        }
                        else {
                            let current = self.western_table_state.selected().unwrap_or(0);
                            let new_index = change_index(current, delta, CONFERENCE_NUM_TEAMS);
                            self.western_table_state.select(Some(new_index));
                        }
                    }
                    StandingsFocus::Division => {
                        let current = self.division_table_state.selected().unwrap_or(0);
                        let new_index = change_index(current, delta, DIVISION_NUM_TEAMS);
                        self.division_table_state.select(Some(new_index));
                    }
                    StandingsFocus::WildCard => {
                        let current = self.wildcard_table_state.selected().unwrap_or(0);
                        let new_index = change_index(current, delta, WILDCARD_NUM_TEAMS);
                        self.wildcard_table_state.select(Some(new_index));
                    }
                }
                
            }
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
