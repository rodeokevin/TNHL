use std::fmt::Debug;

use crate::input::{Action, map_key};
use crate::models::games::GamesResponse;
use crate::models::standings::StandingsResponse;
use crate::sources::AppEvent;
use crate::sources::games::GamesCommand;
use crate::sources::standings::StandingsCommand;
use crate::state::date_input::DateInput;
use crate::state::date_selector::DateSelector;
use crate::state::standings_state::StandingsState;
use chrono::{NaiveDate, ParseError};
use chrono_tz::Tz;
use tokio::sync::mpsc::Sender;

/// Which pane currently has keyboard focus.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum PaneFocus {
    #[default]
    Menu,
    Content,
    DatePicker,
}

impl PaneFocus {
    pub fn switch(self) -> Self {
        match self {
            PaneFocus::Menu => PaneFocus::Content,
            PaneFocus::Content => PaneFocus::Menu,
            PaneFocus::DatePicker => PaneFocus::DatePicker,
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
    pub date_selector: DateSelector,
    pub timezone: Tz,

    pub games_tx: Sender<GamesCommand>,
    pub standings_tx: Sender<StandingsCommand>,

    pub selected_menu: MenuFocus,
    pub standings: StandingsState,
    pub league_data: Option<StandingsResponse>,

    pub games_data: Option<GamesResponse>,
    pub selected_game_index: usize,
    pub sweeping_status_offset: usize, // For the --- under the time remaining
    pub scoring_scroll_offset: usize,
    pub max_scoring_scroll: usize,

    pub focus: PaneFocus,
    pub previous_focus: PaneFocus,
    pub should_quit: bool,
}

impl AppState {
    pub fn new(games_tx: Sender<GamesCommand>, standings_tx: Sender<StandingsCommand>) -> Self {
        Self {
            date_input: DateInput::default(),
            date_selector: DateSelector::default(),
            timezone: Tz::default(),

            games_tx,
            standings_tx,

            selected_menu: MenuFocus::default(),
            standings: StandingsState::default(),
            league_data: None,

            games_data: None,
            selected_game_index: 0,
            sweeping_status_offset: 0,
            scoring_scroll_offset: 0,
            max_scoring_scroll: 0,

            focus: PaneFocus::default(),
            previous_focus: PaneFocus::default(),
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
                let action = map_key(key_event, self.focus, self.selected_menu);
                self.handle_action(action);
            }
            AppEvent::Tick => {
                self.sweeping_status_offset = self.sweeping_status_offset.wrapping_add(1);
            }
        }
    }

    /// Handle actions mapped from key events
    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,

            Action::SwitchPaneFocus => {
                self.previous_focus = self.focus;
                self.focus = self.focus.switch();
            }
            Action::EnterDatePicker => {
                self.previous_focus = self.focus;
                self.focus = PaneFocus::DatePicker;
                self.date_input.text.clear();
            }
            Action::InputChar(c) => {
                self.date_input.is_valid = true; // reset status
                self.date_input.text.push(c);
            }
            Action::MenuUp => {
                let prev = self.selected_menu;
                self.selected_menu = self.selected_menu.prev();
                if prev != self.selected_menu {
                    self.reset_games_selection_state();
                    self.reset_standings_selection_state();
                }
            }
            Action::MenuDown => {
                let prev = self.selected_menu;
                self.selected_menu = self.selected_menu.next();
                if prev != self.selected_menu {
                    self.reset_games_selection_state();
                    self.reset_standings_selection_state();
                }
            }
            Action::GamesScrollUp => {
                self.scoring_scroll_offset = self.scoring_scroll_offset.saturating_sub(1);
            }
            Action::GamesScrollDown => {
                self.scoring_scroll_offset = self
                    .scoring_scroll_offset
                    .saturating_add(1)
                    .min(self.max_scoring_scroll);
            }
            Action::PrevGame => self.shift_game_index(false),
            Action::NextGame => self.shift_game_index(true),

            Action::StandingsUp => self.standings.move_selection(-1),
            Action::StandingsDown => self.standings.move_selection(1),
            Action::StandingsLeft => self.standings.shift_standings_type(false),
            Action::StandingsRight => self.standings.shift_standings_type(true),
            Action::PrevStandingsType => self.standings.cycle_focus(false),
            Action::NextStandingsType => self.standings.cycle_focus(true),

            Action::DateLeft => self.move_date_selector_by_arrow(false),
            Action::DateRight => self.move_date_selector_by_arrow(true),
            Action::DateBackspace => {
                self.date_input.text.pop();
            }
            Action::ExitDatePicker => {
                self.date_input.text.clear();
                self.focus = self.previous_focus;
            }
            Action::UpdateDate => {
                if self.try_update_date_from_input().is_ok() {
                    self.handle_date_change();
                    self.focus = self.previous_focus;
                }
            }

            Action::None => {}
        }
    }

    // Helpers functions for handlind actions
    fn move_date_selector_by_arrow(&mut self, right_arrow: bool) {
        let date = self.date_selector.set_date_with_arrows(right_arrow);
        self.date_input.text.clear();
        self.date_input.text.push_str(&date.to_string());
    }
    fn set_date_from_valid_input(&mut self, date: NaiveDate) {
        self.date_selector.set_date_from_valid_input(date);
    }
    fn try_update_date_from_input(&mut self) -> Result<(), ParseError> {
        let valid_date = self.date_input.validate_input(self.timezone)?;

        self.set_date_from_valid_input(valid_date);
        Ok(())
    }
    fn handle_date_change(&mut self) {
        let date = self.date_selector.date.to_string();
        let games_ok = self
            .games_tx
            .try_send(GamesCommand::SetDate(date.clone()))
            .is_ok();
        let standings_ok = self
            .standings_tx
            .try_send(StandingsCommand::SetDate(date))
            .is_ok();
        if games_ok || standings_ok {
            self.reset_games_selection_state();
            self.reset_standings_selection_state();
        }
    }
    fn shift_game_index(&mut self, forward: bool) {
        let prev = self.selected_game_index;
        if forward {
            let max_index = self.games_data.as_ref().map_or(0, |d| d.games.len());
            self.selected_game_index = self.next_index(self.selected_game_index, max_index);
        } else {
            self.selected_game_index = self.prev_index(self.selected_game_index);
        }
        if self.selected_game_index != prev {
            self.reset_scoring_scroll();
        }
    }
    fn prev_index(&self, index: usize) -> usize {
        index.saturating_sub(1)
    }
    fn next_index(&self, index: usize, max_index: usize) -> usize {
        (index + 1).min(max_index.saturating_sub(1))
    }
    fn reset_scoring_scroll(&mut self) {
        self.scoring_scroll_offset = 0;
        self.max_scoring_scroll = 0;
    }
    fn reset_games_selection_state(&mut self) {
        self.selected_game_index = 0;
        self.reset_scoring_scroll();
    }
    fn reset_standings_selection_state(&mut self) {
        self.standings = StandingsState::default();
    }
}
