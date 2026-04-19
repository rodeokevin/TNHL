use serde::Deserialize;

use crate::models::{TeamAbbrev, TeamName};

#[derive(Debug, Deserialize, Default)]
pub struct BracketResponse {
    pub series: Vec<Series>,
}

impl BracketResponse {
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Round {
    pub round_number: u8,
    pub round_label: String,
    pub round_abbrev: String,
    pub series: Vec<Series>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub series_title: String,
    pub series_abbrev: String,
    pub series_letter: String,
    pub playoff_round: u8,
    pub top_seed_rank: u8,
    pub top_seed_rank_abbrev: String,
    pub top_seed_wins: u8,
    pub bottom_seed_rank: u8,
    pub bottom_seed_rank_abbrev: String,
    pub bottom_seed_wins: u8,
    pub top_seed_team: Option<SeriesTeam>,
    pub bottom_seed_team: Option<SeriesTeam>,
    pub winning_team_id: Option<u32>,
    pub losing_team_id: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesTeam {
    pub id: u32,
    pub abbrev: TeamAbbrev,
    pub name: TeamName,
    pub common_name: TeamName,
    pub wins: Option<u8>,
}
