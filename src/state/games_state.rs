use crate::models::boxscore::BoxscoreResponse;
use crate::models::games::GamesResponse;
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

    pub games_data: Option<GamesResponse>,
    pub boxscore_data: HashMap<u32, BoxscoreResponse>,
    pub selected_game_index: usize,
    pub sweeping_status_offset: usize, // For the --- under the time remaining
    pub scoring_scroll_offset: usize,
    pub max_scoring_scroll: usize,
}
impl GamesState {
    pub fn shift_game_index(&mut self, forward: bool) {
        let prev = self.selected_game_index;
        if forward {
            let max_index = self.games_data.as_ref().map_or(0, |d| d.games.len());
            self.selected_game_index =
                next_index(self.selected_game_index, max_index);
        } else {
            self.selected_game_index = prev_index(self.selected_game_index);
        }
        if self.selected_game_index != prev {
            self.reset_scoring_scroll();
        }
    }
    pub fn reset_scoring_scroll(&mut self) {
        self.scoring_scroll_offset = 0;
        self.max_scoring_scroll = 0;
    }
    pub fn reset_selection_state(&mut self) {
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
}

fn next_index(index: usize, max_index: usize) -> usize {
    (index + 1).min(max_index.saturating_sub(1))
}
fn prev_index(index: usize) -> usize {
    index.saturating_sub(1)
}

impl Default for GamesState {
    fn default() -> Self {
        Self {
            focus: GamesFocus::default(),
            boxscore_selected_team: BoxscoreTeam::default(),
            boxscore_selected_position: BoxscorePosition::default(),

            games_data: None,
            boxscore_data: HashMap::new(),
            selected_game_index: 0,
            sweeping_status_offset: 0,
            scoring_scroll_offset: 0,
            max_scoring_scroll: 0,
        }
    }
}
