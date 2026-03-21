use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fmt;

#[derive(Debug, Deserialize)]
pub struct GamesResponse {
    pub games: Vec<GameData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameData {
    pub id: u32,
    pub venue: Venue,
    #[serde(rename = "startTimeUTC")]
    pub start_time_utc: DateTime<Utc>,
    pub game_state: GameState,
    pub away_team: Team,
    pub home_team: Team,
    pub period: Option<u32>,
    pub clock: Option<Clock>,
    pub period_descriptor: Option<PeriodDescriptor>,
    pub game_outcome: Option<GameOutcome>,
}

#[derive(Debug, Deserialize)]
pub struct Venue {
    pub default: String,
}

#[derive(Debug, Deserialize)]
pub enum GameState {
    FUT,
    PRE,
    LIVE,
    CRIT,
    OVER,
    FINAL,
    OFF,
    #[serde(other)]
    Unknown,
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameState::FUT => write!(f, "Future"),
            GameState::PRE => write!(f, "Pregame"),
            GameState::LIVE => write!(f, "Live"),
            GameState::FINAL => write!(f, "Final"),
            GameState::OFF => write!(f, "Off"),
            GameState::CRIT => write!(f, "Critical"),
            GameState::OVER => write!(f, "Over"),
            GameState::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum PeriodType {
    REG,
    OT,
    SO,
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Debug)]
pub struct Team {
    pub id: u32,
    pub name: TeamName,
    pub abbrev: String,
    pub score: Option<u32>,
    pub sog: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub struct TeamName {
    pub default: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Clock {
    pub time_remaining: String,
    pub seconds_remaining: u32,
    pub running: bool,
    pub in_intermission: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PeriodDescriptor {
    pub number: u32,
    pub period_type: PeriodType,
    pub ot_periods: Option<u32>,
    pub max_regulation_periods: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameOutcome {
    pub last_period_type: PeriodType,
    pub ot_periods: Option<u32>,
}

impl GamesResponse {
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}
