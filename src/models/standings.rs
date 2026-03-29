use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StandingsResponse {
    pub standings: Vec<TeamData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamData {
    pub team_name: TeamName,
    pub team_abbrev: TeamAbbrev,
    pub clinch_indicator: Option<String>,
    pub conference_abbrev: String,
    pub division_abbrev: String,
    pub conference_sequence: u32,
    pub wildcard_sequence: u32,
    pub division_sequence: u32,
    pub league_sequence: u32,
    pub games_played: u32,
    pub wins: u32,
    pub losses: u32,
    pub ot_losses: u32,
    pub points: u32,
    pub point_pctg: f64,
    pub regulation_wins: u32,
    pub regulation_plus_ot_wins: u32,
    pub goal_for: u32,
    pub goal_against: u32,
    pub home_wins: u32,
    pub home_ot_losses: u32,
    pub home_losses: u32,
    pub road_wins: u32,
    pub road_ot_losses: u32,
    pub road_losses: u32,
    pub shootout_wins: u32,
    pub shootout_losses: u32,
    pub l10_wins: u32,
    pub l10_ot_losses: u32,
    pub l10_losses: u32,
    pub streak_code: String,
    pub streak_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct TeamName {
    pub default: String,
}

#[derive(Debug, Deserialize)]
pub struct TeamAbbrev {
    pub default: String,
}

impl StandingsResponse {
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}
