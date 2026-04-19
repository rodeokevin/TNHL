use crate::models::games::{
    boxscore::BoxscoreResponse, game_story::GameStoryReponse, games::GamesResponse,
};
use crate::state::app_state::{table_page_down, table_page_up};
use ratatui::widgets::TableState;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GamesFocus {
    #[default]
    Scoring,
    Stats,
    Boxscore,
}

impl GamesFocus {
    pub fn next(self) -> Self {
        match self {
            GamesFocus::Scoring => GamesFocus::Stats,
            GamesFocus::Stats => GamesFocus::Boxscore,
            GamesFocus::Boxscore => GamesFocus::Scoring,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            GamesFocus::Scoring => GamesFocus::Boxscore,
            GamesFocus::Stats => GamesFocus::Scoring,
            GamesFocus::Boxscore => GamesFocus::Stats,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BoxscoreTeam {
    #[default]
    Away,
    Home,
}

impl BoxscoreTeam {
    pub fn toggle(self) -> Self {
        match self {
            BoxscoreTeam::Away => BoxscoreTeam::Home,
            BoxscoreTeam::Home => BoxscoreTeam::Away,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BoxscorePosition {
    #[default]
    Forwards,
    Defensemen,
    Goalies,
}

pub struct GamesState {
    pub focus: GamesFocus,
    pub boxscore_selected_team: BoxscoreTeam,
    pub boxscore_selected_position: BoxscorePosition,
    pub boxscore_table_state: TableState,
    // Updated during render
    pub visible_rows: usize,

    pub games_data: Option<GamesResponse>,
    pub boxscore_data: HashMap<u32, BoxscoreResponse>,
    pub game_story_data: HashMap<u32, GameStoryReponse>,
    pub selected_game_index: usize,
    // For the dynamic display bar under the time remaining
    pub sweeping_status_offset: usize,
    pub scroll_offset: usize,
    /// Max vertical scroll updated at render
    pub max_scroll: usize,
}

impl Default for GamesState {
    fn default() -> Self {
        fn table() -> TableState {
            let mut t = TableState::default();
            t.select(Some(0));
            t
        }

        Self {
            focus: GamesFocus::default(),
            boxscore_selected_team: BoxscoreTeam::default(),
            boxscore_selected_position: BoxscorePosition::default(),
            boxscore_table_state: table(),
            visible_rows: 0,

            games_data: None,
            boxscore_data: HashMap::new(),
            game_story_data: HashMap::new(),
            selected_game_index: 0,
            sweeping_status_offset: 0,
            scroll_offset: 0,
            max_scroll: 0,
        }
    }
}

impl GamesState {
    /// Set the game index to next if forward == true, otherwise previous
    /// Index only changes if it is valid
    pub fn shift_game_index(&mut self, forward: bool) {
        if forward {
            let max_index = self.games_data.as_ref().map_or(0, |d| d.games.len());
            self.selected_game_index = next_index(self.selected_game_index, max_index);
        } else {
            self.selected_game_index = prev_index(self.selected_game_index);
        }
    }
    pub fn reset_scoring_scroll(&mut self) {
        self.scroll_offset = 0;
        self.max_scroll = 0;
    }
    /// Reset all state in games to default
    pub fn reset_state(&mut self) {
        self.reset_game_state();
        self.selected_game_index = 0;
    }
    /// Reset state when changing games
    pub fn reset_game_state(&mut self) {
        self.focus = GamesFocus::default();
        self.boxscore_selected_position = BoxscorePosition::default();
        self.boxscore_selected_team = BoxscoreTeam::default();
        self.boxscore_table_state.select(Some(0));
        self.reset_scoring_scroll();
    }
    /// Cycle between games display (Scoring, boxscore, stats, etc.)
    pub fn cycle_display(&mut self, forward: bool) {
        self.focus = if forward {
            self.focus.next()
        } else {
            self.focus.prev()
        };
    }
    /// Move rows in boxscore
    pub fn boxscore_row_up(&mut self) {
        self.boxscore_table_state.scroll_up_by(1);
    }
    pub fn boxscore_row_down(&mut self) {
        self.boxscore_table_state.scroll_down_by(1);
    }
    /// Get the number of rows of current boxscore
    fn current_boxscore_len(&self) -> usize {
        let boxscore = self
            .current_game_id()
            .and_then(|id| self.boxscore_data.get(&id));

        match boxscore {
            Some(b) => {
                let team = match b.player_by_game_stats.as_ref() {
                    Some(stats) => match self.boxscore_selected_team {
                        BoxscoreTeam::Away => &stats.away_team,
                        BoxscoreTeam::Home => &stats.home_team,
                    },
                    None => return 0,
                };
                match self.boxscore_selected_position {
                    BoxscorePosition::Forwards => team.forwards.len(),
                    BoxscorePosition::Defensemen => team.defense.len(),
                    BoxscorePosition::Goalies => team.goalies.len(),
                }
            }
            None => 0,
        }
    }
    /// Return the current game id
    pub fn current_game_id(&self) -> Option<u32> {
        self.games_data
            .as_ref()
            .and_then(|g| g.games.get(self.selected_game_index))
            .map(|g| g.id)
    }
    pub fn reset_boxscore_state(&mut self) {
        self.boxscore_selected_position = BoxscorePosition::default();
        self.boxscore_table_state.select(Some(0));
        self.boxscore_selected_team = BoxscoreTeam::default();
    }
    /// Page up for scoring or stats page
    pub fn games_page_up(&mut self) {
        if self.visible_rows != 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(self.visible_rows);
        }
    }
    /// Page down for scoring or stats page
    pub fn games_page_down(&mut self) {
        if self.visible_rows != 0 {
            self.scroll_offset = (self.scroll_offset + self.visible_rows).min(self.max_scroll);
        }
    }
    /// Page up for boxscore
    pub fn boxscore_page_up(&mut self) {
        table_page_up(self.visible_rows, &mut self.boxscore_table_state);
    }
    /// Page down for boxscore
    pub fn boxscore_page_down(&mut self) {
        table_page_down(
            self.visible_rows,
            self.current_boxscore_len(),
            &mut self.boxscore_table_state,
        );
    }
}

fn next_index(index: usize, max_index: usize) -> usize {
    (index + 1).min(max_index.saturating_sub(1))
}
fn prev_index(index: usize) -> usize {
    index.saturating_sub(1)
}
