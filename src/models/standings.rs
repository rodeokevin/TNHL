use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StandingsResponse {
    pub standings: Vec<TeamData>,
}

#[derive(Debug, Deserialize)]
pub struct TeamData {
    #[serde(rename = "teamName")]
    pub team_name: TeamName,
    #[serde(rename = "teamAbbrev")]
    pub team_abbrev: TeamAbbrev,
    #[serde(rename = "conferenceAbbrev")]
    pub conference_abbrev: String,
    #[serde(rename = "divisionAbbrev")]
    pub division_abbrev: String,
    #[serde(rename = "conferenceSequence")]
    pub conference_sequence: u32,
    #[serde(rename = "wildcardSequence")]
    pub wildcard_sequence: u32,
    #[serde(rename = "divisionSequence")]
    pub division_sequence: u32,
    #[serde(rename = "leagueSequence")]
    pub league_sequence: u32,
    #[serde(rename = "gamesPlayed")]
    pub games_played: u32,
    pub wins: u32,
    pub losses: u32,
    #[serde(rename = "otLosses")]
    pub ot_losses: u32,
    pub points: u32,
    #[serde(rename = "pointPctg")]
    pub point_pctg: f64,
    #[serde(rename = "regulationWins")]
    pub regulation_wins: u32,
    #[serde(rename = "regulationPlusOtWins")]
    pub regulation_plus_ot_wins: u32,
    #[serde(rename = "goalFor")]
    pub goal_for: u32,
    #[serde(rename = "goalAgainst")]
    pub goal_against: u32,
    #[serde(rename = "homeWins")]
    pub home_wins: u32,
    #[serde(rename = "homeOtLosses")]
    pub home_ot_losses: u32,
    #[serde(rename = "homeLosses")]
    pub home_losses: u32,
    #[serde(rename = "roadWins")]
    pub road_wins: u32,
    #[serde(rename = "roadOtLosses")]
    pub road_ot_losses: u32,
    #[serde(rename = "roadLosses")]
    pub road_losses: u32,
    #[serde(rename = "shootoutWins")]
    pub shootout_wins: u32,
    #[serde(rename = "shootoutLosses")]
    pub shootout_losses: u32,
    #[serde(rename = "l10Wins")]
    pub l10_wins: u32,
    #[serde(rename = "l10OtLosses")]
    pub l10_ot_losses: u32,
    #[serde(rename = "l10Losses")]
    pub l10_losses: u32,
    #[serde(rename = "streakCode")]
    pub streak_code: String,
    #[serde(rename = "streakCount")]
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
