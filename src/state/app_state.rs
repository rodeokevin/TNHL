use std::fmt::Debug;

use crate::input::{Action, map_key};
use crate::models::{
    boxscore::BoxscoreResponse, games::GamesResponse, standings::StandingsResponse,
};
use crate::sources::{
    AppEvent, boxscore::BoxscoreCommand, games::GamesCommand, standings::StandingsCommand,
};
use crate::state::games_state::{BoxscorePosition, BoxscoreTeam};
use crate::state::{
    date_input::DateInput, date_selector::DateSelector, games_state::GamesState, help::HelpState,
    standings_state::StandingsState,
};
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
    Help,
}

impl PaneFocus {
    pub fn switch(self) -> Self {
        match self {
            PaneFocus::Menu => PaneFocus::Content,
            PaneFocus::Content => PaneFocus::Menu,
            _ => self,
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
    pub boxscore_tx: Sender<BoxscoreCommand>,

    pub selected_menu: MenuFocus,
    pub display_menu: bool,
    pub standings: StandingsState,
    pub games: GamesState,

    pub help: HelpState,

    pub focus: PaneFocus,
    pub previous_focus: PaneFocus,
    pub should_quit: bool,
}

impl AppState {
    pub fn new(
        games_tx: Sender<GamesCommand>,
        standings_tx: Sender<StandingsCommand>,
        boxscore_tx: Sender<BoxscoreCommand>,
    ) -> Self {
        Self {
            date_input: DateInput::default(),
            date_selector: DateSelector::default(),
            timezone: Tz::default(),

            games_tx,
            standings_tx,
            boxscore_tx,

            selected_menu: MenuFocus::default(),
            display_menu: true,
            standings: StandingsState::default(), // all state related to standings
            games: GamesState::default(),         // all state related to games

            help: HelpState::default(),

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
            AppEvent::StandingsUpdate(parsed_standings) => {
                log::info!("Updating standings data");
                self.standings.standings_data = Some(parsed_standings);
            }
            AppEvent::GamesUpdate {
                game_ids,
                parsed_games,
            } => {
                log::info!("Updating games data");
                self.games.games_data = Some(parsed_games);
                log::info!("Sending game ids to boxscore");
                let _ = self
                    .boxscore_tx
                    .try_send(BoxscoreCommand::SetGameIds(game_ids));
            }
            AppEvent::BoxscoreUpdate { game_id, parsed_boxscore } => {
                log::info!("Updating boxscore data");
                self.games.boxscore_data.insert(game_id, parsed_boxscore);
            }
            AppEvent::Input(key_event) => {
                log::info!("Key event detected: {:?}", key_event);
                let action = map_key(key_event, self);
                self.handle_action(action);
            }
            AppEvent::Tick => {
                self.games.sweeping_status_offset =
                    self.games.sweeping_status_offset.wrapping_add(1);
            }
        }
    }

    /// Handle actions mapped from key events
    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,

            Action::SwitchPaneFocus => {
                // Only switch if menu is visible
                if self.display_menu {
                    self.focus = self.focus.switch();
                }
            }
            Action::ToggleDisplayMenu => {
                if self.display_menu {
                    self.focus = PaneFocus::Content;
                } else {
                    self.focus = PaneFocus::Menu;
                }
                self.display_menu = !self.display_menu;
            }
            Action::InputChar(c) => {
                self.date_input.is_valid = true; // reset status
                self.date_input.text.push(c);
            }
            Action::MenuUp => {
                let prev = self.selected_menu;
                self.selected_menu = self.selected_menu.prev();
                if prev != self.selected_menu {
                    self.reset_selections();
                }
            }
            Action::MenuDown => {
                let prev = self.selected_menu;
                self.selected_menu = self.selected_menu.next();
                if prev != self.selected_menu {
                    self.reset_selections();
                }
            }
            Action::PrevGame => {
                self.games.boxscore_selected_position = BoxscorePosition::default();
                self.games.boxscore_selected_team = BoxscoreTeam::default();
                self.games.shift_game_index(false);
            }
            Action::NextGame => {
                self.games.boxscore_selected_position = BoxscorePosition::default();
                self.games.boxscore_selected_team = BoxscoreTeam::default();
                self.games.shift_game_index(true);
            }
            Action::PrevGamesDisplay => {
                self.games.cycle_display(false);
                self.games.reset_scoring_scroll();
                self.games.boxscore_table_state.select(Some(0));
            }
            Action::NextGamesDisplay => {
                self.games.cycle_display(true);
                self.games.reset_scoring_scroll();
                self.games.boxscore_table_state.select(Some(0));
            }
            Action::OverviewScrollUp => {
                self.games.scoring_scroll_offset =
                    self.games.scoring_scroll_offset.saturating_sub(1);
            }
            Action::OverviewScrollDown => {
                self.games.scoring_scroll_offset = self
                    .games
                    .scoring_scroll_offset
                    .saturating_add(1)
                    .min(self.games.max_scoring_scroll);
            }
            Action::BoxscoreUp => self.games.move_boxscore_selection(-1),
            Action::BoxscoreDown => self.games.move_boxscore_selection(1),
            Action::BoxscoreForwards => {
                self.games.boxscore_table_state.select(Some(0));
                self.games.boxscore_selected_position = BoxscorePosition::Forwards
            }
            Action::BoxscoreDefensemen => {
                self.games.boxscore_table_state.select(Some(0));
                self.games.boxscore_selected_position = BoxscorePosition::Defensemen
            }
            Action::BoxscoreGoalies => {
                self.games.boxscore_table_state.select(Some(0));
                self.games.boxscore_selected_position = BoxscorePosition::Goalies
            }
            Action::BoxscoreToggleTeam => {
                self.games.boxscore_table_state.select(Some(0));
                self.games.boxscore_selected_position = BoxscorePosition::default();
                self.games.boxscore_selected_team = self.games.boxscore_selected_team.toggle()
            }

            Action::StandingsUp => self.standings.move_selection(-1),
            Action::StandingsDown => self.standings.move_selection(1),
            Action::StandingsLeft => self.standings.shift_standings_type(false),
            Action::StandingsRight => self.standings.shift_standings_type(true),
            Action::PrevStandingsDisplay => self.standings.cycle_display(false),
            Action::NextStandingsDisplay => self.standings.cycle_display(true),

            Action::EnterDatePicker => {
                self.previous_focus = self.focus;
                self.focus = PaneFocus::DatePicker;
                self.date_input.text.clear();
            }
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

            Action::EnterHelp => {
                self.previous_focus = self.focus;
                self.focus = PaneFocus::Help;
            }
            Action::HelpScrollUp => self.help.previous(),
            Action::HelpScrollDown => self.help.next(),
            Action::HelpPageUp => self.help.page_up(),
            Action::HelpPageDown => self.help.page_down(),
            Action::ExitHelp => self.focus = self.previous_focus,

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
            self.reset_selections();
        }
        self.games.boxscore_data.clear();
    }

    fn reset_selections(&mut self) {
        self.games.reset_selection_state();
        self.standings.reset_selection_state();
    }
}
