use crate::models::boxscore::BoxscoreResponse;

pub struct BoxscoreState {
    pub boxscore_data: Option<BoxscoreResponse>,
    // statistics totals (computed from boxscore_data)
    pub hits: u16,
    pub shots_on_goal: u16,
    pub penalty_minutes: u16,
    pub blocked_shots: u16,
    pub giveaways: u16,
    pub takeaways: u16,
}

impl BoxscoreState {
    pub fn compute_stats_totals() {
        todo!()
    }
}
impl Default for BoxscoreState {
    fn default() -> Self {
        Self {
            boxscore_data: None,
            hits: 0,
            shots_on_goal: 0,
            penalty_minutes: 0,
            blocked_shots: 0,
            giveaways: 0,
            takeaways: 0,
        }
    }
}
