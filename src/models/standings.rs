use serde::Deserialize;

use crate::models::TeamName;

#[derive(Debug, Deserialize)]
pub struct StandingsResponse {
    pub standings: Vec<TeamData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamData {
    pub team_name: TeamName,
    pub team_abbrev: TeamAbbrev,
    pub season_id: u32,
    pub clinch_indicator: Option<String>,
    pub conference_abbrev: String,
    pub division_abbrev: String,
    pub conference_sequence: u8,
    pub wildcard_sequence: u8,
    pub division_sequence: u8,
    pub league_sequence: u8,
    pub games_played: u16,
    pub wins: u8,
    pub losses: u8,
    pub ot_losses: u8,
    pub points: u16,
    pub point_pctg: f64,
    pub regulation_wins: u8,
    pub regulation_plus_ot_wins: u8,
    pub goal_for: u16,
    pub goal_against: u16,
    pub home_wins: u8,
    pub home_ot_losses: u8,
    pub home_losses: u8,
    pub road_wins: u8,
    pub road_ot_losses: u8,
    pub road_losses: u8,
    pub shootout_wins: u8,
    pub shootout_losses: u8,
    pub l10_wins: u8,
    pub l10_ot_losses: u8,
    pub l10_losses: u8,
    pub streak_code: String,
    pub streak_count: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TeamAbbrev {
    pub default: String,
}

impl StandingsResponse {
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}

#[derive(Debug, Deserialize)]
pub struct StandingsSeasonResponse {
    pub seasons: Vec<SeasonBounds>,
}

/// The date range a season's standings are valid for.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeasonBounds {
    pub standings_start: String,
    pub standings_end: String,
}

impl StandingsSeasonResponse {
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}

impl SeasonBounds {
    /// Parsed start date, if valid.
    pub fn start(&self) -> Option<chrono::NaiveDate> {
        chrono::NaiveDate::parse_from_str(&self.standings_start, "%Y-%m-%d").ok()
    }
    /// Parsed end date, if valid.
    pub fn end(&self) -> Option<chrono::NaiveDate> {
        chrono::NaiveDate::parse_from_str(&self.standings_end, "%Y-%m-%d").ok()
    }
}
