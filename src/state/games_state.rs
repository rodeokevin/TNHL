use crate::models::boxscore::BoxscoreResponse;
use crate::models::games::GamesResponse;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct BoxscoreTotals {
    pub away: TeamStats,
    pub home: TeamStats,
}

#[derive(Debug, Clone, Default)]
pub struct TeamStats {
    pub hits: u16,
    pub shots_on_goal: u16,
    pub penalty_minutes: u16,
    pub blocked_shots: u16,
    pub giveaways: u16,
    pub takeaways: u16,
}

pub struct GamesState {
    pub games_data: Option<GamesResponse>,
    pub boxscore_data: HashMap<u32, BoxscoreResponse>,
    pub selected_game_index: usize,
    pub sweeping_status_offset: usize, // For the --- under the time remaining
    pub scoring_scroll_offset: usize,
    pub max_scoring_scroll: usize,
}

impl Default for GamesState {
    fn default() -> Self {
        Self {
            games_data: None,
            boxscore_data: HashMap::new(),
            selected_game_index: 0,
            sweeping_status_offset: 0,
            scoring_scroll_offset: 0,
            max_scoring_scroll: 0,
        }
    }
}
