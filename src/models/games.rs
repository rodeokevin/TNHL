use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fmt;

#[derive(Debug, Deserialize)]
pub struct GamesResponse {
    pub games: Vec<GameData>,
}

impl GamesResponse {
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
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
    pub period_descriptor: Option<PeriodDescriptor>, // If the game is not live, there is no PeriodDescriptor
    pub situation: Option<GameSituation>,
    pub goals: Option<Vec<GoalData>>,
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

#[derive(Debug, Deserialize, PartialEq)]
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
pub struct GameSituation {
    pub home_team: TeamSituation,
    pub away_team: TeamSituation,
    pub time_remaining: String,
    pub situation_code: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TeamSituation {
    pub abbrev: String, // Team name abbrev
    pub strength: u8,
    pub situation_descriptions: Option<Vec<SituationDesc>>,
}

#[derive(Debug, Deserialize)]
pub enum SituationDesc {
    PP,
    EN,
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GoalData {
    pub period_descriptor: PeriodDescriptor,
    pub time_in_period: String,
    pub player_id: u32,
    pub first_name: PlayerName,
    pub last_name: PlayerName,
    pub goal_modifier: GoalModifier,
    pub assists: Vec<AssistInfo>,
    pub team_abbrev: String,
    pub goals_to_date: Option<u32>,
    pub strength: GoalStrength,
}

#[derive(Deserialize, Debug)]
pub struct PlayerName {
    pub default: String,
}
impl fmt::Display for PlayerName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.default)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GoalModifier {
    None,
    PenaltyShot,
    EmptyNet,
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AssistInfo {
    pub player_id: u32,
    pub name: PlayerName,
    pub assists_to_date: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GoalStrength {
    EV,
    SH,
    PP,
    EmptyNet,
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameOutcome {
    pub last_period_type: PeriodType,
    pub ot_periods: Option<u32>,
}
