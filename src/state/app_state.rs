use std::fmt::Debug;

use crate::input::{Action, map_key};
use crate::models::games::games::{GameState, GamesResponse};
use crate::sources::{
    AppEvent, FetchInterval,
    games::{boxscore::BoxscoreCommand, game_story::GameStoryCommand, games::GamesCommand},
    playoffs::bracket::BracketCommand,
    standings::StandingsCommand,
    teams_stats::TeamStatsCommand,
};
use crate::state::team_stats::team_picker::InputError;
use crate::state::{
    date_state::DateState, games_state::BoxscorePosition, games_state::GamesState, help::HelpState,
    playoff_bracket::BracketState, standings_state::StandingsState,
    team_stats::team_stats_state::TeamStatsState,
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
    /// Widget for selecting the team and year for team stats page
    TeamPicker,
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
    TeamStats,
    Playoffs,
}

impl MenuFocus {
    pub fn next(self) -> Self {
        match self {
            MenuFocus::Games => MenuFocus::Standings,
            MenuFocus::Standings => MenuFocus::TeamStats,
            MenuFocus::TeamStats => MenuFocus::Playoffs,
            MenuFocus::Playoffs => MenuFocus::Playoffs,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            MenuFocus::Playoffs => MenuFocus::TeamStats,
            MenuFocus::TeamStats => MenuFocus::Standings,
            MenuFocus::Standings => MenuFocus::Games,
            MenuFocus::Games => MenuFocus::Games,
        }
    }
    pub fn index(&self) -> usize {
        match self {
            MenuFocus::Games => 0,
            MenuFocus::Standings => 1,
            MenuFocus::TeamStats => 2,
            MenuFocus::Playoffs => 3,
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
    pub team_stats_tx: Sender<TeamStatsCommand>,
    pub playoff_bracket_tx: Sender<BracketCommand>,

    pub selected_menu: MenuFocus,
    pub display_menu: bool,

    pub standings: StandingsState,
    pub games: GamesState,
    pub team_stats: TeamStatsState,
    pub playoff_bracket: BracketState,

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
        team_stats_tx: Sender<TeamStatsCommand>,
        playoff_bracket_tx: Sender<BracketCommand>,
    ) -> Self {
        Self {
            games_tx,
            standings_tx,
            boxscore_tx,
            game_story_tx,
            team_stats_tx,
            playoff_bracket_tx,

            date_state: DateState::default(),
            timezone: Tz::default(),

            selected_menu: MenuFocus::default(),
            display_menu: true,

            standings: StandingsState::default(),
            games: GamesState::default(),
            team_stats: TeamStatsState::default(),
            playoff_bracket: BracketState::default(),

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
                log::info!("Setting fetch interval for sources");
                self.set_fetch_interval(self.is_games_live(&parsed_games));
                log::info!("Updating games data");
                self.games.games_data = Some(parsed_games);
                log::info!("Sending game ids to other sources");
                self.boxscore_tx
                    .try_send(BoxscoreCommand::SetGameIds(game_ids.clone()))
                    .ok();
                self.game_story_tx
                    .try_send(GameStoryCommand::SetGameIds(game_ids))
                    .ok();
            }
            AppEvent::BoxscoreUpdate {
                game_id,
                parsed_boxscore,
            } => {
                log::info!("Updating boxscore data for game {}", game_id);
                self.games.boxscore_data.insert(game_id, parsed_boxscore);
            }
            AppEvent::GameStoryUpdate {
                game_id,
                parsed_game_story,
            } => {
                log::info!("Updating game story data for game {}", game_id);
                self.games
                    .game_story_data
                    .insert(game_id, parsed_game_story);
            }
            AppEvent::TeamStatsUpdate(parsed_team_stats) => {
                log::info!("Updating standings data");
                self.team_stats.team_stats_data = Some(parsed_team_stats);
            }
            AppEvent::BracketUpdate(parsed_playoff_bracket) => {
                log::info!("Updating playoff bracket data");
                self.playoff_bracket.playoff_bracket_data = Some(parsed_playoff_bracket);
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
            Action::DatePickerInputChar(c) => {
                self.date_state.is_valid = true; // reset status
                self.date_state.text.push(c);
            }
            Action::TeamPickerInputChar(c) => {
                self.team_stats.team_picker.is_valid = true; // reset status
                self.team_stats.team_picker.text.push(c);
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
                let prev = self.games.selected_game_index;
                self.games.shift_game_index(false);
                if self.games.selected_game_index != prev {
                    self.games.reset_game_state();
                }
            }
            Action::NextGame => {
                let prev = self.games.selected_game_index;
                self.games.shift_game_index(true);
                if self.games.selected_game_index != prev {
                    self.games.reset_game_state();
                }
            }
            Action::PrevGamesDisplay => {
                self.games.cycle_display(false);
                self.games.reset_scoring_scroll();
                self.games.reset_boxscore_state();
            }
            Action::NextGamesDisplay => {
                self.games.cycle_display(true);
                self.games.reset_scoring_scroll();
                self.games.reset_boxscore_state();
            }
            Action::GamesPageUp => self.games.games_page_up(),
            Action::GamesPageDown => self.games.games_page_down(),
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
            Action::BoxscorePageUp => self.games.boxscore_page_up(),
            Action::BoxscorePageDown => self.games.boxscore_page_down(),
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
            // Standings actions
            Action::StandingsUp => self.standings.move_selection(-1),
            Action::StandingsDown => self.standings.move_selection(1),
            Action::StandingsPageUp => self.standings.page_up(),
            Action::StandingsPageDown => self.standings.page_down(),
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
            // Team stats page actions
            Action::TeamStatsUp => self.team_stats.move_selection(-1),
            Action::TeamStatsDown => self.team_stats.move_selection(1),
            Action::TeamStatsPageUp => self.team_stats.page_up(),
            Action::TeamStatsPageDown => self.team_stats.page_down(),
            Action::ToggleTeamStats => self.team_stats.show_skaters = !self.team_stats.show_skaters,

            // Playoffs page actions
            Action::BracketScrollUp => {
                self.playoff_bracket.vertical_scroll_offset = self
                    .playoff_bracket
                    .vertical_scroll_offset
                    .saturating_sub(1);
            }
            Action::BracketScrollDown => {
                self.playoff_bracket.vertical_scroll_offset = self
                    .playoff_bracket
                    .vertical_scroll_offset
                    .saturating_add(1)
                    .min(self.playoff_bracket.vertical_max_scroll);
            }
            Action::BracketScrollLeft => {
                self.playoff_bracket.horizontal_scroll_offset = self
                    .playoff_bracket
                    .horizontal_scroll_offset
                    .saturating_sub(1);
            }
            Action::BracketScrollRight => {
                self.playoff_bracket.horizontal_scroll_offset = self
                    .playoff_bracket
                    .horizontal_scroll_offset
                    .saturating_add(1)
                    .min(self.playoff_bracket.horizontal_max_scroll);
            }
            Action::BracketPageUp => self.playoff_bracket.bracket_page_up(),
            Action::BracketPageDown => self.playoff_bracket.bracket_page_down(),
            Action::BracketPageLeft => self.playoff_bracket.bracket_page_left(),
            Action::BracketPageRight => self.playoff_bracket.bracket_page_right(),

            // Date/year picker actions
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
                self.date_state.date_selection_offset = 0;
                self.date_state.year_selection_offset = 0;
                self.date_state.is_valid = true;
                self.focus = self.previous_focus;
            }
            Action::UpdateDate => {
                if self.try_update_date_from_input().is_ok() {
                    self.handle_date_change();
                    self.focus = self.previous_focus;
                }
            }
            Action::YearLeft => self.date_state.move_year_selector_by_arrow(false),
            Action::YearRight => self.date_state.move_year_selector_by_arrow(true),
            Action::UpdateYear => {
                if self.try_update_year_from_input().is_ok() {
                    self.handle_year_change();
                    self.focus = self.previous_focus;
                }
            }
            // Team picker actions
            Action::EnterTeamPicker => {
                self.previous_focus = self.focus;
                self.focus = PaneFocus::TeamPicker;
                self.team_stats.team_picker.text.clear();
            }
            Action::TeamBackspace => {
                self.team_stats.team_picker.text.pop();
            }
            Action::ExitTeamPicker => {
                self.team_stats.team_picker.text.clear();
                self.team_stats.team_picker.is_valid = true;
                self.focus = self.previous_focus;
            }
            Action::UpdateTeam => {
                if self.try_update_team_from_input().is_ok() {
                    self.handle_team_change();
                    self.focus = self.previous_focus;
                }
            }
            // Help page actions
            Action::EnterHelp => {
                self.previous_focus = self.focus;
                self.focus = PaneFocus::Help;
            }
            Action::HelpScrollUp => self.help.previous(),
            Action::HelpScrollDown => self.help.next(),
            Action::ExitHelp => {
                self.focus = self.previous_focus;
                self.help.reset();
            }

            Action::None => {}
        }
    }

    // Helper functions for handling actions
    fn try_update_date_from_input(&mut self) -> Result<(), ParseError> {
        let valid_date = self.date_state.validate_input_date(self.timezone)?;
        self.date_state.set_date_from_valid_input(valid_date);
        Ok(())
    }
    /// Update data from sources after date change
    pub fn handle_date_change(&mut self) {
        let date = self.date_state.date.to_string();
        let games_res = self.games_tx.try_send(GamesCommand::SetDate(date.clone()));
        let standings_res = self
            .standings_tx
            .try_send(StandingsCommand::SetDate(date.clone()));

        if let Err(e) = &games_res {
            log::error!("Failed to send GamesCommand::SetDate: {:?}", e);
        } else {
            // Clear current data and reset all state in games since new data is incoming
            self.games.boxscore_data.clear();
            self.games.game_story_data.clear();
            self.games.reset_state();
        }
        if let Err(e) = &standings_res {
            log::error!("Failed to send StandingsCommand::SetDate: {:?}", e);
        } else {
            // Reset standings state since new data is incoming
            self.standings.reset_state();
        }
    }
    fn try_update_year_from_input(&mut self) -> Result<(), ()> {
        let valid_year = self.date_state.validate_input_year(self.timezone)?;
        self.date_state.set_year_from_valid_input(valid_year);
        Ok(())
    }
    /// Update data from sources after year change
    pub fn handle_year_change(&mut self) {
        let year = self.date_state.year;
        let playoffs_res = self
            .playoff_bracket_tx
            .try_send(BracketCommand::SetYear(year));
        let team_stats_res = self.team_stats_tx.try_send(TeamStatsCommand::SetYear(year));

        if let Err(e) = &playoffs_res {
            log::error!("Failed to send BracketCommand::SetYear: {:?}", e);
        } else {
            // Clear old data and reset state
            self.playoff_bracket.playoff_bracket_data = None;
            self.playoff_bracket.reset_state();
        }
        if let Err(e) = &team_stats_res {
            log::error!("Failed to send TeamStatsCommand::SetYear: {:?}", e);
        } else {
            // Clear old data and reset state
            self.team_stats.team_stats_data = None;
            self.team_stats.reset_state();
        }
    }
    /// Update the selected team for team stats
    fn try_update_team_from_input(&mut self) -> Result<(), InputError> {
        let valid_team = self.team_stats.team_picker.validate_input()?;
        self.team_stats
            .team_picker
            .set_team_from_valid_input(valid_team);
        Ok(())
    }
    /// Update data from team stats sources after team change
    pub fn handle_team_change(&mut self) {
        let team = self.team_stats.team_picker.current_team;
        let res = self.team_stats_tx.try_send(TeamStatsCommand::SetTeam(team));

        if let Err(e) = &res {
            log::error!("Failed to send TeamStatsCommand::SetTeam: {:?}", e);
        } else {
            self.team_stats.reset_state();
        }
    }

    fn reset_app_state(&mut self) {
        self.games.reset_state();
        self.standings.reset_state();
        self.team_stats.reset_state();
        self.playoff_bracket.reset_state();
    }

    fn set_fetch_interval(&self, live: bool) {
        let (info_interval, games_interval) = if live {
            (
                FetchInterval::InfoShortInterval,
                FetchInterval::GamesShortInterval,
            )
        } else {
            (
                FetchInterval::InfoLongInterval,
                FetchInterval::GamesLongInterval,
            )
        };
        self.games_tx
            .try_send(GamesCommand::SetInterval(games_interval.as_duration()))
            .ok();
        self.boxscore_tx
            .try_send(BoxscoreCommand::SetInterval(info_interval.as_duration()))
            .ok();
        self.game_story_tx
            .try_send(GameStoryCommand::SetInterval(info_interval.as_duration()))
            .ok();
        self.standings_tx
            .try_send(StandingsCommand::SetInterval(info_interval.as_duration()))
            .ok();
        self.team_stats_tx
            .try_send(TeamStatsCommand::SetInterval(info_interval.as_duration()))
            .ok();
    }

    fn is_games_live(&self, parsed_games: &GamesResponse) -> bool {
        parsed_games
            .games
            .iter()
            .any(|g| matches!(g.game_state, GameState::LIVE | GameState::CRIT))
    }
}
