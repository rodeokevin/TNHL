use std::fmt::Debug;

use crate::input::{Action, map_key};
use crate::sources::{
    AppEvent, boxscore::BoxscoreCommand, game_story::GameStoryCommand, games::GamesCommand,
    standings::StandingsCommand,
};
use crate::state::{
    date_state::DateState,
    games_state::GamesState,
    games_state::{BoxscorePosition, BoxscoreTeam},
    help::HelpState,
    standings_state::StandingsState,
};
use chrono::ParseError;
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
    pub date_state: DateState,
    pub timezone: Tz,

    pub games_tx: Sender<GamesCommand>,
    pub standings_tx: Sender<StandingsCommand>,
    pub boxscore_tx: Sender<BoxscoreCommand>,
    pub game_story_tx: Sender<GameStoryCommand>,

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
        game_story_tx: Sender<GameStoryCommand>,
    ) -> Self {
        Self {
            games_tx,
            standings_tx,
            boxscore_tx,
            game_story_tx,

            date_state: DateState::default(),
            timezone: Tz::default(),

            selected_menu: MenuFocus::default(),
            display_menu: true,
            standings: StandingsState::default(),
            games: GamesState::default(),

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
                log::info!("Sending game ids to other sources");
                self.boxscore_tx
                    .try_send(BoxscoreCommand::SetGameIds(game_ids.clone()))
                    .ok();

                self.game_story_tx
                    .try_send(GameStoryCommand::SetGameIds(game_ids.clone()))
                    .ok();
            }
            AppEvent::BoxscoreUpdate {
                game_id,
                parsed_boxscore,
            } => {
                log::info!("Updating boxscore data");
                self.games.boxscore_data.insert(game_id, parsed_boxscore);
            }
            AppEvent::GameStoryUpdate {
                game_id,
                parsed_game_story,
            } => {
                log::info!("Updating game story data");
                self.games
                    .game_story_data
                    .insert(game_id, parsed_game_story);
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
                self.date_state.is_valid = true; // reset status
                self.date_state.text.push(c);
            }
            Action::MenuUp => {
                let prev = self.selected_menu;
                self.selected_menu = self.selected_menu.prev();
                if prev != self.selected_menu {
                    self.reset_app_state();
                }
            }
            Action::MenuDown => {
                let prev = self.selected_menu;
                self.selected_menu = self.selected_menu.next();
                if prev != self.selected_menu {
                    self.reset_app_state();
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
                let prev = self.games.selected_game_index;
                self.games.shift_game_index(true);
                if self.games.selected_game_index != prev {
                    self.games.reset_state();
                }
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
            Action::GamesScrollUp => {
                self.games.scroll_offset = self.games.scroll_offset.saturating_sub(1);
            }
            Action::GamesScrollDown => {
                self.games.scroll_offset = self
                    .games
                    .scroll_offset
                    .saturating_add(1)
                    .min(self.games.max_scroll);
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
            Action::StandingsLeft => {
                self.standings.shift_standings_type(false);
                self.standings.reset_table_state();
            }
            Action::StandingsRight => {
                self.standings.shift_standings_type(true);
                self.standings.reset_table_state();
            }
            Action::PrevStandingsDisplay => {
                self.standings.cycle_display(false);
                self.standings.reset_table_state();
            }
            Action::NextStandingsDisplay => {
                self.standings.cycle_display(true);
                self.standings.reset_table_state();
            }

            Action::EnterDatePicker => {
                self.previous_focus = self.focus;
                self.focus = PaneFocus::DatePicker;
                self.date_state.text.clear();
            }
            Action::DateLeft => self.date_state.move_date_selector_by_arrow(false),
            Action::DateRight => self.date_state.move_date_selector_by_arrow(true),
            Action::DateBackspace => {
                self.date_state.text.pop();
            }
            Action::ExitDatePicker => {
                self.date_state.text.clear();
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

    // Helper functions for handling actions
    fn try_update_date_from_input(&mut self) -> Result<(), ParseError> {
        let valid_date = self.date_state.validate_input(self.timezone)?;

        self.date_state.set_date_from_valid_input(valid_date);
        Ok(())
    }
    /// Update data from sources after date change
    pub async fn handle_date_change(&mut self) {
        let date = self.date_state.date.to_string();

        if let Err(e) = self
            .games_tx
            .send(GamesCommand::SetDate(date.clone()))
            .await
        {
            log::error!("Games channel closed: {:?}", e);
            return;
        }

        if let Err(e) = self
            .standings_tx
            .send(StandingsCommand::SetDate(date.clone()))
            .await
        {
            log::error!("Standings channel closed: {:?}", e);
            return;
        }

        self.games.boxscore_data.clear();
        self.games.game_story_data.clear();
        self.reset_app_state();
    }

    fn reset_app_state(&mut self) {
        self.games.reset_state();
        self.standings.reset_state();
    }
}
