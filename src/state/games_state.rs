use crate::models::games::play_by_play::PlaysResponse;
use crate::models::games::{
    boxscore::BoxscoreResponse, game_story::GameStoryResponse, games::GameState,
    games::GamesResponse,
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
    Pregame,
}

impl GamesFocus {
    pub fn next(self) -> Self {
        match self {
            GamesFocus::Scoring => GamesFocus::Stats,
            GamesFocus::Stats => GamesFocus::Boxscore,
            GamesFocus::Boxscore => GamesFocus::Scoring,
            GamesFocus::Pregame => GamesFocus::Pregame,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            GamesFocus::Scoring => GamesFocus::Boxscore,
            GamesFocus::Stats => GamesFocus::Scoring,
            GamesFocus::Boxscore => GamesFocus::Stats,
            GamesFocus::Pregame => GamesFocus::Pregame,
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
    pub game_story_data: HashMap<u32, GameStoryResponse>,
    pub plays_data: HashMap<u32, PlaysResponse>,
    pub selected_game_index: usize,
    // For the dynamic display bar under the time remaining
    pub sweeping_status_offset: usize,
    pub scroll_offset: usize,
    /// Max vertical scroll updated at render
    pub max_scroll: usize,

    // Play-by-play side pane
    pub plays_visible: bool,
    pub plays_focused: bool,
    pub plays_table_state: TableState,
    pub plays_visible_rows: usize,
    pub plays_len: usize,
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
            plays_data: HashMap::new(),
            selected_game_index: 0,
            sweeping_status_offset: 0,
            scroll_offset: 0,
            max_scroll: 0,

            plays_visible: false,
            plays_focused: false,
            plays_table_state: table(),
            plays_visible_rows: 0,
            plays_len: 0,
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

    pub fn toggle_plays(&mut self) {
        self.plays_visible = !self.plays_visible;
        self.plays_focused = self.plays_visible;
        self.reset_plays_scroll();
    }

    pub fn toggle_plays_focus(&mut self) {
        if self.plays_visible {
            self.plays_focused = !self.plays_focused;
        }
    }

    pub fn reset_plays_scroll(&mut self) {
        self.plays_table_state.select(Some(0));
        *self.plays_table_state.offset_mut() = 0;
    }

    pub fn plays_scroll_up(&mut self) {
        self.plays_table_state.scroll_up_by(1);
    }
    pub fn plays_scroll_down(&mut self) {
        self.plays_table_state.scroll_down_by(1);
    }
    pub fn plays_page_up(&mut self) {
        table_page_up(self.plays_visible_rows, &mut self.plays_table_state);
    }
    pub fn plays_page_down(&mut self) {
        table_page_down(
            self.plays_visible_rows,
            self.plays_len,
            &mut self.plays_table_state,
        );
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
        self.reset_plays_scroll();
    }
    /// Cycle between games display (Scoring, boxscore, stats, etc.)
    pub fn cycle_display(&mut self, forward: bool) {
        // Pregame has no other tabs to cycle to.
        if self.focus == GamesFocus::Pregame {
            return;
        }
        self.focus = if forward {
            self.focus.next()
        } else {
            self.focus.prev()
        };
    }

    /// The `GameState` of the currently selected game, if any.
    pub fn current_game_state(&self) -> Option<GameState> {
        self.games_data
            .as_ref()
            .and_then(|d| d.games.get(self.selected_game_index))
            .map(|g| g.game_state)
    }

    /// Keep `focus` consistent with the selected game's state: pre-game games
    /// use the `Pregame` focus; once a game is live/final, fall back to a normal
    /// tab. Called each render so transitions are handled.
    pub fn sync_focus_to_game_state(&mut self) {
        let is_pregame = matches!(
            self.current_game_state(),
            Some(GameState::FUT | GameState::PRE)
        );
        if is_pregame {
            self.focus = GamesFocus::Pregame;
        } else if self.focus == GamesFocus::Pregame {
            self.focus = GamesFocus::default();
        }
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
    /// Page up for scoring or stats page. Keeps one row of overlap so the first
    /// visible row becomes the last visible row.
    pub fn games_page_up(&mut self) {
        if self.visible_rows != 0 {
            let page = self.visible_rows.saturating_sub(1).max(1);
            self.scroll_offset = self.scroll_offset.saturating_sub(page);
        }
    }
    /// Page down for scoring or stats page. Keeps one row of overlap so the last
    /// visible row becomes the first visible row.
    pub fn games_page_down(&mut self) {
        if self.visible_rows != 0 {
            let page = self.visible_rows.saturating_sub(1).max(1);
            self.scroll_offset = (self.scroll_offset + page).min(self.max_scroll);
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
