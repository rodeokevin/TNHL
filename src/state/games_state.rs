use crate::models::{
    boxscore::BoxscoreResponse, game_story::GameStoryReponse, games::GamesResponse,
};
use ratatui::widgets::TableState;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GamesFocus {
    #[default]
    Overview,
    Boxscore,
}

impl GamesFocus {
    pub fn next(self) -> Self {
        match self {
            GamesFocus::Overview => GamesFocus::Boxscore,
            GamesFocus::Boxscore => GamesFocus::Overview,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            GamesFocus::Overview => GamesFocus::Boxscore,
            GamesFocus::Boxscore => GamesFocus::Overview,
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

    pub games_data: Option<GamesResponse>,
    pub boxscore_data: HashMap<u32, BoxscoreResponse>,
    pub game_story_data: HashMap<u32, GameStoryReponse>,
    pub selected_game_index: usize,
    pub sweeping_status_offset: usize, // For the dynamic display bar under the time remaining
    pub scoring_scroll_offset: usize,
    pub max_scoring_scroll: usize,
}

impl GamesState {
    pub fn shift_game_index(&mut self, forward: bool) {
        let prev = self.selected_game_index;
        if forward {
            let max_index = self.games_data.as_ref().map_or(0, |d| d.games.len());
            self.selected_game_index = next_index(self.selected_game_index, max_index);
        } else {
            self.selected_game_index = prev_index(self.selected_game_index);
        }
        if self.selected_game_index != prev {
            self.reset_scoring_scroll();
            self.boxscore_table_state.select(Some(0));
        }
    }
    pub fn reset_scoring_scroll(&mut self) {
        self.scoring_scroll_offset = 0;
        self.max_scoring_scroll = 0;
    }
    pub fn reset_selection_state(&mut self) {
        self.focus = GamesFocus::default();
        self.boxscore_selected_position = BoxscorePosition::default();
        self.boxscore_selected_team = BoxscoreTeam::default();
        self.selected_game_index = 0;
        self.reset_scoring_scroll();
    }

    // Cycle between games display (overview, boxscore, etc.)
    pub fn cycle_display(&mut self, next: bool) {
        self.focus = if next {
            self.focus.next()
        } else {
            self.focus.prev()
        };
    }

    // Move rows in boxscore
    pub fn move_boxscore_selection(&mut self, delta: i32) {
        let len = self.get_current_boxscore_len();
        let table = &mut self.boxscore_table_state;
        let current = table.selected().unwrap_or(0);

        let new = current as i32 + delta;
        let next = if new < 0 || new >= len as i32 {
            current
        } else {
            new as usize
        };

        table.select(Some(next));
    }

    fn get_current_boxscore_len(&self) -> usize {
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

    pub fn current_game_id(&self) -> Option<u32> {
        self.games_data
            .as_ref()
            .and_then(|g| g.games.get(self.selected_game_index))
            .map(|g| g.id)
    }
}

fn next_index(index: usize, max_index: usize) -> usize {
    (index + 1).min(max_index.saturating_sub(1))
}
fn prev_index(index: usize) -> usize {
    index.saturating_sub(1)
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

            games_data: None,
            boxscore_data: HashMap::new(),
            game_story_data: HashMap::new(),
            selected_game_index: 0,
            sweeping_status_offset: 0,
            scoring_scroll_offset: 0,
            max_scoring_scroll: 0,
        }
    }
}
