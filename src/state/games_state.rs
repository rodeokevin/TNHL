use crate::models::games::{
    boxscore::BoxscoreResponse, game_story::GameStoryReponse, games::GamesResponse,
};
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
    /// Updated during render
    pub visible_rows: usize,

    pub games_data: Option<GamesResponse>,
    pub boxscore_data: HashMap<u32, BoxscoreResponse>,
    pub game_story_data: HashMap<u32, GameStoryReponse>,
    pub selected_game_index: usize,
    pub sweeping_status_offset: usize, // For the dynamic display bar under the time remaining
    pub scroll_offset: usize,
    pub max_scroll: usize,
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
    pub fn move_boxscore_selection(&mut self, delta: i32) {
        let len = self.current_boxscore_len();
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
        if self.visible_rows == 0 {
            return;
        }

        self.scroll_offset = self.scroll_offset
            .saturating_sub(self.visible_rows);
    }
    /// Page down for scoring or stats page
    pub fn games_page_down(&mut self) {
        if self.visible_rows == 0 {
            return;
        }

        self.scroll_offset = (self.scroll_offset + self.visible_rows)
            .min(self.max_scroll);
    }
    /// Page up for boxscore
    pub fn boxscore_page_up(&mut self) {
        if self.visible_rows == 0 {
            return;
        }
        // The first visible row becomes the last visible row
        let offset = self.boxscore_table_state.offset();
        let new_offset = offset.saturating_sub(self.visible_rows - 1);
        *self.boxscore_table_state.offset_mut() = new_offset;
        self.boxscore_table_state.select(Some(new_offset));
    }
    /// Page down for boxscore
    pub fn boxscore_page_down(&mut self) {
        if self.visible_rows == 0 {
            return;
        }
        // The last visible row becomes the first visible row
        // But if last visible row is the last row in the table, simply select it without changing the offset
        let len = self.current_boxscore_len();
        let offset = self.boxscore_table_state.offset();
        let last_visible = if offset + self.visible_rows - 1 >= len - 1 {
            len - 1
        } else {
            *self.boxscore_table_state.offset_mut() = (offset + self.visible_rows - 1).min(len - 1);
            (offset + self.visible_rows - 1).min(len - 1)
        };
        self.boxscore_table_state.select(Some(last_visible));
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
