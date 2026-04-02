use crate::models::games::GamesResponse;

pub struct GamesState {
    pub games_data: Option<GamesResponse>,
    pub selected_game_index: usize,
    pub sweeping_status_offset: usize, // For the --- under the time remaining
    pub scoring_scroll_offset: usize,
    pub max_scoring_scroll: usize,
}

impl Default for GamesState {
    fn default() -> Self {
        Self {
            games_data: None,
            selected_game_index: 0,
            sweeping_status_offset: 0,
            scoring_scroll_offset: 0,
            max_scoring_scroll: 0,
        }
    }
}
