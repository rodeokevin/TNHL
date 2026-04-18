use serde::Deserialize;

use crate::models::games::{Position, games::PlayerName};

#[derive(Debug, Deserialize)]
pub struct TeamStatsResponse {
    pub skaters: Vec<Skater>,
    pub goalies: Vec<Goalie>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skater {
    pub first_name: PlayerName,
    pub last_name: PlayerName,
    pub position_code: Position,
    pub games_played: u8,
    pub goals: u16,
    pub assists: u16,
    pub points: u16,
    pub plus_minus: Option<i16>,
    pub penalty_minutes: u16,
    pub power_play_goals: Option<u8>,
    pub shorthanded_goals: Option<u8>,
    pub game_winning_goals: u8,
    pub overtime_goals: u8,
    pub shots: Option<u16>,
    pub shooting_pctg: Option<f32>,
    pub avg_time_on_ice_per_game: Option<f32>, // in seconds
    pub avg_shifts_per_game: Option<f32>,
    pub faceoff_win_pctg: Option<f32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Goalie {
    pub first_name: PlayerName,
    pub last_name: PlayerName,
    pub games_played: u8,
    pub games_started: u8,
    pub wins: u8,
    pub losses: u8,
    pub ties: Option<u8>,
    pub overtime_losses: Option<u8>,
    pub goals_against_average: f32,
    pub save_percentage: Option<f32>,
    pub shots_against: Option<u32>,
    pub saves: Option<u32>,
    pub goals_against: u32,
    pub shutouts: u8,
    pub goals: u8,
    pub assists: u8,
    pub points: u8,
    pub penalty_minutes: u16,
    pub time_on_ice: u32,
}

impl TeamStatsResponse {
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}
