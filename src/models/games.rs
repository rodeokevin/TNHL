use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GamesResponse {
    pub games: Vec<GameData>,
}

#[derive(Debug, Deserialize)]
pub struct GameData {
    pub id: u32,
    pub venue: Venue,
    #[serde(rename = "startTimeUTC")]
    pub start_time_utc: DateTime<Utc>,
    #[serde(rename = "gameState")]
    pub game_state: String,
    #[serde(rename = "awayTeam")]
    pub away_team: Team,
    #[serde(rename = "homeTeam")]
    pub home_team: Team,
}

#[derive(Debug, Deserialize)]
pub struct Venue {
    pub default: String,
}

#[derive(Deserialize, Debug)]
pub struct Team {
    pub id: u32,
    pub name: TeamName,
    pub abbrev: String,
    pub score: u32,
    pub sog: u32,
}

#[derive(Deserialize, Debug)]
pub struct TeamName {
    pub default: String,
}

impl GamesResponse {
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}
