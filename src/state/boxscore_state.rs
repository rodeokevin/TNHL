use crate::models::boxscore::BoxscoreResponse;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct BoxscoreState {
    pub games: HashMap<u32, GameBoxscore>,
}

#[derive(Debug, Clone)]
pub struct GameBoxscore {
    pub game_id: u32,
    pub data: BoxscoreResponse,
}
